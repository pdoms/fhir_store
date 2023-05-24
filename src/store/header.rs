use crate::error::Result;

pub trait Head {
    fn from_store(data: &[u8]) -> Self;
    fn to_store(&self) -> Result<Vec<u8>>;
}
