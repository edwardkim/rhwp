//! 독립 한컴 Print와 Native/fresh WASM 전체 TSV·확대 판독을 확인한 입력이다.
//! 절대 화면 위치 대신 원본 소유, 첨자/분모 기준선과 최종 SVG 문자 방출 위치를 검사한다.
//! 증적: mydocs/pr/assets/issue7642-extended-script-frame-20261012/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};

const SUBSUP: &str = "LEFT ( {2} over {sqrt {5}} RIGHT ) _{i} ^{2}";
const ROOT_SUP: &str = "LEFT ( {2} over {sqrt {5}} RIGHT ) ^{2}";
const INDEX_SUP: &str = "LEFT ( {1} over {root {3} of {5}} RIGHT ) ^{2}";

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-extended-base-script-diagnostic")
        .join(name);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("독립 저장 입력"))
        .expect("한컴 저장본 읽기");
    core.set_hangul2024_compat(true);
    core
}

fn equations<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a EquationNode)>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        out.push((node, eq));
    }
    for child in &node.children {
        equations(child, out);
    }
}

fn owned(node: &EquationNode, source: (usize, usize, usize)) -> bool {
    (node.section_index, node.para_index, node.control_index)
        == (Some(source.0), Some(source.1), Some(source.2))
}

fn source_addresses(core: &DocumentCore, script: &str) -> Vec<(usize, usize, usize, u32)> {
    let mut found = Vec::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, para) in data.paragraphs.iter().enumerate() {
            for (control, value) in para.controls.iter().enumerate() {
                if let Control::Equation(eq) = value {
                    if eq.script == script {
                        found.push((section, paragraph, control, eq.font_size));
                    }
                }
            }
        }
    }
    found
}

fn script_box(
    value: &LayoutBox,
    parent_x: f64,
    parent_y: f64,
    sub: bool,
) -> (&LayoutBox, f64, f64, &LayoutBox, f64) {
    let x = parent_x + value.x;
    let y = parent_y + value.y;
    match &value.kind {
        LayoutKind::Row(items) if items.len() == 1 => script_box(&items[0], x, y, sub),
        LayoutKind::FontStyle { body, .. } => script_box(body, x, y, sub),
        LayoutKind::SubSup {
            base, sub: lower, ..
        } if sub => (lower, x + lower.x, y + lower.y, base, y + base.y),
        LayoutKind::Superscript { base, sup } if !sub => {
            (sup, x + sup.x, y + sup.y, base, y + base.y)
        }
        other => panic!("기대한 본체/첨자의 단일 소유 구조가 아니다: {other:?}"),
    }
}

fn scalar_baselines(value: &LayoutBox, parent_y: f64, text: &str, out: &mut Vec<f64>) {
    let y = parent_y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            // 독립 Print에서 읽은 원문 숫자·첨자와 HYhwpEQ cmap를 대응한다.
            let glyph = match text {
                "i" => '\u{e0ed}',
                "2" => '\u{e035}',
                "3" => '\u{e036}',
                "5" => '\u{e038}',
                _ => return,
            };
            if run.glyphs.chars().eq(std::iter::once(glyph)) {
                assert_eq!(run.positions.len(), 1, "원본 단일 문자의 표시 보존");
                out.push(y + value.baseline + run.baseline_shift);
            }
        }
        LayoutKind::Text(value_text)
        | LayoutKind::Number(value_text)
        | LayoutKind::FontGlyph {
            text: value_text, ..
        } if value_text == text || (text == "i" && value_text == "𝑖") => {
            out.push(y + value.baseline)
        }
        LayoutKind::Row(items) => {
            for item in items {
                scalar_baselines(item, y, text, out);
            }
        }
        LayoutKind::FontStyle { body, .. }
        | LayoutKind::Paren { body, .. }
        | LayoutKind::Sqrt { body, .. } => scalar_baselines(body, y, text, out),
        LayoutKind::Fraction { numer, denom, .. } => {
            scalar_baselines(numer, y, text, out);
            scalar_baselines(denom, y, text, out);
        }
        _ => {}
    }
}

fn radical_indexes<'a>(value: &'a LayoutBox, out: &mut Vec<&'a LayoutBox>) {
    match &value.kind {
        LayoutKind::Sqrt { index, body } => {
            if let Some(index) = index {
                out.push(index);
            }
            radical_indexes(body, out);
        }
        LayoutKind::Row(items) => {
            for item in items {
                radical_indexes(item, out);
            }
        }
        LayoutKind::FontStyle { body, .. } | LayoutKind::Paren { body, .. } => {
            radical_indexes(body, out)
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            radical_indexes(numer, out);
            radical_indexes(denom, out);
        }
        _ => {}
    }
}

// 사설 수식 face가 없는 CI에서도 실제 SVG의 문자 방출과 페이지 좌표를 검사한다.
// PUA를 공개 글리프로 바꾸거나 대체 글꼴의 윤곽을 한컴 잉크로 간주하지 않는다.
fn emitted_baselines(
    svg: &roxmltree::Document<'_>,
    owner: &RenderNode,
    glyphs: &[&str],
) -> Vec<f64> {
    use std::str::FromStr;
    let mut baselines = Vec::new();
    for node in svg.descendants().filter(|node| node.has_tag_name("text")) {
        if !glyphs.contains(&node.text().unwrap_or_default()) {
            continue;
        }
        let mut ancestors: Vec<_> = node.ancestors().filter(|node| node.is_element()).collect();
        ancestors.reverse();
        let mut transform = usvg::Transform::default();
        for ancestor in ancestors {
            if let Some(value) = ancestor.attribute("transform") {
                let t = svgtypes::Transform::from_str(value).expect("방출된 SVG의 유효한 변환");
                transform = transform.pre_concat(usvg::Transform::from_row(
                    t.a as f32, t.b as f32, t.c as f32, t.d as f32, t.e as f32, t.f as f32,
                ));
            }
        }
        let x = node.attribute("x").unwrap_or("0").parse::<f32>().unwrap();
        let y = node.attribute("y").unwrap_or("0").parse::<f32>().unwrap();
        let mut point = usvg::tiny_skia_path::Point::from_xy(x, y);
        transform.map_point(&mut point);
        let (x, y) = (f64::from(point.x), f64::from(point.y));
        if x >= owner.bbox.x
            && x <= owner.bbox.x + owner.bbox.width
            && y >= owner.bbox.y
            && y <= owner.bbox.y + owner.bbox.height
        {
            assert!(
                node.attribute("font-family").is_some(),
                "표시 face를 보존한다"
            );
            baselines.push(y);
        }
    }
    baselines
}

#[test]
fn fraction_subscript_keeps_its_source_owner_and_emits_below_the_denominator() {
    let core = load("extended-base-script-2024.hwp");
    assert_eq!(core.page_count(), 1, "독립 Print 전체 쪽수");
    let addresses = source_addresses(&core, SUBSUP);
    assert_eq!(addresses.len(), 3, "600/900/1500HU의 독립 대조군");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    equations(&tree.root, &mut nodes);
    let svg = core
        .render_page_svg_layer_with_profile_native(0, rhwp::paint::RenderProfile::Print)
        .unwrap();
    let painted = roxmltree::Document::parse(&svg).expect("최종 Print SVG");
    for &(section, paragraph, control, _) in &addresses {
        let found: Vec<_> = nodes
            .iter()
            .filter(|(_, eq)| owned(eq, (section, paragraph, control)))
            .collect();
        assert_eq!(found.len(), 1, "원본 수식의 누락·중복 없는 단일 방출");
        let (node, eq) = *found[0];
        assert_eq!(eq.script, SUBSUP);
        let (sub, _, sub_y, base, base_y) = script_box(&eq.layout_box, 0.0, 0.0, true);
        let mut denominator = Vec::new();
        scalar_baselines(base, base_y - base.y, "5", &mut denominator);
        assert_eq!(denominator.len(), 1, "근호 분모 5의 내용 보존");
        let mut sub_letters = Vec::new();
        scalar_baselines(sub, sub_y - sub.y, "i", &mut sub_letters);
        assert_eq!(sub_letters.len(), 1, "아래첨자 i의 내용 보존");
        assert!(
            sub_letters[0] > denominator[0],
            "독립 Print의 아래첨자는 분모의 숫자 기준선 아래에 있다"
        );
        let emitted_sub = emitted_baselines(&painted, node, &["i", "𝑖", "\u{e0ed}"]);
        let emitted_denom = emitted_baselines(&painted, node, &["5", "\u{e038}"]);
        assert_eq!(emitted_sub.len(), 1, "최종 SVG의 원본 아래첨자 i 단일 방출");
        assert_eq!(emitted_denom.len(), 1, "최종 SVG의 원본 분모 5 단일 방출");
        assert!(
            emitted_sub[0] > emitted_denom[0],
            "최종 SVG의 실제 변환 뒤에도 i 기준선은 분모 5보다 아래여야 한다"
        );
    }
}

#[test]
fn contained_root_index_preserves_the_same_power_baseline_as_the_plain_root() {
    let core = load("extended-base-script-v4-2024.hwp");
    assert_eq!(core.page_count(), 4, "독립 Print 전체 쪽수");
    let plain = source_addresses(&core, ROOT_SUP);
    let indexed = source_addresses(&core, INDEX_SUP);
    assert_eq!(plain.len(), 15);
    assert_eq!(indexed.len(), 15);
    let mut observed = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        equations(&tree.root, &mut nodes);
        for (node, eq) in nodes {
            if eq.script != ROOT_SUP && eq.script != INDEX_SUP {
                continue;
            }
            let (sup, _, y, base, _) = script_box(&eq.layout_box, 0.0, 0.0, false);
            let mut indexes = Vec::new();
            radical_indexes(base, &mut indexes);
            if eq.script == INDEX_SUP {
                assert_eq!(indexes.len(), 1, "근 지수의 단일 소유와 누락 없는 보존");
                let mut degree = Vec::new();
                scalar_baselines(indexes[0], 0.0, "3", &mut degree);
                assert_eq!(
                    degree.len(),
                    1,
                    "세제곱근의 3을 없애 평범한 근호처럼 통과하면 안 된다"
                );
                assert!(
                    indexes[0].font_size < base.font_size,
                    "Print처럼 근 지수는 본체보다 작은 글자다"
                );
            } else {
                assert!(indexes.is_empty(), "무지수 대조군");
            }
            let mut powers = Vec::new();
            scalar_baselines(sup, y - sup.y, "2", &mut powers);
            assert_eq!(powers.len(), 1, "제곱 첨자의 내용 보존");
            let source = if eq.script == ROOT_SUP {
                &plain
            } else {
                &indexed
            };
            let matches: Vec<_> = source
                .iter()
                .filter(|&&(s, p, c, _)| owned(eq, (s, p, c)))
                .collect();
            assert_eq!(matches.len(), 1, "원본과 출력의 수식 소유 대응");
            observed.push((
                eq.script.clone(),
                matches[0].3,
                powers[0] / eq.font_size,
                node.id,
            ));
        }
    }
    assert_eq!(observed.len(), 30, "누락·중복 없는 두 종류의 첨자");
    for &(_, _, _, unit) in &plain {
        let a: Vec<_> = observed
            .iter()
            .filter(|x| x.0 == ROOT_SUP && x.1 == unit)
            .collect();
        let b: Vec<_> = observed
            .iter()
            .filter(|x| x.0 == INDEX_SUP && x.1 == unit)
            .collect();
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        // 저장 높이의 한 HWPUNIT 양자화 오차를 글자 크기의 비율로 표현한다.
        let precision = 1.0 / f64::from(unit) + f64::EPSILON * 16.0;
        assert!((a[0].2-b[0].2).abs()<=precision,"같은 줄 안에 담긴 근 지수는 분수의 제곱 기준선을 바꾸지 않는다: unit={unit}, plain={}, indexed={}",a[0].2,b[0].2);
    }
}
