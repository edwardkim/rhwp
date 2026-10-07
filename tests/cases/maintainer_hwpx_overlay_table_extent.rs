//! 하단 문단의 글 뒤로 표는 자리차지 표 뒤에 별도 흐름 높이로 더하지 않는다.
//! 독립 한컴 PDF의 1쪽과 하단 전화 기준선이 속한 글줄로 실제 내용 소유를 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn verify_footer(sample: &str, phone: &str, pdf_phone_baseline: f64) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("원본 문서 읽기");
    let document = HwpDocument::from_bytes(&bytes).expect("원본 문서 로드");
    assert_eq!(
        document.page_count(),
        1,
        "기준PDF의 하단 블록이 다음 쪽으로 밀렸다: {sample}"
    );
    let root = document.build_page_render_tree(0).expect("첫 쪽 배치").root;
    fn collect(node: &RenderNode, phone: &str, positions: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.display_or_text().contains(phone) {
                positions.push((node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
        for child in &node.children {
            collect(child, phone, positions);
        }
    }
    let mut positions = Vec::new();
    collect(&root, phone, &mut positions);
    assert_eq!(
        positions.len(),
        1,
        "하단 전화의 누락 또는 중복: {sample}, {positions:?}"
    );
    // PDF 글자 잉크의 상단과 렌더 트리 글줄 상단은 서로 다른 메트릭이다.
    // 같은 96dpi 좌표계의 PDF 기준선이 실제 글줄에 속하는지 검사한다.
    // 이 검사는 세로 위치의 완전한 일치를 주장하지 않으며 직접 overlay를 함께 판독한다.
    assert!(
        positions[0].0 <= pdf_phone_baseline && pdf_phone_baseline <= positions[0].1,
        "독립 PDF 전화 기준선이 실제 글줄 밖이다: {sample}, 실제{:?}, 기준{pdf_phone_baseline}",
        positions[0]
    );
}

#[test]
fn page_anchored_footer_keeps_its_independent_painted_position() {
    verify_footer(
        "samples/issue6535/36404612_page_anchored_footer_block.hwpx",
        "02-6942-1306",
        1040.7825520833333,
    );
}

#[test]
fn overtime_footer_keeps_its_independent_painted_position() {
    verify_footer(
        "samples/issue6102/36310257_overtime_report.hwpx",
        "02-6981-7551",
        1044.0,
    );
}
