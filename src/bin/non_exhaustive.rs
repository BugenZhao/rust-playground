use child::A;

fn main() {
    let _a = A { a: 32 };
}

mod child {
    #[non_exhaustive] // only for other crate
    pub struct A {
        pub a: i32,
    }
}
