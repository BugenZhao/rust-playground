struct Zst;

fn main() {
    let a = Box::leak(Box::new(Zst));
    let b = Box::leak(Box::new(Zst));

    println!("{:p}", a);
    println!("{:p}", b);
    assert_eq!(a as *const Zst, b as *const Zst);
}
