use crate::bitstream::BitStream;


mod DeflateBlock {
    use super::BitStream;
    use crate::huffman_coding::*;
    use crate::low_level_functions::{bytes_vec_to_single, bits_to_byte};

    pub fn parse_next_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) -> bool {
        // given all the remaining bits, parse and return the first block
        data.reset_bit_position();
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

    fn deflate_uncompressed_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {
        data.move_to_next_byte();
        // next 16 bits (2 bytes) are length
        let length_bytes = ( data.next_byte(), data.next_byte() );
        // check against backup length (next 2 bytes), which is the bitwise NOT of len
        assert_eq!(length_bytes, ( !data.next_byte(), !data.next_byte() ) );

        let length = bytes_vec_to_single(&vec![length_bytes.0, length_bytes.1]) as usize;
        symbol_buffer.extend(data.next_n_bytes(length));
    }

    fn deflate_fixed_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {
        loop {
            let symbol = next_fixed_huffman_code(data);

            println!("{}", symbol);

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

    fn deflate_dynamic_huffman_block(data: &mut BitStream, symbol_buffer: &mut Vec<u8>) {

    }

}

pub mod DeflateDecompressor {
    use super::BitStream;
    use super::DeflateBlock;

    pub fn new_parse_deflate(data: Vec<u8>) -> Vec<u8> {
        let mut bit_stream = BitStream::new(data, false);

        let mut decompressed_data: Vec<u8> = Vec::new();

        loop {
            let bfinal = DeflateBlock::parse_next_block(&mut bit_stream, &mut decompressed_data);

            if bfinal {
                break
            }
        }

        decompressed_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_huffman() {
        let decompressed_correct: Vec<u8> = vec![1, 2, 3, 4, 5];
        // ['01100011', '01100100', '01100010', '01100110', '01100001', '00000101', '00000000']
        let compressed: Vec<u8> = vec![99, 100, 98, 102, 97, 5, 0];
        let decompressed = DeflateDecompressor::new_parse_deflate(compressed);
        assert_eq!(decompressed, decompressed_correct);

        let decompressed_correct: Vec<u8> = vec![1, 2, 3, 1, 2, 3, 5, 6, 1, 1, 1, 1, 1, 255, 234];
        // ['01100011', '01100100', '01100010', '01100110', '01100100', '01100010', '01100110', '01100101', '01100011', '00000100', '10000001', '11111111', '10101111', '00000000']
        let compressed: Vec<u8> = vec![99, 100, 98, 102, 100, 98, 102, 101, 99, 4, 129, 255, 175, 0];
        let decompressed = DeflateDecompressor::new_parse_deflate(compressed);
        assert_eq!(decompressed, decompressed_correct);
    }
}
