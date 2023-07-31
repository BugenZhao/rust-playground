use std::hint::black_box;

enum ScalarImpl {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

enum ArrayImpl {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

impl ArrayImpl {
    #[inline(never)]
    fn push_1(&mut self, scalar: ScalarImpl) {
        match self {
            ArrayImpl::I8(v) => match scalar {
                ScalarImpl::I8(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::I16(v) => match scalar {
                ScalarImpl::I16(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::I32(v) => match scalar {
                ScalarImpl::I32(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::I64(v) => match scalar {
                ScalarImpl::I64(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::U8(v) => match scalar {
                ScalarImpl::U8(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::U16(v) => match scalar {
                ScalarImpl::U16(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::U32(v) => match scalar {
                ScalarImpl::U32(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::U64(v) => match scalar {
                ScalarImpl::U64(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::F32(v) => match scalar {
                ScalarImpl::F32(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
            ArrayImpl::F64(v) => match scalar {
                ScalarImpl::F64(i) => v.push(i),
                _ => panic!("Mismatched types"),
            },
        }
    }

    #[inline(never)]
    fn push_2(&mut self, scalar: ScalarImpl) {
        match (self, scalar) {
            (ArrayImpl::I8(v), ScalarImpl::I8(i)) => v.push(i),
            (ArrayImpl::I16(v), ScalarImpl::I16(i)) => v.push(i),
            (ArrayImpl::I32(v), ScalarImpl::I32(i)) => v.push(i),
            (ArrayImpl::I64(v), ScalarImpl::I64(i)) => v.push(i),
            (ArrayImpl::U8(v), ScalarImpl::U8(i)) => v.push(i),
            (ArrayImpl::U16(v), ScalarImpl::U16(i)) => v.push(i),
            (ArrayImpl::U32(v), ScalarImpl::U32(i)) => v.push(i),
            (ArrayImpl::U64(v), ScalarImpl::U64(i)) => v.push(i),
            (ArrayImpl::F32(v), ScalarImpl::F32(i)) => v.push(i),
            (ArrayImpl::F64(v), ScalarImpl::F64(i)) => v.push(i),
            _ => panic!("Mismatched types"),
        }
    }
}

fn main() {
    let array = || black_box(ArrayImpl::I32(vec![]));
    let scalar = || black_box(ScalarImpl::I32(1));

    black_box(array().push_1(scalar()));
    black_box(array().push_2(scalar()));
}
