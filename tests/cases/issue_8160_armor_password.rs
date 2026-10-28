//! #8160: 암호화 HWP5의 armor stdin·전체 응답 봉투 계약 복원.
#![cfg(not(target_arch = "wasm32"))]
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
const PASSWORD: &str = "123456";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run_with_stdin(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin)
        .expect("비밀번호 쓰기");
    child.wait_with_output().expect("rhwp 종료 대기")
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

fn assert_complete_envelope(value: &serde_json::Value, expected_pages: Option<u64>) {
    assert_eq!(value["schemaVersion"], "1.0", "{value}");
    assert!(value["source"].is_string(), "{value}");
    if let Some(expected_pages) = expected_pages {
        assert_eq!(value["pageCount"], expected_pages, "{value}");
    } else {
        assert!(value["pageCount"].as_u64().is_some_and(|pages| pages > 0));
    }
    assert_eq!(value["scanScopes"].as_array().map(Vec::len), Some(9));
    assert!(value["injectionSignals"].is_array(), "{value}");
    assert!(value["signalCount"].is_u64(), "{value}");
    assert!(value["clean"].is_boolean(), "{value}");
    assert_eq!(value["untrustedContent"], true, "{value}");
    assert!(
        value["untrustedFields"]
            .as_array()
            .is_some_and(|fields| fields.iter().any(|field| field == "armoredText")),
        "{value}"
    );

    let armored = value["armoredText"].as_str().expect("armoredText");
    let open = value["safety"]["fenceOpen"].as_str().expect("fenceOpen");
    let close = value["safety"]["fenceClose"].as_str().expect("fenceClose");
    assert!(armored.starts_with(open), "{value}");
    assert!(armored.ends_with(close), "{value}");
}

#[test]
fn issue_8160_encrypted_hwp5_armor_preserves_password_stdin_and_complete_envelope() {
    let rel = "samples/hwp3-sample16-hwp5-2024-password-123456.hwp";
    let pages = Some(64);
    let path = repo(rel);
    let path = path.to_str().expect("암호 문서 경로");
    let args = ["--password-stdin", "armor", path, "--json"];
    let output = run_with_stdin(&args, format!("{PASSWORD}\n").as_bytes());
    let value = parse_success(&args, &output);
    assert_complete_envelope(&value, pages);
}
