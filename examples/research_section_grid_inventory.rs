use rhwp::parse_document;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            collect(&entry.path(), files);
        }
        return;
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if extension.eq_ignore_ascii_case("hwp") || extension.eq_ignore_ascii_case("hwpx") {
        files.push(path.to_path_buf());
    }
}

fn main() {
    let mut files = Vec::new();
    for argument in env::args_os().skip(1) {
        collect(Path::new(&argument), &mut files);
    }
    files.sort();

    for path in files {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(document) = parse_document(&bytes) else {
            continue;
        };
        let snap_para_shape_count = document
            .doc_info
            .para_shapes
            .iter()
            .filter(|shape| shape.attr1 & 0x100 != 0)
            .count();

        for (section_index, section) in document.sections.iter().enumerate() {
            let definition = &section.section_def;
            if definition.line_grid == 0 && definition.char_grid == 0 {
                continue;
            }
            println!(
                "{}",
                json!({
                    "path": path,
                    "section": section_index,
                    "line_grid": definition.line_grid,
                    "char_grid": definition.char_grid,
                    "snap_para_shape_count": snap_para_shape_count,
                    "para_shape_count": document.doc_info.para_shapes.len(),
                })
            );
        }
    }
}
