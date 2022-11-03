#![feature(drain_filter)]

fn main() {
    let mut v = vec![1, 2, 3, 4];
    let mut a = v.drain_filter(|x| *x % 2 == 0);
    dbg!(a.next().unwrap());
    drop(a);
    dbg!(v);
}
