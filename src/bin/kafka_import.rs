use futures::future::try_join_all;
use futures::StreamExt;
use indicatif::ProgressIterator;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Define the Kafka topic and broker location
    let topic = "my-topic";
    let broker = "localhost:29092";

    // Define the number of dummy records to generate
    let num_records_per_partition = 100_000_000;
    let partition = 4;

    // Create a Kafka producer configuration
    let mut producer_config = ClientConfig::new();
    producer_config.set("bootstrap.servers", broker);

    // Create a Kafka producer
    let producer: FutureProducer = producer_config.create().unwrap();

    let mut handles = Vec::new();

    for p in 0..partition {
        let producer = producer.clone();

        let futs = futures::stream::iter(
            (0..num_records_per_partition)
                .map(move |i| i * partition + p)
                .map(move |i| {
                    let record = json!({"id": i, "name": format!("record-{}", i)});

                    let key = i.to_string();
                    let payload = record.to_string();
                    let producer = producer.clone();

                    async move {
                        let future_record = FutureRecord::to(topic)
                            .partition(p)
                            .key(&key)
                            .payload(&payload);
                        producer
                            .send(future_record, Duration::from_secs(0))
                            .await
                            .unwrap()
                    }
                }),
        );

        let handle = tokio::spawn(async move {
            let mut i = 0;
            futs.buffered(20000)
                .inspect(|_| {
                    i += 1;
                    if i % 50000 == 0 {
                        println!("sent {i}");
                    }
                })
                .count()
                .await
        });

        handles.push(handle);
    }

    try_join_all(handles).await.unwrap();

    // // Generate and send dummy JSON records to Kafka
    // for i in (0..num_records).progress() {
    //     let record = json!({"id": i, "name": format!("record-{}", i)});
    //     let partition = i % 4; // assign the record to a partition based on its ID

    //     let key = i.to_string();
    //     let payload = record.to_string();

    //     let future_record = FutureRecord::to(topic)
    //         .partition(partition)
    //         .key(&key)
    //         .payload(&payload);
    //     producer
    //         .send(future_record, Duration::from_secs(0))
    //         .await
    //         .unwrap();
    // }

    producer.flush(Duration::from_secs(10)).unwrap();
}
