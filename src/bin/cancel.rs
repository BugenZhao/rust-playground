use std::time::Duration;

use futures::future::select;

struct Guard {
    msg: String,
    enabled: bool,
}

impl Guard {
    fn new(msg: String) -> Self {
        println!("{} start", msg);
        Self { msg, enabled: true }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.enabled {
            panic!("{} cancelled", self.msg);
        } else {
            println!("{} ok", self.msg);
        }
    }
}

async fn abc() {
    let mut _guard = Guard::new("233".to_owned());
    let _: () = futures::future::pending().await;
    _guard.enabled = false;
}

async fn worker() {
    let mut f1 = Box::pin(abc());
    let mut f2 = Box::pin(tokio::time::sleep(Duration::from_millis(500)));
    select(&mut f1, &mut f2).await;

    let _ = Box::leak(Box::new(f1));
}

#[tokio::main]
async fn main() {
    worker().await;
}
