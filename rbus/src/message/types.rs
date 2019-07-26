use crate::types::DBusType;
use custom_error::custom_error;
use derive_more::*;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Deref, DBusType)]
#[dbus(module = "crate", proxy(String, inner))]
pub struct Interface(String);

custom_error! {
    pub InterfaceError
        InvalidInterfaceName { message: String }
            = "Invalid interface name: {message}",
}

impl Interface {
    pub fn new<T: AsRef<str>>(name: T) -> Result<Self, InterfaceError> {
        lazy_static! {
            static ref INTERFACE_SEGMENT_REGEX: Regex = Regex::new(r"^([[:alpha:]]|_)([[:alnum:]]|_)*$").unwrap();
        }

        let name = name.as_ref();

        if name.len() > 255 {
            return Err(InterfaceError::InvalidInterfaceName {
                message: "Interface names must not exceed the maximum name length (255)".into(),
            });
        }

        if !name.contains('.') {
            return Err(InterfaceError::InvalidInterfaceName {
                message: "Interface names are composed of 2 or more elements separated by a period ('.') character"
                    .into(),
            });
        }

        for segment in name.split('.') {
            if segment.is_empty() {
                return Err(InterfaceError::InvalidInterfaceName {
                    message: "All elements must contain at least one character".into(),
                });
            }

            if !INTERFACE_SEGMENT_REGEX.is_match(segment) {
                return Err(InterfaceError::InvalidInterfaceName {
                    message: "Each element must only contain the ASCII characters \"[A-Z][a-z][0-9]_\" and must not \
                              begin with a digit"
                        .into(),
                });
            }
        }

        Ok(Self(name.into()))
    }
}

impl AsRef<str> for Interface {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

custom_error! {
    pub BusNameError
        InvalidBusName { message: String }
            = "Invalid bus name: {message}",
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DBusType)]
#[dbus(module = "crate", proxy(String, inner))]
pub struct BusName(String);

impl BusName {
    pub fn new<T: AsRef<str>>(name: T) -> Result<Self, BusNameError> {
        lazy_static! {
            static ref BUS_NAME_SEGMENT_REGEX: Regex = Regex::new(r"^([[:alnum:]]|_|-)+$").unwrap();
        }

        let name = name.as_ref();
        let unique = name.starts_with(':');

        if name.len() > 255 {
            return Err(BusNameError::InvalidBusName {
                message: "Bus names must not exceed the maximum name length (255)".into(),
            });
        }

        {
            let name = if unique { &name[1..] } else { name };

            if !name.contains('.') {
                return Err(BusNameError::InvalidBusName {
                    message: "Bus names must contain at least one '.' (period) character (and thus at least two \
                              elements)"
                        .into(),
                });
            }

            if name.starts_with('.') {
                return Err(BusNameError::InvalidBusName {
                    message: "Bus names must not begin with a '.' (period) character".into(),
                });
            }

            for segment in name.split('.') {
                if segment.is_empty() {
                    return Err(BusNameError::InvalidBusName {
                        message: "All elements must contain at least one character".into(),
                    });
                }

                if !unique && segment.starts_with(|c: char| c.is_digit(10)) {
                    return Err(BusNameError::InvalidBusName {
                        message: "Only elements that are part of a unique connection name may begin with a digit, \
                                  elements in other bus names must not begin with a digit."
                            .into(),
                    });
                }

                if !BUS_NAME_SEGMENT_REGEX.is_match(segment) {
                    return Err(BusNameError::InvalidBusName {
                        message: "Each element must only contain the ASCII characters \"[A-Z][a-z][0-9]_-\"".into(),
                    });
                }
            }
        }

        Ok(Self(name.into()))
    }
}

impl AsRef<str> for BusName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DBusType)]
#[dbus(module = "crate", proxy(String, inner))]
pub struct Member(String);

custom_error! {
    pub MemberError
        InvalidMemberName { message: String }
            = "Invalid member name: {message}",
}

impl Member {
    pub fn new<T: AsRef<str>>(name: T) -> Result<Self, MemberError> {
        lazy_static! {
            static ref MEMBER_REGEX: Regex = Regex::new(r"^([[:alpha:]]|_)([[:alnum:]]|_)*$").unwrap();
        }

        let name = name.as_ref();

        if name.is_empty() {
            return Err(MemberError::InvalidMemberName {
                message: "Member name must be at least 1 byte in length".into(),
            });
        }

        if name.len() > 255 {
            return Err(MemberError::InvalidMemberName {
                message: "Member name must not exceed the maximum name length (255)".into(),
            });
        }

        if name.contains('.') {
            return Err(MemberError::InvalidMemberName {
                message: "Member name must not contain the '.' (period) character".into(),
            });
        }

        if !MEMBER_REGEX.is_match(name) {
            return Err(MemberError::InvalidMemberName {
                message: "Member name must only contain the ASCII characters \"[A-Z][a-z][0-9]_\" and may not begin \
                          with a digit"
                    .into(),
            });
        }

        Ok(Self(name.into()))
    }
}

impl AsRef<str> for Member {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DBusType)]
#[dbus(module = "crate", proxy(String, inner))]
pub struct ErrorName(String);

impl ErrorName {
    pub fn new<T: AsRef<str>>(name: T) -> Result<Self, InterfaceError> {
        Ok(Self(Interface::new(name)?.0))
    }
}

impl AsRef<str> for ErrorName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
