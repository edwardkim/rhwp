//! 한컴 Mac 12.30 글자 겹치기 창의 14개 모양을 읽고 그리고 저장한다.
//!
//! `samples/char_overlap/hancom-mac-shapes.*`는 한컴이 만든 같은 문서의 HWP·HWPX·Print PDF다.
//! 첫 문단은 "12"를 창의 모양 순서(원, 글자끼리, 반전 원, 사각형 … 굵은 순환 삼각형)로 넣었다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const ORDER: [u8; 14] = [1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

fn open(ext: &str) -> DocumentCore {
    let path = format!(
        "{}/samples/char_overlap/hancom-mac-shapes.{ext}",
        env!("CARGO_MANIFEST_DIR")
    );
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap()
}

fn first_row_borders(core: &DocumentCore) -> Vec<u8> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::CharOverlap(overlap) => Some(overlap.border_type),
            _ => None,
        })
        .collect()
}

fn page_text(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

#[test]
fn hwpx_keeps_all_fourteen_shapes_through_save() {
    let core = open("hwpx");
    assert_eq!(first_row_borders(&core), ORDER);
    let saved = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(first_row_borders(&saved), ORDER);
}

#[test]
fn hancom_border_glyphs_are_painted_for_every_shape() {
    for ext in ["hwpx", "hwp"] {
        let text = page_text(&open(ext));
        for glyph in ['△', '▲', '☼', '◇', '◆', '▢', '♲', '♺', '♻'] {
            assert!(text.contains(glyph), "{ext}: {glyph} 테두리가 없다");
        }
    }
}

/// 한컴 HWP는 반전 원·사각형·반전 사각형 안의 두 자리 숫자를 자릿수별 PUA로 저장한다.
#[test]
fn hwp_two_digit_pua_shapes_draw_the_number() {
    let text = page_text(&open("hwp"));
    assert!(
        !text
            .chars()
            .any(|ch| (0xF0288..=0xF02E9).contains(&u32::from(ch))),
        "PUA 숫자가 그대로 남았다"
    );
    assert!(text.matches("12").count() >= 4, "{text}");
}

fn overlap_widths(node: &rhwp::renderer::render_tree::RenderNode, out: &mut Vec<(f64, f64)>) {
    if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
        if run.char_overlap.is_some() && run.para_index == Some(0) {
            out.push((node.bbox.width, run.style.font_size));
        }
    }
    for child in &node.children {
        overlap_widths(child, out);
    }
}

/// 한컴 Print는 14모양 모두 전각 한 칸(10pt 9.66pt)에 그린다. PUA 원 숫자를 표시 글자로
/// 다시 재 1.5칸이 되거나, 펼친 글자끼리 겹치기가 반각 숫자 폭이 되면 뒤 글자가 밀린다.
#[test]
fn every_shape_takes_one_full_width_cell() {
    for ext in ["hwp", "hwpx"] {
        let core = open(ext);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut widths = Vec::new();
        overlap_widths(&tree.root, &mut widths);
        assert_eq!(widths.len(), 14, "{ext}");
        for (width, size) in widths {
            assert!(
                (width - size).abs() < 1.0,
                "{ext}: 폭 {width} / 글자 크기 {size}"
            );
        }
    }
}
