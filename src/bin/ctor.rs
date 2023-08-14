use std::sync::atomic::{AtomicBool, Ordering};

use ctor::ctor;

static INITED: AtomicBool = AtomicBool::new(false);

#[ctor]
fn _23foo_233_233_233_ddd() {
    INITED.store(true, Ordering::SeqCst);
}

fn main() {}
