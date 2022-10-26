
pub fn bytes_vec_to_single(hex_vec: &Vec<u8>) -> u32 {
    let mut running_value = 0u32;
    for (index, hex) in hex_vec.iter().rev().enumerate() {
        // if second from end, multiply by 256 (16*16)
        // if third from end, multiply by 512 (16*16)^2 etc.
        let place_value = 256u32.pow(index as u32) * (*hex as u32);
        running_value += place_value;
    }
    running_value
}

pub fn bits_to_byte(bits: &Vec<u8>, big_endian: bool) -> u8 {
    // big endian: first bit is msb
    // little endian: first bit is lsb

    // result is aligned to left (e.g. if 7 bits given, lsb in byte will be 0)
    assert!(bits.len() <= 8);
    let mut running_value = 0u8;
    for &bit in bits {
        
        running_value = match big_endian {
            // shift running value to right, shift bit to '128 value', bitwise OR
            false => (running_value >> 1) | (bit << 7),
            // shift to left, bitwise OR with next bit
            true => (running_value << 1) | bit,
        }
    }
    running_value
}

pub fn adler_32(bytes: &Vec<u8>) -> u32 {
    let mut a = 1u16;
    let mut b = 0u16;

    for &byte in bytes {
        a = (a + (byte as u16)) % 65521;
        b = (b + a) % 65521;
    }

    // using | instead of +, untested
    ((b as u32) << 16) | (a as u32)
}

