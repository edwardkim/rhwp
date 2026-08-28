//! [Issue #6280] 텍스트 없는 셀의 저장 줄 높이가 이미 품은 그림을 다시 더해 표가
//! 1.84배로 부풀고, 가운데 정렬된 제목이 장식 막대를 덮는다 (156742029 21쪽).
//!
//! 21쪽에는 **같은 서식의 표가 둘** 있고, 그중 하나만 깨진다 — 그래서 이 문서 자체가
//! 통제군을 갖는다.
//!
//! ```text
//! pi=474 `타기관 파견 등`  셀[0] ls[0] lh=1400(18.7px)  그림 흐름 30.2px
//!                          content 48.9 = 18.7 + 30.2          (줄이 그림을 못 담음)
//! pi=480 `의원면직`        셀[0] ls[0] lh=3000(40.0px)  그림 흐름 36.7px
//!                          content 76.7 = 40.0 + 36.7          (줄이 이미 그림을 담음)
//! ```
//!
//! 두 셀 모두 `text_len=0 ctrls=1` 이고 그림은 `tac=false wrap=TopAndBottom` 이다.
//! `pi=480` 은 저장 줄 높이(40.0px)가 그림 흐름(36.7px)을 **이미 덮는데** 그림을 또
//! 더해 선언 43.8px 의 1.84배(80.5px)로 부푼다. 칸이 `valign=Center` 라 제목이
//! 내려와 장식 막대(`Rect` y=645.4)를 관통한다.
//!
//! 잠금은 좌표 상수 대신 **불변식 + 통제군**을 건다 — 부푼 표는 선언 높이로 돌아오고,
//! 같은 쪽 통제군 표는 값이 바뀌지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6280/156742029_prosecutor_transfer_list.hwp";
const PAGE: u32 = 20;
/// 두 표의 선언 칸 높이 `h=3287HU` = 43.8px.
const DECLARED_PX: f64 = 43.8;

#[test]
fn issue_6280_stored_line_height_absorbs_its_object() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 21 render tree");

    let mut heights = Vec::new();
    collect_icon_cell_heights(&page.root, &mut heights);
    assert!(
        heights.len() >= 2,
        "21쪽에 아이콘 칸을 가진 표가 둘 이상 있어야 한다: {}",
        heights.len()
    );

    // 어느 표도 선언 높이보다 크게 부풀지 않는다 — 종전 `의원면직` 표는 80.5px 였다.
    for h in &heights {
        assert!(
            *h <= DECLARED_PX + 1.0,
            "표가 선언 높이({DECLARED_PX:.1}px)보다 부풀었다: {h:.1}px (전 80.5)"
        );
    }
}

/// 아이콘 칸의 **세로 테두리** 높이를 모은다 — 그 길이가 곧 칸 높이다.
///
/// 실측(수정 전 `[43.8, 43.8, 80.5, 80.5]` / 수정 후 `[43.8, 43.8, 43.8, 43.8]`) —
/// 표마다 좌·우 두 개씩이라 넷이 나온다.
fn collect_icon_cell_heights(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Line(_))
        && (node.bbox.width - 1.5).abs() < 0.6
        && node.bbox.height > 20.0
    {
        out.push(node.bbox.height);
    }
    for child in &node.children {
        collect_icon_cell_heights(child, out);
    }
}
