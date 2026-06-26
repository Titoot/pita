use crate::error::PitaError;

const BPB_DEFAULT: u32 = 16;
const BLK_PX: u32 = 4;
const GOB_HEIGHT: u32 = 8;

pub fn detect_gob_blocks_y(_wb: u32, hb: u32) -> u32 {
    // https://github.com/ScanMountGoat/tegra_swizzle/blob/main/src/blockheight.rs
    match hb + (hb / 2) {
        h if h >= 128 => 16,
        h if h >= 64 => 8,
        h if h >= 32 => 4,
        h if h >= 16 => 2,
        _ => 1,
    }
}

pub fn compute_width_in_gobs(wb: u32, bpb: u32) -> u32 {
    (wb * bpb).div_ceil(64)
}

fn block_offset(
    bx: u32, by: u32, _wb: u32, gob_blocks_y: u32,
    width_in_gobs: u32, bpb: u32,
) -> u64 {
    debug_assert!(gob_blocks_y > 0, "gob_blocks_y must be > 0");

    let xb = bx * bpb;
    let yh = by / GOB_HEIGHT;
    let bh_shift = gob_blocks_y.trailing_zeros();
    let bh_mask = gob_blocks_y - 1;
    let x_shift = 9 + bh_shift;
    let rob_size = 512 * gob_blocks_y * width_in_gobs;

    (yh >> bh_shift) as u64 * rob_size as u64
        + (xb / 64) as u64 * (1u64 << x_shift)
        + (yh & bh_mask) as u64 * 512
        + ((xb & 0x3f) >> 5) as u64 * 256
        + ((by & 7) >> 1) as u64 * 64
        + ((xb & 0x1f) >> 4) as u64 * 32
        + (by & 1) as u64 * 16
        + (xb & 0x0f) as u64
}

pub fn deswizzle_block_linear(
    data: &[u8],
    width: u32,
    height: u32,
    bpb: u32,
    gob_blocks_y: u32,
    width_in_gobs: u32,
) -> Result<Vec<u8>, PitaError> {
    let bpb = if bpb == 0 { BPB_DEFAULT } else { bpb };
    let wb = width.div_ceil(BLK_PX);
    let hb = height.div_ceil(BLK_PX);
    let groups = hb.div_ceil(gob_blocks_y * GOB_HEIGHT);
    let mip0_gobs = groups as u64 * gob_blocks_y as u64 * width_in_gobs as u64;
    let limit = data.len().min((mip0_gobs * 512) as usize);

    let out_size = (wb as u64 * hb as u64 * bpb as u64) as usize;
    let mut out = vec![0u8; out_size];

    for by in 0..hb {
        for bx in 0..wb {
            let off = block_offset(bx, by, wb, gob_blocks_y, width_in_gobs, bpb);
            let didx = (by as u64 * wb as u64 + bx as u64) * bpb as u64;
            if off as usize + bpb as usize <= limit {
                let src = &data[off as usize..off as usize + bpb as usize];
                let dst = &mut out[didx as usize..didx as usize + bpb as usize];
                dst.copy_from_slice(src);
            }
        }
    }

    Ok(out)
}

pub fn swizzle_block_linear(
    data: &[u8],
    width: u32,
    height: u32,
    bpb: u32,
    gob_blocks_y: u32,
    width_in_gobs: u32,
) -> Result<Vec<u8>, PitaError> {
    let bpb = if bpb == 0 { BPB_DEFAULT } else { bpb };
    let wb = width.div_ceil(BLK_PX);
    let hb = height.div_ceil(BLK_PX);
    let groups = hb.div_ceil(gob_blocks_y * GOB_HEIGHT);
    let mip0_gobs = groups as u64 * gob_blocks_y as u64 * width_in_gobs as u64;
    let out_size = (mip0_gobs * 512) as usize;

    let mut out = vec![0u8; out_size];

    for by in 0..hb {
        for bx in 0..wb {
            let off = block_offset(bx, by, wb, gob_blocks_y, width_in_gobs, bpb);
            let didx = (by as u64 * wb as u64 + bx as u64) * bpb as u64;
            let src = &data[didx as usize..didx as usize + bpb as usize];
            let dst = &mut out[off as usize..off as usize + bpb as usize];
            dst.copy_from_slice(src);
        }
    }

    Ok(out)
}
