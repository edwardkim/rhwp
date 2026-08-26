//! 원본 그림 바이트 메타 — `DocumentCore::get_source_image_bytes_native`.

use crate::envelope::{envelope, load_core, print_json, write_stdout, EXIT_USAGE};
use base64::Engine;
use serde_json::json;

const USAGE: &str = "rhwp-q-kit source-image-bytes <파일> --key <키> [--include-bytes] [--json]";

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut include_bytes = false;
    let mut path: Option<String> = None;
    let mut key: Option<String> = None;
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
            "--key" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --key 뒤에 그림 키가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                key = Some(raw.clone());
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
    let (Some(path), Some(key)) = (path, key) else {
        eprintln!("오류: 파일과 --key 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let got = core.get_source_image_bytes_native(&key);
    let present = got.is_some();
    let mime = got.as_ref().map(|(m, _)| *m);
    let len = got.as_ref().map(|(_, b)| b.len()).unwrap_or(0);
    let mut payload = json!({
        "source": path,
        "key": key,
        "present": present,
        "mime": mime,
        "len": len,
    });
    let mut untrusted: Vec<&str> = vec!["key"];
    if include_bytes {
        if let Some((_, data)) = got.as_ref() {
            payload["bytesBase64"] = json!(base64::engine::general_purpose::STANDARD.encode(data));
            untrusted.push("bytesBase64");
        }
    }
    if json_mode {
        print_json(&envelope("source-image-bytes", payload, &untrusted))
    } else {
        let mime_s = mime.unwrap_or("");
        let line = format!("present={present} mime={mime_s} len={len}");
        if include_bytes {
            if let Some((_, data)) = got.as_ref() {
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
