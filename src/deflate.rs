use crate::bitstream::BitStream;
use crate::low_level_functions::hex_vec_to_single;


struct DeflateBlock {
    bfinal: bool,
    btype: u8,
    uncompressed_data: Vec<u8>
}

impl DeflateBlock {
    pub fn new_from_remaining(data: &BitStream) ->  Self {
        // given all the remaining bits, parse and return the first block
        let bfinal = data.next().unwrap() == 1;
        let btype: u8 = 
            (hex_vec_to_single(&data.next_n(2).to_owned()))
            .try_into()
            .expect("BTYPE overflow in deflate");

        let uncompressed_data = match btype {
            0 => Self::deflate_uncompressed_block(data),
            1 => Self::deflate_fixed_huffman_block(data),
            2 => Self::deflate_dynamic_huffman_block(data),
            3 => panic!("BTYPE has reserved value (11)"),
            _ => panic!("Invalid BTYPE")
        };

        Self {
            bfinal,
            btype,
            uncompressed_data
        }
    }

    fn deflate_uncompressed_block(data: &BitStream) -> Vec<u8> {

    }

    fn deflate_fixed_huffman_block(data: &BitStream) -> Vec<u8> {

    }

    fn deflate_dynamic_huffman_block(data: &BitStream) -> Vec<u8> {

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
        let bit_stream = BitStream::new(data, false);

        let mut blocks: Vec<DeflateBlock> = Vec::new();

        loop {
            let block = DeflateBlock::new_from_remaining(&bit_stream);
            blocks.push(block);
            if block.bfinal {
                break
            }
        }

        DeflateDecompressor { blocks }
    }
}
