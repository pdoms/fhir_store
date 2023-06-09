use crate::error::Result;

pub const PAGE_HEADER_LEN: usize = 72;
pub const STORE_HEADER_LEN: usize = 72;

pub trait Head {
    fn from_store(data: &[u8]) -> Self;
    fn to_store(&self) -> Result<Vec<u8>>;
}
