use crate::{
    marshal::Marshaller,
    types::{impl_type, DBusType, ObjectPath, Signature},
};
use bitflags::bitflags;
use byteordered::Endianness;

impl_type! {
    #[dbus(proxy(u8))]
    Endianness {
        encode(marshaller) {
            let value = match *self {
                Endianness::Little => b'l',
                Endianness::Big => b'B',
            };
            value.encode(marshaller)
        }

        decode(marshaller) {
            use crate::Error;

            let value = marshaller.io().read_u8()?;
            match value {
                b'l' => Ok(Endianness::Little),
                b'B' => Ok(Endianness::Big),
                _ => Err(Error::InvalidEndianness { value }),
            }
        }
    }
}

fn mutate_marshaller<T>(endianness: &Endianness, marshaller: &mut Marshaller<T>) {
    marshaller.endianness = *endianness;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DBusType)]
#[dbus(module = "crate")]
#[repr(u8)]
pub enum MessageType {
    #[doc(hidden)]
    Invalid = 0,
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

bitflags! {
    #[derive(DBusType)]
    #[dbus(module = "crate", proxy(u8, get = "bits", set = "from_bits_truncate"))]
    pub struct Flags: u8 {
        const NO_REPLY_EXPECTED                 = 0x1;
        const NO_AUTO_START                     = 0x2;
        const ALLOW_INTERACTIVE_AUTHORIZATION   = 0x4;
    }
}

#[derive(Debug, Clone, PartialEq, DBusType)]
#[dbus(module = "crate", index(u8))]
pub enum HeaderField {
    #[doc(hidden)]
    Invalid,
    Path(ObjectPath),
    Interface(String),
    Member(String),
    ErrorName(String),
    ReplySerial(u32),
    Destination(String),
    Sender(String),
    Signature(Signature),
    UnixFds(u32),
}

#[derive(Debug, Clone, PartialEq, DBusType)]
#[dbus(packed, module = "crate", size(align = 8))]
pub struct MessageHeader {
    #[dbus(mutate_marshaller = "mutate_marshaller")]
    pub endianness: Endianness,
    pub ty: MessageType,
    pub flags: Flags,
    pub version: u8,
    pub body_size: u32,
    pub serial: u32,
    pub fields: Vec<HeaderField>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_dbus_header_decode_be() {
        let bytes: &[u8] = &[
            b'B', // Big endian
            2,    // reply (simplest message)
            0x2,  // No auto-starting
            1,    // D-Bus version = 1
            0, 0, 0, 4, // body size in bytes = 4
            0x12, 0x34, 0x56, 0x78, // Serial number = 0x12345678
            // variables header fields
            0, 0, 0, 0xf, // array size in bytes = 15
            5,   // ReplySerial
            1, b'u', 0, // Variant signature
            0xab, 0xcd, 0xef, 0x12, // Serial value
            8,    // Signature
            1, b'g', 0, // Variant signature
            1, b'u', 0, // Variant value
            0, // 8-bytes boundary
        ];
        assert_eq!(bytes.len() % 8, 0); // Header must be 8-bytes aligned

        let mut marshaller = Marshaller::new_native(bytes);
        let header = MessageHeader::decode(&mut marshaller).unwrap();
        let rest = marshaller.into_inner();

        let value = MessageHeader {
            endianness: Endianness::Big,
            ty: MessageType::MethodReturn,
            flags: Flags::NO_AUTO_START,
            version: 1,
            body_size: 4,
            serial: 0x12345678,
            fields: vec![
                HeaderField::ReplySerial(0xabcdef12),
                HeaderField::Signature(Signature::new("u").unwrap()),
            ],
        };

        assert_eq!(header, value);
        assert_eq!(rest.len(), 0);
    }
}
