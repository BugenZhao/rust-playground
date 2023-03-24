use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let sem = Semaphore::new(1);
    sem.acquire_many(1).await.unwrap().forget();
    sem.acquire_many(0).await.unwrap().forget();
}
