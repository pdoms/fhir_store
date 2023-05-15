
use std::ptr::NonNull;
use std::alloc::{Layout, alloc_zeroed};

use crate::error::{Error, Result};
use crate::datatypes::{DataType, BU16};


const MEM_CAP: usize = 1024;

pub struct Memory {
    pub buffer: NonNull<u8>,
    pub len: usize,
    pub cap: usize,
    read_cursor: usize
}

impl Memory {
    pub fn new() -> Result<Memory> {
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
        Ok(Self {
            buffer,
            len: 0,
            cap: MEM_CAP,
            read_cursor: 0
        })
    }

    pub fn with_cap(c: usize) -> Result<Memory> {
        let layout = match Layout::array::<u8>(c) {
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
        Ok(Self {
            buffer,
            len: 0,
            cap: MEM_CAP,
            read_cursor: 0
        })
    }

    ///Returns the current position of the writer cursor.
    pub fn len(&self) -> usize {
        self.len
    }

    /// simply advances the cursor, i.e., the len property by 2 bytes. To 
    /// keep room for a length insertion later.
    pub fn reserve_length(&mut self) -> Result<()> {
        self.advance_by(2)
    }

    fn check_len(&self) -> Result<()> {
        if self.len > self.cap-1 {
            return Err(Error::SegmentationFault)
        }  
        Ok(())
    }
    fn check_read(&self, wants: usize) -> Result<()> {
        if self.read_cursor + wants > self.len {
            return Err(Error::SegmentationFault)
        }  
        Ok(())
    }

    pub fn advance_by(&mut self, i: usize) -> Result<()> {
        self.check_len()?;
        self.len += i;
        Ok(())
    }

    fn set_index(&mut self, i: usize, val: u8) -> Result<()> {
        self.check_len()?;
        unsafe {
            self.buffer.as_ptr().add(i).write(val);
        }
        Ok(())
    }

    fn push(&mut self, val: u8) -> Result<()> {
        self.check_len()?;
        unsafe {
            self.buffer.as_ptr().add(self.len).write(val);
        }
        self.len += 1;
        Ok(())
    }
    

    fn set(&mut self, src: *mut u8, len: usize)-> Result<usize> {
        if self.len + len >= self.cap {
            return Err(Error::SegmentationFault)
        }
        unsafe {
            let ptr = self.buffer.as_ptr();
            ptr.add(self.len).copy_from(src, len);
        }
        self.len += len;
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

    pub fn set_bu16(&mut self, mut v: BU16) -> Result<usize> {
        self.set(v.as_mut_ptr(), 2)
    }
    pub fn set_u16_at(&mut self, v: u16, i: usize) -> Result<()> {
        let bytes = v.to_be_bytes();
        self.set_index(i, bytes[0])?;
        self.set_index(i+1, bytes[1])
    }

    pub fn set_data_type(&mut self, dt: DataType) -> Result<()> {
        //unpack
        let (mut value, id, len) = dt.to_memory();
        // set id
        self.set_u16(id)?;
        // set len + 2
        self.set_u16(len + 2)?;
        // set data
        self.set(value.as_mut_ptr(), len as usize)?;
        Ok(())
    }


    pub fn read_u16(&mut self) -> Result<u16> {
        self.check_read(2)?;
        let bytes = unsafe {
             &*std::ptr::slice_from_raw_parts(self.buffer.as_ptr().add(self.read_cursor), 2)
        };
        self.read_cursor += 2;
        match bytes.try_into() {
            Ok(s) => Ok(u16::from_be_bytes(s)),
            Err(err) => Err(Error::Custom(err.to_string()))
        }
    }

    pub fn read_n(&mut self, n: usize) -> Result<Vec<u8>> {
        self.check_read(n)?;
        let bytes = unsafe {
             &*std::ptr::slice_from_raw_parts(self.buffer.as_ptr().add(self.read_cursor), n)
        };
        self.read_cursor += n;
        Ok(bytes.to_vec())
    }
}

