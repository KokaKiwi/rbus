use crate::types::{DBusPackedTypes, DBusType};
use crate::Result;
use byteordered::ByteOrdered;
pub use byteordered::Endianness;
use std::io::{self, Cursor, Read, Write};
use std::ops::{Deref, DerefMut};

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

    pub fn io(&mut self) -> ByteOrdered<&mut T, Endianness> {
        ByteOrdered::runtime(self.inner.get_mut(), self.endianness)
    }
}

impl<T> Marshaller<T>
where
    T: AsRef<[u8]> + AsMut<[u8]> + Write,
{
    fn writer(&mut self) -> Cursor<&mut [u8]> {
        Cursor::new(self.as_mut())
    }
}

impl<T: AsRef<[u8]>> Deref for Marshaller<T> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.get_ref().as_ref()
    }
}

impl<T> DerefMut for Marshaller<T>
where
    T: AsRef<[u8]> + AsMut<[u8]>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.get_mut().as_mut()
    }
}

impl Marshaller<Vec<u8>> {
    pub fn into_vec(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

impl<T> Marshaller<T>
where
    T: AsRef<[u8]> + AsMut<[u8]> + Write,
{
    pub fn write_padding(&mut self, alignment: u8) -> io::Result<()> {
        let padding = vec![0; self.len() % alignment as usize];
        self.writer().write_all(&padding)?;
        Ok(())
    }

    pub fn write_value<U: DBusType>(&mut self, value: &U) -> Result<()> {
        self.write_padding(U::alignment())?;
        value.encode(self)
    }

    pub fn write_packed<U: DBusPackedTypes>(&mut self, value: U) -> Result<()> {
        value.encode(self)
    }
}

impl<T> Marshaller<T>
where
    T: AsRef<[u8]> + Read,
{
    pub fn read_padding(&mut self, alignment: u8) -> io::Result<usize> {
        let padding_len = self.inner.position() as usize % alignment as usize;
        let mut padding = Vec::with_capacity(padding_len);
        unsafe { padding.set_len(padding_len) }
        self.inner.read_exact(&mut padding)?;
        Ok(padding_len)
    }

    pub fn read_value<U: DBusType>(&mut self) -> Result<U> {
        self.read_padding(U::alignment())?;
        U::decode(self)
    }

    pub fn read_packed<U: DBusPackedTypes>(&mut self) -> Result<U> {
        U::decode(self)
    }
}
