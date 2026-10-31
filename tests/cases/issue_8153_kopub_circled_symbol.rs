//! 관계형: 원문 기호의 공급 프로그램 전진폭과 뒤 글자 소유를 보존한다.
//! 주석 갱신: 2026-10-31T18:41:48+09:00; 직전 파일 커밋: 2026-10-30T19:50:11+09:00 (c417310ca).
//! 두 원문 줄은 run 경계와 무관하게 같은 줄에서 맞닿은 연속 run 의 글로 대조한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 59쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 59쪽 Native 100.00% / fresh WASM 100.00%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 및 KoPub 6개 프로그램에서 각 기호의 전진은 900/1000em이다.
//! HWP 5.0 표 33의 글자 크기 1000HU·장평 99%를 원본에서 확인했다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::render_tree::BoundingBox,
    DocumentCore,
};
fn find(node: &LayerNode, out: &mut Vec<(String, BoundingBox, Option<u32>)>) {
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
                    out.push((run.text.clone(), *bbox, run.char_shape_id));
