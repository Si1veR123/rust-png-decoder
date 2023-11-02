use crate::deflate::{DeflateDecompression, DeflateError};

#[derive(Debug)]
pub enum ZlibError {
    NoBytes,
    NoFlg,
    FCheck,
    InvalidFDict,
    Deflate(DeflateError),
    AdlerCheck
}

impl From<DeflateError> for ZlibError {
    fn from(value: DeflateError) -> Self {
        Self::Deflate(value)
    }
}

pub fn adler_32(bytes: impl AsRef<[u8]>) -> u32 {
    let mut a = 1;
    let mut b = 0;

    for byte in bytes.as_ref() {
        a = (a + (*byte as u32)) % 65521;
        b = (b + a) % 65521;
    }

    ((b as u32) << 16) | (a as u32)
}

pub struct ZlibDecompressed {
    pub cm: u8,
    pub cinfo: u8,
    pub dictid: Option<[u8; 4]>,
    pub flevel: u8,
    pub data: Vec<u8>,
    pub adler32: u32,
}

pub struct ZlibParser<'a> {
    bytes: &'a [u8]
}

impl<'a> ZlibParser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn parse_cmf_flg(&mut self) -> Result<(u8, u8), ZlibError> {
        let cmf = *self.bytes.get(0).ok_or(ZlibError::NoBytes)?;
        let flg = *self.bytes.get(1).ok_or(ZlibError::NoFlg)?;

        if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
            return Err(ZlibError::FCheck)
        }

        Ok((cmf, flg))
    }

    /// Panic if less than 4 bytes in self.bytes
    fn parse_adler(&mut self) -> u32 {
        let adler_bytes: [u8; 4] = self.bytes[self.bytes.len()-4..self.bytes.len()].try_into().unwrap();

        let adler32 = (adler_bytes[0] as u32) << 24 |
                         (adler_bytes[1] as u32) << 16 |
                         (adler_bytes[2] as u32) << 8 |
                         adler_bytes[3] as u32;
        
        adler32
    }

    pub fn parse(&mut self) -> Result<ZlibDecompressed, ZlibError> {
        let (cmf, flg) = self.parse_cmf_flg()?;

        let fdict = (flg & 0b0010_0000) >> 5;

        let dictid: Option<[u8; 4]>;
        let data_slice: &[u8];

        match fdict {
            0 => {
                dictid = None;
                data_slice = &self.bytes[2..self.bytes.len()-4];
            },
            1 => {
                dictid = Some(
                    self.bytes[2..6]
                        .try_into()
                        .map_err(|_| ZlibError::InvalidFDict)?
                    );
                
                data_slice = &self.bytes[6..self.bytes.len()-4];
            },
            _ => return Err(ZlibError::InvalidFDict)
        }
        
        let decompressed = DeflateDecompression::new(data_slice).decompress()?;
        let adler32 = self.parse_adler();

        if adler_32(&decompressed) != adler32 {
            return Err(ZlibError::AdlerCheck)
        }
        
        Ok(ZlibDecompressed {
            cm: cmf & 0b0000_1111,
            cinfo: (cmf & 0b1111_0000) >> 4,
            dictid,
            flevel: (flg & 0b1100_0000) >> 6,
            data: decompressed,
            adler32
        })
    }
}
