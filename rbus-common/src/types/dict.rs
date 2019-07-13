use super::{DBusBasicType, DBusType};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

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

impl<K: DBusBasicType, V: DBusType> DBusType for Dict<K, V> {
    fn code() -> u8 {
        b'e'
    }
    fn signature() -> String {
        format!("a{{{}{}}}", K::signature(), V::signature())
    }
    fn alignment() -> u8 {
        8
    }
}

// HashMap
impl<K: DBusBasicType, V: DBusType, S: std::hash::BuildHasher> DBusType for HashMap<K, V, S> {
    fn code() -> u8 {
        b'e'
    }
    fn signature() -> String {
        <Dict<K, V>>::signature()
    }
    fn alignment() -> u8 {
        8
    }
}
