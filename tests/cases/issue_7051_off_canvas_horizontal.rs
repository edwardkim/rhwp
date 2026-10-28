//! [#7051] 가로 off-canvas 진단 계약. 부모 줄 상자는 용지 안에 있어도
//! child 글자 범위가 용지 밖이면 검출해야 한다.
//!
//! 원본 #6778의 배치가 수정되어 실제 넘침은 없어졌다. 정상 원본은 무넘침
//! 대조군으로 유지하고, 공개 scan_page 입력인 render tree의 글자 상자 하나만
//! 의도적으로 옮겨 양성 경계를 만든다. 실제 문서의 결함을 기대값으로 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::diagnostics::layout_anomaly::{scan_document, scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;
const SAMPLE: &str = "samples/issue6778/156757920-animal-welfare-husbandry-guidelines.hwp";
fn move_body_run_outside_paper(
    node: &mut RenderNode,
    paper_right: f64,
    tolerance: f64,
) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && node.bbox.x + node.bbox.width <= paper_right
    {
        for child in &mut node.children {
            if matches!(&child.node_type,RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
                && child.bbox.width > tolerance
            {
                child.bbox.x = paper_right + child.bbox.width.max(2.0 * tolerance);
                return Some(child.bbox.x - paper_right);
            }
        }
    }
    node.children
        .iter_mut()
        .find_map(|child| move_body_run_outside_paper(child, paper_right, tolerance))
}
/// 정상 원본은 용지 안에 있으며, 동일 줄의 의도적 child 이탈은 검출된다.
#[test]
fn off_canvas_sees_horizontal_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let opts = AnomalyOptions::default();
    let mut tree = core
        .build_page_render_tree(0)
        .expect("원본의 실제 render tree");
    let before = scan_page(0, &tree.root, core.page_count(), &opts);
    assert!(
        before
            .off_canvas
            .iter()
            .all(|a| a.over_right <= opts.overflow_tolerance_px),
        "수정된 원본을 진단기의 기존 양성 결함으로 고정하면 안 된다"
    );
    let paper_right = tree.root.bbox.x + tree.root.bbox.width;
    let injected_minimum =
        move_body_run_outside_paper(&mut tree.root, paper_right, opts.overflow_tolerance_px)
            .expect("용지 안 부모 줄의 가시 글자 런");
    let after = scan_page(0, &tree.root, core.page_count(), &opts);
    let detected = after
        .off_canvas
        .iter()
        .map(|a| a.over_right)
        .fold(0.0_f64, f64::max);
    assert!(
        detected >= injected_minimum,
        "child 글자 범위의 이탈을 용지 안 부모 줄 상자로 숨기면 안 된다"
    );
    assert!(
        after
            .off_canvas
            .iter()
            .all(|a| a.over_top <= opts.overflow_tolerance_px
                && a.over_bottom <= opts.overflow_tolerance_px),
        "가로 양성 경계가 세로 축을 바꾸면 안 된다"
    );
}
/// 세로 축은 종전대로다 — 가로를 켰다고 세로 판정이 달라지면 안 된다.
#[test]
fn vertical_axis_is_unchanged() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let anomalies = scan_document(&core, &AnomalyOptions::default()).expect("layout-anomaly");
    // 이 문서의 off-canvas 는 전부 가로다 — 세로 초과는 0 이다.
    let vertical: Vec<_> = anomalies
        .pages
        .iter()
        .flat_map(|page| page.off_canvas.iter())
        .filter(|a| a.over_top > 1.0 || a.over_bottom > 1.0)
        .collect();
    assert!(
        vertical.is_empty(),
        "이 문서에 세로 초과는 없어야 한다(가로 전용 결함). got {}건",
        vertical.len()
    );
}
