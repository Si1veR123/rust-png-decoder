
#[derive(PartialEq)]
pub enum Endian {
    BigEndian,
    LittleEndian
}

pub struct BitReader<'a> {
    data: &'a [u8],
    pub byte_position: usize,
    bit_mask: u8,
    endian: Endian
}

impl<'a> BitReader<'a> {
    /// `endian` determines the order that individual bits are read in a byte
    /// 
    /// `Endian::LittleEndian` will read the least significant bits of a byte first
    pub fn new(data: &'a [u8], endian: Endian) -> Self {
        let initial_mask = match endian {
            Endian::BigEndian => 0b1000_0000,
            Endian::LittleEndian => 0b0000_0001
        };
        Self { data: data, byte_position: 0, bit_mask: initial_mask, endian }
    }

    /// Skip to next byte, unless we are at the end of the last byte
    pub fn skip_to_next_byte(&mut self) {
        let start_of_byte = match self.endian {
            Endian::BigEndian => {
                self.bit_mask == 0b1000_0000
            },
            Endian::LittleEndian => {
                self.bit_mask == 0b0000_0001
            }
        };

        if !start_of_byte {
            self.next_byte();
        }
    }

    /// Skip to next byte, even if we are at the end of the last byte
    fn next_byte(&mut self) {
        self.byte_position += 1;
        self.bit_mask = match self.endian {
            Endian::BigEndian => 0b1000_0000,
            Endian::LittleEndian => 0b0000_0001
        };
    }

    /// Skips to next byte (if not at start), then fills `to` with the next bytes
    pub fn read_byte_slice(&mut self, to: &mut [u8]) -> Option<()> {
        self.skip_to_next_byte();

        let length = to.len();
        let start_index = self.byte_position;

        for byte_index in 0..length {
            let byte = *self.data.get(start_index + byte_index)?;
            to[byte_index] = byte;
        }

        self.byte_position += length;

        Some(())
    }

    pub fn skip_bit(&mut self) {
        match self.endian {
            Endian::BigEndian => {
                if self.bit_mask == 0b0000_0001 {
                    self.bit_mask = 0b1000_0000;
                    self.byte_position += 1;
                } else {
                    self.bit_mask = self.bit_mask >> 1;
                }
            },
            Endian::LittleEndian => {
                if self.bit_mask == 0b1000_0000 {
                    self.bit_mask = 0b0000_0001;
                    self.byte_position += 1;
                } else {
                    self.bit_mask = self.bit_mask << 1;
                }
            }
        }
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        let byte = *self.data.get(self.byte_position)?;
        let bit = ((byte & self.bit_mask) > 0) as u8;
        self.skip_bit();
        Some(bit)
    }

    /// Count must be 64 or less.
    /// Assumes that the first bit read is the MSB e.g.
    /// 
    /// ```
    /// use png_decoder::bitreader::{BitReader, Endian};
    /// 
    /// let mut bitreader = BitReader::new(&[0b1100_0101], Endian::BigEndian);
    /// assert_eq!(bitreader.read_bits(3), Some(6)); // 6 = 0b0000_0110
    /// ```
    pub fn read_bits(&mut self, count: usize) -> Option<u64> {
        let mut byte = 0;

        for _ in 0..count {
            byte = byte << 1;
            byte = byte | self.read_bit()? as u64;
        }

        Some(byte)
    }

    /// Count must be 64 or less.
    /// Assumes that the first bit read is the LSB e.g.
    /// ```
    /// use png_decoder::bitreader::{BitReader, Endian};
    /// 
    /// let mut bitreader = BitReader::new(&[0b1100_0101], Endian::BigEndian);
    /// assert_eq!(bitreader.read_bits_reverse(3), Some(3)); // 3 = 0b0000_0011
    /// ```
    pub fn read_bits_reverse(&mut self, count: usize) -> Option<u64> {
        let mut byte = 0;
        
        for i in 0..count {
            byte = byte | ((self.read_bit()? as u64) << i);
        }

        Some(byte)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        Some(self.read_bits(8)? as u8)
    }

    pub fn read_u8_reverse(&mut self) -> Option<u8> {
        Some(self.read_bits_reverse(8)? as u8)
    }
}
