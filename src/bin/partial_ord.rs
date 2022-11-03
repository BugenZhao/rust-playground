fn main() {
    #[derive(PartialEq, PartialOrd)]
    enum Foo {
        A(i32),
        B(i64),
    }

    dbg!(Foo::A(2).partial_cmp(&Foo::A(3)));
    dbg!(Foo::A(2).partial_cmp(&Foo::B(1)));
}
