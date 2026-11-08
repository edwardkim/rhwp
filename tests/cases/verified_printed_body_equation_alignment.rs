//! 유효한 용지·한컴 저장 메트릭으로 만든 본문 수식의 독립 Print 계약.
//! 출처와 이전 합성 입력의 한계는 PRINTED_EQUATION_ALIGNMENT.md를 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn load(column_start: i32) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-stored-equation-print-recheck-20261017")
        .join(format!("calibrated-equation-cache-{column_start}.hwpx"));
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn find<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a EquationNode)>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        out.push((node, eq));
    }
    for child in &node.children {
        find(child, out);
    }
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn glyphs(item: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64)>) {
    let (x, y) = (x + item.x, y + item.y);
    match &item.kind {
        LayoutKind::HancomGlyphs(run) => out.extend(
            run.glyphs
                .chars()
                .zip(&run.positions)
                .map(|(ch, dx)| (ch, x + dx, y + item.baseline + run.baseline_shift)),
        ),
        LayoutKind::Row(children) => {
            for child in children {
                glyphs(child, x, y, out);
            }
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            glyphs(numer, x, y, out);
            glyphs(denom, x, y, out);
        }
        LayoutKind::FontStyle { body, .. } => glyphs(body, x, y, out),
        _ => {}
    }
}

#[test]
fn printed_body_fraction_preserves_source_ownership_and_center_alignment() {
    for column_start in [0, 6000] {
        let core = load(column_start);
        assert_eq!(core.page_count(), 1, "독립 Print와 전체 쪽수 일치");
        let para = &core.document().sections[0].paragraphs[0];
        assert_eq!(para.line_segs[0].column_start, column_start);
        let source: Vec<_> = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(index, control)| match control {
                Control::Equation(eq) => Some((index, eq)),
                _ => None,
            })
            .collect();
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].1.script, "1 over 2");
        let tree = core.build_page_render_tree(0).unwrap();
        let area = body(&tree.root).expect("본문 영역").bbox;
        let mut found = Vec::new();
        find(&tree.root, &mut found);
        assert_eq!(found.len(), source.len(), "수식의 누락·중복 없음");
        let (node, eq) = found[0];
        assert_eq!(eq.section_index, Some(0));
        assert_eq!(eq.para_index, Some(0));
        assert_eq!(eq.control_index, Some(source[0].0));
        assert_eq!(eq.script, source[0].1.script);
        assert!(node.bbox.x >= area.x && node.bbox.x + node.bbox.width <= area.x + area.width);
        assert!(node.bbox.y >= area.y && node.bbox.y + node.bbox.height <= area.y + area.height);
        let left = node.bbox.x - area.x;
        let right = area.x + area.width - node.bbox.x - node.bbox.width;
        assert!(
            (left - right).abs() <= eq.font_size / 1024.0,
            "본문 양쪽 여백 대칭"
        );
    }
}

#[test]
fn completed_body_fraction_keeps_independent_print_digit_origins_within_two_pixels() {
    for column_start in [0, 6000] {
        let core = load(column_start);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut found = Vec::new();
        find(&tree.root, &mut found);
        assert_eq!(found.len(), 1);
        let (node, eq) = found[0];
        let mut ink = Vec::new();
        glyphs(&eq.layout_box, node.bbox.x, node.bbox.y, &mut ink);
        let digits: Vec<_> = ink
            .iter()
            .filter(|(ch, _, _)| matches!(ch, '\u{e034}' | '\u{e035}'))
            .collect();
        assert_eq!(digits.len(), 2, "분자와 분모를 한 번씩 표시");
        assert_eq!(digits[0].0, '\u{e034}', "원본 분자 1 보존");
        assert_eq!(digits[1].0, '\u{e035}', "원본 분모 2 보존");
        assert!(digits[0].2 < digits[1].2, "분자·분모의 세로 순서");
        // 독립 한컴 Print의 CSS 96dpi 글리프 원점. 두 경로 raw 100%인 1쪽이다.
        for (actual, expected_y) in [digits[0], digits[1]]
            .into_iter()
            .zip([86.94499, 103.88646])
        {
            assert!(
                (actual.1 - 390.82817).abs() <= 2.0,
                "독립 PDF x 원점: {actual:?}"
            );
            assert!(
                (actual.2 - expected_y).abs() <= 2.0,
                "독립 PDF 기준선: {actual:?}"
            );
        }
    }
}
