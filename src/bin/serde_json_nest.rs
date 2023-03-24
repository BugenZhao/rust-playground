use std::collections::HashMap;

use serde_json::json;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Foo {
    field: serde_json::Value,
}

fn main() {
    let field = json!({
        "k1": "v1",
        "k2": "v2",
    });

    let foo = json!({
        "field": field.to_string(),
    });

    let foo2: Foo = serde_json::from_str(&foo.to_string()).unwrap();
    println!("{:?}", foo2);
}
