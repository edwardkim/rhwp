//! #8153: 원문 기호의 공급 프로그램 전진폭과 뒤 글자 소유를 보존한다.
//! 독립 Print 및 KoPub 6개 프로그램에서 각 기호의 전진은 900/1000em이다.
//! HWP 5.0 표 33의 글자 크기 1000HU·장평 99%를 원본에서 확인했다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::BoundingBox,
    DocumentCore,
};
fn find(node: &LayerNode, out: &mut Vec<(String, BoundingBox)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                find(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            find(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for o in ops {
                if let PaintOp::TextRun { bbox, run, .. } = o {
                    if run.text == "  ①, ②, ③, ④, " || run.text == "  ㉮, ㉯, ㉰, ㉱, …"
                    {
                        assert_eq!(run.char_shape_id, Some(89));
                        out.push((run.text.clone(), *bbox));
                    }
                }
            }
        }
    }
}
fn verify(ext: &str) {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut d = DocumentCore::from_bytes(
        &std::fs::read(base.join(format!("samples/2025 행정업무운영 편람(최종).{ext}"))).unwrap(),
    )
    .unwrap();
    d.set_hangul2024_compat(true);
    let tree = d
        .build_page_layer_tree_with_profile(58, RenderProfile::Print)
        .unwrap();
    let mut got = Vec::new();
    find(&tree.root, &mut got);
    assert_eq!(got.len(), 2, "두 원문 줄의 누락·중복 방지");
    // 독립 Print의 기호 폭 900/1000em, 원본 1000HU·장평 99%로 888HU.
    // 원문 useFontSpace=0은 확인된 반각 496HU, 쉼표는 독립 글꼴 메트릭 308HU.
    let expected_hu = 6 * 496 + 4 * 888 + 4 * 308;
    for (text, bbox) in got {
        let expected = expected_hu + if text.ends_with('…') { 888 } else { 0 };
        assert!(
            (bbox.width - f64::from(expected) / 75.0).abs() < 1e-7,
            "원본 크기·장평을 적용한 숫자/한글 원 기호의 전진: {text} width={}",
            bbox.width
        );
    }
}
#[test]
fn hwp_circled_symbol_keeps_source_advance() {
    verify("hwp")
}
#[test]
fn hwpx_circled_symbol_keeps_source_advance() {
    verify("hwpx")
}
