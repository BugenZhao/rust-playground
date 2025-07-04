use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Feature {
    A,
    B,
    C,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum FeatureMaybeUnknown {
    Known(Feature),
    Unknown(String),
}

impl From<Feature> for FeatureMaybeUnknown {
    fn from(feature: Feature) -> Self {
        FeatureMaybeUnknown::Known(feature)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum Tier {
    Free,
    Paid,


    // #[serde(untagged)]
    // Adhoc2(Vec<FeatureMaybeUnknown>),

    #[serde(untagged)]
    Adhoc {
        name: String,
        features: Vec<Feature>,
    },

}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct License {
    name: String,
    tier: Tier,
}

fn print(license: License) {
    let json = serde_json::to_string(&license).unwrap();
    println!("{}", json);

    let deserialized = serde_json::from_str(&json).unwrap();
    assert_eq!(license, deserialized);
}

fn main() {
    print(License {
        name: "Rust".to_string(),
        tier: Tier::Paid,
    });

    print(License {
        name: "Rust".to_string(),
        tier: Tier::Adhoc {
            name: "shit".to_owned(),
            features: vec![Feature::A, Feature::B],
        },
    });

    // print(License {
    //     name: "Rust".to_string(),
    //     tier: Tier::Adhoc2(vec![
    //         Feature::A.into(),
    //         Feature::B.into(),
    //         FeatureMaybeUnknown::Unknown("Unknown".to_string()),
    //     ]),
    // });

    let json = serde_json::json!({
        "name": "Rust",
        "tier": ["A", "Unknown"],
    });

    let deserialized: License = serde_json::from_value(json).unwrap();
    println!("{:?}", deserialized);
}
