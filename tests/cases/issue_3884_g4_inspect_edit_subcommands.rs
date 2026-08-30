//! [#3884 G4] inspect·edit 하위 명령이 capabilities 에 자기서술된다.
//!
//! `.commands[].name` 만 읽으면 `inspect`/`edit` 두 이름만 보인다. 실제 호출은
//! `rhwp inspect <hidden-text|injection|unicode>` 와
//! `rhwp edit <fill-fields|replace-text|set-cell|insert-image|redact|sanitize>`
//! 이다. 하위를 `subcommands[]` 로 실어 `--search` 가 부모를 찾게 한다.
//! 새 CLI 동사는 만들지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn capabilities() -> serde_json::Value {
    let out = run(&["capabilities"]);
    assert_eq!(out.status.code(), Some(0), "capabilities 실행 실패");
    serde_json::from_slice(&out.stdout).expect("capabilities stdout 이 순수 JSON 이 아니다")
}

fn command_entry<'a>(caps: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    caps["commands"]
        .as_array()
        .expect("commands 배열 없음")
        .iter()
        .find(|c| c["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("commands 에 {name} 항목이 없다"))
}

fn declared_names(parent: &str) -> Vec<String> {
    let caps = capabilities();
    let entry = command_entry(&caps, parent);
    let subs = entry["subcommands"]
        .as_array()
        .unwrap_or_else(|| panic!("{parent} 항목에 subcommands 선언이 없다 (#3884 G4)"));
    subs.iter()
        .map(|s| {
            let name = s["name"]
                .as_str()
                .expect("subcommands[].name 누락")
                .to_string();
            let summary = s["summary"].as_str().expect("subcommands[].summary 누락");
            assert!(
                !summary.trim().is_empty(),
                "{parent} {name} 의 summary 가 비었다"
            );
            name
        })
        .collect()
}

fn usage_listed(parent: &str) -> Vec<String> {
    let out = run(&[parent]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{parent} 무인자는 usage 오류(exit 2)여야 한다"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    let anchor = format!("rhwp {parent} <");
    let start = err
        .find(&anchor)
        .unwrap_or_else(|| panic!("{parent} USAGE 에서 '{anchor}' 를 못 찾았다:\n{err}"))
        + anchor.len();
    let rest = &err[start..];
    let end = rest.find('>').expect("USAGE 하위 목록의 '>' 누락");
    rest[..end]
        .split('|')
        .map(|s| s.trim().to_string())
        .collect()
}

#[test]
fn capabilities_lists_g4_inspect_edit_subcommands() {
    let inspect = declared_names("inspect");
    for name in ["hidden-text", "injection", "unicode"] {
        assert!(
            inspect.iter().any(|n| n == name),
            "inspect.subcommands 에 {name} 이 없다: {inspect:?}"
        );
    }
    let edit = declared_names("edit");
    for name in [
        "fill-fields",
        "replace-text",
        "set-cell",
        "insert-image",
        "redact",
        "sanitize",
    ] {
        assert!(
            edit.iter().any(|n| n == name),
            "edit.subcommands 에 {name} 이 없다: {edit:?}"
        );
    }
}

#[test]
fn declared_subcommands_match_dispatch_usage() {
    for parent in ["inspect", "edit"] {
        assert_eq!(
            declared_names(parent),
            usage_listed(parent),
            "{parent}: capabilities.subcommands 선언과 USAGE 실물이 다르다"
        );
    }
}

#[test]
fn search_finds_parent_by_subcommand_keyword() {
    for (keyword, parent) in [("redact", "edit"), ("hidden-text", "inspect")] {
        let out = run(&["capabilities", "--search", keyword, "--json"]);
        assert_eq!(out.status.code(), Some(0));
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)
            .expect("--search --json stdout 이 순수 JSON 이 아니다");
        let names: Vec<&str> = v["commands"]
            .as_array()
            .expect("commands 배열 없음")
            .iter()
            .filter_map(|c| c["name"].as_str())
            .collect();
        assert!(
            names.contains(&parent),
            "--search {keyword} 가 {parent} 를 못 찾았다 (결과: {names:?})"
        );
    }
}
