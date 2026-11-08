//! 관계형: 미해결 외부 그림 글머리표의 미출력·본문 들여쓰기를 검사한다.
//! 주석 갱신: 2026-11-06T18:03:38+09:00.
//! 2026-11-06 측정: 독립 한컴2020 Print, compat2022·96DPI·PDFium·2px raw.
//! paragraph-head-edited.hwp 전1쪽 Native 100.00000% / fresh WASM 100.00000%; 전쪽 최저도 같다.
//! 근거: mydocs/pr/assets/issue_8594/linked-picture-bullet-20261106/README.md.
//! 설치 글꼴이나 절대 px 좌표에 의존하지 않으며 실제 결함으로 수정 전 FAIL → 수정 후 PASS다.
//! 독립 Hancom 2020 Print의 Golf 문단은 미해결 외부 그림 글머리표를 출력하지 않는다.
//! 원본의 그림 참조·외부 Link 속성, 실제 마커 부재와 본문 소유를 함께 확인한다.
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}
#[test]
fn unresolved_link_picture_bullet_keeps_indent_without_placeholder_or_character() {
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/semanticist21_next10_review/paragraph-head-edited.hwp"),
    )
    .unwrap();
    let core = DocumentCore::from_bytes(&input).unwrap();
    let doc = core.document();
    let (pi, para) = doc.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("Golf picture bullet"))
        .unwrap();
    let style = &doc.doc_info.para_shapes[para.para_shape_id as usize];
    let bullet = &doc.doc_info.bullets[usize::from(style.numbering_id) - 1];
    let reference = bullet
        .image_reference()
        .expect("외부 그림 참조가 실제로 있다");
    let record = &doc.doc_info.bin_data_list[usize::from(reference) - 1];
    assert_eq!(record.data_type, rhwp::model::bin_data::BinDataType::Link);
    assert!(
        doc.bin_data_content.iter().all(|item| item.data.is_empty()),
        "외부 그림 바이트가 없는 원본"
    );
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    assert!(!nodes.iter().any(|node|matches!(&node.node_type,RenderNodeType::Image(image) if image.para_index==Some(pi))),"외부 그림이 없을 때 빈 그림 상자를 출력하지 않는다");
    let marker = bullet.bullet_char.to_string();
    assert!(!nodes.iter().any(|node|matches!(&node.node_type,RenderNodeType::TextRun(run) if run.para_index==Some(pi)&&run.text==marker)),"참조된 외부 그림 누락은 문자 글머리표로 바꾸지 않는다");
    let line=nodes.iter().find(|node|node.children.iter().any(|child|matches!(&child.node_type,RenderNodeType::TextRun(run) if run.para_index==Some(pi)&&run.text.starts_with("Golf picture bullet")))).expect("그림과 독립적인 본문 첫 줄");
    let body=line.children.iter().find(|node|matches!(&node.node_type,RenderNodeType::TextRun(run) if run.para_index==Some(pi)&&run.text.starts_with("Golf picture bullet"))).unwrap();
    assert!(
        body.bbox.x > line.bbox.x,
        "마커가 없어도 본문 들여쓰기는 유지한다"
    );
}
