use std::fmt::Display;
use crate::low_level_functions::bytes_vec_to_single;
use crate::zlib::ZLibParser;

// METADATA
pub struct PNGMetadata {
    pub bit_depth: u8,
    pub width: usize,
    pub height: usize,
    pub color_type: u8,
    pub filesize: usize,
}

impl Display for PNGMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Height: {}, Width: {}, Bit Depth: {}, Color Type: {}, Filesize: {}", self.height, self.width, self.bit_depth, self.color_type, self.filesize)
    }
}

// PNG CHUNKS
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

trait ImageData {
    fn from_png_stream(data: &Vec<u8>, width: usize) -> Self;
}

// RAW IMAGE PIXELS
pub struct RGBImageData {
    pub data: Vec<Vec<[u8; 3]>>
}

impl ImageData for RGBImageData {
    fn from_png_stream(data: &Vec<u8>, width: usize) -> Self {
        let mut image_data = Vec::new();

        let bits_row_width = width*3;

        let mut row: usize = 0;
        loop {
            let mut row_pixels = Vec::with_capacity(width);

            let filter_type = data.get(row * (bits_row_width+1));
            if filter_type.is_none() { break }
            if *filter_type.unwrap() != 0 { panic!("Unsupported filter type") }

            let row_start = ((bits_row_width+1)*row) + 1;

            for col in 0..width {
                let pixel_start = row_start+(col*3);
                row_pixels.push([data[pixel_start], data[pixel_start+1], data[pixel_start+2]])
            }
            image_data.push(row_pixels);
            row += 1;
        }
        Self {
            data: image_data
        }
    }
}

// PNG Parser
pub struct PNGParser {
    pub image_data: RGBImageData,
    pub chunks: Vec<PNGChunk>,
    pub metadata: PNGMetadata,
}

impl PNGParser {
    pub fn new(data: Vec<u8>) -> Self {
        let (metadata, chunks, image_data) = Self::parse_png(data);

        Self {
            image_data,
            chunks,
            metadata,
        }
    }

    fn finish_read_chunk(mut_data: &mut Vec<u8>, chunk_length: &u32) {
        // remove parsed bytes from data, and deallocate vec to free up memory
        // TODO: benchmark if this improves performance or memory usage
        mut_data.drain(0..(12+chunk_length) as usize);
        mut_data.shrink_to_fit();
    }

    pub fn parse_png(data: Vec<u8>) -> (PNGMetadata, Vec<PNGChunk>, RGBImageData) {
        let filesize = (&data).len();
        let mut mut_data = data;
        mut_data.drain(0..8);

        let mut chunks: Vec<PNGChunk> = Vec::new();
        let mut idat_combined: Vec<u8> = Vec::new();

        while mut_data.len() > 12 {  // 12 so IEND chunk isn't included
            // first 4 bytes are chunk length
            let chunk_length = bytes_vec_to_single(&mut_data[0..4].to_vec());
            // next 4 bytes are chunk type
            let chunk_type: String = mut_data[4..8].iter().map(|x| *x as char).collect();

            if chunk_type.chars().nth(0).expect("No chunk type").is_lowercase() {
                Self::finish_read_chunk(&mut mut_data, &chunk_length);
                continue;
            }

            // next *chunk length* bytes are chunk data
            let data_chunk_end = 8+(chunk_length as usize);
            let chunk_data = mut_data[8..data_chunk_end].iter().cloned().collect::<Vec<u8>>();
            if chunk_type == "IDAT".to_string() {
                idat_combined.extend(chunk_data);
                Self::finish_read_chunk(&mut mut_data, &chunk_length);
                continue;
            }

            // next 4 bytes are crc-32 check
            let crc = bytes_vec_to_single(&mut_data[data_chunk_end..data_chunk_end+4].iter().cloned().collect());

            let current_chunk = PNGChunk {
                chunk_data,
                chunk_type,
                crc,
            };
            chunks.push(current_chunk);
            Self::finish_read_chunk(&mut mut_data, &chunk_length);
        }

        let ihdr = chunks.get(0).expect("No chunks");
        if ihdr.chunk_type != "IHDR".to_string() {
            panic!("First chunk isn't IHDR. Parse failed.")
        }

        if idat_combined.len() == 0 {
            panic!("No IDAT chunks found")
        }

        let metadata = PNGMetadata {
            width: bytes_vec_to_single(&ihdr.chunk_data[0..4].to_vec()) as usize,
            height: bytes_vec_to_single(&ihdr.chunk_data[4..8].to_vec()) as usize,
            bit_depth: ihdr.chunk_data[8],
            color_type: ihdr.chunk_data[9],
            filesize: filesize as usize,
        };

        let zlib_parser = ZLibParser::new(idat_combined);
        let image_data = RGBImageData::from_png_stream(&zlib_parser.decompressed, metadata.width);

        (metadata, chunks, image_data)
    }
}
