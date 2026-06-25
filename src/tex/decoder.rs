use crate::dds;
use crate::error::PitaError;
use crate::swizzle;
use crate::tex::footer::read_footer;
use crate::tex::gby::resolve_gby;
use crate::PitaResult;

#[derive(Debug)]
pub struct TexDecodeResult {
    pub dds_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: u16,
}

pub fn decode_to_dds(data: &[u8], companion_spr: Option<&[u8]>) -> PitaResult<TexDecodeResult> {
    if data.len() < 4 {
        return Err(PitaError::Truncated);
    }

    if data[0..4] != [0x28, 0xB5, 0x2F, 0xFD] {
        return Err(PitaError::BadMagic);
    }

    let decompressed = zstd::decode_all(data).map_err(|_| PitaError::DecompressFailed)?;
    let footer = read_footer(&decompressed)?;

    let fmt = footer.format;
    let bpb = dds::bytes_per_block(fmt);
    let w = footer.width as u32;
    let h = footer.height as u32;
    let wb = w.div_ceil(4);
    let hb = h.div_ceil(4);

    let gby = resolve_gby(wb, hb, companion_spr);
    let wig = (wb * bpb).div_ceil(64);

    let gob_height = 8u32;
    let groups = hb.div_ceil(gby * gob_height);
    let mip0_gobs = groups * gby * wig;
    let mip0_sz = mip0_gobs * 512;

    let padded = if (decompressed.len() as u32) < mip0_sz {
        let mut padded = Vec::with_capacity(mip0_sz as usize);
        padded.extend_from_slice(&decompressed);
        padded.resize(mip0_sz as usize, 0);
        padded
    } else {
        decompressed
    };

    let pixels = swizzle::deswizzle_block_linear(&padded, w, h, bpb, gby, wig)?;
    let data_size = dds::block_data_size(w, h, fmt);
    let header = dds::build_dds_header(w, h, data_size, fmt);

    let mut dds_data = Vec::with_capacity(header.len() + pixels.len());
    dds_data.extend_from_slice(&header);
    dds_data.extend_from_slice(&pixels);

    Ok(TexDecodeResult {
        dds_data,
        width: w,
        height: h,
        format: fmt,
    })
}
