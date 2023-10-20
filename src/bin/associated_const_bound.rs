#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

trait Satisfied {}

#[derive(Clone, Copy)]
enum MyEnum {
    A,
    B,
}

impl MyEnum {
    const fn is_a(self) -> bool {
        matches!(self, MyEnum::A)
    }
}

struct Cond<const PRED: bool>;

impl Satisfied for Cond<true> {}

trait Foo {
    const FOO: MyEnum;
}

struct TestFoo;

impl Foo for TestFoo {
    const FOO: MyEnum = MyEnum::A;
}

fn foo<T: Foo>()
where
    Cond<{ <T as Foo>::FOO.is_a() }>: Satisfied,
{
}

fn main() {
    foo::<TestFoo>();
}
