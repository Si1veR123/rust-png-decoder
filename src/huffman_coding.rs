fn base_codes_for_lengths(codelengths: &Vec<u8>) -> Vec<u16> {
    let &max_code_length = codelengths.iter().max().unwrap();
    let mut next_code: Vec<u16> = Vec::with_capacity(max_code_length as usize);

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
        next_code.push(code);
    }
    next_code
}

pub fn prefix_codes_from_codelengths(codelengths: Vec<u8>) {
    let mut next_code = base_codes_for_lengths(&codelengths);
}