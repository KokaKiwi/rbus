use custom_error::custom_error;
use lazy_static::lazy_static;
use rbus_derive::impl_type;
use regex::Regex;

// Basic strings
// TODO: Validate strings? (according to DBus specs)
impl_type! {
    #[dbus(basic, align = 4, module = "crate")]
    &str: 's' {
        encode(marshaller) {
            marshaller.io().write_u32(self.len() as u32)?;
            marshaller.io().write_all(self.as_bytes())?;
            marshaller.io().write_u8(0)?;
            Ok(())
        }

        decode(_marshaller) {
            use crate::Error;

            Err(Error::Custom {
                message: "References cannot be decoded".into(),
            })
        }
    }
}

impl_type! {
    #[dbus(basic, align = 4, module = "crate")]
    String: 's' {
        encode(marshaller) {
            self.as_str().encode(marshaller)
        }

        decode(marshaller) {
            let length = marshaller.io().read_u32()?;
            let mut data = vec![0; length as usize];
            marshaller.io().read_exact(&mut data)?;
            marshaller.io().read_u8()?;
            let value = String::from_utf8(data)?;
            Ok(value)
        }
    }
}

// Object path
custom_error! {
    pub ObjectPathError
        InvalidObjectPath { message: String }
            = "Invalid object path: {message}"
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPath(String);

impl ObjectPath {
    pub fn new<T: AsRef<str>>(path: T) -> Result<Self, ObjectPathError> {
        lazy_static! {
            static ref OBJECT_PATH_SEGMENT_REGEX: Regex = Regex::new(r"^([[:alnum:]]|_)+$").unwrap();
        }

        let path = path.as_ref();

        if !path.contains('/') || !path.starts_with('/') {
            return Err(ObjectPathError::InvalidObjectPath {
                message: "The path must begin with an ASCII '/' (integer 47) character, and must consist of elements \
                          separated by slash characters"
                    .into(),
            });
        }

        if path.len() > 1 && path.ends_with('/') {
            return Err(ObjectPathError::InvalidObjectPath {
                message: "A trailing '/' character is not allowed unless the path is the root path (a single '/' \
                          character)"
                    .into(),
            });
        }

        if path.len() > 1 {
            for segment in path.split('/').skip(1) {
                if segment.is_empty() {
                    return Err(ObjectPathError::InvalidObjectPath {
                        message: "No element may be the empty string".into(),
                    });
                }

                dbg!(segment);
                if !OBJECT_PATH_SEGMENT_REGEX.is_match(segment) {
                    return Err(ObjectPathError::InvalidObjectPath {
                        message: "Each element must only contain the ASCII characters \"[A-Z][a-z][0-9]_\"".into(),
                    });
                }
            }
        }

        Ok(ObjectPath(path.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for ObjectPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl_type! {
    #[dbus(basic, align = 4, module = "crate")]
    ObjectPath: 'o' {
        encode(marshaller) {
            self.0.encode(marshaller)
        }

        decode(marshaller) {
            let value = String::decode(marshaller)?;
            Ok(ObjectPath(value))
        }
    }
}

// Signature
custom_error! {
    pub SignatureError
        InvalidSignature = "Invalid signature"
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature(String);

impl Signature {
    pub fn new<T: AsRef<str>>(sig: T) -> Result<Self, SignatureError> {
        let sig = sig.as_ref();
        // TODO: Validate signature

        Ok(Signature(sig.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for Signature {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl_type! {
    #[dbus(basic, module = "crate")]
    Signature: 'g' {
        encode(marshaller) {
            marshaller.io().write_u8(self.0.len() as u8)?;
            marshaller.io().write_all(self.0.as_bytes())?;
            marshaller.io().write_u8(0)?;
            Ok(())
        }

        decode(marshaller) {
            let length = marshaller.io().read_u8()?;
            let mut data = vec![0; length as usize];
            marshaller.io().read_exact(&mut data)?;
            marshaller.io().read_u8()?;
            let value = String::from_utf8(data)?;
            Ok(Signature(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_OBJECT_PATHS: &[&str] = &[
        "/",
        "/a",
        "/_",
        "/a/b/c",
        "/com/example/123",
        "/org/freedesktop/DBus",
        "/org/freedesktop/Telepathy/AccountManager",
    ];

    const INVALID_OBJECT_PATHS: &[&str] = &["", ".", "//", "/a/", "/-", "/com//example/MyApp", "/$"];

    #[test]
    fn test_valid_object_paths() {
        for path in VALID_OBJECT_PATHS {
            assert_ok!(ObjectPath::new(path), path);
        }
    }

    #[test]
    fn test_invalid_object_paths() {
        for path in INVALID_OBJECT_PATHS {
            assert_err!(ObjectPath::new(path), path);
        }
    }
}
