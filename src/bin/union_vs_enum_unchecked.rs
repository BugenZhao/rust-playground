use std::hint::black_box;

enum DataType {
    I32,
    F64,
}

union UScalarInner {
    i32: i32,
    f64: f64,
}
struct UScalar {
    ty: DataType,
    inner: UScalarInner,
}
impl UScalar {
    fn into_i32(self) -> i32 {
        match self.ty {
            DataType::I32 => unsafe { self.inner.i32 },
            DataType::F64 => panic!(),
        }
    }

    unsafe fn into_i32_unchecked(self) -> i32 {
        self.inner.i32
    }
}

enum EScalar {
    I32(i32),
    F64(f64),
}
impl EScalar {
    fn into_i32(self) -> i32 {
        match self {
            EScalar::I32(i) => i,
            EScalar::F64(_) => panic!(),
        }
    }

    unsafe fn into_i32_unchecked(self) -> i32 {
        match self {
            EScalar::I32(i) => i,
            _ => std::hint::unreachable_unchecked(),
        }
    }
}

#[inline(never)]
fn u_unchecked(s: UScalar) -> i32 {
    unsafe { s.into_i32_unchecked() }
}
#[inline(never)]
fn u(s: UScalar) -> i32 {
    s.into_i32()
}

#[inline(never)]
fn e_unchecked(s: EScalar) -> i32 {
    unsafe { s.into_i32_unchecked() }
}
#[inline(never)]
fn e(s: EScalar) -> i32 {
    s.into_i32()
}

pub fn main() {
    let u_arg = || UScalar {
        ty: DataType::I32,
        inner: UScalarInner { i32: 233 },
    };

    let e_arg = || EScalar::I32(233);

    black_box(u(black_box(u_arg())));
    black_box(e(black_box(e_arg())));
    black_box(u_unchecked(black_box(u_arg())));
    black_box(e_unchecked(black_box(e_arg())));
}
