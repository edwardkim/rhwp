//! 독립 Print의 literal·CJK·연산자·그룹 243개 행과 공통 표시 평면을 대조한다.
//! 같은 입력 전 23쪽의 Native/fresh raw가 각각 98% 이상인 증거에 한해 ±2px를 사용한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    renderer::{
        equation::layout::{LayoutBox, LayoutKind},
        render_tree::{EquationNode, RenderNode, RenderNodeType},
    },
    DocumentCore,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn oracle() -> Value {
    serde_json::from_slice(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "mydocs/pr/assets/legacy_literal_operator_plane_20261025/independent_coordinates.json",
        ))
        .unwrap(),
    )
    .unwrap()
}
fn load(corpus: &Value) -> DocumentCore {
    for key in ["input", "pdf"] {
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(corpus[key].as_str().unwrap()),
        )
        .unwrap();
        let digest = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            digest,
            corpus[format!("{key}_sha256")],
            "같은 원본·독립 Print의 바이트를 사용한다"
        );
    }
    DocumentCore::from_bytes(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(corpus["input"].as_str().unwrap()),
        )
        .unwrap(),
    )
    .unwrap()
}
struct Drawn<'a> {
    node: &'a RenderNode,
    equation: &'a EquationNode,
    line: &'a RenderNode,
    page: u32,
}
fn collect<'a>(
    node: &'a RenderNode,
    line: Option<&'a RenderNode>,
    page: u32,
    labels: &mut BTreeMap<usize, (String, f64)>,
    rows: &mut Vec<Drawn<'a>>,
) {
    let line = if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        Some(node)
    } else {
        line
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.cell_context.is_none() {
            if let Some(pi) = run.para_index {
                let label = labels.entry(pi).or_insert((String::new(), node.bbox.x));
                label.0.push_str(&run.text);
                label.1 = label.1.min(node.bbox.x);
            }
        }
    }
    if let RenderNodeType::Equation(equation) = &node.node_type {
        rows.push(Drawn {
            node,
            equation,
            line: line.expect("TAC 수식의 실제 글줄 소속"),
            page,
        });
    }
    for child in &node.children {
        collect(child, line, page, labels, rows)
    }
}
fn glyphs(value: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64, f64)>) {
    let (x, y) = (x + value.x, y + value.y);
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.glyphs.chars().count(), run.positions.len());
            out.extend(run.glyphs.chars().zip(&run.positions).map(|(c, dx)| {
                (
                    c,
                    x + dx,
                    y + value.baseline + run.baseline_shift,
                    value.font_size,
                )
            }));
        }
        LayoutKind::Row(children) => {
            for child in children {
                glyphs(child, x, y, out)
            }
        }
        LayoutKind::FontStyle { body, .. } => glyphs(body, x, y, out),
        LayoutKind::Empty => {}
        other => panic!("완료 행의 같은 글리프 평면이 유지돼야 한다: {other:?}"),
    }
}
fn check(corpus_name: &str, coordinates: bool) {
    let reference = oracle();
    let mut compared = 0;
    for corpus in reference["corpora"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|corpus| corpus["name"] == corpus_name)
    {
        let core = load(corpus);
        assert_eq!(
            u64::from(core.page_count()),
            corpus["page_count"].as_u64().unwrap()
        );
        let trees = (0..core.page_count())
            .map(|page| core.build_page_render_tree(page).unwrap())
            .collect::<Vec<_>>();
        let mut labels = BTreeMap::new();
        let mut rows = Vec::new();
        for (page, tree) in trees.iter().enumerate() {
            collect(&tree.root, None, page as u32 + 1, &mut labels, &mut rows);
        }
        let cases = corpus["cases"].as_array().unwrap();
        assert_eq!(
            rows.len(),
            cases.len(),
            "독립 Print 전체 행의 누락·중복 없음"
        );
        let mut seen = BTreeSet::new();
        for row in rows {
            let equation = row.equation;
            let pi = equation.para_index.unwrap();
            assert!(
                pi % 2 == 1 && seen.insert(pi),
                "원본 수식 행을 한 번만 배치한다"
            );
            let expected = &cases[pi / 2];
            assert_eq!(expected["paragraph"], pi as u64);
            assert_eq!(expected["page"], row.page);
            assert_eq!(expected["script"], equation.script);
            assert_eq!(equation.section_index, Some(0));
            assert_eq!(equation.cell_index, None);
            let control = &core.document().sections[0].paragraphs[pi].controls
                [equation.control_index.unwrap()];
            let Control::Equation(source) = control else {
                panic!("원본 수식 제어 소속")
            };
            assert!(source.version_info.is_empty() && source.common.treat_as_char);
            assert_eq!(source.script, equation.script);
            let RenderNodeType::TextLine(line) = &row.line.node_type else {
                unreachable!()
            };
            assert_eq!(line.para_index, Some(pi));
            assert!(row.node.bbox.y >= row.line.bbox.y, "자기 글줄의 수식 소속");
            assert!(
                (row.node.bbox.width - f64::from(source.common.width) / 75.0).abs() < 1.0 / 75.0,
                "원본 선언 폭과 표시 프레임의 공통 결과"
            );
            let label = labels.get(&(pi - 1)).expect("바로 앞 원본 표식 행");
            assert_eq!(label.0, expected["label"].as_str().unwrap());
            let mut ink = Vec::new();
            glyphs(&equation.layout_box, 0.0, 0.0, &mut ink);
            assert_eq!(
                ink.iter().map(|g| g.0).collect::<String>(),
                expected["characters"]
            );
            assert!(ink.windows(2).all(|p| p[0].1 < p[1].1), "문자 순서 보존");
            assert!(
                ink.iter().all(|g| (g.3 - equation.font_size).abs() < 1e-9),
                "선언 폭으로 자연 글리프 크기를 늘리지 않는다"
            );
            if coordinates {
                let points = expected["points"].as_array().unwrap();
                assert_eq!(ink.len(), points.len());
                for (actual, point) in ink.iter().zip(points) {
                    let actual_x = row.node.bbox.x + actual.1 - label.1;
                    let actual_y = actual.2 - ink[0].2;
                    assert!(
                        (actual_x - point["x_from_label_css_px"].as_f64().unwrap()).abs() <= 2.0,
                        "{} {} 독립 Print의 root 여백·문자 원점: actual={actual_x}, expected={}",
                        corpus["name"],
                        expected["label"],
                        point["x_from_label_css_px"]
                    );
                    assert!(
                        (actual_y - point["baseline_from_first_css_px"].as_f64().unwrap()).abs()
                            <= 2.0,
                        "혼재 CJK와 Type3의 독립 기준선 관계"
                    );
                }
            }
            compared += 1;
        }
        assert_eq!(seen.len(), cases.len());
    }
    let expected = match corpus_name {
        "literal" => 108,
        "cjk" => 72,
        "mixed" => 54,
        "groups" => 9,
        _ => panic!("완료 원본 이름"),
    };
    assert_eq!(
        compared, expected,
        "해당 문서의 모든 크기·너비·혼재·빈 그룹 반례를 확인한다"
    );
}
#[test]
fn completed_legacy_literals_preserve_source_and_independent_print_origins() {
    check("literal", true)
}
#[test]
fn completed_legacy_cjk_operators_preserve_source_and_independent_print_origins() {
    check("cjk", true)
}
#[test]
fn completed_legacy_mixed_operators_preserve_source_and_independent_print_origins() {
    check("mixed", true)
}
#[test]
fn completed_legacy_groups_preserve_source_and_independent_print_origins() {
    check("groups", true)
}
