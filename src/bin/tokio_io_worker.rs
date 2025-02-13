use futures::future::pending;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

async fn bad_io() {
    let mut stream = tokio::net::TcpStream::connect("localhost:8888").await.unwrap();
    stream.read(&mut [0; 1]).await.unwrap();
}

async fn good_io() {
    let mut stream = tokio::net::TcpStream::connect("localhost:9999").await.unwrap();
    stream.write_all(b"hello").await.unwrap();
}

async fn work() {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move {
            println!("doing bad io...");
            bad_io().await;
            println!("start pending...");
            pending::<()>().await;
        })
    });
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .build()
        .unwrap();

    let handle = rt.spawn(work());

    std::thread::sleep(std::time::Duration::from_secs(3));

    rt.spawn(async {
        for i in 0.. {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            good_io().await;
            println!("still alive [{i}]");
        }
    });

    let _ = rt.block_on(handle);
}
