#![feature(generators)]

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;

use futures::Future;
use futures::FutureExt;
use futures::StreamExt;
use tokio::{sync::mpsc, time::sleep};

pub struct PollOnce<F: Future + Unpin> {
    future: F,
}

impl<F: Future + Unpin> Future for PollOnce<F> {
    type Output = Poll<F::Output>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.future.poll_unpin(cx))
    }
}

#[futures_async_stream::stream(item = i32)]
async fn stream() {
    let mut worker = Box::pin(worker());

    loop {
        match worker.next().await {
            Some(item) => yield item,
            None => break,
        }
    }
}

#[tokio::main]
async fn main() {
    let mut stream = Box::pin(stream());
    while let Some(item) = stream.next().await {
        println!("item: {}", item);
    }
}

#[futures_async_stream::stream(item = i32)]
async fn worker() {
    for i in 0..10 {
        sleep(Duration::from_millis(500)).await;
        yield i;
    }
}
