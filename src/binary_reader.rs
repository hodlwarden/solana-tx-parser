//! Binary buffer reader for instruction data.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinaryReaderError {
    #[error("buffer overflow: tried to read {requested} bytes at offset {offset} in buffer of length {length}")]
    Overflow {
        requested: usize,
        offset: usize,
        length: usize,
    },
}

pub struct BinaryReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> BinaryReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn check_bounds(&self, length: usize) -> Result<(), BinaryReaderError> {
        if self.offset + length > self.data.len() {
            return Err(BinaryReaderError::Overflow {
                requested: length,
                offset: self.offset,
                length: self.data.len(),
            });
        }
        Ok(())
    }

    pub fn read_fixed_array(&mut self, length: usize) -> Result<&'a [u8], BinaryReaderError> {
        self.check_bounds(length)?;
        let slice = &self.data[self.offset..self.offset + length];
        self.offset += length;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8, BinaryReaderError> {
        self.check_bounds(1)?;
        let v = self.data[self.offset];
        self.offset += 1;
        Ok(v)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BinaryReaderError> {
        self.check_bounds(2)?;
        let v = u16::from_le_bytes(self.data[self.offset..self.offset + 2].try_into().unwrap());
        self.offset += 2;
        Ok(v)
    }

    pub fn read_u64_le(&mut self) -> Result<u64, BinaryReaderError> {
        self.check_bounds(8)?;
        let v = u64::from_le_bytes(self.data[self.offset..self.offset + 8].try_into().unwrap());
        self.offset += 8;
        Ok(v)
    }

    pub fn read_i64_le(&mut self) -> Result<i64, BinaryReaderError> {
        self.check_bounds(8)?;
        let v = i64::from_le_bytes(self.data[self.offset..self.offset + 8].try_into().unwrap());
        self.offset += 8;
        Ok(v)
    }

    pub fn read_string_u32_len(&mut self) -> Result<String, BinaryReaderError> {
        let len = self.read_u32_le()? as usize;
        self.check_bounds(len)?;
        let s = String::from_utf8_lossy(&self.data[self.offset..self.offset + len]).into_owned();
        self.offset += len;
        Ok(s)
    }

    pub fn read_pubkey(&mut self) -> Result<String, BinaryReaderError> {
        let slice = self.read_fixed_array(32)?;
        Ok(bs58::encode(slice).into_string())
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

// u32 read for string length
impl BinaryReader<'_> {
    pub fn read_u32_le(&mut self) -> Result<u32, BinaryReaderError> {
        self.check_bounds(4)?;
        let v = u32::from_le_bytes(self.data[self.offset..self.offset + 4].try_into().unwrap());
        self.offset += 4;
        Ok(v)
    }
}
