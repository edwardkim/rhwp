//! 검사 유형: 관계형 — 자동 내어쓰기의 마커·본문 영역과 저장 줄의 들여쓰기 소유를 대조한다.
//! 주석 갱신: 2026-11-09T15:55:33.365546+09:00.
//! 원본 report-628-original.hwp의 95·96쪽에 해당하는 구역3 문단469·474를 각각 분리한다.
//! 원문·글자모양·개요 속성·저장 LineSeg는 보존하며, 대조군은 문단 들여쓰기만 0으로 바꾼다.
//! 독립 Print: pdf/planet6897-20261106/pr8642/report-628-original-2020.pdf.
//! source93 전쪽 비교 중 해당 쪽 Native 99.20/99.23%, fresh WASM 99.19/99.21%지만,
//! 번호 원점이 PDF의132.32 CSSpx보다13.33px 왼쪽이다. 점수 통과를 위치 일치로 간주하지 않는다.
//! 이 검사는 실물 절대 좌표 핀이 아니며, 설치 글꼴 없이 줄 상자·번호·본문의 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::paragraph::{LineSeg, NumberingMarker},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn geometry(paragraph: usize, zero_indent: bool) -> (f64, f64, f64) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/planet6897_20261106/pr8642/report-628-original.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut doc = core.document().clone();
    let mut section = doc.sections[3].clone();
    let mut para = section.paragraphs[paragraph].clone();
    let source = para.line_segs.first().unwrap();
    assert_eq!(source.tag & LineSeg::TAG_INDENTATION, 0);
    assert_eq!(source.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY, 0);
    assert_eq!(para.line_segs.len(), 1, "원본 한 줄의 소유를 보존한다");
    let body = para.text.clone();
    let shape = &mut doc.doc_info.para_shapes[para.para_shape_id as usize];
    assert!(
        shape.indent > 0,
        "자동 내어쓰기와 함께 저장한 양수 들여쓰기"
    );
    assert_eq!(shape.margin_left, 0);
    if zero_indent {
        shape.indent = 0;
        shape.raw_data = None;
    }
    para.numbering_marker = NumberingMarker::Unresolved;
    section.paragraphs = vec![para];
    doc.sections = vec![section];
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    fn walk(n: &RenderNode, body: &str, out: &mut (Option<f64>, Option<f64>, Option<f64>)) {
        if matches!(n.node_type, RenderNodeType::Column(_)) {
            out.0 = Some(n.bbox.x);
        }
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if r.text == "1." || r.text == "1)" {
                assert!(
                    out.1.replace(n.bbox.x).is_none(),
                    "마커를 중복 배치하지 않는다"
                );
            }
            if r.text == body {
                assert!(
                    out.2.replace(n.bbox.x).is_none(),
                    "본문을 중복 배치하지 않는다"
                );
            }
        }
        for c in &n.children {
            walk(c, body, out);
        }
    }
    let mut out = (None, None, None);
    walk(&tree.root, &body, &mut out);
    (out.0.unwrap(), out.1.unwrap(), out.2.unwrap())
}

fn preserves_marker_column(paragraph: usize) {
    let (column, marker, text) = geometry(paragraph, false);
    let (_, no_indent_marker, no_indent_text) = geometry(paragraph, true);
    assert!((marker - column).abs() <= 0.1,
        "자동 내어쓰기는 양수 indent를 먼저 빼고 저장 플래그로 다시 버리지 않는다: column={column} marker={marker} text={text}");
    assert!(
        (marker - no_indent_marker).abs() <= 0.1,
        "마커 영역이 indent보다 넓으면 같은 단 시작을 소유한다"
    );
    assert!(
        (text - no_indent_text).abs() <= 0.1,
        "마커가 차지한 동일 영역 뒤에서 본문을 한 번만 시작한다"
    );
    assert!(text > marker, "번호 다음에 원문이 이어진다");
}

#[test]
fn outline_dot_marker_keeps_the_source_column_with_auto_indent() {
    preserves_marker_column(469);
}

#[test]
fn outline_paren_marker_keeps_the_source_column_with_auto_indent() {
    preserves_marker_column(474);
}
