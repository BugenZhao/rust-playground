use simd_json::ValueAccess;

fn main() {
    let s = r#"{"v2": "{\"a\": 233, \"b\": true}"}"#;
    let mut v = s.as_bytes().to_vec();
    let v = simd_json::to_borrowed_value(&mut v[..]).unwrap();
    println!("{}", v);

    let inner = v.get("v2").unwrap().as_str().unwrap();
    println!("{}", inner);

    let mut v = inner.as_bytes().to_vec();
    let v = simd_json::to_borrowed_value(&mut v[..]).unwrap();
    println!("{}", v);
}
