use std::io::{Read, BufRead, Write};
use std::fs::{OpenOptions, File};
use std::println;

const PAGE_SIZE: usize = 1024;
const INIT_PAGES: usize = 4;

type PAGE = &'static [u8;PAGE_SIZE];

fn print_const() {
    println!("INFO: contants\n\t- PAGE_SIZE:  {PAGE_SIZE}\n\t- INIT_PAGES: {INIT_PAGES}");
}

#[derive(Debug)]
pub struct Store {
    file: File,
    header: StoreHeader
}

impl Store {

   pub fn open() -> Result<Self, std::io::Error> {
       print_const();
        match OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open("store.db") {
                Ok(mut f) => {
                    if let Ok(md) = f.metadata() {
                        if md.len() == 0 {
                            println!("INFO: File 'store.db' was empty -> setting {INIT_PAGES} pages and writing header.");
                            // we initialize with INIT_PAGES pages to start with
                            let mut buf: [u8; PAGE_SIZE*INIT_PAGES] = [0; PAGE_SIZE*INIT_PAGES];
                            let header = StoreHeader::new(INIT_PAGES as u16);
                            header.flush_init(buf.as_mut_ptr());
                            f.write(&buf)?;
                            Ok(Self {
                                file: f,
                                header
                            })
                        } else {
                            println!("INFO: Reading Store Header.");
                            let mut buf: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
                            f.read(&mut buf)?;
                            let header = StoreHeader::read_init(&buf.as_ref());
                            Ok(Self {file: f, header})                              
                        }
                    } else {
                        println!("INFO: FILE store.db was empty -> setting {INIT_PAGES} pages and writing header.");
                        // we initialize with INIT_PAGES pages to start with
                        let mut buf: [u8; PAGE_SIZE*INIT_PAGES] = [0; PAGE_SIZE*INIT_PAGES];
                        let header = StoreHeader::new(INIT_PAGES as u16);
                        header.flush_init(buf.as_mut_ptr());
                        f.write(&buf)?;
                        Ok(Self {
                            file: f,
                            header
                        })
                   }
                },
                Err(err) => Err(err)
            }
    }
    pub fn create_resource<D: Read + BufRead>(&mut self, resource_id: &str, data: D) {
        todo!()
    }

    pub fn get_resource_by_id(&mut self, id: &str) {
        todo!()
    }
}


#[derive(Debug)]
struct StoreHeader {
    num_pages: u16,
    page_size: u16,
} 


/// Header of the db main file. For now, this only includes 
/// number of pages and page size. However, the header is one page long.
///
/// Layout:
///
/// |TYPE|num pages|page size|...           |
/// |----|---------|---------|--------------|
/// |LEN |2        |2        |page size - 4 |
impl StoreHeader {
    fn new(num_pages: u16) -> Self {
        Self {
            num_pages,
            page_size: PAGE_SIZE as u16
        }
    }
    
    ///Reads the header on initial opening of file
    fn read_init(head: &[u8]) -> Self {
        let mut num = [0u8;2];
        let mut size = [0u8;2];
        num[0] = head[0]; 
        num[1] = head[1]; 
        size[0] = head[2]; 
        size[1] = head[3]; 
        Self {
            num_pages: u16::from_be_bytes(num),
            page_size: u16::from_be_bytes(size)
        }
    }

    /// Adds the inital header to the empty buffer.
    fn flush_init(&self, buf: *mut u8) {
        let num = self.num_pages.to_be_bytes();
        let size = self.page_size.to_be_bytes();
        unsafe {
         buf.write(num[0]);   
         buf.add(1).write(num[1]);   
         buf.add(2).write(size[0]);   
         buf.add(3).write(size[1]);   
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_open_store() {
        let store = Store::open().unwrap();
        assert_eq!(store.header.num_pages, INIT_PAGES as u16);
        assert_eq!(store.header.page_size, PAGE_SIZE as u16);
    }
}



