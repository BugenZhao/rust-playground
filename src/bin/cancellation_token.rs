#[tokio::main]
async fn main() {
    tokio_util::sync::CancellationToken::new()
        .cancelled_owned()
        .await;
}
