use crate::error::{Result, Error};
use crate::store::memory::Memory;
use crate::resources::ResourceId;
use crate::datatypes::{KEY, get_key, Store, get_expects, StoreWith};

use super::lengths::Lengths;


//pub fn to_store<T: Read>(mut data: T) -> Memory {
//    let mut buffer = Vec::<u8>::new();
//    let _ = data.read_to_end(&mut buffer);
//    let mut parser = Parser::new(&buffer[..]);
//    parser.parse();
//    parser.membuf
//}





pub struct JsonParser<'s> {
    resource: ResourceId,
    src: &'s[u8],
    key: Option<KEY>,
    mem: Memory,
    lengths: Lengths 
}

impl<'s> JsonParser<'s> {
    
    pub fn new_from_slice(src: &'s[u8], resource: &str) -> Result<Self> {
        
        let mut mem = Memory::new()?;
        let resource = ResourceId::try_from(resource)?;
        let mut lengths = Lengths::default();
        //skip 2 bytes for total length
        mem.reserve_length()?;
        //push on stack
        lengths.push(0);
        //set resource id
        mem.set_u16(resource as u16)?;
        Ok(Self {
            src,
            resource,
            key: None,
            mem,
            lengths
        })
    }

    fn finalize(&mut self) {}

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




    //pub fn parse_str(&mut self) {
    //    let result = self.consume_while(|c| {c == b'"'});
    //    let acc_len;
    //    if self.key {
    //        let dt = DataType::from_key_bytes(&result);
    //        acc_len = dt.get_len();
    //        self.membuf.set_data_type(dt).unwrap();  
    //        self.key = false;
    //    } else {
    //        let dt = DataType::vec_u8_to_store(result,DataId::STR);
    //        acc_len = dt.get_len();
    //        self.membuf.set_data_type(dt).unwrap();
    //    }
    //    if self.lengths.len() > 0 {
    //        self.lengths.last_add(acc_len);
    //    } 
    //}
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
                self.mem.set_data_type(dt_str_var)?;
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
        //push length item with current writer cursor
        self.lengths.push(self.mem.len());
        //reserve two for length
        let _ = self.mem.reserve_length();
        let key = self.parse_key()?;
        //set dt in memory
        self.key = Some(key.clone());
        self.mem.set_data_type(key.to_store())?;
        Ok(())
    }

    fn finish_obj(&mut self) {

    }
    
    fn parse_key(&mut self) -> Result<KEY> {
        let key = self.consume_while(|c| c == b'"');
        let key_str = match String::from_utf8(key) {
            Ok(s) => s,
            Err(err) => {return Err(Error::Custom(err.to_string()))}
        };
        println!("Processing key: {key_str}");
        if let Some(key) = get_key(&key_str.to_ascii_lowercase()) {
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
            self.mem.set_data_type(true.to_store())
        } else {
            //parse false
            self.eat_chars("false".len());
            self.mem.set_data_type(false.to_store())
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        if self.peek_char().is_none() {
            //maybe call finalize here???
            return Ok(())
        }
        self.eat_ws();
        println!("Looking at: {}", *self.peek_char().unwrap() as char);
        // we know that we still have u8s left
        match *self.peek_char().unwrap() {
            b'{'        => {
                self.eat_chars(2);
                println!("Looking at: {}", self.peek_char().unwrap());
                self.parse_obj()?;
                self.parse()
            },
            b'}'        => {
                match self.lengths.pop() {
                    Some(len) => {
                        self.mem.set_u16_at(len.1, len.0).unwrap();
                        self.eat_char();
                    },
                    None => {
                        panic!("None outcome for length shortening not yet implemented")
                    }

                };
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
                //set dt in memory
                self.key = Some(key.clone());
                self.mem.set_data_type(key.to_store())?;
                self.parse()
            },
            b':' => {
                self.eat_char();
                self.parse()
            },
            _           => panic!("{}", Error::UnknownSyntaxToken(*self.peek_char().unwrap()).to_string()) 
    }
}

    
    //pub fn parse(&mut self) {
    //    if self.buf.len() <= 0 {
    //        return
    //    }

    //    let first = self.buf[0];
    //    match first {
    //        b' ' => {
    //            self.eat_char();
    //            return self.parse();
    //        }
    //        b'"' => {
    //            self.eat_char();
    //            self.parse_str();
    //            self.eat_char();
    //            return self.parse();
    //        },
    //        b'{' => {
    //            println!("OBJ AT {}", self.membuf.len);
    //            let curs = self.membuf.len;
    //            self.lengths.push(curs);
    //            self.membuf.advance_by(2);
    //            println!("SETTING LEN AT {}", self.membuf.len);
    //            println!("ID: {:?}", DataId::OBJ as u16);
    //            self.membuf.set_u16(DataId::OBJ as u16).unwrap();
    //            self.key = true;
    //            self.eat_char();
    //            return self.parse();
    //        },
    //        b'}' => {
    //            let len = self.lengths.pop();
    //            println!("stack len: {}", self.lengths.len());
    //            println!("loc: {} len: {}", len.0, len.1);
    //            self.membuf.set_u16_at(len.1, len.0).unwrap();
    //            self.eat_char();
    //            return self.parse();
    //        }
    //        _ => {
    //            self.eat_char();
    //            return self.parse()
    //        }
    //    }
    //}
}






#[cfg(test)]
mod test {
    use super::*;


    
    #[test]
    fn test_to_store() {
        let json = r#"{"resourceType": "Patient", "id": "anid", "active": true}"#;
        let mut parser = JsonParser::new_from_slice(json.as_bytes(), "Patient").unwrap();
        assert_eq!(parser.resource, ResourceId::Patient);
        let _  = parser.parse();
    }
}
