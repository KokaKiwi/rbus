use super::DBusType;

impl<T: DBusType> DBusType for Vec<T> {
    fn code() -> u8 { b'a' }
    fn signature() -> String { format!("a{}", T::signature()) }
}
