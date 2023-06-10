use crate::error::{Error, Result};
use phf::{phf_map, phf_set};

pub const ID_LEN: u16 = 2;

const EOL:u16 = 21;
const GENERAL_PURPOSE: u16 = 512;
const MULTIPLE: u16 = 2047;
const GENERAL_PURPOSE_LIST: u16 = 2048;
const KEY_ID_START: u16 = 4096;

//TODO handle Extensions and Ids

#[derive(Clone, Debug)]
#[repr(u16)]
pub enum ID {
    EMPTY,
    STRING,
    BOOLEAN,
    CODE,
    ID,
    URI,
    DATETIME,        //this could be more space efficient as a timestamp on disk but conversion could be slow as it is a string parsing
    POSITIVEINT,     //i32 but non-negative FHIR specs 1-2147483647
    DATE,
    INTEGER,         //i32 min to max 
    ENDOFLIST = EOL,
    LSTRING,
    MULTIPLETYPES = MULTIPLE,
    NARRATIVE = GENERAL_PURPOSE,
    HUMANNAME,
    IDENTIFIER,
    CODABLECONCEPT,
    PERIOD,
    REFERENCE,
    CODING,
    CONTACTPOINT,
    LHUMANNAME = GENERAL_PURPOSE_LIST,
    LIDENTIFIER,
    LCODING,
    LCONTACTPOINT,
    ResourceType = KEY_ID_START,
    Active,
    Text,
    Status,
    Div,
    Name,
    Use,
    Given,
    Family,
    Id,
    Type,
    System,
    Value,
    Period,
    Start,
    End,
    Assigner,
    Reference,
    Display,
    Version,
    Code,
    UserSelected,
    Coding,
    Identifier,
    Telecom,
    Rank,
    Gender,
    BirthDate,
    Deceased,
    MultipleBirth,
}

///Used for Multiple Type Values and helps for JSON parsing.
#[derive(Clone, Debug)]
#[repr(u16)]
pub enum TypeClass {
    STRING,
    NUMERIC,
    BOOLEAN,
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

    pub fn is_multiple(&self) -> bool {
        *self as u16 == MULTIPLE
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
       1u16    => ID::STRING,         //[0,1]
       2u16    => ID::BOOLEAN,        //[0,2]
       3u16    => ID::CODE,           //[0,3]
       4u16    => ID::ID,             //[0,4]
       5u16    => ID::URI,            //[0,5]
       6u16    => ID::DATETIME,       //[0,6]
       7u16    => ID::POSITIVEINT,    //[0,7]
       8u16    => ID::DATE,           //[0,8]
       9u16    => ID::INTEGER,        //[0,9]
       21u16   => ID::ENDOFLIST,      //[0,21]
       22u16   => ID::LSTRING,        //[0,22]
       512u16  => ID::NARRATIVE,      //[2,0]
       513u16  => ID::HUMANNAME,      //[2,1]
       514u16  => ID::IDENTIFIER,     //[2,2]
       515u16  => ID::CODABLECONCEPT, //[2,3]
       516u16  => ID::PERIOD,         //[2,4]
       517u16  => ID::REFERENCE,      //[2,5]
       518u16  => ID::CODING,         //[2,6]
       519u16  => ID::CONTACTPOINT,   //[2,7]
       2047u16 => ID::MULTIPLETYPES,  //[7,255]
       2048u16 => ID::LHUMANNAME,     //[8,0]
       2049u16 => ID::LIDENTIFIER,    //[8,1]
       2050u16 => ID::LCODING,        //[8,2]
       2051u16 => ID::LCONTACTPOINT,  //[8,3]
       4096u16 => ID::ResourceType,   //[16,0]
       4097u16 => ID::Active,         //[16,1]
       4098u16 => ID::Text,           //[16,2]
       4099u16 => ID::Status,         //[16,3]
       4100u16 => ID::Div,            //[16,4]
       4101u16 => ID::Name,           //[16,5]
       4102u16 => ID::Use,            //[16,6]
       4103u16 => ID::Given,          //[16,7]
       4104u16 => ID::Family,         //[16,8]
       4105u16 => ID::Id,             //[16,9]
       4106u16 => ID::Type,           //[16,10]
       4107u16 => ID::System,         //[16,11]
       4108u16 => ID::Value,          //[16,12]
       4109u16 => ID::Period,         //[16,13]
       4110u16 => ID::Start,          //[16,14]
       4111u16 => ID::End,            //[16,15]
       4112u16 => ID::Assigner,       //[16,16]
       4113u16 => ID::Reference,      //[16,17]
       4114u16 => ID::Display,        //[16,18]
       4115u16 => ID::Version,        //[16,19]
       4116u16 => ID::Code,           //[16,20]
       4117u16 => ID::UserSelected,   //[16,21]
       4118u16 => ID::Coding,         //[16,22]
       4119u16 => ID::Identifier,     //[16,23]
       4120u16 => ID::Telecom,        //[16,24]
       4121u16 => ID::Rank,           //[16,25]
       4122u16 => ID::Gender,         //[16,26]
       4123u16 => ID::BirthDate,      //[16,27]
       4124u16 => ID::Deceased,       //[16,28]
       4125u16 => ID::MultipleBirth,  //[16,29]
        
};

static KEYS: phf::Map<&'static str, ID> = phf_map! {
    "id"            => ID::Id,
    "resourcetype"  => ID::ResourceType,
    "active"        => ID::Active,
    "text"          => ID::Text,
    "status"        => ID::Status,
    "div"           => ID::Div,
    "name"          => ID::Name,
    "use"           => ID::Use,
    "family"        => ID::Family,
    "given"         => ID::Given,
    "type"          => ID::Type,
    "system"        => ID::System,
    "value"         => ID::Value,
    "period"        => ID::Period,
    "start"         => ID::Start,
    "end"           => ID::End,
    "assigner"      => ID::Assigner,
    "reference"     => ID::Reference,
    "display"       => ID::Display,
    "version"       => ID::Version,
    "code"          => ID::Code,
    "userselected"  => ID::UserSelected,
    "coding"        => ID::Coding,
    "identifier"    => ID::Identifier,
    "telecom"       => ID::Telecom,
    "rank"          => ID::Rank,
    "gender"        => ID::Gender,
    "birthdate"     => ID::BirthDate,
    "deceased"      => ID::Deceased,
    "multiplebirth" => ID::MultipleBirth,
};


static EXPECTS: phf::Map<u16, ID> = phf_map! {
    4096u16 => ID::STRING,         //resourceType
    4097u16 => ID::BOOLEAN,        //active
    4098u16 => ID::NARRATIVE,      //text
    4099u16 => ID::STRING,         //status
    4100u16 => ID::STRING,         //div
    4101u16 => ID::LHUMANNAME,     //name
    4102u16 => ID::CODE,           //use
    4103u16 => ID::LSTRING,        //given,
    4104u16 => ID::STRING,         //family
    4105u16 => ID::ID,             //id
    4106u16 => ID::CODABLECONCEPT, //type
    4107u16 => ID::URI,            //system
    4108u16 => ID::STRING,         //value
    4109u16 => ID::PERIOD,         //period
    4110u16 => ID::DATETIME,       //start
    4111u16 => ID::DATETIME,       //end
    4112u16 => ID::REFERENCE,      //assigner
    4113u16 => ID::STRING,         //reference
    4114u16 => ID::STRING,         //display
    4115u16 => ID::STRING,         //version
    4116u16 => ID::CODE,           //code
    4117u16 => ID::BOOLEAN,        //userselected
    4118u16 => ID::LCODING,        //coding
    4119u16 => ID::LIDENTIFIER,    //identifier
    4120u16 => ID::LCONTACTPOINT,  //telecom
    4121u16 => ID::POSITIVEINT,    //rank
    4122u16 => ID::CODE,           //gener
    4123u16 => ID::DATE,           //birthdate
    4124u16 => ID::MULTIPLETYPES,  //deceased
    4125u16 => ID::MULTIPLETYPES,  //multiplebirth
};

static HAS_SUBS: phf::Set<u16> = phf_set! {
    512u16,  //NARRATIVE
    513u16,  //HUMANNAME
    514u16,  //IDENTIFIER
    515u16,  //CODABLECONCEPT
    516u16,  //PERIOD
    517u16,  //REFERENCE
    518u16,  //CODING
    519u16,  //CONTACTPOINT
    2048u16, //LHUMANNAME
    2049u16, //LIDENTIFIER
    2050u16, //LCODING
    2051u16, //LCONTACTPOINT
};
///Mapping of [`TypeClass`] to ID
static MULTIPLE_DECEASED: phf::Map<u16, u16> = phf_map! {
   3u16 => 2u16, //BOOLEAN/BOOLEAN 
   1u16 => 6u16, //SRING/DATETIME
};

///Mapping of [`TypeClass`] to ID
static MULTIPLE_MULTIPLE_BIRTH: phf::Map<u16, u16> = phf_map! {
   3u16 => 2u16, //BOOLEAN/BOOLEAN
   2u16 => 9u16, //NUMERIC/INTEGER
};

static HUMANNAME_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4103u16 => ID::LSTRING, //given,
    4104u16 => ID::STRING,  //family
};

static NARRATIVE_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4099u16 => ID::CODE,       //status
    4100u16 => ID::STRING,     //div
};

static IDENTIFIER_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4102u16 => ID::CODE,           //use 0..1
    4106u16 => ID::CODABLECONCEPT, //type [Codeable Concept] 0..1
    4107u16 => ID::URI,            //system [uri] 0..1
    4108u16 => ID::STRING,         //value [string] 0..1
    4109u16 => ID::PERIOD,         //period [Period] 0..1
    4112u16 => ID::REFERENCE,      //Reference(Organization) 0..1
};

static CODEABLECONCEPT_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4098u16 => ID::STRING, //text
    4118u16 => ID::LCODING, //coding
};
static CODING_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4107u16 => ID::URI,     //system [uri] 0..1
    4115u16 => ID::STRING,  //version [string] 0..1
    4116u16 => ID::CODE,    //code [code] 0..1
    4114u16 => ID::STRING,  //display [string] 0..1
    4117u16 => ID::BOOLEAN, //userselected [boolean] 0..1
};
static PERIOD_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4110u16 => ID::DATETIME, //start
    4111u16 => ID::DATETIME, //end
};
static REFERENCE_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4113u16 => ID::STRING,     //reference [string] 0..1
    4106u16 => ID::URI,        //type [uri] 0..1
    4105u16 => ID::IDENTIFIER, //identifier [identifier] 0..1
    4114u16 => ID::STRING,     //display [string] 0..1
};

static CONTACTPOINT_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4102u16 => ID::CODE,        //use 0..1
    4107u16 => ID::URI,         //system [uri] 0..1
    4108u16 => ID::STRING,      //value [string] 0..1
    4109u16 => ID::PERIOD,      //period [Period] 0..1
    4121u16 => ID::POSITIVEINT, //rank [positiveInt] 0..1
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
    HAS_SUBS.contains(&id)
}

pub fn get_from_sub<I: Into<u16>>(id: I, expects_for: u16) -> Option<ID> {
    match id.into() {
        512u16 => {
            NARRATIVE_EXPECTS.get(&expects_for).cloned()
        }
        513u16 | 2048u16 => {
            HUMANNAME_EXPECTS.get(&expects_for).cloned()
        },
        514u16 | 2049u16 => {
            IDENTIFIER_EXPECTS.get(&expects_for).cloned()
        },
        515 => {
            CODEABLECONCEPT_EXPECTS.get(&expects_for).cloned()
        },
        516u16 => {
            PERIOD_EXPECTS.get(&expects_for).cloned()
        },  
        517u16 => {
            REFERENCE_EXPECTS.get(&expects_for).cloned()
        },  
        518u16 | 2050u16 => {
            CODING_EXPECTS.get(&expects_for).cloned()
        },
        519u16 | 2051u16 => {
            CONTACTPOINT_EXPECTS.get(&expects_for).cloned()
        },
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








