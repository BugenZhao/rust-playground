#![feature(backtrace)]

use std::{backtrace::Backtrace, panic::Location};

#[track_caller]
fn foo() {
    bar()
}

#[track_caller]
#[inline(always)]
fn bar() {
    let trace = Backtrace::capture();
    println!("trace: {trace}");
    panic!("we panic")
}

#[track_caller]
fn start_timer() {
    let caller = Location::caller();
    println!("{}:{} start timer", caller.file(), caller.line());
}

fn main() {
    start_timer();
}
