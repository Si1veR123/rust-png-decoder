use std::iter::repeat;

use crate::huffman_coding::HuffmanTree;
use crate::bitreader::{BitReader, Endian};

const CLEN_CODELENGTHS_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

enum LiteralLengthSymbol {
    Literal(u8),
    EndOfBlock,
    Length(u16)
}

#[derive(Debug)]
pub enum DeflateError {
    NoFinalBlock,
    NoBType,
    InvalidBType,
    ReservedBType,
    InvalidUncompressedBlock(usize),
    InvalidFixedBlock(usize),
    InvalidDynamicBlock(usize)
}

const fn value_from_extra_bits(code_index: usize, first_value: usize, extra_bit_count: usize, extra_bits: u16) -> u16 {
    let max_range = 1usize << extra_bit_count;
    let start_of_range = first_value + (code_index * max_range);
    (start_of_range as u16) + extra_bits
}

pub struct DeflateDecompression<'a> {
    bits: BitReader<'a>,
    decompressed_buffer: Vec<u8>
}

impl<'a> DeflateDecompression<'a> {
    pub fn new(deflate_bytes: &'a [u8]) -> Self {
        Self { bits: BitReader::new(deflate_bytes, Endian::LittleEndian), decompressed_buffer: Vec::new() }
    }

    fn decompress_next_block(&mut self) -> Result<bool, DeflateError> {
        let bfinal = self.bits.read_bit().ok_or(DeflateError::NoFinalBlock)?;
    
        let btype = [
            self.bits.read_bit().ok_or(DeflateError::NoBType)?,
            self.bits.read_bit().ok_or(DeflateError::NoBType)?
        ];

        match btype {
            [0, 0] => self.deflate_uncompressed_block()?,
            [1, 0] => self.deflate_fixed_huffman_block()?,
            [0, 1] => self.deflate_dynamic_huffman_block()?,
            [1, 1] => return Err(DeflateError::ReservedBType),
            _ => return Err(DeflateError::InvalidBType)
        };

        Ok(bfinal > 0)
    }

    fn deflate_uncompressed_block(&mut self) -> Result<(), DeflateError> {
        self.bits.skip_to_next_byte();

        let block_byte_index = (self.bits.byte_position / 8) as usize;
        let block_length = self.bits.read_bits_reverse(16)
            .ok_or(DeflateError::InvalidUncompressedBlock(block_byte_index))?
            as u16;
        
        let block_length_check = self.bits.read_bits_reverse(16)
            .ok_or(DeflateError::InvalidUncompressedBlock(block_byte_index))?
            as u16;

        if block_length != (!block_length_check) {
            return Err(DeflateError::InvalidUncompressedBlock(block_byte_index))
        }

        let data_start_index = self.decompressed_buffer.len();
        self.decompressed_buffer.extend(repeat(0).take(block_length as usize));
        let data_destination = &mut self.decompressed_buffer[data_start_index..];

        self.bits.read_byte_slice(data_destination).ok_or(DeflateError::InvalidUncompressedBlock(block_byte_index))?;
        Ok(())
    }

    fn process_lit_len_symbol(&mut self, symbol: u16) -> Option<LiteralLengthSymbol> {
        // in this match statement, all other symbols except length symbols return from the function
        let length = match symbol {
            0..=255 => {
                return Some(LiteralLengthSymbol::Literal(symbol as u8))
            },
            256 => {
                return Some(LiteralLengthSymbol::EndOfBlock)
            },
            257..=264 => {
                // no extra bits
                symbol - 254
            },
            265..=268 => {
                // 1 extra bit
                let extra = self.bits.read_bit()?;
                let length = value_from_extra_bits((symbol - 265) as usize, 11, 1, extra as u16);
                length
            },
            269..=272 => {
                // 2 extra bits
                let extra = self.bits.read_bits_reverse(2)?;
                value_from_extra_bits((symbol - 269) as usize, 19, 2, extra as u16)
            },
            273..=276 => {
                // 3 extra bits
                let extra = self.bits.read_bits_reverse(3)?;
                value_from_extra_bits((symbol - 273) as usize, 35, 3, extra as u16)
            },
            277..=280 => {
                // 4 extra bits
                let extra = self.bits.read_bits_reverse(4)?;
                value_from_extra_bits((symbol - 277) as usize, 67, 4, extra as u16)
            },
            281..=284 => {
                // 5 extra bits
                let extra = self.bits.read_bits_reverse(5)?;
                value_from_extra_bits((symbol - 281) as usize, 131, 5, extra as u16)
            },
            285 => {
                258
            },
            _ => return None
        };

        Some(LiteralLengthSymbol::Length(length))
    }

    fn process_distance_symbol(&mut self, symbol: u16) -> Option<u16> {
        match symbol {
            0..=3 => {
                Some(symbol + 1)
            },
            4..=5 => {
                // 1 extra bit
                let extra = self.bits.read_bit()?;
                let distance = value_from_extra_bits((symbol-4) as usize, 5, 1, extra as u16);
                Some(distance)
            },
            6..=7 => {
                // 2 extra bits
                let extra = self.bits.read_bits_reverse(2)?;
                let distance = value_from_extra_bits((symbol-6) as usize, 9, 2, extra as u16);
                Some(distance)
            },
            8..=9 => {
                // 3 extra bits
                let extra = self.bits.read_bits_reverse(3)?;
                let distance = value_from_extra_bits((symbol-8) as usize, 17, 3, extra as u16);
                Some(distance)
            },
            10..=11 => {
                // 4 extra bits
                let extra = self.bits.read_bits_reverse(4)?;
                let distance = value_from_extra_bits((symbol-10) as usize, 33, 4, extra as u16);
                Some(distance)
            },
            12..=13 => {
                // 5 extra bits
                let extra = self.bits.read_bits_reverse(5)?;
                let distance = value_from_extra_bits((symbol-12) as usize, 65, 5, extra as u16);
                Some(distance)
            },
            14..=15 => {
                // 6 extra bits
                let extra = self.bits.read_bits_reverse(6)?;
                let distance = value_from_extra_bits((symbol-14) as usize, 129, 6, extra as u16);
                Some(distance)
            },
            16..=17 => {
                // 7 extra bits
                let extra = self.bits.read_bits_reverse(7)?;
                let distance = value_from_extra_bits((symbol-16) as usize, 257, 7, extra as u16);
                Some(distance)
            },
            18..=19 => {
                // 8 extra bits
                let extra = self.bits.read_bits_reverse(8)?;
                let distance = value_from_extra_bits((symbol-18) as usize, 513, 8, extra as u16);
                Some(distance)
            },
            20..=21 => {
                // 9 extra bits
                let extra = self.bits.read_bits_reverse(9)?;
                let distance = value_from_extra_bits((symbol-20) as usize, 1025, 9, extra as u16);
                Some(distance)
            },
            22..=23 => {
                // 10 extra bits
                let extra = self.bits.read_bits_reverse(10)?;
                let distance = value_from_extra_bits((symbol-22) as usize, 2049, 10, extra as u16);
                Some(distance)
            },
            24..=25 => {
                // 11 extra bits
                let extra = self.bits.read_bits_reverse(11)?;
                let distance = value_from_extra_bits((symbol-24) as usize, 4097, 11, extra as u16);
                Some(distance)
            },
            26..=27 => {
                // 12 extra bits
                let extra = self.bits.read_bits_reverse(12)?;
                let distance = value_from_extra_bits((symbol-26) as usize, 8193, 12, extra as u16);
                Some(distance)
            },
            28..=29 => {
                // 13 extra bits
                let extra = self.bits.read_bits_reverse(13)?;
                let distance = value_from_extra_bits((symbol-28) as usize, 16385, 13, extra as u16);
                Some(distance)
            },
            _ => return None
        }
    }

    fn fill_length_distance_data(&mut self, length: usize, distance: usize) {
        let (full_copies, extra_value_copies) = (length / distance, length % distance);
        let start_range = self.decompressed_buffer.len()-distance;
        for _ in 0..full_copies {
            self.decompressed_buffer.extend_from_within(start_range..start_range+distance);
        }
        self.decompressed_buffer.extend_from_within(start_range..start_range+extra_value_copies);
    }

    fn next_fixed_compressed_symbol(&mut self) -> Option<bool> {
        let mut bits = self.bits.read_bits(7)? as u16;
        let symbol = if bits <= 23 {
            bits + 256
        } else {
            bits = bits << 1 | self.bits.read_bit()? as u16;
            if bits >= 48 && bits <= 191 {
                bits - 48
            } else if bits >= 192 && bits <= 199 {
                bits + 88
            } else {
                bits = bits << 1 | self.bits.read_bit()? as u16;
                if bits >= 400 && bits <= 511 {
                    bits - 256
                } else {
                    return None
                }
            }
        };

        let symbol_type = self.process_lit_len_symbol(symbol)?;
        match symbol_type {
            LiteralLengthSymbol::EndOfBlock => return Some(true),
            LiteralLengthSymbol::Literal(lit) => self.decompressed_buffer.push(lit),
            LiteralLengthSymbol::Length(length) => {
                let distance_symbol = self.bits.read_bits_reverse(5)?;
                if distance_symbol > 29 {
                    return None
                }

                let distance = self.process_distance_symbol(distance_symbol as u16)?;
                self.fill_length_distance_data(length as usize, distance as usize);
            }
        };

        Some(false)
    }

    fn deflate_fixed_huffman_block(&mut self) -> Result<(), DeflateError> {
        let block_byte_index = self.bits.byte_position;

        while !self.next_fixed_compressed_symbol()
            .ok_or(DeflateError::InvalidFixedBlock(block_byte_index))? {}
        Ok(())
    }

    fn codelength_alphabet_codelengths(&mut self, hclen: usize) -> Option<[u8; 19]> {
        let mut clen_codelengths = [0; 19];
        for i in 0..hclen {
            let next_clen = self.bits.read_bits_reverse(3)? as u8;
            clen_codelengths[CLEN_CODELENGTHS_ORDER[i]] = next_clen;
        }
        Some(clen_codelengths)
    }

    /// Parse the codelengths of the dynamic huffman (literal and length/distance) alphabets
    fn dynamic_alphabet_codelengths<const N: usize>(&mut self, huffman_tree: &HuffmanTree, clen_count: usize) -> Option<[u8; N]> {
        let mut codelengths = [0; N];
        let mut codelengths_parsed = 0;

        while codelengths_parsed < clen_count {
            // symbols will be 18 or less (u8) for the codelength alphabet, or incorrect
            let symbol = huffman_tree.get_next_symbol(&mut self.bits)?;

            match symbol {
                0..=15 => {
                    codelengths[codelengths_parsed] = symbol as u8;
                    codelengths_parsed += 1;
                },
                16 => {
                    let repeat_length = (self.bits.read_bits_reverse(2)? + 3) as usize;
                    let previous = *codelengths
                        .get(codelengths_parsed-1)?;
                    let repeat_slice = &mut codelengths[codelengths_parsed..codelengths_parsed + repeat_length];

                    repeat_slice.fill(previous);
                    codelengths_parsed += repeat_length;
                },
                17 => {
                    let zero_length = self.bits.read_bits_reverse(3)? + 3;
                    codelengths_parsed += zero_length as usize;
                },
                18 => {
                    let zero_length = self.bits.read_bits_reverse(7)? + 11;
                    codelengths_parsed += zero_length as usize;
                },
                _ => return None
            }
        }

        Some(codelengths)
    }

    fn next_dynamic_compressed_symbol(&mut self, lit_len_codes: &HuffmanTree, distance_codes: &HuffmanTree) -> Option<bool> {
        let lit_len_symbol = lit_len_codes.get_next_symbol(&mut self.bits)?;
        let symbol = self.process_lit_len_symbol(lit_len_symbol)?;

        match symbol {
            LiteralLengthSymbol::EndOfBlock => return Some(true),
            LiteralLengthSymbol::Literal(lit) => self.decompressed_buffer.push(lit),
            LiteralLengthSymbol::Length(length) => {
                let distance_symbol = distance_codes.get_next_symbol(&mut self.bits)?;
                let distance = self.process_distance_symbol(distance_symbol)?;
                self.fill_length_distance_data(length as usize, distance as usize);
            }
        };

        Some(false)
    }

    fn deflate_dynamic_huffman_block(&mut self) -> Result<(), DeflateError> {
        let block_byte_index = self.bits.byte_position;

        let hlit_value = self.bits.read_bits_reverse(5).ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;
        let hlit = hlit_value as usize + 257;

        let hdist_value = self.bits.read_bits_reverse(5).ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;
        let hdist = hdist_value as usize + 1;

        let hclen_value = self.bits.read_bits_reverse(4).ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;
        let hclen = hclen_value as usize + 4;
        let clen_codelengths = self.codelength_alphabet_codelengths(hclen)
            .ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;

        let clen_alphabet_huffman_tree = unsafe { HuffmanTree::from_codelengths(&clen_codelengths).unwrap_unchecked() };

        let lit_len_codelengths: [u8; 286] = self.dynamic_alphabet_codelengths(&clen_alphabet_huffman_tree, hlit)
            .ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;
        let lit_len_huffman_tree = unsafe { HuffmanTree::from_codelengths(&lit_len_codelengths).unwrap_unchecked() };

        let distance_codelengths: [u8; 32] = self.dynamic_alphabet_codelengths(&clen_alphabet_huffman_tree, hdist)
            .ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))?;
        let distance_huffman_tree = unsafe { HuffmanTree::from_codelengths(&distance_codelengths).unwrap_unchecked() };

        while !self.next_dynamic_compressed_symbol(&lit_len_huffman_tree, &distance_huffman_tree)
            .ok_or(DeflateError::InvalidDynamicBlock(block_byte_index))? {}

        Ok(())
    }

    pub fn decompress(mut self) -> Result<Vec<u8>, DeflateError> {
        while !self.decompress_next_block()? {}
        Ok(self.decompressed_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_clen_alphabet() {
        let data = [0b00101101, 0b10100111, 0b10100001, 0b00011000, 0b00000110, 0b10100010];
        let mut deflate = DeflateDecompression::new(&data);
        let hclen = 16;
        let clen_alphabet_codelengths = deflate.codelength_alphabet_codelengths(hclen).unwrap();
        assert_eq!(clen_alphabet_codelengths, [3, 0, 5, 4, 3, 3, 5, 3, 2, 0, 0, 0, 0, 0, 0, 0, 5, 5, 4])
    }

    #[test]
    fn extra_bits_test() {
        assert_eq!(value_from_extra_bits(2, 131, 5, 0), 195);
        assert_eq!(value_from_extra_bits(1, 5, 1, 1), 8);
        assert_eq!(value_from_extra_bits(0, 24577, 13, 100), 24677);
        assert_eq!(value_from_extra_bits(4, 3, 0, 0), 7);
    }

    #[test]
    fn test_dynamic_huffman() {
        let deflate = DeflateDecompression::new(&[29, 198, 73, 1, 0, 0, 16, 64, 192, 172, 163, 127, 136, 61, 60, 32, 42, 151, 157, 55, 94, 29, 12]);
        let start = std::time::Instant::now();
        let decompressed = deflate.decompress().unwrap();
        let end = std::time::Instant::now();
        println!("{:?}", end-start);
        assert_eq!(decompressed, vec![97, 98, 97, 97, 98, 98, 98, 97, 98, 97, 97, 98, 97, 98, 98, 97, 97, 98, 97, 98, 97, 97, 97, 97, 98, 97, 97, 97, 98, 98, 98, 98, 98, 97, 97]);
    }
}
