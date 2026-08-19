//! 주소가 있는 문서 검색 CLI query 어댑터.
//!
//! Stage 2에서는 batch와 MCP도 소비하는 JSON envelope helper를 crate root에 보존한다.
//! 재사용 가능한 application/service 경계로의 이행은 Stage 3의 책임이다.

use std::fs;

use crate::{load_document, search_json_value, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// `search` — 주소(구역·문단·**페이지**)를 가진 문서 검색.
///
/// 평문을 뽑아 외부에서 찾으면 주소가 소멸해 근거 제시가 불가능하다. rhwp 는 조판 엔진이
/// 있어 "몇 쪽"에 답할 수 있는 유일한 도구인데, 그 출구가 없었다.
pub(crate) fn search_document(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut query: Option<&str> = None;
    let mut json_mode = false;
    let mut ignore_case = false;
    let mut limit: Option<usize> = None;
    let mut context: Option<usize> = None;

    // POSIX 옵션 종결자. 검색어가 '-' 로 시작하면 종전에는 플래그로 먹혔다 —
    // `-i` 는 대소문자 축을 **조용히** 뒤집고(리터럴 "-i" 를 찾으려던 호출이 다음
    // 위치 인자를 대소문자 무시로 검색한다), 그 외에는 "알 수 없는 옵션" 으로 죽어
    // 하이픈으로 시작하는 문자열은 아예 검색할 수 없었다.
    let mut end_of_options = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" if !end_of_options => end_of_options = true,
            "--json" if !end_of_options => json_mode = true,
            "--ignore-case" | "-i" if !end_of_options => ignore_case = true,
            // [#3787 S7] `--max-matches` 는 자원 상한 어휘를 텍스트 축
            // (`export-text --max-chars`)과 맞춘 이름이고, `--limit`(#3353)은 같은
            // 축의 기존 이름이다. 두 이름이 같은 변수를 채우므로 의미 분기는 없다.
            "--limit" | "--max-matches" if !end_of_options => {
                let flag = args[i].clone();
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => limit = Some(n),
                    _ => {
                        eprintln!("오류: {flag} 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            // [#3835] 매치 앞뒤 문단을 함께 보고 싶은 에이전트용 — 매치가 속한 문단의
            // 앞뒤 N개 문단 텍스트를 matches[].contextBefore/contextAfter 로 얹는다.
            // 기본(플래그 없음)은 종전과 완전히 동일하다.
            "--context" if !end_of_options => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => context = Some(n),
                    _ => {
                        eprintln!("오류: --context 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if !end_of_options && other.starts_with('-') => {
                // 옵션 오타는 계속 거부한다(삼키면 오타가 검색어가 되어 조용히 0건이 된다).
                // 다만 검색어가 정말 '-' 로 시작하는 경우 빠져나갈 길을 알려줘야 한다 —
                // 안내가 없으면 에이전트는 "고치라"는 exit 2 를 받고도 고칠 방법을 모른다.
                eprintln!(
                    "알 수 없는 옵션: {other}\n\
                     힌트: 검색어가 '-' 로 시작한다면 `--` 뒤에 두세요 — \
                     rhwp search <파일> --json -- <검색어>"
                );
                return EXIT_USAGE;
            }
            other => {
                if file_path.is_none() {
                    file_path = Some(other);
                } else if query.is_none() {
                    query = Some(other);
                } else {
                    eprintln!("오류: 인자가 너무 많습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(query)) = (file_path, query) else {
        eprintln!(
            "사용법: rhwp search <파일.hwp|파일.hwpx> <검색어> [--json] [--ignore-case] \
             [--max-matches <N>] [--context <N>]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    // [#3353] 총량을 보고하려면 전수 스캔이 불가피하다 — `--limit` 의 목적은 스캔 시간이
    // 아니라 출력 컨텍스트 절약이므로, 전수 grep 후 표시만 절단한다. 절단 사실을 숨기면
    // 에이전트가 "정확히 N건"과 "N건만 표시(실제 그 이상)"를 구별할 수 없다.
    let all_matches = doc.grep_with_context(query, !ignore_case, None, context);
    let total_match_count = all_matches.len();
    let matches: Vec<_> = match limit {
        Some(n) => all_matches.into_iter().take(n).collect(),
        None => all_matches,
    };
    let truncated = matches.len() < total_match_count;

    if json_mode {
        // [#3353] matchCount 는 반환된 매치 수이고, 추가-전용 totalMatchCount·truncated가
        // 전체 수와 절단 여부를 표현한다. #3346 batch와 하나의 helper를 공유한다.
        let envelope =
            search_json_value(file_path, query, !ignore_case, &matches, total_match_count);
        println!("{envelope}");
        // 매치 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707).
        return EXIT_OK;
    }

    if truncated {
        println!(
            "검색: {:?} in {} — {}건 중 {}건 표시 (--max-matches)",
            query,
            file_path,
            total_match_count,
            matches.len()
        );
    } else {
        println!("검색: {:?} in {} — {}건", query, file_path, matches.len());
    }
    for m in &matches {
        let page = m
            .page
            .map(|p| format!("{}쪽", p + 1))
            .unwrap_or_else(|| "쪽 미배치".to_string());
        println!(
            "  [{}] 구역{}:문단{} +{}  {}",
            page, m.section, m.paragraph, m.char_offset, m.context
        );
    }
    EXIT_OK
}
