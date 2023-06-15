use crate::datatypes::id::ID;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct LengthStack {
    offsets: Vec<u16>,
}

impl LengthStack {
    pub fn push(&mut self, offset: usize) {
        if let Ok(offs) = u16::try_from(offset) { 
            self.offsets.push(offs)
        }
    }
 
    pub fn get_length(&mut self, offset: usize) -> Option<(usize, u16)> {
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
pub struct KeyStack {
    pub keys: Vec<ID>,
    pub multiple: HashMap<u16, u16>  
}

impl KeyStack {

    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn push(&mut self, k: ID) {
        self.keys.push(k)
    }
    pub fn push_multiples(&mut self, map: HashMap<u16, u16>) {
        self.multiple = map;
    }

    pub fn pop(&mut self) {
        self.multiple = HashMap::new();
        if self.keys.len() > 0 {
            self.keys.pop();
        }  
    }

    pub fn last(&self) -> Option<&ID> {
        if self.keys.len() > 0 {
            self.keys.last()
        } else {
            None
        }
    }

    pub fn last_is_general_purpose(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_general_purpose()
        } else {
            false
        }
    }

    pub fn last_is_general_purpose_list(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_gp_list()
        } else {
            false
        }
    }
    pub fn last_is_primitive_list(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_primitive_list()
        } else {
            false
        }
    }

    pub fn last_is_mulitple(&self) -> bool {
        if let Some(last) = self.keys.last() {
            last.is_multiple()
        } else {
            false
        }
    }
}
