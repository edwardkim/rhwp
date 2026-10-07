#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7344] 결재문서 `본문` 누름틀의 시작 마커가 글자처럼 취급하는 표 문단의 첫머리에
//! 놓이면, 그 문단은 저장 줄 계약(`composer::stored_tac_lines`)을 잃고 표를 저장 줄 원점이
//! 아닌 **앞 표 바로 아래**에 쌓는다.
//!
//! `36494836` 1쪽 `pi=4` 는 `[누름틀 시작, 1×1 표, 3×3 표]` 세 컨트롤과 저장 줄 둘을 가진다.
//!
//! ```text
//!   ls[0] vpos=19359 lh=1562 ls=660   ← 1×1 표(1282 + 바깥여백 140·140)
//!   ls[1] vpos=21581 lh=8733          ← 3×3 표(8453 + 바깥여백 140·140)
//! ```
//!
//! 두 표의 상단 간격은 저장 줄 원점 차 `2222HU = 29.6px` 이다. 수정 전 rhwp 는 1×1 표의
//! 높이(`17.1px`)만큼만 내려 3×3 표와 그 뒤 본문 전체를 12.5px 위로 올렸다. 한/글 2024 PDF
//! (MCP 변환)에서 3×3 표 첫 줄은 devel 보다 9.32pt 아래이고 이 수정 뒤 +0.08pt 차이다.
//!
//! 누름틀 시작 마커는 원시 스트림의 8유닛 칸만 차지하고 줄 상자를 만들지 않으므로, 줄
//! 소유 판정에서 구역·단 정의, 머리말/꼬리말과 같이 건너뛴다. 같은 문단에서 닫히는 필드는
//! `empty_control_stream_position` 가 계속 거른다(`field_ranges` 비어 있음 조건).

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7344/36494836_field_marker_tac_lines.hwpx";
const HU_PER_PX: f64 = 75.0;

fn load() -> DocumentCore {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&p).expect("표본 읽기")).expect("문서 로드")
}

/// 본문 최상위 표의 `(문단, 컨트롤) → 상단 y`.
fn table_tops(core: &DocumentCore, page: u32) -> Vec<(usize, usize, f64)> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    fn walk(n: &RenderNode, out: &mut Vec<(usize, usize, f64)>) {
        if let RenderNodeType::Table(t) = &n.node_type {
            if t.cell_context.is_none() {
                if let (Some(pi), Some(ci)) = (t.para_index, t.control_index) {
                    out.push((pi, ci, n.bbox.y));
                }
            }
            return;
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(&tree.root, &mut out);
    out
}

fn top_of(tops: &[(usize, usize, f64)], pi: usize, ci: usize) -> f64 {
    tops.iter()
        .find(|t| t.0 == pi && t.1 == ci)
        .unwrap_or_else(|| panic!("pi={pi} ci={ci} 표가 1쪽에 없다: {tops:?}"))
        .2
}

#[test]
fn field_begin_marker_keeps_the_stored_tac_line_origins() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 2024 출력과 같은 1쪽");

    let para = &core.document().sections[0].paragraphs[4];
    assert!(
        matches!(
            para.controls.first(),
            Some(rhwp::model::control::Control::Field(_))
        ),
        "표본 전제: pi=4 첫 컨트롤이 누름틀 시작이다"
    );
    let segs = &para.line_segs;
    assert_eq!(segs.len(), 2, "표본 전제: 표마다 저장 줄 하나");
    let stored_gap = f64::from(segs[1].vertical_pos - segs[0].vertical_pos) / HU_PER_PX;

    let tops = table_tops(&core, 0);
    let small = top_of(&tops, 4, 1);
    let large = top_of(&tops, 4, 2);
    assert!(
        (large - small - stored_gap).abs() <= 0.5,
        "두 표의 상단 간격 {:.2}px 가 저장 줄 원점 차 {stored_gap:.2}px 와 다르다",
        large - small
    );

    // 다음 문단(pi=5)의 표도 저장 사다리의 간격을 유지해야 한다 — 표 문단의 전진량이
    // 마지막 줄의 저장 원점에서 이어진다는 뜻이다.
    let next = &core.document().sections[0].paragraphs[5];
    let stored_next = f64::from(next.line_segs[0].vertical_pos - segs[1].vertical_pos) / HU_PER_PX;
    let next_top = top_of(&tops, 5, 0);
    assert!(
        (next_top - large - stored_next).abs() <= 0.5,
        "pi=5 표 상단 간격 {:.2}px 가 저장 사다리 {stored_next:.2}px 와 다르다",
        next_top - large
    );
}
