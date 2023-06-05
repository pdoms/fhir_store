use crate::{resourcetypes::ResourceId, datatypes::id::ID};
use crate::error::{Result, Error};

use super::header::Head;

use std::ptr::NonNull;
use std::alloc::{Layout, alloc_zeroed};
use std::mem::size_of;
use uuid::Uuid;


const RESOURCE_CAP: usize = 4096; 

pub struct ResourceHeader {
    typ: ResourceId,
    id: Uuid,  
    size: u16,
    len: u16 
}

impl ResourceHeader {
    fn new_with_id(typ: ResourceId) -> Self {
        let id = Uuid::new_v4();
        let size_len = 2;
        let size = u16::try_from(size_of::<ResourceHeader>()).unwrap() - size_len;
        Self {
            typ,
            id,
            size,
            len: 0,
        }
    }

    fn set_len(&mut self, len: u16) {
        self.len = len;
    }
}

impl Head for ResourceHeader {
    /// Layout:
    /// |Num Bytes |2              |16            |2          |
    /// |----------|---------------|--------------|-----------|
    /// |          |Resource Length| Id / [`Uuid`]|ResourceId |
    fn to_store(&self) -> Result<Vec<u8>> {
        let mut stored = Vec::<u8>::with_capacity(self.size.into());
        stored.extend([0, 0]);
        stored.extend(self.id.clone().into_bytes());
        let typ: u16 = self.typ.clone().into();
        stored.extend(typ.to_be_bytes());
        Ok(stored)
    }

    fn from_store(data: &[u8]) -> Self {
        unimplemented!("from_store() for ResourceHeader")
    }
}


pub struct ResourceWriter {
    header: ResourceHeader,
    cursor: usize,
    buffer: NonNull<u8>
}



impl ResourceWriter {
    /// Creates new [`ResourceWriter`] instance. It also assigns 
    /// an [`Uuid`] to the resource. It will always be set to 
    /// have len [`PAGE_SIZE`].
    pub fn new(typ: ResourceId) -> Result<Self>{
        assert!(RESOURCE_CAP > 0, "RESOURCE_CAP cannot be '0'");
        let layout = match Layout::array::<u8>(RESOURCE_CAP) {
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
        
        let header = ResourceHeader::new_with_id(typ);
        let head = header.to_store()?;

        //write header to buffer
        unsafe {
            let ptr = buffer.as_ptr();
            ptr.copy_from(head.as_ptr(), head.len());
        };
        Ok(Self {
            buffer,
            cursor: header.size.into(),
            header
        })
    }

    //checks if the any operation will cause a segmentation fault or buffer overflow.
    fn len_check(&self, adv: usize) -> Result<()> {
        if self.cursor + adv > RESOURCE_CAP {
            return Err(Error::SegmentationFault)
        }
        Ok(())
    }

    // Advances the cursor by 'i' bytes without writing to them.
    // It returns the cursor position at which the advance was issued.
    fn advance_by(&mut self, i: usize) -> Result<usize> {
        self.len_check(i)?;
        let pos = self.cursor;
        self.cursor += i;   
        Ok(pos)
    }


    // sets a single byte [`u8`] at the provided index 'i'
    fn set_at(&mut self, v: u8, i: usize) -> Result<()> {
        unsafe {
            self.buffer.as_ptr().add(i).write(v);
        }
        Ok(())
    }

    /// Returns the length of the header in bytes.
    pub fn get_header_len(&self) -> usize {
        self.header.size.into()
    }

    /// Returns the current cursor position.
    pub fn len(&self) -> usize {
        self.cursor
    }

    /// Given a [`ptr`], it writes the contents to the buffer.
    pub fn set(&mut self, src: *mut u8, len: usize) -> Result<usize> {
        self.len_check(len)?;
        unsafe {
            let ptr = self.buffer.as_ptr();
            ptr.add(self.cursor).copy_from(src, len);
        }
        self.cursor += len;
        Ok(len)
    }

    /// Convenience function that writes a u16 to the buffer.
    pub fn set_u16<I: Into<u16>>(&mut self, v: I) -> Result<usize> {
        self.set(v.into().to_be_bytes().as_mut_ptr(), 2)
    }

    
    /// Sets the two bytes of a [`u16`] at the given index.
    pub fn set_u16_at<I: Into<u16>>(&mut self, v: I, i: usize) -> Result<usize> {
        let bytes: [u8; 2] = v.into().to_be_bytes();
        self.set_at(bytes[0], i)?;
        self.set_at(bytes[1], i+1)?;
        Ok(2)
    }

    /// Advances the cursor by two bytes without writing to them.
    /// It returns the cursor position at which the reserve was issued.
    pub fn reserve_two(&mut self) -> Result<usize> {
        self.advance_by(2)
    }

    /// Returns a [`*mut u8`] pointer to the buffer.
    pub fn get_mut_ptr(&self) -> *mut u8 {
        self.buffer.as_ptr()
    }

    /// Returns the buffer as a [`Vec`].
    pub fn to_vec(&mut self) -> Vec<u8> {
        let sliced = unsafe {
            std::slice::from_raw_parts_mut(self.get_mut_ptr(), self.cursor)
        };
        sliced.to_vec()
    }
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resourcewriter_new() {
        let writer = ResourceWriter::new(ResourceId::Patient).unwrap();
        assert_eq!(writer.header.typ, ResourceId::Patient);
        assert_eq!(writer.header.len, 0);
        assert_eq!(writer.cursor, writer.header.size.into());
    }

    #[test]
    fn resourcewriter_write() {
        let mut writer = ResourceWriter::new(ResourceId::Patient).unwrap();
        let start = writer.cursor;
        assert_eq!(start, writer.header.size.into());
        let reserved_at = writer.reserve_two().unwrap();
        assert_eq!(writer.len(), start+2);
        assert_eq!(reserved_at, start);
        let v = writer.to_vec();
        assert_eq!(v[start], 0);
        assert_eq!(v[start+1], 0);
        writer.set_u16_at(514u16, reserved_at).unwrap();
        let v = writer.to_vec();
        assert_eq!(v[reserved_at], 2);
        assert_eq!(v[reserved_at+1], 2);
        writer.set_u16(514u16).unwrap();
        let v = writer.to_vec();
        assert_eq!(v[writer.cursor-2], 2);
        assert_eq!(v[writer.cursor-1], 2);
        let start = writer.cursor;
        let mut data = b"Hello, World".to_vec();
        let written = writer.set(data.as_mut_ptr(), data.len()).unwrap();
        assert_eq!(written, data.len());
        let mut vect = writer.to_vec();
        let drained:Vec<u8> = vect.drain(start..start+data.len()).collect();
        assert_eq!(String::from_utf8(drained).unwrap(), "Hello, World".to_string());
        assert!(writer.len_check(4097).is_err());
    }


    


}
