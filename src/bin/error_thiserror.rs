#![feature(error_generic_member_access)]

// Port from `snafu`
mod clean {
    use std::{backtrace::Backtrace, fmt};

    pub struct ReportFormatter<'a>(pub &'a dyn std::error::Error);

    impl<'a> fmt::Display for ReportFormatter<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.cleaned_error_trace(f)?;

            // Note(bugen): may gate on `alternate`.
            if let Some(bt) = std::error::request_ref::<Backtrace>(self.0) {
                writeln!(f, "\nBacktrace:\n{}", bt)?;
            }

            Ok(())
        }
    }

    impl<'a> ReportFormatter<'a> {
        fn cleaned_error_trace(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            const NOTE: char = '*';

            let cleaned_messages: Vec<_> = CleanedErrorText::new(self.0)
                .flat_map(|(_, mut msg, cleaned)| {
                    if msg.is_empty() {
                        None
                    } else {
                        if cleaned {
                            msg.push(' ');
                            msg.push(NOTE);
                        }
                        Some(msg)
                    }
                })
                .collect();

            let mut visible_messages = cleaned_messages.iter();

            let head = match visible_messages.next() {
                Some(v) => v,
                None => return Ok(()),
            };

            writeln!(f, "{}", head)?;

            match cleaned_messages.len() {
                0 | 1 => {}
                2 => writeln!(f, "\nCaused by this error:")?,
                _ => writeln!(f, "\nCaused by these errors (recent errors listed first):")?,
            }

            for (i, msg) in visible_messages.enumerate() {
                // Let's use 1-based indexing for presentation
                let i = i + 1;
                writeln!(f, "{:3}: {}", i, msg)?;
            }

            Ok(())
        }
    }

    /// An iterator over an Error and its sources that removes duplicated
    /// text from the error display strings.
    ///
    /// It's common for errors with a `source` to have a `Display`
    /// implementation that includes their source text as well:
    ///
    /// ```text
    /// Outer error text: Middle error text: Inner error text
    /// ```
    ///
    /// This works for smaller errors without much detail, but can be
    /// annoying when trying to format the error in a more structured way,
    /// such as line-by-line:
    ///
    /// ```text
    /// 1. Outer error text: Middle error text: Inner error text
    /// 2. Middle error text: Inner error text
    /// 3. Inner error text
    /// ```
    ///
    /// This iterator compares each pair of errors in the source chain,
    /// removing the source error's text from the containing error's text:
    ///
    /// ```text
    /// 1. Outer error text
    /// 2. Middle error text
    /// 3. Inner error text
    /// ```

    pub struct CleanedErrorText<'a>(Option<CleanedErrorTextStep<'a>>);

    impl<'a> CleanedErrorText<'a> {
        /// Constructs the iterator.
        pub fn new(error: &'a dyn std::error::Error) -> Self {
            Self(Some(CleanedErrorTextStep::new(error)))
        }
    }

    impl<'a> Iterator for CleanedErrorText<'a> {
        /// The original error, the display string and if it has been cleaned
        type Item = (&'a dyn std::error::Error, String, bool);

        fn next(&mut self) -> Option<Self::Item> {
            use std::mem;

            let mut step = self.0.take()?;
            let mut error_text = mem::take(&mut step.error_text);

            match step.error.source() {
                Some(next_error) => {
                    let next_error_text = next_error.to_string();

                    let cleaned_text = error_text
                        .trim_end_matches(&next_error_text)
                        .trim_end()
                        .trim_end_matches(':');
                    let cleaned = cleaned_text.len() != error_text.len();
                    let cleaned_len = cleaned_text.len();
                    error_text.truncate(cleaned_len);

                    self.0 = Some(CleanedErrorTextStep {
                        error: next_error,
                        error_text: next_error_text,
                    });

                    Some((step.error, error_text, cleaned))
                }
                None => Some((step.error, error_text, false)),
            }
        }
    }

    struct CleanedErrorTextStep<'a> {
        error: &'a dyn std::error::Error,
        error_text: String,
    }

    impl<'a> CleanedErrorTextStep<'a> {
        fn new(error: &'a dyn std::error::Error) -> Self {
            let error_text = error.to_string();
            Self { error, error_text }
        }
    }
}

// https://github.com/rust-lang/rust/issues/117432
mod my_box {
    use std::ops::{Deref, DerefMut};

    #[derive(Clone)]
    pub struct MyBox<T>(Box<T>);

    impl<T> MyBox<T> {
        pub fn new(t: T) -> Self {
            Self(Box::new(t))
        }
    }

    impl<T> Deref for MyBox<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for MyBox<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T: std::fmt::Display> std::fmt::Display for MyBox<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl<T: std::fmt::Debug> std::fmt::Debug for MyBox<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl<T: std::error::Error> std::error::Error for MyBox<T> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            T::source(&*self.0)
        }

        fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
            T::provide(&*self.0, request)
        }
    }
}

// Port from `snafu`
mod context {
    pub trait IntoError<E>
    where
        E: std::error::Error,
    {
        /// The underlying error
        type Source;

        /// Combine the information to produce the error
        fn into_error(self, source: Self::Source) -> E;
    }

    #[easy_ext::ext(ResultExt)]
    pub impl<T, E> Result<T, E> {
        fn context<C, E2>(self, context: C) -> Result<T, E2>
        where
            C: IntoError<E2, Source = E>,
            E2: std::error::Error,
        {
            self.map_err(|error| context.into_error(error))
        }

        fn with_context<C, E2, F>(self, context: F) -> Result<T, E2>
        where
            C: IntoError<E2, Source = E>,
            E2: std::error::Error,
            F: FnOnce() -> C,
        {
            self.map_err(|error| context().into_error(error))
        }
    }
}

// Followings are the use cases.

use context::*;
use std::backtrace::Backtrace;

#[derive(thiserror::Error, Debug)]
pub enum MyErrorInner {
    // No need to include the source error in the message, but reliably maintain the source chain.
    #[error("network error")]
    Network {
        #[from] // This will help us implement `source`.
        error: hyper::Error,
        backtrace: Backtrace, // We're sure that `hyper::Error` does not have `Backtrace` and we want to include it, then write it here
                              // It'll be provided based on the field name `backtrace`.
    },

    // However, it's still okay to interpolate `source` into the message thanks to `clean`.
    #[error("io error: {error}")]
    Io {
        #[from]
        error: std::io::Error,
        backtrace: Backtrace,
    },

    // This shows how to use `context` to construct error type in a more elegant way.
    #[error("cannot parse int from `{from}`")]
    Parse {
        #[source]
        error: std::num::ParseIntError,
        from: String,
    },

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error(transparent)]
    Uncategorized(
        // This will help us implement `source`.
        #[from]
        // Only annotate `backtrace` will it call `provide` on `anyhow::Error`.
        #[backtrace]
        anyhow::Error,
    ),
}

// For better construction of `MyErrorInner::Parse`.
// Manually implement the `XxxSnafu`-like structs. This might be verbose.
struct ParseContext<F> {
    from: F,
}

impl<F> IntoError<MyError> for ParseContext<F>
where
    F: Into<String>,
{
    type Source = std::num::ParseIntError;

    fn into_error(self, source: Self::Source) -> MyError {
        MyErrorInner::Parse {
            error: source,
            from: self.from.into(),
        }
        .into()
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MyError(
    // The reason for such annotations is exactly the same as `anyhow::Error`.
    #[from]
    #[backtrace]
    pub my_box::MyBox<MyErrorInner>, // To make sure the size is one word.

                                     // do not always include a backtrace here.
);

// For `?` to work on the wrapped `MyError`.
impl<E> From<E> for MyError
where
    E: Into<MyErrorInner>,
{
    fn from(error: E) -> Self {
        Self(my_box::MyBox::new(error.into()))
    }
}

async fn work() -> Result<(), MyError> {
    hyper::client::Client::new()
        .get(hyper::Uri::from_static("http://not-exist"))
        .await?;

    Ok(())
}

async fn work_2() -> Result<(), MyError> {
    let from = "not a number";
    let _ = from.parse::<i32>().context(ParseContext { from })?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let error = work().await.unwrap_err();
    print_error(&error);

    let error = work_2().await.unwrap_err();
    print_error(&error);
}

fn print_error(error: &MyError) {
    // Always print the error with `snafu_clean::ReportFormatter`.
    println!("{}", clean::ReportFormatter(error));
}
