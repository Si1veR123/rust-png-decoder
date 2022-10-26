use core::panic;

use crate::{bitstream::BitStream, low_level_functions::bits_to_byte};

const FIXED_HUFFMAN_CODES_8: [u16; 144] = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191];

const FIXED_HUFFMAN_CODES_7: [u16; 24] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];

const FIXED_HUFFMAN_CODES_8_2: [u16; 8] = [192, 193, 194, 195, 196, 197, 198, 199];

const FIXED_HUFFMAN_CODES_9: [u16; 112] = [400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410,
411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439, 440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461,
462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477, 478, 479, 480, 481, 482, 483, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511];

fn base_codes_for_lengths(codelengths: &Vec<u8>) -> Vec<u16> {
    let &max_code_length = codelengths.iter().max().unwrap();
    let mut base_code: Vec<u16> = Vec::with_capacity((max_code_length+1) as usize);
    base_code.push(0);

    // u16 as may have over 8 bit prefixes, wont have 16 bit prefixes
    let mut code: u16 = 0;
    for bits in 1..=max_code_length {
        let prev_occurences = codelengths
            .iter()
            .cloned()
            .filter(|&x| x == (bits-1))
            .collect::<Vec<u8>>()
            .len() as u16;
        code = (code + prev_occurences) << 1;
        base_code.push(code);
    }
    base_code
}

pub fn prefix_codes_from_codelengths(codelengths: Vec<u8>) -> Vec<u16> {
    let mut base_codes = base_codes_for_lengths(&codelengths);

    let mut prefix_codes: Vec<u16> = Vec::with_capacity(codelengths.len());
    for clen in codelengths {
        let clen_u = clen as usize;
        if clen_u > 0 {
            prefix_codes.push(base_codes[clen_u]);
            base_codes[clen_u] += 1;
        }
    }
    prefix_codes
}

pub fn next_fixed_huffman_code(data: &mut BitStream) -> u16 {
    let first_7 = bits_to_byte(&data.next_n(7), true) as u16;
    let symbol7 = FIXED_HUFFMAN_CODES_7.iter().position(|&x| x == first_7);
    println!("{}", first_7);
    if symbol7.is_some() {
        return (symbol7.unwrap() as u16) + 256
    }
    let first_8 = (first_7 << 1) | (data.next().unwrap() as u16);
    
    let symbol8 = FIXED_HUFFMAN_CODES_8.iter().position(|&x| x == first_8);
    if symbol8.is_some() {
        return symbol8.unwrap() as u16
    }
    let symbol8_2 = FIXED_HUFFMAN_CODES_8_2.iter().position(|&x| x == first_8);
    if symbol8_2.is_some() {
        return (symbol8_2.unwrap() as u16) + 280
    }
    let first_9 = (first_8 << 1) | (data.next().unwrap() as u16);
    let symbol9 = FIXED_HUFFMAN_CODES_9.iter().position(|&x| x == first_9);
    if symbol9.is_some() {
        return (symbol9.unwrap() as u16) + 144
    }
    panic!("Can't get fixed huffman code")
}
