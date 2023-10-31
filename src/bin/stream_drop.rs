#![feature(coroutines)]

use std::sync::atomic::{AtomicBool, Ordering};

use futures::{pin_mut, StreamExt};
use futures_async_stream::stream;

struct Resource;

static DROPPED: AtomicBool = AtomicBool::new(false);

impl Drop for Resource {
    fn drop(&mut self) {
        println!("drop resource");
        DROPPED.store(true, Ordering::SeqCst);
    }
}

#[stream(item = i32)]
async fn my_stream() {
    let _resource = Resource;
    yield 1;
    yield 2;
}

#[tokio::main]
async fn main() {
    println!("start");
    let st = my_stream();
    pin_mut!(st);

    st.next().await.unwrap();
    println!("next once");

    st.next().await.unwrap();
    println!("next twice");

    assert!(!DROPPED.load(Ordering::SeqCst));
    assert!(st.next().await.is_none());
    assert!(DROPPED.load(Ordering::SeqCst));
    println!("next None");

    drop(st);
    println!("done");
}
