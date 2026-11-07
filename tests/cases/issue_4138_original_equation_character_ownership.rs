//! 검사 유형: 문자/원본 숫자/현대 문법 4개는 관계형, 기저 괄호/근호 인자 원점 2개는 고정형(192DPI ±2px).
//! 주석 갱신: 2026-11-07 09:02:52 KST.
//! Native/fresh WASM 전 197쪽 최저: 98.47416% / 98.50155%.
//! 129쪽: 98.47416% / 98.50155%. 130쪽: 98.73703% / 98.71245%.
//! 근호 슬롯 보정 뒤 12쪽은 99.35295% / 99.51005%로 회복했다. 독립 PDF의 E 원점을 고정한다.
//! 2026-11-07 측정: 192DPI·compat2024·PDFium·2px 색상 경계 보정 전 내용 실루엣 raw.
//! 근거: mydocs/pr/assets/issue4138/original-radical-slot-20261107/README.md.
//! 원점 기준은 독립 Hancom2024 Print PDF12·129쪽이며, 후보 PNG로 기대 좌표를 만들지 않았다.
//! 일반 괄호는 문자, LEFT/RIGHT는 구조다. 소수점 때문에 원본 숫자 전체를 다른 face로 보내지 않는다.

use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    DocumentCore,
};
use serde_json::Value;
fn selected(node: &LayerNode) -> Option<Value> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children.iter().find_map(selected),
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            selected(child)
        }
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
            if let PaintOp::Equation { equation, .. } = op {
                if equation.script.starts_with("P=460") {
                    return Some(serde_json::to_value(&equation.layout_box).unwrap());
                }
            }
            None
        }),
    }
}
fn inspect(v: &Value, chars: &mut Vec<(char, f64, String)>, structural: &mut usize) {
    if let Some(k) = v.get("kind") {
        if k.get("Paren").is_some() {
            *structural += 1;
        }
        if let Some(run) = k.get("HancomGlyphs") {
            for c in run["glyphs"]
                .as_str()
                .unwrap()
                .chars()
                .filter(|c| matches!(c, '(' | ')'))
            {
                chars.push((
                    c,
                    v["font_size"].as_f64().unwrap(),
                    run["font_family"].as_str().unwrap().into(),
                ));
            }
        }
    }
    match v {
        Value::Array(a) => {
            for x in a {
                inspect(x, chars, structural)
            }
        }
        Value::Object(o) => {
            for x in o.values() {
                inspect(x, chars, structural)
            }
        }
        _ => {}
    }
}
#[test]
fn original_literal_parentheses_keep_the_original_character_program_around_scripted_bases() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path=root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(128, RenderProfile::Print)
        .unwrap();
    let layout = selected(&tree.root).unwrap();
    let mut chars = vec![];
    let mut structural = 0;
    inspect(&layout, &mut chars, &mut structural);
    assert_eq!(
        structural, 0,
        "일반 괄호를 LEFT/RIGHT 확장 구조로 바꾸지 않는다"
    );
    assert_eq!(chars.iter().map(|x| x.0).collect::<String>(), "()()");
    assert!(chars.iter().all(|x| x.2 == "RHWP Hancom Legacy Equation"));
    assert!(
        chars.windows(2).all(|w| (w[0].1 - w[1].1).abs() < 1e-8),
        "네 일반 괄호는 본체 크기의 문자다"
    );
}
#[test]
fn public_modern_parser_keeps_its_parenthesized_script_base() {
    let tree = rhwp::renderer::equation::parser::parse("(A_i)^2");
    assert!(
        format!("{tree:?}").contains("Paren"),
        "현대 공개 문법의 괄호 소유를 보존한다"
    );
}

fn equation_for_prefix(node: &LayerNode, prefix: &str) -> Option<Value> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            children.iter().find_map(|n| equation_for_prefix(n, prefix))
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            equation_for_prefix(child, prefix)
        }
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
            if let PaintOp::Equation { equation, .. } = op {
                if equation.script.starts_with(prefix) {
                    return Some(serde_json::to_value(&equation.layout_box).unwrap());
                }
            }
            None
        }),
    }
}
fn matching_runs<'a>(v: &'a Value, text: &str, out: &mut Vec<&'a Value>) {
    if let Some(run) = v.get("kind").and_then(|k| k.get("HancomGlyphs")) {
        if run["glyphs"] == text {
            out.push(v)
        }
    }
    match v {
        Value::Array(a) => {
            for x in a {
                matching_runs(x, text, out)
            }
        }
        Value::Object(o) => {
            for x in o.values() {
                matching_runs(x, text, out)
            }
        }
        _ => {}
    }
}
fn decimal_run(prefix: &str, text: &str, scripted: bool) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path=root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(129, RenderProfile::Print)
        .unwrap();
    let layout = equation_for_prefix(&tree.root, prefix).unwrap();
    let mut runs = vec![];
    matching_runs(&layout, text, &mut runs);
    assert_eq!(
        runs.len(),
        1,
        "소수점 하나 때문에 숫자 run 전체를 다른 face로 보내지 않는다"
    );
    let run = runs[0];
    assert_eq!(
        run["kind"]["HancomGlyphs"]["font_family"],
        "RHWP Hancom Legacy Equation"
    );
    if scripted {
        assert!(
            run["font_size"].as_f64().unwrap() < layout["font_size"].as_f64().unwrap(),
            "원본 위첨자의 자식 크기 소유도 보존한다"
        );
    }
}
#[test]
fn source_decimal_number_keeps_its_original_font_program() {
    decimal_run("E`=", "0.5", false)
}
#[test]
fn source_decimal_exponent_keeps_its_original_font_program_and_child_size() {
    decimal_run("C_L =", "0.4", true)
}

fn source_record(
    node: &LayerNode,
    frames: &[[f64; 6]],
    prefix: &str,
) -> Option<(Value, [f64; 2], Vec<[f64; 6]>)> {
    use rhwp::paint::GroupKind;
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            let mut next = frames.to_vec();
            if let GroupKind::Group(g) = group_kind {
                if let Some(m) = g.paint_transform {
                    next.push(m)
                }
            }
            children
                .iter()
                .find_map(|n| source_record(n, &next, prefix))
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            source_record(child, frames, prefix)
        }
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
            if let PaintOp::Equation { bbox, equation } = op {
                if equation.script.starts_with(prefix) {
                    return Some((
                        serde_json::to_value(&equation.layout_box).unwrap(),
                        [bbox.x, bbox.y],
                        frames.to_vec(),
                    ));
                }
            }
            None
        }),
    }
}
fn selected_glyph_origins(
    v: &Value,
    mut x: f64,
    mut y: f64,
    wanted: &str,
    out: &mut Vec<(char, f64, f64)>,
) {
    if let Some(k) = v.get("kind") {
        x += v["x"].as_f64().unwrap();
        y += v["y"].as_f64().unwrap();
        if let Some(run) = k.get("HancomGlyphs") {
            for (c, offset) in run["glyphs"]
                .as_str()
                .unwrap()
                .chars()
                .zip(run["positions"].as_array().unwrap())
            {
                if wanted.contains(c) {
                    out.push((
                        c,
                        x + offset.as_f64().unwrap(),
                        y + v["baseline"].as_f64().unwrap()
                            + run["baseline_shift"].as_f64().unwrap(),
                    ));
                }
            }
        } else {
            selected_glyph_origins(k, x, y, wanted, out)
        }
    } else {
        match v {
            Value::Array(a) => {
                for w in a {
                    selected_glyph_origins(w, x, y, wanted, out)
                }
            }
            Value::Object(o) => {
                for w in o.values() {
                    selected_glyph_origins(w, x, y, wanted, out)
                }
            }
            _ => {}
        }
    }
}
/// 고정형: 완료129쪽의 첫 기저 괄호 두 원점을 독립 Print와192DPI ±2px로 대조한다.
/// 뒤 기저의 남은 간격 차이를 잘못된 고정 기대값으로 굳히지 않는다.
#[test]
fn original_first_script_base_delimiters_match_independent_print_pixels() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path=root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(128, RenderProfile::Print)
        .unwrap();
    let (layout, origin, frames) = source_record(&tree.root, &[], "P=460").unwrap();
    let mut points = vec![];
    selected_glyph_origins(&layout, origin[0], origin[1], "()", &mut points);
    assert_eq!(points.len(), 4);
    // 독립 PDF129쪽 Type3 원점(pt)을96DPI CSS 좌표로 변환한 기준이다.
    let expected = [('(', 181.66154, 945.68652), (')', 226.59719, 945.68652)];
    for ((character, mut x, mut y), (wanted, px, py)) in points.into_iter().take(2).zip(expected) {
        for [a, b, c, d, e, f] in &frames {
            let nx = a * x + c * y + e;
            y = b * x + d * y + f;
            x = nx;
        }
        assert_eq!(character, wanted);
        assert!(((x - px) * 2.0).abs() <= 2.0, "괄호 x: {x} / {px}");
        assert!(((y - py) * 2.0).abs() <= 2.0, "괄호 y: {y} / {py}");
    }
}

/// 고정형: 기존12쪽은 Native99.02184% / fresh WASM99.18145%인 완료 쪽이다.
/// 같은 입력 재인쇄에서도 일치한 독립 Type3 E 원점을192DPI ±2px로 검사한다.
#[test]
fn original_radical_operand_origin_matches_independent_print_pixels() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(11, RenderProfile::Print)
        .unwrap();
    let (layout, origin, frames) = source_record(&tree.root, &[], "sqrt {E}").unwrap();
    let mut points = vec![];
    selected_glyph_origins(&layout, origin[0], origin[1], "E", &mut points);
    assert_eq!(
        points.len(),
        1,
        "근호 인자의 실제 원본 E glyph를 한 번 선택한다"
    );
    let (_, mut x, mut y) = points[0];
    for [a, b, c, d, e, f] in &frames {
        let nx = a * x + c * y + e;
        y = b * x + d * y + f;
        x = nx;
    }
    // 독립 한컴 Print PDF12쪽의 원점(165.030563pt,482.712463pt)을96DPI로 변환했다.
    let (expected_x, expected_y) = (220.0407511393229, 643.6166178385416);
    assert!(
        ((x - expected_x) * 2.0).abs() <= 2.0,
        "원본 근호 E x: {x} / {expected_x}"
    );
    assert!(
        ((y - expected_y) * 2.0).abs() <= 2.0,
        "원본 근호 E y: {y} / {expected_y}"
    );
}
