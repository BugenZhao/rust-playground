#![feature(exhaustive_patterns)]

struct Annotated<E> {
    extra: E,
    variant: Variant<E>,
}

enum Variant<E> {
    Number,
    String,
    List(Box<Variant<E>>),
    Struct(StructType<E>),
}

struct StructType<E> {
    fields: Box<[Annotated<E>]>,
}

struct WithFieldName {
    name: String,
}

struct WithIdName {
    id: i32,
    name: String,
}

// 1.
type DataType = Variant<WithFieldName>;
// 2.
type Field = Annotated<WithFieldName>;

// 3.
struct ColumnDesc {
    inner: Annotated<WithIdName>,

    // top level fields...
    default_expr: (),
}

fn main() {}
