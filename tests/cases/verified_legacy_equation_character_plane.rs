//! 구형 CHAR 표시의 원본 프레임·소유·독립 Print 글리프 전진을 보호한다.
//! 전 9쪽 Native 99.98690% 이상 / fresh WASM 100% 및 확대 판독 뒤 등록했다.
//! 사설 글꼴 윤곽의 CI 공급을 가정하지 않고 같은 수식 내부 원점을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;

use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{BoundingBox, EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn reference() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/issue1949/legacy-character-plane.print.json"
    ))
    .unwrap()
}

fn read_file(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn load(oracle: &Value) -> DocumentCore {
    for key in ["input", "pdf"] {
        let bytes = read_file(oracle[key].as_str().unwrap());
        assert_eq!(
            Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            oracle[format!("{key}_sha256")],
            "같은 원본·독립 Print 쌍"
        );
    }
    DocumentCore::from_bytes(&read_file(oracle["input"].as_str().unwrap())).unwrap()
}

struct Drawn<'a> {
    node: &'a RenderNode,
    equation: &'a EquationNode,
    line: &'a RenderNode,
    body: BoundingBox,
}

fn collect<'a>(
    node: &'a RenderNode,
    line: Option<&'a RenderNode>,
    body: Option<BoundingBox>,
    out: &mut Vec<Drawn<'a>>,
) {
    let line = if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        Some(node)
    } else {
        line
    };
    let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
        Some(node.bbox)
    } else {
        body
    };
    if let RenderNodeType::Equation(equation) = &node.node_type {
        out.push(Drawn {
            node,
            equation,
            line: line.expect("TAC 수식의 글줄 소속"),
            body: body.expect("본문 소속"),
        });
    }
    for child in &node.children {
        collect(child, line, body, out);
    }
}

fn glyphs(value: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64, f64)>) {
    let (x, y) = (x + value.x, y + value.y);
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.positions.len(), run.glyphs.chars().count());
            out.extend(run.glyphs.chars().zip(&run.positions).map(|(ch, dx)| {
                (
                    ch,
                    x + dx,
                    y + value.baseline + run.baseline_shift,
                    value.font_size,
                )
            }));
        }
        LayoutKind::Row(children) => {
            for child in children {
                glyphs(child, x, y, out);
            }
        }
        LayoutKind::FontStyle { body, .. } => glyphs(body, x, y, out),
        LayoutKind::Empty => {}
        other => panic!("완료한 문자 행의 구조를 다른 템플릿으로 바꾸지 않는다: {other:?}"),
    }
}

#[test]
fn legacy_character_rows_preserve_source_ownership_content_and_declared_frame() {
    let oracle = reference();
    let core = load(&oracle);
    assert_eq!(
        core.page_count(),
        oracle["page_count"].as_u64().unwrap() as u32
    );
    let cases = oracle["cases"].as_array().unwrap();
    let mut seen = BTreeSet::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut rows = Vec::new();
        collect(&tree.root, None, None, &mut rows);
        for row in rows {
            let equation = row.equation;
            let para = equation.para_index.unwrap();
            let control = equation.control_index.unwrap();
            assert!(seen.insert(para), "원본 문자 행의 중복 없음");
            assert_eq!(equation.section_index, Some(0));
            assert_eq!(equation.cell_index, None);
            assert_eq!(cases[para]["print_page"], page + 1, "독립 Print 쪽 소속");
            let source = &core.document().sections[0].paragraphs[para].controls[control];
            let Control::Equation(source) = source else {
                panic!("원본 수식 제어 소속");
            };
            assert!(source.version_info.is_empty());
            assert!(source.common.treat_as_char);
            assert_eq!(equation.script, source.script);
            assert_eq!(cases[para]["script"], equation.script);
            let RenderNodeType::TextLine(line) = &row.line.node_type else {
                unreachable!();
            };
            assert_eq!(line.section_index, equation.section_index);
            assert_eq!(line.para_index, equation.para_index);
            let bounds = row.node.bbox;
            assert!(bounds.x >= row.body.x);
            assert!(bounds.x + bounds.width <= row.body.x + row.body.width);
            assert!(bounds.y >= row.line.bbox.y);
            assert!(bounds.y + bounds.height <= row.line.bbox.y + row.line.bbox.height);
            let declared = f64::from(source.common.width) * 96.0 / 7200.0;
            assert!((equation.layout_box.width - declared).abs() <= declared * 1e-9);
            let mut ink = Vec::new();
            glyphs(&equation.layout_box, 0.0, 0.0, &mut ink);
            let text: String = ink.iter().map(|item| item.0).collect();
            assert_eq!(cases[para]["characters"], text, "문자 순서·누락·중복");
            assert!(ink.windows(2).all(|pair| pair[0].1 < pair[1].1));
            assert!(
                ink.iter().all(|item| {
                    (item.2 - ink[0].2).abs() <= 1e-9 && (item.3 - equation.font_size).abs() <= 1e-9
                }),
                "선언 너비로 글리프 크기나 같은 줄 기준선을 바꾸지 않는다"
            );
        }
    }
    assert_eq!(seen, (0..cases.len()).collect(), "전 186개 원본 행 보존");
}

#[test]
fn completed_legacy_character_pages_match_print_relative_origins_within_two_pixels() {
    let oracle = reference();
    let core = load(&oracle);
    let cases = oracle["cases"].as_array().unwrap();
    let mut compared = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut rows = Vec::new();
        collect(&tree.root, None, None, &mut rows);
        for row in rows {
            let para = row.equation.para_index.unwrap();
            assert_eq!(cases[para]["print_page"], page + 1);
            let expected = cases[para]["relative_x_96dpi"].as_array().unwrap();
            let mut ink = Vec::new();
            glyphs(&row.equation.layout_box, 0.0, 0.0, &mut ink);
            assert_eq!(ink.len(), expected.len());
            for (actual, expected) in ink.iter().zip(expected) {
                let relative_x = actual.1 - ink[0].1;
                assert!(
                    (relative_x - expected.as_f64().unwrap()).abs() <= 2.0,
                    "독립 Print 글리프 전진: page={}, para={para}, actual={relative_x}, expected={expected}",
                    page + 1
                );
            }
            compared += 1;
        }
    }
    assert_eq!(compared, cases.len());
}
