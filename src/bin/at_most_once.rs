#![feature(trait_alias)]

use std::time::Duration;

use futures::{
    stream::{once, AbortHandle, Abortable},
    Stream, StreamExt,
};

fn at_most_once(pred: bool) -> impl Stream<Item = i32> {
    once(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if pred {
            Some(233)
        } else {
            None
        }
    })
    .map(futures::stream::iter)
    .flatten()
}

#[tokio::main]
async fn main() {
    let a: Vec<_> = at_most_once(true).collect().await;
    dbg!(a);
    let b: Vec<_> = at_most_once(false).collect().await;
    dbg!(b);
}
