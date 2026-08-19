//! 주소가 붙은 본문 검색. `DocumentCore::grep` 만 부르며 문서를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

pub fn run_grep(args: &[String]) -> i32 {
    let usage = "rhwp-agent grep <파일> --q <문자열> [--ignore-case] [--limit <N>] [--context <N>] [--json]";
    let mut json_mode = false;
    let mut ignore_case = false;
    let mut path: Option<String> = None;
    let mut query: Option<String> = None;
    let mut limit: usize = 200;
    let mut context: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--ignore-case" => {
                ignore_case = true;
                i += 1;
            }
            "--q" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("오류: --q 뒤에 검색어가 필요합니다.");
                    return EXIT_USAGE;
                };
                query = Some(v.clone());
                i += 2;
            }
            "--limit" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --limit 뒤에 숫자가 필요합니다.");
                    return EXIT_USAGE;
                };
                if v == 0 {
                    eprintln!("오류: --limit 은 1 이상이어야 합니다.");
                    return EXIT_USAGE;
                }
                limit = v;
                i += 2;
            }
            "--context" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --context 뒤에 숫자가 필요합니다.");
                    return EXIT_USAGE;
                };
                context = Some(v);
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
    let Some(query) = query.filter(|s| !s.is_empty()) else {
        eprintln!("오류: --q 검색어가 필요합니다.");
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

    let matches = core.grep_with_context(&query, !ignore_case, Some(limit), context);
    let truncated = matches.len() >= limit;
    let payload = json!({
        "source": path,
        "query": query,
        "ignoreCase": ignore_case,
        "limit": limit,
        "matchCount": matches.len(),
        "truncated": truncated,
        "matches": matches,
    });
    if json_mode {
        print_json(&envelope(
            "grep",
            payload,
            &[
                "query",
                "matches[].text",
                "matches[].context",
                "matches[].contextBefore[]",
                "matches[].contextAfter[]",
            ],
        ));
    } else {
        crate::outln!("{}", payload["matchCount"]);
        for m in payload["matches"].as_array().cloned().unwrap_or_default() {
            let page = m["page"]
                .as_u64()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into());
            crate::outln!(
                "p{} s{}:{} +{}  {}",
                page,
                m["section"],
                m["paragraph"],
                m["charOffset"],
                m["context"].as_str().unwrap_or("")
            );
        }
    }
    EXIT_OK
}
