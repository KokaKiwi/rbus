use super::DBusType;

pub trait PackedDBusTypes {
    fn signature() -> String;
}

macro_rules! impl_tuple_type {
    ($($index:tt: $ty:ident),*) => {
        impl<$($ty: DBusType),*> PackedDBusTypes for ($($ty),*,) {
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
