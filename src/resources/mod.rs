use crate::error::{Result, Error};
#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ResourceId {
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


