//! [#5511 Stage 1~2] 최상위 dispatch와 자기서술·모듈 소유권의 characterization 계약.
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
const DATA_EXTRACTION_SOURCE: &str = include_str!("../src/cli/queries/data_extraction.rs");
const DIAGNOSTICS_SOURCE: &str = include_str!("../src/cli/queries/diagnostics.rs");
const DIGEST_SOURCE: &str = include_str!("../src/cli/queries/digest.rs");
const DOCUMENT_INVENTORY_SOURCE: &str = include_str!("../src/cli/queries/document_inventory.rs");
const EXPLAIN_SOURCE: &str = include_str!("../src/cli/queries/explain.rs");
const EXPLORE_SOURCE: &str = include_str!("../src/cli/queries/explore.rs");
const SECURITY_INSPECTION_SOURCE: &str = include_str!("../src/cli/queries/security_inspection.rs");
const STRUCTURED_OBJECTS_SOURCE: &str = include_str!("../src/cli/queries/structured_objects.rs");

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

#[test]
fn document_inventory_queries_are_owned_by_the_query_module() {
    for (command, handler) in [
        ("word-count", "word_count"),
        ("bookmarks", "bookmarks"),
        ("charts", "charts"),
        ("fields", "show_fields"),
    ] {
        assert!(
            DOCUMENT_INVENTORY_SOURCE.contains(&format!("pub(crate) fn {handler}(")),
            "{handler} 구현이 document_inventory 모듈에 있어야 한다"
        );
        assert!(
            !MAIN_SOURCE.contains(&format!("fn {handler}(")),
            "{handler} 구현이 main.rs로 되돌아가면 안 된다"
        );
        assert!(
            MAIN_SOURCE.contains(&format!(
                "Some(\"{command}\") => exit_with(cli::queries::document_inventory::{handler}(&args[2..]))"
            )),
            "{command} dispatch가 query 모듈 API를 사용해야 한다"
        );
    }
}

#[test]
fn data_extraction_query_is_owned_by_the_query_module() {
    assert!(
        DATA_EXTRACTION_SOURCE.contains("pub(crate) fn extract_data_command("),
        "extract_data_command 구현이 data_extraction 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn extract_data_command("),
        "extract_data_command 구현이 main.rs로 되돌아가면 안 된다"
    );
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compact_main.contains(
            "Some(\"extract-data\")=>exit_with(cli::queries::data_extraction::extract_data_command("
        ),
        "extract-data dispatch가 query 모듈 API를 사용해야 한다"
    );
    assert!(
        compact_main.contains("data_extraction::extract_data_command(&args[2..]"),
        "extract-data dispatch가 사용자 인자를 그대로 전달해야 한다"
    );
}

#[test]
fn digest_query_is_owned_by_the_query_module() {
    assert!(
        DIGEST_SOURCE.contains("pub(crate) fn digest_document("),
        "digest_document 구현이 digest 모듈에 있어야 한다"
    );
    assert!(
        DIGEST_SOURCE.contains("fn parse_digest_pages("),
        "digest 전용 범위 파서가 digest 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn digest_document("),
        "digest_document 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn parse_digest_pages("),
        "digest 전용 범위 파서가 main.rs로 되돌아가면 안 된다"
    );
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compact_main.contains(
            "Some(\"digest\")=>exit_with(cli::queries::digest::digest_document(&args[2..]))"
        ),
        "digest dispatch가 query 모듈 API를 사용해야 한다"
    );
}

#[test]
fn explain_query_is_owned_by_the_query_module() {
    assert!(
        EXPLAIN_SOURCE.contains("pub(crate) fn explain_document("),
        "explain_document 구현이 explain 모듈에 있어야 한다"
    );
    for helper in [
        "explain_table_summary",
        "explain_table_phrase",
        "explain_summary",
        "explain_json_value",
    ] {
        assert!(
            EXPLAIN_SOURCE.contains(&format!("fn {helper}(")),
            "{helper} 구현이 explain 모듈에 있어야 한다"
        );
        assert!(
            !MAIN_SOURCE.contains(&format!("fn {helper}(")),
            "{helper} 구현이 main.rs로 되돌아가면 안 된다"
        );
    }
    assert!(
        !MAIN_SOURCE.contains("fn explain_document("),
        "explain_document 구현이 main.rs로 되돌아가면 안 된다"
    );
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compact_main.contains(
            "Some(\"explain\")=>exit_with(cli::queries::explain::explain_document(&args[2..]))"
        ),
        "explain dispatch가 query 모듈 API를 사용해야 한다"
    );
}

#[test]
fn explore_query_is_owned_by_the_query_module() {
    assert!(
        EXPLORE_SOURCE.contains("pub(crate) fn explore_document("),
        "explore_document 구현이 explore 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn explore_document("),
        "explore_document 구현이 main.rs로 되돌아가면 안 된다"
    );
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compact_main.contains(
            "Some(\"explore\")=>exit_with(cli::queries::explore::explore_document(&args[2..]))"
        ),
        "explore dispatch가 query 모듈 API를 사용해야 한다"
    );
}

#[test]
fn hidden_text_query_is_owned_by_the_security_inspection_module() {
    assert!(
        SECURITY_INSPECTION_SOURCE.contains("pub(crate) fn inspect_hidden_text("),
        "inspect_hidden_text 구현이 security_inspection 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn inspect_hidden_text("),
        "inspect_hidden_text 구현이 main.rs로 되돌아가면 안 된다"
    );
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        compact_main.contains(
            "Some(\"hidden-text\")=>cli::queries::security_inspection::inspect_hidden_text(&args[1..])"
        ),
        "inspect hidden-text dispatch가 security query 모듈 API를 사용해야 한다"
    );
}

#[test]
fn structured_object_queries_are_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    for (command, handler) in [
        ("form-value", "form_value"),
        ("header-footer", "header_footer"),
        ("headers-footers", "headers_footers"),
    ] {
        assert!(
            STRUCTURED_OBJECTS_SOURCE.contains(&format!("pub(crate) fn {handler}(")),
            "{handler} 구현이 structured_objects 모듈에 있어야 한다"
        );
        assert!(
            !MAIN_SOURCE.contains(&format!("fn {handler}(")),
            "{handler} 구현이 main.rs로 되돌아가면 안 된다"
        );
        assert!(
            compact_main.contains(&format!("cli::queries::structured_objects::{handler}(")),
            "{command} dispatch가 query 모듈 API를 사용해야 한다"
        );
    }
}

#[test]
fn note_shape_diagnostic_is_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        DIAGNOSTICS_SOURCE.contains("pub(crate) fn dump_note_shape("),
        "dump_note_shape 구현이 diagnostics 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn dump_note_shape("),
        "dump_note_shape 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::diagnostics::dump_note_shape("),
        "dump-note-shape dispatch가 diagnostics 모듈 API를 사용해야 한다"
    );
}

#[test]
fn endnote_line_diagnostic_is_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        DIAGNOSTICS_SOURCE.contains("pub(crate) fn dump_endnote_lines("),
        "dump_endnote_lines 구현이 diagnostics 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn dump_endnote_lines("),
        "dump_endnote_lines 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::diagnostics::dump_endnote_lines("),
        "dump-endnote-lines dispatch가 diagnostics 모듈 API를 사용해야 한다"
    );
}

#[test]
fn extent_diagnostic_is_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        DIAGNOSTICS_SOURCE.contains("pub(crate) fn dump_extents("),
        "dump_extents 구현이 diagnostics 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn dump_extents("),
        "dump_extents 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::diagnostics::dump_extents("),
        "dump-extents dispatch가 diagnostics 모듈 API를 사용해야 한다"
    );
}

#[test]
fn document_diagnostic_is_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        DIAGNOSTICS_SOURCE.contains("pub(crate) fn diag_document("),
        "diag_document 구현이 diagnostics 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn diag_document("),
        "diag_document 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::diagnostics::diag_document("),
        "diag dispatch가 diagnostics 모듈 API를 사용해야 한다"
    );
}

#[test]
fn raw_record_diagnostic_is_owned_by_the_query_module() {
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        DIAGNOSTICS_SOURCE.contains("pub(crate) fn dump_raw_records("),
        "dump_raw_records 구현이 diagnostics 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn dump_raw_records("),
        "dump_raw_records 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::diagnostics::dump_raw_records("),
        "dump-records dispatch가 diagnostics 모듈 API를 사용해야 한다"
    );
}

#[test]
fn search_query_is_owned_by_its_query_module() {
    let search_source = include_str!("../src/cli/queries/search.rs");
    let compact_main: String = MAIN_SOURCE
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        search_source.contains("pub(crate) fn search_document("),
        "search_document 구현이 search query 모듈에 있어야 한다"
    );
    assert!(
        !MAIN_SOURCE.contains("fn search_document("),
        "search_document 구현이 main.rs로 되돌아가면 안 된다"
    );
    assert!(
        compact_main.contains("cli::queries::search::search_document("),
        "search dispatch가 search query 모듈 API를 사용해야 한다"
    );
}
