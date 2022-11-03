use std::fmt::Debug;
use std::sync::Arc;

use tokio::sync::{AcquireError, Mutex, Semaphore};

struct Inner<T> {
    put: Semaphore,
    get: Semaphore,
    noti: Semaphore,
    data: Mutex<Option<T>>,
}

impl<T> Debug for Inner<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct UnbufferedSender<T>(Arc<Inner<T>>);

#[derive(Debug)]
pub struct UnbufferedReceiver<T>(Arc<Inner<T>>);

pub fn unbuffered_channel<T>() -> (UnbufferedSender<T>, UnbufferedReceiver<T>) {
    let inner = Arc::new(Inner {
        put: Semaphore::new(1),
        get: Semaphore::new(0),
        noti: Semaphore::new(0),
        data: Mutex::new(None),
    });

    let tx = UnbufferedSender(inner.clone());
    let rx = UnbufferedReceiver(inner);
    (tx, rx)
}

impl<T> Inner<T> {
    async fn put(&self, item: T) -> Result<(), AcquireError> {
        self.put.acquire().await?.forget();
        *self.data.lock().await = Some(item);
        self.get.add_permits(1);
        self.noti.acquire().await?.forget();
        Ok(())
    }

    async fn get(&self) -> Result<T, AcquireError> {
        self.get.acquire().await?.forget();
        let data = self.data.lock().await.take().unwrap();
        self.noti.add_permits(1);
        self.put.add_permits(1);
        Ok(data)
    }
}

impl<T> UnbufferedSender<T> {
    pub async fn send(&self, item: T) -> Result<(), AcquireError> {
        self.0.put(item).await
    }
}

impl<T> UnbufferedReceiver<T> {
    pub async fn recv(&self) -> Result<T, AcquireError> {
        self.0.get().await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_unbuffered_channel() {
        let (tx, rx) = unbuffered_channel();
        let handle1 = {
            let chan = tx.clone();
            tokio::spawn(async move {
                println!("before put 233");
                chan.send(233).await.unwrap();
                println!("after put 233, before put 234");
                chan.send(234).await.unwrap();
                println!("after put 234");
            })
        };

        let handle2 = {
            let chan = tx.clone();
            tokio::spawn(async move {
                println!("before put 100");
                chan.send(100).await.unwrap();
                println!("after put 100, before put 200");
                chan.send(200).await.unwrap();
                println!("after put 200");
            })
        };

        let handle3 = {
            let chan = tx.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                println!("awake");

                println!("before put 888");
                chan.send(888).await.unwrap();
                println!("after put 888");
            })
        };

        sleep(Duration::from_millis(250)).await;

        let v1 = rx.recv().await.unwrap();
        println!("get {}", v1);
        let v2 = rx.recv().await.unwrap();
        println!("get {}", v2);
        let v3 = rx.recv().await.unwrap();
        println!("get {}", v3);
        let v4 = rx.recv().await.unwrap();
        println!("get {}", v4);

        let vs = [v1, v2, v3, v4];
        let pos_of = |target: i32| vs.iter().position(|v| *v == target).unwrap();
        assert!(pos_of(233) < pos_of(234));
        assert!(pos_of(100) < pos_of(200));

        handle1.await.unwrap();
        handle2.await.unwrap();

        let v = rx.recv().await.unwrap();
        println!("get {}", v);
        assert_eq!(v, 888);

        handle3.await.unwrap();
    }
}
