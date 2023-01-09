use std::io::Cursor;

trait Buf {
    fn get_i16(&mut self) -> i16;
    fn get_i32(&mut self) -> i32;
}

impl<T: Buf> Buf for &mut T {
    fn get_i16(&mut self) -> i16 {
        (&mut **self).get_i16()
    }

    fn get_i32(&mut self) -> i32 {
        (&mut **self).get_i32()
    }
}

enum DataType {
    Int16,
    Int32,
    Struct(Vec<DataType>),
}

enum Scalar {
    Int16(i16),
    Int32(i32),
    Struct(Vec<Scalar>),
}

fn de(t: DataType, mut buf: impl Buf) -> Scalar {
    match t {
        DataType::Int16 => Scalar::Int16(buf.get_i16()),
        DataType::Int32 => Scalar::Int32(buf.get_i32()),
        DataType::Struct(ts) => Scalar::Struct(ts.into_iter().map(|t| de(t, &mut buf)).collect()),
    }
}

impl Buf for &'_ [u8] {
    fn get_i16(&mut self) -> i16 {
        let (left, right) = self.split_at(2);
        let i = i16::from_be_bytes(left.try_into().unwrap());
        *self = right;
        i
    }

    fn get_i32(&mut self) -> i32 {
        let (left, right) = self.split_at(4);
        let i = i32::from_be_bytes(left.try_into().unwrap());
        *self = right;
        i
    }
}

/// Foos a bar.
///
/// # Example
///
/// ```compile_fail
/// de(
///     DataType::Struct(vec![DataType::Int16, DataType::Int16]),
///     &[0x12_u8, 0x34, 0x56, 0x78][..],
/// );
/// ```
fn main() {}
