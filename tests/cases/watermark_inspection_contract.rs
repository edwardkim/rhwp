//! [#5511 Stage 2] `inspect watermark` CLI characterization 계약.
//!
//! 탐지 코어의 단위 테스트만으로는 handler 이동 중 인자·문서 순회·봉투·암호·출력 계약이
//! 바뀌는 것을 잡지 못한다. 실제 HWP5를 편집해 세 탐지축을 심고 HWPX로도 변환하여,
//! 공개 CLI의 현재 동작을 이동 전에 고정한다. 합성 문서는 임시 디렉터리에만 만든다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/2026_oss_rst.hwp";
const HWP3_SAMPLE: &str = "samples/hwp3-sample.hwp";
const PASSWORD_SAMPLE: &str = "samples/HWP5-password-123456.hwpx";
const ANCHOR: &str = "제출 방법";
const PASSWORD: &str = "123456";
const WRONG_PASSWORD: &str = "wrong-password-must-not-echo";

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
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}\n{}", describe(args, output)))
}

fn inspect_json(path: &Path, extra: &[&str]) -> serde_json::Value {
    let p = path.to_str().expect("UTF-8 경로");
    let mut args = vec!["inspect", "watermark", p, "--json"];
    args.extend_from_slice(extra);
    let output = run(&args);
    parse_success(&args, &output)
}

fn zero_width_hi() -> String {
    "0100100001101001"
        .chars()
        .map(|bit| if bit == '0' { '\u{200B}' } else { '\u{200C}' })
        .collect()
}

struct AttackDocuments {
    dir: PathBuf,
    hwp: PathBuf,
    hwpx: PathBuf,
}

impl AttackDocuments {
    fn create(tag: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("시각")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "rhwp-watermark-contract-{}-{tag}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).expect("임시 디렉터리 생성");
        let hwp = dir.join("attack.hwp");
        let hwpx = dir.join("attack.hwpx");
        let payload = format!("\u{0422}otal{}결과\t \t 확인", zero_width_hi());

        let source = repo(SAMPLE);
        let source = source.to_str().expect("샘플 경로");
        let hwp_arg = hwp.to_str().expect("HWP 출력 경로");
        let edit_args = [
            "edit",
            "replace-text",
            source,
            "--find",
            ANCHOR,
            "--replace",
            payload.as_str(),
            "-o",
            hwp_arg,
            "--json",
        ];
        let edited = run(&edit_args);
        assert_eq!(
            edited.status.code(),
            Some(0),
            "공격 HWP 생성 실패\n{}",
            describe(&edit_args, &edited)
        );

        let hwpx_arg = hwpx.to_str().expect("HWPX 출력 경로");
        let export_args = ["export-hwpx", hwp_arg, hwpx_arg, "--json"];
        let exported = run(&export_args);
        assert_eq!(
            exported.status.code(),
            Some(0),
            "공격 HWPX 생성 실패\n{}",
            describe(&export_args, &exported)
        );
        assert!(hwp.is_file() && hwpx.is_file(), "합성 문서 누락");

        Self { dir, hwp, hwpx }
    }
}

impl Drop for AttackDocuments {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.hwp);
        let _ = std::fs::remove_file(&self.hwpx);
        let _ = std::fs::remove_dir(&self.dir);
    }
}

fn finding_kinds(value: &serde_json::Value) -> BTreeSet<&str> {
    value["findings"]
        .as_array()
        .expect("findings 배열")
        .iter()
        .map(|finding| finding["kind"].as_str().expect("finding kind"))
        .collect()
}

#[test]
fn clean_documents_report_a_complete_empty_envelope() {
    for rel in [SAMPLE, HWP3_SAMPLE] {
        let path = repo(rel);
        let value = inspect_json(&path, &[]);
        assert_eq!(value["schemaVersion"], "1.0", "{rel}: {value}");
        assert_eq!(value["kindFilter"], "all", "{rel}: {value}");
        assert_eq!(value["clean"], true, "{rel}: {value}");
        assert_eq!(value["findingCount"], 0, "{rel}: {value}");
        assert_eq!(value["findings"], serde_json::json!([]), "{rel}: {value}");
        assert!(
            value["scannedChars"].as_u64().unwrap_or(0) > 0,
            "{rel}: 실제 문자를 훑어야 한다 — {value}"
        );
        assert_eq!(
            value["severityCounts"],
            serde_json::json!({"high": 0, "medium": 0, "low": 0}),
            "{rel}: {value}"
        );
        assert_eq!(
            value["kindCounts"],
            serde_json::json!({"hidden_char": 0, "homoglyph": 0, "whitespace": 0}),
            "{rel}: {value}"
        );
    }
}

#[test]
fn all_three_axes_are_detected_in_real_hwp_and_hwpx_documents() {
    let docs = AttackDocuments::create("three-axes");
    for (format, path) in [("hwp", &docs.hwp), ("hwpx", &docs.hwpx)] {
        let value = inspect_json(path, &[]);
        assert_eq!(value["clean"], false, "{format}: {value}");
        assert_eq!(value["findingCount"], 3, "{format}: {value}");
        assert_eq!(
            finding_kinds(&value),
            BTreeSet::from(["hidden_char", "homoglyph", "whitespace"]),
            "{format}: {value}"
        );
        assert_eq!(
            value["kindCounts"],
            serde_json::json!({"hidden_char": 1, "homoglyph": 1, "whitespace": 1}),
            "{format}: {value}"
        );
        assert_eq!(
            value["severityCounts"],
            serde_json::json!({"high": 1, "medium": 1, "low": 1}),
            "{format}: {value}"
        );
        let findings = value["findings"].as_array().expect("findings");
        assert!(
            findings.iter().any(|finding| finding["detail"]
                .as_str()
                .is_some_and(|detail| detail.contains(r#"ASCII "Hi""#))),
            "제로폭 payload 복호 누락: {format}: {value}"
        );
        assert!(
            findings.iter().all(|finding| finding["location"]
                .as_str()
                .is_some_and(|location| location.starts_with("cell["))),
            "표 셀 중첩 위치가 보존되어야 한다: {format}: {value}"
        );
    }
}

#[test]
fn kind_filter_partitions_the_findings_exactly() {
    let docs = AttackDocuments::create("filters");
    for (filter, expected_kind) in [
        ("hidden", "hidden_char"),
        ("homoglyph", "homoglyph"),
        ("whitespace", "whitespace"),
    ] {
        let value = inspect_json(&docs.hwp, &["--kind", filter]);
        assert_eq!(value["kindFilter"], filter, "{value}");
        assert_eq!(value["findingCount"], 1, "{value}");
        assert_eq!(
            finding_kinds(&value),
            BTreeSet::from([expected_kind]),
            "{value}"
        );
    }
    let all = inspect_json(&docs.hwp, &["--kind", "all"]);
    assert_eq!(all["kindFilter"], "all", "{all}");
    assert_eq!(all["findingCount"], 3, "{all}");
}

#[test]
fn human_output_exposes_decoded_payload_and_nested_location() {
    let docs = AttackDocuments::create("human");
    let path = docs.hwp.to_str().expect("경로");
    let args = ["inspect", "watermark", path];
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
        "숨은 마크 검사",
        "탐지 3건",
        r#"ASCII "Hi""#,
        "cell[0:0].para[3]",
    ] {
        assert!(text.contains(needle), "{needle} 누락\n{text}");
    }
}

#[test]
fn failures_keep_stdout_empty_and_preserve_exit_classes() {
    let sample = repo(SAMPLE);
    let sample = sample.to_str().expect("경로").to_string();
    let cases: Vec<(Vec<&str>, i32, &str)> = vec![
        (vec!["inspect", "watermark"], 2, "파일 경로 누락"),
        (
            vec!["inspect", "watermark", &sample, "--kind"],
            2,
            "kind 값 누락",
        ),
        (
            vec!["inspect", "watermark", &sample, "--kind", "없는축"],
            2,
            "알 수 없는 kind",
        ),
        (
            vec!["inspect", "watermark", &sample, "--wat"],
            2,
            "알 수 없는 옵션",
        ),
        (
            vec!["inspect", "watermark", &sample, &sample],
            2,
            "위치 인자 초과",
        ),
        (
            vec!["inspect", "watermark", "없는-watermark-문서.hwp", "--json"],
            1,
            "파일 없음",
        ),
    ];
    for (args, code, why) in cases {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{why}\n{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{why}\n{}",
            describe(&args, &output)
        );
        assert!(
            !output.stderr.is_empty(),
            "{why}\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn scanning_never_modifies_the_input_document() {
    let docs = AttackDocuments::create("immutable");
    let before = std::fs::read(&docs.hwp).expect("스캔 전 읽기");
    for extra in [
        vec![],
        vec!["--kind", "hidden"],
        vec!["--kind", "homoglyph"],
        vec!["--kind", "whitespace"],
    ] {
        let _ = inspect_json(&docs.hwp, &extra);
    }
    let path = docs.hwp.to_str().expect("경로");
    let human = run(&["inspect", "watermark", path]);
    assert_eq!(human.status.code(), Some(0));
    let after = std::fs::read(&docs.hwp).expect("스캔 후 읽기");
    assert_eq!(before, after, "inspect watermark가 입력 문서를 변경했다");
}

#[test]
fn password_paths_preserve_success_usage_and_runtime_exit_contracts() {
    let fixture = repo(PASSWORD_SAMPLE);
    let path = fixture.to_str().expect("암호 문서 경로");

    let no_password_args = ["inspect", "watermark", path, "--json"];
    let no_password = run(&no_password_args);
    assert_eq!(
        no_password.status.code(),
        Some(2),
        "{}",
        describe(&no_password_args, &no_password)
    );
    assert!(no_password.stdout.is_empty());

    let wrong_args = [
        "--password",
        WRONG_PASSWORD,
        "inspect",
        "watermark",
        path,
        "--json",
    ];
    let wrong = run(&wrong_args);
    assert_eq!(
        wrong.status.code(),
        Some(1),
        "{}",
        describe(&wrong_args, &wrong)
    );
    assert!(wrong.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&wrong.stderr).contains(WRONG_PASSWORD));

    let success_args = [
        "--password",
        PASSWORD,
        "inspect",
        "watermark",
        path,
        "--json",
    ];
    let success = run(&success_args);
    let value = parse_success(&success_args, &success);
    assert_eq!(value["clean"], true, "{value}");
    assert!(value["scannedChars"].as_u64().unwrap_or(0) > 0, "{value}");
}

#[test]
fn capabilities_declares_the_watermark_contract() {
    let capabilities_args = ["capabilities"];
    let capabilities = parse_success(&capabilities_args, &run(&capabilities_args));
    let inspect = capabilities["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|command| command["name"] == "inspect")
        .expect("inspect command");
    let watermark = inspect["subcommands"]
        .as_array()
        .expect("inspect subcommands")
        .iter()
        .find(|subcommand| subcommand["name"] == "watermark")
        .expect("watermark subcommand");
    assert!(watermark["summary"]
        .as_str()
        .is_some_and(|summary| summary.contains("숨은 마크")));
}
