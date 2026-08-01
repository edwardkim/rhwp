//! [#3696] `export-doclang --json` — DocLang XML 산출의 기계 계약 (#3608 1-C).
//! 계약 모양은 산출물 축(#3596)·export-hml(#3616)과 동일: 동작 무변경,
//! `--json` 에서만 stdout 순수 JSON, 실패 경로 stdout 비움.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

// doclang 은 HWP5/HWPX 입력 전용이라 HWP3 공용 샘플 대신 HWP5 샘플을 쓴다 (#3359 규약).
const SAMPLE: &str = "samples/para-001.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-dclgjson-{tag}-{}-{}.dclg.xml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// 산출 XML 을 quick-xml 로 끝까지 재파싱해 well-formed 임을 확인하고, 루트가
/// `<doclang version="0.6">` 인지 실측한다 — 매니페스트가 가리키는 산출물이
/// 실제로 소비 가능하다는 증적 (doclang_export.rs 의 최소 불변식 재사용).
fn assert_consumable_doclang(path: &Path) {
    let xml = std::fs::read_to_string(path).expect("산출 XML 읽기");
    let mut reader = quick_xml::Reader::from_str(&xml);
    let mut buf = Vec::new();
    let mut root_checked = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) if !root_checked => {
                assert_eq!(e.name().as_ref(), b"doclang", "루트 요소");
                let version = e
                    .attributes()
                    .flatten()
                    .find(|a| a.key.as_ref() == b"version")
                    .expect("version 속성");
                assert_eq!(version.value.as_ref(), b"0.6", "DocLang 버전");
                root_checked = true;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("산출 XML 이 well-formed 가 아닙니다: {e}"),
        }
        buf.clear();
    }
    assert!(root_checked, "doclang 루트 요소가 없습니다");
}

#[test]
fn export_doclang_json_envelope() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("env");
    let args = [
        "export-doclang",
        p.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout 순수 JSON 아님 ({e})\n{}", describe(&args, &output)));
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "doclang", "{v}");
    assert_eq!(v["doclangVersion"], "0.6", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["output"].as_str(), out.to_str(), "{v}");
    let bytes = v["bytes"].as_u64().expect("bytes");
    let meta = std::fs::metadata(&out).expect("산출물 실존");
    assert_eq!(meta.len(), bytes, "보고 bytes ≠ 실제 크기");
    // 기본(인라인 자원) 모드: assetsDir 는 null, assetCount 는 0.
    assert!(v["assetsDir"].is_null(), "{v}");
    assert_eq!(v["assetCount"], 0, "{v}");
    assert!(v["lossCount"].is_u64(), "{v}");

    // 산출물이 실제로 소비 가능해야 한다 (DocLang 소비 파이프라인 실측).
    assert_consumable_doclang(&out);

    let _ = std::fs::remove_file(&out);
}

#[test]
fn export_doclang_json_runtime_failure_keeps_stdout_empty() {
    let out = temp_path("fail");
    let args = [
        "export-doclang",
        "없는파일-dclgjson.hwp",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다\n{}",
        describe(&args, &output)
    );
}

#[test]
fn capabilities_reports_export_doclang_json() {
    let output = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("caps");
    let entry = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "export-doclang")
        .expect("export-doclang 등재");
    assert_eq!(entry["json"], true, "{entry}");

    let mcp = run(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("mcp");
    assert!(
        m["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "hwp_export_doclang"),
        "hwp_export_doclang 누락"
    );
}
