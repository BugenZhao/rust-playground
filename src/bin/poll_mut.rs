use std::{marker::PhantomPinned, time::Duration};

use futures::Future;
use pin_project_lite::pin_project;
use tokio::task::futures::TaskLocalFuture;

pin_project! {
    struct MyFuture<Fut> {
        #[pin]
        inner: Fut,
        value: i32,
        _phantom: PhantomPinned, // no-op due to pin_project's safe API
    }
}

impl<Fut> MyFuture<Fut> {
    fn into_value(self) -> i32 {
        self.value
    }
}

impl<Fut: Future> Future for MyFuture<Fut> {
    type Output = Fut::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        *this.value += 1;
        this.inner.poll(cx)
    }
}

fn check_unpin<T: Unpin>(_: &T) {}

async fn work() {
    let fut = async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        42
    };
    let mut my_fut = MyFuture {
        // inner: fut,
        inner: Box::pin(fut), // <- !
        value: 0,
        _phantom: PhantomPinned,
    };

    check_unpin(&my_fut); // still unpin

    let ret = (&mut my_fut).await;
    println!("ret: {}", ret);

    let my_fut_2 = my_fut;

    let value = my_fut_2.into_value();
    println!("value: {}", value);
}

#[tokio::main]
async fn main() {
    work().await
}
