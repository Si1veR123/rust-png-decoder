#![allow(dead_code)]

mod png_parser;
mod low_level_functions;
mod zlib;
mod deflate;
mod bitstream;
mod huffman_coding;

use png_parser::PNGParser;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const FP: &str = r"E:\Programming\Rust\rust-png-decoder\pngtest_100x100.png";

/*
// fixed huffman codes generation
let mut codelengths = vec![8; 144];
codelengths.append(&mut ( vec![9; 112]));
codelengths.append(&mut vec![7; 24]);
codelengths.append(&mut vec![8; 8]);
let prefixcodes = huffman_coding::prefix_codes_from_codelengths(codelengths);

println!("{:?}", prefixcodes);
*/

fn main() {
    let mut file_buffer = Vec::<u8>::new();

    {
        let path = Path::new(FP);
        let mut file = File::open(path).expect("File not found");
        let _file_size = file.read_to_end(&mut file_buffer).expect("Can't read file");
    }

    let parser = PNGParser::new(file_buffer);
    
    println!("{}", parser.metadata);

    for chunk in &parser.chunks {
        println!("{}", chunk);
    }

    println!("{:?}", parser.image_data.data);
}
