use crate::error::PitaError;
use crate::PitaResult;

#[derive(Debug, Clone, Copy)]
pub struct SprPair {
    pub start: u64,
    pub end: u64,
}

impl SprPair {
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

pub fn parse_pairs(data: &[u8]) -> PitaResult<[SprPair; 14]> {
    let pairs_offset = 0x38usize;
    let pairs_size = 14 * 16;

    if data.len() < pairs_offset + pairs_size {
        return Err(PitaError::Truncated);
    }

    let mut pairs = [SprPair {
        start: 0,
        end: 0,
    }; 14];

    for (i, pair) in pairs.iter_mut().enumerate() {
        let off = pairs_offset + i * 16;
        let start = u64::from_le_bytes([
            data[off],
            data[off + 1],
            data[off + 2],
            data[off + 3],
            data[off + 4],
            data[off + 5],
            data[off + 6],
            data[off + 7],
        ]);
        let end = u64::from_le_bytes([
            data[off + 8],
            data[off + 9],
            data[off + 10],
            data[off + 11],
            data[off + 12],
            data[off + 13],
            data[off + 14],
            data[off + 15],
        ]);
        *pair = SprPair { start, end };
    }

    let file_size = data.len() as u64;

    // Section 13 end is unreliable in some files; clamp to file_size if needed
    if pairs[13].end > file_size {
        pairs[13].end = file_size;
    }

    Ok(pairs)
}
