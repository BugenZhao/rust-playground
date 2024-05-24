// use linkme::distributed_slice;

// #[distributed_slice]
// pub static BENCHMARKS: [fn(&mut Bencher)];

#[cfg(not(bugen))]
fn foo() {
    extern "Rust" {
        static FOO: i32;
    }
    std::hint::black_box(unsafe { FOO });
}

#[cfg(bugen)]
fn foo() {
    println!("foo");
}

fn main() {
    foo();
}
