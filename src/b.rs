use std::time::Duration;

use tokio::{sync::mpsc, time::sleep, task::block_in_place};

async fn arm(times: usize) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    tx.send(1).unwrap();

    let item = rx.recv().await.unwrap();
    println!("{}: {}", times, item);

    // block_in_place(|| {
    //     std::thread::sleep(Duration::from_millis(120));
    // });
    sleep(Duration::from_millis(120)).await;

    // sleep(Duration::from_millis(120)).await;
    println!("{}: 2", times);
}

// #[tokio::main]
async fn main() {
    let slice = "233";
    println!("{}", slice[slice.len()..]);
}
