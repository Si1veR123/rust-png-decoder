use std::fmt::Display;
use crate::low_level_functions::bytes_vec_to_single;
use crate::token::Token;
use crate::zlib::{new_parse_zlib, ZLibInfo};

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
}

impl Display for PNGChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CHUNK_LENGTH {}, CHUNK_TYPE {}", self.chunk_data.len(), self.chunk_type)
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
    pub tokens: Vec<Token>,
    pub image_data: RGBImageData,
}

impl PNGParser {
    pub fn new(data: Vec<u8>) -> Self {
        let (tokens, image_data) = Self::parse_png(data);

        Self {
            image_data,
            tokens
        }
    }

    fn finish_read_chunk(mut_data: &mut Vec<u8>, chunk_length: &u32) {
        // remove parsed bytes from data, and deallocate vec to free up memory
        // TODO: benchmark if this improves performance or memory usage
        mut_data.drain(0..(12+chunk_length) as usize);
        mut_data.shrink_to_fit();
    }

    fn parse_png(data: Vec<u8>) -> (Vec<Token>, RGBImageData) {
        let mut tokens: Vec<Token> = Vec::new();

        let filesize = (&data).len();
        let mut mut_data = data;
        let header = mut_data.drain(0..8);

        tokens.push(
            Token {
                bits: header.into_iter().collect(),
                using_bytes: true,
                nest_level: 2,
                data: "png header".to_string(),
                token_type: "header".to_string(),
                description: "All PNGs contain these bytes".to_string()
            }
        );

        let mut idat_combined: Vec<u8> = Vec::new();
        let mut ihdr: Option<PNGChunk> = None;
        let mut parsing_idat = false;
        let mut decompressed = Vec::new();

        while mut_data.len() > 12 {  // 12 so IEND chunk isn't included

            // first 4 bytes are chunk length
            let chunk_length_bytes = mut_data[0..4].to_vec();
            let chunk_length = bytes_vec_to_single(&chunk_length_bytes);

            // next 4 bytes are chunk type
            let chunk_type_bytes = &mut_data[4..8];
            let chunk_type: String = chunk_type_bytes.iter().map(|x| *x as char).collect();

            // non essential chunks are skipped
            if chunk_type.chars().nth(0).expect("No chunk type").is_lowercase() {
                Self::finish_read_chunk(&mut mut_data, &chunk_length);
                continue;
            }

            // next *chunk length* bytes are chunk data
            let data_chunk_end = 8+(chunk_length as usize);
            let mut chunk_data = mut_data[8..data_chunk_end].iter().cloned().collect::<Vec<u8>>();

            if chunk_type == "IHDR".to_string() {
                ihdr = Some(PNGChunk {
                    chunk_type: chunk_type.clone(),
                    chunk_data: chunk_data.clone()
                });
            } else {
                if chunk_type == "IDAT".to_string() {
                    if !parsing_idat {
                        // first IDAT chunk
                        tokens.push(
                            Token {
                                bits: vec![],
                                using_bytes: false,
                                nest_level: 2,
                                data: "IDAT start".to_string(),
                                token_type: "idat_start".to_string(),
                                description: "Start of image data chunks, following data is all IDAT chunks combined".to_string()
                            }
                        );
                    }

                    parsing_idat = true;

                    idat_combined.extend(chunk_data);
                    Self::finish_read_chunk(&mut mut_data, &chunk_length);
                    continue;
                } else if parsing_idat {
                    // ended idat chunks
                    let (zlib_tokens, decompressed_d) = new_parse_zlib(&idat_combined);
                    decompressed = decompressed_d;
                    tokens.extend(zlib_tokens);

                    tokens.push(
                        Token {
                            bits: vec![],
                            using_bytes: false,
                            nest_level: 2,
                            data: "IDAT end".to_string(),
                            token_type: "idat_end".to_string(),
                            description: "End of combined IDAT chunks".to_string()
                        }
                    );
                }
            }

            // next 4 bytes are crc-32 check
            let crc_bytes = &mut_data[data_chunk_end..data_chunk_end+4];
            // let crc = bytes_vec_to_single(&crc_bytes.iter().cloned().collect());

            tokens.push(
                Token {
                    bits: chunk_length_bytes,
                    using_bytes: true,
                    nest_level: 2,
                    data: format!("length {}", chunk_length),
                    token_type: "chunk_length".to_string(),
                    description: "Number of bytes in chunk data".to_string()
                }
            );

            tokens.push(
                Token {
                    bits: chunk_type_bytes.to_vec(),
                    using_bytes: true,
                    nest_level: 2,
                    data: format!("chunk type {}", chunk_type),
                    token_type: "chunk_type".to_string(),
                    description: "Type of chunk".to_string()
                }
            );

            tokens.push(
                Token {
                    bits: chunk_data,
                    using_bytes: true,
                    nest_level: 2,
                    data: "chunk data".to_string(),
                    token_type: "chunk_data".to_string(),
                    description: "Chunk bytes".to_string()
                }
            );

            tokens.push(
                Token {
                    bits: crc_bytes.to_vec(),
                    using_bytes: true,
                    nest_level: 2,
                    data: "crc-32".to_string(),
                    token_type: "crc_32".to_string(),
                    description: "crc-32 check on chunk type and chunk data".to_string()
                }
            );

            Self::finish_read_chunk(&mut mut_data, &chunk_length);
        }

        let ihdr = ihdr.expect("No IHDR Chunk");

        if idat_combined.len() == 0 {
            panic!("No IDAT chunks found")
        }

        let width_bytes = ihdr.chunk_data[0..4].to_vec();
        let width = bytes_vec_to_single(&width_bytes) as usize;

        let height_bytes = ihdr.chunk_data[4..8].to_vec();
        let height = bytes_vec_to_single(&height_bytes) as usize;

        // header data placed 1 token after the main PNG header
        tokens.insert(1, 
            Token {
                bits: width_bytes,
                using_bytes: true,
                nest_level: 2,
                data: format!("Width: {}", width),
                token_type: "width".to_string(),
                description: "Image width".to_string()
            }
        );

        tokens.insert(2, 
            Token {
                bits: height_bytes,
                using_bytes: true,
                nest_level: 2,
                data: format!("Height: {}", height),
                token_type: "height".to_string(),
                description: "Image height".to_string()
            }
        );

        tokens.insert(3, 
            Token {
                bits: vec![ihdr.chunk_data[8]],
                using_bytes: true,
                nest_level: 2,
                data: format!("Bit depth: {}", ihdr.chunk_data[8]),
                token_type: "bit_depth".to_string(),
                description: "Image bit depth".to_string()
            }
        );

        tokens.insert(4, 
            Token {
                bits: vec![ihdr.chunk_data[9]],
                using_bytes: true,
                nest_level: 2,
                data: format!("Color Type: {}", ihdr.chunk_data[9]),
                token_type: "color_type".to_string(),
                description: "PNG image color type".to_string()
            }
        );

        let image_data = RGBImageData::from_png_stream(&decompressed, width);

        (tokens, image_data)
    }
}
