use custom_error::custom_error;
use rbus_derive::impl_type;

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
        InvalidObjectPath = "Invalid object path"
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPath(String);

impl ObjectPath {
    pub fn new<T: AsRef<str>>(path: T) -> Result<Self, ObjectPathError> {
        // TODO: Validate path
        let path = path.as_ref();

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
        // TODO: Validate signature
        let sig = sig.as_ref();

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
