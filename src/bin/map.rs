#![feature(try_blocks)]

use std::collections::{hash_map::Entry, HashMap};

fn main() {
    let v = vec![Ok(233), Err(456)];
    for i in v {}
}
