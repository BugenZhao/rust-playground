trait T {
    fn abs(self) -> Self;
}

impl T for i64 {
    fn abs(self) -> Self {
        2 * self
    }
}

fn main() {
    let x = 42;
    let a = || x.abs(); // 84
    let b = || x.abs(); // 42

    println!("{}", b());
    println!("{}", a());
}
