#![feature(exhaustive_patterns)]

struct Outer<E> {
    extra: E,
    variant: Inner<E>,
}

enum Inner<E> {
    Number,
    String,
    List(Box<Inner<E>>),
    Struct(StructType<E>),
}

struct StructType<E> {
    fields: Box<[Outer<E>]>,
}

struct WithFieldName {
    name: String,
}

struct WithIdName {
    id: i32,
    name: String,
}

// 1.
type DataType = Inner<WithFieldName>;
// 2.
type Field = Outer<WithFieldName>;

// 3.
struct ColumnDesc {
    inner: Outer<WithIdName>,

    // top level fields...
    default_expr: (),
}

fn main() {}
