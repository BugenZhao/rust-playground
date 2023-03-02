use std::time::Duration;

use async_backtrace::frame;
use futures::channel::oneshot::{self, Receiver};
use futures::future::{pending, select};
use futures::FutureExt;
use tokio::time::sleep;

#[async_backtrace::framed]
async fn work(rx: Receiver<()>) {
    let mut fut = frame!(pending::<()>()).boxed();

    // poll `fut` under the `select` span
    let _ = frame!(select(
        frame!(sleep(Duration::from_millis(500))).boxed(),
        &mut fut
    ))
    .await;

    // `select` span closed so `fut` is detached
    // the elapsed time of `fut` should be preserved

    // wait for the signal to continue
    frame!(rx).await.unwrap();

    // poll `fut` under the root `work` span, and it'll be remounted
    fut.await
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(work(rx));

    sleep(Duration::from_millis(100)).await;
    let tree = async_backtrace::taskdump_tree(true);

    // work [106.290ms]
    //   select [106.093ms]
    //     sleep [106.093ms]
    //     fut [106.093ms]
    println!("{tree}");

    sleep(Duration::from_secs(1)).await;
    let tree = async_backtrace::taskdump_tree(true);

    // work [1.112s]
    //   rx [606.944ms]
    // [Detached 4]
    //   fut [1.112s]
    println!("{tree}");

    tx.send(()).unwrap();
    sleep(Duration::from_secs(1)).await;
    let tree = async_backtrace::taskdump_tree(true);

    // work [2.117s]
    //   fut [2.117s]
    println!("{tree}");
}
