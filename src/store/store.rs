use std::io::{Read, Write, Seek};
use std::fs::{OpenOptions, File};
use crate::error::{Result, Error};

const INIT_PAGES: usize = 4;
pub const PAGE_SIZE: usize = 4096;
type PAGE = &'static [u8;PAGE_SIZE];

fn print_const() {
    println!("INFO: constants\n\t- PAGE_SIZE:  {PAGE_SIZE}\n\t- INIT_PAGES: {INIT_PAGES}");
}

#[derive(Debug)]
pub struct Store {
    file: File,
    header: StoreHeader,
}

impl Store {

    pub fn open() -> std::result::Result<Self, std::io::Error> {
    print_const();
        match OpenOptions::new()
            .read(true)
            .write(true)
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
                            f.sync_all()?;
                            Ok(Self {
                               file: f,
                               header,
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
                        f.sync_all()?;
                        Ok(Self {
                            file: f,
                            header,
                        })
                   }
                },
                Err(err) => Err(err)
            }
    }

    pub fn get_page_copy(&mut self, num: usize) -> Vec<u8> {
        let start = num * PAGE_SIZE;
        self.file.seek(std::io::SeekFrom::Start(start as u64)).unwrap();
        let mut buf = [0u8; PAGE_SIZE];
        self.file.read_exact(&mut buf).unwrap();
        buf.to_vec()
    }
    //pub fn create_resource_from_json<D: Read>(&mut self, resource_id: &str, mut data: D) -> Result<()> {
    //    let mut buf = Vec::new();
    //    match data.read_to_end(&mut buf) {
    //        Ok(_) => {
    //            let page_num = self.header.inc_top();
    //            let mut parser = JsonParser::new_from_slice(&buf, resource_id, page_num as usize)?;
    //            parser.parse()?;
    //            let ptr = parser.flush();
    //            let len = parser.len();
    //            println!("LENGTH {}", len);
    //            let page_offset = (page_num - 1) * PAGE_SIZE as u16;
    //            //TODO ERROR
    //            println!("OFFSET: {page_offset}");
    //            self.file.seek(std::io::SeekFrom::Start(page_offset as u64)).unwrap();
    //            //TODO make sure mem size is not longer than page size
    //            let buf: &[u8] = unsafe {
    //                let buf = std::ptr::slice_from_raw_parts(ptr, len);
    //                &*buf
    //            };
    //            println!("BUFFER: {:?}", buf);
    //            
    //            self.file.write_all(buf).unwrap();
    //            self.file.sync_data().unwrap();
    //            Ok(())
    //        },
    //        Err(err) => Err(Error::Custom(err.to_string()))
    //    }
    //}

    pub fn get_resource_by_id(&mut self, id: &str) {
        todo!()
    }
}




#[derive(Debug)]
struct StoreHeader {
    num_pages: u16,
    page_size: u16,
    top_page: u16
} 


/// Header of the db main file. For now, this only includes 
/// number of pages, page size and top_page. However, the header is one page long.
///
/// Layout:
///
/// |TYPE|num pages|page size|top page|...           |
/// |----|---------|---------|--------|--------------|
/// |LEN |2        |2        |2       |page size - 4 |
impl StoreHeader {
    fn new(num_pages: u16) -> Self {
        Self {
            num_pages,
            page_size: PAGE_SIZE as u16,
            top_page: 1
        }
    }
    
    ///Reads the header on initial opening of file
    fn read_init(head: &[u8]) -> Self {
        let mut num = [0u8;2];
        let mut size = [0u8;2];
        let mut top = [0u8;2];
        num[0] = head[0]; 
        num[1] = head[1]; 
        size[0] = head[2]; 
        size[1] = head[3]; 
        top[0] = head[4]; 
        top[1] = head[5]; 
        Self {
            num_pages: u16::from_be_bytes(num),
            page_size: u16::from_be_bytes(size),
            top_page: u16::from_be_bytes(top)
        }
    }

    /// Adds the inital header to the empty buffer.
    fn flush_init(&self, buf: *mut u8) {
        let num = self.num_pages.to_be_bytes();
        let size = self.page_size.to_be_bytes();
        let top = self.top_page.to_be_bytes();
        unsafe {
            buf.write(num[0]);   
            buf.add(1).write(num[1]);   
            buf.add(2).write(size[0]);   
            buf.add(3).write(size[1]);   
            buf.add(4).write(top[0]);   
            buf.add(5).write(top[1]);   
        }
    }

    fn inc_top(&mut self) -> u16 {
        self.top_page+=1;
        self.top_page
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_open_store() {
        let mut store = Store::open().unwrap();
        assert_eq!(store.header.num_pages, INIT_PAGES as u16);
        assert_eq!(store.header.page_size, PAGE_SIZE as u16);
        let json = File::open("example.json").unwrap();
      //  store.create_resource_from_json("patient", json).unwrap();
    }
}



