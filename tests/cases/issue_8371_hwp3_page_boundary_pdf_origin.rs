//! 검사 유형: 관계형·고정형. 글꼴 파일이나 시스템 글꼴에 의존하지 않는 쪽 경계 검사.
//! 주석 갱신: 2026-10-30T01:43:46+09:00; 신규 파일로 직전 파일 커밋 없음.
//! 원본 `samples/hwp3-sample10.hwp`, 한컴 2020 Print, 96dpi, compat2022.
//! 312쪽 Native/fresh WASM raw는 각각 100.00%, 두 backend 전763쪽 최저는 90% 이상이다.
//! 독립 PDF 312쪽 첫 `SQL)`의 글자 원점은 (113.3786, 142.4044) CSS px다.
//! 첫 좌표는 글줄 시작, 둘째 좌표는 기준선이다. 원본 Print에서 PDFium 문자 원점을 읽었다.
//! 전쪽 TSV와 대표 이미지는 mydocs/pr/assets/issue_8371/, 출처는
//! mydocs/working/task_m100_8371_stage1.md를 따른다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(_)) {
        found.push(node);
    }
    for child in &node.children {
        collect(child, found);
    }
}

#[test]
fn cached_page_boundary_keeps_source_order_and_pdf_line_origin() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample10.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 763, "한컴 Print의 물리 쪽수");
    let previous = core.build_page_render_tree(310).unwrap();
    let current = core.build_page_render_tree(311).unwrap();
    let mut prev = Vec::new();
    let mut next = Vec::new();
    collect(&previous.root, &mut prev);
    collect(&current.root, &mut next);
    fn text(node: &RenderNode) -> &str {
        match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.as_str(),
            _ => "",
        }
    }
    assert!(
        prev.iter().any(|node| text(node) == "생성 점"),
        "선행 쪽의 제목 보존"
    );
    assert!(
        !next.iter().any(|node| text(node) == "생성 점"),
        "제목의 쪽 중복 방지"
    );
    let matching: Vec<_> = next
        .iter()
        .filter(|node| text(node).contains("SQL) select * from deptsnap2;"))
        .collect();
    assert_eq!(matching.len(), 1, "현재 쪽의 SQL 문단은 정확히 한 번 표시");
    let node = *matching[0];
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    let baseline = node.bbox.y + run.baseline;
    assert!(
        (node.bbox.x - 113.3786).abs() <= 1.0,
        "독립 PDF의 첫 글자 x 원점: {}",
        node.bbox.x
    );
    assert!(
        (baseline - 142.4044).abs() <= 1.0,
        "독립 PDF의 글줄 기준선: {baseline}"
    );
    let following = next
        .iter()
        .find(|node| text(node).contains("MASTER SITE (REMOTE)"))
        .expect("같은 쪽 후속 본문");
    assert!(
        node.bbox.y < following.bbox.y,
        "SQL 문단은 후속 본문보다 앞에 있어야 한다"
    );
}
