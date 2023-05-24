use std::fmt::{self, Display};
use std;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    Expected(String, String),
    UnknownStoreId(u16),
    UnknownResourceStr(String),
    UnknownResourceId(u16),
    UnknownSyntaxToken(u8),
    UnknownKeyInJson(String),
    MemoryAllocation,
    LayoutSetting,
    BufferOverflow,
    BufferUnderflow,
    SegmentationFault,
    IdMaxLen,
    StoreUnitMaxLen,
    EOF,

}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg)             => formatter.write_str(msg),
            Error::Expected(exp, got)      => formatter.write_fmt(format_args!("PARSING: expected '{exp}' got '{got}'")),
            Error::UnknownStoreId(id)       => formatter.write_fmt(format_args!("CONVERSION: unknown StoreId: '{id}'")),
            Error::UnknownResourceStr(rec) => formatter.write_fmt(format_args!("CONVERSION: unknown ResourceId string: '{rec}'")),
            Error::UnknownResourceId(rec)  => formatter.write_fmt(format_args!("CONVERSION: unknown ResourceId: '{rec}'")),
            Error::UnknownSyntaxToken(tkn) => formatter.write_fmt(format_args!("PARSING: unknown syntax token '{}'", *tkn as char)),
            Error::UnknownKeyInJson(k)     => formatter.write_fmt(format_args!("PARSING: unknown key in json '{k}'")),
            Error::MemoryAllocation        => formatter.write_str("MEMORY: error allocating memory"),
            Error::LayoutSetting           => formatter.write_str("MEMORY: error setting layout"),
            Error::BufferOverflow          => formatter.write_str("MEMORY: buffer overflow"),
            Error::BufferUnderflow         => formatter.write_str("MEMORY: buffer underflow"),
            Error::SegmentationFault       => formatter.write_str("MEMORY: segfault"),
            Error::IdMaxLen                => formatter.write_str("CONVERSION: id max length is 64 characters"),
            Error::StoreUnitMaxLen         => formatter.write_fmt(
                format_args!("CONVERSION: store unit max length of {} reached", u16::MAX)),
            Error::EOF                     => formatter.write_str("PARSING: unexpected end of input"),
        }
    }
}

impl std::error::Error for Error {}
