use core::panic;

use crate::{bitstream::BitStream, low_level_functions::bits_to_byte};

// === CONSTANTS ===

// Fixed huffman codes constructed from codelengths given in RFC 1951 3.2.6
const FIXED_HUFFMAN_CODES_8: [u16; 144] = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100,
101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151,
152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191];

const FIXED_HUFFMAN_CODES_7: [u16; 24] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];

const FIXED_HUFFMAN_CODES_8_2: [u16; 8] = [192, 193, 194, 195, 196, 197, 198, 199];

const FIXED_HUFFMAN_CODES_9: [u16; 112] = [400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423, 424, 425, 426, 427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439, 440, 441,
442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477, 478, 479, 480, 481, 482, 483, 484, 485, 486, 487, 488, 489, 490, 491, 492,
493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511];

// RFC 1951 3.2.5
const LENGTH_BASES: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LENGTH_EXTRA_BITS: [usize; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASES: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
const DIST_EXTRA_BITS: [usize; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

// ==============


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

pub fn next_huffman_symbol(data: &mut BitStream, symbols: &Vec<u16>, prefixes: &Vec<u16>, min_codelength: u8) -> u16 {
    // could be error if min codelength > 8
    let mut current_prefix = bits_to_byte(&data.next_n(min_codelength as usize), true) as u16;
    let symbol;

    loop {
        let prefix_position = prefixes.iter().position(|&x| x == current_prefix);
        if prefix_position.is_none() {
            current_prefix = (current_prefix << 1) | (data.next().unwrap() as u16);
            continue
        }
        symbol = *symbols.get( prefix_position.unwrap() ).unwrap();
        break;
    }

    symbol
}

pub fn next_fixed_huffman_symbol(data: &mut BitStream) -> u16 {
    let first_7 = bits_to_byte(&data.next_n(7), true) as u16;
    let symbol7 = FIXED_HUFFMAN_CODES_7.iter().position(|&x| x == first_7);
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

pub fn huffman_codes_from_codelengths(codelengths: &Vec<u8>) -> (Vec<u16>, Vec<u16>) {
    // returns same size vectors of symbols, and prefixes
    let mut symbols = Vec::new();

    for (i, &code) in codelengths.iter().enumerate() {
        if code > 0 {
            symbols.push(i as u16);
        }
    }

    // this is a vector of the prefixes for each of the code lengths
    let prefixes = prefix_codes_from_codelengths(
        codelengths
        .iter()
        .cloned()
        .filter(|&x| x > 0)
        .collect()
    );

    (symbols, prefixes)
}

pub fn decode_length(data: &mut BitStream, length_sym: u16) -> u16 {
    let index = (length_sym - 257) as usize;
    let length_base = *LENGTH_BASES.get(index).unwrap();

    let num_extra_bits = *LENGTH_EXTRA_BITS.get(index).unwrap();

    if num_extra_bits > 0 {
        let extra_bits = data.next_n(num_extra_bits);
        bits_to_byte(&extra_bits, true) as u16 + length_base
    } else {
        length_base
    }
}

pub fn decode_distance(data: &mut BitStream, dist_sym: u8) -> u16 {
    let index = dist_sym as usize;
    let dist_base = *DIST_BASES.get(index).unwrap();

    let num_extra_bits = *DIST_EXTRA_BITS.get(index).unwrap();

    if num_extra_bits > 0 {
        let extra_bits = data.next_n(num_extra_bits);
        let mut extra_bits_value: u16 = 0;

        // same as bits_to_byte function, with added support for u16, as may be up to 13 bits
        for bit in extra_bits {
            extra_bits_value = (extra_bits_value << 1) | (bit as u16);
        }

        dist_base + extra_bits_value

    } else {
        dist_base
    }
}

pub fn decode_duplicate_reference(prev_literals: &Vec<u8>, length: u16, distance: u16) -> Vec<u8> {
    let mut literals: Vec<u8> = Vec::new();
    
    let mut position: usize = prev_literals.len();
    position -= distance as usize;

    for _i in 0..(length) {
        literals.push(*prev_literals.get(position).unwrap());
        if position == prev_literals.len()-1 {
            position -= distance as usize;
        }
        position += 1;
    }
    literals
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_length() {
        // 10011010 11011111 00010111
        let mut bs = BitStream::new(vec![154, 223, 23], false);

        // no extra bits, 259 symbol
        assert_eq!(decode_length(&mut bs, 259), 5);

        // 1 extra bit, 268 symbol, where next bit is 0
        assert_eq!(decode_length(&mut bs, 268), 17);

        // 1 extra bit, 268 symbol, where next bit is 1
        assert_eq!(decode_length(&mut bs, 268), 18);

        // 5 extra bits, 282 symbol, next bits 01100, 175 = 163 + 12
        assert_eq!(decode_length(&mut bs, 282), 175);

        // no extra bits, 285 symbol
        assert_eq!(decode_length(&mut bs, 285), 258);

        // false
        // next bit 1
        assert_ne!(decode_length(&mut bs, 268), 17);
        
        // next bits 111
        assert_ne!(decode_length(&mut bs, 274), 46);
    }

    #[test]
    fn test_decode_distance() {
        // 10011010 11011111 00010111
        let mut bs = BitStream::new(vec![154, 223, 23], false);

        // no extra bits
        assert_eq!(decode_distance(&mut bs, 2), 3);

        // next bit 0
        assert_eq!(decode_distance(&mut bs, 5), 7);

        // next bits 101100
        assert_eq!(decode_distance(&mut bs, 14), 173);
        
        // next bits 1111110111110
        assert_eq!(decode_distance(&mut bs, 28), 24511);
    }

    #[test]
    fn test_decode_duplicate_reference() {
        assert_eq!(decode_duplicate_reference(&vec![1, 2, 3, 4, 5], 3, 4), vec![2, 3, 4]);

        // repeated reference test
        assert_eq!(decode_duplicate_reference(&vec![1, 2, 3, 4, 5, 6, 7, 8], 8, 3), vec![6, 7, 8, 6, 7, 8, 6, 7]);
    }

    #[test]
    fn test_huffman_codes_from_codelengths() {
        let codelengths = vec![0, 4, 1, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 2];
        let (huffman_code_symbols, huffman_code_prefixes) = huffman_codes_from_codelengths(&codelengths);

        assert_eq!(huffman_code_symbols, vec![1, 2, 4, 16, 17, 18]);
        assert_eq!(huffman_code_prefixes, vec![12, 0, 13, 14, 15, 2]);
    }
}
