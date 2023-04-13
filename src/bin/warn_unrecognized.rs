use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

struct WarnUnrecognized<T> {
    inner: HashMap<String, Value>,
    _marker: std::marker::PhantomData<*const T>,
}

impl<T> Default for WarnUnrecognized<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<'de, T> Deserialize<'de> for WarnUnrecognized<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = HashMap::deserialize(deserializer)?;
        if !inner.is_empty() {
            eprintln!(
                "Unrecognized fields in `{}`: {:?}",
                std::any::type_name::<T>(),
                inner.keys()
            );
        }
        Ok(WarnUnrecognized {
            inner,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<T> Serialize for WarnUnrecognized<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    pub url: String,

    #[serde(default)]
    pub nested: NestedConfig,

    #[serde(flatten, default)]
    pub unrecognized: WarnUnrecognized<Config>,
}

#[derive(Default, Serialize, Deserialize)]
struct NestedConfig {
    #[serde(default)]
    pub user: String,

    #[serde(flatten, default)]
    pub unrecognized: WarnUnrecognized<NestedConfig>,
}

fn main() {
    let v1 = json!({
        "url": "example.com",
        "unrecognized": "field",
        "nested": {
            "user": "bugen",
            "unrecognized2": "field2",
        }
    });

    let v2 = json!({
        "url": "example.com",
    });

    let _: Config = serde_json::from_value(v1).unwrap();
    let _: Config = serde_json::from_value(v2).unwrap();
}
