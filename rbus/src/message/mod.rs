use crate::marshal::Marshaller;
use crate::types::DBusType;
pub use builder::*;
pub use header::*;

mod builder;
mod header;

#[derive(Debug, Clone, DBusType)]
#[dbus(module = "crate")]
pub struct Message<T: DBusType> {
    header: MessageHeader,
    data: T,
}

impl<T: DBusType> Message<T> {
    pub fn new(data: T) -> MessageBuilder<T> {
        MessageBuilder::new(data)
    }
}
