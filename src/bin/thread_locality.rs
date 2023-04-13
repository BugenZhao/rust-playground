use std::time::Duration;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        tokio::spawn(work()).await.unwrap();
    });
}

async fn work() {
    println!("current thread 1: {:?}", std::thread::current().id());

    tokio::spawn(async {
        println!("current thread 2: {:?}", std::thread::current().id());
        tokio::time::sleep(Duration::from_secs(1)).await;
    })
    .await
    .unwrap();
}
