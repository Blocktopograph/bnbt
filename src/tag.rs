#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Tag {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

impl TryFrom<u8> for Tag {
    type Error = crate::error::NBTError;

    fn try_from(value: u8) -> crate::error::Result<Self> {
        match value {
            0 => Ok(Tag::End),
            1 => Ok(Tag::Byte),
            2 => Ok(Tag::Short),
            3 => Ok(Tag::Int),
            4 => Ok(Tag::Long),
            5 => Ok(Tag::Float),
            6 => Ok(Tag::Double),
            7 => Ok(Tag::ByteArray),
            8 => Ok(Tag::String),
            9 => Ok(Tag::List),
            10 => Ok(Tag::Compound),
            11 => Ok(Tag::IntArray),
            12 => Ok(Tag::LongArray),
            id => Err(crate::error::NBTError::invalid_tag_id(id)),
        }
    }
}
