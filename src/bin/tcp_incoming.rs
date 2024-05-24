#[tokio::main]
async fn main() {
    let socket1 =
        tonic::transport::server::TcpIncoming::new("127.0.0.1:8080".parse().unwrap(), true, None)
            .unwrap();
    let socket2 =
        tonic::transport::server::TcpIncoming::new("127.0.0.1:8080".parse().unwrap(), true, None)
            .unwrap_err();
}
