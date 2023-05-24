use crate::error::{Error, Result};
use phf::phf_map;

use super::tree::Tree;


///Bytes representation of a u16
pub type U16B = [u8;2];
// convenience 
type BYTES = Box<[u8]>;

const PRIMITIVE_MAX: u16 = 20;
const DT_MAX: u16 = 256;

const FHIR_ID_LEN: usize = 64;


/// Size of metadata as in - 2 bytes length, 2 bytes id
pub const META_DATA_SIZE: usize = 4;


/// DataType holds value, DataTypeId and size in be_bytes. 
#[derive(Debug, Clone)]
pub enum DT {
    MISC(BYTES),
    BASE64BINARY(BYTES),
    BOOL(BYTES),
    CANONICAL(BYTES),
    CODE(BYTES),
    DATE(BYTES),
    DATETIME(BYTES),
    DEC(BYTES),
    ID(BYTES),
    INSTANT(BYTES),
    INTEGER(BYTES),
    INTEGER64(BYTES),
    MARKDOWN(BYTES),
    OID(BYTES),
    STRING(BYTES),
    POSITIVEINT(BYTES),
    TIME(BYTES),
    URI(BYTES),
    URL(BYTES),
    UUID(BYTES),
    UNSIGNEDINT(BYTES),
    IDENTIFIER,
    HUMANNAME,
    NARRATIVE,
    LIST,
    OBJ,
    EMPTY,
    RSRC,
}

/// These are either DataType Ids or Key Ids repr as u16. 
/// The first 20 (with the exception of [`STOREID::MISC`]) represent the  
/// [primitive](`https://www.hl7.org/fhir/datatypes.html`) fhir type. Ids starting from 256 represent the keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum StoreId {
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
    LIST,
    OBJ,
    EMPTY,
    RSRC,
    IDENTIFIER,
    HUMANNAME,
    NARRATIVE,
    //KEYS 
    Anonymous = DT_MAX,
    ResourceType,
    Id,
    Active,
    Identifier,
    Name,
    Text,
    Status,
    Div

}


#[derive(Debug, Clone)]
pub enum DTR {
    MISC,
    BASE64BINARY(String),
    BOOL(bool),
    CANONICAL(BYTES),
    CODE(BYTES),
    DATE(BYTES),
    DATETIME(BYTES),
    DEC(BYTES),
    ID(String),
    INSTANT(BYTES),
    INTEGER(BYTES),
    INTEGER64(BYTES),
    MARKDOWN(BYTES),
    OID(BYTES),
    STRING(String),
    POSITIVEINT(BYTES),
    TIME(BYTES),
    URI(BYTES),
    URL(BYTES),
    UUID(BYTES),
    UNSIGNEDINT(BYTES),
    LIST,
    OBJ(Tree),
    EMPTY,
    RSRC(Tree),
    RSRCT(String),
    NARRATIVE(Tree),
}


impl DT {
    pub fn store(&self) -> Result<Vec<u8>> {
        match self {
            DT::BOOL(val) => dt_serialized(val.clone(), StoreId::BOOL),
            DT::ID(val) => dt_serialized(val.clone(), StoreId::ID),
            DT::STRING(val) => dt_serialized(val.clone(), StoreId::STRING),
            _ => unimplemented!("DT -> store unimplemented for ") 
        }
    }

    pub fn store_len(&self) -> usize {
        match self {
            DT::BOOL(val) => val.len(),
            DT::ID(val) => val.len(),
            DT::STRING(val) => val.len(),
            _ => unimplemented!("DT -> store_len unimplemented for {:?}", self) 
        }
    }
}
impl StoreId {
    pub fn as_bytes(self) -> U16B {
        (self as u16).to_be_bytes()
    }

    pub fn is_primitive(&self) -> bool {
        (*self as u16) <= PRIMITIVE_MAX
    }

    pub fn is_general_purpose(&self) -> bool {
        (*self as u16) > PRIMITIVE_MAX && (*self as u16) < DT_MAX
    }
}

impl TryFrom<u16> for StoreId {
    type Error = Error;
    fn try_from(i: u16) -> Result<StoreId> {
        match i {
            0  => Ok(StoreId::MISC),
            1  => Ok(StoreId::BASE64BINARY),
            2  => Ok(StoreId::BOOL),
            3  => Ok(StoreId::CANONICAL),
            4  => Ok(StoreId::CODE),
            5  => Ok(StoreId::DATE),
            6  => Ok(StoreId::DATETIME),
            7  => Ok(StoreId::DEC),
            8  => Ok(StoreId::ID),
            9  => Ok(StoreId::INSTANT),
            10 => Ok(StoreId::INTEGER),
            11 => Ok(StoreId::INTEGER64),
            12 => Ok(StoreId::MARKDOWN),
            13 => Ok(StoreId::OID),
            14 => Ok(StoreId::STRING),
            15 => Ok(StoreId::POSITIVEINT),
            16 => Ok(StoreId::TIME),
            17 => Ok(StoreId::URI),
            18 => Ok(StoreId::URL),
            19 => Ok(StoreId::UUID),
            20 => Ok(StoreId::UNSIGNEDINT),
            21 => Ok(StoreId::LIST),
            22 => Ok(StoreId::OBJ),
            23 => Ok(StoreId::EMPTY),
            24 => Ok(StoreId::RSRC),
            25 => Ok(StoreId::IDENTIFIER),
            26 => Ok(StoreId::HUMANNAME),
            27 => Ok(StoreId::NARRATIVE),
            257 => Ok(StoreId::ResourceType),
            258 => Ok(StoreId::Id),
            259 => Ok(StoreId::Active),
            260 => Ok(StoreId::Identifier),
            261 => Ok(StoreId::Name),
            262 => Ok(StoreId::Text),
            263 => Ok(StoreId::Status),
            264 => Ok(StoreId::Div),
             _ => Err(self::Error::UnknownStoreId(i))
        }
    }
}

impl DTR {
    pub fn from_store(id: u16, data: &mut [u8]) -> Result<Self> {
        match id {
            2  => Ok(DTR::BOOL(u8_slice_to_bool(data)?)),
            8  => Ok(DTR::STRING(u8_slice_to_string(data)?)),
            14 => Ok(DTR::STRING(u8_slice_to_string(data)?)),
            257 => Ok(DTR::RSRCT(u8_slice_to_string(data)?)),
             _ => Err(self::Error::UnknownStoreId(id))
        }
    }

    pub fn from_store_with_tree(id: u16, tree: Tree) -> Result<Self> {
        match id {
            27 => Ok(DTR::NARRATIVE(tree)),
             _ => Err(self::Error::UnknownStoreId(id))
        }
    }
}

//################# DTR Handlers ###########################################

fn u8_slice_to_bool(data: &[u8]) -> Result<bool> {
    if data.len() != 1  {
        Err(Error::Expected("&[u8] with len 1".to_string(), "&[u8] with len > 1 or len == 0".to_string()))
    } else {
        match data[0] {
            1 => Ok(true),
            0 => Ok(false), 
            _ => Err(Error::Expected("0 or 1".to_string(), data[0].to_string()))
        }
    }
}

//mainly to convert the error to a custom error
fn u8_slice_to_string(data: &[u8]) -> Result<String> {
    match String::from_utf8(data.to_vec()) {
        Ok(s) => Ok(s),
        Err(err) => Err(Error::Custom(err.to_string()))
    }
}





//################## Converter Traits #######################################

/// Trait to be implemented for primitive datatype conversion.
pub trait ToStore {
    fn to_store(&self) -> DT;
}

/// Trait to be implemented for primitive and other datatype conversions, that takes a type hint.
pub trait ToStoreWith {
    fn to_store_with(&self, id: StoreId) -> Result<DT>;
}



//################## From Conversion Methods DT #############################

impl ToStore for bool {
    fn to_store(&self) -> DT {
        match self {
            true => DT::BOOL(Box::new([1])),
            false => DT::BOOL(Box::new([0])),
        }
    }
}


impl ToStore for f64 {
    fn to_store(&self) -> DT {
        DT::DEC(Box::from(self.to_be_bytes()))
    }
}


impl ToStore for i32 {
    fn to_store(&self) -> DT {
        DT::UNSIGNEDINT(Box::from(self.to_be_bytes()))
    }
}
impl ToStore for i64 {
    fn to_store(&self) -> DT {
        DT::UNSIGNEDINT(Box::from(self.to_be_bytes()))
    }
}

impl ToStore for u32 {
    fn to_store(&self) -> DT {
        DT::UNSIGNEDINT(Box::from(self.to_be_bytes()))
    }
}

impl ToStore for String {
    fn to_store(&self) -> DT {
        DT::STRING(Box::from(self.as_bytes()))
    }
}

impl ToStoreWith for String {
    fn to_store_with(&self, id: StoreId) -> Result<DT> {
        let len =  self.len();
        match id {
            StoreId::ID => {
                if len > FHIR_ID_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DT::ID(Box::from(self.as_bytes())))
              }
          },
          _ => unimplemented!("{id:?} not implemented yet for String")
        }
    }
}

impl ToStoreWith for &str {
    fn to_store_with(&self, id: StoreId) -> Result<DT> {
        let len =  self.len();
        match id {
            StoreId::ID => {
                if len > FHIR_ID_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DT::ID(Box::from(self.as_bytes())))
              }
          },
          StoreId::STRING => {
                Ok(DT::STRING(Box::from(self.as_bytes())))
          }
          _ => unimplemented!("{id:?} not implemented yet for &str")
        }
    }
}

impl ToStoreWith for &[u8] {
    fn to_store_with(&self, id: StoreId) -> Result<DT> {
        let len =  self.len();
        match id {
            StoreId::ID => {
                if len > FHIR_ID_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DT::ID(Box::from(*self)))
              }
          },
            StoreId::STRING => {
                Ok(DT::STRING(Box::from(*self)))
            }
          _ => unimplemented!("{id:?} not implemented yet for &[u8]")
        }
    }
}

impl ToStoreWith for Vec<u8> {
    fn to_store_with(&self, id: StoreId) -> Result<DT> {
        let len =  self.len();
        match id {
            StoreId::ID => {
                if len > FHIR_ID_LEN {
                    return Err(Error::IdMaxLen)
                } else {
                    Ok(DT::ID(Box::from(self.as_slice())))
              }
          },
            StoreId::STRING => {
                Ok(DT::STRING(Box::from(self.as_slice())))
            }
          _ => unimplemented!("{id:?} not implemented yet for &[u8]")
        }
    }
}
//################## Lookup tables ###########################################

static KEYS: phf::Map<&'static str, StoreId> = phf_map! {
    "anonymous"    => StoreId::Anonymous,
    "resourcetype" => StoreId::ResourceType,
    "identifier"   => StoreId::Identifier,
    "id"           => StoreId::Id,
    "active"       => StoreId::Active,
    "name"         => StoreId::Name,
    "text"         => StoreId::Text,
    "status"       => StoreId::Status,
    "div"          => StoreId::Div,
};

static KEYS_INVERT: phf::Map<u16, &'static [u8]> = phf_map! {
    257u16 => b"resourceType", //resourceType
    258u16 => b"id",           //id
    259u16 => b"active",       //active
    260u16 => b"identifier",   //identifier
    261u16 => b"name",         //name
    262u16 => b"text",    //narrative
    263u16 => b"status",       //status
    264u16 => b"div",          //div
};

static EXPECTS: phf::Map<u16, StoreId> = phf_map! {
    257u16 => StoreId::STRING,     //resourceType
    258u16 => StoreId::ID,         //id
    259u16 => StoreId::BOOL,       //active
    260u16 => StoreId::IDENTIFIER, //identifier
    261u16 => StoreId::HUMANNAME,  //name
    262u16 => StoreId::NARRATIVE,  //narrative
    263u16 => StoreId::STRING,     //status
    264u16 => StoreId::STRING,     //div
};


pub fn key_for_str(k: &str) -> Option<StoreId> {
    KEYS.get(k.to_ascii_lowercase().as_str()).cloned()
}

pub fn store_id_for_u16(k: u16) -> Option<&'static [u8]> {
    KEYS_INVERT.get(&k).cloned()
}

pub fn get_expects(exp: u16) -> Option<StoreId> {
    EXPECTS.get(&exp).cloned()
}
//################## Util funcs ##############################################

fn get_fixed_size<T: Sized>() -> u16 {
    std::mem::size_of::<T>() as u16
}

fn u16_from_usize(i: usize) -> Result<u16> {
    match u16::try_from(i) {
        Ok(uint) => Ok(uint),
        Err(err) => Err(Error::Custom(err.to_string()))
    }
}

pub fn u16_bytes_from_usize(i: usize) -> Result<U16B> {
    Ok(u16_from_usize(i)?.to_be_bytes())
}

fn check_u16_max(i: usize) -> Result<()> {
    if i > u16::MAX as usize {
        return Err(Error::StoreUnitMaxLen)
    } 
    Ok(())
}

fn dt_serialized(val: Box<[u8]>, id: StoreId) -> Result<Vec<u8>> {
    let mut container = Vec::<u8>::with_capacity(val.len()+4);
    let len = u16_from_usize(val.len())?.to_be_bytes();
    container.extend(len);
    container.extend(id.as_bytes());
    container.extend_from_slice(&*val);
    Ok(container)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dt_conversion() {
        let b = true;
        let ser = b.to_store().store().unwrap();
        assert_eq!(ser.len(), 5);
        assert_eq!(ser.capacity(), 5);
        assert_eq!(ser, vec![0,1,0,2,1])
    }

    #[test]
    fn key_lookup() {
        assert_eq!(key_for_str("resourceType").unwrap(), StoreId::ResourceType);
        assert_eq!(store_id_for_u16(StoreId::Id as u16).unwrap(), [105, 100]);
    }

    #[test]
    fn dtr_conversion() {
        let mut d = "hello ⁌".as_bytes().to_vec();
        let dtr = DTR::from_store(StoreId::STRING as u16, &mut d).unwrap();
        if let DTR::STRING(val) = dtr {
            assert_eq!(val,"hello ⁌".to_string());
        }
        let mut sparkle_heart = vec![0, 159, 146, 150];
        assert!(DTR::from_store(StoreId::STRING as u16, &mut sparkle_heart).is_err());

        let dtr_bool = DTR::from_store(StoreId::BOOL as u16, &mut[1]).unwrap();
        if let DTR::BOOL(val) = dtr_bool {
            assert!(val);
        }
        assert!(DTR::from_store(StoreId::BOOL as u16, &mut[1, 2]).is_err());
        assert!(DTR::from_store(StoreId::BOOL as u16, &mut[3]).is_err());

    }
}

