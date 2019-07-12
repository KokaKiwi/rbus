use rbus_derive::impl_basic_type;

impl_basic_type!(u8, 'y');
impl_basic_type!(bool, 'b');
impl_basic_type!(i16, 'n');
impl_basic_type!(u16, 'q');
impl_basic_type!(i32, 'i');
impl_basic_type!(u32, 'u');
impl_basic_type!(i64, 'x');
impl_basic_type!(u64, 't');
impl_basic_type!(f64, 'd');

// Unix FD
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnixFd(pub u32);

impl_basic_type!(UnixFd, 'h');
