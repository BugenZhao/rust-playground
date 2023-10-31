#![feature(extract_if)]

fn main() {
    let mut v = vec![1, 2, 3, 4];
    let mut a = v.extract_if(|x| *x % 2 == 0);
    dbg!(a.next().unwrap());
    drop(a);
    dbg!(v);
}
