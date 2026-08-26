//! M09-1 + M09-g: 수식 명령 골든 (변형·예외·미구현 정직 표).
//!
//! OVER/SQRT/ROOT/MATRIX/PMATRIX/BMATRIX/DMATRIX/EQALIGN/PILE 의 현재
//! 파서·레이아웃·SVG 출력을 잠근다. 엔진을 고치거나 디스패치를 리팩터하지
//! 않는다(M09-2). 미구현·축약 동작도 현행 그대로 고정한다.
//!
//! 카탈로그: `src/renderer/equation/fixtures/catalog.tsv`
//! 골든 갱신: `UPDATE_EQ_GOLDENS=1 cargo test --test <suite> equation_command_goldens`

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use rhwp::renderer::equation::ast::MatrixStyle;
use rhwp::renderer::equation::layout::{EqLayout, LayoutBox, LayoutKind};
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::svg_render::render_equation_svg;

const FONT_SIZE: f64 = 20.0;
const COLOR: &str = "#000000";
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
    script: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT)
}

fn catalog_path() -> PathBuf {
    fixture_root().join(CATALOG_FILE)
}

fn golden_path(case: &GoldenCase) -> PathBuf {
    fixture_root()
        .join(&case.dir)
        .join(format!("{}.golden", case.id))
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
            script: cols[4].to_string(),
        });
    }
    cases
}

fn snapshot_of(case: &GoldenCase) -> String {
    let ast = parse(&case.script);
    let layout = EqLayout::new(FONT_SIZE).layout(&ast);
    let svg = render_equation_svg(&layout, COLOR, FONT_SIZE);
    let mut layout_ir = String::new();
    dump_layout(&mut layout_ir, &layout, 0);
    if case.dir == "m09_1" {
        format!(
            "# M09-1 equation golden — current-engine lock\n\
             command: {}\n\
             script: {}\n\
             \n\
             === ast ===\n\
             {:#?}\n\
             \n\
             === layout ===\n\
             {}\n\
             === svg ===\n\
             {}",
            case.command, case.script, ast, layout_ir, svg
        )
    } else {
        format!(
            "# M09-g equation golden — current-engine lock (variant/exception/unimplemented honesty)\n\
             command: {}\n\
             script: {}\n\
             honesty: {}\n\
             \n\
             === ast ===\n\
             {:#?}\n\
             \n\
             === layout ===\n\
             {}\n\
             === svg ===\n\
             {}",
            case.command, case.script, case.honesty, ast, layout_ir, svg
        )
    }
    .replace("\r\n", "\n")
}

fn dump_layout(out: &mut String, lb: &LayoutBox, indent: usize) {
    let pad = "  ".repeat(indent);
    let metrics = format!(
        "x={:.4} y={:.4} w={:.4} h={:.4} bl={:.4}",
        lb.x, lb.y, lb.width, lb.height, lb.baseline
    );
    match &lb.kind {
        LayoutKind::Row(children) => {
            let _ = writeln!(out, "{pad}Row {metrics}");
            for child in children {
                dump_layout(out, child, indent + 1);
            }
        }
        LayoutKind::Text(text) => {
            let _ = writeln!(out, "{pad}Text({text:?}) {metrics}");
        }
        LayoutKind::Number(text) => {
            let _ = writeln!(out, "{pad}Number({text:?}) {metrics}");
        }
        LayoutKind::Symbol(text) => {
            let _ = writeln!(out, "{pad}Symbol({text:?}) {metrics}");
        }
        LayoutKind::MathSymbol(text) => {
            let _ = writeln!(out, "{pad}MathSymbol({text:?}) {metrics}");
        }
        LayoutKind::Function(text) => {
            let _ = writeln!(out, "{pad}Function({text:?}) {metrics}");
        }
        LayoutKind::Fraction { numer, denom } => {
            let _ = writeln!(out, "{pad}Fraction {metrics}");
            let _ = writeln!(out, "{pad}  numer:");
            dump_layout(out, numer, indent + 2);
            let _ = writeln!(out, "{pad}  denom:");
            dump_layout(out, denom, indent + 2);
        }
        LayoutKind::Atop { top, bottom } => {
            let _ = writeln!(out, "{pad}Atop {metrics}");
            let _ = writeln!(out, "{pad}  top:");
            dump_layout(out, top, indent + 2);
            let _ = writeln!(out, "{pad}  bottom:");
            dump_layout(out, bottom, indent + 2);
        }
        LayoutKind::Sqrt { index, body } => {
            let _ = writeln!(out, "{pad}Sqrt {metrics}");
            if let Some(index) = index {
                let _ = writeln!(out, "{pad}  index:");
                dump_layout(out, index, indent + 2);
            }
            let _ = writeln!(out, "{pad}  body:");
            dump_layout(out, body, indent + 2);
        }
        LayoutKind::Superscript { base, sup } => {
            let _ = writeln!(out, "{pad}Superscript {metrics}");
            let _ = writeln!(out, "{pad}  base:");
            dump_layout(out, base, indent + 2);
            let _ = writeln!(out, "{pad}  sup:");
            dump_layout(out, sup, indent + 2);
        }
        LayoutKind::Subscript { base, sub } => {
            let _ = writeln!(out, "{pad}Subscript {metrics}");
            let _ = writeln!(out, "{pad}  base:");
            dump_layout(out, base, indent + 2);
            let _ = writeln!(out, "{pad}  sub:");
            dump_layout(out, sub, indent + 2);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            let _ = writeln!(out, "{pad}SubSup {metrics}");
            let _ = writeln!(out, "{pad}  base:");
            dump_layout(out, base, indent + 2);
            let _ = writeln!(out, "{pad}  sub:");
            dump_layout(out, sub, indent + 2);
            let _ = writeln!(out, "{pad}  sup:");
            dump_layout(out, sup, indent + 2);
        }
        LayoutKind::BigOp { symbol, sub, sup } => {
            let _ = writeln!(out, "{pad}BigOp({symbol:?}) {metrics}");
            if let Some(sub) = sub {
                let _ = writeln!(out, "{pad}  sub:");
                dump_layout(out, sub, indent + 2);
            }
            if let Some(sup) = sup {
                let _ = writeln!(out, "{pad}  sup:");
                dump_layout(out, sup, indent + 2);
            }
        }
        LayoutKind::Limit { is_upper, sub } => {
            let _ = writeln!(out, "{pad}Limit(is_upper={is_upper}) {metrics}");
            if let Some(sub) = sub {
                let _ = writeln!(out, "{pad}  sub:");
                dump_layout(out, sub, indent + 2);
            }
        }
        LayoutKind::Matrix { cells, style } => {
            let style = match style {
                MatrixStyle::Plain => "Plain",
                MatrixStyle::Paren => "Paren",
                MatrixStyle::Bracket => "Bracket",
                MatrixStyle::Vert => "Vert",
            };
            let _ = writeln!(out, "{pad}Matrix({style}) {metrics}");
            for (row_i, row) in cells.iter().enumerate() {
                let _ = writeln!(out, "{pad}  row{row_i}:");
                for cell in row {
                    dump_layout(out, cell, indent + 2);
                }
            }
        }
        LayoutKind::Rel { arrow, over, under } => {
            let _ = writeln!(out, "{pad}Rel {metrics}");
            let _ = writeln!(out, "{pad}  arrow:");
            dump_layout(out, arrow, indent + 2);
            let _ = writeln!(out, "{pad}  over:");
            dump_layout(out, over, indent + 2);
            if let Some(under) = under {
                let _ = writeln!(out, "{pad}  under:");
                dump_layout(out, under, indent + 2);
            }
        }
        LayoutKind::EqAlign { rows } => {
            let _ = writeln!(out, "{pad}EqAlign {metrics}");
            for (row_i, (left, right)) in rows.iter().enumerate() {
                let _ = writeln!(out, "{pad}  row{row_i}.left:");
                dump_layout(out, left, indent + 2);
                let _ = writeln!(out, "{pad}  row{row_i}.right:");
                dump_layout(out, right, indent + 2);
            }
        }
        LayoutKind::Paren { left, right, body } => {
            let _ = writeln!(out, "{pad}Paren({left:?},{right:?}) {metrics}");
            dump_layout(out, body, indent + 1);
        }
        LayoutKind::Decoration { kind, body } => {
            let _ = writeln!(out, "{pad}Decoration({kind:?}) {metrics}");
            dump_layout(out, body, indent + 1);
        }
        LayoutKind::FontStyle { style, body } => {
            let _ = writeln!(out, "{pad}FontStyle({style:?}) {metrics}");
            dump_layout(out, body, indent + 1);
        }
        LayoutKind::Space(width) => {
            let _ = writeln!(out, "{pad}Space({width:.4}) {metrics}");
        }
        LayoutKind::Newline => {
            let _ = writeln!(out, "{pad}Newline {metrics}");
        }
        LayoutKind::Empty => {
            let _ = writeln!(out, "{pad}Empty {metrics}");
        }
    }
}

fn update_requested() -> bool {
    matches!(
        std::env::var("UPDATE_EQ_GOLDENS").as_deref(),
        Ok("1") | Ok("true")
    )
}

fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn equation_command_goldens_lock_parse_layout_svg() {
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

    for dir in ["m09_1", "m09_expand"] {
        std::fs::create_dir_all(fixture_root().join(dir)).expect("create fixture dir");
    }
    let update = update_requested();
    let mut mismatches = Vec::new();

    for case in &cases {
        let actual = snapshot_of(case);
        let path = golden_path(case);
        if update || !path.exists() {
            if !update && !path.exists() {
                mismatches.push(format!(
                    "{}: 골든 파일 없음 ({}) — UPDATE_EQ_GOLDENS=1 로 생성",
                    case.id,
                    path.display()
                ));
                continue;
            }
            std::fs::write(&path, actual.as_bytes())
                .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
            continue;
        }
        let expected = normalize_lf(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
        );
        if expected != actual {
            mismatches.push(format!(
                "{} ({})\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
                case.id,
                path.display()
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "수식 골든 불일치 {}건 (현행 엔진 잠금. 의도된 변경이면 UPDATE_EQ_GOLDENS=1):\n\n{}",
        mismatches.len(),
        mismatches.join("\n\n")
    );
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
