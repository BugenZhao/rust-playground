fn main() {
    let v = vec!["233".to_owned()].into_boxed_slice();

    for a in v.into_vec()
    // impl<T> [T]
    {
        println!("{}", a);
    }
}
