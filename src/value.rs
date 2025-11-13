use std::{borrow::Cow, collections::BTreeMap};

use crate::{
    error::{NBTError, Result},
    tag::Tag,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(Cow<'a, str>),
    List(Vec<Value<'a>>),
    Compound(BTreeMap<Cow<'a, str>, Value<'a>>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl<'a> Value<'a> {
    pub fn tag(&self) -> Tag {
        match self {
            Value::End => Tag::End,
            Value::Byte(_) => Tag::Byte,
            Value::Short(_) => Tag::Short,
            Value::Int(_) => Tag::Int,
            Value::Long(_) => Tag::Long,
            Value::Float(_) => Tag::Float,
            Value::Double(_) => Tag::Double,
            Value::ByteArray(_) => Tag::ByteArray,
            Value::String(_) => Tag::String,
            Value::List(_) => Tag::List,
            Value::Compound(_) => Tag::Compound,
            Value::IntArray(_) => Tag::IntArray,
            Value::LongArray(_) => Tag::LongArray,
        }
    }

    pub fn compound() -> Self {
        Value::Compound(BTreeMap::new())
    }

    pub fn insert<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Value<'a>>,
    {
        match self {
            Value::Compound(map) => {
                map.insert(key.into(), value.into());
                Ok(())
            }
            _ => Err(NBTError::custom_msg("Not a compound")),
        }
    }

    pub fn list(capacity: usize) -> Self {
        Value::List(Vec::with_capacity(capacity))
    }

    pub fn push<V: Into<Value<'a>>>(&mut self, value: V) -> Result<()> {
        match self {
            Value::List(vec) => {
                vec.push(value.into());
                Ok(())
            }
            _ => Err(NBTError::custom_msg("Not a list")),
        }
    }

    pub fn extend<I, V>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = V>,
        V: Into<Value<'a>>,
    {
        match self {
            Value::List(vec) => {
                vec.extend(iter.into_iter().map(Into::into));
                Ok(())
            }
            _ => Err(NBTError::custom_msg("Not a list")),
        }
    }

    pub fn list_from_iter<I, V>(iter: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Self>,
    {
        Value::List(iter.into_iter().map(Into::into).collect())
    }

    pub fn list_tag(&self) -> Option<Tag> {
        match self {
            Value::List(vec) if !vec.is_empty() => Some(vec[0].tag()),
            Value::List(_) => None,
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value<'a>>> {
        match self {
            Value::List(vec) => Some(vec),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Value::List(vec) => Some(vec),
            _ => None,
        }
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Value::Byte(value as i8)
    }
}

impl<'a> From<i16> for Value<'a> {
    fn from(value: i16) -> Self {
        Value::Short(value)
    }
}

impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl<'a> From<&'a [i32]> for Value<'a> {
    fn from(value: &'a [i32]) -> Self {
        Value::IntArray(value.to_vec())
    }
}

impl<'a> From<i64> for Value<'a> {
    fn from(value: i64) -> Self {
        Value::Long(value)
    }
}

impl<'a> From<&'a [i64]> for Value<'a> {
    fn from(value: &'a [i64]) -> Self {
        Value::LongArray(value.to_vec())
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Value::String(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Value::String(Cow::Borrowed(value))
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Value::String(value)
    }
}

impl<'a, K> From<BTreeMap<K, Value<'a>>> for Value<'a>
where
    K: Into<Cow<'a, str>>,
{
    fn from(map: BTreeMap<K, Value<'a>>) -> Self {
        Value::Compound(map.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}
