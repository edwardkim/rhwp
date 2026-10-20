//! 머리말·꼬리말 마당 6~10(진하게·밑줄)이 1~5와 똑같이 그려지던 결함의 회귀 가드.
//!
//! 새 머리말·꼬리말 문단(`Paragraph::default()`)은 글자 모양 목록이 비어 있다. 마당 적용은
//! 그 목록에 이미 있는 항목의 ID만 바꿔, 진하게·밑줄 모양을 만들고도 글자에 붙이지 못했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::UnderlineType;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// 첫 쪽 머리말(꼬리말)에 그린 글자 run 마다 (글자, 진하게, 밑줄).
fn drawn_styles(doc: &HwpDocument, is_header: bool) -> Vec<(String, bool, UnderlineType)> {
    let tree = doc.build_page_render_tree(0).expect("쪽 렌더 트리");
    let mut runs = Vec::new();
    collect_runs(&tree.root, is_header, false, &mut runs);
    runs
}

fn collect_runs(
    node: &RenderNode,
    is_header: bool,
    inside: bool,
    out: &mut Vec<(String, bool, UnderlineType)>,
) {
    let inside = inside
        || match node.node_type {
            RenderNodeType::Header => is_header,
            RenderNodeType::Footer => !is_header,
            _ => false,
        };
    if let (true, RenderNodeType::TextRun(run)) = (inside, &node.node_type) {
        let text = run.display_or_text().trim();
        if !text.is_empty() {
            out.push((text.to_string(), run.style.bold, run.style.underline));
        }
    }
    for child in &node.children {
        collect_runs(child, is_header, inside, out);
    }
}

/// 마당 6~10 은 모든 글자를 진하게·밑줄로, 1~5 는 보통 글자로 그린다. 저장 후 다시 열어도 같다.
#[test]
fn styled_templates_draw_bold_underline_after_save_and_reopen() {
    let mut wrong = Vec::new();
    for (is_header, area) in [(true, "머리말"), (false, "꼬리말")] {
        for template in 1..=10u8 {
            let mut doc = HwpDocument::create_empty();
            doc.create_blank_document_native().expect("빈 문서");
            doc.set_file_name("보고서.hwp");
            doc.apply_hf_template_native(0, is_header, 0, template)
                .expect("마당 적용");
            let styled = template > 5;
            let underline = if styled {
                UnderlineType::Bottom
            } else {
                UnderlineType::None
            };

            let hwp = HwpDocument::from_bytes(&doc.export_hwp().expect("HWP 저장"))
                .expect("HWP 다시 열기");
            let hwpx = HwpDocument::from_bytes(&doc.export_hwpx().expect("HWPX 저장"))
                .expect("HWPX 다시 열기");
            for (label, doc) in [("저장 전", &doc), ("HWP", &hwp), ("HWPX", &hwpx)] {
                let runs = drawn_styles(doc, is_header);
                assert!(
                    !runs.is_empty(),
                    "{area} 마당 {template} {label}: 그린 글자가 없다"
                );
                for (text, bold, line) in runs {
                    if (bold, line) != (styled, underline) {
                        wrong.push(format!(
                            "{area} 마당 {template} {label} {text:?}: 진하게 {bold}, 밑줄 {line:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "마당과 다르게 그린 글자 {}개:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
