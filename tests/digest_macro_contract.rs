//! [#3633] `digest` 매크로 계약 테스트 — 초소형 모델용 매크로 도구 축 1호.
//!
//! 계약: `digest --json` 은 한 줄 JSON 봉투(schemaVersion·source·format·pageCount·
//! paraCount·outline·excerpt·truncated·nextStep)를 stdout 에 낸다. `nextStep` 은
//! 고정 문자열이다 — 체이닝을 못 하는 모델이 다음 행동을 지어내지 않고 받아 적는
//! 유도 계약이므로, 문구 변경은 본 테스트가 잡는 의도적 결정이어야 한다.
//! 실패 시 stdout 은 0바이트(소비자는 stdout 만 파싱), 종료 코드는 [#2707] 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 파싱까지 성공하는 실제 샘플 (cli_json_contract.rs 와 동일 원천).
const SAMPLE: &str = "samples/hwp3-sample.hwp";

/// [#3633] nextStep 고정 문자열 계약 — 구현과 문자 그대로 일치해야 한다.
const NEXT_STEP: &str = "더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

// ── ① 봉투 필드·nextStep·절단 계약 ─────────────────────────────────────────

#[test]
fn digest_json_envelope_contract() {
    let sample = sample_path();
    let args = ["digest", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    // 봉투는 한 줄 — 초소형 모델이 줄 단위로 그대로 삼킬 수 있어야 한다.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "봉투는 한 줄이어야 합니다.\n{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["format"], "hwp3", "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["paraCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["outline"].is_array(), "{v}");
    // outline 항목은 문자열(상위 노드 제목) — 트리 전체를 싣지 않는다(컨텍스트 절약).
    for item in v["outline"].as_array().unwrap() {
        assert!(
            item.is_string(),
            "outline 은 제목 문자열 배열이어야 합니다: {v}"
        );
    }
    assert!(v["excerpt"].is_string(), "{v}");
    assert!(!v["excerpt"].as_str().unwrap().is_empty(), "{v}");
    assert!(v["truncated"].is_boolean(), "{v}");
    // nextStep 고정 문자열 계약.
    assert_eq!(v["nextStep"], NEXT_STEP, "{v}");
}

#[test]
fn digest_max_chars_truncates_excerpt() {
    let sample = sample_path();
    let args = [
        "digest",
        "--json",
        "--max-chars",
        "16",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    let excerpt = v["excerpt"].as_str().expect("excerpt 문자열");
    assert!(
        excerpt.chars().count() <= 16,
        "excerpt 는 --max-chars 이하(문자 수 기준)여야 합니다: {v}"
    );
    assert_eq!(
        v["truncated"], true,
        "절단이 일어났으면 truncated=true 여야 합니다: {v}"
    );
    // 절단돼도 nextStep 유도는 살아 있어야 한다.
    assert_eq!(v["nextStep"], NEXT_STEP, "{v}");
}

#[test]
fn digest_invalid_max_chars_exit_usage() {
    let sample = sample_path();
    let args = [
        "digest",
        "--json",
        "--max-chars",
        "코끼리",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

// ── ② 실패 stdout 순수성 ───────────────────────────────────────────────────

#[test]
fn digest_missing_file_exit_runtime_silent_stdout() {
    let args = ["digest", "--json", "없는파일-digest.hwp"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn digest_multiple_files_exit_usage_silent_stdout() {
    let first = sample_path();
    let second = sample_path();
    let args = [
        "digest",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "추가 입력을 조용히 무시하면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

// ── ③ capabilities/MCP 등재 ────────────────────────────────────────────────

#[test]
fn digest_registered_in_capabilities() {
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let commands = cap["commands"].as_array().expect("commands 배열");
    let digest = commands
        .iter()
        .find(|c| c["name"] == "digest")
        .unwrap_or_else(|| panic!("capabilities 에 digest 누락: {cap}"));
    assert_eq!(digest["json"], true, "{digest}");
    assert!(digest["summary"].is_string(), "{digest}");
    let fields = digest["recordFields"].as_array().expect("recordFields");
    for expected in ["outline", "excerpt", "truncated", "nextStep"] {
        assert!(
            fields.iter().any(|f| f == expected),
            "digest recordFields 에 {expected} 누락: {digest}"
        );
    }
}

#[test]
fn digest_registered_in_mcp_with_compact_description() {
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let tools = mcp["tools"].as_array().expect("tools 배열");
    let digest = tools
        .iter()
        .find(|t| t["name"] == "hwp_digest")
        .unwrap_or_else(|| panic!("MCP 도구 hwp_digest 누락: {mcp}"));
    assert_eq!(digest["cli"]["command"], "digest", "{digest}");
    let required = digest["inputSchema"]["required"].as_array().unwrap();
    assert!(required.iter().any(|r| r == "path"), "{digest}");
    // [#3633] 초소형 모델 컨텍스트 절약 계약: 설명은 40자 이내로 극단 압축한다.
    let desc = digest["description"].as_str().expect("description");
    assert!(
        desc.chars().count() <= 40,
        "hwp_digest 설명은 40자 이내여야 합니다 ({}자): {desc}",
        desc.chars().count()
    );
}

#[test]
fn digest_mcp_forwards_optional_max_chars() {
    let sample = sample_path();
    let mut child = Command::new(rhwp_bin())
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp-serve");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "hwp_digest",
            "arguments": { "path": sample, "maxChars": 16 }
        }
    });
    writeln!(stdin, "{request}").unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0, "조기 종료");
    let response: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    let excerpt = response["result"]["structuredContent"]["excerpt"]
        .as_str()
        .unwrap_or_else(|| panic!("MCP digest 실패: {response}"));
    assert!(
        excerpt.chars().count() <= 16,
        "maxChars 가 CLI에 전달돼야 합니다: {response}"
    );
    let _ = child.kill();
    let _ = child.wait();
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
