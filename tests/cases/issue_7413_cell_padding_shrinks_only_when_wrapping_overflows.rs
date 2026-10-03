//! [#7413] 칸 안 여백 축소는 **줄바꿈으로 해결되지 않을 때만** 발화한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `shrunk_cell_horizontal_padding` 은 칸 문단의 **자연 폭**(줄바꿈하지 않은 한 줄 폭)이
//! 가용 너비의 1.15배를 넘으면 선언된 칸 안 여백을 양쪽 1.0px 까지 깎았다. 그 조건은
//! "이 문단은 줄바꿈이 필요하다"는 뜻일 뿐이라, **줄바꿈하면 칸에 들어가는 문단까지 전부**
//! 걸렸다.
//!
//! 축소가 실제로 지탱하는 계약은 "칸 안 글자가 칸 밖으로 새지 않는다" 하나다. 축소를
//! 통째로 끄고 `samples/**` 전수를 재면 겹침이 4건 늘고, 그 넷이 전부 **줄바꿈 결과가
//! 행 높이를 넘는** 자리였다.
//!
//! ```text
//! 20099369_yeongwol_forms 3쪽   칸 w=41.9 h=43.6 가 2줄 45px 를 요구
//! issue5169 12쪽 Cell5          칸 h=59.2 가 4줄 83.2px (바닥 21.1px 초과)
//! issue5169 12쪽 Cell51         칸 h=29.4 가 2줄 46.7px (17.3px 초과)
//! ```
//!
//! 그래서 묻는 것을 바꿨다 — **줄바꿈한 결과가 칸 높이를 넘는가.** 넘지 않으면 줄바꿈이
//! 답이고 선언 여백을 그대로 둔다.
//!
//! # 이 검사가 잠그는 것
//!
//! 양방향을 함께 잠근다. 한쪽만 잠그면 규칙이 한 방향으로 무너져도 통과한다.
//!
//! 1. **깎지 말아야 할 칸** — 줄바꿈이 칸 높이에 들어가면 선언 여백이 남는다.
//! 2. **깎아야 할 칸** — 줄바꿈이 칸 높이를 넘으면 종전대로 깎아 글자가 칸 밖으로
//!    새지 않는다.
//!
//! # 기대값의 출처
//!
//! 두 입력 모두 대응하는 한/글 출력 PDF 가 없다. 그래서 기대값을 **정본이 아니라 기하
//! 계약**에서 가져온다 — 문서가 선언한 안 여백과, 글자가 칸 안에 머무는가. 정본 대조가
//! 가능한 문서들(`80168`·`2022 국립국어원`)에서 이 변경이 한/글에 가까워진다는 증거는
//! 이슈 `#7413` 의 Visual Sweep 기록에 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 줄바꿈이 칸 높이에 들어가는 칸 — 선언 여백이 남아야 한다.
const FITS: &str = "samples/hwpx/table-text.hwpx";
/// 줄바꿈이 칸 높이를 넘는 칸 — 종전대로 깎여 글자가 칸 밖으로 안 나가야 한다.
const OVERFLOWS: &str = "samples/issue5699/20099369_yeongwol_forms.hwp";

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// `(칸 bbox, 글자 왼끝, 글자 오른끝)` 를 칸 단위로 모은다.
fn cell_text_extents(node: &RenderNode, cell: Option<&RenderNode>, out: &mut Vec<(f64, f64, f64, f64)>) {
    let next_cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (RenderNodeType::TextRun(_), Some(c)) = (&node.node_type, next_cell) {
        out.push((c.bbox.x, c.bbox.width, node.bbox.x, node.bbox.x + node.bbox.width));
    }
    for child in &node.children {
        cell_text_extents(child, next_cell, out);
    }
}

/// `(칸 x, 칸 y, 칸 폭, 그 칸의 TextLine 수)`.
fn cell_line_counts(
    node: &RenderNode,
    cell: Option<&RenderNode>,
    out: &mut Vec<(f64, f64, f64, usize)>,
) {
    if let RenderNodeType::TableCell(_) = node.node_type {
        let mut count = 0usize;
        fn walk(n: &RenderNode, count: &mut usize) {
            if matches!(n.node_type, RenderNodeType::TextLine(_)) {
                *count += 1;
            }
            for c in &n.children {
                walk(c, count);
            }
        }
        walk(node, &mut count);
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, count));
    }
    for child in &node.children {
        cell_line_counts(child, cell, out);
    }
}

/// 줄바꿈으로 해결되는 칸은 선언 안 여백을 지킨다.
#[test]
fn a_cell_that_wraps_within_its_height_keeps_its_declared_padding() {
    let core = load(FITS);
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut rows = Vec::new();
    cell_text_extents(&page.root, None, &mut rows);

    // 대상 칸: 숫자 한 줄이 들어가는 폭 76.4px 칸.
    let target: Vec<_> = rows
        .iter()
        .filter(|(cx, cw, ..)| (*cx - 115.30).abs() < 0.2 && (*cw - 76.40).abs() < 0.2)
        .collect();
    assert!(
        !target.is_empty(),
        "정답지 전제가 깨졌다 — 대상 칸(x=115.30, w=76.40)을 못 찾았다. 쪽 구성이 바뀌었으면          이 검사의 좌표부터 다시 정해야 한다."
    );

    let left = target.iter().map(|(_, _, x0, _)| *x0).fold(f64::MAX, f64::min);
    let right = target.iter().map(|(_, _, _, x1)| *x1).fold(f64::MIN, f64::max);
    let (cx, cw) = (target[0].0, target[0].1);
    let pad_left = left - cx;
    let pad_right = cx + cw - right;

    assert!(
        pad_left > 3.0 && pad_right > 3.0,
        "줄바꿈이 칸 높이에 들어가는데도 안 여백이 깎였다 — 축소 하한(1.0px)까지 내려가면          여기서 1.0 근처가 나온다. 좌 {pad_left:.2}px / 우 {pad_right:.2}px"
    );
}

/// 줄바꿈이 칸 높이를 넘는 칸은 종전대로 깎여 **한 줄**로 담긴다.
///
/// 이 칸(y=865.4, h=43.6, w=41.9)은 줄바꿈하면 22.5px 두 줄 = 45px 로 행 높이를 넘는다.
/// 축소를 끄면 둘째 줄이 표 바닥을 9.4px 지나 아래 본문 줄과 겹친다(이슈 `#7413` 기록).
#[test]
fn b_cell_that_cannot_wrap_within_its_height_still_shrinks() {
    let core = load(OVERFLOWS);
    let page = core.build_page_render_tree(2).expect("3쪽 render tree");
    let mut lines = Vec::new();
    cell_line_counts(&page.root, None, &mut lines);

    let target: Vec<_> = lines
        .iter()
        .filter(|(cx, cy, cw, _)| {
            (*cw - 41.9).abs() < 0.3 && (*cy - 865.4).abs() < 1.0 && *cx > 0.0
        })
        .collect();
    assert_eq!(
        target.len(),
        1,
        "정답지 전제가 깨졌다 — 대상 칸(y=865.4, w=41.9)을 하나로 못 집었다. 쪽 구성이          바뀌었으면 이 검사의 좌표부터 다시 정해야 한다. 후보={target:?}"
    );
    let (_, _, _, line_count) = target[0];
    assert_eq!(
        *line_count, 1,
        "줄바꿈이 칸 높이(43.6px)를 넘는 칸이 두 줄로 나뉘었다 — 여백을 깎아 한 줄로 담아야          둘째 줄이 표 바닥을 지나 아래 본문과 겹치지 않는다."
    );
}
