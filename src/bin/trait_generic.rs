trait Foo {
    type Bar;
    fn baz(self) -> Self::Bar;
}

impl<T> Foo for Result<T, std::io::Error> {
    type Bar = Result<T, std::io::Error>;

    fn baz(self) -> Self::Bar {
        self
    }
}

impl Foo for std::io::Error {
    type Bar = std::io::Error;

    fn baz(self) -> Self::Bar {
        self
    }
}

fn main() {}
