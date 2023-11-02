use rustc_hash::FxHashMap;

use crate::bitreader::BitReader;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Prefix {
    pub length: u8,
    pub prefix: u16
}

pub struct HuffmanTree {
    pub symbol_prefixes: FxHashMap<Prefix, u16>,
    min_codelength: u8,
    max_codelength: u8
}

impl HuffmanTree {
    /// `codelengths` is a slice of codelengths where the nth codelength represents the symbol n.
    /// 
    /// Returns None if `codelengths` is empty
    pub fn from_codelengths(codelengths: &[u8]) -> Option<Self> {
        let max_clen = *codelengths.iter().max()?;

        // Count number of occurences of codelengths, and find min codelength
        let mut min_clen = u8::MAX;
        let mut clen_counts = vec![0usize; (max_clen as usize)+1];
        for &symbol_clen in codelengths.iter().filter(|&&x| x > 0) {
            min_clen = min_clen.min(symbol_clen);
            clen_counts[symbol_clen as usize] += 1;
        }

        // Create the base code for each codelength
        let mut code = 0;
        for codelength in 1..(max_clen as usize) {
            code = (code + clen_counts[codelength]) << 1;
            // reuse vec for base codes
            clen_counts[codelength] = code;
        }
        let mut next_codes = clen_counts;
        
        // Create a mapping of prefixes to symbols
        let mut symbol_prefixes = FxHashMap::default();

        for (symbol, &symbol_clen) in codelengths.iter().enumerate().filter(|(_, &x)| x > 0) {
            let next_code = next_codes.get_mut(symbol_clen as usize - 1).unwrap();
            symbol_prefixes.insert(Prefix { length: symbol_clen, prefix: *next_code as u16}, symbol as u16);
            *next_code += 1;
        }

        Some(Self {
            symbol_prefixes,
            max_codelength: max_clen,
            min_codelength: min_clen
        })
    }

    /// If the huffman tree contains codelengths longer than 16 bits, returns None
    pub fn get_next_symbol(&self, data: &mut BitReader) -> Option<u16> {
        if self.max_codelength > 16 {
            return None
        }
        
        let mut current_prefix = data.read_bits(self.min_codelength as usize)? as u16;
        let mut current_length = self.min_codelength;

        for _ in self.min_codelength..=self.max_codelength {
            if let Some(symbol) = self.get_symbol(&Prefix { length: current_length, prefix: current_prefix }) {
                return Some(symbol)
            }

            current_prefix = (current_prefix << 1) | data.read_bits(1)? as u16;
            current_length += 1;
        }

        None
    }

    pub fn get_symbol(&self, code: &Prefix) -> Option<u16> {
        self.symbol_prefixes.get(code).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_codelengths_test() {
        let codelengths = vec![0, 4, 1, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 2];
        let tree = HuffmanTree::from_codelengths(&codelengths).unwrap();
        
        let correct_symbol_prefixes = FxHashMap::from_iter([
            (Prefix { length: 4, prefix: 13 }, 4),
            (Prefix { length: 4, prefix: 14 }, 16),
            (Prefix { length: 1, prefix: 0 }, 2),
            (Prefix { length: 4, prefix: 15 }, 17),
            (Prefix { length: 2, prefix: 2 }, 18),
            (Prefix { length: 4, prefix: 12 }, 1)
        ]);

        assert_eq!(correct_symbol_prefixes, tree.symbol_prefixes);
    }
}
