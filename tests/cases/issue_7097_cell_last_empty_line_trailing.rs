//! [#7097] 칸의 **마지막 줄이 빈 줄**이면 그 줄의 trailing 줄간격을 칸 높이에 넣지 않는다.
//!
//! `samples/issue2470/36382471_masked.hwpx` 1쪽 바깥 표 2행이 8.05px 과대했다. 저장 사다리와
//! 표 선언이 한 점으로 모이는데 rhwp 만 어긋났다.
//!
//! ```text
//! 표 선언 총높이  68562HU = 914.16px
//! 2행 = 저장 사다리 끝(25372HU = 338.29) + 칸 여백 282HU(3.76) = 342.05px
//!   한/글  342.05  → 행 합이 표 선언과 일치
//!   rhwp   350.10  → +8.05  (마지막 빈 줄 lh=1000 sp=600 의 trailing 이 들어갔다)
//! ```
//!
//! `#5923` 이 비-TAC 표에서 칸 마지막 줄 trailing 을 제외했고, `#6681` 이 TAC 예외에서
//! 「개체만 있는 마지막 줄」을 뺐다 — 근거는 "그 줄 뒤에 붙일 줄이 없다" 였다. 글자가 아예
//! 없는 빈 마지막 줄도 같은 근거가 그대로 적용된다.
//!
//! 정본: `pdf/issue2470/36382471_masked-2022.pdf`(Hwp 2022 12.0.0.4547) 낱말 짝짓기로
//! 3행 `중랑물재생센터` Δy −8.04, 2행 안쪽(`vertAlign=CENTER`) `2026. 6.` Δy −3.97 이었다.
//!
//! 두 번째 문서로도 확인했다 — `148759031`(인천세관 수출입동향) 3쪽 표 괘선이 한컴 engine
//! 2020 정본과 수정 전 +4px, 수정 후 **0** 이다(job `cca861d2-407d-470f-8ca6-1cffcc3d2965`).
//!
//! 반례: 마지막 문단에 글자가 있는 TAC 다문단 칸은 종전 회계 그대로다(`Task #874/#1086`
//! 보존 핀, `#6660` 그림 위치 정본). 예외를 통째로 걷어내면 그 핀들이 깨진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue2470/36382471_masked.hwpx";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 1쪽 바깥 표(폭 653.9px)의 칸들.
fn wide_cells(core: &DocumentCore) -> Vec<BoundingBox> {
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let mut cells: Vec<BoundingBox> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(_) if n.bbox.width > 640.0 => Some(n.bbox),
            _ => None,
        })
        .collect();
    cells.sort_by(|a, b| a.y.total_cmp(&b.y));
    cells
}

#[test]
fn issue_7097_row_height_drops_the_trailing_spacing_of_an_empty_last_line() {
    let cells = wide_cells(&load());
    // 2행 칸: 한/글 342.05 (저장 사다리 끝 338.29 + 칸 여백 3.76). 결함 시 350.10.
    let row2 = cells
        .iter()
        .find(|b| (b.y - 540.1).abs() < 1.0)
        .expect("2행 칸(y≈540.1)");
    assert!(
        (row2.height - 342.05).abs() < 1.0,
        "#7097: 2행 칸 높이는 342.05 여야 한다 — 결함 시 350.10: {:.2}",
        row2.height
    );
}

#[test]
fn issue_7097_following_row_moves_up_by_the_same_amount() {
    let cells = wide_cells(&load());
    // 3행은 통째로 8.05px 위로 — 한/글 882.1, 결함 시 890.1.
    let row3 = cells.iter().find(|b| b.y > 700.0).expect("3행 칸");
    assert!(
        (row3.y - 882.1).abs() < 1.0,
        "#7097: 3행 칸은 882.1 에서 시작해야 한다 — 결함 시 890.1: {:.2}",
        row3.y
    );
    // 위쪽 흐름(1행)은 움직이지 않는다 — 이 축은 마지막 빈 줄에만 걸린다.
    let row1 = cells.first().expect("1행 칸");
    assert!(
        (row1.y - 117.1).abs() < 1.0,
        "#7097 통제군: 1행 칸(117.1)은 불변이어야 한다: {:.2}",
        row1.y
    );
}

#[test]
fn issue_7097_page_count_unchanged() {
    assert_eq!(load().page_count(), 2, "#7097: 쪽수는 2 여야 한다");
}
