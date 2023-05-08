use std::io::Write;
use std::{fmt, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
}

#[derive(Debug)]
struct ParseConfigError;

impl fmt::Display for ParseConfigError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("could not parse configuration file")
    }
}

// It's also possible to implement `Error` instead.
impl error_stack::Context for ParseConfigError {}

struct Suggestion(&'static str);

fn parse_config(path: impl AsRef<Path>) -> anyhow::Result<Config> {
    let path = path.as_ref();

    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open file: `{}`", path.display()))?;
    let config = serde_json::from_reader(file)?;

    Ok(config)
}

fn main() {
    let e1 = parse_config("fake.json").unwrap_err();
    println!("{:?}", e1);

    let e2 = parse_config({
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{}", r#"{"host": "localhost", "pot": 8080}"#).unwrap();
        f.into_temp_path()
    })
    .unwrap_err();
    println!("{:?}", e2);

    // for s in e2.request_ref::<Suggestion>() {
    //     println!("suggestion: {}", s.0);
    // }
}
