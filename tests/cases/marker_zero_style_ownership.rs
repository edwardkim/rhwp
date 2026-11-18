//! 관계형: 유효한 0번 머리 글자 모양과 같은 모양의 다른 ID는 동일한 마커·본문을 만든다.
//! R33 근거 갱신 2026-11-07T13:58:21+09:00; 직전 파일 변경 f00d9a4895ada1212bfd23c0ea4e941024c847d0 (2026-11-07T11:03:07+09:00).
//! 제품 ef77d740e1, 전94쪽 96 DPI·Print raw 최저 Native 99.98% / fresh WASM 99.99%.
//! 검사5쪽은 각각100.00%/99.99%, 독립 PDF와94쪽 일치·직접 확대 판독 완료.
//! 최신 증거: mydocs/pr/assets/planet6897_20261106/pr8611/r33-source-symbol/.
//! 주석 갱신: 2026-11-07T10:50:04+09:00. 새 파일이므로 직전 변경 커밋은 없다.
//! 2026-11-07, 제품 91109ff833dd5e29e120bb960b7410f15dae1e8f의 수정 전 실제 측정.
//! samples/basic/sungeo.hwp ↔ pdf/basic/sungeo-hwp-2020.pdf, 전체 94쪽이 양쪽에서 일치한다.
//! Print·96 DPI·보정 전 2px raw: Native/fresh WASM 전쪽 최저 및 검사 5쪽 각각 98.93%.
//! 같은 쪽 review/overlay 직접 판독과 원본 DocInfo BULLET·글자 모양 바이트를 대조했다.
//! 증거: mydocs/pr/assets/planet6897_20261106/pr8611/r29-before-zero-reference/.
//! 독립 Print는 약 14pt·#000080·양각 마커다. 원본은 0번 글자 모양 1400HU·같은 색·양각을
//! 참조하고, 실제 본문은 10번 글자 모양 1300HU다. 0을 참조 없음으로 보아서는 안 된다.
//! 입력의 크기 환산은 원본 속성 계약이며 실제 배치의 절대 px 핀이나 설치 글꼴을 요구하지 않는다.
//! 전체 source·Native/fresh WASM의 base 대비 무회귀 완료는 이 관계 검사의 통과와 별도다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode},
    DocumentCore,
};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("정식 고사성어 원본")).unwrap()
}

fn runs<'a>(node: &'a RenderNode, output: &mut Vec<(&'a RenderNode, &'a TextRunNode)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(1) && run.para_index == Some(63) {
            output.push((node, run));
        }
    }
    for child in &node.children {
        runs(child, output);
    }
}

#[test]
fn zero_is_a_valid_source_style_with_its_own_size_color_and_emboss() {
    let core = load();
    let source = &core.document().doc_info.char_shapes[0];
    assert_eq!(core.document().doc_info.bullets[0].char_shape_id, 0);
    let tree = core.build_page_render_tree(4).unwrap();
    let mut output = Vec::new();
    runs(&tree.root, &mut output);
    let marker = output
        .iter()
        .find(|(_, run)| run.char_start.is_none())
        .unwrap()
        .1;
    assert_eq!(marker.style.font_size, f64::from(source.base_size) / 75.0);
    assert_eq!(marker.style.color, source.text_color);
    assert_eq!(marker.style.emboss, source.emboss);
    let text = output
        .iter()
        .filter(|(_, run)| run.char_start.is_some())
        .map(|(_, run)| run.text.as_str())
        .collect::<String>();
    assert_eq!(text, core.document().sections[1].paragraphs[63].text);
}

fn owned_geometry(core: &DocumentCore) -> Vec<(String, f64, f64, f64, f64, u32, bool)> {
    let tree = core.build_page_render_tree(4).unwrap();
    let mut output = Vec::new();
    runs(&tree.root, &mut output);
    output
        .into_iter()
        .map(|(node, run)| {
            (
                run.text.clone(),
                node.bbox.x,
                node.bbox.y,
                node.bbox.width,
                run.style.font_size,
                run.style.color,
                run.style.emboss,
            )
        })
        .collect()
}

#[test]
fn aliasing_the_same_source_style_does_not_move_marker_or_body() {
    let original = load();
    let mut alias = load();
    let mut document = alias.document().clone();
    let id = document.doc_info.char_shapes.len() as u32;
    document
        .doc_info
        .char_shapes
        .push(document.doc_info.char_shapes[0].clone());
    document.doc_info.bullets[0].char_shape_id = id;
    alias.set_document(document);
    assert_eq!(owned_geometry(&original), owned_geometry(&alias));
}

#[test]
fn explicit_no_reference_still_inherits_the_paragraph_style() {
    let mut core = load();
    let mut document = core.document().clone();
    document.doc_info.bullets[0].char_shape_id = u32::MAX;
    core.set_document(document);
    let body = &core.document().doc_info.char_shapes[10];
    let tree = core.build_page_render_tree(4).unwrap();
    let mut output = Vec::new();
    runs(&tree.root, &mut output);
    let marker = output
        .iter()
        .find(|(_, run)| run.char_start.is_none())
        .unwrap()
        .1;
    assert_eq!(marker.style.font_size, f64::from(body.base_size) / 75.0);
    assert_eq!(marker.style.color, body.text_color);
    assert_eq!(marker.style.emboss, body.emboss);
}
