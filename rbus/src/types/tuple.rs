use super::DBusType;
use crate::marshal::Marshaller;
use crate::Result;
use std::io;

pub trait DBusPackedTypes: Sized {
    fn signature() -> String;

    fn encode<Inner>(&self, marshaller: &mut Marshaller<Inner>) -> Result<()>
    where
        Inner: AsRef<[u8]> + AsMut<[u8]> + io::Write;
    fn decode<Inner>(marshaller: &mut Marshaller<Inner>) -> Result<Self>
    where
        Inner: AsRef<[u8]> + io::Read;
}

macro_rules! impl_tuple_packed_type {
    ($($index:tt: $ty:ident),*) => {
        impl<$($ty: DBusType),*> DBusPackedTypes for ($($ty),*,) {
            fn signature() -> String {
                [$($ty::signature()),*].concat()
            }

            fn encode<Inner>(&self, marshaller: &mut Marshaller<Inner>) -> Result<()>
            where
                Inner: AsRef<[u8]> + AsMut<[u8]> + io::Write
            {
                $(marshaller.write_value(&self.$index)?;)*
                Ok(())
            }

            fn decode<Inner>(marshaller: &mut Marshaller<Inner>) -> Result<Self>
            where
                Inner: AsRef<[u8]> + io::Read
            {
                Ok(($(marshaller.read_value::<$ty>()?),*,))
            }
        }
    }
}

impl_tuple_packed_type!(0: A);
impl_tuple_packed_type!(0: A, 1: B);
impl_tuple_packed_type!(0: A, 1: B, 2: C);
impl_tuple_packed_type!(0: A, 1: B, 2: C, 3: D);
impl_tuple_packed_type!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_tuple_packed_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_tuple_packed_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_tuple_packed_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_simple_packed1() {
        assert_eq!(<(u64,)>::signature(), "t");
    }

    #[test]
    fn test_signature_struct_packed() {
        #[derive(DBusType)]
        #[dbus(module = "crate")]
        struct Arg(u8, String);

        assert_eq!(<(u32, Arg)>::signature(), "u(ys)");
    }
}
