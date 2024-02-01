use std::error::Error;

use thiserror::Error;
use thiserror_ext::AsReport;

#[derive(Error, Debug)]
#[error("{message}")]
struct MyError {
    message: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
}

#[easy_ext::ext]
impl<T, E> Result<T, E>
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn into_my_error(self, message: impl Into<String>) -> Result<T, MyError> {
        self.map_err(|e| MyError {
            message: message.into(),
            source: Some(e.into()),
        })
    }
}

fn test() -> Result<(), MyError> {
    "foo".parse::<i32>().into_my_error("failed to parse")?;
    Ok(())
}

fn main() {
    let e = test().unwrap_err();
    println!("{}", e.as_report());
}
