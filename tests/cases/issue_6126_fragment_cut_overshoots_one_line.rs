//! [Issue #6126] 쪽 조각 셀의 줄-단위 컷이 상자에 들어가지 않는 마지막 한 줄을
//! 얹어 괘선에 잘린다 (3171199 별표 1, 3쪽 "- 1주 이상 2주미만 …").
//!
//! 근인: 셀 안 **중첩 표 host 문단**의 줄 간격(`line_spacing`)이 컷 회계에서
//! 빠진다. 렌더러는 중첩 표를 지나 흐름을 `lh + ls` 만큼 전진시키는데 유닛
//! 높이는 `lh` 에서 멈춰, 중첩 표 하나당 9.6px 씩 컷이 짧게 잡혔다. 이 조각은
//! 중첩 표가 둘이라 19.2px 짧았고, 그만큼 한 줄을 더 얹어 마지막 줄이 조각
//! 상자 밖(칸 하단 +7.6px)에 그려졌다.
//!
//! 이 계상은 #5880 이 HWPX 저장 사다리에 이미 도입했다. 그 갈래가 요구하는
//! 사다리 전제(쪽 스케일 리셋·선형-정확)는 HWPX 저장 형상 전용이라, native
//! HWP5 는 **문단 델타 등식**(다음 문단 저장 vpos 델타 == lh + ls) 증거만으로
//! 계상한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6126/3171199_design_capability_criteria.hwp";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 2;

#[test]
fn issue_6126_fragment_cut_keeps_every_line_inside_the_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 한글 2020 오라클도 7쪽 — 컷이 한 줄 줄어도 쪽 수는 그대로다.
    assert_eq!(core.page_count(), 7, "한글 오라클과 같은 7쪽이어야 한다");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 3 render tree");
    let mut violations: Vec<String> = Vec::new();
    collect_overflowing_lines(&page.root, None, &mut violations);
    assert!(
        violations.is_empty(),
        "조각 셀 밖으로 나간 줄이 있다 (괘선에 잘린다): {}",
        violations.join(", ")
    );
}

/// 칸 상자 아래로 삐져나온 텍스트 줄을 모은다.
fn collect_overflowing_lines(node: &RenderNode, cell: Option<&RenderNode>, out: &mut Vec<String>) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (Some(cell), RenderNodeType::TextRun(run)) = (cell, &node.node_type) {
        let cell_bottom = cell.bbox.y + cell.bbox.height;
        let line_bottom = node.bbox.y + node.bbox.height;
        // 0.5px 은 반올림 여유. 결함은 7.6px 이라 이 여유와 구별된다.
        if line_bottom > cell_bottom + 0.5 && !run.text.trim().is_empty() {
            out.push(format!(
                "{:?}(줄 아래끝 {:.1} > 칸 {:.1})",
                run.text.chars().take(12).collect::<String>(),
                line_bottom,
                cell_bottom
            ));
        }
    }
    for child in &node.children {
        collect_overflowing_lines(child, cell, out);
    }
}
