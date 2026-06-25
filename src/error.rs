use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PitaError {
    None = 0,
    InvalidArg = -1,
    BadMagic = -2,
    Truncated = -3,
    DecompressFailed = -4,
    BadFooter = -5,
    DeswizzleOverflow = -6,
    SprHeaderInvalid = -7,
    SprSectionOverlap = -8,
    Internal = -99,
}

impl PitaError {
    pub fn c_str(&self) -> &'static str {
        match self {
            PitaError::None => "Success\0",
            PitaError::InvalidArg => "Invalid argument\0",
            PitaError::BadMagic => "Bad magic bytes (not zstd)\0",
            PitaError::Truncated => "Truncated file\0",
            PitaError::DecompressFailed => "Zstd decompression failed\0",
            PitaError::BadFooter => "Bad footer\0",
            PitaError::DeswizzleOverflow => "Deswizzle offset overflow\0",
            PitaError::SprHeaderInvalid => "Invalid SPR header\0",
            PitaError::SprSectionOverlap => "SPR sections overlap\0",
            PitaError::Internal => "Internal error\0",
        }
    }
}

impl fmt::Display for PitaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.c_str();
        write!(f, "{}", &s[..s.len() - 1])
    }
}

impl std::error::Error for PitaError {}
