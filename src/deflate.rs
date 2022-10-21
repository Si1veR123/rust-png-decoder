use crate::bitstream::BitStream;

struct DeflateBlock {
    bfinal: u8,
    btype: u8,
    original_bit_length: usize,
    decompressed_data: Vec<u8>
}
/*
impl DeflateBlock {
    pub fn new_from_remaining(data: &BitStream) ->  Self {
        // given all the remaining bits, parse and return the first block
        let bfinal = data.next();
    }

    pub fn deflate_block(data: Vec<u8>) -> Vec<u8> {

    }
    fn deflate_uncompressed_block(data: Vec<u8>) -> Vec<u8> {

    }
    fn deflate_fixed_huffman_block(data: Vec<u8>) -> Vec<u8> {

    }
    fn deflate_dynamic_huffman_block(data: Vec<u8>) -> Vec<u8> {

    }
}
*/
pub struct DeflateDecompressor {
    blocks: Vec<DeflateBlock>
}

impl DeflateDecompressor {
    pub fn new(data: Vec<u8>) -> Self {
        Self::new_parse_deflate(data)
    }

    fn new_parse_deflate(data: Vec<u8>) -> Self {
        let bit_stream = BitStream::new(data, false);

        for bit in bit_stream {
            
        }

        DeflateDecompressor { blocks: vec![] }
    }
}
