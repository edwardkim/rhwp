//! [#5511 Stage 1] 최상위 dispatch와 자기서술 표면의 characterization 계약.
//!
//! handler 이동 전에 실제 `main()` 102개 arm과 catalog를 양방향으로 대조한다.
//! help·capabilities·MCP의 현재 결과도 같은 catalog와 비교하되, 이 단계에서는
//! 기존 출력 생성기를 바꾸지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Output};

#[path = "../src/cli/catalog.rs"]
mod catalog;

use catalog::{commands, Visibility};

const MAIN_SOURCE: &str = include_str!("../src/main.rs");

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn json(args: &[&str]) -> serde_json::Value {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "rhwp {} 실패:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout JSON")
}

/// `fn main()`의 최상위 match만 잘라 `Some("...")` arm을 읽는다.
fn dispatch_names() -> Vec<&'static str> {
    let body = MAIN_SOURCE
        .split_once("fn main() {")
        .expect("main 시작")
        .1
        .split_once("/// [#3263]")
        .expect("main 다음 경계")
        .0;
    let mut names = Vec::new();
    for line in body.lines() {
        let mut rest = line;
        while let Some((_, after)) = rest.split_once("Some(\"") {
            let (name, tail) = after.split_once("\"").expect("Some 문자열 끝");
            if !name.starts_with('-') {
                names.push(name);
            }
            rest = tail;
        }
    }
    names
}

fn help_names() -> BTreeSet<String> {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("  ")?;
            if rest.starts_with(' ') || rest.starts_with('-') {
                return None;
            }
            let token = rest.split_whitespace().next()?;
            token
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                .then(|| token.to_string())
        })
        .collect()
}

#[test]
fn catalog_is_unique_and_matches_all_top_level_dispatch_arms() {
    let catalog_names: BTreeSet<&str> = commands().iter().map(|command| command.name).collect();
    let dispatch = dispatch_names();
    let dispatch_set: BTreeSet<&str> = dispatch.iter().copied().collect();

    assert_eq!(commands().len(), 102, "characterization 기준선");
    assert_eq!(catalog_names.len(), commands().len(), "catalog 이름 중복");
    assert_eq!(dispatch.len(), dispatch_set.len(), "dispatch arm 이름 중복");
    assert_eq!(dispatch_set, catalog_names, "dispatch↔catalog drift");
}

#[test]
fn capabilities_order_and_metadata_match_catalog() {
    let value = json(&["capabilities"]);
    let live = value["commands"].as_array().expect("commands");
    let expected: Vec<_> = commands()
        .iter()
        .filter(|command| command.in_capabilities())
        .collect();
    let live_names: Vec<&str> = live
        .iter()
        .map(|command| command["name"].as_str().expect("name"))
        .collect();
    let expected_names: Vec<&str> = expected.iter().map(|command| command.name).collect();
    assert_eq!(
        live_names, expected_names,
        "이름 또는 did-you-mean 순서 drift"
    );

    for (actual, declared) in live.iter().zip(expected) {
        assert_eq!(
            actual["category"],
            declared.category.as_str(),
            "{}",
            declared.name
        );
        assert_eq!(
            actual["json"].as_bool().unwrap_or(false),
            declared.json_contract,
            "{} json 계약",
            declared.name
        );
        assert_eq!(
            actual["batch"].as_bool().unwrap_or(false),
            declared.batch,
            "{} batch 계약",
            declared.name
        );
        assert_eq!(
            actual["requiresFeature"].as_str(),
            declared.requires_feature,
            "{} feature 계약",
            declared.name
        );
    }
}

#[test]
fn help_and_mcp_participation_match_catalog() {
    let expected_help: BTreeSet<String> = commands()
        .iter()
        .filter(|command| command.in_help())
        .map(|command| command.name.to_string())
        .collect();
    assert_eq!(help_names(), expected_help, "help↔catalog drift");

    let manifest = json(&["capabilities", "--mcp"]);
    let live_mcp: BTreeSet<&str> = manifest["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|tool| tool["cli"]["command"].as_str())
        .collect();
    let expected_mcp: BTreeSet<&str> = commands()
        .iter()
        .filter(|command| command.mcp)
        .map(|command| command.name)
        .collect();
    assert_eq!(live_mcp, expected_mcp, "MCP↔catalog drift");
}

#[test]
fn exceptional_visibility_is_small_explicit_and_explained() {
    let mut hidden = BTreeMap::new();
    let mut dispatch_only = BTreeMap::new();
    for command in commands() {
        match command.visibility {
            Visibility::Public => {}
            Visibility::Hidden(reason) => {
                assert!(!reason.trim().is_empty(), "{} hidden 사유", command.name);
                hidden.insert(command.name, reason);
            }
            Visibility::DispatchOnly(reason) => {
                assert!(
                    !reason.trim().is_empty(),
                    "{} dispatch-only 사유",
                    command.name
                );
                dispatch_only.insert(command.name, reason);
            }
        }
    }

    assert_eq!(
        hidden.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(["core-pages", "dump-extents", "measure-width"])
    );
    assert_eq!(
        dispatch_only.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(["dump-anchors", "dump-carets", "export-llm", "ir-sweep"])
    );
}
