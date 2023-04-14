use std::hash::Hash;

fn main() {
    let a = 0x12345678u32;
    let mut hasher = crc32fast::Hasher::new();
    a.hash(&mut hasher);
    println!("{:X}", hasher.finalize());
}
