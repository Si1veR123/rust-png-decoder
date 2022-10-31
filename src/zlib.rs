use std::fmt::Display;
use crate::deflate::new_parse_deflate;
use crate::low_level_functions::{bytes_vec_to_single, adler_32};


pub struct ZLibInfo {
    cm: u8,
    cinfo: u8,
    dictid: Option<[u8; 4]>,
    flevel: u8,
    adler32: u32,
}

impl Display for ZLibInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CM: {} CINFO: {} FLEVEL: {} ADLER32: {}", self.cm, self.cinfo, self.flevel, self.adler32)
    }
}

pub fn new_parse_zlib(data: Vec<u8>) -> (ZLibInfo, Vec<u8>) {
    let cmf = data.get(0).expect("No ZLib stream found");
    let flg = data.get(1).expect("ZLib stream has one byte");
    // checksum, when cmf and flg are viewed as a 16 bit int, must be multiple of 31
    assert_eq!((((*cmf as u16) << 8) | (*flg as u16))%31, 0);

    let fdict = (flg & 32u8) >> 5;
    let dictid: Option<[u8; 4]>;
    let deflate_data_start: usize;
    match fdict {
        0 => {
            dictid = None;
            deflate_data_start = 2;
        },
        1 => {
            dictid = Some(data[2..6].try_into().expect("Can't get bytes 2-5 in ZLib stream"));
            deflate_data_start = 6;
        },
        _ => panic!("Incorrect fdict bit")
    }

    let adler32_check = bytes_vec_to_single(&data[data.len()-4..].to_vec());
    let decompressed = new_parse_deflate(
        data[deflate_data_start..(data.len()-4)]
            .try_into()
            .expect("Can't get deflate compressed data")
        );
    
    assert_eq!(adler_32(&decompressed), adler32_check);
    
    (
        ZLibInfo {
            cm: cmf & 15u8,  // bit 0-3 from least to most significant bit
            cinfo: (cmf & 240u8) >> 4,
            dictid,
            flevel: (flg & 192u8) >> 6,
            adler32: adler32_check,
        },
        decompressed
    )
}
