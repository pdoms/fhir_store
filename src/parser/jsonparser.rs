use crate::data::datatype::{StoreId, has_sub, get_from_sub, u16_from_usize};
use crate::error::{Result, Error};
use crate::store::writer::Resource;
use crate::data::ResourceId;
use crate::data::datatype::{key_for_str,ToStoreWith, ToStore, get_expects};
use super::lengths::Lengths;

#[derive(PartialEq, Eq)]
enum CompoundId {
    OBJ,
    LIST,
    NONE
}

pub struct JsonParser<'s> {
    resource: ResourceId,
    src: &'s[u8],
    key: Option<StoreId>,
    rsrc: Resource,
    lengths: Lengths,
    len: usize,
    compounds: Vec<CompoundId>
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
            len: 0,
            compounds: Vec::<CompoundId>::new()
        })
    }

    pub fn flush(&mut self) -> *mut u8 {
        self.rsrc.get_mut_ptr()
    }

    fn finalize(&mut self) {
        let (loc, length) = self.lengths.pop().unwrap();
        println!("NOTE SURE!!!! len: {} loc: {loc} length: {length}", self.rsrc.len());
        self.len = self.rsrc.len();
        println!("SETTING {length} at {loc}");
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

    fn consume_string(&mut self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
        while self.peek_char().is_some() {
            let ch = self.next_char().unwrap();
            if ch == b'\\' {
                if let Some(peeked) = self.peek_char() {
                    match *peeked {
                        b'"' => {
                            //result.push(ch);
                            result.push(self.next_char().unwrap());
                        },
                        _ => result.push(ch)
                    }
                }
            } else if ch == b'"' {
                return result
            } else {
                result.push(ch);
            }
        }
        result
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


    fn get_expected(&self, key: u16) -> Option<StoreId> {
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

    fn parse_str(&mut self) -> Result<()> {
        if let Some(key) = &self.key {
            let expects = self.get_expected((*key).into());
            if let Some(exp) = expects {
                if exp.is_primitive() {
                    let parsed = self.consume_string();
                    println!("Processing String: {s}", s = String::from_utf8(parsed.clone()).unwrap());
                    let peek = self.peek_char();
                    if peek.is_some() {
                        if peek.unwrap() == &b'"' {
                            self.eat_char();
                        }  
                    }
                    let dt_str_var = parsed.to_store_with(exp)?;
                    let b = self.rsrc.set_data_type(dt_str_var)?;
                    self.lengths.add_to_last(b as u16);
                } else {
                    let mut values = Vec::<Vec<u8>>::new();
                    loop {
                        let parsed = self.consume_string();
                        println!("Processing LString: {s}", s = String::from_utf8(parsed.clone()).unwrap());
                        values.push(parsed);
                        self.eat_ws();
                        let peek = self.peek_char();
                        if peek.is_some() {
                            if peek.unwrap() == &b',' {
                                self.eat_char();
                                self.eat_ws();
                                let peek = self.peek_char(); 
                                if peek.is_some() && peek.unwrap() == &b'"' {
                                    self.eat_char()
                                }
                            } else {
                                break;
                            } 
                        } else {
                            break;
                        }
                    }
                    let len_at = self.rsrc.len();
                    self.rsrc.reserve_length()?;
                    self.rsrc.set_u16(exp.into())?;
                    let mut total_len: u16 = 0;
                    for value in values {
                        //len and data only 
                        let dt = value.to_store_with(exp)?;
                        self.rsrc.set_data_type(dt)?;
                    }
                    //end of list here as len = 2, end of list, and list type 
                    //where data usually goes
                    self.rsrc.set_u16(2)?;
                    let b = self.rsrc.set_u16(StoreId::ENDOFLIST.into())?;
                    //subtracting 2 as we don't write the store id to the buffer 
                    total_len += u16_from_usize(b-2)?;
                    self.rsrc.set_u16_at(total_len, len_at)?;
                    self.lengths.add_to_last(total_len);
                }
                self.key = None;
                Ok(())
            } else {
                todo!("NIY2");
            }
        } else {
            todo!("NIY3 Key: {:?}", self.key);
        }
    }

    fn parse_obj(&mut self) -> Result<()> {
        println!("FOUND OBJ");
        //push length item with current writer cursor
        self.lengths.push(self.rsrc.len());
        //reserve two for length
        let _ = self.rsrc.reserve_length();
        self.lengths.add_to_last(2);
        let key = self.parse_key()?;
        self.compounds.push(CompoundId::OBJ);
        println!("FOUND OBJ KEY {key:?}");
        //set dt in Resource
        self.key = Some(key.clone());
        let b = self.rsrc.set_u16(key as u16)?;
        self.lengths.add_to_last(b as u16);
        Ok(())
    }

    fn finish_obj(&mut self) {
        println!("FINISHING OBJ");
        //TODO check if ok to pop
        self.compounds.pop();
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

    fn parse_list(&mut self) -> Result<()> {
        println!("FOUND LIST");
        self.lengths.push(self.rsrc.len());
        let _ =self.rsrc.reserve_length();
        self.lengths.add_to_last(2);
        if let Some(key) = self.key {
            let b = self.rsrc.set_u16(key as u16)?;
            println!("LIST KEY: {}", key as u16);
            self.lengths.add_to_last(b as u16);
            self.compounds.push(CompoundId::LIST);
        }
        Ok(())
    }


    fn finish_list(&mut self) {
        println!("FINISHING LIIST");
        //TODO check if ok to pop
        self.compounds.pop();
        match self.lengths.pop() {
            Some(len) => {
                self.rsrc.set_u16_at(len.1, len.0).unwrap();
                self.eat_char();
            },
            None => {
                panic!("None outcome for length shortening not yet implemented")
            }
        }
    }
    
    fn parse_key(&mut self) -> Result<StoreId> {
        let key = self.consume_while(|c| c == b'"');
        let key_str = match String::from_utf8(key) {
            Ok(s) => s,
            Err(err) => {return Err(Error::Custom(err.to_string()))}
        };
        println!("Processing key: {key_str}");
        if let Some(key) = key_for_str(&key_str) {
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
            b'{' => {
                self.consume_while(|x| x == b'"');
                self.eat_char();
                self.parse_obj()?;
                self.parse()
            },
            b'}' => {
                self.finish_obj();
                self.parse()
            },
            b'"' => {
                self.eat_char();
                self.parse_str()?;
                self.parse()
            },
            b'[' => {
               self.eat_char();
               self.parse_list()?;
               self.parse() 
            },
            b']' => {
                self.finish_list();
                self.parse()
            },
            //b'0'..=b'9' => {},
            //b'-'        => {},
            b't' | b'f' => {
                self.parse_bool()?;
                self.parse()
            },
            b',' => {
                self.eat_char();
                self.eat_ws();
                if self.compounds.len() > 0 && self.compounds.last().unwrap() == &CompoundId::OBJ {
                    if let Some(ch) = self.peek_char() {
                        if *ch == b'"' {
                            self.eat_char();
                        } else {
                            return Err(Error::Expected("\"".to_string(), ch.to_string()))
                        }
                    }
                    let key = self.parse_key()?;
                    println!("KEY AFTER COMMA: {key:?}");
                    //set dt in Resource
                    self.key = Some(key.clone());
                    //adding length of key, for flow
                    let a = self.rsrc.set_u16(2)?;
                    let b = self.rsrc.set_u16(key as u16)?;
                    self.lengths.add_to_last((a + b) as u16);
                }
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
    use std::ptr::slice_from_raw_parts;

    use super::*;

    #[test]
    fn test_to_store() {
        let json = r#"{"resourceType": "Patient", "id": "anid", "active": true}"#;
        let mut parser = JsonParser::new_from_slice(json.as_bytes(), "Patient", 1).unwrap();
        assert_eq!(parser.resource, ResourceId::Patient);
        let _  = parser.parse();
        println!("LENGTH: {}", parser.len());
        println!("DATA: {:?}", unsafe {&*slice_from_raw_parts(parser.rsrc.buffer.as_ptr(), parser.len())})
    }
    
    #[test]
    fn test_to_store_obj() {
        let json = r#"{"resourceType": "Patient", "id": "anid", "active": true, "text": {
    "status" : "generated",
    "div" : "<div xmlns=\"http://www.w3.org/1999/xhtml\"><p style=\"border: 1px #661aff solid; background-color: #e6e6ff; padding: 10px;\"><b>Jim </b> male, DoB: 1974-12-25 ( Medical record number: 12345\u00a0(use:\u00a0USUAL,\u00a0period:\u00a02001-05-06 --&gt; (ongoing)))</p><hr/><table class=\"grid\"><tr><td style=\"background-color: #f3f5da\" title=\"Record is active\">Active:</td><td>true</td><td style=\"background-color: #f3f5da\" title=\"Known status of Patient\">Deceased:</td><td colspan=\"3\">false</td></tr><tr><td style=\"background-color: #f3f5da\" title=\"Alternate names (see the one above)\">Alt Names:</td><td colspan=\"3\"><ul><li>Peter James Chalmers (OFFICIAL)</li><li>Peter James Windsor (MAIDEN)</li></ul></td></tr><tr><td style=\"background-color: #f3f5da\" title=\"Ways to contact the Patient\">Contact Details:</td><td colspan=\"3\"><ul><li>-unknown-(HOME)</li><li>ph: (03) 5555 6473(WORK)</li><li>ph: (03) 3410 5613(MOBILE)</li><li>ph: (03) 5555 8834(OLD)</li><li>534 Erewhon St PeasantVille, Rainbow, Vic  3999(HOME)</li></ul></td></tr><tr><td style=\"background-color: #f3f5da\" title=\"Nominated Contact: Next-of-Kin\">Next-of-Kin:</td><td colspan=\"3\"><ul><li>Bénédicte du Marché  (female)</li><li>534 Erewhon St PleasantVille Vic 3999 (HOME)</li><li><a href=\"tel:+33(237)998327\">+33 (237) 998327</a></li><li>Valid Period: 2012 --&gt; (ongoing)</li></ul></td></tr><tr><td style=\"background-color: #f3f5da\" title=\"Patient Links\">Links:</td><td colspan=\"3\"><ul><li>Managing Organization: <a href=\"organization-example-gastro.html\">Organization/1</a> &quot;Gastroenterology&quot;</li></ul></td></tr></table></div>"
  },"name" : [{
    "use" : "official",
    "family" : "Chalmers",
    "given" : ["Peter",
        "James"]
  },
  {
    "use" : "usual",
    "given" : ["Jim"]
  },
  {
    "use" : "maiden",
    "family" : "Windsor",
    "given" : ["Peter",
        "James"]
  }]
}"#;
        let mut parser = JsonParser::new_from_slice(json.as_bytes(), "Patient", 1).unwrap();
        let _ = parser.parse();
        println!("DATA: {:?}", unsafe {&*slice_from_raw_parts(parser.rsrc.buffer.as_ptr(), parser.len())})
    }
}
