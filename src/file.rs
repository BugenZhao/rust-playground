use std::{os::unix::prelude::FileExt, rc::Rc};

use tokio::fs::File;

fn check_sync<T: Sync>() {}

async fn work() {
    let file = File::open("a.txt").await.unwrap();
}

fn work_sync() {
    use std::fs::File;

    let file = File::open("a.txt").unwrap();
    check_sync::<File>();
}

fn main() {}
