use std::sync::Arc;

use rkyv::{
    munge::munge,
    rancor::Error,
    seal::Seal,
    string::ArchivedString,
    vec::ArchivedVec,
    with::{AsString, AsVec, Inline, InlineAsBox},
    Archive, Serialize,
};

use crate::mmm::ArchivedRow;

#[derive(Clone, Debug)]
struct SpanString {
    data: Arc<str>,
    start: usize,
    end: usize,
}

impl SpanString {
    fn as_str(&self) -> &str {
        &self.data[self.start..self.end]
    }
}

// mod span_string_archive_with {
//     use rkyv::{
//         rancor::{Fallible, Source},
//         string::{ArchivedString, StringResolver},
//         with::{ArchiveWith, AsString, SerializeWith},
//         SerializeUnsized,
//     };

//     use crate::ArchivedDatum;

//     use super::SpanString;

//     impl ArchiveWith<SpanString> for AsString {
//         type Archived = ArchivedString;

//         type Resolver = StringResolver;

//         fn resolve_with(
//             field: &SpanString,
//             resolver: Self::Resolver,
//             out: rkyv::Place<Self::Archived>,
//         ) {
//             ArchivedString::resolve_from_str(field.as_str(), resolver, out);
//         }
//     }

//     impl<S> SerializeWith<SpanString, S> for AsString
//     where
//         str: SerializeUnsized<S>,
//         S: Fallible + ?Sized,
//         S::Error: Source,
//     {
//         fn serialize_with(
//             field: &SpanString,
//             serializer: &mut S,
//         ) -> Result<Self::Resolver, <S as Fallible>::Error> {
//             ArchivedString::serialize_from_str(field.as_str(), serializer)
//         }
//     }
// }

#[derive(Clone, Debug)]
enum Datum {
    // SpanString(#[rkyv(with = AsString)] SpanString),
    String(String),
    Bytes(Vec<u8>),
}

impl Datum {
    fn to_ref(&self) -> DatumRef {
        match self {
            // Datum::SpanString(s) => DatumRef::String(s.as_str()),
            Datum::String(s) => DatumRef::String(s.as_str()),
            Datum::Bytes(a) => DatumRef::Bytes(a.as_slice()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Archive)]
enum DatumRef<'a> {
    String(#[rkyv(with = AsString)] &'a str),
    Bytes(#[rkyv(with = AsVec)] &'a [u8]),
}

type ArchivedDatum = ArchivedDatumRef<'static>;

impl ArchivedDatumRef<'_> {
    fn to_ref(&self) -> DatumRef<'_> {
        match self {
            ArchivedDatumRef::String(s) => DatumRef::String(s.as_str()),
            ArchivedDatumRef::Bytes(b) => DatumRef::Bytes(&*b),
        }
    }
}

#[auto_impl::auto_impl(&)]
trait Row {
    /// Randomly access the datum ref in `index`.
    fn at(&self, index: usize) -> DatumRef;

    /// Length.
    fn len(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct VecDatum(Vec<Datum>);

impl Row for VecDatum {
    fn at(&self, index: usize) -> DatumRef {
        self.0[index].to_ref()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct VecDatumRef<'a>(Vec<DatumRef<'a>>);

impl<'a> Row for VecDatumRef<'a> {
    fn at(&self, index: usize) -> DatumRef {
        self.0[index].clone()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

mod mmm {
    use rkyv::{
        rancor::{Fallible, Source},
        vec::VecResolver,
        with::{ArchiveWith, SerializeWith},
        SerializeUnsized,
    };

    use super::*;

    pub(crate) struct MyWith;

    impl<R: Row> ArchiveWith<R> for MyWith {
        type Archived = ArchivedVec<ArchivedDatum>;

        type Resolver = VecResolver;

        fn resolve_with(row: &R, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
            ArchivedVec::resolve_from_len(row.len(), resolver, out);
        }
    }

    impl<R: Row, S> SerializeWith<R, S> for MyWith
    where
        S: Fallible + ?Sized + rkyv::ser::Allocator + rkyv::ser::Writer,
        S::Error: Source,
    {
        fn serialize_with(
            row: &R,
            serializer: &mut S,
        ) -> Result<Self::Resolver, <S as Fallible>::Error> {
            ArchivedVec::serialize_from_iter((0..row.len()).map(|i| row.at(i)), serializer)
        }
    }

    // TODO: new type
    pub type ArchivedRow = ArchivedVec<ArchivedDatum>;

    impl Row for ArchivedRow {
        fn at(&self, index: usize) -> DatumRef {
            self[index].to_ref()
        }

        fn len(&self) -> usize {
            self.len()
        }
    }
}

#[derive(Clone, Debug, Serialize, Archive)]
struct RowHelper<R: Row>(#[rkyv(with = mmm::MyWith)] R);

fn main() {
    let row = VecDatum(vec![
        Datum::String("hello".to_owned()),
        Datum::Bytes(b"world".to_vec()),
    ]);

    let srow = rkyv::to_bytes::<Error>(&RowHelper(&row)).unwrap();

    // print!("{:x?}", srow);
    let drow = rkyv::access::<ArchivedRow, Error>(&srow).unwrap_or_else(|e| panic!("{e}"));

    println!("{:?}", (&drow).at(0));
    println!("{:?}", (&drow).at(1));
}
