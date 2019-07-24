use super::{DBusBasicType, DBusType};
use rbus_derive::impl_type;
use std::{collections::HashMap, hash::Hash, iter::FromIterator, ops::Deref};

// Dict (list of dict entries)
#[derive(Debug, Clone, PartialEq)]
pub struct DictEntry<K, V>(K, V);

impl<K, V> DictEntry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        DictEntry(key, value)
    }

    pub fn key(&self) -> &K {
        &self.0
    }

    pub fn value(&self) -> &V {
        &self.1
    }
}

impl<K, V> From<(K, V)> for DictEntry<K, V> {
    fn from((key, value): (K, V)) -> DictEntry<K, V> {
        DictEntry(key, value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dict<K, V>(Vec<DictEntry<K, V>>);

impl<K, V> Dict<K, V> {
    pub fn into_hashmap(self) -> HashMap<K, V>
    where
        K: Eq + Hash,
    {
        self.0.into_iter().map(|entry| (entry.0, entry.1)).collect()
    }
}

impl<K, V> Deref for Dict<K, V> {
    type Target = [DictEntry<K, V>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<K, V> From<Vec<DictEntry<K, V>>> for Dict<K, V> {
    fn from(value: Vec<DictEntry<K, V>>) -> Dict<K, V> {
        Dict(value)
    }
}

impl<K, V> From<Vec<(K, V)>> for Dict<K, V> {
    fn from(value: Vec<(K, V)>) -> Dict<K, V> {
        Dict(value.into_iter().map(From::from).collect())
    }
}

impl<K, V> FromIterator<DictEntry<K, V>> for Dict<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = DictEntry<K, V>>,
    {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl<K, V> FromIterator<(K, V)> for Dict<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl_type! {
    #[dbus(align = 8, module = "crate")]
    impl<K: DBusBasicType, V: DBusType> DictEntry<K, V>: 'e' {
        signature() {
            format!("{{{}{}}}", K::signature(), V::signature())
        }

        encode(marshaller) {
            marshaller.write_padding(Self::alignment())?;
            self.0.encode(marshaller)?;
            marshaller.write_padding(Self::alignment())?;
            self.1.encode(marshaller)?;
            Ok(())
        }

        decode(marshaller) {
            marshaller.read_padding(Self::alignment())?;
            let key = K::decode(marshaller)?;
            marshaller.read_padding(Self::alignment())?;
            let value = V::decode(marshaller)?;

            Ok(DictEntry(key, value))
        }
    }
}

impl_type! {
    #[dbus(align = 8, module = "crate")]
    impl<K: DBusBasicType, V: DBusType> Dict<K, V>: 'e' {
        signature() {
            <Vec<DictEntry<K, V>>>::signature()
        }

        encode(marshaller) {
            self.0.encode(marshaller)
        }

        decode(marshaller) {
            let values = <Vec<DictEntry<K, V>>>::decode(marshaller)?;
            Ok(Dict(values))
        }
    }
}

// HashMap
impl_type! {
    #[dbus(align = 8, module = "crate")]
    impl<K, V> HashMap<K, V>: 'e'
    where
        K: DBusBasicType + Eq + Hash,
        V: DBusType,
    {
        signature() {
            <Dict<K, V>>::signature()
        }

        encode(marshaller) {
            let data = self.iter().collect::<Dict<_, _>>();
            data.encode(marshaller)
        }

        decode(marshaller) {
            let dict = <Dict<_, _>>::decode(marshaller)?;
            Ok(dict.into_hashmap())
        }
    }
}
