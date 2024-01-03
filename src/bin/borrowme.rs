#[borrowme::borrowme]
#[derive(Clone)]
struct ScalarRef<'a> {
    my_int: i64,
    my_str: &'a str,
}

fn main() {}
