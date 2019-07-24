use super::*;
use crate::{error::*, types::*};
use byteordered::Endianness;
use std::num::NonZeroU32;

pub struct MessageBuilder<T: DBusType> {
    endianness: Endianness,
    ty: MessageType,
    flags: Flags,
    version: u8,
    serial: NonZeroU32,
    fields: Vec<HeaderField>,
    data: T,
}

impl<T: DBusType> MessageBuilder<T> {
    pub(super) fn new(data: T) -> MessageBuilder<T> {
        MessageBuilder {
            endianness: Endianness::native(),
            ty: MessageType::Invalid,
            flags: Flags::empty(),
            version: 1,
            serial: rand::random(),
            fields: Vec::new(),
            data,
        }
    }

    pub fn message_type(&mut self, ty: MessageType) {
        self.ty = ty;
    }

    pub fn add_field(&mut self, field: HeaderField) {
        self.fields.push(field);
    }

    pub fn add_signature(&mut self) {
        let signature = Signature::new(T::signature()).unwrap();
        self.add_field(HeaderField::Signature(signature));
    }

    pub fn serial(&mut self) -> u32 {
        self.serial.get()
    }

    pub fn build(self) -> Result<Message<T>> {
        let body = self.encode_data()?;
        let body_size = body.len();

        let header = MessageHeader {
            endianness: self.endianness,
            ty: self.ty,
            flags: self.flags,
            version: self.version,
            body_size: body_size as u32,
            serial: self.serial.get(),
            fields: self.fields,
        };

        Ok(Message {
            header,
            data: self.data,
        })
    }

    fn encode_data(&self) -> Result<Vec<u8>> {
        use crate::marshal::Marshaller;

        let mut marshaller = Marshaller::new_native(Vec::new());
        self.data.encode(&mut marshaller)?;
        Ok(marshaller.into_inner())
    }
}
