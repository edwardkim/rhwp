//! [#3884 G4 / 로드맵 R7] edit·inspect 하위 명령 자기서술 계약.
//!
//! 계약: `capabilities` 의 edit·inspect 항목이 `subcommands`(이름+요약)를 싣고,
//! 그 선언이 디스패치 실물과 일치한다. 실물의 오라클은 두 개다 —
//! ① 부모 명령의 USAGE 문자열(디스패치 코드 바로 옆에서 자기 목록을 이미 실어
//!    나른다), ② 실행 거동(선언된 하위는 "알 수 없는" 없이 usage 오류, 미선언
//!    하위는 "알 수 없는" 거부). 선언·실물 어느 쪽이 먼저 바뀌어도 여기서 깨진다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

const PARENTS: [&str; 2] = ["edit", "inspect"];

/// capabilities 선언에서 부모 항목의 subcommands 이름 목록.
fn declared_subcommands(parent: &str) -> Vec<String> {
    let out = run(&["capabilities"]);
    assert_eq!(out.status.code(), Some(0), "capabilities 실행 실패");
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities stdout 이 순수 JSON 이 아니다");
    let entry = v["commands"]
        .as_array()
        .expect("commands 배열 없음")
        .iter()
        .find(|c| c["name"].as_str() == Some(parent))
        .unwrap_or_else(|| panic!("commands 에 {parent} 항목이 없다"))
        .clone();
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

/// 부모를 인자 없이 불러 USAGE 의 `<a|b|c>` 목록을 읽는다 — 디스패치 쪽 실물.
fn usage_listed_subcommands(parent: &str) -> Vec<String> {
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
fn declared_subcommands_match_dispatch_usage() {
    for parent in PARENTS {
        let declared = declared_subcommands(parent);
        let usage = usage_listed_subcommands(parent);
        assert_eq!(
            declared, usage,
            "{parent}: capabilities.subcommands 선언과 USAGE 실물이 다르다 — 한쪽만 고쳤다"
        );
    }
}

#[test]
fn every_declared_subcommand_actually_dispatches() {
    for parent in PARENTS {
        for sub in declared_subcommands(parent) {
            let out = run(&[parent, &sub]);
            let err = String::from_utf8_lossy(&out.stderr);
            assert_eq!(
                out.status.code(),
                Some(2),
                "{parent} {sub}: 파일 없는 호출은 usage 오류(exit 2)여야 한다"
            );
            assert!(
                !err.contains("알 수 없는"),
                "{parent} {sub}: 선언된 하위가 디스패치에서 미지 명령으로 거부됐다:\n{err}"
            );
        }
    }
}

#[test]
fn undeclared_subcommand_is_rejected() {
    for parent in PARENTS {
        let out = run(&[parent, "zz-없는-축"]);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2));
        assert!(
            err.contains("알 수 없는"),
            "{parent}: 미선언 하위가 조용히 지나갔다:\n{err}"
        );
    }
}

/// R7 DoD: `capabilities --search` 가 하위 명령 이름으로 부모를 찾는다.
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
