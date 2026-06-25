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

#[test]
fn tex_decode_all_no_crash() {
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

    let mut tex_files = Vec::new();
    collect_tex_files(&scene_dir, &mut tex_files);
    tex_files.sort();

    let mut ok = 0u32;

    for path in &tex_files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let spr_path = path.with_extension("spr");
        let spr_data = if spr_path.exists() {
            std::fs::read(&spr_path).ok()
        } else {
            None
        };

        match pita::tex::decoder::decode_to_dds(&data, spr_data.as_deref()) {
            Ok(result) => {
                assert!(
                    result.dds_data.len() > 148,
                    "DDS too small for {}",
                    path.display()
                );
                assert_eq!(
                    &result.dds_data[0..4],
                    b"DDS ",
                    "Bad DDS magic for {}",
                    path.display()
                );
                assert!(result.width >= 4, "Width too small for {}", path.display());
                assert!(result.height >= 4, "Height too small for {}", path.display());
                ok += 1;
            }
            Err(e) => {
                panic!("Failed to decode {}: {}", path.display(), e);
            }
        }
    }

    assert!(ok > 0, "No .tex files found in {}", scene_dir.display());
    eprintln!("Decoded {} files, {} OK", tex_files.len(), ok);
}
