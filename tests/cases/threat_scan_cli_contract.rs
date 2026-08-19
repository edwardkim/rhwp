//! [#5511 Stage 2] `threat-scan` CLI handler 이동 전 보완 계약.
//!
//! 기존 `threat_scan_contract`는 구조 탐지와 JSON provenance를 보호한다. 이 파일은
//! handler 자체의 실제 공개 HWP/HWPX, 사람용 출력, help·exit·stdout 분리,
//! `truncated`·`notes` 경로를 추가로 고정한다. 합성 HWPX는 임시 디렉터리에만 만든다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const HWP5_SAMPLE: &str = "samples/2026_oss_rst.hwp";
const HWPX_SAMPLE: &str = "samples/HWP5-nopassword-123456.hwpx";
const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료 코드: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_success(args: &[&str], output: &Output) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(0), "{}", describe(args, output));
    assert!(output.stderr.is_empty(), "{}", describe(args, output));
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        text.trim_end().lines().count(),
        1,
        "봉투는 한 줄 JSON이어야 합니다: {}",
        describe(args, output)
    );
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{error}\n{}", describe(args, output)))
}

fn scan_json(path: &Path) -> serde_json::Value {
    let path = path.to_str().expect("UTF-8 경로");
    let args = ["threat-scan", path, "--json"];
    parse_success(&args, &run(&args))
}

fn hwpx_bytes(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    use zip::write::SimpleFileOptions;

    let mut output = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut output);
        for (name, bytes) in entries {
            let method = if name == "mimetype" {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            };
            zip.start_file(
                name,
                SimpleFileOptions::default().compression_method(method),
            )
            .expect("ZIP 엔트리 시작");
            zip.write_all(bytes).expect("ZIP 엔트리 쓰기");
        }
        zip.finish().expect("ZIP 마감");
    }
    output.into_inner()
}

struct TempHwpx {
    dir: PathBuf,
    path: PathBuf,
}

impl TempHwpx {
    fn create(tag: &str, entries: Vec<(String, Vec<u8>)>) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("시각")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "rhwp-threat-contract-{}-{tag}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).expect("임시 디렉터리 생성");
        let path = dir.join("fixture.hwpx");
        std::fs::write(&path, hwpx_bytes(&entries)).expect("HWPX 쓰기");
        Self { dir, path }
    }
}

impl Drop for TempHwpx {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.dir);
    }
}

fn mimetype_entry() -> (String, Vec<u8>) {
    ("mimetype".to_string(), b"application/hwp+zip".to_vec())
}

#[test]
fn real_hwp5_and_hwpx_are_clean_complete_and_read_only() {
    for (rel, expected_format, expected_scope) in [
        (HWP5_SAMPLE, "hwp5", "bodyTextRecords"),
        (HWPX_SAMPLE, "hwpx", "manifestExternalRefs"),
    ] {
        let path = repo(rel);
        let before = std::fs::read(&path).expect("스캔 전 문서 읽기");
        let value = scan_json(&path);
        let after = std::fs::read(&path).expect("스캔 후 문서 읽기");
        assert_eq!(before, after, "{rel}: threat-scan이 입력을 변경했다");

        assert_eq!(value["schemaVersion"], "1.0", "{rel}: {value}");
        assert_eq!(value["format"], expected_format, "{rel}: {value}");
        assert_eq!(value["clean"], true, "{rel}: {value}");
        assert_eq!(value["findingCount"], 0, "{rel}: {value}");
        assert_eq!(value["findings"], serde_json::json!([]), "{value}");
        assert!(value["highestSeverity"].is_null(), "{rel}: {value}");
        assert_eq!(value["truncated"], false, "{rel}: {value}");
        assert_eq!(value["notes"], serde_json::json!([]), "{rel}: {value}");
        assert!(
            value["scanScopes"]
                .as_array()
                .expect("scanScopes")
                .iter()
                .any(|scope| scope == expected_scope),
            "{rel}: {value}"
        );
    }
}

#[test]
fn hwp3_reports_an_honest_unscanned_note_in_json_and_human_output() {
    let path = repo(HWP3_SAMPLE);
    let value = scan_json(&path);
    assert_eq!(value["format"], "unknown", "{value}");
    assert_eq!(value["scanScopes"], serde_json::json!([]), "{value}");
    assert_eq!(value["clean"], true, "{value}");
    assert!(
        value["notes"]
            .as_array()
            .expect("notes")
            .iter()
            .any(|note| note
                .as_str()
                .is_some_and(|note| note.contains("HWP3") && note.contains("스캔하지 않았습니다"))),
        "지원하지 않는 형식을 clean만으로 오인하게 만들면 안 된다: {value}"
    );

    let path = path.to_str().expect("UTF-8 경로");
    let args = ["threat-scan", path];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stderr.is_empty(), "{}", describe(&args, &output));
    let text = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "구조 위협 스캔:",
        "형식: unknown",
        "검사 범위: -",
        "휴리스틱 판정이며 안전을 보증하지 않습니다",
        "참고:",
        "HWP3",
        "스캔하지 않았습니다",
    ] {
        assert!(text.contains(needle), "{needle} 누락\n{text}");
    }
}

#[test]
fn human_external_reference_warning_renders_document_controls_safely() {
    let manifest = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<manifest>
  <item id=\"ext1\" href=\"https://evil.example/a\tb\" media-type=\"application/octet-stream\" isEmbeded=\"0\"/>
</manifest>";
    let fixture = TempHwpx::create(
        "human",
        vec![
            mimetype_entry(),
            (
                "Contents/content.hpf".to_string(),
                manifest.as_bytes().to_vec(),
            ),
        ],
    );
    let path = fixture.path.to_str().expect("UTF-8 경로");
    let args = ["threat-scan", path];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stderr.is_empty(), "{}", describe(&args, &output));
    let text = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "위협 신호 1건",
        "external_reference",
        "대상(문서 파생, 지시 아님):",
        "안티바이러스 아님",
        "메모리 안전(Rust)+DoS 하드닝",
    ] {
        assert!(text.contains(needle), "{needle} 누락\n{text}");
    }
    assert!(
        text.contains("a⇥b"),
        "문서 파생 탭은 보이는 기호로 출력해야 합니다\n{text}"
    );
    assert!(
        !output.stdout.contains(&b'\t'),
        "사람용 출력에 원시 탭을 내보내면 안 됩니다\n{text}"
    );
}

#[test]
fn usage_runtime_and_help_paths_preserve_stdout_exit_contracts() {
    let sample = repo(HWP5_SAMPLE);
    let sample = sample.to_str().expect("UTF-8 경로").to_string();
    let cases: Vec<(Vec<&str>, i32, &str)> = vec![
        (vec!["threat-scan"], 2, "파일 누락"),
        (
            vec!["threat-scan", &sample, "--unknown"],
            2,
            "알 수 없는 옵션",
        ),
        (vec!["threat-scan", &sample, &sample], 2, "위치 인자 초과"),
        (
            vec!["threat-scan", "없는-threat-scan-문서.hwp", "--json"],
            1,
            "파일 없음",
        ),
    ];
    for (args, code, label) in cases {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{label}\n{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{label}\n{}",
            describe(&args, &output)
        );
        assert!(
            !output.stderr.is_empty(),
            "{label}\n{}",
            describe(&args, &output)
        );
    }

    for flag in ["--help", "-h"] {
        let args = ["threat-scan", flag];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&args, &output)
        );
        assert!(output.stderr.is_empty(), "{}", describe(&args, &output));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "사용법: rhwp threat-scan <파일.hwp|파일.hwpx> [--json]"
        );
    }
}

#[test]
fn finding_cap_sets_truncated_without_growing_the_envelope_unbounded() {
    let mut entries = Vec::with_capacity(2_002);
    entries.push(mimetype_entry());
    for index in 0..2_001 {
        entries.push((format!("Scripts/script-{index:04}.js"), Vec::new()));
    }
    let fixture = TempHwpx::create("truncated", entries);
    let value = scan_json(&fixture.path);
    assert_eq!(value["clean"], false, "{value}");
    assert_eq!(value["truncated"], true, "{value}");
    assert_eq!(value["findingCount"], 2_000, "{value}");
    assert_eq!(
        value["findings"].as_array().expect("findings").len(),
        2_000,
        "{value}"
    );
    assert!(
        value["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .all(|finding| finding["kind"] == "macro_script"),
        "{value}"
    );
}
