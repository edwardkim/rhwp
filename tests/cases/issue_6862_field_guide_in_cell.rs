//! [#6862] 빈 누름틀 안내문이 **가운데 정렬 칸에서 칸 중앙부터** 시작하고, 칸 안에서
//! 접히지 않아 용지 밖까지 나간다.
//!
//! `2249811` 1쪽은 안내문이 전부 표 칸 안이다. 두 갈래가 겹쳐 있다.
//!
//! **① 정렬 앵커를 시작점으로 쓴다.** 빈 누름틀 줄은 글자 폭이 0 이라
//! `find_x_for_char` 가 돌려주는 것은 **정렬 앵커**(가운데 정렬이면 줄 중앙)다.
//! 안내문을 거기서 오른쪽으로 그리면 통째로 자기 폭의 절반만큼 밀린다.
//!
//! ```text
//!   칸 286.9..670.1  중앙 478.5   안내문 폭 668.0
//!     종전 시작 478.5           = 중앙 (폭을 안 뺐다)
//!     정상 시작 478.5 − 334.0   = 144.5
//! ```
//!
//! **② 칸 안에서는 접지 않는다.** `#6111` 이 본문 안내문 접기를 넣으면서 셀은
//! "가용 폭 기준이 다르므로 종전대로 한 줄"로 남겼는데, 그 기준은 이미 손에 있다 —
//! **이 줄의 상자**(`line_node.bbox`)가 칸 안여백까지 반영한 텍스트 상자다.
//!
//! ```text
//!                 안내문 글리프 x     용지(793.7) 밖
//!   수정 전     178.8 .. 1139.8          41 글리프
//!   수정 후     178.8 ..  656.0           0
//!   안내문 줄 수   9 → 15 (칸 안에서 접힌다)
//! ```
//!
//! ⚠ 이 축은 `layout-anomaly` 가 보지 못한다 — 안내문 노드는 `with_editor_only` 라
//! 기하 이상탐지 대상이 아니다. 용지를 352.8px 넘겨도 다섯 지표가 전부 침묵한다.
//!
//! ⚠ 인쇄·PDF 에는 원래 안 나온다. engine 2020 정본에 빨간 글씨가 하나도 없고 rhwp 도
//! `RenderProfile::shows_editor_visuals()` 가 `Screen`·`FastPreview` 에서만 참이다 —
//! 이것은 **편집 화면 정합** 축이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6862/2249811-vessel-collision-analysis-form.hwp";
/// 이 문서가 조판하는 A4 세로 종이 폭(px, 96dpi).
const PAPER_WIDTH_PX: f64 = 793.7;

fn open_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

/// 안내문 노드 — 빨간 기울임 편집 전용 `TextRun`.
///
/// `(x, x + width, y)` 로 모은다.
fn guide_runs(node: &RenderNode, out: &mut Vec<(f64, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.style.italic && run.style.color == 0x0000FF && !run.text.trim().is_empty() {
            out.push((node.bbox.x, node.bbox.x + node.bbox.width, node.bbox.y));
        }
    }
    for child in &node.children {
        guide_runs(child, out);
    }
}

fn page_guides() -> Vec<(f64, f64, f64)> {
    let core = open_sample();
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut out = Vec::new();
    guide_runs(&tree.root, &mut out);
    out
}

/// 안내문을 품은 칸들의 오른쪽 끝 — 안내문은 그 안에 있어야 한다.
fn cell_right_edges(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if let RenderNodeType::TableCell(_) = &node.node_type {
        out.push((
            node.bbox.x,
            node.bbox.x + node.bbox.width,
            node.bbox.y,
            node.bbox.y + node.bbox.height,
        ));
    }
    for child in &node.children {
        cell_right_edges(child, out);
    }
}

#[test]
fn issue_6862_field_guides_stay_inside_the_paper() {
    let guides = page_guides();
    assert!(
        guides.len() >= 8,
        "1쪽에는 빈 누름틀 안내문이 여럿 있어야 한다 (실측 {})",
        guides.len()
    );
    let worst = guides
        .iter()
        .map(|(_, right, _)| *right)
        .fold(f64::MIN, f64::max);
    assert!(
        worst <= PAPER_WIDTH_PX,
        "안내문이 용지({PAPER_WIDTH_PX}) 밖으로 나가면 안 된다 — 결함 시 1146.5 까지 \
         나갔다. 실측 최대 우단 {worst:.1}"
    );
}

#[test]
fn issue_6862_field_guides_stay_inside_their_cell() {
    let core = open_sample();
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut cells = Vec::new();
    cell_right_edges(&tree.root, &mut cells);
    assert!(!cells.is_empty(), "1쪽은 표 문서다");

    let mut checked = 0usize;
    for (left, right, top) in page_guides() {
        // 이 안내문을 품는 가장 좁은 칸을 찾는다.
        let owner = cells
            .iter()
            // 안내문의 **시작점**을 품는 칸이 그 안내문의 주인이다.
            .filter(|(cx, cright, cy, cbottom)| {
                left + 0.5 >= *cx && left < *cright && top + 0.5 >= *cy && top < *cbottom
            })
            .min_by(|a, b| {
                (a.1 - a.0)
                    .partial_cmp(&(b.1 - b.0))
                    .expect("유한한 칸 폭")
            });
        let Some((cell_left, cell_right, _, _)) = owner else {
            continue;
        };
        checked += 1;
        assert!(
            right <= cell_right + 0.5,
            "안내문이 자기 칸({cell_left:.1}..{cell_right:.1}) 밖으로 나가면 안 된다 \
             — 실측 {left:.1}..{right:.1}"
        );
    }
    assert!(
        checked >= 6,
        "칸 안 안내문을 6개 이상 확인해야 한다 (확인 {checked})"
    );
}
