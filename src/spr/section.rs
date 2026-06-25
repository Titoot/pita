use crate::error::PitaError;
use crate::spr::pair::SprPair;
use crate::PitaResult;

#[derive(Debug)]
pub struct SprSections {
    pub blobs: Vec<Vec<u8>>,
}

impl SprSections {
    pub fn get(&self, i: usize) -> &[u8] {
        if i < self.blobs.len() {
            &self.blobs[i]
        } else {
            &[]
        }
    }

    pub fn len(&self, i: usize) -> usize {
        self.blobs.get(i).map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self, i: usize) -> bool {
        self.len(i) == 0
    }
}

pub fn extract_sections(data: &[u8], pairs: &[SprPair; 14]) -> PitaResult<SprSections> {
    let file_size = data.len() as u64;

    for pair in pairs.iter() {
        if !pair.is_empty() {
            if pair.start > file_size || pair.end > file_size {
                return Err(PitaError::SprSectionOverlap);
            }
            if pair.start >= pair.end {
                return Err(PitaError::SprSectionOverlap);
            }
        }
    }

    // Check no two non-empty sections overlap
    for i in 0..14 {
        for j in (i + 1)..14 {
            if pairs[i].is_empty() || pairs[j].is_empty() {
                continue;
            }
            let a_start = pairs[i].start;
            let a_end = pairs[i].end;
            let b_start = pairs[j].start;
            let b_end = pairs[j].end;

            if a_start < b_end && b_start < a_end {
                return Err(PitaError::SprSectionOverlap);
            }
        }
    }

    let mut blobs = Vec::with_capacity(14);
    for pair in pairs.iter() {
        if pair.is_empty() {
            blobs.push(Vec::new());
        } else {
            let start = pair.start as usize;
            let end = pair.end as usize;
            blobs.push(data[start..end].to_vec());
        }
    }

    Ok(SprSections { blobs })
}
