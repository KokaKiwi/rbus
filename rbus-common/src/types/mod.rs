pub use array::*;
pub use basic::*;
pub use rbus_derive::DBusType;
pub use string::*;

mod array;
mod basic;
mod string;

pub trait DBusType {
    fn code() -> u8;
    fn signature() -> String;
}

pub trait DBusBasicType: DBusType {
}
