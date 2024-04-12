#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(untagged)]
enum StartOffset {
    V0 {
        #[serde(rename = "start_offset")]
        last_seen_offset: i64,
    },
    V1 {
        #[serde(rename = "start_offset_v1")]
        start_offset: i64,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct KafkaSplit {
    topic: String,
    partition: i32,

    /// The offset pointing to the first message to read from the partition. Inclusive.
    #[serde(flatten)]
    start_offset: Option<StartOffset>,
    /// The offset pointing to the next position of the last message to read from the partition. Exclusive.
    stop_offset: Option<i64>,
}

fn main() {
    let split = KafkaSplit {
        topic: "topic".to_string(),
        partition: 0,
        start_offset: Some(StartOffset::V1 { start_offset: 0 }),
        stop_offset: Some(1),
    };

    let serialized = serde_json::to_string(&split).unwrap();
    println!("{}", serialized);

    let serialized = r#"{"topic":"topic","partition":0,"start_offset":0,"stop_offset":1}"#;
    let deserialized: KafkaSplit = serde_json::from_str(&serialized).unwrap();
    println!("{:?}", deserialized);
}
