use std::{collections::HashMap, convert::Infallible};

use serde::{de::value::MapDeserializer, forward_to_deserialize_any, Deserialize};

#[derive(serde::Deserialize, Debug)]
struct Config {
    foo: String,
    qux: Option<String>,
    // foo: i32,
    #[serde(flatten)]
    nested: Nested,
}

#[derive(serde::Deserialize, Debug)]
struct Nested {
    fuz: String,
}

fn main() {
    struct OptionStr<'de>(&'de str);

    impl<'de> serde::Deserializer<'de> for OptionStr<'de> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.visit_str(self.0)
        }

        // Comment out this function will cause error.
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.visit_some(self)
        }

        forward_to_deserialize_any! {
            // option
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    impl<'de> serde::de::IntoDeserializer<'de> for OptionStr<'de> {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self::Deserializer {
            self
        }
    }

    let mut map = HashMap::new();
    map.insert("foo".to_owned(), "42".to_owned());
    map.insert("fuz".to_owned(), "baz".to_owned());
    map.insert("qux".to_owned(), "quux".to_owned());

    let c = Config::deserialize(MapDeserializer::<_, serde::de::value::Error>::new(
        map.iter().map(|(k, v)| (k.as_str(), OptionStr(v.as_str()))),
    ))
    .unwrap();

    println!("{:?}", c);
}
