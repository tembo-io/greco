use std::string::FromUtf8Error;

use actix_web::ResponseError;
use url::ParseError;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP server error: {0}")]
    Actix(#[from] actix_web::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Failed to parse URL: {0}")]
    Url(#[from] ParseError),
    #[error("Failed to convert with Pandoc")]
    Pandoc,
    #[error("README had no recognizable extension")]
    NoExtensionFound,
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Failed to build String from bytes: {0}")]
    Utf8(#[from] FromUtf8Error) 
}

impl ResponseError for Error {}
