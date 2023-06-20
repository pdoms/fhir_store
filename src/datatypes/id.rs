use std::collections;

use crate::error::{Error, Result};
use phf::{phf_map, phf_set};

pub const ID_LEN: u16 = 2;

const EOL:u16 = 21;
const GENERAL_PURPOSE: u16 = 512;
const MULTIPLE: u16 = 2047;
const GENERAL_PURPOSE_LIST: u16 = 2048;
const KEY_ID_START: u16 = 4096;


//TODO handle Extensions and Ids

#[derive(Clone, Debug, PartialEq, Eq)]
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
    INTEGER64,       //i64
    DECIMAL,         //f64
    BASE64BINARY,    
    URL,
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
    ADDRESS,
    ATTACHMENT,
    BACKBONECONTACT, 
    BACKBONECOMMUNICATION, 
    BACKBONELINK,
    LHUMANNAME = GENERAL_PURPOSE_LIST,
    LIDENTIFIER,
    LCODING,
    LCONTACTPOINT,
    LADDRESS,
    LATTACHMENT,
    LBACKBONECONTACT,
    LCODABLECONCEPT,
    LBACKBONECOMMUNICATION,
    LBACKBONELINK,
    LREFERENCE,
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
    Address,
    Line,
    City,
    District,
    State,
    PostalCode,
    Country,
    MaritalStatus,
    Attachment,
    Photo,
    ContentType,
    Language,
    Data,
    Url,
    Size,
    Hash,
    Title,
    Creation,
    Height,
    Width,
    Frames,
    Duration,
    Pages,
    Contact,
    Relationship,
    Organization,
    Communication,
    Preferred,
    GeneralPractitioner,
    ManagingOrganization,
    Link,
    Other,
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
    1u16    => ID::STRING,                 //[0,1]
    2u16    => ID::BOOLEAN,                //[0,2]
    3u16    => ID::CODE,                   //[0,3]
    4u16    => ID::ID,                     //[0,4]
    5u16    => ID::URI,                    //[0,5]
    6u16    => ID::DATETIME,               //[0,6]
    7u16    => ID::POSITIVEINT,            //[0,7]
    8u16    => ID::DATE,                   //[0,8]
    9u16    => ID::INTEGER,                //[0,9]
    10u16   => ID::INTEGER64,              //[0,10]
    11u16   => ID::DECIMAL,                //[0,11]
    12u16   => ID::BASE64BINARY,           //[0,12]
    13u16   => ID::URL,                    //[0,13
    21u16   => ID::ENDOFLIST,              //[0,21]
    22u16   => ID::LSTRING,                //[0,22]
    512u16  => ID::NARRATIVE,              //[2,0]
    513u16  => ID::HUMANNAME,              //[2,1]
    514u16  => ID::IDENTIFIER,             //[2,2]
    515u16  => ID::CODABLECONCEPT,         //[2,3]
    516u16  => ID::PERIOD,                 //[2,4]
    517u16  => ID::REFERENCE,              //[2,5]
    518u16  => ID::CODING,                 //[2,6]
    519u16  => ID::CONTACTPOINT,           //[2,7]
    520u16  => ID::ADDRESS,                //[2,8]
    521u16  => ID::ATTACHMENT,             //[2,9]
    522u16  => ID::BACKBONECONTACT,        //[2,10]
    523u16  => ID::BACKBONECOMMUNICATION,  //[2,11]
    524u16  => ID::BACKBONELINK,           //[2,12]
    2047u16 => ID::MULTIPLETYPES,          //[7,255]
    2048u16 => ID::LHUMANNAME,             //[8,0]
    2049u16 => ID::LIDENTIFIER,            //[8,1]
    2050u16 => ID::LCODING,                //[8,2]
    2051u16 => ID::LCONTACTPOINT,          //[8,3]
    2052u16 => ID::LADDRESS,               //[8,4]
    2053u16 => ID::LATTACHMENT,            //[8,5]
    2054u16 => ID::LBACKBONECONTACT,       //[8,6]
    2055u16 => ID::LCODABLECONCEPT,        //[8,7]
    2056u16 => ID::LBACKBONECOMMUNICATION, //[8,8]
    2057u16 => ID::LBACKBONELINK,          //[8,9]
    2058u16 => ID::LREFERENCE,             //[8,10]
    4096u16 => ID::ResourceType,           //[16,0]
    4097u16 => ID::Active,                 //[16,1]
    4098u16 => ID::Text,                   //[16,2]
    4099u16 => ID::Status,                 //[16,3]
    4100u16 => ID::Div,                    //[16,4]
    4101u16 => ID::Name,                   //[16,5]
    4102u16 => ID::Use,                    //[16,6]
    4103u16 => ID::Given,                  //[16,7]
    4104u16 => ID::Family,                 //[16,8]
    4105u16 => ID::Id,                     //[16,9]
    4106u16 => ID::Type,                   //[16,10]
    4107u16 => ID::System,                 //[16,11]
    4108u16 => ID::Value,                  //[16,12]
    4109u16 => ID::Period,                 //[16,13]
    4110u16 => ID::Start,                  //[16,14]
    4111u16 => ID::End,                    //[16,15]
    4112u16 => ID::Assigner,               //[16,16]
    4113u16 => ID::Reference,              //[16,17]
    4114u16 => ID::Display,                //[16,18]
    4115u16 => ID::Version,                //[16,19]
    4116u16 => ID::Code,                   //[16,20]
    4117u16 => ID::UserSelected,           //[16,21]
    4118u16 => ID::Coding,                 //[16,22]
    4119u16 => ID::Identifier,             //[16,23]
    4120u16 => ID::Telecom,                //[16,24]
    4121u16 => ID::Rank,                   //[16,25]
    4122u16 => ID::Gender,                 //[16,26]
    4123u16 => ID::BirthDate,              //[16,27]
    4124u16 => ID::Deceased,               //[16,28]
    4125u16 => ID::MultipleBirth,          //[16,29]
    4126u16 => ID::Address,                //[16,30]
    4127u16 => ID::Line,                   //[16,31]
    4128u16 => ID::City,                   //[16,32]
    4129u16 => ID::District,               //[16,33]
    4130u16 => ID::State,                  //[16,34]
    4131u16 => ID::PostalCode,             //[16,35]
    4132u16 => ID::Country,                //[16,36]
    4133u16 => ID::MaritalStatus,          //[16,37]
    4134u16 => ID::Attachment,             //[16,38]
    4135u16 => ID::Photo,                  //[16,39]
    4136u16 => ID::ContentType,            //[16,40]
    4137u16 => ID::Language,               //[16,41]
    4138u16 => ID::Data,                   //[16,42]
    4139u16 => ID::Url,                    //[16,43]
    4140u16 => ID::Size,                   //[16,44]
    4141u16 => ID::Hash,                   //[16,45]
    4142u16 => ID::Title,                  //[16,46]
    4143u16 => ID::Creation,               //[16,47]
    4144u16 => ID::Height,                 //[16,48]
    4145u16 => ID::Width,                  //[16,49]
    4146u16 => ID::Frames,                 //[16,50]
    4147u16 => ID::Duration,               //[16,51]
    4148u16 => ID::Pages,                  //[16,52]
    4149u16 => ID::Contact,                //[16,52]
    4150u16 => ID::Relationship,           //[16,53]
    4151u16 => ID::Organization,           //[16,54]
    4152u16 => ID::Communication,          //[16,55]
    4153u16 => ID::Preferred,              //[16,56]
    4154u16 => ID::GeneralPractitioner,    //[16,57]
    4155u16 => ID::ManagingOrganization,   //[16,58]
    4156u16 => ID::Link,                   //[16,59]
    4157u16 => ID::Other,                  //[16,60]
};

static KEYS: phf::Map<&'static str, ID> = phf_map! {
    "id"                   => ID::Id,
    "resourcetype"         => ID::ResourceType,
    "active"               => ID::Active,
    "text"                 => ID::Text,
    "status"               => ID::Status,
    "div"                  => ID::Div,
    "name"                 => ID::Name,
    "use"                  => ID::Use,
    "family"               => ID::Family,
    "given"                => ID::Given,
    "type"                 => ID::Type,
    "system"               => ID::System,
    "value"                => ID::Value,
    "period"               => ID::Period,
    "start"                => ID::Start,
    "end"                  => ID::End,
    "assigner"             => ID::Assigner,
    "reference"            => ID::Reference,
    "display"              => ID::Display,
    "version"              => ID::Version,
    "code"                 => ID::Code,
    "userselected"         => ID::UserSelected,
    "coding"               => ID::Coding,
    "identifier"           => ID::Identifier,
    "telecom"              => ID::Telecom,
    "rank"                 => ID::Rank,
    "gender"               => ID::Gender,
    "birthdate"            => ID::BirthDate,
    "deceased"             => ID::Deceased,
    "multiplebirth"        => ID::MultipleBirth,
    "address"              => ID::Address,
    "line"                 => ID::Line,
    "city"                 => ID::City,
    "district"             => ID::District,
    "state"                => ID::State,
    "postalcode"           => ID::PostalCode,
    "country"              => ID::Country,
    "maritalstatus"        => ID::MaritalStatus,
    "attachment"           => ID::Attachment,
    "photo"                => ID::Photo,
    "contenttype"          => ID::ContentType,
    "langauge"             => ID::Language,
    "data"                 => ID::Data,
    "url"                  => ID::Url,
    "size"                 => ID::Size,
    "hash"                 => ID::Hash,
    "title"                => ID::Title,
    "creation"             => ID::Creation,
    "height"               => ID::Height,
    "width"                => ID::Width,
    "frames"               => ID::Frames,
    "duration"             => ID::Duration,
    "pages"                => ID::Pages,
    "contact"              => ID::Contact,
    "relationship"         => ID::Relationship,
    "organization"         => ID::Organization,
    "communication"        => ID::Communication,
    "preferred"            => ID::Preferred,
    "generalPractitioner"  => ID::GeneralPractitioner,
    "managingOrganization" => ID::ManagingOrganization,
    "link"                 => ID::Link,
    "other"                => ID::Other,
};


static EXPECTS: phf::Map<u16, ID> = phf_map! {
    4096u16 => ID::STRING,                 //resourceType
    4097u16 => ID::BOOLEAN,                //active
    4098u16 => ID::NARRATIVE,              //text
    4099u16 => ID::STRING,                 //status
    4100u16 => ID::STRING,                 //div
    4101u16 => ID::LHUMANNAME,             //name
    4102u16 => ID::CODE,                   //use
    4103u16 => ID::LSTRING,                //given,
    4104u16 => ID::STRING,                 //family
    4105u16 => ID::ID,                     //id
    4106u16 => ID::CODABLECONCEPT,         //type
    4107u16 => ID::URI,                    //system
    4108u16 => ID::STRING,                 //value
    4109u16 => ID::PERIOD,                 //period
    4110u16 => ID::DATETIME,               //start
    4111u16 => ID::DATETIME,               //end
    4112u16 => ID::REFERENCE,              //assigner
    4113u16 => ID::STRING,                 //reference
    4114u16 => ID::STRING,                 //display
    4115u16 => ID::STRING,                 //version
    4116u16 => ID::CODE,                   //code
    4117u16 => ID::BOOLEAN,                //userselected
    4118u16 => ID::LCODING,                //coding
    4119u16 => ID::LIDENTIFIER,            //identifier
    4120u16 => ID::LCONTACTPOINT,          //telecom
    4121u16 => ID::POSITIVEINT,            //rank
    4122u16 => ID::CODE,                   //gener
    4123u16 => ID::DATE,                   //birthdate
    4124u16 => ID::MULTIPLETYPES,          //deceased
    4125u16 => ID::MULTIPLETYPES,          //multiplebirth
    4126u16 => ID::LADDRESS,               //address
    4127u16 => ID::LSTRING,                //line
    4128u16 => ID::STRING,                 //city
    4129u16 => ID::STRING,                 //district
    4130u16 => ID::STRING,                 //state
    4131u16 => ID::STRING,                 //postalcode
    4132u16 => ID::STRING,                 //country
    4133u16 => ID::CODABLECONCEPT,         //maritalstatus
    4134u16 => ID::LATTACHMENT,            //maritalstatus
    4135u16 => ID::LATTACHMENT,            //attachment
    4136u16 => ID::CODE,                   //contenttype
    4137u16 => ID::CODE,                   //language
    4138u16 => ID::BASE64BINARY,           //data
    4139u16 => ID::URL,                    //url
    4140u16 => ID::INTEGER64,              //size
    4141u16 => ID::BASE64BINARY,           //hash
    4142u16 => ID::STRING,                 //title
    4143u16 => ID::DATETIME,               //creation
    4144u16 => ID::POSITIVEINT,            //height
    4145u16 => ID::POSITIVEINT,            //width
    4146u16 => ID::POSITIVEINT,            //frames
    4147u16 => ID::DECIMAL,                //duration
    4148u16 => ID::POSITIVEINT,            //pages
    4149u16 => ID::LBACKBONECONTACT,       //pages
    4150u16 => ID::LCODABLECONCEPT,        //relationship
    4151u16 => ID::REFERENCE,              //Organization
    4152u16 => ID::LBACKBONECOMMUNICATION, //Organization
    4153u16 => ID::BOOLEAN,                //preferred
    4154u16 => ID::LREFERENCE,             //preferred
    4155u16 => ID::REFERENCE,              //preferred
    4156u16 => ID::LBACKBONELINK,          //preferred
    4157u16 => ID::REFERENCE,              //preferred

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
    520u16,  //ADDRESS
    521u16,  //ATTACHMENT
    522u16,  //BACKBONECONTACT
    523u16,  //BACKBONECOMMUNICATION
    524u16,  //BACKBONELINK
    2048u16, //LHUMANNAME
    2049u16, //LIDENTIFIER
    2050u16, //LCODING
    2051u16, //LCONTACTPOINT
    2052u16, //LADDRESS
    2053u16, //LATTACHMENT
    2054u16, //LBACKBONECONTACT
    2055u16, //LCODEABLECONCEPT
    2056u16, //LBACKBONECOMMUNICATION
    2057u16, //LBACKBONELINK
    2058u16, //LREFERENCE
};

///Mapping of [`TypeClass`] to ID
static MULTIPLE_DECEASED: phf::Map<u16, u16> = phf_map! {
   2u16 => 2u16, //BOOLEAN/BOOLEAN 
   0u16 => 6u16, //SRING/DATETIME
};

///Mapping of [`TypeClass`] to ID
static MULTIPLE_MULTIPLE_BIRTH: phf::Map<u16, u16> = phf_map! {
   2u16 => 2u16, //BOOLEAN/BOOLEAN
   1u16 => 9u16, //NUMERIC/INTEGER
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

static ADDRESS_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4102u16 => ID::CODE,        //use [code] 0..1
    4106u16 => ID::CODE,        //type [code] 0..1
    4098u16 => ID::STRING,      //text [STRING] 0..1
    4127u16 => ID::LSTRING,     //line [string] 0..*
    4128u16 => ID::STRING,      //city [string] 0..1
    4129u16 => ID::STRING,      //district [string] 0..1
    4130u16 => ID::STRING,      //state [string] 0..1
    4131u16 => ID::STRING,      //postalcode [string] 0..1 
    4132u16 => ID::STRING,      //country [string] 0..1
    4109u16 => ID::PERIOD,      //period [Period] 0..1
};


static ATTACHMENT_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4135u16 => ID::CODE,         //contentType [code] 0..1
    4136u16 => ID::CODE,         //language [code] 0..1
    4137u16 => ID::BASE64BINARY, //data [base64-binary] 0..1
    4138u16 => ID::URL,          //url [url] 0..1
    4139u16 => ID::INTEGER64,    //size [integer64] 0..1
    4140u16 => ID::BASE64BINARY, //hash [base64-binary] 0..1
    4141u16 => ID::STRING,       //title [STRING] 0..1
    4142u16 => ID::DATETIME,     //creation [dateTime] 0..1
    4143u16 => ID::POSITIVEINT,  //height [positiveInt] 0..1
    4144u16 => ID::POSITIVEINT,  //width [positiveInt] 0..1
    4145u16 => ID::POSITIVEINT,  //frames [positiveInt] 0..1
    4146u16 => ID::DECIMAL,      //duration [decimal] 0..1
    4147u16 => ID::POSITIVEINT,  //pages [positiveInt] 0..1
};

static BACKBONECONTACT_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4150u16 => ID::LCODABLECONCEPT,  //relationship  [LCODABLECONCEPT] 0..*
    4101u16 => ID::HUMANNAME,        //name          [HUMANNAME] 0..1
    4120u16 => ID::LCONTACTPOINT,    //telecom       [LCONTACTPOINT] 0..*
    4126u16 => ID::ADDRESS,          //address       [ADDRESS] 0..1
    4122u16 => ID::CODE,             //gender        [CODE] 0..1
    4151u16 => ID::REFERENCE,        //organization  [Reference(Organization)] 0..1
    4109u16 => ID::PERIOD,           //period        [PERIOD] 0..1
};

static BACKBONECOMMUNICATION_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4137u16 => ID::CODABLECONCEPT, //language [CODEABLECONCEPT] 1..1
    4153u16 => ID::BOOLEAN         //preferred [BOOLEAN] 1..1
};

static BACKBONELINK_EXPECTS: phf::Map<u16, ID> = phf_map! {
    4157u16 => ID::REFERENCE, //other [Reference(Patient)] 1..1
    4106u16 => ID::CODE         //type [CODE] 1..1
};


pub fn copy_multiple<I: Into<u16>>(id: I) -> collections::HashMap<u16, u16> {
    let mut map = collections::HashMap::<u16, u16>::new();
    match id.into() {
        4124u16 => {
            for (k,v) in MULTIPLE_DECEASED.into_iter() {
                map.insert(*k, *v);
            }
            map
        },  //deceased
        4125u16 => {
            for (k,v) in MULTIPLE_MULTIPLE_BIRTH.into_iter() {
                map.insert(*k, *v);
            }
            map
        },//multiplebirth
        _ => unreachable!("must be a bug")
    }
}

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
        515 | 2055u16 => {
            CODEABLECONCEPT_EXPECTS.get(&expects_for).cloned()
        },
        516u16 => {
            PERIOD_EXPECTS.get(&expects_for).cloned()
        },  
        517u16 | 2058 => {
            REFERENCE_EXPECTS.get(&expects_for).cloned()
        },  
        518u16 | 2050u16 => {
            CODING_EXPECTS.get(&expects_for).cloned()
        },
        519u16 | 2051u16 => {
            CONTACTPOINT_EXPECTS.get(&expects_for).cloned()
        },
        520u16 | 2052u16 => {
            ADDRESS_EXPECTS.get(&expects_for).cloned()
        },
        521u16 | 2053u16 => {
            ATTACHMENT_EXPECTS.get(&expects_for).cloned()
        },
        522u16 | 2054u16 => {
            BACKBONECONTACT_EXPECTS.get(&expects_for).cloned()
        }
        523u16 | 2056u16 => {
            BACKBONECOMMUNICATION_EXPECTS.get(&expects_for).cloned()
        },
        524u16 | 2057u16 => {
            BACKBONELINK_EXPECTS.get(&expects_for).cloned()
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

