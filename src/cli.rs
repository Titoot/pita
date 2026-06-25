use std::path::Path;

use pita::tex::decoder;

pub fn decode(path: &str, output: Option<&str>) -> bool {
    let p = Path::new(path);

    if p.is_dir() {
        decode_dir(p, output)
    } else if p.is_file() {
        decode_file(p, output)
    } else {
        eprintln!("Error: '{}' is not a file or directory", path);
        false
    }
}

fn decode_file(tex_path: &Path, output: Option<&str>) -> bool {
    let data = match std::fs::read(tex_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading {}: {}", tex_path.display(), e);
            return false;
        }
    };

    let spr_path = tex_path.with_extension("spr");
    let spr_data = if spr_path.exists() {
        std::fs::read(&spr_path).ok()
    } else {
        None
    };

    match decoder::decode_to_dds(&data, spr_data.as_deref()) {
        Ok(result) => {
            let out_path = match output {
                Some(o) => {
                    let op = Path::new(o);
                    if op.is_dir() {
                        op.join(tex_path.file_stem().unwrap()).with_extension("dds")
                    } else {
                        op.to_path_buf()
                    }
                }
                None => tex_path.with_extension("dds"),
            };

            match std::fs::write(&out_path, &result.dds_data) {
                Err(e) => {
                    eprintln!("Error writing {}: {}", out_path.display(), e);
                    false
                }
                Ok(_) => {
                    println!(
                        "{}: {}x{} fmt={} -> {}",
                        tex_path.file_stem().unwrap().to_string_lossy(),
                        result.width,
                        result.height,
                        result.format,
                        out_path.display()
                    );
                    true
                }
            }
        }
        Err(e) => {
            eprintln!("Error decoding {}: {}", tex_path.display(), e);
            false
        }
    }
}

fn decode_dir(dir: &Path, output: Option<&str>) -> bool {
    let out_root = match output {
        Some(o) => Path::new(o).to_path_buf(),
        None => dir.parent().unwrap_or(dir).join(format!(
            "{}_dds",
            dir.file_name().unwrap().to_string_lossy()
        )),
    };

    let mut ok = 0u32;
    let mut fail = 0u32;

    let entries = match walk_tex_files(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error walking directory: {}", e);
            return false;
        }
    };

    for tex_path in entries {
        let rel = tex_path.strip_prefix(dir).unwrap_or(&tex_path);
        let out_dir = out_root.join(rel.parent().unwrap_or(Path::new("")));
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("Error creating directory {}: {}", out_dir.display(), e);
            fail += 1;
            continue;
        }

        let stem = tex_path.file_stem().unwrap();
        let out_path = out_dir.join(stem).with_extension("dds");

        let data = match std::fs::read(&tex_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error reading {}: {}", tex_path.display(), e);
                fail += 1;
                continue;
            }
        };

        let spr_path = tex_path.with_extension("spr");
        let spr_data = if spr_path.exists() {
            std::fs::read(&spr_path).ok()
        } else {
            None
        };

        match decoder::decode_to_dds(&data, spr_data.as_deref()) {
            Ok(result) => {
                if let Err(e) = std::fs::write(&out_path, &result.dds_data) {
                    eprintln!("Error writing {}: {}", out_path.display(), e);
                    fail += 1;
                } else {
                    ok += 1;
                }
            }
            Err(e) => {
                eprintln!("Error decoding {}: {}", tex_path.display(), e);
                fail += 1;
            }
        }
    }

    println!("\nDecoded {} files ({} OK, {} failed)", ok + fail, ok, fail);
    fail == 0
}

fn walk_tex_files(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        collect_tex_files(dir, &mut files)?;
    }
    Ok(files)
}

fn collect_tex_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tex_files(&path, files)?;
        } else if path.extension().map(|e| e == "tex").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(())
}
