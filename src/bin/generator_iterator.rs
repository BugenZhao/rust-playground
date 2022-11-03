#![feature(generators)]

use yield_iter::generator;

struct MyArray(Vec<u8>);

impl MyArray {
    fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        generator! {
            for &x in &self.0 {
                yield x;
            }
        }
    }
}

fn main() {
    for i in MyArray(vec![1, 2, 3]).iter() {
        println!("{}", i);
    }
}
