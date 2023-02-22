mod t {
    use crate::a::A;

    mod private {
        pub trait Super {}
    }

    pub trait Base: private::Super {}

    impl private::Super for A {}
}

mod a {
    use crate::t::Base;

    pub struct A;

    impl Base for A {}
}

mod b {
    // use crate::t::private::Super;
    use crate::t::Base;

    pub struct B;

    // impl Super for B {}
    // impl Base for B {}
}

fn main() {}
