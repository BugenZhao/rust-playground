use diff::Diff;

#[derive(Diff)]
struct Foo {
    interval_ms: u32,
    url: String,
}

fn main() {
    let a = Foo {
        interval_ms: 1000,
        url: "https://example.com".to_string(),
    };

    let b = Foo {
        interval_ms: 2145,
        url: "https://example.com".to_string(),
    };

    let diff = a.diff(&b);

    assert_eq!(diff.interval_ms, 1145); // confusing
    assert!(diff.url.is_none());
}
