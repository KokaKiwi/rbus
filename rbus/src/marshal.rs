use crate::types::{DBusPackedTypes, DBusType};
use byteordered::{ByteOrdered, Endianness};
use std::io;

pub struct Marshaller<T> {
    inner: T,
    pub endianness: Endianness,
}

impl<T> Marshaller<T> {
    pub fn new(inner: T, endianness: Endianness) -> Marshaller<T> {
        Marshaller { inner, endianness }
    }

    pub fn io(&mut self) -> ByteOrdered<&mut T, Endianness> {
        ByteOrdered::runtime(&mut self.inner, self.endianness)
    }
}

impl<T: AsRef<[u8]>> Marshaller<T> {
    fn len(&self) -> usize {
        self.inner.as_ref().len()
    }
}

impl<T> Marshaller<T>
where
    T: AsRef<[u8]> + io::Write,
{
    pub fn write_padding(&mut self, alignment: u8) -> io::Result<()> {
        let padding = vec![0; self.len() % alignment as usize];
        self.inner.write_all(&padding)?;
        Ok(())
    }

    pub fn write_value<U: DBusType>(&mut self, value: U) -> io::Result<()> {
        self.write_padding(U::alignment())?;
        value.encode(self)?;
        Ok(())
    }

    pub fn write_packed<U: DBusPackedTypes>(&mut self, value: U) -> io::Result<()> {
        unimplemented!()
    }
}

impl<T> Marshaller<T>
where
    T: AsRef<[u8]> + io::Read,
{
    pub fn read_padding(&mut self, alignment: u8) -> io::Result<usize> {
        let padding_len = self.len() % alignment as usize;
        let mut padding = Vec::with_capacity(padding_len);
        unsafe { padding.set_len(padding_len) }
        self.inner.read_exact(&mut padding)?;
        Ok(padding_len)
    }

    pub fn read_value<U: DBusType>(&mut self) -> io::Result<U> {
        self.read_padding(U::alignment())?;
        let value = U::decode(self)?;
        Ok(value)
    }

    pub fn read_packed<U: DBusPackedTypes>(&mut self) -> io::Result<U> {
        unimplemented!()
    }
}
