// https://github.com/rust-lang/rust/issues/117432

#![feature(error_generic_member_access)]

#[derive(Debug)]
struct Foo;

#[derive(Debug)]
struct MyError {
    foo: Foo,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyError")
    }
}

impl std::error::Error for MyError {
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        request.provide_ref::<Foo>(&Foo);
    }
}

fn foo_provided<T: std::error::Error>(e: &T) -> bool {
    std::error::request_ref::<Foo>(e).is_some()
}

fn main() {
    let e = MyError { foo: Foo };

    assert!(foo_provided::<MyError>(&e)); // ok
    assert!(foo_provided::<&MyError>(&&e)); // ok
    assert!(foo_provided::<Box<MyError>>(&Box::new(e))); // fails
}
