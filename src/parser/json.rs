use crate::datatypes::id::{ID, get_expects, get_from_sub, get_key_id, ID_LEN, copy_multiple, TypeClass};
use crate::error::{Result, Error};
use crate::resourcetypes::ResourceId;
use crate::store::resourcewriter::ResourceWriter;
use super::datetime::Fhir_DateTime;
use super::stacks::*;

use std::ops::{AddAssign, MulAssign};
use std::ptr::slice_from_raw_parts;
use std::str;

pub fn from_json(src: &[u8]) -> Vec<u8> {
    let mut parser = JsonParser {
        src,
        cursor: 0,
        writer: ResourceWriter::new(ResourceId::Patient).unwrap(),
        lengths: LengthStack::default(),
        keys: KeyStack::default(),
        after_comma: false
    };
    parser.parse();
    parser.get_buffer()
}




struct JsonParser<'p> {
    src: &'p[u8],
    cursor: usize,
    writer: ResourceWriter,
    lengths: LengthStack,
    keys: KeyStack,
    after_comma: bool 

}

impl<'p> JsonParser<'p> {
    
    fn print_buffer(&self) {
        let data = unsafe {&*slice_from_raw_parts(self.writer.get_mut_ptr(), self.writer.len())};
        let relevant = data.to_vec().drain(self.writer.get_header_len()..).collect::<Vec<u8>>();
        println!("==================================================================");
        println!("DATA: {:?}", relevant);
        println!("DATA LENGTH: {}",relevant.len());
        println!("WRITER LEN (HEADER ADJ): {}", self.writer.len() - self.writer.get_header_len());
        println!("LengthCollector DATA: {:?}", self.lengths);
        println!("==================================================================");
    }

    fn get_buffer(&self) -> Vec<u8> {
        let data = unsafe {&*slice_from_raw_parts(self.writer.get_mut_ptr(), self.writer.len())};
        data.to_vec().drain(self.writer.get_header_len()..).collect::<Vec<u8>>()
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
                self.eat_whitespace();
                let len = self.writer.len();
                self.writer.reserve_two().unwrap();
                self.lengths.push(len);
                if self.keys.len() > 0 {
                    if let Some(k) = self.keys.last() {
                        if k.is_general_purpose() {
                            //insert gp id and length
                            self.writer.set_u16(2u16).unwrap();
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
                self.eat_whitespace();
                let offset = self.writer.len();
                if let Some((location, length)) = self.lengths.get_length(offset) {
                    self.writer.set_u16_at(length-2, location).unwrap();
                }
                self.keys.pop();
                return self.parse()
            }
            b'[' => {
                self.eat_char();
                self.eat_whitespace();
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
                self.eat_whitespace();
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
            },
            //TODO handle signed numbers
            //TODO handle floats/decimals
            b'0'..=b'9' => {
                match self.parse_numeric() {
                    Ok(_) => self.parse(),
                    Err(err) => panic!("{}", err)
                };
            },
            b't' | b'f' => {
                self.set_bool(*ch);
                return self.parse()
            }
            _ => panic!("UNKNOWN CHARACTER -> {}, data: {}", *ch as char, String::from_utf8(self.src.to_vec()).unwrap())
        }
    }

    fn consume_while<P: FnMut(u8) -> bool>(&mut self, mut pred: P) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
        while self.peek_char().is_some() && pred(*self.peek_char().unwrap()) {
            result.push(self.next_char().unwrap());
        }
        result
    }

    //will most likely only be used at [`ID::STRING`] and any numeric [`ID`]
    fn handle_multiple(&mut self, class: TypeClass) {
        if self.keys.last_is_mulitple() {
            match class {
                TypeClass::STRING | TypeClass::NUMERIC => {
                    let multies = self.keys.multiple.clone();
                    self.keys.pop();
                    if let Some(id) = multies.get(&(class as u16)) {
                        if let Ok(key) = ID::try_from(*id) {
                            self.keys.push(key)
                        }
                    }
                },
                TypeClass::BOOLEAN => {
                    self.keys.pop();
                },
            }
        }
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

    fn parse_numeric(&mut self) -> Result<()> {
        self.handle_multiple(TypeClass::NUMERIC);
        if let Some(key) = self.keys.last() {
            match key {
                ID::POSITIVEINT => {
                    let value: i32 = self.parse_unsigned()?;
                    self.set_unit(ID::POSITIVEINT, &mut value.to_be_bytes());
                    Ok(())
                },
                ID::INTEGER => {
                    //TODO: can be minus too
                    let value: i32 = self.parse_unsigned()?;
                    self.set_unit(ID::INTEGER, &mut value.to_be_bytes());
                    Ok(())
                },
            _ => return Err(Error::Expected("Integer".to_string(), "something else".to_string()))
            }
        } else {
           return Err(Error::UnknownExpect)
        }
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_char().unwrap() {
            ch @ b'0'..=b'9' => T::from(ch - b'0'),
            _ => return Err(Error::Expected("Integer".to_string(), "something else".to_string()))
        };

        while self.peek_char().is_some() {
            if !self.peek_char().unwrap().is_ascii_digit() {
                return Ok(int)
            }
            match self.next_char() {
                Some(ch @ b'0'..=b'9') => {
                    int *= T::from(10);
                    int += T::from(ch-b'0');
                },
                 _ => {
                     return Ok(int)
                 } 
            }
        }
        Ok(int)
    }



    fn parse_primitive_list(&mut self) {
        let offset = self.writer.len();
        let _ = self.writer.reserve_two();
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
        let _ = self.writer.reserve_two();
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
        self.handle_multiple(TypeClass::STRING);
        match self.keys.last() {
            Some(key) => {
                if *key == ID::DATE || *key == ID::DATETIME {
                    //TODO handle Error
                    let as_str = str::from_utf8(data).unwrap();
                    let dt = Fhir_DateTime::from_string(as_str).unwrap();
                    self.set_unit(key.clone(), &mut dt.timestamp_millis_bytes())
                } else {
                    self.set_unit(key.clone(), data)
                }
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
        println!("FOUND KEY: {}", String::from_utf8(key_bytes.clone()).unwrap());
        if let Some(key_id) = get_key_id(&key_bytes) {
            if self.keys.len() > 0 {
                println!("KeyStack has members {:?}", self.keys);
                //if last is gp take the get_expected as function
                if self.keys.last_is_general_purpose() {
                    println!("Key is general purpose");
                    let k = self.keys.last().unwrap();
                    if let Some(expects) = get_from_sub::<u16>(*k as u16, key_id.clone().into()) {
                        println!("Key expects {:?}", expects);
                        if expects.is_multiple() {
                            let map = copy_multiple(key_id.clone());
                            self.keys.push_multiples(map);
                        }
                        self.keys.push(expects);
                    } else {
                        panic!("No EXPECTS found at general purpose.")
                    }
                } else {
                    println!("Key is no general purpose");
                    if let Some(expects) = get_expects::<u16>(key_id.clone().into()) {
                        if expects.is_multiple() {
                            let map = copy_multiple(key_id.clone());
                            self.keys.push_multiples(map);
                        }
                        println!("Key expects {:?}", expects);
                        self.keys.push(expects);
                    } else {
                        panic!("No EXPECTS found.")
                    }
                }
                let _ = self.writer.set_u16(ID_LEN);
                let _ = self.writer.set_u16(key_id);
                self.check_n_eat(b'"', "set_key() has keys");
            } else {
                println!("KeyStack has no members");
                if let Some(expects) = get_expects::<u16>(key_id.clone().into()) {
                    println!("Key expects {:?}", expects);
                        if expects.is_multiple() {
                            let map = copy_multiple(key_id.clone());
                            self.keys.push_multiples(map);
                        }
                    self.keys.push(expects);
                } else {
                    panic!("No EXPECTS found.")
                }
                let _ = self.writer.set_u16(ID_LEN);
                let _ = self.writer.set_u16(key_id);
                self.check_n_eat(b'"', "set_key() has no keys");
                }
            } else {
                self.print_buffer();
                panic!("Unknown Key {}", String::from_utf8(key_bytes).unwrap());
            }
    }
}





#[cfg(test)]
mod test {
    use super::*;
    use crate::store::bufreader::read_buffer;   
    use std::{fs::read_to_string, fs::File, io::Read};

    fn assert_data(expects: Vec<u8>, result: Vec<u8>) {
        for (i, byte) in expects.iter().enumerate() {
            if *byte != result[i] {
                println!("ERROR AT: {} -> expected {} got {}", i, byte, result[i]);
            }
        }
    }

    fn parse_byte_file(f: &str) -> Vec<u8>  {
        let mut result = Vec::<u8>::new();
        for line in read_to_string(f).unwrap().lines() {
            let relevant = match line.split_once("//") {
                Some(data) => data.0,
                None => line
            };
            let bytes: Vec<u8> = relevant
                .trim()
                .split(" ")
                .filter(|x| !x.is_empty())
                .map(|x| x.parse::<u8>().unwrap()).collect();
            result.extend(bytes)
        }
        result
    }

    #[test]
    fn json_parse_key_value() {
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
    fn json_parse_several_key_values() {
        let data = br#"{"resourceType": "patient", "active": true}"#;
        let expects: Vec<u8> = vec![0, 24, 0, 2, 16, 0, 0, 9, 0, 1, 112, 97, 116, 105, 101, 110, 116, 0, 2, 16, 1, 0, 3, 0, 2, 1];
        let read: Vec<u16> = vec![24, 2, 4096, 9, 1, 2, 4097, 3, 2];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }
    #[test]
    fn json_parse_obj_as_value() {
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
    fn json_parse_list_of_primitives() {
        let data = br#"{"given": ["Rainer", "Maria"]}"#;
        //                          len  id_l key   len  type len data                          len data                     
        let expects: Vec<u8> = vec![0,23, 0,2, 16,7, 0,17, 0,22, 0,6, 82, 97, 105, 110, 101, 114, 0,5, 77, 97, 114, 105, 97];
        let read: Vec<u16> =vec![23, 2, 4103, 17, 22, 6,5];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }

    #[test]
    fn json_parse_list_of_obj() {
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

    #[test]
    fn json_parse_numerics() {
        //positive int
        let data = br#"{"rank": 123456}"#;
        let expects: Vec<u8> = vec![0, 12, 0, 2, 16, 25, 0, 6, 0, 7, 0, 1, 226, 64];
        let read: Vec<u16> = vec![12, 2, 4121, 6, 7];
        let result = from_json(data);
        assert_eq!(result, expects);
        assert_eq!(read_buffer(&result), read);
    }


    #[test]
    fn json_parse_multiple() {
        let data_is_boolean = br#"{"deceased": true}"#;
        let data_is_date_time = br#"{"deceased": "2015-02-07T13:28:17-05:00"}"#;
        let data_is_integer = br#"{"multipleBirth": 1}"#;
        let expects_boolean: Vec<u8> = vec![0, 9, 0, 2, 16, 28, 0, 3, 0, 2, 1];
        let result_boolean = from_json(data_is_boolean);
        assert_eq!(result_boolean, expects_boolean);

        let expects_date_time: Vec<u8> = vec![0, 16, 0, 2, 16, 28, 0, 10, 0, 6, 0, 0, 1, 75, 101, 76, 165, 232];
        let result_date_time = from_json(data_is_date_time);
        assert_eq!(result_date_time, expects_date_time);
        let expects_integer: Vec<u8>   = vec![0, 12, 0, 2, 16, 29, 0, 6, 0, 9, 0, 0, 0, 1];
        let result_integer = from_json(data_is_integer);
        assert_eq!(result_integer, expects_integer);

    }

    //#[test]
    fn json_parse_patient() {
        let mut fd = File::open("test_data/general_person_example_no_extension.json").unwrap();
        let mut data = Vec::<u8>::new();
        fd.read_to_end(&mut data).unwrap();
        let expects = parse_byte_file("test_data/general_person_example_bytes.txt");
        let result = from_json(&data);


    }
}
