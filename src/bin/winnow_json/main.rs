#![feature(trait_alias)]

use std::{collections::HashMap, error::Error};

use expect_test::{expect, Expect};
use winnow::{
    combinator::{alt, delimited, repeat, separated, separated_pair, terminated},
    error::StrContext,
    stream::{Compare, StreamIsPartial},
    token::{any, none_of, take_while},
    PResult, Parser,
};

type Array = Vec<Value>;
type Object = HashMap<String, Value>;

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
    let c = none_of('"').parse_next(input)?;

    if c == '\\' {
        any.verify_map(|c: char| {
            Some(match c {
                '"' | '\\' | '/' => c,
                'n' => '\n',
                _ => return None,
            })
        })
        .context(StrContext::Label("escape char"))
        .parse_next(input)
    } else {
        Ok(c)
    }
}

fn string(input: &mut impl Stream) -> PResult<String> {
    delimited(
        '"',
        repeat(0.., char).fold(String::new, |mut string, c| {
            string.push(c);
            string
        }),
        '"',
    )
    .parse_next(input)
}

fn ws(input: &mut impl Stream) -> PResult<()> {
    take_while(0.., |c| " \t\r\n".contains(c))
        .void()
        .parse_next(input)
}

fn key_value(input: &mut impl Stream) -> PResult<(String, Value)> {
    separated_pair(string, (ws, ':', ws), value).parse_next(input)
}

fn object(input: &mut impl Stream) -> PResult<Object> {
    delimited('{', separated(0.., delimited(ws, key_value, ws), ','), '}').parse_next(input)
}

fn value(input: &mut impl Stream) -> PResult<Value> {
    alt((string.map(Value::String), object.map(Value::Object))).parse_next(input)
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
        expect![[r#"Object({"foo": Object({"hello": String("world")}), "abc": String("def")})"#]],
    );

    do_test(
        r#"
        { "abc": shit }
        "#,
        expect![[r#"
            ERROR: parse error at line 2, column 10
              |
            2 |         { "abc": shit }
              |          ^
        "#]],
    );
}

fn main() {}
