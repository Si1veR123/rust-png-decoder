
#[derive(Debug)]
pub struct Token {
    pub bits: Vec<u8>,
    pub using_bytes: bool, // if the 'bits' field actually stores byte values instead
    pub nest_level: u8, // 0 is most nested
    pub data: String,
    pub token_type: String,
    pub description: String,
}


pub fn literal_token(literal: u8, bits: Option<Vec<u8>>, nest_level: u8) -> Token {
    let data = (literal.to_string()) + ": " + &(literal as char).to_string();

    if bits.is_none() {
        return Token {
            bits: vec![literal],
            using_bytes: true,
            nest_level,
            data,
            token_type: "literal".to_string(),
            description: "literal 0-255 value".to_string()
        }
    }

    Token {
        bits: bits.unwrap(),
        using_bytes: false,
        nest_level,
        data,
        token_type: "literal".to_string(),
        description: "literal 0-255 value".to_string()
    }
}

pub fn reference_token(bits: Vec<u8>, distance: u16, length: u16, nest_level: u8) -> Token {
    Token {
        bits,
        using_bytes: false,
        nest_level,
        data: format!("<{}, {}>", length, distance),
        token_type: "string reference".to_string(),
        description: "Duplicates a string from the stream".to_string(),
    }
}
