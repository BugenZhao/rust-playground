use std::{borrow::Borrow, str::from_utf8_unchecked};

#[derive(Debug)]
enum ScalarRef<'a> {
    Int64(i64),
    String(&'a str),
}

#[derive(Debug, Clone)]
enum Scalar {
    Int64(i64),
    String(String),
}

// impl ToOwned for ScalarRef<'_> {
//     type Owned = Scalar;

//     fn to_owned(&self) -> Self::Owned {
//         match self {
//             ScalarRef::Int64(i) => Scalar::Int64(*i),
//             ScalarRef::String(s) => Scalar::String(s.to_owned()),
//         }
//     }
// }

enum DataType {
    Int64,
    String,
}

fn deserialize<'a>(mut buf: &'a [u8], schema: &[DataType]) -> Vec<ScalarRef<'a>> {
    let mut row = Vec::with_capacity(schema.len());
    for ty in schema {
        match ty {
            DataType::Int64 => {
                let (be, rest) = buf.split_at(8);
                let int = i64::from_be_bytes(be.try_into().unwrap());
                buf = rest;

                row.push(ScalarRef::Int64(int));
            }
            DataType::String => {
                let (be, rest) = buf.split_at(8);
                let len = u64::from_be_bytes(be.try_into().unwrap()) as usize;
                buf = rest;

                let (bytes, rest) = buf.split_at(len);
                let str = unsafe { from_utf8_unchecked(bytes) };
                buf = rest;

                row.push(ScalarRef::String(str))
            }
        }
    }
    row
}

#[test]
fn test_serialize() {
    let bytes = [
        &233_i64.to_be_bytes(),
        &114514_i64.to_be_bytes(),
        &12_u64.to_be_bytes(),
        "hello, world".as_bytes(),
        &1919810_i64.to_be_bytes(),
    ]
    .concat();

    let row = deserialize(
        &bytes,
        &[
            DataType::Int64,
            DataType::Int64,
            DataType::String,
            DataType::Int64,
        ],
    );

    println!("{:?}", row);
}
