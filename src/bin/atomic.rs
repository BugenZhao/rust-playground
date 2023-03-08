use std::sync::atomic::{AtomicI32, Ordering};

fn main() {
    let a = AtomicI32::new(0);
    #[allow(invalid_atomic_ordering)]
    a.load(Ordering::Release);
}
