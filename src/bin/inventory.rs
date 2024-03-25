struct Foo;

impl Foo {
    fn new() -> Self {
        Self::const_new()
    }

    const fn const_new() -> Self {
        Self
    }
}

struct FooBuilder(fn() -> Foo);

inventory::collect!(FooBuilder);

inventory::submit! {
    FooBuilder(Foo::new)
}

fn main() {}
