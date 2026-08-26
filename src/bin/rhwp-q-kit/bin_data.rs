//! 임베드 바이너리 길이·종류 — `DocumentCore::get_bin_data`.

use crate::envelope::{envelope, load_core, parse_usize, print_json, write_stdout, EXIT_USAGE};
use base64::Engine;
use serde_json::json;

const USAGE: &str = "rhwp-q-kit bin-data <파일> --index <N> [--include-bytes] [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut include_bytes = false;
    let mut path: Option<String> = None;
    let mut index: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--include-bytes" => {
                include_bytes = true;
                i += 1;
            }
            "--index" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --index 뒤에 0 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                index = Some(match parse_usize("--index", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                });
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let (Some(path), Some(index)) = (path, index) else {
        eprintln!("오류: 파일과 --index 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let slot = core.document().bin_data_content.get(index);
    let bytes = core.get_bin_data(index);
    let present = bytes.is_some();
    let len = bytes.as_ref().map(|b| b.len()).unwrap_or(0);
    let id = slot.map(|s| s.id);
    let extension = slot.map(|s| s.extension.as_str());
    let mut payload = json!({
        "source": path,
        "index": index,
        "present": present,
        "len": len,
        "id": id,
        "extension": extension,
    });
    let mut untrusted: Vec<&str> = Vec::new();
    if include_bytes {
        if let Some(data) = bytes.as_ref() {
            payload["bytesBase64"] = json!(base64::engine::general_purpose::STANDARD.encode(data));
            untrusted.push("bytesBase64");
        }
    }
    if json_mode {
        print_json(&envelope("bin-data", payload, &untrusted))
    } else {
        let ext = extension.unwrap_or("");
        let line = format!("present={present} len={len} extension={ext}");
        if include_bytes {
            if let Some(data) = bytes.as_ref() {
                write_stdout(&format!(
                    "{line} bytesBase64={}",
                    base64::engine::general_purpose::STANDARD.encode(data)
                ))
            } else {
                write_stdout(&line)
            }
        } else {
            write_stdout(&line)
        }
    }
}
