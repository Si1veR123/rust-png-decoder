use crate::bitstream::BitStream;
use crate::low_level_functions::{bytes_vec_to_single, bits_to_byte};


struct DeflateBlock {
    bfinal: bool,
    btype: (u8, u8),
    uncompressed_data: Vec<u8>
}

impl DeflateBlock {
    pub fn new_from_remaining(data: &mut BitStream) ->  Self {
        // given all the remaining bits, parse and return the first block
        data.reset_bit_position();
        let bfinal = data.next().unwrap() == 1;
        let btype = (data.next().unwrap(), data.next().unwrap());
        let uncompressed_data = match btype {
            (0, 0) => Self::deflate_uncompressed_block(data),
            (0, 1) => Self::deflate_fixed_huffman_block(data),
            (1, 0) => Self::deflate_dynamic_huffman_block(data),
            (1, 1) => panic!("BTYPE has reserved value (11)"),
            _ => panic!("Invalid BTYPE")
        };
        Self {
            bfinal,
            btype,
            uncompressed_data
        }
    }

    fn deflate_uncompressed_block(data: &mut BitStream) -> Vec<u8> {
        data.move_to_next_byte();
        // next 16 bits (2 bytes) are length
        let length_bytes = ( data.next_byte(), data.next_byte() );
        // check against backup length (next 2 bytes), which is the bitwise NOT of len
        assert_eq!(length_bytes, ( !data.next_byte(), !data.next_byte() ) );

        let length = bytes_vec_to_single(&vec![length_bytes.0, length_bytes.1]) as usize;
        data.next_n_bytes(length)
    }

    fn deflate_fixed_huffman_block(data: &mut BitStream) -> Vec<u8> {
        Vec::new()
    }

    fn deflate_dynamic_huffman_block(data: &mut BitStream) -> Vec<u8> {
        Vec::new()
    }

}

pub struct DeflateDecompressor {
    blocks: Vec<DeflateBlock>
}

impl DeflateDecompressor {
    pub fn new(data: Vec<u8>) -> Self {
        Self::new_parse_deflate(data)
    }

    fn new_parse_deflate(data: Vec<u8>) -> Self {
        let mut bit_stream = BitStream::new(data, false);

        let mut blocks: Vec<DeflateBlock> = Vec::new();

        loop {
            let block = DeflateBlock::new_from_remaining(&mut bit_stream);
            if block.bfinal {
                blocks.push(block);
                break
            }
            blocks.push(block);
            
        }

        DeflateDecompressor { blocks }
    }
}
