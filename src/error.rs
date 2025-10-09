use std::{
    fmt::{self, Display},
    string::FromUtf8Error,
};
pub enum Error {
    Utf8Error(FromUtf8Error),
    IO(std::io::Error),
    SerdeJson(serde_json::Error),
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
            Error::IO(e) => write!(f, "I/O error: {}", e),
            Error::SerdeJson(e) => write!(f, "serde_json error: {}", e),
        }
    }
}
