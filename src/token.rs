use std::fmt::Debug;



pub struct Token {
    pub bits: Vec<u8>,
    pub using_bytes: bool, // if the 'bits' field actually stores byte values instead
    pub nest_level: u8, // 0 is most nested
    pub data: String,
    pub token_type: String,
    pub description: String,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // json for token
        write!(f, "{{\"bits\": {:?}, \"using_bytes\": {}, \"nest_level\": {}, \"data\": \"{}\", \"token_type\": \"{}\", \"description\": \"{}\"}}", self.bits, self.using_bytes, self.nest_level, self.data, self.token_type, self.description)
    }
}

pub fn literal_token(literal: u8, bits: Option<Vec<u8>>, nest_level: u8) -> Token {
    // under 32 can cause problematic json
    let data;
    if literal > 31 {
        if literal == 92 {
            // escape \
            data = (literal.to_string()) + r": \\ ";
        } else if literal == 34 {
            // escape "
            data = (literal.to_string()) + ": " + r"\" + "\"";
        } else {
            data = (literal.to_string()) + ": " + &(literal as char).to_string();
        }
    } else {
        data = literal.to_string();
    }

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
