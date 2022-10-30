use crate::bitstream::BitStream;


mod deflate_block {
    use core::panic;

    use super::BitStream;
    use crate::huffman_coding::*;
    use crate::low_level_functions::{bytes_vec_to_single, bits_to_byte};

    pub fn parse_next_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> bool {
        // given all the remaining bits, parse and return the first block
        let bfinal = data.next().unwrap() == 1;

        let btype = (data.next().unwrap(), data.next().unwrap());
        match btype {
            (0, 0) => deflate_uncompressed_block(data, symbol_buffer),
            (1, 0) => deflate_fixed_huffman_block(data, symbol_buffer),
            (0, 1) => deflate_dynamic_huffman_block(data, symbol_buffer),
            (1, 1) => panic!("BTYPE has reserved value (11)"),
            _ => panic!("Invalid BTYPE")
        }
        bfinal
    }

    pub fn deflate_uncompressed_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {

        data.move_to_next_byte();
        // next 16 bits (2 bytes) are length
        let length_bytes = ( data.next_byte(), data.next_byte() );
        // check against backup length (next 2 bytes), which is the bitwise NOT of len
        assert_eq!(length_bytes, ( !data.next_byte(), !data.next_byte() ));

        let length = bytes_vec_to_single(&vec![length_bytes.0, length_bytes.1]) as usize;
        symbol_buffer.extend(data.next_n_bytes(length));
    }

    pub fn deflate_fixed_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {
        println!("FIXED");
        loop {
            let symbol = next_fixed_huffman_symbol(data);

            if symbol > 256 {
                let length = decode_length(data, symbol);
                let distance_symbol = bits_to_byte(&data.next_n(5), true);
                let distance = decode_distance(data, distance_symbol);

                let duplicate_values = decode_duplicate_reference(symbol_buffer, length, distance);
                symbol_buffer.extend(duplicate_values);
            }
            else if symbol == 256 {
                break
            } else {
                symbol_buffer.push(symbol as u8);
            }
        }
    }

    pub fn decode_codelengths(data: &mut BitStream, num_of_codes: usize, code_length_prefixes: &Vec<u16>, code_length_symbols: &Vec<u16>) -> Vec<u8> {
        // given huffman codes (symbols and prefixes) for the codelength table, decode a given number of codes from the bitstream

        let mut decoded_codelengths: Vec<u8> = Vec::new();

        loop {
            let mut prefix_code: u16 = 0;

            loop {
                // read next bit to prefix code
                prefix_code = (prefix_code << 1) | (data.next().unwrap() as u16);

                let prefix_position = code_length_prefixes.iter().position(|&x| x == prefix_code);
                
                if prefix_position.is_none() { continue }

                // get position, and use position to index code_length_symbols to find the symbol
                let symbol = *code_length_symbols.get( prefix_position.unwrap() ).unwrap() as u8; // code length symbols <= 18
                match symbol {
                    0..=15 => {
                        // literal code length
                        decoded_codelengths.push(symbol);
                    },
                    16 => {
                        // Copy the previous code length 3-6 times, 2 extra bits
                        let &prev_symbol = decoded_codelengths.last().unwrap();
                        let repitions = (bits_to_byte(&data.next_n(2), false) >> 6u8) + 3;
                        
                        for _ in 0..repitions {
                            decoded_codelengths.push(prev_symbol);
                        }
                    },
                    17 => {
                        // Copy 0 3-10 times, 3 extra bits
                        let repitions = (bits_to_byte(&data.next_n(3), false) >> 5u8) + 3;
                        for _ in 0..repitions {
                            decoded_codelengths.push(0);
                        }
                    },
                    18 => {
                        // Copy 0 11-138 times, 7 extra bits
                        let repitions = (bits_to_byte(&data.next_n(7), false) >> 1u8) + 11;
                        for _ in 0..repitions {
                            decoded_codelengths.push(0);
                        }
                    }
                    _ => panic!("Invalid code length symbol")
                }

                break;
            }

            if decoded_codelengths.len() >= num_of_codes {break}
        }
        decoded_codelengths
    }

    pub fn deflate_dynamic_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {
        println!("DYNAMIC");
        let num_of_normal_codes = ((bits_to_byte(&data.next_n(5), false) >> 3) as u16 + 257) as usize;
        let num_of_dist_codes = ((bits_to_byte(&data.next_n(5), false) >> 3) + 1) as usize;

        // 1) Parse codelength huffman codes
        let num_of_codelength_codes = ((bits_to_byte(&data.next_n(4), false) >> 4) + 4) as usize;

        // reorder codelength codelengths
        const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        let mut code_length_codelengths = [0u8; 19];
        for i in 0..(num_of_codelength_codes) {
            let codelength_codelength = bits_to_byte(&data.next_n(3), false) >> 5;
            code_length_codelengths[ORDER[i]] = codelength_codelength;
        }

        let (code_length_symbols, code_length_prefixes) = huffman_codes_from_codelengths(&code_length_codelengths.to_vec());

        // 2) Parse main huffman codelengths
        let decoded_normal_codelengths = decode_codelengths(data, num_of_normal_codes, &code_length_prefixes, &code_length_symbols);
        let decoded_distance_codelengths = decode_codelengths(data, num_of_dist_codes, &code_length_prefixes, &code_length_symbols);
    

        let (huffman_normal_symbols, huffman_normal_prefixes) = huffman_codes_from_codelengths(&decoded_normal_codelengths);
        let (huffman_distance_symbols, huffman_distance_prefixes) = huffman_codes_from_codelengths(&decoded_distance_codelengths);

        let minimum_normal_codelength = decoded_normal_codelengths.iter().cloned().filter(|&x| x > 0).min().unwrap();
        let minimum_distance_codelength = decoded_distance_codelengths.iter().cloned().filter(|&x| x > 0).min().unwrap();

        // 3) Parse data using huffman codes
        loop {
            let symbol: u16 = next_huffman_symbol(data, &huffman_normal_symbols, &huffman_normal_prefixes, minimum_normal_codelength);

            if symbol > 256 {
                let length = decode_length(data, symbol);

                let distance_symbol = next_huffman_symbol(data, &huffman_distance_symbols, &huffman_distance_prefixes, minimum_distance_codelength) as u8;

                let distance = decode_distance(data, distance_symbol);

                let duplicate_values = decode_duplicate_reference(symbol_buffer, length, distance);
                symbol_buffer.extend(duplicate_values);
            }
            else if symbol == 256 {
                break
            } else {
                symbol_buffer.push(symbol as u8);
            }
        }
    }

}

pub mod deflate_decompressor {
    use super::BitStream;
    use super::deflate_block;

    pub fn new_parse_deflate(data: Vec<u8>) -> Vec<u8> {
        let mut bit_stream = BitStream::new(data, false);

        let mut decompressed_data: Vec<u8> = Vec::new();

        loop {
            let bfinal = deflate_block::parse_next_block(&mut bit_stream, &mut decompressed_data);

            if bfinal {
                break
            }
        }

        decompressed_data
    }
}

#[cfg(test)]
mod tests {
    use super::{*, deflate_block::decode_codelengths};

    #[test]
    fn test_decode_codelengths() {
        // data: &mut BitStream, num_of_codes: usize, code_length_prefixes: &Vec<u16>, code_length_symbols: &Vec<u16>) -> Vec<u8>

        let code_length_prefixes: Vec<u16> = vec![12, 0, 13, 14, 15, 2];
        let code_length_symbols: Vec<u16> = vec![1, 2, 4, 16, 17, 18];

        let num_of_codes: usize = 260;

        // test data from https://blog.za3k.com/understanding-gzip-2/
        // 01011001 01000111 11111111 00010000 01111011 01111000 01000000 01010100 00101110 00111011 01101111 10111100 00111010 00011000 00000000
        let mut bs = BitStream::new(vec![89, 71, 255, 16, 123, 120, 64, 84, 46, 59, 111, 188, 58, 24, 0], false);

        let result = decode_codelengths(&mut bs, num_of_codes, &code_length_prefixes, &code_length_symbols);

        let expected: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,

            1, 2,

            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            
            4, 4, 4, 4
        ];

        assert_eq!(result, expected);

        // same stream, parsing distance codes
        let num_of_dist_codes = 7;
        let dist_result = decode_codelengths(&mut bs, num_of_dist_codes, &code_length_prefixes, &code_length_symbols);
        let expected_dist = vec![
            2, 0, 0, 0, 2, 2, 2
        ];

        assert_eq!(dist_result, expected_dist);
    }

    #[test]
    fn test_fixed_huffman() {
        // correct cases generated using zlib library in python

        let decompressed_correct: Vec<u8> = vec![1, 2, 3, 4, 5];

        // ['01100011', '01100100', '01100010', '01100110', '01100001', '00000101', '00000000']
        let compressed: Vec<u8> = vec![99, 100, 98, 102, 97, 5, 0];

        let decompressed = deflate_decompressor::new_parse_deflate(compressed);
        assert_eq!(decompressed, decompressed_correct);



        let decompressed_correct: Vec<u8> = vec![1, 2, 3, 1, 2, 3, 5, 6, 1, 1, 1, 1, 1, 255, 234];

        // ['01100011', '01100100', '01100010', '01100110', '01100100', '01100010', '01100110', '01100101', '01100011', '00000100', '10000001', '11111111', '10101111', '00000000']
        let compressed: Vec<u8> = vec![99, 100, 98, 102, 100, 98, 102, 101, 99, 4, 129, 255, 175, 0];

        let decompressed = deflate_decompressor::new_parse_deflate(compressed);
        assert_eq!(decompressed, decompressed_correct);



        let decompressed_correct: Vec<u8> = vec![133, 4, 200, 200, 4, 200, 143, 56, 255, 255, 255, 255, 255, 255, 255, 255, 2, 3, 180, 105, 1, 1, 1, 48, 99, 45, 255, 255, 255, 255];

        // ['01101011', '01100101', '00111001', '01110001', '10000010', '11100101', '01000100', '10111111', '11000101', '01111111', '00101000', '01100000', '01100010', '11011110',
        // '10010010', '11001001', '11001000', '11001000', '01101000', '10010000', '10101100', '00001011', '11100010', '00000001', '00000000']
        let compressed: Vec<u8> = vec![107, 101, 57, 113, 130, 229, 68, 191, 197, 127, 40, 96, 98, 222, 146, 201, 200, 200, 104, 144, 172, 11, 226, 1, 0];

        let decompressed = deflate_decompressor::new_parse_deflate(compressed);
        assert_eq!(decompressed, decompressed_correct);
    }

    #[test]
    fn test_dynamic_huffman() {
        // 00011101 11000110 01001001 00000001 00000000 00000000 00010000 01000000 11000000 10101100 10100011 01111111 10001000 00111101 00111100 00100000 00101010 10010111 10011101 00110111 01011110 00011101 00001100
        let mut bs = BitStream::new(vec![29, 198, 73, 1, 0, 0, 16, 64, 192, 172, 163, 127, 136, 61, 60, 32, 42, 151, 157, 55, 94, 29, 12], false);
        bs.advance_bit_counter(3);

        let mut result = Vec::new();
        deflate_block::deflate_dynamic_huffman_block(&mut bs, &mut result);

        println!("result {:?}", result);
        
        assert_eq!(result, vec![97, 98, 97, 97, 98, 98, 98, 97, 98, 97, 97, 98, 97, 98, 98, 97, 97, 98, 97, 98, 97, 97, 97, 97, 98, 97, 97, 97, 98, 98, 98, 98, 98, 97, 97]);
    }
}
