trait Const: Default {}

macro_rules! def_const_type {
    ($ty:ident) => {
        paste::paste! {
            #[derive(Default)]
            struct [<Const $ty:upper>]<const VALUE: $ty>;
            impl<const VALUE: $ty> Const for [<Const $ty:upper>]<VALUE> {}
        }
    };
}

def_const_type!(i16);
def_const_type!(i32);
def_const_type!(i64);

fn constant<T: Const>() -> T {
    T::default()
}

pub struct Foo {
    int: ConstI32<42>,
}

fn main() {
    let foo = Foo { int: constant() };
}
