//! search-all — `search_all_text_native`. 조회만 한다.

use serde_json::json;

use crate::envelope::{
    envelope, load_core, parse_json_string, print_json, write_stdout, EXIT_USAGE,
};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str =
        "rhwp-q-kit search-all <파일> --q <문자열> [--ignore-case] [--include-cells] [--json]";
    let mut path = None;
    let mut query = None;
    let mut json_mode = false;
    let mut case_sensitive = true;
    let mut include_cells = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--ignore-case" => case_sensitive = false,
            "--include-cells" => include_cells = true,
            "--q" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --q 뒤에 검색어가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                query = Some(v.clone());
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
    let Some(query) = query else {
        eprintln!("오류: --q 검색어가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.search_all_text_native(&query, case_sensitive, include_cells) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 검색 실패 - {e}");
            return crate::envelope::EXIT_RUNTIME;
        }
    };
    let matches = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let match_count = matches.as_array().map(|a| a.len()).unwrap_or(0);
    let payload = json!({
        "source": path,
        "query": query,
        "caseSensitive": case_sensitive,
        "includeCells": include_cells,
        "matchCount": match_count,
        "matches": matches,
    });
    if json_mode {
        print_json(&envelope("search-all", payload, &["query", "matches"]))
    } else {
        write_stdout(&format!("matches={match_count}"))
    }
}
