use crate::types::{ObjectPathError, SignatureError};
use custom_error::custom_error;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

custom_error! {
    pub Error
        Io { source: io::Error }
            = "I/O error: {source}",
        FromUtf8 { source: std::string::FromUtf8Error }
            = "UTF-8 decoding error: {source}",
        Signature { source: SignatureError }
            = "Signature error: {source}",
        ObjectPath { source: ObjectPathError }
            = "Object path error: {source}",
        InvalidVariant { value: u64 }
            = "Invalid variant value: {value}",
        Custom { message: String }
            = "{message}",
        Unknown
            = "Unknown error",
}
