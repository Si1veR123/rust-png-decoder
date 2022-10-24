fn base_codes_for_lengths(codelengths: &Vec<u8>) -> Vec<u16> {
    let &max_code_length = codelengths.iter().max().unwrap();
    let mut base_code: Vec<u16> = Vec::with_capacity((max_code_length+1) as usize);
    base_code.push(0);

    // u16 as may have over 8 bit prefixes, wont have 16 bit prefixes
    let mut code: u16 = 0;
    for bits in 1..=max_code_length {
        let prev_occurences = codelengths
            .iter()
            .cloned()
            .filter(|&x| x == (bits-1))
            .collect::<Vec<u8>>()
            .len() as u16;
        code = (code + prev_occurences) << 1;
        base_code.push(code);
    }
    base_code
}

pub fn prefix_codes_from_codelengths(codelengths: Vec<u8>) -> Vec<u16> {
    let mut base_codes = base_codes_for_lengths(&codelengths);

    let mut prefix_codes: Vec<u16> = Vec::with_capacity(codelengths.len());
    for clen in codelengths {
        let clen_u = clen as usize;
        if clen_u > 0 {
            prefix_codes.push(base_codes[clen_u]);
            base_codes[clen_u] += 1;
        }
    }
    prefix_codes
}