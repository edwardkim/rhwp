//! 검사 유형: 관계형.
//! 주석 갱신: 2026-11-03T01:30:17+09:00.
//! 2026-11-03 독립 한컴 2020 Print, 96 DPI, 2px raw: 저장 HWPX 전 12쪽
//! Native 최저 99.99%, fresh WASM 최저 100.00%; 12쪽 각각 100.00%, 100.00%.
//! 근거: mydocs/pr/archives/pr_8316_review.md의 HWPX 저장본의 12쪽 원점 재검토.
//! 저장 전후 12쪽의 표·후속 본문 소속과 상대 위치를 비교하며 설치 글꼴을 요구하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn collect<'a>(node: &'a RenderNode, tables: &mut Vec<BoundingBox>, tail: &mut Vec<BoundingBox>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        tables.push(node.bbox);
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains("Blinker-light") {
            tail.push(node.bbox);
        }
    }
    for child in &node.children {
        collect(child, tables, tail);
    }
}

#[test]
fn 재구성_단독_표의_저장_전후_쪽과_후속_본문_위치가_유지된다() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp");
    let source =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 HWP")).expect("원본 열기");
    let saved = DocumentCore::from_bytes(&source.export_hwpx_native().expect("HWPX 저장"))
        .expect("저장본 열기");
    assert_eq!(source.page_count(), saved.page_count(), "저장 전후 쪽수");
    let mut source_tables = Vec::new();
    let mut source_tail = Vec::new();
    let mut saved_tables = Vec::new();
    let mut saved_tail = Vec::new();
    collect(
        &source.build_page_render_tree(11).expect("원본 12쪽").root,
        &mut source_tables,
        &mut source_tail,
    );
    collect(
        &saved.build_page_render_tree(11).expect("저장본 12쪽").root,
        &mut saved_tables,
        &mut saved_tail,
    );
    assert_eq!(source_tables.len(), 1, "원본 표 소속");
    assert_eq!(saved_tables.len(), 1, "저장본 표 소속");
    assert_eq!(source_tail.len(), 1, "원본 후속 본문 소속");
    assert_eq!(saved_tail.len(), 1, "저장본 후속 본문 소속");
    for (table, tail) in [
        (source_tables[0], source_tail[0]),
        (saved_tables[0], saved_tail[0]),
    ] {
        assert!(
            table.y + table.height <= tail.y,
            "표 뒤에 후속 본문이 위치해야 함"
        );
    }
    let source_offset = source_tail[0].y - source_tables[0].y;
    let saved_offset = saved_tail[0].y - saved_tables[0].y;
    assert!(
        (source_offset - saved_offset).abs() < source_tables[0].height * 0.01,
        "표와 후속 본문의 상대 간격 보존"
    );
    assert!(
        (source_tables[0].y - saved_tables[0].y).abs() <= 2.0,
        "검증된 12쪽 표 원점의 저장 전후 차이"
    );
}
