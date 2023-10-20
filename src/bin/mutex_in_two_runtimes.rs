use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

fn main() {
    let mutex = Arc::new(Mutex::new(0));

    let rt1 = tokio::runtime::Runtime::new().unwrap();
    let rt2 = tokio::runtime::Runtime::new().unwrap();

    let mutex_clone1 = Arc::clone(&mutex);
    rt1.spawn(async move {
        loop {
            let mut data = mutex_clone1.lock().await;
            *data += 1;
            println!("Runtime 1 incremented the value to: {}", *data);
        }
    });

    let mutex_clone2 = Arc::clone(&mutex);
    rt2.spawn(async move {
        loop {
            let mut data = mutex_clone2.lock().await;
            *data += 1;
            println!("Runtime 2 incremented the value to: {}", *data);
        }
    });

    // Let the spawned tasks run for a while.
    std::thread::sleep(Duration::from_secs(5));
}
