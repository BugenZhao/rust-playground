use std::{marker::PhantomPinned, pin::Pin};

use futures::{future::FusedFuture, Future, FutureExt};

#[derive(Default)]
struct MyFuture {
    _p: PhantomPinned,
}

impl Future for MyFuture {
    type Output = i32;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

impl FusedFuture for MyFuture {
    fn is_terminated(&self) -> bool {
        todo!()
    }
}

fn test_unpin<T: Unpin>(_: &T) {}

async fn foo() {
    let fut = MyFuture::default();

    let mut pinned_fut = std::pin::pin!(fut);
    futures::select! {
        _ = &mut pinned_fut => {}
        _ = tokio::time::sleep(std::time::Duration::from_secs(1)).fuse() => {}
    }

    let _ret = pinned_fut.await;
}

fn foo_1() {
    // let mut fut = MyFuture::default();

    // let pinned_fut = std::pin::pin!(fut);
    // drop(pinned_fut);

    // let _moved_fut = std::mem::take(&mut fut);
}

fn foo_2() {
    let mut fut = MyFuture::default();

    let pinned_fut = unsafe { Pin::new_unchecked(&mut fut) };
    drop(pinned_fut);

    let _moved_fut = std::mem::take(&mut fut);
}

#[tokio::main]
async fn main() {
    let mut f = MyFuture::default();

    // let mut pinned_f = std::pin::pin!(f);

    let mut pinned_f = Box::pin(f);

    // let mut pinned_f = Box::new(f);

    (&mut pinned_f).await;

    // Pin::
}

struct SelfReferencial {
    array: [i32; 10],
    elem_ref: *const i32,
}

impl SelfReferencial {
    fn new() -> Self {
        let mut this = Self {
            array: [0; 10],
            elem_ref: std::ptr::null(),
        };
        this.elem_ref = &this.array[0] as *const i32;
        this
    }
}
