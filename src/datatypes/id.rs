use crate::error::{Error, Result};

use phf::{phf_map, phf_set};

pub const ID_LEN: u16 = 2;

const EOL:u16 = 21;
const GENERAL_PURPOSE: u16 = 512;
const GENERAL_PURPOSE_LIST: u16 = 2048;
const KEY_ID_START: u16 = 4096;

#[derive(Clone, Debug)]
#[repr(u16)]
pub enum ID {
    EMPTY,
    STRING,
    BOOLEAN,
    CODE,
    ID,
    ENDOFLIST = EOL,
    LSTRING,
    NARRATIVE = GENERAL_PURPOSE,
    HUMANNAME,
    LHUMANNAME = GENERAL_PURPOSE_LIST,
    ResourceType = KEY_ID_START,
    Active,
    Text,
    Status,
    Div,
    Name,
    Use,
    Given,
    Family
}


impl ID {
    pub fn is_primitive(&self) -> bool {
        (*self as u16) < EOL 
    }

    pub fn is_primitive_list(&self) -> bool {
        let cast = *self as u16;
        cast > EOL && cast < GENERAL_PURPOSE
    }

    pub fn is_gp_list(&self) -> bool {
        let cast = *self as u16;
        cast >= GENERAL_PURPOSE_LIST && cast < KEY_ID_START
    }

    pub fn is_general_purpose(&self) -> bool {
        let cast = *self as u16;
        cast >= GENERAL_PURPOSE && cast < GENERAL_PURPOSE_LIST 
    }
    pub fn is_key(&self) -> bool {
        (*self as u16) >= KEY_ID_START
    }

    pub fn to_store(&self) -> [u8; 2] {
        (*self as u16).to_be_bytes()
    }

    
}

impl TryFrom<u16> for ID {
    type Error = Error;
    fn try_from(value: u16) -> Result<Self> {
        match IDS.get(&value) {
            Some(id) => Ok(id.clone()),
            None => Err(Error::Conversion("u16".to_string(), "ID".to_string())) 
        }
    }
}

impl Into<u16> for ID {
    fn into(self) -> u16 {
        self as u16
    }
} 


static IDS: phf::Map<u16, ID> = phf_map! {
       1u16    => ID::STRING,
       2u16    => ID::BOOLEAN,
       3u16    => ID::CODE,
       4u16    => ID::ID,
       21u16   => ID::ENDOFLIST,
       22u16   => ID::LSTRING,
       512u16  => ID::NARRATIVE,
       513u16  => ID::HUMANNAME,
       2048u16 => ID::LHUMANNAME,
       4096u16 => ID::ResourceType,
       4097u16 => ID::Active,
       4098u16 => ID::Text,
       4099u16 => ID::Status,
       4100u16 => ID::Div,
       4101u16 => ID::Name,
       4102u16 => ID::Use,
       4103u16 => ID::Given,
       4104u16 => ID::Family
};

static KEYS: phf::Map<&'static str, ID> = phf_map! {
    "resourcetype" => ID::ResourceType,
    "active"       => ID::Active,
    "text"         => ID::Text,
    "status"       => ID::Status,
    "div"          => ID::Div,
    "name"         => ID::Name,
    "use"          => ID::Use,
    "family"       => ID::Family,
    "given"        => ID::Given
};


static EXPECTS: phf::Map<u16, ID> = phf_map! {
    4096u16 => ID::STRING,     //resourceType
    4097u16 => ID::BOOLEAN,    //active
    4098u16 => ID::NARRATIVE,  //text    
    4099u16 => ID::STRING,     //status
    4100u16 => ID::STRING,     //div
    4101u16 => ID::LHUMANNAME, //name
    4102u16 => ID::CODE,       //use
    4103u16 => ID::LSTRING,    //given,
    4104u16 => ID::STRING,     //family
};

static HUMANNAME_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4103u16 => ID::LSTRING, //given,
    4104u16 => ID::STRING,  //family
};

static NARRATIVE_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4099u16 => ID::CODE,       //status
    4100u16 => ID::STRING,     //div
};

static HAS_SUPS: phf::Set<u16> = phf_set! {
    512u16, //NARRATIVE
    513u16, //HUMANNAME
    2048u16, //LHUMANNAME
};

pub fn get_expected<I: Into<u16>+Clone>(key: I) -> Option<ID> {
    if let Some(exp) = get_expects(key.clone()) {
        if has_sub(exp.clone().into()) {
            return get_from_sub(key, exp.into())
        } else {
            return Some(exp)
        }
    }
    None
}

pub fn get_expects<I: Into<u16>>(exp: I) -> Option<ID> {
    EXPECTS.get(&exp.into()).cloned()
}

pub fn has_sub(id: u16) -> bool {
    HAS_SUPS.contains(&id)
}

pub fn get_from_sub<I: Into<u16>>(id: I, expects_for: u16) -> Option<ID> {
    match id.into() {
        513u16 | 2048u16 => {
            HUMANNAME_EXPECTS.get(&expects_for).cloned()
        },
        512u16 => {
            NARRATIVE_EXPECTS.get(&expects_for).cloned()
        }
        _ => None
    }
}

pub fn get_key_id(key: &[u8]) -> Option<ID> {
    if let Ok(s) = std::str::from_utf8(key) {
        KEYS.get(&s.to_ascii_lowercase()).cloned()
    } else {
        None
    }
}








