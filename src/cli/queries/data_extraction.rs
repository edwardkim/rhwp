//! 행정문서의 날짜·금액·수량을 주소와 함께 조회하는 CLI 어댑터.

use std::fs;

use crate::{extract_data_json_value, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// `extract-data` — 행정문서의 날짜·금액·수량을 **주소와 함께** 뽑는다.
///
/// 문서 구조화의 공통 프리미티브다. 평문을 뽑아 밖에서 정규식을 돌리면 값은 얻어도
/// "어느 쪽 몇 번째 문단"이 소멸해 근거 제시가 안 된다. 인식 규칙과 정규화 규약은
/// `document_core::queries::extract_data` 모듈 문서에 있다.
pub(crate) fn extract_data_command(args: &[String]) -> i32 {
    use rhwp::document_core::queries::extract_data::DataKind;

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    let mut limit: Option<usize> = None;
    let mut kind_arg = "all".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--kind" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some("all") => kind_arg = "all".to_string(),
                    Some(value) if DataKind::parse(value).is_some() => {
                        kind_arg = value.to_string();
                    }
                    _ => {
                        eprintln!("오류: --kind 는 date|amount|number|all 중 하나여야 합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--limit" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => limit = Some(n),
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.is_none() {
                    file_path = Some(other);
                } else {
                    eprintln!("오류: 인자가 너무 많습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp extract-data <파일.hwp|파일.hwpx> [--kind date|amount|number|all] [--limit <N>] [--json]"
        );
        return EXIT_USAGE;
    };

    let selected: Vec<DataKind> = if kind_arg == "all" {
        DataKind::ALL.to_vec()
    } else {
        DataKind::parse(&kind_arg).into_iter().collect()
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

    // [#3353 과 같은 이유] 총량을 보고하려면 전수 스캔이 불가피하다 — `--limit` 은 스캔
    // 시간이 아니라 출력 컨텍스트를 아끼는 장치이므로, 전수 추출 후 표시만 절단한다.
    let all_items = doc.extract_data(&selected);
    let total_item_count = all_items.len();
    let mut counts = serde_json::Map::new();
    for kind in &selected {
        let n = all_items.iter().filter(|it| it.kind == *kind).count();
        counts.insert(kind.as_str().to_string(), serde_json::json!(n));
    }
    let counts = serde_json::Value::Object(counts);

    let items: Vec<_> = match limit {
        Some(n) => all_items.into_iter().take(n).collect(),
        None => all_items,
    };

    if json_mode {
        let envelope =
            extract_data_json_value(file_path, &kind_arg, &items, total_item_count, &counts);
        println!("{envelope}");
        // 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707).
        return EXIT_OK;
    }

    let summary = selected
        .iter()
        .map(|k| format!("{} {}", k.as_str(), counts[k.as_str()]))
        .collect::<Vec<_>>()
        .join(" · ");
    if items.len() < total_item_count {
        println!(
            "추출: {} — {}건 중 {}건 표시 (--limit)  [{}]",
            file_path,
            total_item_count,
            items.len(),
            summary
        );
    } else {
        println!("추출: {} — {}건  [{}]", file_path, items.len(), summary);
    }
    for item in &items {
        let page = item
            .page
            .map(|p| format!("{}쪽", p + 1))
            .unwrap_or_else(|| "쪽 미배치".to_string());
        // 정규화 불가는 감추지 않고 그대로 보인다 — 소비자가 raw 로 판단해야 한다.
        let normalized = match &item.normalized {
            Some(v) => serde_json::to_string(v).unwrap_or_else(|_| "?".to_string()),
            None => "null(정규화 불가)".to_string(),
        };
        let unit = item
            .unit
            .as_deref()
            .map(|u| format!(" {u}"))
            .unwrap_or_default();
        println!(
            "  [{}] 구역{}:문단{} +{}  {:<7} {}  → {}{}",
            page,
            item.section,
            item.paragraph,
            item.char_offset,
            item.kind.as_str(),
            item.raw,
            normalized,
            unit
        );
    }
    EXIT_OK
}
