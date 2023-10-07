use std::time::Duration;

use futures::FutureExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(1)
        .build()
        .unwrap();

    rt.block_on(work().boxed());
}

async fn spawn_work() {
    tokio::spawn(work()).await.unwrap();
}

async fn work() {
    let l = TcpListener::bind("0.0.0.0:10188").await.unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    loop {
        let mut r = l.accept().await.unwrap();
        println!("connected to {}", r.1);
        r.0.set_nodelay(true).unwrap();

        rt.spawn(async move {
            let len = r.0.read_i32().await.unwrap();
            let protocol_num = r.0.read_i32().await.unwrap();
            println!("len: {}, protocol_num: {}", len, protocol_num);
            infinite_loop().await;
            r.0.write_all(b"hello, pg").await.unwrap();
        });
    }
}

async fn infinite_loop() {
    let f = || loop {
        std::hint::spin_loop();
    };
    tokio::task::block_in_place(f);
}
