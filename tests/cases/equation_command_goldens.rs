//! 수식 카탈로그의 항목·명령·정직 태그를 검사한다.
//!
//! 독립 한컴 시각 기준을 확보하지 못한 AST·배치·SVG snapshot 복합 검사는
//! #7445의 하위 이슈 #8046으로 이관했다. 카탈로그와 golden은 보존하며,
//! 제거 전 검사 원문은 mydocs/pr/assets/issue7445/remaining-regressions-20261019/에 있다.

use std::path::{Path, PathBuf};

const FIXTURE_ROOT: &str = "src/renderer/equation/fixtures";
const CATALOG_FILE: &str = "catalog.tsv";

/// M09-1 원본 31종 id. 확장 카탈로그가 이 집합을 반드시 포함한다.
const M09_1_IDS: &[&str] = &[
    "over_simple",
    "over_grouped",
    "over_nested",
    "over_glued_denom",
    "over_in_paren",
    "sqrt_ident",
    "sqrt_group",
    "sqrt_index_paren",
    "sqrt_index_brace",
    "root_ident",
    "root_group",
    "root_index",
    "matrix_2x2",
    "matrix_col",
    "matrix_row",
    "matrix_no_brace",
    "pmatrix_2x2",
    "pmatrix_1x1",
    "pmatrix_frac",
    "bmatrix_2x2",
    "bmatrix_identity",
    "bmatrix_1x2",
    "dmatrix_2x2",
    "dmatrix_col",
    "dmatrix_1x1",
    "eqalign_two",
    "eqalign_one",
    "eqalign_no_amp",
    "pile_center",
    "lpile_left",
    "rpile_right",
];

const REQUIRED_COMMANDS: &[&str] = &[
    "OVER", "SQRT", "ROOT", "MATRIX", "PMATRIX", "BMATRIX", "DMATRIX", "EQALIGN", "PILE",
];

struct GoldenCase {
    id: String,
    dir: String,
    command: String,
    honesty: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT)
}

fn catalog_path() -> PathBuf {
    fixture_root().join(CATALOG_FILE)
}

fn load_catalog() -> Vec<GoldenCase> {
    let raw = std::fs::read_to_string(catalog_path())
        .unwrap_or_else(|e| panic!("read catalog {}: {e}", catalog_path().display()));
    let mut cases = Vec::new();
    for (lineno, line) in raw.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "catalog.tsv:{}: expected 5 TAB columns, got {}",
            lineno + 1,
            cols.len()
        );
        cases.push(GoldenCase {
            id: cols[0].to_string(),
            dir: cols[1].to_string(),
            command: cols[2].to_string(),
            honesty: cols[3].to_string(),
        });
    }
    cases
}

#[test]
fn equation_command_goldens_honesty_tags_are_known() {
    const KNOWN: &[&str] = &[
        "implemented",
        "implemented-variant",
        "root-aliases-sqrt",
        "pile-layout-as-row",
        "matrix-no-brace-empty",
        "missing-operand",
        "case-insensitive",
        "atop-vs-over",
        "latex-frac",
        "vmatrix-unimpl-text",
        "smallmatrix-unimpl-text",
        "ladder-fallback-matrix",
        "benzene-placeholder",
        "bigg-size-ignored",
        "choose-empty-top",
        "longdiv-simplified-row",
        "color-passthrough",
        "unknown-command-text",
        "cases-related",
        "rel-buildrel",
        "phantom-space",
        "latex-env-partial",
        "latex-space",
        "latex-text",
        "latex-stack",
        "limit-cmd",
        "left-script",
    ];
    let cases = load_catalog();
    assert!(
        cases.len() >= 200,
        "M09-g 카탈로그가 너무 작다 ({}건)",
        cases.len()
    );

    let m09_1: Vec<_> = cases.iter().filter(|c| c.dir == "m09_1").collect();
    assert_eq!(m09_1.len(), 31, "M09-1 원본 31종이 카탈로그에 없다");
    for id in M09_1_IDS {
        assert!(
            cases.iter().any(|c| c.id == *id && c.dir == "m09_1"),
            "M09-1 id {id} 가 카탈로그에 없다"
        );
    }
    for command in REQUIRED_COMMANDS {
        assert!(
            cases.iter().any(|c| c.command == *command),
            "명령 {command} 골든이 없다"
        );
    }
    let mut ids = cases.iter().map(|c| c.id.as_str()).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), cases.len(), "골든 id 가 중복된다");

    for case in &cases {
        assert!(
            KNOWN.contains(&case.honesty.as_str()),
            "{}: unknown honesty tag {}",
            case.id,
            case.honesty
        );
    }
    let expand = cases.iter().filter(|c| c.dir == "m09_expand").count();
    assert!(expand >= 160, "확장 골든이 부족하다 ({expand})");
}
