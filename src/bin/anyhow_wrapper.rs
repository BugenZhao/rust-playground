#![feature(error_generic_member_access)]

use std::{backtrace::Backtrace, ops::Deref};

use anyhow::anyhow;
use thiserror_ext::AsReport;

/// Define a newtype wrapper around [`anyhow::Error`].
///
/// # Motivation
///
/// [`anyhow::Error`] is good enough if one just wants to make the error type
/// informative but not necessarily actionable. However, given a value of type
/// [`anyhow::Error`], it is hard to tell which module or crate it comes from,
/// which may blur the boundary between modules when passing it around, leading
/// to abuse.
///
/// # Usage
///
/// ```ignore
/// def_anyhow_newtype! {
///    /// Documentation for the newtype.
///    #[derive(..)]
///    pub MyError;
/// }
/// ```
///
/// This will define a newtype `MyError` around [`anyhow::Error`].
///
/// The newtype can be converted from any type that implements `Into<anyhow::Error>`,
/// so the developing experience is kept almost the same. To construct a new error,
/// one can still use macros like `anyhow::anyhow!` or `risingwave_common::bail!`.
///
/// Since `bail!` and `?` already imply an `into()` call, developers do not need to
/// care about the conversion from [`anyhow::Error`] to the newtype most of the time.
///
/// # Limitation
///
/// Note that the newtype does not implement [`std::error::Error`] just like
/// [`anyhow::Error`]. However, it can be dereferenced to `dyn std::error::Error`
/// to be used in places like `thiserror`'s `#[source]` attribute.
#[macro_export]
macro_rules! def_anyhow_newtype {
    ($(#[$attr:meta])* $vis:vis $name:ident $(;)?) => {
        $(#[$attr])* $vis struct $name(::anyhow::Error);

        impl $name {
            /// Unwrap the newtype to get the inner [`anyhow::Error`].
            pub fn into_inner(self) -> ::anyhow::Error {
                self.0
            }
        }

        impl std::ops::Deref for $name {
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

        impl From<$name> for Box<dyn std::error::Error + Send + Sync + 'static> {
            fn from(value: $name) -> Self {
                value.0.into()
            }
        }

        paste::paste! {
            /// Provides the `context` method for `Result` of [`ConnectorError`].
            ///
            /// This trait is the supplement of the [`anyhow::Context`] trait as it cannot be
            /// implemented for types outside of the crate.
            #[easy_ext::ext([< $name Context >])]
            $vis impl<T> Result<T, $name> {
                /// Wrap the error value with additional context.
                fn context<C>(self, context: C) -> Result<T, ::anyhow::Error>
                where
                    C: std::fmt::Display + Send + Sync + 'static,
                {
                    ::anyhow::Context::context(self.map_err(|error| error.0), context)
                }

                /// Wrap the error value with additional context that is evaluated lazily
                /// only once an error does occur.
                fn with_context<C, F>(self, context: F) -> Result<T, ::anyhow::Error>
                where
                    C: std::fmt::Display + Send + Sync + 'static,
                    F: FnOnce() -> C,
                {
                    ::anyhow::Context::with_context(self.map_err(|error| error.0), context)
                }
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
