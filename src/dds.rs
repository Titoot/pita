// Game format constants (from footer format field)
pub(crate) const FMT_BC7: u16 = 1;
pub(crate) const FMT_BC7_LARGE: u16 = 4;

pub fn bytes_per_block(format: u16) -> u32 {
    match format {
        FMT_BC7 | FMT_BC7_LARGE => 16,
        _ => 16,
    }
}

pub fn dxgi_format(_format: u16) -> u32 {
    98 // DXGI_FORMAT_BC7_UNORM
}

pub fn build_dds_header(width: u32, height: u32, data_size: u32, format: u16) -> Vec<u8> {
    let mut hdr = Vec::with_capacity(148);

    hdr.extend_from_slice(b"DDS ");
    hdr.extend_from_slice(&124u32.to_le_bytes());
    hdr.extend_from_slice(&0x00081007u32.to_le_bytes());
    hdr.extend_from_slice(&height.to_le_bytes());
    hdr.extend_from_slice(&width.to_le_bytes());
    hdr.extend_from_slice(&data_size.to_le_bytes());
    hdr.extend_from_slice(&0u32.to_le_bytes());
    hdr.extend_from_slice(&0u32.to_le_bytes());
    hdr.extend_from_slice(&[0u8; 44]);
    hdr.extend_from_slice(&32u32.to_le_bytes());
    hdr.extend_from_slice(&4u32.to_le_bytes());
    hdr.extend_from_slice(b"DX10");
    hdr.extend_from_slice(&[0u8; 20]);
    hdr.extend_from_slice(&0x1000u32.to_le_bytes());
    hdr.extend_from_slice(&[0u8; 16]);
    hdr.extend_from_slice(&dxgi_format(format).to_le_bytes());
    hdr.extend_from_slice(&3u32.to_le_bytes());
    hdr.extend_from_slice(&0u32.to_le_bytes());
    hdr.extend_from_slice(&1u32.to_le_bytes());
    hdr.extend_from_slice(&0u32.to_le_bytes());

    hdr
}

pub fn block_data_size(width: u32, height: u32, format: u16) -> u32 {
    let wb = width.div_ceil(4);
    let hb = height.div_ceil(4);
    wb * hb * bytes_per_block(format)
}
