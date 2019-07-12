pub use array::*;
pub use basic::*;
pub use dict::*;
#[doc(hidden)]
pub use rbus_derive::DBusType;
pub use string::*;
pub use tuple::*;

mod array;
mod basic;
mod dict;
mod string;
mod tuple;

pub trait DBusType {
    fn code() -> u8;
    fn signature() -> String;
}

impl<T: DBusType> DBusType for &T {
    fn code() -> u8 { T::code() }
    fn signature() -> String { T::signature() }
}

pub trait DBusBasicType: DBusType {
}

impl<T: DBusBasicType> DBusBasicType for &T {}
