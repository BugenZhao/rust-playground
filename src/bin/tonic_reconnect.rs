#[tokio::main]
async fn main() {
    tonic::transport::Endpoint::from_static("http://[::1]:50051")
        .connect()
        .await
        .expect("Failed to connect to server");
}
