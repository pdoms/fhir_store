
use std::println;

use crate::error::{Result, Error};
use crate::store::resource::Resource;
use crate::resources::ResourceId;
use crate::datatypes::{KEY, get_key, Store, get_expects, StoreWith};

use super::lengths::Lengths;

pub struct JsonParser<'s> {
    resource: ResourceId,
    src: &'s[u8],
    key: Option<KEY>,
    rsrc: Resource,
    lengths: Lengths,
    len: usize
}

impl<'s> JsonParser<'s> {
    
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn new_from_slice(src: &'s[u8], resource: &str, page_num: usize) -> Result<Self> {
        
        let resource = ResourceId::try_from(resource)?;
        let mut rsrc = Resource::new(resource.clone(), page_num)?;
        let mut lengths = Lengths::default();
        //push on stack
        lengths.push(rsrc.len());
        //skip 2 bytes for total length
        rsrc.reserve_length()?;
        lengths.add_to_last(2);
        Ok(Self {
            src,
            resource,
            key: None,
            rsrc,
            lengths,
            len: 0
        })
    }

    pub fn flush(&mut self) -> *mut u8 {
        self.rsrc.get_mut_ptr()
    }

    fn finalize(&mut self) {
        let (loc, length) = self.lengths.pop().unwrap();
        println!("NOTE SURE!!!! len: {} loc: {loc} length: {length}", self.rsrc.len);
        self.len = self.rsrc.len();
        self.rsrc.set_u16_at(length, loc).unwrap();
    }

    fn eat_char(&mut self) {
        self.src = &self.src[1..];
    }
    fn eat_chars(&mut self, n: usize) {
        for _ in 0..n {
            self.eat_char();
        }
    }

    fn next_char(&mut self) -> Result<u8> {

        if self.src.len() == 0 {
            Err(Error::EOF)
        } else {
            let ch = Ok(self.src[0]);
            self.eat_char();
            ch
        }
    }
    fn peek_char(&self) -> Option<&u8> {
        if self.src.len() == 0 {
            None
        } else {
            Some(&self.src[0])
        }
    }

    fn eat_ws(&mut self) {
        self.consume_while(|c| !c.is_ascii_whitespace());
    }

    fn consume_while<F>(&mut self, mut p: F) -> Vec<u8> 
    where F: FnMut(u8) -> bool
    {
        let mut result = Vec::<u8>::new();
        while self.peek_char().is_some() && !p(self.src[0]) {
            //we checked that the next char exists.
            let ch = self.next_char().unwrap();
            result.push(ch);
        }
        result
    }

    fn parse_str(&mut self) -> Result<()> {
        let parsed = self.consume_while(|c| c == b'"');
        println!("Processing String: {s}", s = String::from_utf8(parsed.clone()).unwrap());
        let peek = self.peek_char();
        if peek.is_some() {
            if peek.unwrap() == &b'"' {
                self.eat_char();
            } else {
                return Err(Error::Expected("\"".to_string(), (*peek.unwrap() as char).to_string()))
            }
        }
        if let Some(key) = &self.key {
            let expects = get_expects(key);
            if let Some(exp) = expects {
                let dt_str_var = parsed.to_store_with(exp)?;
                let b = self.rsrc.set_data_type(dt_str_var)?;
                self.lengths.add_to_last(b as u16);
                self.key = None;
                Ok(())
            } else {
                todo!("NIY2");
            }
        } else {
            todo!("NIY3");
        }
    }

    fn parse_obj(&mut self) -> Result<()> {
        println!("FOUND OBJ");
        //push length item with current writer cursor
        self.lengths.push(self.rsrc.len());
        println!("{:?}", self.lengths);
        //reserve two for length
        let _ = self.rsrc.reserve_length();
        self.lengths.add_to_last(2);
        let key = self.parse_key()?;
        //set dt in Resource
        self.key = Some(key.clone());
        let b = self.rsrc.set_data_type(key.to_store())?;
        self.lengths.add_to_last(b as u16);
        Ok(())
    }

    fn finish_obj(&mut self) {
        match self.lengths.pop() {
            Some(len) => {
                self.rsrc.set_u16_at(len.1, len.0).unwrap();
                self.eat_char();
            },
            None => {
                panic!("None outcome for length shortening not yet implemented")
            }

        };
    }
    
    fn parse_key(&mut self) -> Result<KEY> {
        let key = self.consume_while(|c| c == b'"');
        let key_str = match String::from_utf8(key) {
            Ok(s) => s,
            Err(err) => {return Err(Error::Custom(err.to_string()))}
        };
        println!("Processing key: {key_str}");
        if let Some(key) = get_key(&key_str.to_ascii_lowercase()) {
            println!("KEY ID: {}", key as u16);
            let peek = self.peek_char();
            if peek.is_some() {
                if peek.unwrap() == &b'"' {
                    self.eat_char();
                } else {
                    return Err(Error::Expected("\"".to_string(), (*peek.unwrap() as char).to_string()))
                }
            }
            Ok(key)
        } else {
            return Err(Error::UnknownKeyInJson(key_str))
        }
    }


    fn parse_signed(&mut self) {}
    fn parse_unsigned(&mut self) {}

    fn parse_bool(&mut self) -> Result<()> {
        //it can not be None
        if *self.peek_char().unwrap() == b't' {
            //parse true
            self.eat_chars("true".len());
            let b = self.rsrc.set_data_type(true.to_store())?;
            self.lengths.add_to_last(b as u16);
            Ok(())
        } else {
            //parse false
            self.eat_chars("false".len());
            let b = self.rsrc.set_data_type(false.to_store())?;
            self.lengths.add_to_last(b as u16);
            Ok(())
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        self.eat_ws();
        if self.peek_char().is_none() {
            self.finalize();
            return Ok(())
        }
        // we know that we still have u8s left
        match *self.peek_char().unwrap() {
            b'{'        => {
                self.eat_chars(2);
                self.parse_obj()?;
                self.parse()
            },
            b'}'        => {
                self.finish_obj();
                self.parse()
            },
            b'"'        => {
                self.eat_char();
                self.parse_str()?;
                self.parse()
            },
            //b'['        => {},
            //b']'        => {},
            //b'0'..=b'9' => {},
            //b'-'        => {},
            b't' | b'f' => {
                self.parse_bool()?;
                self.parse()
            },
            b',' => {
                self.eat_char();
                self.eat_ws();
                if let Some(ch) = self.peek_char() {
                    if *ch == b'"' {
                        self.eat_char();
                    } else {
                        return Err(Error::Expected("\"".to_string(), ch.to_string()))
                    }
                }
                let key = self.parse_key()?;
                //set dt in Resource
                self.key = Some(key.clone());
                let b = self.rsrc.set_data_type(key.to_store())?;
                self.lengths.add_to_last(b as u16);
                self.parse()
            },
            b':' => {
                self.eat_char();
                self.parse()
            },
            _           => panic!("{}", Error::UnknownSyntaxToken(*self.peek_char().unwrap()).to_string()) 
        }
    }

}






#[cfg(test)]
mod test {
    use super::*;


    
    #[test]
    fn test_to_store() {
        let json = r#"{"resourceType": "Patient", "id": "anid", "active": true}"#;
        let mut parser = JsonParser::new_from_slice(json.as_bytes(), "Patient", 1).unwrap();
        assert_eq!(parser.resource, ResourceId::Patient);
        let _  = parser.parse();
        parser.finalize();
        assert!(parser.len() > 0);
    }
}
