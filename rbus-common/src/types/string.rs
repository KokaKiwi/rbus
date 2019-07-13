use custom_error::custom_error;
use rbus_derive::impl_type;

// Basic strings
// TODO: Validate strings? (according to DBus specs)
impl_type! {
    #[dbus(basic, align = 4)]
    str: 's'
}
impl_type! {
    #[dbus(basic, align = 4)]
    String: 's'
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
    #[dbus(basic, align = 4)]
    ObjectPath: 'o'
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
    #[dbus(basic)]
    Signature: 'g'
}
