use crate::error::{Error, Result};
use crate::store::resource::{PAGE_SIZE, ResourceHeader}; 
use super::ResourceId;
use crate::datatypes::{StoreWith, DataType};


const UUID_LEN: usize = 16;

pub struct ResourceReader {
    pub inner: *mut u8,
    pub cursor: usize,
    pub header: ResourceHeader
}

impl ResourceReader {
    pub fn from_store_page(page: *mut u8) -> Result<Self> {
        let mut reader = Self {
            inner: page,
            cursor: 0,
            header: ResourceHeader::empty()
        };
        reader.construct_header()?;
        Ok(reader)
    }

    fn check_read(&self, adv: usize) -> Result<()> {
        if (self.cursor + adv) > PAGE_SIZE {
            return Err(Error::SegmentationFault);
        }
        Ok(())
    }

    fn construct_header(&mut self) -> Result<()> {
        let pn = self.read_u16()? as usize;
        let rsrc_id = ResourceId::try_from(self.read_u16()?)?;
        let id = self.read_n(16)?.as_slice().to_store_with(DataId::ID)?;
        let header = ResourceHeader::with_id(rsrc_id.clone(), pn, id.clone());
        self.header = header;
        self.cursor = 72;
        Ok(())
    }

    pub fn peek_u16(&self) -> Result<u16> {
        self.check_read(2)?;
        let bytes = unsafe {
             &*std::ptr::slice_from_raw_parts(self.inner.add(self.cursor), 2)
        };
        match bytes.try_into() {
            Ok(s) => Ok(u16::from_be_bytes(s)),
            Err(err) => Err(Error::Custom(err.to_string()))
        }
    }
 
    pub fn read_u16(&mut self) -> Result<u16> {
        self.check_read(2)?;
        let bytes = unsafe {
             &*std::ptr::slice_from_raw_parts(self.inner.add(self.cursor), 2)
        };
        self.cursor += 2;
        match bytes.try_into() {
            Ok(s) => Ok(u16::from_be_bytes(s)),
            Err(err) => Err(Error::Custom(err.to_string()))
        }
    }

    pub fn read_n(&mut self, n: usize) -> Result<Vec<u8>> {
        self.check_read(n)?;
        let bytes = unsafe {
             &*std::ptr::slice_from_raw_parts(self.inner.add(self.cursor), n)
        };
        self.cursor += n;
        Ok(bytes.to_vec())
    }

    fn read_unit(&mut self) -> Result<(DataType, u16)>{
        let len = self.read_u16()?;
        let id = self.read_u16()?;
        let data = self.read_n(len as usize)?;
        Ok((DataType::try_from_memory(id, len, &data)?, id))
    }

    pub fn to_tree(&mut self) {}

    

    //pub fn to_json(&mut self) -> Result<Vec<u8>> {
    //    let tot_len = self.read_u16()?;
    //    let len_obj = self.read_u16()?;
    //    let mut lengths = Vec::<u16>::new();
    //    lengths.push(tot_len);
    //    lengths.push(len_obj);

    //    let mut expects_prim = false;

    //    //even though json will have a different length than
    //    //the page representation, it still is a good start 
    //    let mut trgt = Vec::with_capacity(tot_len as usize);

    //    // open json 
    //    trgt.push(OBO);
    //    
    //    //while self.cursor < PAGE_SIZE {
    //        //read unit
    //        let (unit, id) => 
    //        println!("================> {:?}", self.read_unit().unwrap());
    //        //    panic!();
    //        //let _ = self.read_u16()?;
    //        //let id = self.read_u16()?;
    //        //
    //        //if !id_is_key(id) {
    //        //    //handle as data id
    //        //    let did = DataId::try_from(id).unwrap(); 
    //        //    match did {
    //        //        DataId::KEYID => {
    //        //            let keyid = self.read_u16().unwrap();
    //        //            trgt.push(b'"');
    //        //            trgt.extend_from_slice(get_key_inv(keyid).unwrap());
    //        //            trgt.extend_from_slice(b"\": ");
    //        //            //expects
    //        //            let expects = DataId::try_from(get_expects_u16(keyid).unwrap() as u16)?;
    //        //            expects_prim = expects.is_primitive();
    //        //        }
    //        //        _ => unimplemented!("TO JSON NOT IMPL YET")
    //        //    }
    //        //} else {
    //        //    //handle as key
    //        //}

    //        //if expects_prim {
    //        //    let prim = self.read_unit()?;
    //        //    let json_str = prim.to_json_bytes();
    //        //    trgt.extend_from_slice(&json_str);
    //        //}
 
    //    //}
    //    
    //   Ok(trgt)
    //}
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::store::store::Store;

    #[test]
    fn test_reader() {
        let mut store = Store::open().unwrap();
        let page = store.get_page_copy(1);
        println!("{page:?}");
        let mut reader = ResourceReader::from_store_page(store.get_page_copy(1).as_mut_ptr()).unwrap();
        let read = reader.to_json().unwrap();
        println!("JSON STRING {}", String::from_utf8(read).unwrap())
    }
}
