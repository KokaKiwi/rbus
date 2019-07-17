use crate::marshal::Marshaller;
pub use array::*;
pub use basic::*;
pub use dict::*;
#[doc(hidden)]
pub use rbus_derive::DBusType;
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

    fn encode<T: AsRef<[u8]> + io::Write>(&self, marshaller: &mut Marshaller<T>) -> io::Result<()>;
    fn decode<T: AsRef<[u8]> + io::Read>(marshaller: &mut Marshaller<T>) -> io::Result<Self>;
}

pub trait DBusBasicType: DBusType {}
