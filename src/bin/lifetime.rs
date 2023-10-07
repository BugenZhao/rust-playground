use std::ops::Deref;

enum Foo {
    A,
    B,
}

#[derive(Clone)]
struct Ref<'a, T: 'a>(&'a T);

impl<'a, T> Deref for Ref<'a, T> {
    type Target = &'a T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        println!("Dropping Ref");
    }
}

fn get_ref() -> Ref<'static, Foo> {
    Ref(&Foo::A)
}

fn main() {
    if matches!(*get_ref(), Foo::A) {
        println!("A");
    }
}
