use std::{backtrace::Backtrace, fmt::Debug};

pub struct NBTError {
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub kind: NBTErrorKind,
    pub backtrace: Backtrace,
}

#[derive(Debug)]
pub enum NBTErrorKind {
    IO,
    UnexpectedEOF,
    FromUTF8,
    InvalidTagID(u8),
    InvalidStringLength(usize),
    InvalidFormat,
    Custom(String),
}

pub type Result<T> = std::result::Result<T, NBTError>;

impl std::fmt::Display for NBTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)?;
        write!(f, ": {:?}", &self.source)?;

        write!(f, "\nBacktrace:\n{:?}", self.backtrace)
    }
}

impl std::fmt::Debug for NBTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)?;
        write!(f, ": {:?}", &self.source)?;

        write!(f, "\nBacktrace:\n{:?}", self.backtrace)
    }
}

impl std::error::Error for NBTError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| &**s as &dyn std::error::Error)
    }
}

impl NBTError {
    fn new(source: Box<dyn std::error::Error + Send + Sync>, kind: NBTErrorKind) -> Self {
        Self {
            source: Some(source),
            kind,
            backtrace: Backtrace::force_capture(),
        }
    }

    fn no_source(kind: NBTErrorKind) -> Self {
        Self {
            source: None,
            kind,
            backtrace: Backtrace::force_capture(),
        }
    }

    pub fn io(source: std::io::Error) -> Self {
        Self::new(Box::new(source), NBTErrorKind::IO)
    }

    pub fn unexpected_eof() -> Self {
        Self::no_source(NBTErrorKind::UnexpectedEOF)
    }

    pub fn from_utf8(source: std::string::FromUtf8Error) -> Self {
        Self::new(Box::new(source), NBTErrorKind::FromUTF8)
    }

    pub fn invalid_tag_id(id: u8) -> Self {
        Self::no_source(NBTErrorKind::InvalidTagID(id))
    }

    pub fn invalid_string_length(len: usize) -> Self {
        Self::no_source(NBTErrorKind::InvalidStringLength(len))
    }

    pub fn custom_msg<S: Into<String>>(msg: S) -> Self {
        Self::no_source(NBTErrorKind::Custom(msg.into()))
    }
}

impl From<std::io::Error> for NBTError {
    fn from(source: std::io::Error) -> Self {
        Self::io(source)
    }
}

impl From<std::string::FromUtf8Error> for NBTError {
    fn from(source: std::string::FromUtf8Error) -> Self {
        Self::from_utf8(source)
    }
}
