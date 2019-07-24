use std::io;

pub struct Cursor<T> {
    inner: T,
    write_pos: usize,
    read_pos: usize,
}

impl<T> Cursor<T> {
    pub fn new(inner: T) -> Cursor<T> {
        Cursor {
            inner,
            write_pos: 0,
            read_pos: 0,
        }
    }

    #[inline]
    fn padding(offset: usize, alignment: usize) -> usize {
        (alignment - (offset % alignment)) % alignment
    }

    #[inline]
    pub fn write_position(&self) -> usize {
        self.write_pos
    }

    #[inline]
    pub fn write_padding(&self, alignment: usize) -> usize {
        Self::padding(self.write_pos, alignment)
    }

    #[inline]
    pub fn read_position(&self) -> usize {
        self.read_pos
    }

    #[inline]
    pub fn read_padding(&self, alignment: usize) -> usize {
        Self::padding(self.read_pos, alignment)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }

    #[inline]
    pub fn reset(&mut self) {
        self.reset_write();
        self.reset_read();
    }

    #[inline]
    pub fn reset_write(&mut self) {
        self.write_pos = 0;
    }

    #[inline]
    pub fn reset_read(&mut self) {
        self.read_pos = 0;
    }

    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> io::Write for Cursor<T>
where
    T: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner.write(buf)?;
        self.write_pos = self.write_pos.wrapping_add(res);
        Ok(res)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T> io::Read for Cursor<T>
where
    T: io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.inner.read(buf)?;
        self.read_pos = self.read_pos.wrapping_add(res);
        Ok(res)
    }
}
