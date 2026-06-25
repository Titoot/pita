use crate::spr::header::SprHeader;
use crate::spr::pair::{parse_pairs, SprPair};
use crate::spr::section::{extract_sections, SprSections};
use crate::PitaResult;

#[derive(Debug)]
pub struct SprDecoded {
    pub header: SprHeader,
    pub pairs: [SprPair; 14],
    pub sections: SprSections,
}

pub fn decode(data: &[u8]) -> PitaResult<SprDecoded> {
    let header = SprHeader::parse(data)?;
    let pairs = parse_pairs(data)?;
    let sections = extract_sections(data, &pairs)?;

    Ok(SprDecoded {
        header,
        pairs,
        sections,
    })
}
