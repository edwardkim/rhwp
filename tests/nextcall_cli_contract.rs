//! [#4220 T4] CLI 실패 봉투 수복 힌트 — exit 2 사용법 오류 stderr 의 정형 수복 한 줄.
//!
//! 계약 3면:
//! 1. **문법** — 수복 줄은 stderr 의 **마지막 줄** 하나뿐이고, `수복: ` 접두어 뒤는
//!    한 줄 JSON `{"nextCall":{"name":...,"subcommand"?,...,"why":...}}` 이다.
//!    `nextCall.name` 은 반드시 실존 명령(capabilities 단일 출처 — R72 와 같은 계약),
//!    `subcommand` 가 있으면 그 명령의 실존 하위 명령이다.
//! 2. **오제안 0** — 다음 호출이 결정론적으로 정해지지 않는 경로(임계 밖 오타,
//!    하위 명령 누락)에는 수복 줄 자체가 없다. R72 가 MCP 쪽에서 지켜온 원칙 그대로다.
//! 3. **stdout 0 B 무침해** — 실패 3면 계약(#2707: exit 2·stdout 0 B·stderr 안내)은
//!    그대로다. 수복 줄은 stderr 에만 더해지는 추가 전용 확장이다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

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

/// stderr 에서 수복 줄을 찾아 JSON 으로 파싱한다. 문법 계약(마지막 줄·단일 줄)을
/// 여기서 함께 단언한다 — 소비자는 "마지막 `수복: ` 줄 하나"만 파싱하면 된다.
fn parse_recovery_line(args: &[&str], output: &Output) -> serde_json::Value {
    let err = String::from_utf8_lossy(&output.stderr);
    let recovery_lines: Vec<&str> = err.lines().filter(|l| l.starts_with("수복: ")).collect();
    assert_eq!(
        recovery_lines.len(),
        1,
        "수복 줄은 정확히 하나여야 한다\n{}",
        describe(args, output)
    );
    let last = err
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .expect("stderr 가 비어 있다");
    assert!(
        last.starts_with("수복: "),
        "수복 줄은 stderr 의 마지막 줄이어야 한다 (마지막 줄: {last:?})\n{}",
        describe(args, output)
    );
    let json_part = last.strip_prefix("수복: ").unwrap();
    serde_json::from_str(json_part).unwrap_or_else(|e| {
        panic!(
            "수복 줄 본문이 한 줄 JSON 이어야 한다({e}): {json_part}\n{}",
            describe(args, output)
        )
    })
}

/// capabilities 의 명령 이름 목록 — nextCall.name 실존 검사의 단일 출처.
fn capability_command_names() -> Vec<String> {
    let out = run(&["capabilities"]);
    assert_eq!(out.status.code(), Some(0), "capabilities 실행 실패");
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities stdout 이 순수 JSON 이 아니다");
    v["commands"]
        .as_array()
        .expect("commands 배열 없음")
        .iter()
        .filter_map(|c| c["name"].as_str().map(String::from))
        .collect()
}

// ── ① 문법 — 정형 수복 줄 ────────────────────────────────────────────────

/// 미지 명령 + 확신 교정(#3694 임계 내): 수복 줄이 교정 명령을 nextCall 어휘로 싣는다.
#[test]
fn unknown_command_recovery_line_carries_corrected_next_call() {
    let args = ["exprot-svg"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(
        out.stdout.is_empty(),
        "실패 경로 stdout 은 0 바이트여야 한다\n{}",
        describe(&args, &out)
    );
    let v = parse_recovery_line(&args, &out);
    assert_eq!(v["nextCall"]["name"], "export-svg", "{v}");
    assert!(
        !v["nextCall"]["why"]
            .as_str()
            .unwrap_or("")
            .trim()
            .is_empty(),
        "why 는 비어 있으면 안 된다: {v}"
    );
    // arguments 는 싣지 않는다 — 나머지 인자가 옳다고 검증한 바 없다(오제안 0).
    assert!(
        v["nextCall"].get("arguments").is_none(),
        "검증하지 않은 인자를 되울리면 안 된다: {v}"
    );
}

/// 수복 줄의 nextCall.name 은 반드시 실존 명령이다 (R72 와 같은 실존 계약).
#[test]
fn recovery_next_call_name_exists_in_capabilities() {
    let names = capability_command_names();
    for args in [&["exprot-svg"][..], &[][..]] {
        let out = run(args);
        assert_eq!(out.status.code(), Some(2), "{}", describe(args, &out));
        let v = parse_recovery_line(args, &out);
        let name = v["nextCall"]["name"].as_str().expect("nextCall.name 누락");
        assert!(
            names.iter().any(|n| n == name),
            "nextCall.name 이 실존 명령이 아니다: {name} ({v})"
        );
    }
}

/// 명령 누락: 발견 경로는 결정론적이다 — capabilities 자기서술로 보낸다.
#[test]
fn missing_command_recovery_points_to_capabilities() {
    let args: [&str; 0] = [];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(out.stdout.is_empty(), "{}", describe(&args, &out));
    let v = parse_recovery_line(&args, &out);
    assert_eq!(v["nextCall"]["name"], "capabilities", "{v}");
}

/// 미지 inspect 하위 명령 + 확신 교정: subcommand 필드가 실존 하위 명령을 싣는다.
#[test]
fn inspect_unknown_subcommand_recovery_carries_subcommand() {
    let args = ["inspect", "hiden-text"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    assert!(out.stdout.is_empty(), "{}", describe(&args, &out));
    let v = parse_recovery_line(&args, &out);
    assert_eq!(v["nextCall"]["name"], "inspect", "{v}");
    assert_eq!(v["nextCall"]["subcommand"], "hidden-text", "{v}");

    // subcommand 실존 — capabilities 의 inspect.subcommands 선언과 대조.
    let caps = run(&["capabilities"]);
    let caps: serde_json::Value = serde_json::from_slice(&caps.stdout).expect("capabilities JSON");
    let declared: Vec<&str> = caps["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "inspect")
        .expect("inspect 항목 없음")["subcommands"]
        .as_array()
        .expect("inspect.subcommands 없음")
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(
        declared.contains(&"hidden-text"),
        "교정 대상이 선언된 하위 명령이 아니다: {declared:?}"
    );
}

// ── ② 오제안 0 — 불확실 경로에는 수복 줄이 없다 ─────────────────────────

/// 임계 밖 오타(#3694 gibberish 계약과 같은 입력): 힌트도 수복 줄도 없다.
#[test]
fn gibberish_command_gets_no_recovery_line() {
    let args = ["코끼리코끼리"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("수복:"),
        "임계 밖 오타에 수복 줄을 지어내면 안 된다(오제안 0)\n{}",
        describe(&args, &out)
    );
}

/// inspect 하위 명령 누락: 어느 축을 원했는지 결정론적으로 알 수 없다 — 침묵.
#[test]
fn inspect_missing_subcommand_gets_no_recovery_line() {
    let args = ["inspect"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(2), "{}", describe(&args, &out));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("수복:"),
        "하위 명령 누락은 다음 호출이 결정론적이지 않다 — 침묵해야 한다\n{}",
        describe(&args, &out)
    );
}

/// 실행 실패(exit 1)는 수복 대상 부류가 아니다 — 인자를 고칠 것이 없는데
/// 교정을 지어내면 재시도 래퍼가 "내 호출이 틀렸다"로 오독한다.
#[test]
fn runtime_failure_gets_no_recovery_line() {
    let args = ["export-text", "없는파일-t4.hwp"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(1), "{}", describe(&args, &out));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("수복:"),
        "런타임 실패에 수복 줄을 지어내면 안 된다\n{}",
        describe(&args, &out)
    );
}

// ── ③ stdout 0 B 무침해 + 기존 산문 무회귀 ──────────────────────────────

/// 수복 줄이 붙어도 기존 산문 계약(오류·힌트·사용법 안내)은 그대로다.
#[test]
fn recovery_line_is_additive_to_existing_prose() {
    let args = ["exprot-svg"];
    let out = run(&args);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("알 수 없는 명령"), "{err}");
    assert!(
        err.contains("힌트: 가장 가까운 명령은 'export-svg' 입니다"),
        "{err}"
    );
    assert!(err.contains("사용법: rhwp <명령> [옵션]"), "{err}");
    assert!(
        out.stdout.is_empty(),
        "수복 줄은 stderr 전용이다 — stdout 오염 금지\n{}",
        describe(&args, &out)
    );
}
