use crate::error::PitaError;
use crate::PitaResult;

// Known header field indices
pub(crate) const IDX_FLAGS: usize = 0;
pub(crate) const IDX_CNT: usize = 1;
pub(crate) const IDX_FIELD5: usize = 5;
pub(crate) const IDX_FIELD7: usize = 7;
pub(crate) const IDX_FIELD9: usize = 9;
pub(crate) const IDX_FIELD12: usize = 12;

#[derive(Debug, Clone, Copy)]
pub struct SprHeader {
    pub raw: [u32; 14],
}

impl SprHeader {
    pub fn parse(data: &[u8]) -> PitaResult<Self> {
        if data.len() < 56 {
            return Err(PitaError::Truncated);
        }

        let mut raw = [0u32; 14];
        for (i, val) in raw.iter_mut().enumerate() {
            let off = i * 4;
            *val = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        }

        Ok(SprHeader { raw })
    }

    pub fn flags(&self) -> u32 { self.raw[IDX_FLAGS] }
    pub fn cnt(&self) -> u32 { self.raw[IDX_CNT] }
    pub fn h14(&self) -> u32 { self.raw[IDX_FIELD5] }
    pub fn h1c(&self) -> u32 { self.raw[IDX_FIELD7] }
    pub fn h24(&self) -> u32 { self.raw[IDX_FIELD9] }
    pub fn h30(&self) -> u32 { self.raw[IDX_FIELD12] }
}
