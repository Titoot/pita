use crate::swizzle;

pub fn get_spr_gby(spr_data: &[u8]) -> Option<u32> {
    if spr_data.len() < 0x40 {
        return None;
    }
    let cnt = u32::from_le_bytes(spr_data[4..8].try_into().ok()?);
    let tbl = u32::from_le_bytes(spr_data[0x38..0x3c].try_into().ok()?);
    if cnt == 1 {
        let extra_off = (tbl as usize) + 32;
        if extra_off + 208 <= spr_data.len() {
            let v = u32::from_le_bytes(
                spr_data[extra_off + 204..extra_off + 208].try_into().ok()?,
            );
            if [1u32, 2, 4, 8, 16].contains(&v) {
                return Some(v);
            }
        }
    }
    None
}

pub fn resolve_gby(wb: u32, hb: u32, companion_spr: Option<&[u8]>) -> u32 {
    if let Some(spr) = companion_spr {
        if let Some(gby) = get_spr_gby(spr) {
            return gby;
        }
    }
    swizzle::detect_gob_blocks_y(wb, hb)
}
