//! layer-tree — `get_page_layer_tree_native(page)`. 조회만 한다.

use serde_json::json;

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_u32, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit layer-tree <파일> --page <N> [--json]";
    let mut path = None;
    let mut json_mode = false;
    let mut page = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--page" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match parse_u32("--page", v) {
                    Ok(n) => page = Some(n),
                    Err(c) => return c,
                }
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let Some(page) = page else {
        eprintln!("오류: --page 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_page_layer_tree_native(page) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 레이어 트리 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let tree = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": path,
        "page": page,
        "tree": tree,
    });
    if json_mode {
        print_json(&envelope("layer-tree", payload, &["tree"]))
    } else {
        write_stdout(&format!("page={page} bytes={}", raw.len()))
    }
}
