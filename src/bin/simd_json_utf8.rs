fn main() {
    // a json with some invalid utf-8
    let mut json = b"{\"invalid\": \"\x80\"}".to_vec();
    let error = simd_json::to_borrowed_value(&mut json).unwrap_err();
    println!("{}", error);
}
