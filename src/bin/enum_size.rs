enum Foo {
    Large(Box<String>),
    Small(u16),
}

enum Bar {
    A,
    B,
    C,
}

struct FooBar {
    foo: Foo,
    bar: Bar,
}

fn main() {}
