//! 한/글 문서(호환 대상이 한/글)에서 저장 줄 없는 빈 문단에 붙은 문단 기준 자리차지 직선은 그
//! 앵커 문단의 줄을 그대로 두고, 띠에 걸리는 뒤 줄만 띠 아래로 민다. 띠가 앵커 줄 상자 안이면
//! 앵커 줄이 띠를 흡수해 밀림이 없다.
//!
//! 재현물 `samples/issue8582/hwp_band.hwpx` (`samples/test-image.hwpx` 조합, 11pt, 줄 피치 1100 HU):
//! `BAND<o>` 글 문단, 자리차지 직선(높이 120 HU, 문단 기준 세로 오프셋 o = 2374·600·0 HU)만 가진
//! 빈 문단, 빈 문단 3개, `AFTER<o>` 글 문단. 실문서(HWP5, 저장 줄 없음)와 같은 경로를 지나도록
//! 시험에서 저장 줄을 지우고 HWP5 로 다시 저장해 잰다.
//!
//! 기준: 웹한글기안기 인쇄 `pdf/issue8582/hwp_band-webhwp-print.pdf`. `BAND` → `AFTER` 거리(HU):
//! o=2374 5784, o=600 5496, o=0 5496. 밀림 없는 다섯 줄은 5500 HU 다.
//! - o=2374: 앵커 줄(0~1100) 다음 줄(1100~2200)은 띠 위, 그다음 줄(2200~3300)이 띠(2374~2494)에
//!   걸려 띠 아래(2494)로 밀린다(+294).
//! - o=600·0: 띠가 앵커 줄 상자 안이라 흡수되고 밀림이 없다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

fn hwp5_without_stored_lines(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("재현물");
    let mut doc = rhwp::parser::parse_document(&bytes).expect("HWPX 읽기");
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            para.line_segs.clear();
        }
    }
    rhwp::serializer::serialize_document(&doc).expect("HWP5 저장")
}

fn label_tops(node: &RenderNode, out: &mut Vec<(String, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.text.trim();
        if text.starts_with("BAND") || text.starts_with("AFTER") {
            out.push((text.to_string(), node.bbox.y));
        }
    }
    for child in &node.children {
        label_tops(child, out);
    }
}

#[test]
fn hwp_target_topbottom_line_keeps_anchor_line_and_pushes_only_overlapping_lines() {
    let bytes = hwp5_without_stored_lines("hwp_band");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tops = Vec::new();
    label_tops(&tree.root, &mut tops);
    let top = |label: &str| {
        tops.iter()
            .find(|(text, _)| text == label)
            .map(|(_, y)| *y)
            .unwrap_or_else(|| panic!("{label} 줄 {tops:?}"))
    };
    let spans: Vec<(i32, f64, f64)> = [(2374, 5784.0), (600, 5496.0), (0, 5496.0)]
        .into_iter()
        .map(|(offset, expected)| {
            let span = (top(&format!("AFTER{offset}")) - top(&format!("BAND{offset}"))) * HU_PER_PX;
            (offset, span, expected)
        })
        .collect();
    for (offset, span, expected) in &spans {
        assert!(
            (span - expected).abs() <= 60.0,
            "오프셋 {offset}: BAND→AFTER {span:.0} HU, 웹한글기안기 {expected:.0} HU ({spans:?})"
        );
    }
}
