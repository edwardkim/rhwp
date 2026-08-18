//! M09-1: 수식 명령 골든 30종.
//!
//! OVER/SQRT/ROOT/MATRIX/PMATRIX/BMATRIX/DMATRIX/EQALIGN/PILE 의
//! 현재 파서·레이아웃·SVG 출력을 잠근다. 엔진을 고치거나 디스패치를
//! 리팩터하지 않는다(M09-2). 미구현·축약 동작도 현행 그대로 고정한다.
//!
//! 골든 갱신: `UPDATE_EQ_GOLDENS=1 cargo test --test <suite> equation_command_goldens`

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use rhwp::renderer::equation::ast::MatrixStyle;
use rhwp::renderer::equation::layout::{EqLayout, LayoutBox, LayoutKind};
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::svg_render::render_equation_svg;

const FONT_SIZE: f64 = 20.0;
const COLOR: &str = "#000000";
const FIXTURE_DIR: &str = "src/renderer/equation/fixtures/m09_1";

struct GoldenCase {
    id: &'static str,
    command: &'static str,
    script: &'static str,
}

/// 명령별 대표 스크립트. 현행 엔진 동작을 잠그는 입력이지 정답 오라클이 아니다.
const CASES: &[GoldenCase] = &[
    // OVER
    GoldenCase {
        id: "over_simple",
        command: "OVER",
        script: "1 OVER 2",
    },
    GoldenCase {
        id: "over_grouped",
        command: "OVER",
        script: "{a+b} OVER {c-d}",
    },
    GoldenCase {
        id: "over_nested",
        command: "OVER",
        script: "{1 OVER 2} OVER 3",
    },
    GoldenCase {
        id: "over_glued_denom",
        command: "OVER",
        script: "7 OVER10",
    },
    GoldenCase {
        id: "over_in_paren",
        command: "OVER",
        script: "LEFT ( x OVER y RIGHT )",
    },
    // SQRT
    GoldenCase {
        id: "sqrt_ident",
        command: "SQRT",
        script: "SQRT x",
    },
    GoldenCase {
        id: "sqrt_group",
        command: "SQRT",
        script: "SQRT {a+b}",
    },
    GoldenCase {
        id: "sqrt_index_paren",
        command: "SQRT",
        script: "SQRT(3) of x",
    },
    GoldenCase {
        id: "sqrt_index_brace",
        command: "SQRT",
        script: "SQRT {3} of {x}",
    },
    // ROOT — 현행 파서는 SQRT 와 동일 분기로 처리한다.
    GoldenCase {
        id: "root_ident",
        command: "ROOT",
        script: "ROOT x",
    },
    GoldenCase {
        id: "root_group",
        command: "ROOT",
        script: "ROOT {a+b}",
    },
    GoldenCase {
        id: "root_index",
        command: "ROOT",
        script: "ROOT(3) of x",
    },
    // MATRIX
    GoldenCase {
        id: "matrix_2x2",
        command: "MATRIX",
        script: "MATRIX{a & b # c & d}",
    },
    GoldenCase {
        id: "matrix_col",
        command: "MATRIX",
        script: "MATRIX{1 # 2 # 3}",
    },
    GoldenCase {
        id: "matrix_row",
        command: "MATRIX",
        script: "MATRIX{a & b & c}",
    },
    GoldenCase {
        id: "matrix_no_brace",
        command: "MATRIX",
        script: "MATRIX",
    },
    // PMATRIX
    GoldenCase {
        id: "pmatrix_2x2",
        command: "PMATRIX",
        script: "PMATRIX{a & b # c & d}",
    },
    GoldenCase {
        id: "pmatrix_1x1",
        command: "PMATRIX",
        script: "PMATRIX{x}",
    },
    GoldenCase {
        id: "pmatrix_frac",
        command: "PMATRIX",
        script: "PMATRIX{{1} OVER {2} & 0 # 0 & 1}",
    },
    // BMATRIX
    GoldenCase {
        id: "bmatrix_2x2",
        command: "BMATRIX",
        script: "BMATRIX{a & b # c & d}",
    },
    GoldenCase {
        id: "bmatrix_identity",
        command: "BMATRIX",
        script: "BMATRIX{1 & 0 # 0 & 1}",
    },
    GoldenCase {
        id: "bmatrix_1x2",
        command: "BMATRIX",
        script: "BMATRIX{x & y}",
    },
    // DMATRIX
    GoldenCase {
        id: "dmatrix_2x2",
        command: "DMATRIX",
        script: "DMATRIX{a & b # c & d}",
    },
    GoldenCase {
        id: "dmatrix_col",
        command: "DMATRIX",
        script: "DMATRIX{x # y}",
    },
    GoldenCase {
        id: "dmatrix_1x1",
        command: "DMATRIX",
        script: "DMATRIX{1}",
    },
    // EQALIGN
    GoldenCase {
        id: "eqalign_two",
        command: "EQALIGN",
        script: "EQALIGN{x &= 1 # y &= 2}",
    },
    GoldenCase {
        id: "eqalign_one",
        command: "EQALIGN",
        script: "EQALIGN{a + b &= c}",
    },
    GoldenCase {
        id: "eqalign_no_amp",
        command: "EQALIGN",
        script: "EQALIGN{x = 1 # y = 2}",
    },
    // PILE / LPILE / RPILE — 레이아웃은 전용 kind 없이 Row 로 쌓는다.
    GoldenCase {
        id: "pile_center",
        command: "PILE",
        script: "PILE{a # b # c}",
    },
    GoldenCase {
        id: "lpile_left",
        command: "PILE",
        script: "LPILE{a # b}",
    },
    GoldenCase {
        id: "rpile_right",
        command: "PILE",
        script: "RPILE{a # b}",
    },
];

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR)
}

fn golden_path(id: &str) -> PathBuf {
    fixture_dir().join(format!("{id}.golden"))
}

fn snapshot_of(case: &GoldenCase) -> String {
    let ast = parse(case.script);
    let layout = EqLayout::new(FONT_SIZE).layout(&ast);
    let svg = render_equation_svg(&layout, COLOR, FONT_SIZE);
    let mut layout_ir = String::new();
    dump_layout(&mut layout_ir, &layout, 0);
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

#[test]
fn equation_command_goldens_lock_parse_layout_svg() {
    // 이슈 본문은 "약 30종". OVER 5 + SQRT 4 + ROOT 3 + MATRIX 4 + PMATRIX 3
    // + BMATRIX 3 + DMATRIX 3 + EQALIGN 3 + PILE 3 = 31.
    assert_eq!(CASES.len(), 31, "M09-1 골든 개수가 카탈로그와 어긋난다");
    let required = [
        "OVER", "SQRT", "ROOT", "MATRIX", "PMATRIX", "BMATRIX", "DMATRIX", "EQALIGN", "PILE",
    ];
    for command in required {
        assert!(
            CASES.iter().any(|c| c.command == command),
            "명령 {command} 골든이 없다"
        );
    }
    let mut ids = CASES.iter().map(|c| c.id).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), CASES.len(), "골든 id 가 중복된다");

    std::fs::create_dir_all(fixture_dir()).expect("create fixture dir");
    let update = update_requested();
    let mut mismatches = Vec::new();

    for case in CASES {
        let actual = snapshot_of(case);
        let path = golden_path(case.id);
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
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
