#![feature(error_generic_member_access)]
#![feature(provide_any)]
#![feature(error_iter)]
#![feature(type_name_of_val)]

use thiserror::Error;
use traced::Traced;

mod traced {
    use std::backtrace::{Backtrace, BacktraceStatus};
    use thiserror::Error;

    struct Inner<E> {
        error: E,
        backtrace: Backtrace,
    }

    impl<E> Inner<E>
    where
        E: std::error::Error + 'static,
    {
        #[track_caller]
        fn new(error: E) -> Self {
            let requested = (&error as &dyn std::error::Error).request_ref::<Backtrace>();

            let backtrace = if requested.is_some() {
                Backtrace::disabled()
            } else {
                Backtrace::capture()
            };

            Self { error, backtrace }
        }

        fn causes(&self) -> impl Iterator<Item = &(dyn std::error::Error + 'static)> {
            (&self.error as &dyn std::error::Error).sources().skip(1)
        }
    }

    impl<E> std::error::Error for Inner<E>
    where
        E: std::error::Error + 'static,
        Self: std::fmt::Debug + std::fmt::Display,
    {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            E::source(&self.error)
        }

        fn provide<'a>(&'a self, demand: &mut std::any::Demand<'a>) {
            if let BacktraceStatus::Captured = self.backtrace.status() {
                demand.provide_ref::<Backtrace>(&self.backtrace);
            }
            E::provide(&self.error, demand);
        }
    }

    impl<E> std::fmt::Display for Inner<E>
    where
        E: std::error::Error + 'static,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.error)?;

            if f.alternate() {
                for cause in self.causes() {
                    write!(f, ": {}", cause)?;
                }
            }

            Ok(())
        }
    }

    impl<E> std::fmt::Debug for Inner<E>
    where
        E: std::error::Error + 'static,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self)?;

            let mut causes = self.causes().peekable();

            if causes.peek().is_some() {
                write!(f, "\n\nCaused by:")?;
                for cause in causes {
                    write!(f, "\n - {}", cause)?;
                }
            }

            if let Some(backtrace) = (self as &dyn std::error::Error).request_ref::<Backtrace>() {
                writeln!(f, "\n\nStack Backtrace:\n{}", backtrace)?;
            }

            Ok(())
        }
    }

    pub struct Traced<E>(Box<Inner<E>>);

    impl<E> From<E> for Traced<E>
    where
        E: std::error::Error + 'static,
    {
        #[track_caller]
        fn from(value: E) -> Self {
            Self(Box::new(Inner::new(value)))
        }
    }

    impl<E> std::error::Error for Traced<E>
    where
        E: std::error::Error + 'static,
        Self: std::fmt::Debug + std::fmt::Display,
    {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Inner::source(&*self.0)
        }

        fn provide<'a>(&'a self, demand: &mut std::any::Demand<'a>) {
            Inner::provide(&*self.0, demand)
        }
    }

    impl<E> std::fmt::Display for Traced<E>
    where
        E: std::error::Error + 'static,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Inner::fmt(&*self.0, f)
        }
    }

    impl<E> std::fmt::Debug for Traced<E>
    where
        E: std::error::Error + 'static,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Inner::fmt(&*self.0, f)
        }
    }
}

#[derive(Error, Debug)]
enum HummockErrorInner {
    #[error("Magic number mismatch: expected {expected}, found: {found}.")]
    MagicMismatch { expected: u32, found: u32 },
    #[error("Invalid format version: {0}.")]
    InvalidFormatVersion(u32),
}

pub type HummockError = Traced<HummockErrorInner>;

#[derive(Error, Debug)]
enum StreamErrorInner {
    #[error("hummock error")]
    Hummock(
        #[source]
        #[backtrace]
        HummockError,
    ),
    #[error("internal: {0}")]
    InternalError(String),
}

type StreamError = Traced<StreamErrorInner>;

impl From<HummockError> for StreamError {
    fn from(value: HummockError) -> Self {
        StreamErrorInner::Hummock(value).into()
    }
}

fn hummock_inner() -> Result<(), HummockError> {
    let err = HummockErrorInner::InvalidFormatVersion(233).into();
    Err(err)
}

fn hummock() -> Result<(), HummockError> {
    hummock_inner()
}

fn err() -> Result<(), StreamError> {
    hummock()?;
    Ok(())
}

fn main() {
    let err = err().unwrap_err();
    println!("Display:\n{}\n\n", err);
    println!("Display Alternate:\n{:#}\n\n", err);
    println!("Debug:\n{:?}\n\n", err);
}
