use std::ptr::NonNull;
use std::alloc::{Layout, alloc_zeroed};
use uuid::Uuid;
use crate::error::{Error, Result};
use crate::data::datatype::{DT, U16B, u16_bytes_from_usize, META_DATA_SIZE};
use crate::data::ResourceId;

use super::header::Head;


pub const PAGE_SIZE: usize = 4096;
const MEM_CAP: usize = PAGE_SIZE;
/// page num = 2; rsrc_id = 2; id = max 64 + 2 (id) + 2 (len)
/// -> total: 72
const HEADER_CAP: usize = 72;


#[derive(Debug)]
pub struct ResourceHeader {
    page_num: usize,
    rsrc_id: ResourceId,
    id: DT,
}

impl ResourceHeader {
    fn new(rsrc_id: ResourceId, page_num: usize) -> Self {
        //TODO should be v7
        let id = DT::ID(Box::new(Uuid::new_v4().into_bytes()));
        Self {
            rsrc_id,
            page_num,
            id,
        }
    }

    pub fn empty() -> Self {
        Self { page_num: 0, rsrc_id: ResourceId::Empty, id: DT::EMPTY }
    }

    pub fn with_id(rsrc_id: ResourceId, page_num: usize, id: DT) -> Self {
        Self {
            rsrc_id,
            page_num,
            id,
        }
    }

}

impl Head for ResourceHeader {
    fn to_store(&self) -> Result<Vec<u8>>  {
        let mut dst = Vec::<u8>::with_capacity(MEM_CAP);
        dst.extend(u16_bytes_from_usize(self.page_num)?);
        dst.extend((self.rsrc_id as u16).to_be_bytes());
        dst.extend(self.id.store()?);
        Ok(dst)
    }

    fn from_store(data: &[u8]) -> Self {
        todo!("from store")
    }
}



pub struct Resource {
    pub buffer: NonNull<u8>,
    pub cursor: usize,
    pub cap: usize,
    pub header: ResourceHeader
}

impl Resource {
    pub fn new(id: ResourceId, page_num: usize) -> Result<Resource> {
        if MEM_CAP == 0 {
            panic!("MEM CAP cannot be zero")
        }
        let layout = match Layout::array::<u8>(MEM_CAP) {
            Ok(layout) => layout,
            Err(_) => panic!("{}", Error::LayoutSetting)
        };
        //SAFETY: The layout can not be null, as type is defined as u8 and MEM_CAP is 
        //initialized at compile time.
        let ptr = unsafe {
            alloc_zeroed(layout)   
        };
        let buffer = match NonNull::new(ptr) {
            Some(buffer) => buffer,
            None => panic!("{}", Error::MemoryAllocation)
        };
        
        let header = ResourceHeader::new(id, page_num);
        let head = header.to_store()?;
        //write header to buffer
        unsafe {
            let ptr = buffer.as_ptr();
            ptr.copy_from(head.as_ptr(), head.len());
        };
        Ok(Self {
            buffer,
            cursor: HEADER_CAP,
            cap: MEM_CAP,
            header
        })
    }


    ///Returns the current position of the writer cursor.
    pub fn len(&self) -> usize {
        self.cursor
    }

    /// simply advances the cursor, i.e., the len property by 2 bytes. To 
    /// keep room for a length insertion later.
    pub fn reserve_length(&mut self) -> Result<()> {
        self.advance_by(2)
    }

    fn check_len(&self) -> Result<()> {
        if self.cursor > self.cap-1 {
            return Err(Error::SegmentationFault)
        }  
        Ok(())
    }

    pub fn advance_by(&mut self, i: usize) -> Result<()> {
        self.check_len()?;
        self.cursor += i;
        Ok(())
    }

    fn set_index(&mut self, i: usize, val: u8) -> Result<()> {
        self.check_len()?;
        unsafe {
            self.buffer.as_ptr().add(i).write(val);
        }
        Ok(())
    }

 //   fn push(&mut self, val: u8) -> Result<()> {
 //       self.check_len()?;
 //       unsafe {
 //           self.buffer.as_ptr().add(self.len).write(val);
 //       }
 //       self.len += 1;
 //       Ok(())
 //   }
    

    fn set(&mut self, src: *mut u8, len: usize)-> Result<usize> {
        if self.cursor + len >= self.cap {
            return Err(Error::SegmentationFault)
        }
        unsafe {
            let ptr = self.buffer.as_ptr();
            ptr.add(self.cursor).copy_from(src, len);
        }
        self.cursor += len;
        Ok(len)
    }

    fn get(&mut self, i: usize) -> Result<u8> {
         self.check_len()?;
        let val = unsafe {
            let ptr = self.buffer.as_ptr();
            ptr.add(i).read()
        };
        Ok(val)
    }
    pub fn set_u16(&mut self, v: u16) -> Result<usize> {
        self.set(v.to_be_bytes().as_mut_ptr(), 2)
    }

    pub fn set_bu16(&mut self, mut v: U16B) -> Result<usize> {
        self.set(v.as_mut_ptr(), 2)
    }

    pub fn set_u16_at(&mut self, v: u16, i: usize) -> Result<()> {
        let bytes = v.to_be_bytes();
        self.set_index(i, bytes[0])?;
        self.set_index(i+1, bytes[1])
    }

    pub fn set_data_type(&mut self, dt: DT) -> Result<usize> {
        let len = dt.store_len() + META_DATA_SIZE;
        self.set(dt.store()?.as_mut_ptr(), len)?;
        Ok(len)
    }

    //pub fn read_u16(&mut self) -> Result<u16> {
    //    self.check_read(2)?;
    //    let bytes = unsafe {
    //         &*std::ptr::slice_from_raw_parts(self.buffer.as_ptr().add(self.read_cursor), 2)
    //    };
    //    self.read_cursor += 2;
    //    match bytes.try_into() {
    //        Ok(s) => Ok(u16::from_be_bytes(s)),
    //        Err(err) => Err(Error::Custom(err.to_string()))
    //    }
    //}

    //pub fn read_n(&mut self, n: usize) -> Result<Vec<u8>> {
    //    self.check_read(n)?;
    //    let bytes = unsafe {
    //         &*std::ptr::slice_from_raw_parts(self.buffer.as_ptr().add(self.read_cursor), n)
    //    };
    //    self.read_cursor += n;
    //    Ok(bytes.to_vec())
    //}

    pub fn get_mut_ptr(&self) -> *mut u8 {
        self.buffer.as_ptr()
    }
}


#[cfg(test)]
mod test {
    use crate::data::datatype::ToStore;

    use super::*;

    #[test]
    fn resource_header() {
        let header = ResourceHeader::new(ResourceId::Patient, 1);
        assert_eq!(header.page_num, 1);
        assert_eq!(header.rsrc_id, ResourceId::Patient);
        assert_eq!(header.id.store_len(), 16);
    }

    #[test]
    fn resource_writer() {
        let mut writer = Resource::new(ResourceId::Patient, 1).unwrap();
        assert_eq!(writer.len(), HEADER_CAP);
        writer.reserve_length().unwrap();
        assert_eq!(writer.cursor, HEADER_CAP+2);
        writer.set_index(HEADER_CAP+3, 1).unwrap();
        assert_eq!(writer.get(HEADER_CAP+3).unwrap(), 1);
        writer.set_u16_at(1, HEADER_CAP).unwrap();
        assert_eq!(writer.get(HEADER_CAP).unwrap(), 0);
        assert_eq!(writer.get(HEADER_CAP+1).unwrap(), 1);
        assert_eq!(writer.set([2,2,2,2,2].as_mut_ptr(), 5).unwrap(), 5);
        assert_eq!(writer.get(HEADER_CAP+5).unwrap(), 2);
        let datum = true.to_store();
        assert_eq!(writer.set_data_type(datum).unwrap(), 5);
    }
}

