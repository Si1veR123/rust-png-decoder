
struct DeflateBlock {

}

pub struct DeflateDecompressor {
    data: Vec<u8>
}

impl DeflateDecompressor {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
}
