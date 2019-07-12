pub use rbus_common::{types};
use types::DBusType;

#[derive(DBusType)]
pub struct Test(u8, Vec<u16>, String);

#[derive(DBusType)]
pub struct Inner(Test, Test, Vec<String>);
