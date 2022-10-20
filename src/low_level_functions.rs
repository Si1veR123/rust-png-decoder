
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

    // using | instead of +, untested
    ((b as u32) << 16) | (a as u32)
}

// Bit stream that takes a Vec of u8 bytes and can be iterated over, returning the bits, big endian or little endian.
pub struct BitStream {
    bytes: Vec<u8>,
    byte_position: usize,
    bit_position: u8,  // big endian bit position (0 is 128 value, 7 is 1)
    big_endian: bool,
}
impl BitStream {
    pub fn new(data: Vec<u8>, big_endian: bool) -> Self {
        Self {
            bytes: data,
            byte_position: 0,
            bit_position: if big_endian {7} else {0},
            big_endian,
        }
    }
}
impl Iterator for BitStream {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.bit_position < 8);
        let return_val = match self.bytes.get(self.byte_position) {
            // get bit at bit_position of current_byte
            Some(current_byte) => Some((current_byte >> self.bit_position) & 1u8),
            None => None,
        };

        // increment/decrement bit positions and increment byte position
        // big endian
        if self.big_endian {
            // prevent going below 0
            if self.bit_position == 0 {
                self.bit_position = 7;
                self.byte_position += 1;
            } else {
                self.bit_position -= 1;
            }

        // little endian
        } else {
            self.bit_position = (self.bit_position + 1) % 8;
            if self.bit_position == 0 {
                self.byte_position += 1
            }
        }
        return_val
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.bytes.get(n) {
            Some(byte) => Some(*byte),
            None => None
        }
    }
}
