use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

use crate::error::PitaError;
use crate::spr;
use crate::tex::decoder;

const SPR_FIELD_COUNT: i32 = 14;
const SPR_SECTION_COUNT: i32 = 14;

#[no_mangle]
pub extern "C" fn pita_tex_decode(
    tex_data: *const u8,
    tex_len: usize,
    spr_data: *const u8,
    spr_len: usize,
    out_dds: *mut *mut u8,
    out_len: *mut usize,
    out_width: *mut c_int,
    out_height: *mut c_int,
    out_format: *mut c_int,
) -> c_int {
    if tex_data.is_null() || out_dds.is_null() || out_len.is_null() {
        return PitaError::InvalidArg as c_int;
    }

    let tex_slice = unsafe { std::slice::from_raw_parts(tex_data, tex_len) };
    let spr_slice = if !spr_data.is_null() && spr_len > 0 {
        Some(unsafe { std::slice::from_raw_parts(spr_data, spr_len) })
    } else {
        None
    };

    match decoder::decode_to_dds(tex_slice, spr_slice) {
        Ok(mut result) => {
            result.dds_data.shrink_to_fit();
            let len = result.dds_data.len();
            let ptr = result.dds_data.as_ptr();
            std::mem::forget(result.dds_data);

            unsafe {
                *out_dds = ptr as *mut u8;
                *out_len = len;
                if !out_width.is_null() {
                    *out_width = result.width as c_int;
                }
                if !out_height.is_null() {
                    *out_height = result.height as c_int;
                }
                if !out_format.is_null() {
                    *out_format = result.format as c_int;
                }
            }
            PitaError::None as c_int
        }
        Err(e) => e as c_int,
    }
}

#[no_mangle]
pub extern "C" fn pita_tex_free(dds_ptr: *mut u8, len: usize) {
    if !dds_ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(dds_ptr, len, len);
        }
    }
}

#[no_mangle]
pub extern "C" fn pita_spr_decode(
    data: *const u8,
    len: usize,
) -> *mut c_void {
    if data.is_null() || len == 0 {
        return std::ptr::null_mut();
    }

    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    match spr::decoder::decode(slice) {
        Ok(decoded) => {
            Box::into_raw(Box::new(decoded)) as *mut c_void
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn pita_spr_header(
    spr: *const c_void,
    field: c_int,
    out: *mut u32,
) -> c_int {
    if spr.is_null() || out.is_null() {
        return PitaError::InvalidArg as c_int;
    }

    let decoded = unsafe { &*(spr as *const spr::decoder::SprDecoded) };

    if !(0..SPR_FIELD_COUNT).contains(&field) {
        return -1;
    }

    unsafe {
        *out = decoded.header.raw[field as usize];
    }
    0
}

#[no_mangle]
pub extern "C" fn pita_spr_section(
    spr: *const c_void,
    section: c_int,
    out_data: *mut *const u8,
    out_size: *mut usize,
) -> c_int {
    if spr.is_null() || out_data.is_null() || out_size.is_null() {
        return PitaError::InvalidArg as c_int;
    }

    let decoded = unsafe { &*(spr as *const spr::decoder::SprDecoded) };

    if !(0..SPR_SECTION_COUNT).contains(&section) {
        return -1;
    }

    let blob = &decoded.sections.blobs[section as usize];
    unsafe {
        *out_data = blob.as_ptr();
        *out_size = blob.len();
    }
    0
}

#[no_mangle]
pub extern "C" fn pita_spr_free(spr: *mut c_void) {
    if !spr.is_null() {
        unsafe {
            let _ = Box::from_raw(spr as *mut spr::decoder::SprDecoded);
        }
    }
}

#[no_mangle]
pub extern "C" fn pita_error_string(code: c_int) -> *const c_char {
    let err = match code {
        0 => &PitaError::None,
        -1 => &PitaError::InvalidArg,
        -2 => &PitaError::BadMagic,
        -3 => &PitaError::Truncated,
        -4 => &PitaError::DecompressFailed,
        -5 => &PitaError::BadFooter,
        -6 => &PitaError::DeswizzleOverflow,
        -7 => &PitaError::SprHeaderInvalid,
        -8 => &PitaError::SprSectionOverlap,
        -99 => &PitaError::Internal,
        _ => &PitaError::Internal,
    };
    err.c_str().as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};

    fn get_romfs_dir() -> Option<PathBuf> {
        std::env::var("PITA_ROMFS_DIR").ok().map(PathBuf::from)
    }

    fn collect_tex_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_tex_files(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("tex") {
                    files.push(path);
                }
            }
        }
    }

    fn collect_spr_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_spr_files(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("spr") {
                    files.push(path);
                }
            }
        }
    }

    fn scene_dir() -> Option<PathBuf> {
        let romfs = get_romfs_dir()?;
        let dir = romfs.join("graph").join("scene");
        if dir.exists() { Some(dir) } else { None }
    }

    #[test]
    fn c_api_tex_decode() {
        let scene = match scene_dir() {
            Some(d) => d,
            None => {
                eprintln!("Skipping: PITA_ROMFS_DIR/graph/scene not found");
                return;
            }
        };

        let mut tex_files = Vec::new();
        collect_tex_files(&scene, &mut tex_files);
        if tex_files.is_empty() {
            eprintln!("Skipping: no .tex files found");
            return;
        }
        tex_files.sort();

        for tex_path in &tex_files {
            let tex_data = std::fs::read(tex_path).unwrap();

            let mut out_dds: *mut u8 = std::ptr::null_mut();
            let mut out_len: usize = 0;
            let mut out_width: i32 = 0;
            let mut out_height: i32 = 0;
            let mut out_format: i32 = 0;

            let ret = super::pita_tex_decode(
                tex_data.as_ptr(),
                tex_data.len(),
                std::ptr::null(),
                0,
                &mut out_dds,
                &mut out_len,
                &mut out_width,
                &mut out_height,
                &mut out_format,
            );

            assert_eq!(ret, 0, "pita_tex_decode({}) returned {}", tex_path.display(), ret);
            assert!(!out_dds.is_null(), "out_dds is null for {}", tex_path.display());
            assert!(out_len > 148, "DDS too small for {}", tex_path.display());
            assert!(out_width >= 4, "width for {}", tex_path.display());
            assert!(out_height >= 4, "height for {}", tex_path.display());

            let dds_slice = unsafe { std::slice::from_raw_parts(out_dds, out_len) };
            assert_eq!(&dds_slice[0..4], b"DDS ", "Bad DDS magic for {}", tex_path.display());
            super::pita_tex_free(out_dds, out_len);
        }
    }

    #[test]
    fn c_api_tex_decode_invalid() {
        let garbage = b"not a tex file";
        let mut out_dds: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;

        let ret = super::pita_tex_decode(
            garbage.as_ptr(),
            garbage.len(),
            std::ptr::null(),
            0,
            &mut out_dds,
            &mut out_len,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        assert!(ret < 0, "Expected error code, got {}", ret);
    }

    #[test]
    fn c_api_spr_decode() {
        let scene = match scene_dir() {
            Some(d) => d,
            None => {
                eprintln!("Skipping: PITA_ROMFS_DIR/graph/scene not found");
                return;
            }
        };

        let mut spr_files = Vec::new();
        collect_spr_files(&scene, &mut spr_files);
        if spr_files.is_empty() {
            eprintln!("Skipping: no .spr files found");
            return;
        }
        spr_files.sort();

        for spr_path in &spr_files {
            let spr_data = std::fs::read(spr_path).unwrap();
            let spr = super::pita_spr_decode(spr_data.as_ptr(), spr_data.len());
            assert!(!spr.is_null(), "pita_spr_decode returned null for {}", spr_path.display());

            let mut cnt: u32 = 0;
            let ret = super::pita_spr_header(spr, 1, &mut cnt);
            assert_eq!(ret, 0, "pita_spr_header failed for {}", spr_path.display());

            let mut sec_data: *const u8 = std::ptr::null();
            let mut sec_size: usize = 0;
            let ret = super::pita_spr_section(spr, 0, &mut sec_data, &mut sec_size);
            assert_eq!(ret, 0, "pita_spr_section failed for {}", spr_path.display());

            super::pita_spr_free(spr as *mut c_void);
        }
    }

    #[test]
    fn c_api_error_string() {
        let s = super::pita_error_string(0);
        assert!(!s.is_null());
        let cstr = unsafe { std::ffi::CStr::from_ptr(s) };
        assert_eq!(cstr.to_str().unwrap(), "Success");

        let s = super::pita_error_string(-2);
        assert!(!s.is_null());
        let cstr = unsafe { std::ffi::CStr::from_ptr(s) };
        assert!(!cstr.to_bytes().is_empty());
    }
}
