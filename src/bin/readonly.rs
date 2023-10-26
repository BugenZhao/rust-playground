mod inner {
    #[readonly::make]
    pub struct Foo {
        #[readonly]
        pub bar: String,
        pub fuz: i32,
    }

    impl Foo {
        pub fn new(bar: String, fuz: i32) -> Self {
            Self { bar, fuz }
        }
    }
}

fn main() {
    let foo = inner::Foo::new("bar".to_string(), 42);
}
