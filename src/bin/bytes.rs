use bytes::Bytes;

fn main() {
    let bytes: &[u8] = &[
        1, 14, 0, 0, 0, 1, 0, 0, 21, 0, 23, 93, 52, 220, 92, 56, 243, 243, 236, 98, 121, 55,
    ];

    println!("{:?}", Bytes::from_static(bytes));
}
