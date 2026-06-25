use std::path::{Path, PathBuf};

fn get_romfs_dir() -> Option<PathBuf> {
    std::env::var("PITA_ROMFS_DIR").ok().map(PathBuf::from)
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

#[test]
fn spr_decode_all() {
    let romfs = match get_romfs_dir() {
        Some(d) => d,
        None => {
            eprintln!("Skipping: PITA_ROMFS_DIR not set");
            return;
        }
    };

    let scene_dir = romfs.join("graph").join("scene");
    if !scene_dir.exists() {
        eprintln!("Skipping: {} not found", scene_dir.display());
        return;
    }

    let mut spr_files = Vec::new();
    collect_spr_files(&scene_dir, &mut spr_files);
    spr_files.sort();

    let mut ok = 0u32;

    for path in &spr_files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        match pita::spr::decoder::decode(&data) {
            Ok(_) => {
                ok += 1;
            }
            Err(e) => {
                panic!("Failed to decode {}: {}", path.display(), e);
            }
        }
    }

    assert!(ok > 0, "No .spr files found in {}", scene_dir.display());
    eprintln!("Parsed {} .spr files, {} OK", spr_files.len(), ok);
}

#[test]
fn spr_header_fields() {
    let romfs = match get_romfs_dir() {
        Some(d) => d,
        None => {
            eprintln!("Skipping: PITA_ROMFS_DIR not set");
            return;
        }
    };

    let scene_dir = romfs.join("graph").join("scene");
    if !scene_dir.exists() {
        return;
    }

    let mut spr_files = Vec::new();
    collect_spr_files(&scene_dir, &mut spr_files);

    for path in &spr_files {
        let data = std::fs::read(path).unwrap();
        if let Ok(decoded) = pita::spr::decoder::decode(&data) {
            let h = &decoded.header;
            // Verify header fields are within reasonable ranges
            assert!(
                h.cnt() <= 1000,
                "cnt too large in {}: {}",
                path.display(),
                h.cnt()
            );
            assert!(
                h.h1c() <= 1000,
                "h1c too large in {}: {}",
                path.display(),
                h.h1c()
            );
        }
    }
}
