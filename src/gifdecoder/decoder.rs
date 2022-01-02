use std::{io, str};
use std::fs::File;
use bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter, Endianness, LittleEndian};
use weezl::{BitOrder, decode::Decoder};
use super::{Result, Error};

// ref: https://www.w3.org/Graphics/GIF/spec-gif89a.txt
// ref: https://wiki.whatwg.org/wiki/GIF
pub fn decode<R>(r: R) -> Result<()> 
    where R: io::Read {
    let mut reader = BitReader::endian(r, LittleEndian);

    if reader.read::<u8>(8)? != b'G' ||     // 'G'
        reader.read::<u8>(8)? != b'I' ||    // 'I'
        reader.read::<u8>(8)? != b'F' {     // 'F'
        return Err(Error::FormatError); 
    }

    reader.skip(8)?; // '8'
    let version = reader.read::<u8>(8)?; // '7' or '9'
    reader.skip(8)?; // 'a'

    if version == b'7' {            // 87a
    } else if version == b'9' {     // 89a
    } else {
        return Err(Error::InvalidVersion);
    }

    let screen_width = reader.read::<u16>(16)?;
    let screen_height = reader.read::<u16>(16)?;
    
    let gct_size = reader.read::<u8>(3)?;
    let gct_sort_flag = reader.read_bit()?;
    let color_resoln = reader.read::<u8>(3)? + 1;
    let gct_flag = reader.read_bit()?;

    let bg_index = reader.read::<u8>(8)?;
    let pixel_aspect = reader.read::<u8>(8)?;
    
    println!("screen_width: {}, screen_height: {}", screen_width, screen_height);
    println!("gct_flag: {},  color_resoln: {},  gct_sort_flag: {},  gct_size: {}", 
        gct_flag, color_resoln, gct_sort_flag, gct_size);
    println!("bg_index: {}, pixel_aspect: {}", bg_index, pixel_aspect);

    let mut gct = Vec::new();

    if gct_flag {
        let gct_size = 2 << gct_size;
        for i in 0..gct_size {
            let rgb = (reader.read::<u32>(8)? << 16) | 
                (reader.read::<u32>(8)? << 8) | 
                reader.read::<u32>(8)? &
                0x00ffffff;
            gct.push(rgb);
            println!("gct {:03}: {:#010x}", i, rgb);
        }
    }

    let mut image_count = 0;

    loop {
        let ext_intro = reader.read::<u8>(8)?;
        println!("ext_intro: {:#x}", ext_intro);
        match ext_intro {
            0x21 => {
                let ext_label = reader.read::<u8>(8)?;
                println!("ext_label: {:#x}", ext_label);
                match ext_label {
                    0xff => { // Application Extension
                        let block_size = reader.read::<u32>(8)?; // fixed value 11
                        println!("block_size: {}", block_size);

                        let mut appl_id = [0x00 as u8; 8];
                        reader.read_bytes(&mut appl_id)?;
                        println!("appl_id: {:02X?}", appl_id); 

                        let mut appl_auth_code = [0x00 as u8; 3];
                        reader.read_bytes(&mut appl_auth_code)?;
                        println!("appl_auth_code: {:02X?}", appl_auth_code);

                        let sub_block_data = read_sub_block(&mut reader)?;
                        if &appl_id == b"NETSCAPE" && &appl_auth_code == b"2.0" {
                            let mut sub_block_reader = BitReader::endian(&sub_block_data as &[u8], LittleEndian);
                                println!("loop_count: {}", loop_count);
                            } else if sub_block_id == 0x02 {
                                // ref: http://www.vurdalakov.net/misc/gif/netscape-buffering-application-extension
                                let buffer_size = sub_block_reader.read::<u32>(32)?;
                                println!("buffer_size: {}", buffer_size);
                            }
                    },
                    0xf9 => { // Graphic Control Extension
                        let block_size = reader.read::<u8>(8)?; // fixed value 4
                        println!("block_size: {}", block_size);

                        let trans_flag = reader.read_bit()?;
                        let user_input_flag = reader.read_bit()?;
                        let disposal_method = reader.read::<u8>(3)?;
                        reader.skip(3)?;    // Reserved
                        let delay_time = reader.read::<u16>(16)?;    
                        let trans_color_index = reader.read::<u8>(8)?;
                        println!("trans_flag: {}, user_input_flag: {}, disposal_method: {}, delay_time: {}, trans_color_index: {}", 
                            trans_flag, user_input_flag, disposal_method, delay_time, trans_color_index);

                        skip_sub_block(&mut reader)?;
                    },
                    0xfe => { // Comment Extension
                        let sub_block_data = read_sub_block(&mut reader)?;
                        let comment = str::from_utf8(&sub_block_data)?;
                        println!("comment: {}", comment);
                    },
                    0x01 => {   // Plain Text Extension
                        let block_size = reader.read::<u8>(8)?; // fixed value 12
                        println!("block_size: {}", block_size);

                        reader.skip(12 * 8)?;

                        skip_sub_block(&mut reader)?;
                    },
                    _ => {
                        println!("unknown ext_label: {:#x}", ext_label);
                        break;
                    },
                }
            },
            0x2c => {
                image_count += 1;

                let image_left = reader.read::<u16>(16)?;
                let image_top = reader.read::<u16>(16)?;
                println!("image_left: {}, image_top: {}", image_left, image_top);

                let image_width = reader.read::<u16>(16)?;
                let image_height =  reader.read::<u16>(16)?;
                println!("image_width: {}, image_height: {}", image_width, image_height);

                let lct_size = reader.read::<u8>(3)?;
                reader.skip(2)?;
                let lct_sort_flag = reader.read_bit()?;
                let interlace_flag = reader.read_bit()?;
                let lct_flag = reader.read_bit()?;
                println!("lct_flag: {}, interlace_flag: {}, lct_sort_flag: {}, lct_size: {}", 
                    lct_flag, interlace_flag, lct_sort_flag, lct_size);

                let mut lct = Vec::new();

                if lct_flag {
                    let lct_size = 2 << lct_size;
                    for i in 0..lct_size {
                        let rgb = (reader.read::<u32>(8)? << 16) | 
                            (reader.read::<u32>(8)? << 8) | 
                            reader.read::<u32>(8)? &
                            0x00ffffff;
                        lct.push(rgb);
                        println!("lct {:03}: {:#010x}", i, rgb);
                    }
                }

                let code_size = reader.read::<u8>(8)?;
                println!("code_size: {}", code_size);

                let compress_data = read_sub_block(&mut reader)?;
                println!("image {}, compress_data size: {}", image_count, compress_data.len());
                // println!("{:02X?}", compress_data);

                // let mut file = File::create(format!("/volumes/file/data/{}.dat", image_count))?;
                // file.write_all(&compress_data)?;

                let decompress_data = Decoder::new(BitOrder::Lsb, code_size).decode(&compress_data)?;
                println!("decompress_data size: {}", decompress_data.len());

                let mut rgb_data: Vec<u32> = Vec::with_capacity(decompress_data.len());
                for i in 0..decompress_data.len() {
                    rgb_data.insert(rgb_data.len(), gct[decompress_data[i] as usize]);
                }

                let (ptr, len, cap) = rgb_data.into_raw_parts();
                let rgb_data = unsafe {
                    let ptr = ptr as *mut u8;
                    Vec::from_raw_parts(ptr, len, cap)
                };

                let mut bmp_file = File::create(format!("/volumes/file/data/{}.bmp", image_count))?;
                make_bmp(&mut bmp_file, image_width as u32, image_height as u32, &rgb_data)?;
            },
            0x3b => {
                println!("GIF is end.");
                break;
            }
            _ => {
                return Err(Error::InvalidExtension);
            }
        }
    }

    Ok(())
}

fn read_sub_block<R, E>(reader: &mut BitReader<R, E>) -> Result<Vec<u8>>
    where R: io::Read, E: Endianness {
    let mut sub_block_data = Vec::new();

    loop {
        let sub_block_size = reader.read::<u8>(8)?;
        // println!("sub_block_size: {}", sub_block_size);
        if sub_block_size == 0 { // Block Terminator
            break;
        }

        let size = sub_block_size as usize;
        let base = sub_block_data.len();
        sub_block_data.resize(base + size, 0x00);
        let data = &mut sub_block_data[base..(base + size)];
        reader.read_bytes(data)?;
    }

    Ok(sub_block_data)
}


fn skip_sub_block<R, E>(reader: &mut BitReader<R, E>) -> Result<()>
    where R: io::Read, E: Endianness {

    loop {
        let sub_block_size = reader.read::<u8>(8)?;
        // println!("sub_block_size: {}", sub_block_size);
        if sub_block_size == 0 { // Block Terminator
            break;
        }

        reader.skip(sub_block_size as u32 * 8)?;
    }

    Ok(())
}

fn make_bmp<W>(writer: &mut W, width: u32, height: u32, data: &[u8]) -> Result<()> 
    where W: io::Write {
    let mut writer = BitWriter::endian(writer, LittleEndian);
    writer.write_bytes(&[b'B', b'M'])?; // bfType
    writer.write(32, 0x36 + data.len() as u32)?; // bfSize
    writer.write(16, 0x00)?; // bfReserved1
    writer.write(16, 0x00)?; // bfReserved2
    writer.write(32, 0x36)?; // bfOffbits
    writer.write(32, 0x28)?; // biSize
    writer.write(32, width)?; // biWidth
    writer.write(32, height)?; // biHeight
    writer.write(16, 0x01)?; // biPlanes
    writer.write(16, 32)?; // biBitCount
    writer.write(32, 0x00)?; // biCompression
    writer.write(32, 0x00)?; // biSizeImage 
    writer.write(32, 0x00)?; // biXPelsPerMeter 
    writer.write(32, 0x00)?; // biYPelsPerMeter 
    writer.write(32, 0x00)?; // biClrUsed 
    writer.write(32, 0x00)?; // biClrImportant 
    writer.write_bytes(data)?;
    Ok(())
}

