use paste::paste;

use crate::{
    error::{NBTError, Result},
    tag::Tag,
    value::Value,
};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{Read, Write},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Endian {
    #[default]
    Big,
    Little,
}

#[derive(Debug, Clone, Default)]
pub struct NBTCodec {
    pub endian: Endian,
}

impl NBTCodec {
    pub fn new(endian: Endian) -> Self {
        Self { endian }
    }

    pub fn big_endian() -> Self {
        Self::new(Endian::Big)
    }

    pub fn little_endian() -> Self {
        Self::new(Endian::Little)
    }
}

macro_rules! gen_nbt_codec_trait {
    ($($name:ident: $read_ty:ty, $write_ty:ty);* $(;)?) => {
        $(gen_nbt_codec_trait!(@internal $name, $read_ty, $write_ty);)*
    };
    ($($name:ty: $read_ty:ty, $write_ty:ty);* $(;)?) => {
        $(gen_nbt_codec_trait!(@internal $name, $read_ty, $write_ty);)*
    };

    (@internal $name:ty, $read_ty:ty, $write_ty:ty) => {
        paste! {
            fn [<read_ $name>]<R: Read>(&self, reader: &mut R) -> Result<$read_ty>;
            fn [<write_ $name>]<W: Write>(&self, writer: &mut W, value: $write_ty) -> Result<()>;
        }
    };
}

macro_rules! gen_simple {
    ($($ty:ty),*) => {
        $(gen_nbt_codec_trait!($ty: $ty, $ty);)*
    };
}

pub trait NBTCodecTrait {
    fn read_tag<R: Read>(&self, reader: &mut R) -> Result<(Option<Cow<'_, str>>, Value<'_>)>;

    fn write_tag<W: Write>(
        &self,
        writer: &mut W,
        name: Option<Cow<'_, str>>,
        value: &Value<'_>,
    ) -> Result<()>;

    fn read_value<R: Read>(&self, reader: &mut R, tag: &Tag) -> Result<Value<'_>>;

    fn write_value<W: Write>(&self, writer: &mut W, value: &Value<'_>) -> Result<()>;

    gen_nbt_codec_trait!(
        string: String, &str;
        list: Value<'_>, &Value<'_>;
        compound: Value<'_>, &Value<'_>;
        byte_array: Vec<i8>, &[i8];
        int_array: Vec<i32>, &[i32];
        long_array: Vec<i64>, &[i64];
    );

    gen_simple!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);
}

macro_rules! gen_nbt_codec_impl {
    ($($name:ty: $read_ty:ty, $write_ty:ty);* $(;)?) => {
        $(
            paste! {
                fn [<read_ $name>]<R: Read>(&self, reader: &mut R) -> Result<$read_ty> {
                    let mut buf = [0u8; std::mem::size_of::<$read_ty>()];
                    reader.read_exact(&mut buf)?;

                    match self.endian {
                        Endian::Big => Ok($read_ty::from_be_bytes(buf)),
                        Endian::Little => Ok($read_ty::from_le_bytes(buf)),
                    }
                }

                fn [<write_ $name>]<W: Write>(&self, writer: &mut W, value: $write_ty) -> Result<()> {
                    let buf = match self.endian {
                        Endian::Big => $write_ty::to_be_bytes(value),
                        Endian::Little => $write_ty::to_le_bytes(value),
                    };

                    writer.write_all(&buf)?;

                    Ok(())
                }
            }
        )*
    };
}

macro_rules! gen_simple_impl {
    ($($ty:ident),* $(,)?) => {
        gen_nbt_codec_impl! {$($ty: $ty, $ty;)*}
    };
}

impl NBTCodecTrait for NBTCodec {
    fn read_tag<R: Read>(&self, reader: &mut R) -> Result<(Option<Cow<'_, str>>, Value<'_>)> {
        let tag = Tag::try_from(self.read_u8(reader)?)?;

        let name = self.read_string(reader)?;

        let name_opt = if !name.is_empty() {
            Some(Cow::Owned(name))
        } else {
            None
        };

        let value = self.read_value(reader, &tag)?;

        Ok((name_opt, value))
    }

    fn write_tag<W: Write>(
        &self,
        writer: &mut W,
        name: Option<Cow<'_, str>>,
        value: &Value<'_>,
    ) -> Result<()> {
        self.write_u8(writer, value.tag() as u8)?;

        let wraped_name = match name {
            Some(n) => n.into_owned(),
            None => String::new(),
        };

        self.write_string(writer, &wraped_name)?;

        self.write_value(writer, value)?;

        Ok(())
    }

    fn read_value<R: Read>(&self, reader: &mut R, tag: &Tag) -> Result<Value<'_>> {
        match tag {
            Tag::End => Ok(Value::End),
            Tag::Byte => Ok(Value::Byte(self.read_i8(reader)?)),
            Tag::Short => Ok(Value::Short(self.read_i16(reader)?)),
            Tag::Int => Ok(Value::Int(self.read_i32(reader)?)),
            Tag::Long => Ok(Value::Long(self.read_i64(reader)?)),
            Tag::Float => Ok(Value::Float(self.read_f32(reader)?)),
            Tag::Double => Ok(Value::Double(self.read_f64(reader)?)),
            Tag::ByteArray => Ok(Value::ByteArray(self.read_byte_array(reader)?)),
            Tag::String => Ok(Value::String(Cow::Owned(self.read_string(reader)?))),
            Tag::List => Ok(self.read_list(reader)?),
            Tag::Compound => Ok(self.read_compound(reader)?),
            Tag::IntArray => Ok(Value::IntArray(self.read_int_array(reader)?)),
            Tag::LongArray => Ok(Value::LongArray(self.read_long_array(reader)?)),
        }
    }

    fn write_value<W: Write>(&self, writer: &mut W, value: &Value<'_>) -> Result<()> {
        match value {
            Value::End => Ok(()),
            Value::Byte(v) => self.write_i8(writer, *v),
            Value::Short(v) => self.write_i16(writer, *v),
            Value::Int(v) => self.write_i32(writer, *v),
            Value::Long(v) => self.write_i64(writer, *v),
            Value::Float(v) => self.write_f32(writer, *v),
            Value::Double(v) => self.write_f64(writer, *v),
            Value::ByteArray(v) => self.write_byte_array(writer, v),
            Value::String(v) => self.write_string(writer, v.as_ref()),
            Value::IntArray(v) => self.write_int_array(writer, v),
            Value::LongArray(v) => self.write_long_array(writer, v),
            Value::List(_) => self.write_list(writer, value),
            Value::Compound(_) => self.write_compound(writer, value),
        }
    }

    fn read_string<R: Read>(&self, reader: &mut R) -> Result<String> {
        let length = self.read_u16(reader)?;

        let mut buf = vec![0u8; length as usize];
        reader.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }

    fn write_string<W: Write>(&self, writer: &mut W, value: &str) -> Result<()> {
        self.write_u16(writer, value.len() as u16)?;
        writer.write_all(value.as_bytes())?;
        Ok(())
    }

    fn read_list<R: Read>(&self, reader: &mut R) -> Result<Value<'_>> {
        let element_tag_id = self.read_i8(reader)?;
        let element_tag = Tag::try_from(element_tag_id as u8)?;
        let length = self.read_i32(reader)?;

        if length < 0 || length > i16::MAX as i32 {
            return Err(NBTError::invalid_string_length(length as usize));
        }

        let mut list = Vec::with_capacity(length as usize);
        for _ in 0..length {
            list.push(self.read_value(reader, &element_tag)?);
        }

        Ok(Value::List(list))
    }

    fn write_list<W: Write>(&self, writer: &mut W, value: &Value<'_>) -> Result<()> {
        if let Value::List(list) = value {
            if list.is_empty() {
                self.write_i8(writer, Tag::End as i8)?;
                self.write_i32(writer, 0)?;
                return Ok(());
            }

            let first_tag = list[0].tag();
            for (i, value) in list.iter().enumerate() {
                if value.tag() != first_tag {
                    return Err(NBTError::custom_msg(format!(
                        "List type mismatch at index {}: expected {:?}, got {:?}",
                        i,
                        first_tag,
                        value.tag()
                    )));
                }
            }

            self.write_i8(writer, first_tag as i8)?;
            self.write_i32(writer, list.len() as i32)?;

            for value in list {
                self.write_value(writer, value)?;
            }

            return Ok(());
        }

        Err(NBTError::invalid_tag_id(value.tag() as u8))
    }

    fn read_compound<R: Read>(&self, reader: &mut R) -> Result<Value<'_>> {
        let mut compound = BTreeMap::new();

        loop {
            let tag_id = self.read_i8(reader)?;
            let tag = Tag::try_from(tag_id as u8)?;

            if tag == Tag::End {
                break;
            }

            let name = self.read_string(reader)?;
            let value = self.read_value(reader, &tag)?;

            compound.insert(Cow::Owned(name), value);
        }

        Ok(Value::Compound(compound))
    }

    fn write_compound<W: Write>(&self, writer: &mut W, value: &Value<'_>) -> Result<()> {
        let Value::Compound(map) = value else {
            return Err(NBTError::invalid_tag_id(value.tag() as u8));
        };

        for (name, val) in map {
            self.write_i8(writer, val.tag() as i8)?;

            self.write_string(writer, name.as_ref())?;

            self.write_value(writer, val)?;
        }

        self.write_i8(writer, Tag::End as i8)?;
        Ok(())
    }

    fn read_byte_array<R: Read>(&self, reader: &mut R) -> Result<Vec<i8>> {
        let size = self.read_u32(reader)? as usize;
        let mut buf = vec![0u8; size];
        reader.read_exact(&mut buf)?;
        Ok(buf.into_iter().map(|b| b as i8).collect())
    }

    fn write_byte_array<W: Write>(&self, writer: &mut W, value: &[i8]) -> Result<()> {
        self.write_u32(writer, value.len() as u32)?;

        let bytes: Vec<u8> = value.iter().map(|&b| b as u8).collect();
        writer.write_all(&bytes)?;

        Ok(())
    }

    fn read_int_array<R: Read>(&self, reader: &mut R) -> Result<Vec<i32>> {
        let size = self.read_u32(reader)? as usize;
        (0..size).map(|_| self.read_i32(reader)).collect()
    }

    fn write_int_array<W: Write>(&self, writer: &mut W, value: &[i32]) -> Result<()> {
        self.write_u32(writer, value.len() as u32)?;

        const CHUNK_SIZE: usize = 1024;

        for chunk in value.chunks(CHUNK_SIZE) {
            for &val in chunk {
                self.write_i32(writer, val)?;
            }
        }

        Ok(())
    }

    fn read_long_array<R: Read>(&self, reader: &mut R) -> Result<Vec<i64>> {
        let size = self.read_u32(reader)? as usize;
        (0..size).map(|_| self.read_i64(reader)).collect()
    }

    fn write_long_array<W: Write>(&self, writer: &mut W, value: &[i64]) -> Result<()> {
        self.write_u32(writer, value.len() as u32)?;

        const CHUNK_SIZE: usize = 512;

        for chunk in value.chunks(CHUNK_SIZE) {
            for &val in chunk {
                self.write_i64(writer, val)?;
            }
        }

        Ok(())
    }

    gen_simple_impl!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);
}

mod tests {

    #[test]
    fn feature() {
        use crate::codec::{NBTCodec, NBTCodecTrait};
        use crate::value::Value;
        use std::io::{BufReader, BufWriter};

        let a = NBTCodec::little_endian();

        let mut map = Value::compound();
        map.insert("BSTVVL", Value::list_from_iter(vec![6969, 696969]))
            .unwrap();

        let buf = Vec::with_capacity(14);
        let mut writer = BufWriter::new(buf);

        a.write_tag(&mut writer, None, &map).unwrap();

        println!("{:?}", writer.buffer());

        let mut reader = BufReader::new(writer.buffer());

        let b = a.read_tag(&mut reader);
        println!("{:?}", b);
    }
}
