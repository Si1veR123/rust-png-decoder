mod png_parser;
mod low_level_functions;
mod zlib;
mod deflate;
mod bitstream;

use png_parser::PNGParser;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{exit, ExitCode};

const FP: &str = r"E:\Programming\Rust\png-decoder\pngtest_1x1.png";

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

    println!("{}", parser.zlib_parser);
}
