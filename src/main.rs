use png_decoder::zlib::ZlibParser;

use std::fs::read_dir;
use std::fs::read;
use std::time::Instant;

fn main() {
    let test_data = read_dir("zlib_tests").unwrap();
    let mut times = Vec::new();

    for file in test_data {
        let fp = file.unwrap().path();
        let data = read(&fp).unwrap();
        let start = Instant::now();
        let mut zlib = ZlibParser::new(&data);
        let _zlib_decompressed = zlib.parse().unwrap();
        let end = Instant::now();
        times.push(end-start);
        println!("{:?}", end-start);
    }
}
