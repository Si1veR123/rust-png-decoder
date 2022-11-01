use std::fmt::Display;
use crate::deflate::new_parse_deflate;
use crate::low_level_functions::bytes_vec_to_single;
use crate::token::Token;


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

pub fn new_parse_zlib(data: &Vec<u8>) -> (Vec<Token>, Vec<u8>) {
    let mut tokens = Vec::new();

    let &cmf = data.get(0).expect("No ZLib stream found");

    tokens.push(
        Token {
            bits: vec![cmf],
            using_bytes: true,
            nest_level: 1,
            data: format!("CINFO: {}{}{}{} CM: {}{}{}{}", cmf&1, (cmf&2)>>1, (cmf&4)>>2, (cmf&8)>>3, (cmf&16)>>4, (cmf&32)>>5, (cmf&64)>>6, (cmf&128)>>7), // messy way of making binary string
            token_type: "CMF".to_string(),
            description: "0-3 is compression method, 4-7 is compression info".to_string()
        }
    );

    let &flg = data.get(1).expect("ZLib stream has one byte");

    tokens.push(
        Token {
            bits: vec![flg],
            using_bytes: true,
            nest_level: 1,
            data: format!("FLEVEL: {}{} FDICT: {} FCHECK: {}{}{}{}{}", flg&1, (flg&2)>>1, (flg&4)>>2, (flg&8)>>3, (flg&16)>>4, (flg&32)>>5, (flg&64)>>6, (flg&128)>>7), // messy way of making binary string
            token_type: "FLG".to_string(),
            description: "0-4 are check bits, 5 shows if there is preset dictionary, 6-7 is compression level".to_string()
        }
    );

    // checksum, when cmf and flg are viewed as a 16 bit int, must be multiple of 31
    assert_eq!((((cmf as u16) << 8) | (flg as u16))%31, 0);

    let fdict = (flg & 32u8) >> 5;
    let deflate_data_start: usize;
    match fdict {
        0 => {
            deflate_data_start = 2;
        },
        1 => {
            let dictdata: [u8; 4] = data[2..6].try_into().expect("Can't get bytes 2-5 in ZLib stream");

            tokens.push(
                Token {
                    bits: vec![dictdata[0], dictdata[1], dictdata[2], dictdata[3]],
                    using_bytes: true,
                    nest_level: 1,
                    data: "DICT".to_string(),
                    token_type: "DICT".to_string(),
                    description: "Optional preset dictionary".to_string()
                }
            );

            deflate_data_start = 6;
        },
        _ => panic!("Incorrect fdict bit")
    }

    let adler32_bytes = data[data.len()-4..].to_vec();
    let adler32_check = bytes_vec_to_single(&adler32_bytes);


    let (decompressed_tokens, decompressed) = new_parse_deflate(
        data[deflate_data_start..(data.len()-4)]
            .try_into()
            .expect("Can't get deflate compressed data")
        );
    
    tokens.extend(decompressed_tokens);

    tokens.push(
        Token { bits: adler32_bytes, using_bytes: true, nest_level: 1, data: adler32_check.to_string(), token_type: "adler_32".to_string(), description: "Adler 32 Check".to_string() }
    );

    (tokens, decompressed)
}
