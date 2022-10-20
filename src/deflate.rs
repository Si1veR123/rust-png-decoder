use crate::low_level_functions::BitStream;


struct DeflateBlock {
    bfinal: bool,
    btype: u8,
}

pub struct DeflateDecompressor {
    blocks: Vec<DeflateBlock>
}

impl DeflateDecompressor {
    pub fn new(data: Vec<u8>) -> Self {
        Self::new_parse_deflate(data)
    }

    fn new_parse_deflate(data: Vec<u8>) -> Self {
        let bit_stream = BitStream::new(data.iter().rev().cloned().collect(), false);
        for d in bit_stream {
            d;
        }
        
        DeflateDecompressor { blocks: vec![] }
    }
}
