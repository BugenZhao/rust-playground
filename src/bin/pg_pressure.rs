#[tokio::main]
async fn main() {
    let futs = (0..512).map(|_| async {
        let (client, conn) = tokio_postgres::connect(
            "host=localhost user=root dbname=dev port=4566",
            tokio_postgres::NoTls,
        )
        .await
        .unwrap();

        tokio::spawn(conn);

        client.simple_query("SELECT my_sleep();").await.unwrap();
    });

    let r = futures::future::join_all(futs).await;
    dbg!(r);
}
