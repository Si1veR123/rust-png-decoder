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
const FP: &str = r"E:\Programming\Rust\rust-png-decoder\pngtest_1x1.png";

fn main() {
    /*
    let mut codelengths = vec![8; 144];
    codelengths.append(&mut ( vec![9; 112]));
    codelengths.append(&mut vec![7; 24]);
    codelengths.append(&mut vec![8; 8]);
    let prefixcodes = huffman_coding::prefix_codes_from_codelengths(codelengths);

    println!("{:?}", prefixcodes);
    */

    let mut bs = bitstream::BitStream::new(vec![12], false);
    bs.reset_bit_position();
    let i = huffman_coding::next_fixed_huffman_code(&mut bs);
    println!("{}", bs.current_abs_bit_position());
    println!("{}", i);  

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
