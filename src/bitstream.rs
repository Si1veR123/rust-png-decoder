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
    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.bytes[start..end]
    }

    fn advance_bit_counter(&mut self, n: usize) {
        if self.big_endian {
            self.bit_position = 7 - self.bit_position;
        }

        // find bit pos relative to entire stream
        let abs_bit_pos = (self.byte_position << 3) + (self.bit_position as usize);
        let new_bit_pos = abs_bit_pos + n;

        self.byte_position = new_bit_pos / 8;
        self.bit_position = (new_bit_pos % 8) as u8;

        if self.big_endian {
            self.bit_position = 7 - self.bit_position;
        }
    }

    pub fn current_bit_position(&self) -> usize {
        match self.big_endian {
            false => (self.byte_position*8) + (self.bit_position as usize),
            true => (self.byte_position*8) + (7-self.bit_position as usize)
        }
    }
    pub fn next_n(&mut self, n: usize) -> &[u8] {
        self.advance_bit_counter(n);
        let return_val = self.slice(0, n);
        return_val
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
        self.advance_bit_counter(1);
        return_val
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self.bytes.get(n) {
            Some(byte) => Some(*byte),
            None => None
        }
    }
}
