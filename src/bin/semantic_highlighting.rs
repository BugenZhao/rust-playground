#![feature(custom_inner_attributes)]
#![allow(unused_assignments)]
#![allow(non_upper_case_globals)]
#![rustfmt::skip]

use std::ops::AddAssign;

fn main() {}

pub fn foo(
    a: i32,     // Argument, immutable
    mut b: i32, // Argument, mutable
) -> i32 {
    b += 1;

    let mut x = a + b; // Local variable, mutable
    let y = a - b;     // Local variable, immutable

    x.add_assign(114);     // Method, taking `&mut self`
    x += 514;              // Method `add_assign`, taking `&mut self`
    y.abs()                // Method, taking `&self` or `self`
}

const CamelCase: i32 = 0;

fn ident() {
    use std::i64::MIN; // Constant
    use CamelCase;     // ...even if we use the camel case

    use std::sync::atomic::AtomicI64;         // Type
    use std::sync::atomic::Ordering::Relaxed; // Enum variant

    use usize;           // Primitive type
    use libc::uintptr_t; // Type
    use futures::stream; // Module
    use futures::join;   // Function
}
