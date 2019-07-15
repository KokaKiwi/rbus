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
    fn alignment() -> u8;
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
}

pub trait DBusBasicType: DBusType {}

impl<T: DBusBasicType> DBusBasicType for &T {}
