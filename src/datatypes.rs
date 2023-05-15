use phf::phf_map;

use crate::error::{Error, Result};

const ID_MAX_LEN: usize = 64;

pub type BU16 = [u8; 2];
pub type BYTES = Box<[u8]>;

/// DataType holds value, DataTypeId and size in be_bytes. 
#[derive(Debug)]
pub enum DataType {
    MISC(BYTES, u16, u16),
    BASE64BINARY(BYTES, u16, u16),
    BOOL(BYTES, u16, u16),
    CANONICAL(BYTES, u16, u16),
    CODE(BYTES, u16, u16),
    DATE(BYTES, u16, u16),
    DATETIME(BYTES, u16, u16),
    DEC(BYTES, u16, u16),
    ID(BYTES, u16, u16),
    INSTANT(BYTES, u16, u16),
    INTEGER(BYTES, u16, u16),
    INTEGER64(BYTES, u16, u16),
    MARKDOWN(BYTES, u16, u16),
    OID(BYTES, u16, u16),
    STRING(BYTES, u16, u16),
    POSITIVEINT(BYTES, u16, u16),
    TIME(BYTES, u16, u16),
    URI(BYTES, u16, u16),
    URL(BYTES, u16, u16),
    UUID(BYTES, u16, u16),
    UNSIGNEDINT(BYTES, u16, u16),
    KEY(BYTES, u16, u16),
    KEYID(BYTES, u16, u16),
    Identifier,
    HUMANNAME,
    LIST,
    OBJ,
}

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum DataId {
    MISC,
    BASE64BINARY,
    BOOL,
    CANONICAL,
    CODE,
    DATE,
    DATETIME,
    DEC,
    ID,
    INSTANT,
    INTEGER,
    INTEGER64,
    MARKDOWN,
    OID,
    STRING,
    POSITIVEINT,
    TIME,
    URI,
    URL,
    UUID,
    UNSIGNEDINT,
    KEY,
    KEYID,
    Identifier,
    HUMANNAME,
    LIST,
    OBJ,
}




impl DataId {
    pub fn as_bytes(self) -> [u8; 2] {
        (self as u16).to_be_bytes()
    }
}

impl TryFrom<u16> for DataId {
    type Error = Error;
    fn try_from(i: u16) -> Result<DataId> {
        match i {
            0  => Ok(DataId::MISC),
            1  => Ok(DataId::BASE64BINARY),
            2  => Ok(DataId::BOOL),
            3  => Ok(DataId::CANONICAL),
            4  => Ok(DataId::CODE),
            5  => Ok(DataId::DATE),
            7  => Ok(DataId::DATETIME),
            8  => Ok(DataId::DEC),
            9  => Ok(DataId::ID),
            10 => Ok(DataId::INSTANT),
            11 => Ok(DataId::INTEGER),
            12 => Ok(DataId::INTEGER64),
            13 => Ok(DataId::MARKDOWN),
            14 => Ok(DataId::OID),
            15 => Ok(DataId::STRING),
            16 => Ok(DataId::POSITIVEINT),
            17 => Ok(DataId::TIME),
            18 => Ok(DataId::URI),
            19 => Ok(DataId::URL),
            20 => Ok(DataId::UUID),
            21 => Ok(DataId::UNSIGNEDINT),
            22 => Ok(DataId::KEY),
            23 => Ok(DataId::KEYID),
            24 => Ok(DataId::LIST),
            26 => Ok(DataId::OBJ),
            27 => Ok(DataId::Identifier),
            28 => Ok(DataId::HUMANNAME),
             _ => Err(self::Error::UnknownDataId(i))
        }
    }
}

pub trait Store {
    fn to_store(self) -> DataType;
}

pub trait StoreWith {
    fn to_store_with(self, id: DataId) -> Result<DataType>;
}

impl Store for bool {
    fn to_store(self) -> DataType {
        let size =get_fixed_size::<bool>();
        let id: u16 = DataId::BOOL as u16;
        match self {
            true => DataType::BOOL(Box::new([1]), id, size),
            false => DataType::BOOL(Box::new([0]), id, size),
        }
    }
}


impl Store for f64 {
    fn to_store(self) -> DataType {
        DataType::DEC(Box::from(self.to_be_bytes()), DataId::DEC as u16, get_fixed_size::<u64>())
    }
}


impl Store for i32 {
    fn to_store(self) -> DataType {
        DataType::UNSIGNEDINT(Box::from(self.to_be_bytes()), DataId::INTEGER as u16, get_fixed_size::<i32>())
    }
}
impl Store for i64 {
    fn to_store(self) -> DataType {
        DataType::UNSIGNEDINT(Box::from(self.to_be_bytes()), DataId::INTEGER64 as u16, get_fixed_size::<i64>())
    }
}

impl Store for u32 {
    fn to_store(self) -> DataType {
        DataType::UNSIGNEDINT(Box::from(self.to_be_bytes()), DataId::UNSIGNEDINT as u16, get_fixed_size::<u32>())
    }
}

impl Store for String {
    fn to_store(self) -> DataType {
        DataType::STRING(Box::from(self.as_bytes()), DataId::STRING as u16, u16::try_from(self.len()).unwrap())
    }
}

impl StoreWith for String {
    fn to_store_with(self, id: DataId) -> Result<DataType> {
        let len =  self.len();
        let length = get_u16(len)?;
        match id {
            DataId::ID => {
                if len > ID_MAX_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DataType::ID(Box::from(self.as_bytes()), DataId::ID as u16, length))
              }
          },
          DataId::KEY => {
                check_u16_max(len)?;
                Ok(DataType::KEY(Box::from(self.as_bytes()), DataId::KEY as u16, length))
          }
          _ => unimplemented!("{id:?} not implemented yet for String")
        }
    }
}

impl StoreWith for &str {
    fn to_store_with(self, id: DataId) -> Result<DataType> {
        let len =  self.len();
        let length = get_u16(len)?;
        match id {
            DataId::ID => {
                if len > ID_MAX_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DataType::ID(Box::from(self.as_bytes()), DataId::ID as u16, length))
              }
          },
          DataId::KEY => {
                check_u16_max(len)?;
                Ok(DataType::KEY(Box::from(self.as_bytes()), DataId::KEY as u16, length))
          },
          DataId::STRING => {
                Ok(DataType::STRING(Box::from(self.as_bytes()), DataId::STRING as u16, length))
          }
          _ => unimplemented!("{id:?} not implemented yet for &str")
        }
    }
}

impl StoreWith for &[u8] {
    fn to_store_with(self, id: DataId) -> Result<DataType> {
        let len =  self.len();
        let length = get_u16(len)?;
        match id {
            DataId::ID => {
                if len > ID_MAX_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DataType::ID(Box::from(self), DataId::ID as u16, length))
              }
          },
          DataId::KEY => {
                check_u16_max(len)?;
                Ok(DataType::KEY(Box::from(self), DataId::KEY as u16, length))
          }
          _ => unimplemented!("{id:?} not implemented yet for &str")
        }
        
    }
}

impl Store for KEY {
    fn to_store(self) -> DataType {
        DataType::KEYID(Box::new((self as u16).to_be_bytes()), DataId::KEYID as u16, get_fixed_size::<u16>())
    }
}

impl StoreWith for Vec<u8> {
    fn to_store_with(self, id: DataId) -> Result<DataType> {
        let len =  self.len();
        let length = get_u16(len)?;
        match id {
            DataId::ID => {
                if len > ID_MAX_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DataType::ID(Box::from(self), DataId::ID as u16, length))
              }
          },
          DataId::KEY => {
                check_u16_max(len)?;
                Ok(DataType::KEY(Box::from(self), DataId::KEY as u16, length))
          },
          DataId::STRING => {
                Ok(DataType::STRING(Box::from(self), DataId::STRING as u16, length))
          }
          _ => unimplemented!("{id:?} not implemented yet for Vec<u8>")
        }
        
    }
}


impl DataType {
    pub fn get_len_adjusted(&self) -> u16 {
        match self {
            DataType::KEY(_, _, len) => len+4,
            DataType::BOOL(_, _, len) => len+4,
            _ => unreachable!("unknown datatype")
        }
    }

    pub fn to_memory(self) -> (Box<[u8]>, u16, u16) {
        match self {
            DataType::KEY(val, id, len) => (val, id, len),
            DataType::KEYID(val, id, len) => (val, id, len),
            DataType::STRING(val, id, len) => (val, id, len),
            DataType::BOOL(val, id, len) => (val, id, len),
            DataType::ID(val, id, len) => (val, id, len),
            _ => unreachable!("unknown datatype {self:?}")
        }
    }
}


fn get_fixed_size_bytes<T: Sized>() -> BU16 {
    (std::mem::size_of::<T>() as u16).to_be_bytes()
}
fn get_fixed_size<T: Sized>() -> u16 {
    std::mem::size_of::<T>() as u16
}

fn get_u16(i: usize) -> Result<u16> {
    match u16::try_from(i) {
        Ok(uint) => Ok(uint),
        Err(err) => Err(Error::Custom(err.to_string()))
    }
}

fn check_u16_max(i: usize) -> Result<()> {
    if i > u16::MAX as usize {
        return Err(Error::StoreUnitMaxLen)
    } 
    Ok(())
}

#[derive(Clone)]
#[repr(u16)]
pub enum KEY {
    Anonymous,
    ResourceType,
    Id,
    Active,
    Identifier,
    Name,
}

static KEYS: phf::Map<&'static str, KEY> = phf_map! {
    "anonymous"    => KEY::Anonymous,
    "resourcetype" => KEY::ResourceType,
    "identifier"   => KEY::Identifier,
    "id"           => KEY::Id,
    "active"       => KEY::Active,
    "name"         => KEY::Name,
};

static EXPECTS: phf::Map<u16, DataId> = phf_map! {
    1u16 => DataId::STRING,     //resourceType
    2u16 => DataId::ID,         //id
    3u16 => DataId::BOOL,       //active
    4u16 => DataId::Identifier, //identifier
    5u16 => DataId::HUMANNAME   //name
};

pub fn get_key(k: &str) -> Option<KEY> {
    KEYS.get(k).cloned()
} 
pub fn get_expects(exp: &KEY) -> Option<DataId> {
    EXPECTS.get(&(*exp as u16)).cloned()
} 









