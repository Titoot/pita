use crate::error::PitaError;
use crate::PitaResult;

#[derive(Debug, Clone, Copy)]
pub struct TexFooter {
    pub mip_code: u16,
    pub unknown: u16,
    pub width: u16,
    pub height: u16,
    pub format: u16,
}

impl TexFooter {
    pub fn mip_count(&self) -> u32 {
        match self.mip_code {
            0x63 => 2,
            0x64 => 3,
            0x65 => 4,
            _ => 1,
        }
    }
}

pub fn read_footer(decompressed: &[u8]) -> PitaResult<TexFooter> {
    let file_size = decompressed.len();
    if file_size < 0x1000 {
        return Err(PitaError::Truncated);
    }

    let fb = file_size - 0x1000;

    if fb + 10 > file_size {
        return Err(PitaError::Truncated);
    }

    let mip_code = u16::from_le_bytes([decompressed[fb], decompressed[fb + 1]]);
    let unknown = u16::from_le_bytes([decompressed[fb + 2], decompressed[fb + 3]]);
    let width = u16::from_le_bytes([decompressed[fb + 4], decompressed[fb + 5]]);
    let height = u16::from_le_bytes([decompressed[fb + 6], decompressed[fb + 7]]);
    let fmt = u16::from_le_bytes([decompressed[fb + 8], decompressed[fb + 9]]);

    if width < 4 || height < 4 || width > 8192 || height > 8192 {
        return Err(PitaError::BadFooter);
    }

    let valid_mip_codes = [0u16, 0x63, 0x64, 0x65];
    if !valid_mip_codes.contains(&mip_code) {
        return Err(PitaError::BadFooter);
    }

    Ok(TexFooter {
        mip_code,
        unknown,
        width,
        height,
        format: fmt,
    })
}
