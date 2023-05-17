pub mod resource_reader;

use crate::error::{Result, Error};
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum ResourceId {
    Empty,
    Patient
}


impl TryFrom<&str> for ResourceId {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "patient" => Ok(ResourceId::Patient),
            _ => Err(Error::UnknownResourceStr(value.to_string()))
        }
    }
}

impl TryFrom<u16> for ResourceId {
    type Error = Error;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            0 => Ok(ResourceId::Empty),
            1 => Ok(ResourceId::Patient),
            _ => Err(Error::UnknownResourceId(value))
        }
    }
}


