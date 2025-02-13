// use zerocopy::{FromBytes, TryFromBytes};

// #[derive(TryFromBytes)]
// struct IntArray {
//     data: Box<[i32]>,
// }

// struct ListArray {
//     offsets: Vec<u32>,
//     value: Box<Array>,
// }

// #[derive(TryFromBytes)]
// enum Array {
//     Int(Box<IntArray>),
//     List(ListArray),
// }

// struct ListValue {
//     value: ListArray,
// }

// struct ListRef<'a> {
//     array: &'a Array,
//     start: u32,
//     end: u32,
// }

fn main() {}
