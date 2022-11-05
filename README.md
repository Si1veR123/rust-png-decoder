# ZLib/PNG decoder and breakdown

## About
This is a website that breaks down a compressed ZLib stream or a PNG binary into 'tokens' demonstrating the structure of the format.  

## Rust Decoder/Tokenisation
The rust library, when compiled to web assembly (wasm), exposes 2 functions to JavaScript, `decode_zlib` and `decode_png`. These each decode an array of bytes to an array of tokens, which are returned as a string in JSON format. Each token contains information about a section of the compressed data.  

The decoder/tokenisation implemented in Rust can be found in [src](./src/).  

## JavaScript front-end
The javascript used on the site calls one of the functions from the wasm binary, parses the resulting tokens, and generates the HTML to display the tokens.  
The website front-end can be found in [pkg](./pkg/).  

## Running the website
The pkg/ folder can be served. To rebuild the rust wasm binary
use [wasm-pack](https://developer.mozilla.org/en-US/docs/WebAssembly/Rust_to_wasm). Once the wasm files are generated (into the pkg folder), the pkg/ folder can be served.
