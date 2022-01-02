
mod error;
mod decoder;

pub use error::{Result, Error};
pub use decoder::decode;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::gifdecoder::*;

    // #[test]
    fn test_decode() {
        // let file = File::open("/volumes/file/nyan_cat.gif").unwrap();
        let file = File::open("/volumes/file/rainbow.gif").unwrap();
        println!("{:?}", decode(file));
    }

    // #[test]
    fn test_vec() {
        let mut data: Vec<u8> = Vec::new();

        let base = data.len();
        data.resize(base + 3, 0);
        let d = &mut data[base..(base + 3)];
        d[0] = 0x0a;
        d[1] = 0x0b;
        d[2] = 0x0c;

        let base = data.len();
        data.resize(base + 3, 0);
        let d = &mut data[base..(base + 3)];
        d[0] = 0x0d;
        d[1] = 0x0e;
        d[2] = 0x0f;

        println!("{:?}", data);
    }

    #[test]
    fn test_vec_alloc() {
        let mut data: Vec<u8> = Vec::new();
        data.push(0xbb);
        // let mut data: Vec<u8> = Vec::with_capacity(16);

        let mut data1: Vec<u8> = Vec::new();
        data1.push(0xbb);

        let (ptr, len, cap) = data.into_raw_parts();
        let mut data = unsafe {
            println!("ptr: {:08X}, len: {}, cap: {}", ptr as usize, len, cap);
            Vec::from_raw_parts(ptr, len, cap)
        };

        for _ in 0..('9' as u32 - '0' as u32) {
            data.push(0xcc);
        }


        let (ptr, len, cap) = data.into_raw_parts();
        unsafe {
            println!("ptr: {:08X}, len: {}, cap: {}", ptr as usize, len, cap);
        }
    }
}

