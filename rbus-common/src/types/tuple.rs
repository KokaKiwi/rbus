use super::DBusType;

pub trait DBusPackedTypes {
    fn signature() -> String;
}

macro_rules! impl_tuple_type {
    ($($index:tt: $ty:ident),*) => {
        impl<$($ty: DBusType),*> DBusPackedTypes for ($($ty),*,) {
            fn signature() -> String {
                [$($ty::signature()),*].concat()
            }
        }
    }
}

impl_tuple_type!(0: A);
impl_tuple_type!(0: A, 1: B);
impl_tuple_type!(0: A, 2: B, 3: C);
impl_tuple_type!(0: A, 2: B, 3: C, 4: D);
impl_tuple_type!(0: A, 2: B, 3: C, 4: D, 5: E);
impl_tuple_type!(0: A, 2: B, 3: C, 4: D, 5: E, 6: F);
impl_tuple_type!(0: A, 2: B, 3: C, 4: D, 5: E, 6: F, 7: G);
impl_tuple_type!(0: A, 2: B, 3: C, 4: D, 5: E, 6: F, 7: G, 8: H);

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
        #[dbus(type_path = "crate::types::DBusType")]
        struct Arg(u8, String);

        assert_eq!(<(u32, Arg)>::signature(), "u(ys)");
    }
}
