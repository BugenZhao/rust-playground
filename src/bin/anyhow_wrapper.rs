#![feature(error_generic_member_access)]

use std::{backtrace::Backtrace, ops::Deref};

use anyhow::anyhow;
use thiserror_ext::AsReport;

macro_rules! def_anyhow_newtype {
    ($(#[$attr:meta])* $vis:vis $name:ident) => {
        $(#[$attr])* $vis struct $name(::anyhow::Error);

        impl Deref for $name {
            type Target = ::anyhow::Error;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<E> From<E> for $name
        where
            E: Into<::anyhow::Error>,
        {
            fn from(value: E) -> Self {
                Self(value.into())
            }
        }
    };
}

// Note that `MyError` does not `impl Error`,
// otherwise `From` implementation will get duplicated.
//
// But `backtrace`, `source`, `as_report` still works as it can be deref-ed
// into `dyn Error`. This is similar to how `anyhow::Error` works.
def_anyhow_newtype!(pub MyError);

#[derive(thiserror::Error, Debug)]
pub enum OuterError {
    #[error("inner")]
    Inner(
        #[from]
        #[backtrace]
        MyError,
    ),
}

fn main() {
    let e: MyError = anyhow!("233").into();
    let o = OuterError::Inner(e);
    println!("{}", o.as_report());
}
