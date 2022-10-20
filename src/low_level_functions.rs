
pub fn hex_vec_to_single(hex_vec: &Vec<u8>) -> u32 {
    let mut running_value = 0u32;
    for (index, hex) in hex_vec.iter().rev().enumerate() {
        // if second from end, multiply by 256 (16*16)
        // if third from end, multiply by 512 (16*16)^2 etc.
        let place_value = 256u32.pow(index as u32) * (*hex as u32);
        running_value += place_value;
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

    ((b as u32) << 16) + (a as u32)
}
