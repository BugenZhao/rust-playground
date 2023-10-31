#![feature(error_generic_member_access)]
#![feature(provide_any)]
#![feature(error_iter)]
#![feature(type_name_of_val)]

use snafu::{prelude::*, AsBacktrace, Backtrace, GenerateImplicitData, Location, Whatever};

#[derive(Debug)]
struct MyBacktrace(Option<Box<Backtrace>>);

impl AsBacktrace for MyBacktrace {
    fn as_backtrace(&self) -> Option<&Backtrace> {
        self.0.as_ref().map(|b| &**b)
    }
}

impl GenerateImplicitData for MyBacktrace {
    fn generate() -> Self {
        Self(<Option<Backtrace>>::generate().map(Box::new))
    }

    fn generate_with_source(source: &dyn snafu::Error) -> Self
    where
        Self: Sized,
    {
        Self(<Option<Backtrace>>::generate_with_source(source).map(Box::new))
    }
}

#[derive(Debug)]
struct Suggestion(String);

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not read file {path}"))]
    #[snafu(provide(opt, ref, Suggestion => suggestion))]
    #[snafu(provide(ref, Location => location))]
    ConfigFile {
        source: std::io::Error,
        path: String,
        backtrace: MyBacktrace,
        location: Location,
        suggestion: Option<Suggestion>,
    },

    #[snafu(display("Could not parse address"), context(false))]
    ParseAddr { source: std::net::AddrParseError },

    #[snafu(whatever, display("{message}"))]
    Uncategorized {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        #[snafu(provide(false))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

#[derive(Debug, Snafu)]
enum OuterError {
    #[snafu(display("inner error"), context(false))]
    Inner {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },
}

fn read_config_file(path: &str) -> Result<String, Error> {
    std::fs::read_to_string(path).context(ConfigFileSnafu {
        path,
        suggestion: Suggestion("better luck".into()),
    })
}

fn parse_address(address: &str) -> Result<std::net::IpAddr, Error> {
    let addr = address.parse()?;

    Ok(addr)
}

fn whatever() -> Result<(), Error> {
    // snafu::Report::
    whatever!("hello")
}

fn whatever_2() -> Result<(), Error> {
    let _ = whatever!(parse_address("bad"), "bad context");
    Ok(())
}

fn whatever_3() -> Result<(), Error> {
    let _ = parse_address("bad").whatever_context::<_, Error>("bad context")?;
    Ok(())
}

fn outer() -> Result<(), OuterError> {
    let _ = whatever()?;
    Ok(())
}

fn main() {}
