use futures::Future;
use maybe_async::maybe_async;

#[maybe_async(AFIT)]
trait MyTrait {
    fn info(&self) -> String;

    async fn fetch(&self);

    fn fetch_2(&self) -> impl Future<Output = ()> + Send;
}

struct MyStruct;

// #[maybe_async::sync_impl]
// impl MyTrait for MyStruct {}

fn main() {}
