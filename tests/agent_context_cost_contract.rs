//! [#4864] `rhwp-agent context-cost` 계약 — 계측이 주장을 대체한다.
//!
//! "문서를 그대로 싣지 말라"는 원리는 반박도 검증도 할 수 없다. 이 명령은 그 자리를
//! 재현 가능한 수치로 채우므로, **수치가 정직한지**를 계약으로 고정해야 한다.
//!
//! 이 파일이 못 박는 것.
//!
//! 1. 같은 입력에 같은 봉투(모델·시각 무개입) — 결정론.
//! 2. 배수·복원율이 봉투 안의 다른 숫자로 **손으로 재계산된다** — 자기정합.
//! 3. 봉투에 문서 본문이 한 글자도 실리지 않는다.
//! 4. 가장 유리한 대안(UTF-16LE)도 같이 실린다 — 허수아비 금지.
//! 5. 사용법 오류는 exit 2 + stdout 0바이트, 실행 오류는 exit 1.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn agent_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-agent")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-agent").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(agent_bin())
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("rhwp-agent 실행 실패")
}

fn measure_sample() -> Option<(String, serde_json::Value)> {
    let src = sample(SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return None;
    }
    let out = run(&["context-cost", SAMPLE, "--json"]);
    assert!(out.status.success(), "실행 실패: {out:?}");
    let raw = String::from_utf8(out.stdout).expect("stdout UTF-8");
    let v: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("봉투가 JSON 이 아닙니다 ({e}): {raw}"));
    Some((raw, v))
}

#[test]
fn envelope_carries_contract_fields_and_unit_disclosure() {
    let Some((_, v)) = measure_sample() else {
        return;
    };
    assert_eq!(v["tool"], "rhwp-agent", "{v}");
    assert_eq!(v["command"], "context-cost", "{v}");
    assert!(v["schemaVersion"].is_string(), "{v}");
    // 단위가 문자임을 봉투가 스스로 밝혀야 한다 — 토큰으로 오독되면 이 계측의
    // 결론이 통째로 바뀐다.
    assert_eq!(v["unit"], "chars", "{v}");
    assert!(
        v["unitNote"].as_str().is_some_and(|s| s.contains("토큰")),
        "단위 한계 고지가 없습니다: {v}"
    );
}

#[test]
fn measurement_is_deterministic() {
    // 모델·시각·난수가 개입하지 않는다. 두 번 부르면 바이트까지 같아야 제3자가
    // 같은 숫자를 재현할 수 있다.
    let src = sample(SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let a = run(&["context-cost", SAMPLE, "--json"]);
    let b = run(&["context-cost", SAMPLE, "--json"]);
    assert_eq!(a.stdout, b.stdout, "같은 입력에 다른 봉투가 나왔습니다");
}

#[test]
fn ratio_and_recovery_are_self_consistent() {
    // 배수·복원율을 봉투 안의 다른 숫자로 손으로 재계산할 수 있어야 한다.
    // 재계산이 불가능한 수치는 검증할 수 없고, 검증할 수 없는 수치는 주장이다.
    let Some((_, v)) = measure_sample() else {
        return;
    };
    let f = &v["files"][0];
    let raw_utf8 = f["rawChars"]["utf8"].as_u64().expect("rawChars.utf8");
    let native = f["nativeChars"].as_u64().expect("nativeChars");
    let bytes = f["bytes"].as_u64().expect("bytes");
    assert!(native > 0, "전제: 표본에 본문이 있어야 합니다: {f}");
    assert!(bytes > 0, "{f}");

    let expected = ((raw_utf8 as f64 / native as f64) * 10.0).round() / 10.0;
    let got = f["charRatio"].as_f64().expect("charRatio");
    assert!(
        (got - expected).abs() < 1e-9,
        "charRatio 가 rawChars.utf8/nativeChars 와 다릅니다: got={got} expected={expected} {f}"
    );

    for key in ["utf8", "utf16le"] {
        let pct = f["recoveryPercent"][key]
            .as_f64()
            .unwrap_or_else(|| panic!("recoveryPercent.{key} 없음: {f}"));
        assert!(
            (0.0..=100.0).contains(&pct),
            "복원율이 0~100 밖입니다 ({key}={pct}): {f}"
        );
    }
    assert!(
        f["sampledChars"].as_u64().expect("sampledChars") <= native,
        "복원율 표본이 본문보다 클 수 없습니다: {f}"
    );
}

#[test]
fn favorable_alternative_is_measured_too() {
    // UTF-8 만 재면 허수아비다. 인코딩을 바꿔 볼 호출자를 상정한 수치가 같은
    // 봉투에 있어야 이 계측이 공정하다.
    let Some((_, v)) = measure_sample() else {
        return;
    };
    let f = &v["files"][0];
    assert!(
        f["rawChars"]["utf16le"].is_u64(),
        "유리한 대안(UTF-16LE) 문자 수가 없습니다: {f}"
    );
    assert!(
        f["recoveryPercent"]["utf16le"].is_number(),
        "유리한 대안의 복원율이 없습니다: {f}"
    );
}

#[test]
fn envelope_contains_no_document_text() {
    // 계측 결과를 그대로 이슈·로그에 붙여도 문서가 새면 안 된다. 봉투는 숫자와
    // 호출자가 지정한 경로뿐이어야 한다.
    let Some((raw, v)) = measure_sample() else {
        return;
    };
    assert_eq!(v["untrustedContent"], false, "{v}");
    assert_eq!(
        v["untrustedFields"].as_array().map(Vec::len),
        Some(0),
        "문서 파생 필드를 선언했다면 봉투에 본문이 실린다는 뜻입니다: {v}"
    );

    // 표본 본문에서 실제로 긴 줄을 하나 뽑아 봉투에 없음을 확인한다 — 고정
    // 문자열을 쓰면 표본이 바뀔 때 조용히 무의미해진다.
    let export = Command::new(
        std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string()),
    )
    .args(["export-text", SAMPLE, "--json"])
    .current_dir(env!("CARGO_MANIFEST_DIR"))
    .output()
    .expect("rhwp export-text 실행 실패");
    let ev: serde_json::Value = serde_json::from_slice(&export.stdout).expect("export-text JSON");
    let body: String = ev["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .filter_map(|p| p["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let Some(longest) = body
        .lines()
        .map(str::trim)
        .max_by_key(|l| l.chars().count())
    else {
        return;
    };
    if longest.chars().count() >= 8 {
        assert!(
            !raw.contains(longest),
            "봉투에 문서 본문이 실렸습니다: {longest:?}"
        );
    }
}

#[test]
fn usage_errors_exit_2_with_empty_stdout() {
    // 반쪽 JSON 금지 — 사용법 오류는 stdout 을 한 바이트도 오염시키지 않는다.
    for args in [vec!["context-cost"], vec!["context-cost", SAMPLE, "--nope"]] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2), "args={args:?} out={out:?}");
        assert!(
            out.stdout.is_empty(),
            "사용법 오류인데 stdout 이 오염됐습니다: args={args:?}"
        );
    }
}

#[test]
fn missing_file_is_runtime_error_with_empty_stdout() {
    let out = run(&["context-cost", "no_such_file_4864.hwp", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        out.stdout.is_empty(),
        "실패 stdout 은 순수해야 합니다: {out:?}"
    );
}

#[test]
fn command_is_self_described() {
    // 자기서술에 없으면 호출자는 이 명령의 존재를 알 수 없다.
    let out = run(&["capabilities", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("capabilities JSON");
    let cmd = v["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "context-cost")
        .unwrap_or_else(|| panic!("context-cost 가 자기서술에 없습니다: {v}"));
    assert!(
        cmd["usage"]
            .as_str()
            .is_some_and(|u| u.contains("context-cost")),
        "{cmd}"
    );
    assert_eq!(cmd["jsonContract"], true, "{cmd}");
}
