use std::fmt::Display;
use crate::low_level_functions::hex_vec_to_single;
use crate::zlib::ZLibParser;

pub struct PNGMetadata {
    pub bit_depth: u8,
    pub width: u32,
    pub height: u32,
    pub color_type: u8,
    pub filesize: u32,
}

impl Display for PNGMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Height: {}, Width: {}, Bit Depth: {}, Color Type: {}, Filesize: {}", self.height, self.width, self.bit_depth, self.color_type, self.filesize)
    }
}

pub struct PNGChunk {
    pub chunk_type: String,
    pub chunk_data: Vec<u8>,
    pub crc: u32,
}

impl Display for PNGChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CHUNK_LENGTH {}, CHUNK_TYPE {}, CRC {}", self.chunk_data.len(), self.chunk_type, self.crc)
    }
}

pub struct PNGParser {
    pub zlib_parser: ZLibParser,
    pub chunks: Vec<PNGChunk>,
    pub metadata: PNGMetadata,
}

impl PNGParser {
    pub fn new(data: Vec<u8>) -> Self {
        let (metadata, chunks, zlib_parser) = Self::parse_png(data);

        Self {
            zlib_parser,
            chunks,
            metadata,
        }
    }

    pub fn parse_png(data: Vec<u8>) -> (PNGMetadata, Vec<PNGChunk>, ZLibParser) {
        let filesize = (&data).len();
        let mut byte_iter = data;
        byte_iter.drain(0..8);

        let mut chunks: Vec<PNGChunk> = Vec::new();
        let mut idat_combined: Vec<u8> = Vec::new();

        while byte_iter.len() > 12 {  // 12 so IEND chunk isn't included
            // first 4 bytes are chunk length
            let chunk_length = hex_vec_to_single(&byte_iter[0..4].to_vec());
            // next 4 bytes are chunk type
            let chunk_type: String = byte_iter[4..8].iter().map(|x| *x as char).collect();

            if chunk_type.chars().nth(0).expect("No chunk type").is_lowercase() {
                // skip all non critical chunks
                byte_iter.drain(0..(12+chunk_length) as usize);
                continue;
            }

            // next *chunk length* bytes are chunk data
            let data_chunk_end = 8+(chunk_length as usize);
            let chunk_data = byte_iter[8..data_chunk_end].iter().cloned().collect::<Vec<u8>>();
            if chunk_type == "IDAT".to_string() {
                idat_combined.extend(chunk_data);
                byte_iter.drain(0..(12+chunk_length) as usize);
                continue;
            }

            // next 4 bytes are crc-32 check
            let crc = hex_vec_to_single(&byte_iter[data_chunk_end..data_chunk_end+4].iter().cloned().collect());

            let current_chunk = PNGChunk {
                chunk_data,
                chunk_type,
                crc,
            };
            chunks.push(current_chunk);
            byte_iter.drain(0..(12+chunk_length) as usize);
        }

        let ihdr = chunks.get(0).expect("No chunks");
        if ihdr.chunk_type != "IHDR".to_string() {
            panic!("First chunk isn't IHDR. Parse failed.")
        }

        if idat_combined.len() == 0 {
            panic!("No IDAT chunks found")
        }

        let metadata = PNGMetadata {
            width: hex_vec_to_single(&ihdr.chunk_data[0..4].to_vec()),
            height: hex_vec_to_single(&ihdr.chunk_data[4..8].to_vec()),
            bit_depth: ihdr.chunk_data[8],
            color_type: ihdr.chunk_data[9],
            filesize: filesize as u32,
        };

        let zlib_parser = ZLibParser::new(idat_combined);
        (metadata, chunks, zlib_parser)
    }
}
