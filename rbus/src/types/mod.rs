use crate::{marshal::Marshaller, Result};
pub use array::*;
pub use basic::*;
pub use dict::*;
#[doc(hidden)]
pub use rbus_derive::{impl_type, DBusType};
use std::io;
pub use string::*;

mod array;
mod basic;
mod dict;
mod string;
mod tuple;

pub trait DBusType: Sized {
    fn code() -> u8;
    fn signature() -> String;
    fn alignment() -> u8;

    fn encode<Inner>(&self, marshaller: &mut Marshaller<Inner>) -> Result<()>
    where
        Inner: io::Write;
    fn decode<Inner>(marshaller: &mut Marshaller<Inner>) -> Result<Self>
    where
        Inner: io::Read;
}

impl_type! {
    impl<T: DBusType> &T {
        code() {
            T::code()
        }

        signature() {
            T::signature()
        }

        alignment() {
            T::alignment()
        }

        encode(marshaller) {
            (*self).encode(marshaller)
        }

        decode(_marshaller) {
            use crate::Error;

            Err(Error::Custom {
                message: "References cannot be decoded".into(),
            })
        }
    }
}

pub trait DBusBasicType: DBusType {}

impl<T: DBusBasicType> DBusBasicType for &T {}
