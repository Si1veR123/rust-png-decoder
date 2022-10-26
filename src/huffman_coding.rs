use core::panic;

use crate::{bitstream::BitStream, low_level_functions::bits_to_byte};

const FIXED_HUFFMAN_CODES: [u16; 288] = [3, 35, 19, 51, 11, 43, 27, 59, 7, 39, 23, 55, 15, 47, 31, 63, 1, 65, 33, 97, 17, 81, 49, 113, 9, 73, 41, 105, 25, 89, 57, 121, 5, 69, 37, 101, 21, 85, 53, 117, 13, 77, 45, 109, 29, 93, 61, 125, 3, 67, 35, 99, 19, 83, 51, 115, 11, 75, 43, 107, 27, 91, 59, 123, 7, 71, 39, 103, 23, 87, 55, 119, 15, 79, 47, 111, 31, 95, 63, 127, 1, 129, 65, 193, 33, 161, 97, 225, 17, 145, 81, 209, 49, 177, 113, 241, 9, 137, 73, 201, 41, 169, 105, 233, 25, 153, 89, 217, 57, 185, 121, 249, 5, 133, 69, 197, 37, 165, 101, 229, 21, 149, 85, 213, 53, 181, 117, 245, 13, 141, 77, 205, 45, 173, 109, 237, 29, 157, 93, 221, 61, 189, 125, 253, 19, 275, 147, 403, 83, 339, 211, 467, 51, 307, 179, 435, 115, 371, 243, 499, 11, 267, 139, 395, 75, 331, 203, 459, 43, 299, 171, 427, 107, 363, 235, 491, 27, 283, 155, 411, 91, 347, 219, 475, 59, 315, 187, 443, 123, 379, 251, 507, 7, 263, 135, 391, 71, 327, 199, 455, 39, 295, 167, 423, 103, 359, 231, 487, 23, 279, 151, 407, 87, 343, 215, 471, 55, 311, 183, 439, 119, 375, 247, 503, 15, 271, 143, 399, 79, 335, 207, 463, 47, 303, 175, 431, 111, 367, 239, 495, 31, 287, 159, 415, 95, 351, 223, 479, 63, 319, 191, 447, 127, 383, 255, 511, 0, 1, 1, 3, 1, 5, 3, 7, 1, 9, 5, 13, 3, 11, 7, 15, 1, 17, 9, 25, 5, 21, 13, 29, 3, 131, 67, 195, 35, 163, 99, 227];


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
    let first_7 = (bits_to_byte(&data.next_n(7)) >> 1) as u16;
    let symbol7 = FIXED_HUFFMAN_CODES.iter().position(|&x| x == first_7);
    if symbol7.is_some() {
        return symbol7.unwrap() as u16
    }
    let first_8 = (first_7 << 1) | (data.next().unwrap() as u16);
    let symbol8 = FIXED_HUFFMAN_CODES.iter().position(|&x| x == first_8);
    if symbol8.is_some() {
        return symbol8.unwrap() as u16
    }
    let first_9 = (first_8 << 1) | (data.next().unwrap() as u16);
    let symbol9 = FIXED_HUFFMAN_CODES.iter().position(|&x| x == first_9);
    if symbol9.is_some() {
        return symbol9.unwrap() as u16
    }
    panic!("Can't get fixed huffman code")
}
