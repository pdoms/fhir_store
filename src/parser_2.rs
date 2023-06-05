use std::os::unix::raw::off_t;
use std::ptr::slice_from_raw_parts;

use crate::error::Error;
use crate::{store::writer::Resource, data::resource_reader};
use crate::data::ResourceId;
use phf::{phf_map, phf_set};

const HEADER_LEN: usize = 72;

const EOL:u16 = 21;
const GENERAL_PURPOSE: u16 = 512;
const GENERAL_PURPOSE_LIST: u16 = 2048;
const KEY_ID_START: u16 = 4096;

#[derive(Clone, Debug)]
#[repr(u16)]
enum ID {
    EMPTY,
    STRING,
    BOOLEAN,
    CODE,
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
    fn is_primitive(&self) -> bool {
        (*self as u16) < EOL 
    }

    fn is_primitive_list(&self) -> bool {
        let cast = *self as u16;
        cast > EOL && cast < GENERAL_PURPOSE
    }

    fn is_gp_list(&self) -> bool {
        let cast = *self as u16;
        cast >= GENERAL_PURPOSE_LIST && cast < KEY_ID_START
    }

    fn is_general_purpose(&self) -> bool {
        let cast = *self as u16;
        cast >= GENERAL_PURPOSE && cast < GENERAL_PURPOSE_LIST 
    }
    fn is_key(&self) -> bool {
        (*self as u16) >= KEY_ID_START
    }


    fn to_store(&self) -> [u8; 2] {
        (*self as u16).to_be_bytes()
    }

    
}

impl TryFrom<u16> for ID {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match IDS.get(&value) {
            Some(id) => Ok(id.clone()),
            None => Err("Damn not found {value}".to_string())
        }
    }
}
static IDS: phf::Map<u16, ID> = phf_map! {
       1u16    => ID::STRING,
       2u16    => ID::BOOLEAN,
       3u16    => ID::CODE,
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

fn get_expected(key: u16) -> Option<ID> {
    if let Some(exp) = get_expects(key) {
        let exp_u = exp as u16;
        if has_sub(exp_u) {
            return get_from_sub(key, exp_u)
        } else {
            return Some(exp)
        }
    }
    None
}

fn get_expects(exp: u16) -> Option<ID> {
    EXPECTS.get(&exp).cloned()
}

fn has_sub(id: u16) -> bool {
    HAS_SUPS.contains(&id)
}

fn get_from_sub(id: u16, expects_for: u16) -> Option<ID> {
    match id {
        513u16 | 2048u16 => {
            HUMANNAME_EXPECTS.get(&expects_for).cloned()
        },
        512u16 => {
            NARRATIVE_EXPECTS.get(&expects_for).cloned()
        }
        _ => None
    }
}

fn get_key_id(key: &[u8]) -> Option<ID> {
    if let Ok(s) = std::str::from_utf8(key) {
        KEYS.get(&s.to_ascii_lowercase()).cloned()
    } else {
        None
    }
}



pub fn from_json(src: &[u8]) -> Vec<u8> {
    let mut parser = JsonParser {
        src,
        cursor: 0,
        writer: Resource::new(ResourceId::Patient, 1).unwrap(),
        lengths: LengthCollector::default(),
        keys: KeyStack::default(),
        after_comma: false
    };
    parser.parse();
    parser.get_buffer()
}





#[derive(Default, Debug)]
struct LengthCollector {
    offsets: Vec<u16>,
}

impl LengthCollector {
    fn push(&mut self, offset: usize) {
        if let Ok(offs) = u16::try_from(offset) { 
            self.offsets.push(offs)
        }
    }
 
    fn get_length(&mut self, offset: usize) -> Option<(usize, u16)> {
        if let Some(last) = self.offsets.pop() {
            match u16::try_from(offset) {
                Ok(offs) => {
                    let calc_offset = offs - last;
                    Some((last as usize, calc_offset))},
                Err(_) => None
            }
        } else {
            match u16::try_from(offset) {
                Ok(offs) => {
                    Some((0, offs))},
                Err(_) => None
            }
        }
    }
} 


#[derive(Default, Debug, Clone)]
struct KeyStack {
    keys: Vec<ID>
}

impl KeyStack {

    fn len(&self) -> usize {
        self.keys.len()
    }
    fn push(&mut self, k: ID) {
        self.keys.push(k)
    }
    fn pop(&mut self) {
        if self.keys.len() > 0 {
            self.keys.pop();
        }  
    }

    fn last(&self) -> Option<&ID> {
        if self.keys.len() > 0 {
            self.keys.last()
        } else {
            None
        }
    }

    fn last_is_general_purpose(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_general_purpose()
        } else {
            false
        }
    }

    fn last_is_general_purpose_list(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_gp_list()
        } else {
            false
        }
    }
    fn last_is_primitive_list(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_primitive_list()
        } else {
            false
        }
    }
}



struct JsonParser<'p> {
    src: &'p[u8],
    cursor: usize,
    writer: Resource,
    lengths: LengthCollector,
    keys: KeyStack,
    after_comma: bool 

}

impl<'p> JsonParser<'p> {
    
    fn print_buffer(&self) {
        let data = unsafe {&*slice_from_raw_parts(self.writer.buffer.as_ptr(), self.writer.len())};
        let relevant = data.to_vec().drain(HEADER_LEN..).collect::<Vec<u8>>();
        println!("==================================================================");
        println!("DATA: {:?}", relevant);
        println!("DATA LENGTH: {}",relevant.len());
        println!("WRITER LEN (HEADER ADJ): {}", self.writer.len() - HEADER_LEN);
        println!("LengthCollector DATA: {:?}", self.lengths);
        println!("==================================================================");
    }

    fn get_buffer(&self) -> Vec<u8> {
        let data = unsafe {&*slice_from_raw_parts(self.writer.buffer.as_ptr(), self.writer.len())};
        data.to_vec().drain(HEADER_LEN..).collect::<Vec<u8>>()
    }

    fn eat_char(&mut self) {
        self.src = &self.src[1..];
    }

    fn peek_char(&self) -> Option<&u8>  {
        if self.src.len() == 0 {
            None
        } else {
            Some(&self.src[0])
        }
    }
    fn next_char(&mut self) -> Option<u8> {
        if self.peek_char().is_none() {
            None
        } else {
            let ch = self.src[0];
            self.eat_char();
            Some(ch)
        }
    }
    fn eat_chars(&mut self, num: usize) {
        for _ in 0..num {
            self.eat_char();
        }
    }

    fn eat_whitespace(&mut self) {
        let _ = self.consume_while(|c| c.is_ascii_whitespace());
    }

    fn next_is(&self, target: u8) {
        assert!(self.peek_char().is_some() && self.peek_char().unwrap() == &target, "EXPECTED {target}")
    }
    fn check_n_eat(&mut self, target: u8, call: &str) {
        if let Some(ch) = self.peek_char() {
            if ch == &target {
                self.eat_char();
            } else {
                panic!("EXPECTED HERE '{}' got '{}' called by {call}", target as char, *ch as char)
            }
        } else {
            panic!("EXPECTED '{}'", target as char)
        }
    }



    fn parse(&mut self) {
        self.eat_whitespace(); 
        if self.peek_char().is_none() {
            //println!("PARSING DONE");
            //self.print_buffer();
            return
        }
        let ch = self.peek_char().unwrap();
        match *ch {
            b'{' => {
                self.eat_char();
                let len = self.writer.len();
                self.writer.reserve_length().unwrap();
                self.lengths.push(len);
                if self.keys.len() > 0 {
                    if let Some(k) = self.keys.last() {
                        if k.is_general_purpose() {
                            //insert gp id and length
                            self.writer.set_u16(2).unwrap();
                            self.writer.set_u16(*k as u16).unwrap();
                        }
                    }
                }
                self.after_comma = false;
                self.set_key();
                return self.parse()
            },
            b'}' => {
                self.eat_char();
                let offset = self.writer.len();
                if let Some((location, length)) = self.lengths.get_length(offset) {
                    self.writer.set_u16_at(length-2, location).unwrap();
                }
                return self.parse()
            }
            b'[' => {
                self.eat_char();
                //check expected
                if self.keys.last_is_primitive_list() {
                    self.parse_primitive_list();   
                }
                if self.keys.last_is_general_purpose_list() {
                    self.prepare_gp_list();
                    return self.parse()
                }
                return self.parse()
            },
            b']' => {
                self.eat_char();
                let offset = self.writer.len();
                if let Some((location, length)) =self.lengths.get_length(offset) {
                    self.writer.set_u16_at(length-2, location).unwrap();
                }
                self.keys.pop();
                return self.parse()
            },
            b'"' => {
                let mut data = self.parse_string();
                self.set_string(&mut data);
                self.keys.pop();
                return self.parse()
            },
            b':' => {
                self.eat_char();
                return self.parse()
            },
            b',' => {
                self.eat_char();
                self.eat_whitespace();
                if let Some(peeked) =  self.peek_char() {
                    if peeked == &b'{' || peeked == &b'[' {
                        self.after_comma = true;
                        return self.parse()
                    }
                }
                self.set_key();
                return self.parse()
            }

            b't' | b'f' => {
                self.set_bool(*ch);
                return self.parse()
            }
            _ => panic!("UNKNOWN CHARACTER -> {}", *ch as char)
        }
    }

    fn consume_while<P: FnMut(u8) -> bool>(&mut self, mut pred: P) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
        while self.peek_char().is_some() && pred(*self.peek_char().unwrap()) {
            result.push(self.next_char().unwrap());
        }
        result
    }

    fn parse_string(&mut self) -> Vec<u8> {
        self.check_n_eat(b'"', "parse_string()");
        let mut result = Vec::<u8>::new(); 
        while self.peek_char().is_some() {
            let ch = self.next_char().unwrap();
            if ch == b'\\' {
                if let Some(next) = self.peek_char() {
                    if next == &b'"' {
                        result.push(self.next_char().unwrap());
                    } else {
                        result.push(ch);
                    }
                }
            } else {
                if ch == b'"' {
                    return result
                } else {
                    result.push(ch)
                }
            }
        }
        result
    }


    fn parse_primitive_list(&mut self) {
        let offset = self.writer.len();
        let _ = self.writer.reserve_length();
        self.lengths.push(offset);
        if let Some(expects) = self.keys.clone().last() {
            let _ = self.writer.set_u16(*expects as u16);
            while self.peek_char().is_some() {
                match *expects {
                    ID::LSTRING => {
                        self.eat_whitespace();
                        let mut data = self.parse_string();
                        self.set_primitive_list_item(&mut data);
                    }
                    _ => unimplemented!("at parse_primitive_list")
                }
                if let Some(peeked) = self.peek_char() {
                    if peeked != &b',' {
                        break;
                    } else {
                        self.eat_char();
                        self.eat_whitespace();
                    }
                }
            }
        }
    }

    fn prepare_gp_list(&mut self) {
        let offset = self.writer.len();
        let _ = self.writer.reserve_length();
        self.lengths.push(offset);
        if let Some(expects) = self.keys.clone().last() {
            let _ = self.writer.set_u16(*expects as u16);
        }
    }

    fn set_unit(&mut self, id: ID, data: &mut [u8]) {
        let len = data.len();
        self.writer.set_u16(2u16 + len as u16).unwrap();
        self.writer.set_u16(id as u16).unwrap();
        self.writer.set(data.as_mut_ptr(), len).unwrap();
    }

    fn set_string(&mut self, data: &mut [u8]) {
        match self.keys.last() {
            Some(key) => {
                self.set_unit(key.clone(), data)
            }
            None => ()
        }
    }

    fn set_primitive_list_item(&mut self, data: &mut [u8]) {
        let len = data.len();
        self.writer.set_u16(len as u16).unwrap();
        self.writer.set(data.as_mut_ptr(), len).unwrap();
    }

    fn set_bool(&mut self, ch: u8) {
        match ch {
            b't' => {
                self.eat_chars(4);
                self.set_unit(ID::BOOLEAN, &mut [1]);
            },
            b'f' => {
                self.eat_chars(5);
                self.set_unit(ID::BOOLEAN, &mut [0]);
            }
            _ => unreachable!("this is likely a bug in the json parser.")
        }
    }

    fn set_key(&mut self) {
        self.check_n_eat(b'"', "set_key() first_check");
        let key_bytes = self.consume_while(|c| c != b'"');
        if let Some(key_id) = get_key_id(&key_bytes) {
            if self.keys.len() > 0 {
                //if last is gp take the get_expected as function
                if self.keys.last_is_general_purpose() {
                    let k = self.keys.last().unwrap();
                    if let Some(expects) = get_from_sub(*k as u16, key_id as u16) {
                        self.keys.push(expects);
                    }
                } else {
                    if let Some(expects) = get_expects(key_id as u16) {
                        self.keys.push(expects);
                    }
                }
                let _ = self.writer.set_u16(2);
                let _ = self.writer.set_u16(key_id as u16);
                self.check_n_eat(b'"', "set_key() has keys");
            } else {
                if let Some(expects) = get_expects(key_id as u16) {
                    self.keys.push(expects);
                }
                let _ = self.writer.set_u16(2);
                let _ = self.writer.set_u16(key_id as u16);
                self.check_n_eat(b'"', "set_key() has no keys");
                }
            } 
    }
}

fn read_buffer(mut buf: &[u8]) -> Vec<u16> {
    let len: [u8; 2] = buf[..2].try_into().unwrap();
    buf = &buf[2..];
    let mut result = Vec::<u16>::new();
    result.push(u16::from_be_bytes(len));
    let mut gp_list = 0;
    let mut current = 0;
    while buf.len() > 0 {
        if gp_list > 0 && current == 0 {
            let length: [u8; 2] = buf[..2].try_into().unwrap();
            buf = &buf[2..];
            let length = u16::from_be_bytes(length);
            result.push(length);
            //gp_list -= 2;
            current = length;
        }



        let len: [u8; 2] = buf[..2].try_into().unwrap();
        buf = &buf[2..];
        if gp_list > 0 {
            gp_list -= 2;
        }
        if current > 0 {
            current -= 2;
        }
        let pos_id: [u8; 2] = buf[..2].try_into().unwrap();
        let id = ID::try_from(u16::from_be_bytes(pos_id)).unwrap();
        buf = &buf[2..];
        if gp_list > 0 {
            gp_list -= 2;
        }
        if current > 0 {
            current -= 2;
        }
        let l = u16::from_be_bytes(len);
        result.push(l);
        result.push(id as u16);
        if let Some(expects) = get_expects(id as u16) {
            if expects.is_general_purpose() {
                let gp_len: [u8; 2] = buf[..2].try_into().unwrap();
                let le = u16::from_be_bytes(gp_len);
                result.push(le);
                buf = &buf[2..];
                if gp_list > 0 {
                    gp_list -= 2;
                }
                if current > 0 {
                    current -= 2;
                }
                continue;
            }
        }
        if id.is_primitive_list() {
            let mut temp_cur = 0;
            let trgt = l-2-4;
            if gp_list > 0 {
                gp_list -= l-2;
            }
            if current > 0 {
                current -= trgt;
            }
            while temp_cur < trgt {
                let length: [u8; 2] = buf[..2].try_into().unwrap();
                buf = &buf[2..];
                let length = u16::from_be_bytes(length);
                result.push(length);
                buf = &buf[length as usize..];
                temp_cur += 2 + length;
            }
        } else if id.is_gp_list() {
            gp_list = u16::from_be_bytes(len);
            let length: [u8; 2] = buf[..2].try_into().unwrap();
            //gp_list -= 2;
            buf = &buf[2..];
            let length = u16::from_be_bytes(length);
            result.push(length);
            current = length;
        } else if !id.is_key() {
            buf = &buf[(l - 2 )as usize..];
            if gp_list > 0 {
                gp_list -= l;
            }
            if current > 0 {
                current -= l;
            }
        }

    }
    return result
}



#[cfg(test)]
mod test {
    use super::*;
    

    fn assert_data(expects: Vec<u8>, result: Vec<u8>) {
        for (i, byte) in expects.iter().enumerate() {
            if *byte != result[i] {
                println!("ERROR AT: {} -> expected {} got {}", i, byte, result[i]);
            }
        }
    }

    #[test]
    fn parse_key_value() {
        let data = br#"{"resourceType": "patient"}"#;

        // HEADER (skip for now) -> // ResourceID [16]  | ResourceType [2] 
        // Total Length [2] 
        // BODY form byte 6 on
        // Length [2] | KeyId [2] | Length (7) [2] | ID (STRING) [2]  | DATA 
        let expects: Vec<u8> = vec![0, 15, 0, 2, 16, 0, 0, 9, 0, 1, 112, 97, 116, 105, 101, 110, 116];
        let read: Vec<u16> = vec![15, 2, 4096, 9, 1];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }

    #[test]
    fn parse_several_key_values() {
        let data = br#"{"resourceType": "patient", "active": true}"#;
        let expects: Vec<u8> = vec![0, 24, 0, 2, 16, 0, 0, 9, 0, 1, 112, 97, 116, 105, 101, 110, 116, 0, 2, 16, 1, 0, 3, 0, 2, 1];
        let read: Vec<u16> = vec![24, 2, 4096, 9, 1, 2, 4097, 3, 2];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }
    #[test]
    fn parse_obj_as_value() {
        let data = br#"{"text": {"status": "done", "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\">"}}"#;
                                        //    text          NARRATIVE     status
                                        //ilen gpid  olen  ilen gpid      key  
        let expects: Vec<u8> = vec![0,72, 0,2, 16,2, 0,66, 0,2, 2,0, 0,2, 16,3, 0,6, 0,3, 100, 111, 110, 101, 0,2, 16,4, 0, 44, 0,1,  60, 100, 105, 118, 32, 120, 109, 108, 110, 115, 61, 34, 104, 116, 116, 112, 58, 47, 47, 119, 119, 119, 46, 119, 51, 46, 111, 114, 103, 47, 49, 57, 57, 57, 47, 120, 104, 116, 109, 108, 34, 62];
        let read: Vec<u16> = vec![72, 2, 4098, 66, 2, 512, 2, 4099, 6,3, 2, 4100, 44,1];
        let result = from_json(data);
        assert_data(expects.clone(), result.clone());
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }

    #[test]
    fn parse_list_of_primitives() {
        let data = br#"{"given": ["Rainer", "Maria"]}"#;
        //                          len  id_l key   len  type len data                          len data                     
        let expects: Vec<u8> = vec![0,23, 0,2, 16,7, 0,17, 0,22, 0,6, 82, 97, 105, 110, 101, 114, 0,5, 77, 97, 114, 105, 97];
        let read: Vec<u16> =vec![23, 2, 4103, 17, 22, 6,5];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }

    #[test]
    fn parse_list_of_obj() {
        let data = br#"{"resourceType": "patient", "name": [{"use" : "official", "family" : "Chalmers", "given" : ["Peter", "James"]}, {"use" : "usual", "given": ["Jim"]}]}"#;
        //                        t_len  len    id    len    kid   data                               
        let expects: Vec<u8> = vec![0,107, 0,2, 16,0, 0, 9,  0, 1, 112, 97, 116, 105, 101, 110, 116, //16 
        //status          LHUMANNAME 
        //len kid   keylen             
        0,2,  16,5,  0, 86, 8,0, //8       
        //len -> data [HUMANNAME]
        0,54,  //2  
        //len, key, len,  id   data
        0,2,  16, 6,  0,10, 0,3, 111, 102, 102, 105, 99, 105, 97, 108, //16
        //len key  len  id   data  //16
        0,2,  16,8, 0,10, 0,1, 67, 104, 97, 108, 109, 101, 114, 115,   
        //len kid  len   id    len  data                   len data    //22
        0,2,  16,7, 0,16, 0,22, 0,5, 80, 101, 116, 101, 114, 0,5, 74, 97, 109, 101, 115, 
        //len -> data [HUMANNAME] 
        0,26, //2
        //len kid  len  id   data   /13
        0,2,  16,6, 0,7, 0,3, 117, 115, 117, 97, 108, 
        //len kid  len   id   len  data  //13
        0,2,  16,7, 0,7, 0,22, 0,3, 74, 105, 109, 
        ];

        let read: Vec<u16> =vec![107, 2, 4096, 9, 1, 2, 4101, 86, 2048, 
        54, 
        2, 4102, 10, 3,
        2, 4104, 10, 1,
        2, 4103, 16, 22, 5, 5,
        26,
        //LEFT OF HERE
        2, 4102, 7, 3,
        2, 4103, 7, 22, 3,
        ];
        let result = from_json(data);
        assert_data(expects.clone(), result.clone());
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);

    }
}
