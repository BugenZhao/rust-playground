#![allow(dead_code)]
#![feature(test)]

extern crate test;

use std::mem::size_of;

#[derive(Clone)]
struct MyStr {
    inner: triomphe::Arc<String>,
}

impl From<&'static str> for MyStr {
    fn from(s: &'static str) -> Self {
        let string = unsafe { String::from_raw_parts(s.as_ptr() as *mut _, s.len(), 0) };
        let inner = triomphe::Arc::new(string);
        std::mem::forget(inner.clone());
        Self { inner }
    }
}

// impl From<&'static str> for MyStr {
//     fn from(s: &'static str) -> Self {
//         let string = unsafe { String::from_raw_parts(s.as_ptr() as *mut _, s.len(), 0) };
//         let inner = triomphe::Arc::new(string);
//         let raw = triomphe::Arc::into_raw(inner);

//         let inner = unsafe {
//             let ref_count =
//                 (raw as *const u8).offset(-(std::mem::size_of::<usize>() as isize)) as *mut usize;
//             *ref_count = 2;
//             triomphe::Arc::from_raw(raw)
//         };

//         Self { inner }
//     }
// }

// impl From<&'static str> for MyStr {
//     fn from(s: &'static str) -> Self {
//         #[repr(C)]
//         struct MyRaw {
//             count: usize,
//             string: String,
//         }
//         let string = unsafe { String::from_raw_parts(s.as_ptr() as *mut _, s.len(), 0) };
//         let raw = Box::into_raw(Box::new(MyRaw { count: 2, string }));
//         let inner = unsafe {
//             let ptr =
//                 (raw as *const u8).offset(std::mem::size_of::<usize>() as isize) as *mut String;
//             triomphe::Arc::from_raw(ptr)
//         };

//         Self { inner }
//     }
// }

impl From<String> for MyStr {
    fn from(s: String) -> Self {
        let inner = triomphe::Arc::new(s);
        Self { inner }
    }
}

impl MyStr {
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

fn main() {
    dbg!(size_of::<triomphe::Arc<String>>());
    dbg!(size_of::<triomphe::Arc<str>>());
    dbg!(size_of::<std::sync::Arc<str>>());
    dbg!(size_of::<MyStr>());
}

#[cfg(test)]
mod benches {
    use std::iter::repeat_with;

    use super::*;
    use itertools::Itertools;
    use test::{black_box, Bencher};

    const STATIC_STR: &str = "Hello";
    const STATIC_LONG_STR: &str = "HelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHelloHello";

    #[bench]
    fn bench_my_str_from_static_str(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = MyStr::from(STATIC_STR);
                black_box(s);
            }
        });
    }

    #[bench]
    fn bench_arc_str_from_static_str(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = std::sync::Arc::<str>::from(STATIC_STR);
                black_box(s);
            }
        })
    }

    #[bench]
    fn bench_my_str_from_static_str_long(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = MyStr::from(STATIC_LONG_STR);
                black_box(s);
            }
        });
    }

    #[bench]
    fn bench_arc_str_from_static_str_long(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = std::sync::Arc::<str>::from(STATIC_LONG_STR);
                black_box(s);
            }
        })
    }

    #[bench]
    fn bench_my_str_from_string_long(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = MyStr::from(STATIC_LONG_STR.to_string());
                black_box(s);
            }
        });
    }

    #[bench]
    fn bench_arc_str_from_string_long(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..100000 {
                let s = std::sync::Arc::<str>::from(STATIC_LONG_STR.to_string());
                black_box(s);
            }
        });
    }

    #[bench]
    fn bench_my_str_from_static_str_ref(b: &mut Bencher) {
        b.iter(|| {
            let s = MyStr::from(STATIC_STR);
            for _ in 0..100000 {
                black_box(s.as_str());
            }
        });
    }

    #[bench]
    fn bench_arc_str_from_static_str_ref(b: &mut Bencher) {
        b.iter(|| {
            let s = std::sync::Arc::<str>::from(STATIC_STR);
            for _ in 0..100000 {
                black_box(s.as_ref());
            }
        })
    }
}
