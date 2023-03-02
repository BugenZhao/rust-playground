use futures::future::{join, pending};

#[async_backtrace::framed]
async fn my_pending() {
    let () = pending().await;
}

#[async_backtrace::framed]
async fn work_2() {
    let _ = tokio::spawn(my_pending()).await;
}

#[async_backtrace::framed]
async fn work() {
    let _ = join(work_2(), work_2()).await;
}

#[tokio::main]
async fn main() {
    tokio::spawn(work());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("{}", async_backtrace::taskdump_tree(false));
}
