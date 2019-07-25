use super::{impl_type, DBusType};

macro_rules! impl_tuple_dbus_type {
    ($($index:tt: $ty:ident),*) => {
        impl_type! {
            #[dbus(align = 1)]
            impl<$($ty: DBusType),*> ($($ty,)*) {
                signature() {
                    let signatures: &[String] = &[$(<$ty>::signature()),*];
                    signatures.concat()
                }

                #[allow(unused_variables)]
                encode(marshaller) {
                    $(marshaller.write_value(&self.$index)?;)*
                    Ok(())
                }

                #[allow(unused_variables)]
                decode(marshaller) {
                    Ok(($(marshaller.read_value::<$ty>()?,)*))
                }
            }
        }
    }
}

impl_tuple_dbus_type!();
impl_tuple_dbus_type!(0: A);
impl_tuple_dbus_type!(0: A, 1: B);
impl_tuple_dbus_type!(0: A, 1: B, 2: C);
impl_tuple_dbus_type!(0: A, 1: B, 2: C, 3: D);
impl_tuple_dbus_type!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_tuple_dbus_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_tuple_dbus_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_tuple_dbus_type!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);

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
