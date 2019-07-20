use super::DBusType;
use rbus_derive::impl_type;

impl_type! {
    #[dbus(align = 4, module = "crate")]
    impl<T: DBusType> Vec<T>: 'a' {
        signature() {
            format!("a{}", T::signature())
        }

        encode(marshaller) {
            use crate::marshal::Marshaller;

            let mut inner = Marshaller::new(Vec::new(), marshaller.endianness);
            for value in self.iter() {
                inner.write_value(value)?;
            }
            let data = inner.into_vec();

            marshaller.io().write_u32(data.len() as u32)?;
            marshaller.write_padding(T::alignment())?;
            marshaller.io().write_all(&data)?;
            Ok(())
        }

        decode(marshaller) {
            use crate::marshal::Marshaller;

            let length = marshaller.io().read_u32()?;
            marshaller.read_padding(T::alignment())?;

            let mut data = vec![0; length as usize];
            marshaller.io().read_exact(&mut data)?;

            let mut values = vec![];
            let mut inner = Marshaller::new(data.as_slice(), marshaller.endianness);
            while !inner.is_empty() {
                let value = inner.read_value()?;
                values.push(value);
            }

            Ok(values)
        }
    }
}
