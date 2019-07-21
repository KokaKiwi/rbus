use crate::marshal::Marshaller;
use crate::Result;
pub use array::*;
pub use basic::*;
pub use dict::*;
#[doc(hidden)]
pub use rbus_derive::{impl_type, DBusType};
use std::io;
pub use string::*;
pub use tuple::*;

mod array;
mod basic;
mod dict;
mod string;
mod tuple;

pub trait DBusType: Sized {
    fn code() -> u8;
    fn signature() -> String;
    fn alignment() -> u8;

    fn encode<Inner: AsRef<[u8]> + io::Write>(
        &self,
        marshaller: &mut Marshaller<Inner>,
    ) -> Result<()>;
    fn decode<Inner: AsRef<[u8]> + io::Read>(marshaller: &mut Marshaller<Inner>) -> Result<Self>;
}

impl<T: DBusType> DBusType for &T {
    fn code() -> u8 {
        T::code()
    }
    fn signature() -> String {
        T::signature()
    }
    fn alignment() -> u8 {
        T::alignment()
    }

    fn encode<Inner: AsRef<[u8]> + io::Write>(
        &self,
        marshaller: &mut Marshaller<Inner>,
    ) -> Result<()> {
        (*self).encode(marshaller)
    }
    fn decode<Inner: AsRef<[u8]> + io::Read>(_marshaller: &mut Marshaller<Inner>) -> Result<Self> {
        use crate::Error;

        Err(Error::Custom {
            message: "References cannot be decoded".into(),
        })
    }
}

pub trait DBusBasicType: DBusType {}

impl<T: DBusBasicType> DBusBasicType for &T {}
