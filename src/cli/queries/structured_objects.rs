//! 문서의 양식 개체와 머리말·꼬리말을 조회하는 read-only CLI 어댑터.
//!
//! Stage 2에서는 기존 binary-local load seam을 보존한다. service 계층 이행은 Stage 3의
//! 책임이다.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// 양식 개체 값 조회 — 코어 `get_form_value_native`.
pub(crate) fn form_value(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp form-value <파일.hwp|파일.hwpx|파일.hml> --section N --para N --ctrl N [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
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
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
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
    let raw = match doc.get_form_value_native(section, para, ctrl) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 양식 값 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let form: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 양식 JSON 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
        });
        if let Some(obj) = form.as_object() {
            for (k, v) in obj {
                envelope[k] = v.clone();
            }
        }
        println!("{}", provenance::marked(envelope, "form-value"));
        return EXIT_OK;
    }
    if form["ok"] == true {
        println!(
            "{file_path}: 양식 {} name={} value={} text={} caption={} enabled={}",
            form["formType"].as_str().unwrap_or(""),
            form["name"].as_str().unwrap_or(""),
            form["value"],
            form["text"].as_str().unwrap_or(""),
            form["caption"].as_str().unwrap_or(""),
            form["enabled"]
        );
    } else {
        let err = form["error"].as_str().unwrap_or("not a form object");
        println!("{file_path}: 양식 아님 — {err} (구역 {section} 문단 {para} 컨트롤 {ctrl})");
    }
    EXIT_OK
}

/// `header-footer` — 구역의 머리말/꼬리말 한 건 조회.
pub(crate) fn header_footer(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    }
                }
            }
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
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp header-footer <파일.hwp|파일.hwpx|파일.hml> [--header|--footer] [--section N] [--apply-to 0|1|2] [--json]"
        );
        return EXIT_USAGE;
    };
    let is_header = is_header.unwrap_or(true);
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
    let raw = match doc.get_header_footer_native(section, is_header, apply_to) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말/꼬리말 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 머리말/꼬리말 JSON 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let exists = parsed["exists"].as_bool().unwrap_or(false);
    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "exists": exists,
        });
        if exists {
            if let Some(obj) = parsed.as_object() {
                for key in [
                    "kind",
                    "label",
                    "paraIndex",
                    "controlIndex",
                    "paraCount",
                    "text",
                ] {
                    if let Some(v) = obj.get(key) {
                        envelope[key] = v.clone();
                    }
                }
            }
        }
        println!("{}", provenance::marked(envelope, "header-footer"));
        return EXIT_OK;
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    if exists {
        let text = parsed["text"].as_str().unwrap_or("");
        println!("{file_path}: {kind} 있음 (구역 {section} apply-to {apply_to})");
        if !text.is_empty() {
            println!("  {text}");
        }
    } else {
        println!("{file_path}: {kind} 없음 (구역 {section} apply-to {apply_to})");
    }
    EXIT_OK
}

/// [#5044] `headers-footers` — 문서 머리말/꼬리말 목록.
pub(crate) fn headers_footers(args: &[String]) -> i32 {
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
        eprintln!("사용법: rhwp headers-footers <파일.hwp|파일.hwpx|파일.hml> [--json]");
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
    let raw = match doc.get_header_footer_list_native(0, true, 0) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 머리말/꼬리말 조회 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 머리말/꼬리말 JSON 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let items = parsed
        .get("items")
        .cloned()
        .unwrap_or(serde_json::json!([]));
    let count = items.as_array().map(|a| a.len()).unwrap_or(0);
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "count": count,
            "headersFooters": items,
        });
        println!("{}", provenance::marked(envelope, "headers-footers"));
        return EXIT_OK;
    }
    println!("{file_path}: 머리말/꼬리말 {count}개");
    if let Some(arr) = items.as_array() {
        for h in arr {
            let label = h["label"].as_str().unwrap_or("");
            let sec = h["sectionIdx"].as_u64().unwrap_or(0);
            let apply = h["applyTo"].as_u64().unwrap_or(0);
            println!("  {label}  구역 {sec} apply-to {apply}");
        }
    }
    EXIT_OK
}
