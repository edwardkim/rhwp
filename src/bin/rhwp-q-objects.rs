//! 문서 컨트롤 사슬을 조회하는 읽기 전용 CLI.
//!
//! `src/bin/*.rs` 자동 인식이라 Cargo.toml 을 건드리지 않는다. 본 CLI(`src/main.rs`)와
//! 경합하지 않도록 공개 조회 API 만 부른다 — `apply_`/`insert_`/`delete_`/`set_*` 금지.

use rhwp::document_core::DocumentCore;
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;

const EXIT_OK: i32 = 0;
const EXIT_RUNTIME: i32 = 1;
const EXIT_USAGE: i32 = 2;
const USAGE: &str = "rhwp-q-objects <파일> [--json]";
const UNTRUSTED: &[&str] = &["source", "controls"];

fn envelope(command: &str, payload: Value, untrusted: &[&str]) -> Value {
    let mut out = json!({
        "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
        "tool": "rhwp-q-objects",
        "command": command,
        "version": rhwp::version(),
        "untrustedContent": !untrusted.is_empty(),
        "untrustedFields": untrusted,
    });
    if let (Some(dst), Some(src)) = (out.as_object_mut(), payload.as_object()) {
        for (key, value) in src {
            dst.insert(key.clone(), value.clone());
        }
    }
    out
}

fn write_stdout(text: &str) -> i32 {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    if let Err(e) = writeln!(lock, "{text}") {
        eprintln!("오류: stdout 쓰기 실패 - {e}");
        return EXIT_RUNTIME;
    }
    EXIT_OK
}

fn parse_args(args: &[String]) -> Result<(bool, String), i32> {
    let mut json = false;
    let mut path: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return Err(EXIT_USAGE);
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return Err(EXIT_USAGE);
                }
                path = Some(other.to_string());
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return Err(EXIT_USAGE);
    };
    Ok((json, path))
}

fn inspect(path: &str) -> Result<Value, i32> {
    let data = std::fs::read(path).map_err(|e| {
        eprintln!("오류: 파일을 읽을 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?;
    let core = DocumentCore::from_bytes(&data).map_err(|e| {
        eprintln!("오류: 문서를 열 수 없습니다 - {path}: {e}");
        EXIT_RUNTIME
    })?;
    let controls: Value = serde_json::from_str(&core.controls_json()).map_err(|e| {
        eprintln!("오류: 컨트롤 JSON 이 깨졌습니다 - {e}");
        EXIT_RUNTIME
    })?;
    let objects: Value = serde_json::from_str(&core.objects_json()).map_err(|e| {
        eprintln!("오류: 개체 JSON 이 깨졌습니다 - {e}");
        EXIT_RUNTIME
    })?;
    Ok(envelope(
        "objects",
        json!({
            "source": path,
            "controlCount": controls.as_array().map(Vec::len).unwrap_or(0),
            "objectCount": objects.as_array().map(Vec::len).unwrap_or(0),
            "controls": controls,
            "objects": objects,
        }),
        UNTRUSTED,
    ))
}

fn print_text(report: &Value) -> i32 {
    let source = report["source"].as_str().unwrap_or("");
    let controls = report["controls"].as_array();
    let objects = report["objects"].as_array();
    let mut lines = Vec::new();
    lines.push(source.to_string());
    lines.push(format!("controls\t{}", controls.map(Vec::len).unwrap_or(0)));
    if let Some(items) = controls {
        for (i, item) in items.iter().enumerate() {
            lines.push(format!(
                "{i}\t{}\t{}\t{}",
                item["ctrlId"].as_str().unwrap_or(""),
                item["userDesc"].as_str().unwrap_or(""),
                item["ctrlCh"]
            ));
        }
    }
    lines.push(format!("objects\t{}", objects.map(Vec::len).unwrap_or(0)));
    if let Some(items) = objects {
        for (i, item) in items.iter().enumerate() {
            lines.push(format!(
                "{i}\t{}\tpara={}\tctrl={}",
                item["kind"].as_str().unwrap_or(""),
                item["para"],
                item["controlIndex"]
            ));
        }
    }
    write_stdout(&lines.join("\n"))
}

fn run(args: Vec<String>) -> i32 {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return write_stdout(USAGE);
    }
    let (json, path) = match parse_args(&args) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let report = match inspect(&path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(text) => write_stdout(&text),
            Err(e) => {
                eprintln!("오류: JSON 직렬화 실패 - {e}");
                EXIT_RUNTIME
            }
        }
    } else {
        print_text(&report)
    }
}

fn main() {
    std::process::exit(run(std::env::args().skip(1).collect()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_path(rel: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(rel)
            .to_string_lossy()
            .into_owned()
    }

    fn sample() -> String {
        let form = repo_path("samples/form-01.hwp");
        if PathBuf::from(&form).is_file() {
            form
        } else {
            repo_path("samples/hwp_table_test.hwp")
        }
    }

    #[test]
    fn help_exits_0() {
        assert_eq!(run(vec!["--help".into()]), EXIT_OK);
        assert_eq!(run(vec!["-h".into()]), EXIT_OK);
    }

    #[test]
    fn unknown_flag_exits_2() {
        assert_eq!(run(vec!["--nope".into()]), EXIT_USAGE);
        assert_eq!(run(vec![sample(), "--weird".into()]), EXIT_USAGE);
    }

    #[test]
    fn missing_path_exits_2() {
        assert_eq!(run(vec![]), EXIT_USAGE);
        assert_eq!(run(vec!["--json".into()]), EXIT_USAGE);
    }

    #[test]
    fn extra_file_exits_2() {
        assert_eq!(run(vec![sample(), sample()]), EXIT_USAGE);
    }

    #[test]
    fn missing_file_exits_1() {
        assert_eq!(
            run(vec![repo_path("samples/__no_such_q_objects__.hwp")]),
            EXIT_RUNTIME
        );
    }

    #[test]
    fn unparseable_file_exits_1() {
        assert_eq!(run(vec![repo_path("README.md")]), EXIT_RUNTIME);
    }

    #[test]
    fn sample_envelope_is_read_only_query() {
        let path = sample();
        let report = inspect(&path).expect("표본 문서를 열 수 있어야 한다");
        assert_eq!(
            report["schemaVersion"],
            json!(rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION)
        );
        assert_eq!(report["tool"], "rhwp-q-objects");
        assert_eq!(report["command"], "objects");
        assert_eq!(report["version"], rhwp::version());
        assert_eq!(report["untrustedContent"], true);
        assert_eq!(report["untrustedFields"], json!(["source", "controls"]));
        assert_eq!(report["source"], path);
        let controls = report["controls"].as_array().expect("controls 배열");
        assert_eq!(report["controlCount"], controls.len());
        assert!(!controls.is_empty(), "표본은 컨트롤 사슬이 비면 안 된다");
        assert!(controls.iter().all(|item| {
            item.get("ctrlId").is_some()
                && item.get("ctrlCh").is_some()
                && item.get("userDesc").is_some()
        }));
        let objects = report["objects"].as_array().expect("objects 배열");
        assert_eq!(report["objectCount"], objects.len());
        assert_eq!(run(vec!["--json".into(), path]), EXIT_OK);
    }
}
