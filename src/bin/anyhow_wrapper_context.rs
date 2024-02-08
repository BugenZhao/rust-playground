#![feature(error_generic_member_access)]

use std::{backtrace::Backtrace, num::ParseIntError, ops::Deref};

use anyhow::anyhow;
use thiserror_ext::AsReport;

/// Define a newtype wrapper around [`anyhow::Error`].
///
/// # Construction
///
/// Unlike [`anyhow::Error`], the newtype **CANNOT** be converted from any error
/// types implicitly. Instead, it can only be converted from [`anyhow::Error`]
/// by default. Users are encouraged to...
///
/// - Either use [`anyhow::Context`] to attach detailed information to the
///   source error before converting it to the newtype.
///
/// - Or specify the default context for each source error type in this macro
///   as shown in the example below, which will enable the implicit conversion
///   `From` the source error type to the newtype. This should not be preferred
///   in most cases as the context could be less informative, but it is still
///   useful during refactoring, or if the source error type is believed to be
///   self-explanatory.
///
/// To construct a new error from scratch, one can still use macros like
/// `anyhow::anyhow!` or `risingwave_common::bail!`. Since `bail!` and `?`
/// already imply an `into()` call, developers do not need to care about the
/// type conversion most of the time.
///
/// # Example
///
/// ```ignore
/// def_anyhow_newtype! {
///    /// Documentation for the newtype.
///    #[derive(..)]
///    pub MyError,
///
///    mysql::Error => "failed to interact with MySQL",
///    postgres::Error => "failed to interact with PostgreSQL",
/// }
/// ```
///
/// # Discussion
///
/// - What's the purpose of the newtype?
///   * It is to provide extra type information for errors, which makes it
///     clearer to identify which module or crate the error comes from when
///     it is passed around.
///   * It enforces the developer to attach context (explicitly or by default)
///     when doing type conversion, which makes the error more informative.
///
/// - Is the effect essentially the same as `thiserror`?
///   * Yes, but we're here intentionally making the error type less actionable
///     to make it informative with no fear.
///   * To elaborate, consider the following `thiserror` example:
///     ```rust,ignore
///     #[derive(thiserror::Error, Debug)]
///     pub enum MyError {
///         #[error("failed to interact with MySQL")]
///         MySql(#[from] mysql::Error),
///         #[error(transparent)]
///         Other(#[from] anyhow::Error),
///     }
///     ```
///     This gives the caller an illusion that all errors related to MySQL are
///     under the `MySql` variant, which is not true as one could attach context
///     to an `mysql::Error` with [`anyhow::Context`] and make it go into the
///     `Other` variant.
///
///     By doing type erasure with `anyhow`, we're making it clear that the
///     error is not actionable.
#[macro_export]
macro_rules! def_anyhow_newtype {
    (
        $(#[$attr:meta])* $vis:vis $name:ident
        $(, $from:ty => $context:literal)* $(,)?
    ) => {
        #[derive(::thiserror::Error, ::std::fmt::Debug)]
        #[error(transparent)]
        $(#[$attr])* $vis struct $name(#[from] #[backtrace] ::anyhow::Error);

        impl $name {
            /// Unwrap the newtype to get the inner [`anyhow::Error`].
            pub fn into_inner(self) -> ::anyhow::Error {
                self.0
            }
        }

        $(
            impl From<$from> for $name {
                fn from(error: $from) -> Self {
                    Self(::anyhow::Error::new(error).context($context))
                }
            }
        )*
    };
}

def_anyhow_newtype! {
    pub MyError,
    ParseIntError => "failed to parse integer",
}

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
    let _e1: MyError = anyhow!("233").into();
    let e2: MyError = "foo".parse::<i32>().unwrap_err().into();

    let o = OuterError::Inner(e2);
    println!("{}", o.as_report());
}
