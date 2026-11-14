//! 관계형: 같은 셀의 주석 전용 줄은 자기 문단·저장 줄에서 번호를 한 번만 그린다.
//! 주석 갱신: 2026-11-15T00:07:54+09:00.
//! 보정 전 원본 50쪽 Native/fresh WASM 최저 98.58%, 대상 37쪽도 각각 98.58%다.
//! 2026-11-14 정본 PNG Print / compat2022 / 96 DPI / PDFium / raw 2px 측정.
//! 근거: mydocs/pr/assets/issues7727_7732_20261114/marker-partition/cause.json.
//! 독립 한컴 2020 Print 37쪽의 24)·25)는 각각 정책실현·재정조달 뒤의 별도 줄이다.
//! 검사에는 설치 글꼴이나 절대 px를 요구하지 않는다. 다른 문단 번호의 중첩과 누락을 검출한다.

use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::BoundingBox,
    DocumentCore,
};

fn collect(node: &LayerNode, markers: &mut Vec<(u16, BoundingBox)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, markers);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, markers);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::FootnoteMarker { bbox, marker } = op {
                    if matches!(marker.number, 24 | 25) {
                        markers.push((marker.number, *bbox));
                    }
                }
            }
        }
    }
}

#[test]
fn separate_note_only_lines_keep_distinct_source_paragraphs() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(source).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core
        .build_page_layer_tree_with_profile(36, RenderProfile::Print)
        .unwrap();
    let mut markers = Vec::new();
    collect(&tree.root, &mut markers);
    markers.sort_by_key(|(number, _)| *number);
    assert_eq!(
        markers
            .iter()
            .map(|(number, _)| *number)
            .collect::<Vec<_>>(),
        [24, 25],
        "각 번호는 공통 주석 paint로 한 번만 전달되어야 한다"
    );
    let first = markers[0].1;
    let second = markers[1].1;
    assert!(
        first.y + first.height < second.y,
        "서로 다른 문단의 별도 주석 줄이 겹치면 안 된다: {markers:?}"
    );
}
