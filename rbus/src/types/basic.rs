use rbus_derive::impl_basic_type;

impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    u8: 'y'
}
impl_basic_type! {
    #[dbus(align = 4, module = "crate")]
    bool: 'b' {
        encode(marshaller) {
            marshaller.io().write_u32(*self as u32)?;
            Ok(())
        }

        decode(marshaller) {
            let value = marshaller.io().read_u32()?;
            Ok(value != 0)
        }
    }
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    i16: 'n'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    u16: 'q'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    i32: 'i'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    u32: 'u'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    i64: 'x'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    u64: 't'
}
impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    f64: 'd'
}

// Unix FD
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnixFd(pub u32);

impl_basic_type! {
    #[dbus(align = "size", module = "crate")]
    UnixFd: 'h' {
        encode(marshaller) {
            marshaller.io().write_u32(self.0);
            Ok(())
        }

        decode(marshaller) {
            let value = marshaller.io().read_u32()?;
            Ok(UnixFd(value))
        }
    }
}
