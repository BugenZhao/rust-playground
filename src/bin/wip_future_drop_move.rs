use std::{marker::PhantomData, pin::Pin, task::Poll, time::Duration};

use futures::{
    future::{pending, poll_immediate},
    pin_mut, Future, FutureExt,
};
use pin_project::UnsafeUnpin;
use pin_project_lite::__private::AlwaysUnpin;
use tokio::{task::yield_now, task_local};

enum MyFut {
    Start,
    Next { array: [i32; 5], ptr: *const i32 },
}

impl Future for MyFut {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match *self {
            MyFut::Start => {
                self.set(MyFut::Next {
                    array: [114514, 2, 3, 4, 5],
                    ptr: &0,
                });
                Poll::Pending
            }
            MyFut::Next { .. } => unreachable!(),
        }
    }
}

async fn test() {
    struct Array([i32; 5]);

    impl Drop for Array {
        fn drop(&mut self) {
            println!("array dropped at {:p}", &self.0 as *const _);
        }
    }

    let mut array = Array([114514, 2, 3, 4, 5]);

    println!("array: {:p}", &array.0 as *const _);

    struct Guard<'a>(&'a mut i32);

    impl<'a> Guard<'a> {
        fn new(value: &'a mut i32) -> Self {
            println!("guard created at {:p}", value as *mut _);
            Self(value)
        }
    }

    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            let _ = FOO.get();

            println!("guard dropped at {:p}", self.0 as *mut _);
            *self.0 += 1;
            println!("value: {}", self.0)
        }
    }

    let _guard = Guard::new(&mut array.0[0]);

    yield_now().await;

    tokio::time::sleep(Duration::from_secs(100000)).await;

    drop(_guard);

    // yield_now().await;
    // pending().await
}

task_local! {
    static FOO: i32;
}

struct BadUnpin<T>(T);

impl<T> Unpin for BadUnpin<T> {}

impl<T: Future> Future for BadUnpin<T> {
    type Output = T::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }.poll(cx)
    }
}

async fn work() {
    let mut scoped_0 = FOO.scope(42, test()).fuse();

    // assert!(std::mem::size_of_val(&scoped) < 2048);

    // let h = tokio::spawn(scoped);

    // tokio::time::sleep(Duration::from_secs(1)).await;

    // h.abort();

    // pin_mut!(scoped);

    // let mut scoped_0 = BadUnpin(scoped).fuse();

    let mut scoped = unsafe { Pin::new_unchecked(&mut scoped_0) };

    futures::select! {
        _ = &mut scoped => {}
        _ = tokio::time::sleep(Duration::from_secs(1)).fuse() => {}
    }

    println!("selected");

    drop(scoped);
    drop(scoped_0);
}

fn move_to_drop<T>(t: T) {
    drop(t);
}

#[tokio::main]
async fn main() {
    work().await
}
