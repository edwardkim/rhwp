//! 공개 gist 원본의 수식 소유·순서와 최종 레이아웃의 로만 단위 글자.
//!
//! 원본 SHA-256:351d4b8b482dadffa2cf3a79037ad9397563ab670ce33c3770254c35d5c20a93.
//! 독립 Print:pdf/issue7873/eq-rm-quarterspace-original-2020.pdf,1쪽,
//! 5db93c43998d89a9000f964f1a170ec4c46d5ff9e250090b40afca04a6760b93.
//! f7ddf477e의96dpi/print 전1쪽 Native98.48018/WASM98.24952%와 양쪽 직접
//! 판독을 확인했다. Print의 m/s는 정체이며 역따옴표는 보이는 글자가 아니다.
//! 첫 수식의 독립 글자 원점127.52/기준선444.48px는±2px로 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::equation::symbols::FontStyleKind;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn unit_letters(layout: &LayoutBox, style: Option<FontStyleKind>, out: &mut String) {
    match &layout.kind {
        LayoutKind::Row(children) => {
            for child in children {
                unit_letters(child, style, out);
            }
        }
        LayoutKind::Fraction { numer, denom } => {
            unit_letters(numer, style, out);
            unit_letters(denom, style, out);
        }
        LayoutKind::FontStyle { style, body } => unit_letters(body, Some(*style), out),
        LayoutKind::HancomGlyphs(run) => visible_units(&run.glyphs, style, out),
        LayoutKind::Text(text) | LayoutKind::Function(text) => visible_units(text, style, out),
        LayoutKind::Number(_)
        | LayoutKind::Symbol(_)
        | LayoutKind::MathSymbol(_)
        | LayoutKind::Space(_)
        | LayoutKind::Empty => {}
        other => panic!("원본 세 수식의 예상하지 않은 배치 종류: {other:?}"),
    }
}

fn visible_units(text: &str, style: Option<FontStyleKind>, out: &mut String) {
    assert!(!text.contains('`'), "rm 뒤 간격 토큰은 가시 글자가 아니다");
    for ch in text.chars().filter(|ch| matches!(ch, 'm' | 's')) {
        assert_eq!(style, Some(FontStyleKind::Roman), "독립 Print의 정체 단위");
        out.push(ch);
    }
}

#[test]
fn original_equations_keep_control_ownership_roman_units_and_print_origin() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue7873/eq-rm-quarterspace-original.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print 전1쪽");
    let source: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(pi, p)| {
            p.controls.iter().enumerate().filter_map(move |(ci, c)| {
                if let Control::Equation(eq) = c {
                    Some((pi, ci, eq.script.as_str()))
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(
        source.iter().map(|(_, _, s)| *s).collect::<Vec<_>>(),
        [
            "x=6 rm m",
            "{변위의``크기} over {걸린``시간}",
            "{4 rm ``m} over {8 rm ``s} = {1} over {2} rm ``m/s",
        ],
        "공개 원본의 스크립트를 바꾸지 않는다"
    );
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Column(0)))
        .unwrap();
    let equations: Vec<_> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Equation(eq) => Some((*n, eq)),
            _ => None,
        })
        .collect();
    assert_eq!(equations.len(), source.len(), "세 수식의 누락·중복 없음");
    for (((node, eq), &(pi, ci, script)), independent_units) in
        equations.iter().zip(&source).zip(["m", "", "msms"])
    {
        assert_eq!(eq.section_index, Some(0));
        assert_eq!(eq.para_index, Some(pi));
        assert_eq!(eq.control_index, Some(ci));
        assert_eq!(eq.script, script, "원본 소유와 표시 순서");
        assert!(node.bbox.width > 0.0 && node.bbox.height > 0.0);
        assert!(node.bbox.x >= column.bbox.x);
        assert!(node.bbox.y >= column.bbox.y);
        assert!(node.bbox.x + node.bbox.width <= column.bbox.x + column.bbox.width);
        assert!(node.bbox.y + node.bbox.height <= column.bbox.y + column.bbox.height);
        let mut units = String::new();
        unit_letters(&eq.layout_box, None, &mut units);
        assert_eq!(units, independent_units, "최종 배치에서 정체 m/s 보존");
    }
    let (first, equation) = equations[0];
    assert!((first.bbox.x - 127.52).abs() <= 2.0);
    assert!((first.bbox.y + equation.layout_box.baseline - 444.48).abs() <= 2.0);
    assert!(equations.windows(2).all(|p| p[0].0.bbox.y < p[1].0.bbox.y));
}
