#![allow(dead_code)]

mod png_parser;
mod low_level_functions;
mod zlib;
mod deflate;
mod bitstream;
mod huffman_coding;
mod token;


use png_parser::PNGParser;
use wasm_bindgen::prelude::*;
use zlib::new_parse_zlib;

extern crate web_sys;

#[wasm_bindgen]
pub fn decode_png(data: Vec<u8>) -> String {
    let parser = PNGParser::new(data);
    
    format!("{:?}", parser.tokens)
}

#[wasm_bindgen]
pub fn decode_zlib(data: Vec<u8>) -> String {
    let (tokens, _decompressed) = new_parse_zlib(&data);
    let token_string = format!("{:?}", tokens);
    token_string
}

