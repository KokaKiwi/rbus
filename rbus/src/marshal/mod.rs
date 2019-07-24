use crate::{types::DBusType, Result};
use byteordered::ByteOrdered;
pub use byteordered::Endianness;
use cursor::Cursor;
use std::{
    io::{self, Read, Write},
    ops::Deref,
};

mod cursor;

pub struct Marshaller<T> {
    inner: Cursor<T>,
    pub endianness: Endianness,
}

impl<T> Marshaller<T> {
    pub fn new(inner: T, endianness: Endianness) -> Marshaller<T> {
        Marshaller {
            inner: Cursor::new(inner),
            endianness,
        }
    }

    pub fn new_native(inner: T) -> Marshaller<T> {
        Marshaller::new(inner, Endianness::native())
    }

    pub fn io(&mut self) -> ByteOrdered<&mut Cursor<T>, Endianness> {
        ByteOrdered::runtime(&mut self.inner, self.endianness)
    }

    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }
}

impl<T> Deref for Marshaller<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> Marshaller<T>
where
    T: Write,
{
    pub fn write_padding(&mut self, alignment: u8) -> io::Result<()> {
        let padding = vec![0; self.inner.read_padding(alignment as usize)];
        self.inner.write_all(&padding)
    }

    pub fn write_value<U: DBusType>(&mut self, value: &U) -> Result<()> {
        value.encode(self)
    }
}

impl<T> Marshaller<T>
where
    T: Read,
{
    pub fn read_padding(&mut self, alignment: u8) -> io::Result<()> {
        let mut padding = Vec::with_capacity(self.inner.read_padding(alignment as usize));
        unsafe { padding.set_len(padding.capacity()) }
        self.inner.read_exact(&mut padding)
    }

    pub fn read_value<U: DBusType>(&mut self) -> Result<U> {
        U::decode(self)
    }
}
