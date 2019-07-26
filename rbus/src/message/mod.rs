use crate::types::DBusType;
pub use builder::*;
pub use header::*;

mod builder;
mod header;
pub mod types;

#[derive(Debug, Clone, DBusType)]
#[dbus(module = "crate")]
pub struct Message<T: DBusType> {
    header: MessageHeader,
    data: T,
}

impl<T: DBusType> Message<T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(data: T) -> MessageBuilder<T> {
        MessageBuilder::new(data)
    }
}
