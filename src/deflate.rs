use crate::bitstream::BitStream;
use crate::huffman_coding::*;
use crate::low_level_functions::{bytes_vec_to_single, bits_to_byte};
use crate::token::{Token, literal_token, reference_token};


fn parse_next_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> (bool, Vec<Token>) {
    // given all the remaining bits, add to symbol buffer, return bfinal and tokens
    let bfinal_byte = data.next().unwrap();
    let bfinal = bfinal_byte == 1;

    let bfinal_token = Token {
        bits: vec![bfinal_byte],
        using_bytes: false,
        nest_level: 0,
        data: bfinal_byte.to_string(),
        token_type: "bfinal".to_string(),
        description: if bfinal {"final block".to_string()} else {"not final block".to_string()}
    };

    let btype = (data.next().unwrap(), data.next().unwrap());
    let btype_token = Token {
        bits: vec![btype.0, btype.1],
        using_bytes: false,
        nest_level: 0,
        data: if btype == (0, 0) {"uncompressed".to_string()} else if btype == (1, 0) {"fixed".to_string()} else if btype == (0, 1) {"dynamic".to_string()} else {"invalid".to_string()},
        token_type: "btype".to_string(),
        description: "specifies block compression type".to_string()
    };

    let mut decompressing_tokens = match btype {
        (0, 0) => deflate_uncompressed_block(data, symbol_buffer),
        (1, 0) => deflate_fixed_huffman_block(data, symbol_buffer),
        (0, 1) => deflate_dynamic_huffman_block(data, symbol_buffer),
        (1, 1) => panic!("BTYPE has reserved value (11)"),
        _ => panic!("Invalid BTYPE")
    };

    decompressing_tokens.insert(0, bfinal_token);
    decompressing_tokens.insert(1, btype_token);

    (bfinal, decompressing_tokens)
}

fn deflate_uncompressed_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    
    if data.bit_position != 0 {
        let padding_token = Token {
            bits: vec![0; (8-data.bit_position) as usize],
            using_bytes: false,
            nest_level: 0,
            data: "padding".to_string(),
            token_type: "padding".to_string(),
            description: "padding to next byte".to_string(),
        };

        tokens.push(padding_token);
    
        data.move_to_next_byte();
    }
    
    // next 16 bits (2 bytes) are length
    let length_bytes = ( data.next_byte(), data.next_byte() );

    // check against backup length (next 2 bytes), which is the bitwise NOT of len
    let compliment_bytes = ( data.next_byte(), data.next_byte() );

    assert_eq!(length_bytes, (!compliment_bytes.0, !compliment_bytes.1));

    let length = bytes_vec_to_single(&vec![length_bytes.1, length_bytes.0]) as usize;

    let length_token = Token {
        bits: vec![length_bytes.0, length_bytes.1],
        using_bytes: true,
        nest_level: 0,
        data: length.to_string(),
        token_type: "bytes length".to_string(),
        description: "number of bytes to read from block".to_string(),
    };

    let length_compliment_token = Token {
        bits: vec![compliment_bytes.0, compliment_bytes.1],
        using_bytes: true,
        nest_level: 0,
        data: "n/a".to_string(),
        token_type: "complement bytes".to_string(),
        description: "bytes length with flipped bits".to_string(),
    };

    tokens.push(length_token);
    tokens.push(length_compliment_token);

    for _l in 0..length {
        let next_byte = data.next_byte();

        let token = literal_token(next_byte, None, 0);
        tokens.push(token);

        symbol_buffer.push(next_byte)
    }

    tokens
}


fn deflate_fixed_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    loop {
        let (symbol, bits) = next_fixed_huffman_symbol(data);
        if symbol > 256 {
            let (extra_length_bits, length) = decode_length(data, symbol);
            let distance_symbol_bits = data.next_n(5);
            let distance_symbol = bits_to_byte(&distance_symbol_bits, true);
            let (extra_distance_bits, distance) = decode_distance(data, distance_symbol);

            let duplicate_values = decode_duplicate_reference(symbol_buffer, length, distance);

            // bits + extra_length_bits + distance_symbol_bits + extra_distance_bits
            tokens.push(reference_token([bits, extra_length_bits, distance_symbol_bits, extra_distance_bits].concat(), distance, length, 0));

            symbol_buffer.extend(duplicate_values);
        }
        else if symbol == 256 {
            tokens.push(
                Token {
                    bits,
                    using_bytes: false,
                    nest_level: 0,
                    data: "256".to_string(),
                    token_type: "end of block".to_string(),
                    description: "All data from block has been decoded".to_string()
                }
            );
            break
        } else {
            let symbol = symbol as u8;

            let token = literal_token(symbol, Some(bits), 0);
            tokens.push(token);

            symbol_buffer.push(symbol);
        }
    }
    tokens
}

fn decode_codelengths(data: &mut BitStream, num_of_codes: usize, code_length_prefixes: &Vec<u16>, code_length_symbols: &Vec<u16>, code_length_codelengths: &Vec<u8>) -> (Vec<u8>, Vec<Token>) {
    // given huffman codes (symbols and prefixes) for the codelength table, decode a given number of codes from the bitstream

    let mut tokens: Vec<Token> = Vec::new();
    let mut decoded_codelengths: Vec<u8> = Vec::new();

    loop {
        let mut prefix_code: u16 = 0;
        let mut prefix_code_bits: Vec<u8> = Vec::new();
        let mut current_codelength = 0;

        loop {
            // read next bit to prefix code
            let next_bit = data.next().unwrap();
            prefix_code_bits.push(next_bit);

            prefix_code = (prefix_code << 1) | (next_bit as u16);
            current_codelength += 1;

            // valid prefixes are where the codelength of this prefix == current_codelength
            let mut valid_prefixes = code_length_prefixes
                .iter()
                .cloned()
                .enumerate()
                .filter(
                    |(i, _)| *code_length_codelengths.get(*i).unwrap() == current_codelength
                );

            let prefix_position = valid_prefixes.position(|(_, x)| x == prefix_code);
            
            if prefix_position.is_none() { continue }

            // get position, and use position to index code_length_symbols to find the symbol
            let symbol = *code_length_symbols.get( code_length_prefixes.iter().position(|&x| x == prefix_code).unwrap() ).unwrap() as u8; // code length symbols <= 18

            match symbol {
                0..=15 => {
                    // literal code length
                    decoded_codelengths.push(symbol);
                    tokens.push(
                        literal_token(symbol, Some(prefix_code_bits), 0)
                    );
                },
                16 => {
                    // Copy the previous code length 3-6 times, 2 extra bits
                    let &prev_symbol = decoded_codelengths.last().unwrap();
                    let next_bits = data.next_n(2);
                    let repitions = (bits_to_byte(&next_bits, false) >> 6u8) + 3;

                    prefix_code_bits.extend(next_bits);
                    
                    tokens.push(reference_token(prefix_code_bits, 1, repitions as u16, 0));

                    for _ in 0..repitions {
                        decoded_codelengths.push(prev_symbol);
                    }
                },
                17 => {
                    // Copy 0 3-10 times, 3 extra bits
                    let next_bits = data.next_n(3);
                    
                    let repitions = (bits_to_byte(&next_bits, false) >> 5u8) + 3;

                    let zero_vector = vec![0, repitions];
                    prefix_code_bits.extend(next_bits);
                    tokens.push(
                        Token {
                            bits: prefix_code_bits,
                            using_bytes: false,
                            nest_level: 0,
                            data: format!("{:?}", zero_vector),
                            token_type: "repeated_0".to_string(),
                            description: "Repeat 0 3-10 times".to_string(),
                        }
                    );

                    for _ in 0..repitions {
                        decoded_codelengths.push(0);
                    }
                },
                18 => {
                    // Copy 0 11-138 times, 7 extra bits
                    let next_bits = data.next_n(7);

                    let repitions = (bits_to_byte(&next_bits, false) >> 1u8) + 11;

                    let zero_vector = vec![0, repitions];
                    prefix_code_bits.extend(next_bits);
                    tokens.push(
                        Token {
                            bits: prefix_code_bits,
                            using_bytes: false,
                            nest_level: 0,
                            data: format!("{:?}", zero_vector),
                            token_type: "repeated_0_long".to_string(),
                            description: "Repeat 0 11-138 times".to_string(),
                        }
                    );

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
    (decoded_codelengths, tokens)
}

fn deflate_dynamic_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let num_of_normal_codes_bits = data.next_n(5);

    let num_of_normal_codes = ((bits_to_byte(&num_of_normal_codes_bits, false) >> 3) as u16 + 257) as usize;

    tokens.push(
        Token {
            bits: num_of_normal_codes_bits,
            using_bytes: false,
            nest_level: 0,
            data: num_of_normal_codes.to_string(),
            token_type: "HLIT".to_string(),
            description: "# of Literal/Length codes".to_string(),
        }
    );

    let num_of_dist_codes_bits = data.next_n(5);
    let num_of_dist_codes = ((bits_to_byte(&num_of_dist_codes_bits, false) >> 3) + 1) as usize;

    tokens.push(
        Token {
            bits: num_of_dist_codes_bits,
            using_bytes: false,
            nest_level: 0,
            data: num_of_dist_codes.to_string(),
            token_type: "HDIST".to_string(),
            description: "# of Distance codes".to_string(),
        }
    );

    // 1) Parse codelength huffman codes
    let num_of_codelength_codes_bits = data.next_n(4);
    let num_of_codelength_codes = ((bits_to_byte(&num_of_codelength_codes_bits, false) >> 4) + 4) as usize;

    tokens.push(
        Token {
            bits: num_of_codelength_codes_bits,
            using_bytes: false,
            nest_level: 0,
            data: num_of_codelength_codes.to_string(),
            token_type: "HCLEN".to_string(),
            description: "# of Code Length codes".to_string(),
        }
    );

    // reorder codelength codelengths
    const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut code_length_codelengths = vec![0u8; 19];
    let mut code_length_codelengths_bits = Vec::new();

    for i in 0..(num_of_codelength_codes) {
        let bits = data.next_n(3);
        let codelength_codelength = bits_to_byte(&bits, false) >> 5;

        code_length_codelengths_bits.extend(bits);

        code_length_codelengths[ORDER[i]] = codelength_codelength;
    }

    tokens.push(
        Token {
            bits: code_length_codelengths_bits,
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", code_length_codelengths),
            token_type: "CLEN codelengths".to_string(),
            description: "Codelengths for codelength alphabet, reordered.".to_string(),
        }
    );

    let (code_length_symbols, code_length_prefixes) = huffman_codes_from_codelengths(&code_length_codelengths);
    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", code_length_symbols),
            token_type: "cl_symbols".to_string(),
            description: "Symbols for codelength alphabet".to_string(),
        }
    );
    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", code_length_prefixes),
            token_type: "cl_prefixes".to_string(),
            description: "Prefixes for codelength alphabet in base 10".to_string(),
        }
    );

    // 2) Parse main huffman codelengths
    
    let filtered_code_length_codelengths = code_length_codelengths.iter().cloned().filter(|&x| x > 0).collect();
    let (decoded_normal_codelengths, decoded_normal_codelengths_tokens) = decode_codelengths(data, num_of_normal_codes, &code_length_prefixes, &code_length_symbols, &filtered_code_length_codelengths);
    let (decoded_distance_codelengths, decoded_distance_codelengths_tokens) = decode_codelengths(data, num_of_dist_codes, &code_length_prefixes, &code_length_symbols, &filtered_code_length_codelengths);
    tokens.extend(decoded_normal_codelengths_tokens);
    tokens.extend(decoded_distance_codelengths_tokens);

    let (huffman_normal_symbols, huffman_normal_prefixes) = huffman_codes_from_codelengths(&decoded_normal_codelengths);

    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", huffman_normal_symbols),
            token_type: "normal_symbols".to_string(),
            description: "Symbols for normal alphabet".to_string(),
        }
    );
    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", huffman_normal_prefixes),
            token_type: "normal_prefixes".to_string(),
            description: "Prefixes for normal alphabet in base 10".to_string(),
        }
    );
    let huffman_normal_codelengths: Vec<u8> = decoded_normal_codelengths.iter().cloned().filter(|&x| x > 0).collect();

    let (huffman_distance_symbols, huffman_distance_prefixes) = huffman_codes_from_codelengths(&decoded_distance_codelengths);

    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", huffman_distance_symbols),
            token_type: "distance_symbols".to_string(),
            description: "Symbols for distance alphabet".to_string(),
        }
    );
    tokens.push(
        Token {
            bits: vec![],
            using_bytes: false,
            nest_level: 0,
            data: format!("{:?}", huffman_distance_prefixes),
            token_type: "distance_prefixes".to_string(),
            description: "Prefixes for distance alphabet".to_string(),
        }
    );

    let huffman_distance_codelengths: Vec<u8> = decoded_distance_codelengths.iter().cloned().filter(|&x| x > 0).collect();

    // 3) Parse data using huffman codes
    loop {
        let (symbol, symbol_bits) = next_huffman_symbol(data, &huffman_normal_symbols, &huffman_normal_prefixes, &huffman_normal_codelengths, true);
        if symbol > 256 {
            let (extra_length_bits, length) = decode_length(data, symbol);

            let (distance_symbol, distance_symbol_bits) = next_huffman_symbol(data, &huffman_distance_symbols, &huffman_distance_prefixes, &huffman_distance_codelengths, true);

            let (extra_distance_bits, distance) = decode_distance(data, distance_symbol as u8);
            let duplicate_values = decode_duplicate_reference(symbol_buffer, length, distance);
            symbol_buffer.extend(duplicate_values);

            // bits + extra_length_bits + distance_symbol_bits + extra_distance_bits
            let all_bits = [symbol_bits, extra_length_bits, distance_symbol_bits, extra_distance_bits].concat();

            tokens.push(
                reference_token(
                    all_bits, distance, length, 0
                )
            )
        }
        else if symbol == 256 {
            tokens.push(
                Token {
                    bits: symbol_bits,
                    using_bytes: false,
                    nest_level: 0,
                    data: "256".to_string(),
                    token_type: "end of block".to_string(),
                    description: "All data from block has been decoded".to_string()
                }
            );
            break
        } else {
            let symbol = symbol as u8;
            tokens.push( literal_token(symbol, Some(symbol_bits), 0) );
            symbol_buffer.push(symbol as u8);
        }
    }
    tokens
}

pub fn new_parse_deflate(data: Vec<u8>) -> (Vec<Token>, Vec<u8>) {
    let mut bit_stream = BitStream::new(data, false);

    let mut all_tokens: Vec<Token> = Vec::new();
    let mut decompressed_data = Vec::new();

    loop {
        let (bfinal, tokens) = parse_next_block(&mut bit_stream, &mut decompressed_data);

        all_tokens.extend(tokens);

        if bfinal {
            break
        }
    }

    let padding = (bit_stream.bytes.len()*8) - bit_stream.current_abs_bit_position();
    
    if padding > 0 {
        all_tokens.push(
            Token {
                bits: vec![0; padding],
                using_bytes: false,
                nest_level: 0,
                data: "End of deflate padding".to_string(),
                token_type: "padding".to_string(),
                description: "Padding after deflate stream to next byte boundary".to_string()
            }
        )
    }

    (all_tokens, decompressed_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // 00011101 11000110 01001001 00000001 00000000 00000000 00010000 01000000 11000000 10101100 10100011 01111111 10001000 00111101 00111100 00100000 00101010 10010111 10011101 00110111 01011110 00011101 00001100
        let data = vec![29, 198, 73, 1, 0, 0, 16, 64, 192, 172, 163, 127, 136, 61, 60, 32, 42, 151, 157, 55, 94, 29, 12];

        let (tokens, decompressed) = new_parse_deflate(data);

        println!("{:?}", tokens);
        assert_eq!(decompressed,  vec![97, 98, 97, 97, 98, 98, 98, 97, 98, 97, 97, 98, 97, 98, 98, 97, 97, 98, 97, 98, 97, 97, 97, 97, 98, 97, 97, 97, 98, 98, 98, 98, 98, 97, 97]);
    }
}
