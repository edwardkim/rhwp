//! [#3830] `batch extract-data` 계약 회귀 테스트.
//!
//! 편람 입력을 사용하는 검사는 별도 PR로 이관한다. 인자·capabilities·MCP 선언 검사는 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn write_stdin_ignoring_early_exit(child: &mut std::process::Child, stdin_body: &str) {
    use std::io::ErrorKind;
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
}

fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    write_stdin_ignoring_early_exit(&mut child, stdin_body);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn batch_kind_and_limit_flags_rejected_for_other_subcommands() {
    // --kind·--limit 는 extract-data 축 전용이다(--query 가 search 전용인 것과 같은 규약).
    for extra in [vec!["--kind", "date"], vec!["--limit", "3"]] {
        let mut args = vec!["batch", "info", "--json"];
        args.extend(extra.clone());
        let output = run_with_stdin(&args, "");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn batch_extract_data_invalid_kind_is_usage_error() {
    let args = ["batch", "extract-data", "--json", "--kind", "bogus"];
    let output = run_with_stdin(&args, "");
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn capabilities_batch_declares_extract_data_axis() {
    // 드리프트 가드: 축을 추가했으면 자기서술도 같이 갱신되어야 한다.
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities"])
        .output()
        .expect("rhwp 실행 실패");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
    let subs: Vec<&str> = v["batch"]["subcommands"]
        .as_array()
        .expect("batch.subcommands")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    assert!(
        subs.contains(&"extract-data"),
        "capabilities 의 batch 축에 extract-data 가 없습니다: {subs:?}"
    );
    let flags: Vec<&str> = v["batch"]["flags"]
        .as_array()
        .expect("batch.flags")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();
    for expected in ["--kind", "--limit"] {
        assert!(
            flags.contains(&expected),
            "batch 플래그에 {expected} 누락: {flags:?}"
        );
    }
    assert!(
        v["batch"]["mcp"]["available"]
            .as_array()
            .expect("mcp.available")
            .iter()
            .any(|s| s
                .as_str()
                .is_some_and(|s| s.contains("hwp_batch_extract_data"))),
        "capabilities 의 batch.mcp.available 에 hwp_batch_extract_data 언급이 없습니다: {v}"
    );
}

#[test]
fn mcp_batch_extract_data_tool_is_invocable_from_its_declaration() {
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("rhwp 실행 실패");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("MCP JSON");
    let tools = v["tools"].as_array().expect("tools");

    let tool = tools
        .iter()
        .find(|t| t["name"] == "hwp_batch_extract_data")
        .expect("hwp_batch_extract_data 도구가 있어야 합니다");

    // 드리프트 가드: inputSchema 는 type·properties·required 를 갖춰야 한다.
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    let properties = tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties");
    assert!(
        tool["inputSchema"]["required"].is_array(),
        "required 배열이 있어야 합니다: {tool}"
    );

    // --json 은 MCP 를 통한 호출에서 필수다(다른 hwp_batch_* 와 같은 규약) — 인자
    // 템플릿에 항상 박혀 있어야 한다.
    let args_str = tool["cli"]["args"].to_string();
    assert!(
        args_str.contains("--json"),
        "cli.args 에 --json 이 항상 포함돼야 합니다: {args_str}"
    );

    // paths 는 stdin 으로 전달되므로 args 템플릿에는 {paths} 자리표시자가 없어야 하고,
    // MCP_STDIN_TOOLS 목록에 이 도구 이름이 등재돼 있어야 한다(런타임에서 실제 stdin
    // 배선 여부는 mcp_serve.rs 의 run_cli_tool 이 그 목록을 참조해 검증한다).
    assert!(
        !args_str.contains("{paths}"),
        "paths 는 stdin 전용이라 cli.args 자리표시자에 나오면 안 됩니다: {args_str}"
    );

    // 선언된 속성은 paths 를 제외하고 전부 CLI 플래그로 배선돼야 한다 — 선언만 있고
    // 배선이 없으면 클라이언트가 보낸 값이 조용히 무시된다.
    let optional_args_str = tool["cli"]["optionalArgs"].to_string();
    for (name, placeholder) in [
        ("kind", "{kind}"),
        ("limit", "{limit}"),
        ("threads", "{threads}"),
    ] {
        assert!(properties.contains_key(name), "{tool}");
        assert!(
            args_str.contains(placeholder) || optional_args_str.contains(placeholder),
            "{name} 속성이 선언돼 있지만 CLI 로 배선되지 않았습니다: {tool}"
        );
    }
    assert!(properties.contains_key("paths"), "{tool}");
}
