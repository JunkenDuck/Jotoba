use std::{fmt::Display, num::ParseIntError, string::FromUtf8Error};

use diesel::result::Error as DbError;
use strum::ParseError;
use tokio_diesel::AsyncError;

#[derive(Debug)]
pub enum Error {
    ParseInt(ParseIntError),
    XmlError(quick_xml::Error),
    Utf8Error(FromUtf8Error),
    Utf8StrError(std::str::Utf8Error),
    ParseError,
    Undefined,
    DbError(DbError),
    Checkout(r2d2::Error),
}

impl From<DbError> for Error {
    fn from(err: DbError) -> Self {
        Self::DbError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::VariantNotFound => Self::ParseError,
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

impl From<AsyncError> for Error {
    fn from(err: AsyncError) -> Self {
        match err {
            AsyncError::Checkout(co) => Self::Checkout(co),
            AsyncError::Error(err) => Self::DbError(err),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8StrError(err)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Self::XmlError(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}