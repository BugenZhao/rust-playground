#![feature(allocator_api)]

use std::{
    alloc::{Allocator, Global},
    str::Utf8Error,
};

type Datum<A = Global> = Option<Vec<u8, A>>;

fn from_utf8<A: Allocator>(v: Vec<u8, A>) -> Result<Box<str, A>, Utf8Error> {
    let b = v.into_boxed_slice();
    std::str::from_utf8(&b)?;
    let (ptr, alloc) = Box::into_raw_with_allocator(b);
    let s = unsafe { Box::from_raw_in(ptr as *mut str, alloc) };
    Ok(s)
}

fn main() {
    let _datum: Datum = Some(vec![1]);
}
