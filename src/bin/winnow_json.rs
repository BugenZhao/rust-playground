#![feature(trait_alias)]

use std::{collections::BTreeMap, error::Error};

use expect_test::{expect, Expect};
use winnow::{
    combinator::{
        alt, cut_err, delimited, preceded, repeat, separated, separated_pair, terminated,
    },
    error::StrContext,
    stream::{Compare, StreamIsPartial},
    token::{any, none_of, take_while},
    PResult, Parser,
};

type Array = Vec<Value>;
type Object = BTreeMap<String, Value>;

#[derive(Debug)]
enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
}

trait Stream: winnow::stream::Stream<Token = char> + StreamIsPartial + Compare<char> {}
impl<S> Stream for S where S: winnow::stream::Stream<Token = char> + StreamIsPartial + Compare<char> {}

fn char(input: &mut impl Stream) -> PResult<char> {
    alt((
        preceded(
            '\\',
            // must cut error
            cut_err(any.verify_map(|c| {
                Some(match c {
                    '"' | '\\' | '/' => c,
                    'n' => '\n',
                    _ => return None,
                })
            })),
        ),
        none_of('"'),
    ))
    .parse_next(input)
}

fn string(input: &mut impl Stream) -> PResult<String> {
    // i.e. delimited, but manually add `cut_err` once we encounter "
    // cut_err:
    //   - could affect correctness under `alt`
    //   - while in most cases, only affect error reporting
    preceded(
        '"',
        cut_err(terminated(
            repeat(0.., char).fold(String::new, |mut string, c| {
                string.push(c);
                string
            }),
            '"',
        )),
    )
    .context(StrContext::Label("string"))
    .parse_next(input)
}

fn ws(input: &mut impl Stream) -> PResult<()> {
    take_while(0.., |c| " \t\r\n".contains(c))
        .void()
        .parse_next(input)
}

fn key_value(input: &mut impl Stream) -> PResult<(String, Value)> {
    separated_pair(
        string.context(StrContext::Label("key")),
        cut_err((ws, ':', ws)),
        cut_err(value),
    )
    .parse_next(input)
}

fn object(input: &mut impl Stream) -> PResult<Object> {
    delimited('{', separated(0.., delimited(ws, key_value, ws), ','), '}').parse_next(input)
}

fn value(input: &mut impl Stream) -> PResult<Value> {
    alt((string.map(Value::String), object.map(Value::Object)))
        .context(StrContext::Label("value"))
        .parse_next(input)
}

fn doc(input: &mut impl Stream) -> PResult<Value> {
    delimited(ws, value, ws).parse_next(input)
}

fn parse(input: &str) -> Result<Value, Box<dyn Error>> {
    doc.parse(input).map_err(|e| e.to_string().into())
}

fn do_test(input: &str, expect: Expect) {
    expect.assert_eq(&match parse(input) {
        Ok(value) => format!("{:?}", value),
        Err(e) => format!("ERROR: {e}"),
    });
}

#[test]
fn test() {
    do_test(
        r#"
        "hello"
        "#,
        expect![[r#"String("hello")"#]],
    );

    do_test(
        r#"
        { "abc": "def" }
        "#,
        expect![[r#"Object({"abc": String("def")})"#]],
    );

    do_test(
        r#"
        { "abc": "def", "foo": { "hello": "world" } }
        "#,
        expect![[r#"Object({"abc": String("def"), "foo": Object({"hello": String("world")})})"#]],
    );

    do_test(
        r#"
        { "abc": shit }
        "#,
        expect![[r#"
            ERROR: parse error at line 2, column 18
              |
            2 |         { "abc": shit }
              |                  ^
            invalid value"#]],
    );

    do_test(
        r#"
        { "abc"": shit }
        "#,
        expect![[r#"
            ERROR: parse error at line 2, column 16
              |
            2 |         { "abc"": shit }
              |                ^
            invalid value"#]],
    );

    do_test(
        r#"
        { abc }
        "#,
        expect![[r#"
            ERROR: parse error at line 2, column 10
              |
            2 |         { abc }
              |          ^
            invalid value"#]],
    );

    do_test(
        r#"
        "\a"
        "#,
        expect![[r#"
            ERROR: parse error at line 2, column 11
              |
            2 |         "\a"
              |           ^
            invalid string"#]],
    );
}

fn main() {}
