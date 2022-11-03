#![allow(dead_code)]

struct Wrapper<T: ?Sized> {
    count: usize,
    inner: T,
}

// struct Wrapper2<S: ?Sized, T: ?Sized> {
//     count: usize,
//     inner: T,
//     inner2: S,
// }

fn size_of_ref<T: ?Sized>() -> usize {
    std::mem::size_of::<&T>()
}

fn main() {
    dbg!(size_of_ref::<u8>());
    dbg!(size_of_ref::<[u8]>());
    dbg!(size_of_ref::<Wrapper<u8>>());
    dbg!(size_of_ref::<Wrapper<[u8]>>());
    dbg!(size_of_ref::<Wrapper<str>>());
}
