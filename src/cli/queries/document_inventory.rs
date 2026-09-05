//! 문서 전체의 개수와 포함 개체를 열거하는 read-only CLI 어댑터.
//!
//! Stage 2의 첫 수직 절편은 기존 binary-local load seam을 그대로 사용한다. 문서 열기와
//! 오류 타입을 service 계층으로 옮기는 작업은 Stage 3의 책임이다.

use std::fs;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{
    collect_field_records, fields_json_value, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};

/// [#4999] `word-count` — IR 본문에서 구역·문단·글자·어절·쪽 수를 센다.
pub(crate) fn word_count(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
    }
    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp word-count <파일.hwp|파일.hwpx|파일.hml> [--json]");
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
    let mut paragraph_count = 0usize;
    let mut char_count = 0usize;
    let mut word_count = 0usize;
    for section in &doc.document().sections {
        paragraph_count += section.paragraphs.len();
        for para in &section.paragraphs {
            char_count += para.text.chars().count();
            word_count += para.text.split_whitespace().count();
        }
    }
    let section_count = doc.document().sections.len();
    let page_count = doc.page_count();
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "sectionCount": section_count,
            "paragraphCount": paragraph_count,
            "charCount": char_count,
            "wordCount": word_count,
            "pageCount": page_count,
        });
        println!("{}", envelope);
        return EXIT_OK;
    }
    println!(
        "{file_path}: 구역 {section_count} · 문단 {paragraph_count} · 글자 {char_count} · 어절 {word_count} · 쪽 {page_count}"
    );
    EXIT_OK
}

/// [#5025] `bookmarks` — 문서 책갈피 목록.
pub(crate) fn bookmarks(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
    }
    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp bookmarks <파일.hwp|파일.hwpx|파일.hml> [--json]");
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
    let raw = match doc.get_bookmarks_native() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 책갈피 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let items: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 책갈피 JSON 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let count = items.as_array().map(|a| a.len()).unwrap_or(0);
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "count": count,
            "bookmarks": items,
        });
        println!("{}", envelope);
        return EXIT_OK;
    }
    println!("{file_path}: 책갈피 {count}개");
    if let Some(arr) = items.as_array() {
        for b in arr {
            let name = b["name"].as_str().unwrap_or("");
            let sec = b["sec"].as_u64().unwrap_or(0);
            let para = b["para"].as_u64().unwrap_or(0);
            let ctrl = b["ctrlIdx"].as_u64().unwrap_or(0);
            println!("  {name}  구역 {sec} 문단 {para} 컨트롤 {ctrl}");
        }
    }
    EXIT_OK
}

/// `charts` — 문서 차트 목록.
pub(crate) fn charts(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
    }
    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp charts <파일.hwp|파일.hwpx|파일.hml> [--json]");
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
    let raw = match doc.list_charts_native() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 차트 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let items: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 차트 JSON 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let count = items.as_array().map(|a| a.len()).unwrap_or(0);
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "count": count,
            "charts": items,
        });
        println!("{}", envelope);
        return EXIT_OK;
    }
    println!("{file_path}: 차트 {count}개");
    if let Some(arr) = items.as_array() {
        for c in arr {
            let idx = c["index"].as_u64().unwrap_or(0);
            let sec = c["section"].as_u64().unwrap_or(0);
            let para = c["paragraph"].as_u64().unwrap_or(0);
            let ctrl = c["control"].as_u64().unwrap_or(0);
            println!("  {idx}  구역 {sec} 문단 {para} 컨트롤 {ctrl}");
        }
    }
    EXIT_OK
}

/// `fields` — 누름틀/필드 조사 (읽기 전용).
///
/// rhwp 는 이미 필드에 값을 **쓸 수** 있는데(`set_field_value_by_name`) 조회 API 는
/// WASM/스튜디오 경로에만 있어, 브라우저 밖 에이전트는 "이 서식이 무엇을 요구하는지"
/// 알 방법이 없었다. 기존 `collect_all_fields()` 를 그대로 노출한다(라이브러리 무변경).
pub(crate) fn show_fields(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => file_path = Some(other),
        }
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp fields <파일.hwp|파일.hwpx> [--json]");
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

    let fields = collect_field_records(&doc);

    if json_mode {
        let envelope = fields_json_value(file_path, &fields);
        println!("{envelope}");
        return EXIT_OK;
    }

    println!("문서 로드: {} (필드 {}개)", file_path, fields.len());
    for f in &fields {
        let name = f["name"].as_str().unwrap_or("");
        let label = if name.is_empty() {
            "(이름 없음)"
        } else {
            name
        };
        println!(
            "  [{}] {} = {:?}{}",
            f["fieldType"].as_str().unwrap_or("?"),
            label,
            f["value"].as_str().unwrap_or(""),
            if f["editableInForm"] == true {
                ""
            } else {
                " (서식 편집 불가)"
            }
        );
    }
    EXIT_OK
}
