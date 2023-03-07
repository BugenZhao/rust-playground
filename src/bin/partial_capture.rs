use std::ops::AddAssign;

#[derive(Debug)]
struct Bomb {
    a: i32,
    b: i32,
    c: Vec<i32>,
}

impl Drop for Bomb {
    fn drop(&mut self) {
        println!("Bomb! {:?}", self);
    }
}

fn main() {
    let mut bomb = Bomb {
        a: 1,
        b: 2,
        c: vec![3, 4, 5],
    };

    let mut f = move || {
        bomb.a.add_assign(100); // copied, no effect on `bomb`
        bomb.b.add_assign(200); // copied, no effect on `bomb`
    };
    f();
    println!("done f");

    let mut g = move || {
        bomb.a.add_assign(300); // moved due to next line, `bomb` is modified
        bomb.c.push(600); // moved, `bomb` is modified
    };
    g();

    println!("done g");

    // cannot use `bomb` anymore: borrow of moved value: `bomb`
    // println!("{bomb:?}")

    // drop here (?!)
    // Bomb! Bomb { a: 301, b: 2, c: [3, 4, 5, 600] }
}
