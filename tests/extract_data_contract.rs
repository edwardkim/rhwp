//! [#3719 §6-10] `extract-data` 출력 계약 회귀 테스트 — 값과 **주소**가 한 몸이다.
//!
//! 이 명령의 존재 이유는 값 옆에 붙는 주소(구역·문단·**쪽**·문자 오프셋)다. 평문을 뽑아
//! 밖에서 정규식을 돌려도 값 자체는 얻지만 근거 제시가 안 된다. 주소가 사라지면 기능
//! 전체가 무의미해지므로 그 정합성을 계약으로 고정한다.
//!
//! 두 번째 축은 **모름의 보존**이다. 두 자리 연도·한글 수사 금액은 정규화할 수 없으므로
//! `normalized: null` 이어야 한다 — 여기서 그럴듯한 값을 지어내면 소비자는 틀린 값을
//! 옳은 값과 구별할 방법이 없다.
//!
//! 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 실제 정부 양식 — 날짜 표기가 `2026. 1.`(연·월)뿐이라 부분 날짜 규약의 실증 대상이다.
const SAMPLE_FORM: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";
/// 실제 예산 문서 — `3,180백만원`·`12.03억원` 처럼 단위 배수와 소수가 섞인 금액이 실재한다.
const SAMPLE_BUDGET: &str = "samples/2022년 국립국어원 업무계획.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
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

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

fn run_ok(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(output.status.code(), Some(0), "{}", describe(args, &output));
    parse_json(args, &output)
}

#[test]
fn year_month_only_notation_is_a_partial_date_not_a_guess() {
    // 이 양식의 유일한 날짜는 `2026. 1.` 이다. 없는 날(日)을 1일로 채우면 조용히 틀린
    // 값이 되므로 부분 날짜 `2026-01` 로 둔다.
    let p = sample(SAMPLE_FORM);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "date",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    let found = items
        .iter()
        .find(|m| m["normalized"] == "2026-01")
        .unwrap_or_else(|| panic!("`2026. 1.` 을 부분 날짜로 뽑지 못했습니다: {v}"));
    assert_eq!(found["raw"], "2026. 1.", "{found}");
    assert!(found["page"].as_u64().is_some(), "쪽 주소 누락: {found}");
}

#[test]
fn decimal_with_a_unit_multiplier_stays_exact() {
    // `12.03억원` = 1,203,000,000. 부동소수 곱셈이면 끝자리가 흔들린다 — 정수 연산이라야
    // 예산 문서의 합계가 맞는다.
    let p = sample(SAMPLE_BUDGET);
    let v = run_ok(&[
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "amount",
        "--json",
    ]);
    let items = v["items"].as_array().expect("items");
    for (raw, expected) in [
        ("3,180백만원", 3_180_000_000i64),
        ("12.03억원", 1_203_000_000),
        ("2.15백만원", 2_150_000),
        ("57억원", 5_700_000_000),
    ] {
        let item = items
            .iter()
            .find(|m| m["raw"] == raw)
            .unwrap_or_else(|| panic!("`{raw}` 을 뽑지 못했습니다: {v}"));
        assert_eq!(
            item["normalized"].as_i64(),
            Some(expected),
            "{raw} 정규화 오류: {item}"
        );
    }
}

#[test]
fn item_count_zero_or_not_the_exit_code_is_success() {
    // 0건은 실패가 아니다 — 1은 런타임 실패 전용이다(#2707). 값이 거의 없는 문서로도
    // 파이프라인이 멈추지 않아야 한다.
    let p = sample("samples/hwp3-sample.hwp");
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let args = ["extract-data", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(
        v["itemCount"].as_u64().unwrap(),
        v["items"].as_array().unwrap().len() as u64,
        "{v}"
    );
}

#[test]
fn default_output_is_human_summary_not_json() {
    let p = sample(SAMPLE_FORM);
    let args = ["extract-data", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "기본 출력은 JSON 이 아니어야 합니다(--json 전용).\n{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_file_exit_runtime_and_silent_stdout() {
    let args = ["extract-data", "없는파일-extract-data.hwp", "--json"];
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
fn missing_path_exit_usage_and_silent_stdout() {
    let args = ["extract-data", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
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
fn invalid_kind_exit_usage_and_silent_stdout() {
    let p = sample(SAMPLE_FORM);
    let args = [
        "extract-data",
        p.to_str().unwrap(),
        "--kind",
        "치즈",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
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
fn declared_flags_are_accepted_by_the_cli() {
    // 드리프트 가드: capabilities 가 광고하는 플래그는 실제로 받아야 한다. 선언만 있고
    // 받지 않으면 매니페스트로 도구를 자동 생성한 에이전트가 영영 못 쓰는 기능이 된다.
    let cap = run_ok(&["capabilities"]);
    let entry = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "extract-data")
        .unwrap_or_else(|| panic!("capabilities 에 extract-data 누락: {cap}"));
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(flags.contains(&"--json") && flags.contains(&"--kind") && flags.contains(&"--limit"));

    let p = sample(SAMPLE_FORM);
    let path = p.to_str().unwrap();
    for args in [
        vec!["extract-data", path, "--json"],
        vec!["extract-data", path, "--json", "--kind", "amount"],
        vec!["extract-data", path, "--json", "--limit", "1"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "선언된 플래그를 CLI 가 거부했습니다.\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn mcp_tool_declares_and_wires_every_input() {
    // 드리프트 가드: 스키마에 쓴 인자는 반드시 자식 CLI 에 닿아야 한다. 닿지 않으면
    // 서버는 그 인자를 조용히 버리고 성공을 보고한다.
    let mcp = run_ok(&["capabilities", "--mcp"]);
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_extract_data")
        .unwrap_or_else(|| panic!("hwp_extract_data 도구 누락: {mcp}"));

    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert!(tool["inputSchema"]["properties"].is_object(), "{tool}");
    let required = tool["inputSchema"]["required"]
        .as_array()
        .unwrap_or_else(|| panic!("required 배열 누락: {tool}"));
    assert!(required.iter().any(|r| r == "path"), "{tool}");
    assert_eq!(tool["cli"]["command"], "extract-data", "{tool}");

    let wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .chain(
            tool["cli"]["optionalArgs"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|o| o["when"].as_str().map(str::to_string)),
        )
        .collect();
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .unwrap()
        .keys()
    {
        // password 는 argv 가 아니라 stdin 축이다(cli.passwordStdin 계약).
        if key == "password" {
            continue;
        }
        assert!(
            wired.contains(key),
            "inputSchema 에만 있고 배선되지 않은 인자: {key} — {tool}"
        );
    }
}
