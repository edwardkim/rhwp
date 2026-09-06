//! [Issue #6778] Square(어울림) 표 **옆 레인**이 가로만 적용되고 세로는 표 아래로
//! 밀려, 옆에 흘러야 할 4줄이 본문 하한·용지 밖으로 나갔다.
//!
//! `#4090` 이 세운 규칙 — 저장 host 줄높이가 표 높이의 1/4 미만이면 한/글은 표 옆으로
//! 글을 흘리므로, 흐름은 host 줄만 전진하고 표 높이는 **세로 배제 밴드**로 잡는다 —
//! 이 **조판(typeset)에만** 있고 렌더에는 짝이 없었다.
//!
//! `156757920` 1쪽 실측:
//!
//! ```text
//!   pi=12 빈 문단 + 표  wrap=어울림  227.5x212.9px   저장 host 줄 lh=1400HU=18.7px
//!   pi=13 저장 ls[0]    vpos=50106  cs=17626(235.0px)  sw=30562(407.5px)
//!
//!                     저장 사다리   페이지네이터   렌더(수정 전)
//!   pi=13 흐름 y          668.1        668.1          870.2     ← +202.1px
//!   pi=13 x(단 기준)      235.0          —            235.0     ✔ 가로는 맞다
//! ```
//!
//! `+202.1px` 은 표 높이 212.9px 에서 11px 못 미치는 값이다. 렌더가 이미 표 아래에
//! 와 있어 저장 vpos 로의 되돌림이 `lazy_base = -15159 HU` 나 되는 큰 역행이라
//! `VPOS_CORR_SKIP` 가드가 사다리 스냅을 기각한다 — 사다리가 있는데도 못 쓴다.
//!
//! ⭐ 수정은 항목 루프에서 `y_offset` 을 host 줄 전진으로 되돌리고, 표 바닥을 밴드로
//! 기억했다가 레인이 끝나는 문단에서 닫는다(조판의 `close_square_band` 와 같은 계약).
//!
//! ⚠⚠ **발동 조건은 저장 사다리가 다음 항목을 개체 오른쪽 레인에 두었는가 하나다.**
//! 레인 판정은 개체 상자와 직접 대조한다 — 두 조건을 **모두** 만족해야 한다.
//!
//! 1. 줄 시작이 **개체의 오른쪽 경계 밖**(`column_start >= object_right_hu`).
//!    개체와 겹치지 않으려면 레인은 그 밖에서 시작할 수밖에 없다. 폭만 우연히 맞는
//!    **큰 들여쓰기 문단은 여기서 갈린다**(156757920: `cs=17626` vs 우단 `17070`).
//! 2. 좁아진 폭이 **개체 폭의 절반 이상** — 개체가 실제로 그 줄을 밀어낸 증거.
//!
//! ⚠ `#4090`(156492236)은 개체가 오른쪽(`horz=문단(26319)`)이고 후속 문단이
//! `cs=0` 인 **왼쪽 레인**이라 술어를 통과하지 못한다(실측: 그 문서의 Square 표
//! 후보 전부 `next_is_lane=false`). 이 겹을 빼면 그 문서의 레인과 표 아래 꼬리가
//! 함께 위로 밀려 글자겹침이 **4 → 64건**이 된다.
//!
//! ⚠ **음성 대조는 공개 fixture 로 표현한다.** 초판은 술어 시험 5건을
//! `src/renderer/layout/tests.rs` 에 넣었는데, 기여 규칙이 `.rs` source 본문의 신규
//! 회귀 시험 증가를 허용하지 않는다(PR #6784 지적 — CI Lint 차단). `#4090`
//! (156492236 5쪽)이 마침 **폭 조건은 만족하지만 오른쪽 레인이 아닌** 형상
//! (`cs=0 sw=26319`, 단 폭 48188)이라 그 문서로 옮겼다
//! — `left_lane_flow_is_untouched`.
//!
//! ⚠ "폭 조건 만족 + 일반 들여쓰기" 형상은 `samples/` **990건을 전수로 훑어도 없다**
//! (가장 가까운 `hwpctl_API_v2.4.hwp` 는 `tac=true` 라 이 경로에 오지도 않는다).
//! 개체 우단 대조는 그 경우를 **기하로** 막는다 — `cs` 가 개체 우단 밖이어야 한다.
//!
//! ⚠ 초기 판에 있던 "렌더가 표 높이를 통째로 태웠는가"(≥50%) 겹은 **제거했다**.
//! 실측하니 두 문서 모두 비율이 1.0 을 넘어(156757920 `1.017`, 156492236
//! `0.998~1.645`) 아무것도 가르지 못했고, 이름·주석이 말하는 계약과도 달랐다.
//!
//! 결과(`layout-anomaly`): 넘침 6 → **0**, 용지밖 2 → **0**, 글자겹침 1 → **0**,
//! 쪽수 12 유지(한/글 2024 = 12쪽).

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6778/156757920-animal-welfare-husbandry-guidelines.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(path).expect("#6778 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

fn collect_text_bottoms(node: &RenderNode, out: &mut impl FnMut(f64)) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out(node.bbox.y + node.bbox.height);
    }
    for child in &node.children {
        collect_text_bottoms(child, out);
    }
}

/// 1쪽 — Square 표 옆 레인의 글이 본문 하한을 넘으면 안 된다.
///
/// 수정 전에는 4줄이 하한 아래로 나갔고 그중 2줄은 용지(1122.5px) 밖
/// `+10.2 / +40.0px` 였다.
#[test]
fn square_beside_lane_text_stays_inside_the_body() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 12, "한/글 2024 와 같은 12쪽이어야 한다");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let bottom = body.bbox.y + body.bbox.height;

    let mut worst = 0.0f64;
    collect_text_bottoms(body, &mut |b| worst = worst.max(b - bottom));

    assert!(
        worst <= 0.5,
        "1쪽 본문이 하한을 넘으면 안 된다 — #6778 회귀          \
         (초과 {worst:.1}px, 본문 하한 {bottom:.1}; 수정 전 +40.0px 로 용지 밖)"
    );
}

/// 옆 레인의 첫 줄이 표 **상단 근처**에서 시작해야 한다(표 아래가 아니라).
///
/// 저장 사다리 `pi=13 vpos=50106` = 단 기준 668.1px. 수정 전 렌더는 870.2px 였다.
#[test]
fn square_beside_lane_starts_next_to_the_table_not_below_it() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");

    // 그 쪽의 Square 표(폭이 단의 절반 미만인 유일한 표) 상자.
    let mut table_top = f64::MAX;
    let mut table_bottom = 0.0f64;
    let mut table_right = 0.0f64;
    fn walk(node: &RenderNode, body_w: f64, top: &mut f64, bottom: &mut f64, right: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) && node.bbox.width < body_w * 0.5
        {
            *top = top.min(node.bbox.y);
            *bottom = bottom.max(node.bbox.y + node.bbox.height);
            *right = right.max(node.bbox.x + node.bbox.width);
        }
        for child in &node.children {
            walk(child, body_w, top, bottom, right);
        }
    }
    walk(
        body,
        body.bbox.width,
        &mut table_top,
        &mut table_bottom,
        &mut table_right,
    );
    assert!(table_top < f64::MAX, "Square 표를 찾아야 한다");

    // 표의 세로 구간 안에서, 표 **오른쪽**에 그려진 글줄의 최상단.
    // 수정 전에는 그 창에 글이 하나도 없었다(전부 표 아래로 밀렸다).
    let mut lane_top = f64::MAX;
    fn lane_walk(node: &RenderNode, top: f64, bottom: f64, right: f64, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::TextRun(_))
            && node.bbox.x >= right - 1.0
            && node.bbox.y >= top - 1.0
            && node.bbox.y < bottom
        {
            *out = out.min(node.bbox.y);
        }
        for child in &node.children {
            lane_walk(child, top, bottom, right, out);
        }
    }
    lane_walk(body, table_top, table_bottom, table_right, &mut lane_top);

    assert!(
        lane_top < table_bottom - 100.0,
        "옆 레인의 글은 표 상단 쪽에서 시작해야 한다 — #6778 회귀                   (레인 최상단 {lane_top:.1}, 표 {table_top:.1}..{table_bottom:.1} 우단 {table_right:.1};           수정 전에는 그 창에 글이 하나도 없었다)"
    );
}

/// 밴드 **종료** 계약 — 레인을 벗어난 첫 줄은 표 바닥 아래에서 시작한다.
///
/// `pi=15` 는 좁은 레인 줄 3개(`x=310.6 w=407.5`) 뒤 `line=3` 에서 전폭
/// (`x=75.6 w=642.5`)으로 돌아온다. 그 전폭 줄이 표의 세로 구간 안에서 시작하면
/// 표와 글이 겹친다 — 되감기만 하고 밴드를 닫지 않았을 때의 실패 형상이다.
///
/// 저장 vpos 가 없는 항목까지 함께 보호하려고, **줄 단위가 아니라 그 쪽 전체**에서
/// "표 상단보다 아래에서 시작하는 첫 비-레인 줄"을 찾아 판정한다.
#[test]
fn square_band_closes_below_the_table_bottom() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");

    let mut table_top = f64::MAX;
    let mut table_bottom = 0.0f64;
    let mut table_right = 0.0f64;
    fn walk(node: &RenderNode, body_w: f64, top: &mut f64, bottom: &mut f64, right: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) && node.bbox.width < body_w * 0.5
        {
            *top = top.min(node.bbox.y);
            *bottom = bottom.max(node.bbox.y + node.bbox.height);
            *right = right.max(node.bbox.x + node.bbox.width);
        }
        for child in &node.children {
            walk(child, body_w, top, bottom, right);
        }
    }
    walk(
        body,
        body.bbox.width,
        &mut table_top,
        &mut table_bottom,
        &mut table_right,
    );
    assert!(table_top < f64::MAX, "Square 표를 찾아야 한다");

    // 표 상단보다 **아래에서 시작하는** 줄만 본다. 표 상단에 놓이는 host 앵커 줄은
    // 밴드의 시작점 자체라 대상이 아니다.
    //
    // ⚠ 표 **안쪽** 칸 글줄은 대상이 아니다(칸 글줄도 좁은 x 를 갖는다) — Table
    // 자손은 통째로 건너뛴다.
    let mut lines: Vec<(f64, f64, f64)> = Vec::new();
    fn collect(node: &RenderNode, out: &mut Vec<(f64, f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            out.push((node.bbox.y, node.bbox.x, node.bbox.width));
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    collect(body, &mut lines);
    lines.retain(|(y, ..)| *y > table_top + 0.5);
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));

    let first_non_lane = lines
        .iter()
        .find(|(_, x, _)| *x < table_right - 1.0)
        .copied()
        .expect("레인을 벗어난 줄이 있어야 한다");

    assert!(
        first_non_lane.0 >= table_bottom - 0.5,
        "레인을 벗어난 첫 줄은 표 바닥 아래에서 시작해야 한다 — #6778 밴드 종료 계약 \
         (첫 비-레인 줄 y={:.1} x={:.1} w={:.1}, 표 바닥 {:.1} 우단 {:.1})",
        first_non_lane.0,
        first_non_lane.1,
        first_non_lane.2,
        table_bottom,
        table_right
    );
}

/// ⚠ 음성 — **왼쪽 레인은 이 축이 건드리지 않는다** (`#4090` 156492236 5쪽).
///
/// 이 문서의 형상이 정확히 "폭 조건은 만족하지만 오른쪽 레인이 아닌" 경우다.
///
/// ```text
///   pi=44 Square 표   y=138.2..314.7  x=426.5 w=280.3     ← 개체가 오른쪽
///   pi=45 line0..5    y=154.2..324.3  x= 75.6 w=350.9     ← 글은 왼쪽으로 흐른다
///         저장 사다리  cs=0  sw=26319 (단 48188)          ← 폭 조건 통과, cs=0 이라 제외
///   pi=45 line6       y=369.7         x= 75.6 w=642.5     ← 표 바닥 아래에서 전폭 복귀
/// ```
///
/// 술어가 `column_start > 0` 만 보던 초판에서 이 문단을 레인으로 오인하면 흐름이
/// 되감겨 레인과 표 아래 꼬리가 함께 위로 밀리고 글자겹침이 **4 → 64건**이 된다.
/// 개체 우단 대조(`column_start >= object_right_hu`)가 `cs=0` 을 걸러 낸다.
#[test]
fn left_lane_flow_is_untouched() {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue4090/156492236_규제샌드박스_min.hwpx"),
    )
    .expect("#4090 공개 fixture 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(4).expect("5쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");

    // 오른쪽에 놓인 Square 표.
    let mut table: Option<(f64, f64, f64)> = None; // (top, bottom, left)
    fn walk_table(node: &RenderNode, body_w: f64, out: &mut Option<(f64, f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) && node.bbox.width < body_w * 0.5
        {
            let v = (node.bbox.y, node.bbox.y + node.bbox.height, node.bbox.x);
            if out.is_none_or(|cur| v.0 < cur.0) {
                *out = Some(v);
            }
        }
        for child in &node.children {
            walk_table(child, body_w, out);
        }
    }
    walk_table(body, body.bbox.width, &mut table);
    let (top, bottom, left) = table.expect("5쪽 Square 표를 찾아야 한다");
    assert!(
        left > body.bbox.x + body.bbox.width * 0.4,
        "이 문서의 개체는 오른쪽에 있다 (표 x={left:.1}, 본문 {:.1}..)",
        body.bbox.x
    );

    // 표의 세로 구간 안에서, **왼쪽**(단 시작)에서 시작하는 좁은 글줄이 있어야 한다.
    let mut beside_left = 0usize;
    fn walk_lines(node: &RenderNode, top: f64, bottom: f64, x0: f64, w: f64, out: &mut usize) {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && node.bbox.y >= top - 1.0
            && node.bbox.y < bottom
            && (node.bbox.x - x0).abs() <= 1.0
            && node.bbox.width < w * 0.75
        {
            *out += 1;
        }
        for child in &node.children {
            walk_lines(child, top, bottom, x0, w, out);
        }
    }
    walk_lines(
        body,
        top,
        bottom,
        body.bbox.x,
        body.bbox.width,
        &mut beside_left,
    );

    assert!(
        beside_left >= 5,
        "왼쪽 레인 글줄이 제자리에 있어야 한다 — #6778 이 이 문서를 건드리면 안 된다 \
         (표 {top:.1}..{bottom:.1} x={left:.1}, 왼쪽 좁은 줄 {beside_left}개; 기대 6개)"
    );
}
