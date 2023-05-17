use phf::phf_map;

use crate::error::{Error, Result};
use indexmap::map::IndexMap;
const ID_MAX_LEN: usize = 64;
pub const ID_DIV: usize = 256;
pub const PRIM_DIV: usize = 20;

pub const COM: u8 = b',';
pub const SPA: u8 = b' ';
pub const OBO: u8 = b'{';
pub const OBC: u8 = b'}';
const QUO: u8 = b'"';
const COL: u8 = b':';

const KEN: &[u8] = b"\":";
const COS: &[u8] = b", ";


pub type BU16 = [u8; 2];
pub type BYTES = Box<[u8]>;

#[derive(Clone, Debug)]
pub struct Tree {
    map: IndexMap<KEY, DataType>
}

impl Tree {
    fn new() -> Self {
        Self { map: IndexMap::new() }
    }
}


/// DataType holds value, DataTypeId and size in be_bytes. 
#[derive(Debug, Clone)]
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
    IDENTIFIER,
    HUMANNAME,
    LIST,
    OBJ(Tree),
    EMPTY,
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
    IDENTIFIER,
    HUMANNAME,
    LIST,
    OBJ,
    EMPTY
}



impl DataId {
    pub fn as_bytes(self) -> [u8; 2] {
        (self as u16).to_be_bytes()
    }
    pub fn is_primitive(self) -> bool {
        (self as usize) < PRIM_DIV 
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
            6  => Ok(DataId::DATETIME),
            7  => Ok(DataId::DEC),
            8  => Ok(DataId::ID),
            9  => Ok(DataId::INSTANT),
            10 => Ok(DataId::INTEGER),
            11 => Ok(DataId::INTEGER64),
            12 => Ok(DataId::MARKDOWN),
            13 => Ok(DataId::OID),
            14 => Ok(DataId::STRING),
            15 => Ok(DataId::POSITIVEINT),
            16 => Ok(DataId::TIME),
            17 => Ok(DataId::URI),
            18 => Ok(DataId::URL),
            19 => Ok(DataId::UUID),
            20 => Ok(DataId::UNSIGNEDINT),
            21 => Ok(DataId::KEY),
            22 => Ok(DataId::KEYID),
            23 => Ok(DataId::LIST),
            24 => Ok(DataId::OBJ),
            25 => Ok(DataId::IDENTIFIER),
            26 => Ok(DataId::HUMANNAME),
            27 => Ok(DataId::OBJ),
            28 => Ok(DataId::EMPTY),
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
    pub fn get_as_vec(self) -> Vec<u8>  {
        let vec = Vec::with_capacity(128);
        match self {
            DataType::KEY(val, id, len)    => to_vector(vec, val, id, len),
            DataType::KEYID(val, id, len)  => to_vector(vec, val, id, len),
            DataType::STRING(val, id, len) => to_vector(vec, val, id, len),
            DataType::BOOL(val, id, len)   => to_vector(vec, val, id, len),
            DataType::ID(val, id, len)     => to_vector(vec, val, id, len),
            _                              => unreachable!("unknown datatype {self:?}")
        }
    }

    pub fn try_from_memory(id: u16, len: u16, data: &[u8]) -> Result<DataType> {
        let dt = DataId::try_from(id)?;
        let data = data.to_vec().into_boxed_slice();
        match dt {
            DataId::MISC         => Ok(DataType::MISC(data, id, len)),
            DataId::BASE64BINARY => Ok(DataType::BASE64BINARY(data, id, len)),
            DataId::BOOL         => Ok(DataType::BOOL(data, id, len)),
            DataId::CANONICAL    => Ok(DataType::CANONICAL(data, id, len)),
            DataId::CODE         => Ok(DataType::CODE(data, id, len)),
            DataId::DATE         => Ok(DataType::DATE(data, id, len)),
            DataId::DATETIME     => Ok(DataType::DATETIME(data, id, len)),
            DataId::DEC          => Ok(DataType::DEC(data, id, len)),
            DataId::ID           => Ok(DataType::ID(data, id, len)),
            DataId::INSTANT      => Ok(DataType::INSTANT(data, id, len)),
            DataId::INTEGER      => Ok(DataType::INTEGER(data, id, len)),
            DataId::INTEGER64    => Ok(DataType::INTEGER64(data, id, len)),
            DataId::MARKDOWN     => Ok(DataType::MARKDOWN(data, id, len)),
            DataId::OID          => Ok(DataType::OID(data, id, len)),
            DataId::STRING       => Ok(DataType::STRING(data, id, len)),
            DataId::POSITIVEINT  => Ok(DataType::POSITIVEINT(data, id, len)),
            DataId::TIME         => Ok(DataType::TIME(data, id, len)),
            DataId::URI          => Ok(DataType::URI(data, id, len)),
            DataId::URL          => Ok(DataType::URL(data, id, len)),
            DataId::UUID         => Ok(DataType::UUID(data, id, len)),
            DataId::UNSIGNEDINT  => Ok(DataType::UNSIGNEDINT(data, id, len)),
            DataId::KEY          => Ok(DataType::KEY(data, id, len)),
            DataId::KEYID        => Ok(DataType::KEYID(data, id, len)),
            _                    => Err(Error::Custom("NOT IMPLEMENTED YET".to_string()))
        }
    } 

    pub fn to_json_bytes(self) -> Vec<u8> {
        match self {
            DataType::KEY(val, _, len)    => {
                let mut v = Vec::with_capacity(len as usize);
                v.push(QUO);
                v.extend_from_slice(&*val);
                v.extend_from_slice(KEN);
                v
            },
            DataType::STRING(val, _, len) => {
                let mut v = Vec::with_capacity(len as usize);
                v.push(QUO);
                v.extend_from_slice(&*val);
                v.push(QUO);
                v
            },
            DataType::BOOL(val, _, _)   => {
                if val[0] == 0 {
                    b" fasle ".to_vec()
                } else {
                    b" true ".to_vec()
                }
            },
            _                              => unreachable!("unknown datatype {self:?}")
        }
    }
}





fn to_vector(mut vec: Vec<u8>, val: Box<[u8]>, id: u16, len: u16) -> Vec<u8> {
    vec.extend_from_slice(&id.to_be_bytes());
    vec.extend_from_slice(&len.to_be_bytes());
    vec.extend_from_slice(&*val);
    vec
}


fn get_fixed_size_bytes<T: Sized>() -> BU16 {
    (std::mem::size_of::<T>() as u16).to_be_bytes()
}
fn get_fixed_size<T: Sized>() -> u16 {
    std::mem::size_of::<T>() as u16
}

pub fn get_u16(i: usize) -> Result<u16> {
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


#[derive(Clone, Debug)]
#[repr(u16)]
pub enum KEY {
    Anonymous = ID_DIV as u16,
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

static KEYINVERT: phf::Map<u16, &'static [u8]> = phf_map! {
    257u16 => b"resourceType", //resourceType
    258u16 => b"id",           //id
    259u16 => b"active",       //active
    260u16 => b"identifier",   //identifier
    270u16 => b"name",         //name
};

static EXPECTS: phf::Map<u16, DataId> = phf_map! {
    257u16 => DataId::STRING,     //resourceType
    258u16 => DataId::ID,         //id
    259u16 => DataId::BOOL,       //active
    260u16 => DataId::IDENTIFIER, //identifier
    270u16 => DataId::HUMANNAME   //name
};

pub fn get_key(k: &str) -> Option<KEY> {
    KEYS.get(k).cloned()
} 
pub fn get_key_inv(k: u16) -> Option<&'static [u8]> {
    KEYINVERT.get(&k).cloned()
} 
pub fn get_expects(exp: &KEY) -> Option<DataId> {
    EXPECTS.get(&(*exp as u16)).cloned()
} 

pub fn get_expects_u16(exp: u16) -> Option<DataId> {
    EXPECTS.get(&exp).cloned()
}

pub fn id_is_key(id: u16) -> bool {
    id >= ID_DIV as u16
}





