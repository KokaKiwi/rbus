use super::DBusType;
use crate::marshal::Marshaller;
use crate::Result;
use std::io;

pub trait DBusPackedTypes: Sized {
    fn signature() -> String;

    fn encode<T: AsRef<[u8]> + io::Write>(&self, marshaller: &mut Marshaller<T>) -> Result<()>;
    fn decode<T: AsRef<[u8]> + io::Read>(marshaller: &mut Marshaller<T>) -> Result<Self>;
}

macro_rules! impl_tuple_type {
    ($($index:tt: $ty:ident),*) => {
        impl<$($ty: DBusType),*> DBusPackedTypes for ($($ty),*,) {
            fn signature() -> String {
                [$($ty::signature()),*].concat()
            }

            fn encode<T: AsRef<[u8]> + io::Write>(&self, marshaller: &mut Marshaller<T>) -> Result<()> {
                $(self.$index.encode(marshaller)?;)*
                Ok(())
            }

            fn decode<T: AsRef<[u8]> + io::Read>(marshaller: &mut Marshaller<T>) -> Result<Self> {
                Ok(($($ty::decode(marshaller)?),*,))
            }
        }
    }
}

impl_tuple_type!(0: A);
impl_tuple_type!(0: A, 1: B);
impl_tuple_type!(0: A, 1: B, 2: C);
impl_tuple_type!(0: A, 1: B, 2: C, 3: D);
impl_tuple_type!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_tuple_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_tuple_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_tuple_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);

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
