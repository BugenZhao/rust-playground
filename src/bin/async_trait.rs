#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]

use futures::Future;

trait Trait {
    fn foo(&self) -> impl Future<Output = i32> + '_;
}

impl Trait for i32 {
    async fn foo(&self) -> i32 {
        *self
    }
}

impl Trait for i64 {
    async fn foo(&self) -> i32 {
        *self as _
    }
}

async fn work(t: &impl Trait) -> i32 {
    t.foo().await
}

#[tokio::main]
async fn main() {
    dbg!(work(&32i32).await);
    dbg!(work(&64i64).await);
}
