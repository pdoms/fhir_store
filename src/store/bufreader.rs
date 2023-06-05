use crate::datatypes::id::{ID, get_expects};
pub fn read_buffer(mut buf: &[u8]) -> Vec<u16> {
    let len: [u8; 2] = buf[..2].try_into().unwrap();
    buf = &buf[2..];
    let mut result = Vec::<u16>::new();
    result.push(u16::from_be_bytes(len));
    let mut gp_list = 0;
    let mut current = 0;
    while buf.len() > 0 {
        if gp_list > 0 && current == 0 {
            let length: [u8; 2] = buf[..2].try_into().unwrap();
            buf = &buf[2..];
            let length = u16::from_be_bytes(length);
            result.push(length);
            //gp_list -= 2;
            current = length;
        }



        let len: [u8; 2] = buf[..2].try_into().unwrap();
        buf = &buf[2..];
        if gp_list > 0 {
            gp_list -= 2;
        }
        if current > 0 {
            current -= 2;
        }
        let pos_id: [u8; 2] = buf[..2].try_into().unwrap();
        let id = ID::try_from(u16::from_be_bytes(pos_id)).unwrap();
        buf = &buf[2..];
        if gp_list > 0 {
            gp_list -= 2;
        }
        if current > 0 {
            current -= 2;
        }
        let l = u16::from_be_bytes(len);
        result.push(l);
        result.push(id as u16);
        if let Some(expects) = get_expects(id.clone()) {
            if expects.is_general_purpose() {
                let gp_len: [u8; 2] = buf[..2].try_into().unwrap();
                let le = u16::from_be_bytes(gp_len);
                result.push(le);
                buf = &buf[2..];
                if gp_list > 0 {
                    gp_list -= 2;
                }
                if current > 0 {
                    current -= 2;
                }
                continue;
            }
        }
        if id.is_primitive_list() {
            let mut temp_cur = 0;
            let trgt = l-2-4;
            if gp_list > 0 {
                gp_list -= l-2;
            }
            if current > 0 {
                current -= trgt;
            }
            while temp_cur < trgt {
                let length: [u8; 2] = buf[..2].try_into().unwrap();
                buf = &buf[2..];
                let length = u16::from_be_bytes(length);
                result.push(length);
                buf = &buf[length as usize..];
                temp_cur += 2 + length;
            }
        } else if id.is_gp_list() {
            gp_list = u16::from_be_bytes(len);
            let length: [u8; 2] = buf[..2].try_into().unwrap();
            //gp_list -= 2;
            buf = &buf[2..];
            let length = u16::from_be_bytes(length);
            result.push(length);
            current = length;
        } else if !id.is_key() {
            buf = &buf[(l - 2 )as usize..];
            if gp_list > 0 {
                gp_list -= l;
            }
            if current > 0 {
                current -= l;
            }
        }

    }
    return result
}
