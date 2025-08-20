use rkyv::{
    munge::munge, rancor::Error, seal::Seal, vec::ArchivedVec, Archive, ArchiveUnsized,
    Deserialize, Serialize,
};

#[derive(Clone, Debug, Serialize, Deserialize, Archive)]
enum Datum {
    String(String),
    Bytes(Vec<u8>),
}

impl Datum {
    fn to_ref(&self) -> DatumRef {
        match self {
            Datum::String(s) => DatumRef::String(s.as_str()),
            Datum::Bytes(a) => DatumRef::Bytes(a.as_slice()),
        }
    }
}

impl ArchivedDatum {
    fn to_ref(&self) -> DatumRef {
        match self {
            ArchivedDatum::String(s) => DatumRef::String(s.as_str()),
            ArchivedDatum::Bytes(a) => DatumRef::Bytes(a.as_slice()),
        }
    }
}

#[derive(Clone, Debug)]
enum DatumRef<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
}

#[auto_impl::auto_impl(&)]
trait Row {
    /// Randomly access the datum ref in `index`.
    fn at(&self, index: usize) -> DatumRef;

    /// Length.
    fn len(&self) -> usize;
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive)]
pub struct VecDatum(Vec<Datum>);

impl Row for VecDatum {
    fn at(&self, index: usize) -> DatumRef {
        self.0[index].to_ref()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Row for ArchivedVecDatum {
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

fn main() {
    let row = VecDatum(vec![
        Datum::String("hello".to_owned()),
        Datum::Bytes(b"world".to_vec()),
    ]);

    let srow = rkyv::to_bytes::<Error>(&row).unwrap();

    // print!("{:x?}", srow);
    let drow = unsafe { rkyv::access_unchecked::<ArchivedVecDatum>(&srow) };

    println!("{:?}", (&drow).at(0));
    println!("{:?}", (&drow).at(1));
}
