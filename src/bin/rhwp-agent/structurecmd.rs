//! 목차·조문 구조. `export-structure` 와 같은 코어를 읽기만 한다.

use crate::envelope::{
    envelope, load_core, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use rhwp::document_core::queries::structure::{build_structure, StructureMode};
use serde_json::json;

pub fn run_structure(args: &[String]) -> i32 {
    let usage = "rhwp-agent structure <파일> [--mode auto|outline|clause] [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut mode = StructureMode::Auto;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--mode" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("오류: --mode 뒤에 auto|outline|clause 가 필요합니다.");
                    return EXIT_USAGE;
                };
                let Some(parsed) = StructureMode::parse(v) else {
                    eprintln!("오류: --mode 는 auto|outline|clause 여야 합니다.");
                    return EXIT_USAGE;
                };
                mode = parsed;
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let data = match read_file(&path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
            return EXIT_RUNTIME;
        }
    };
    let tree = build_structure(core.document(), mode);
    let payload = json!({
        "source": path,
        "mode": tree.mode,
        "nodeCount": tree.node_count,
        "structure": tree,
    });
    if json_mode {
        print_json(&envelope(
            "structure",
            payload,
            &["structure.roots[].heading", "structure.roots[].body[]"],
        ));
    } else {
        crate::outln!("nodes={}", tree.node_count);
    }
    EXIT_OK
}
