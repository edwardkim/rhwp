//! [Issue #6795] 쪽에 걸쳐 쪼개진 자리차지 표의 **형제 표**가 그 조각 위에 겹쳐 놓이던
//! 결함의 가드.
//!
//! ## 두 기계가 같은 착각을 한다 — 이어지는 쪽을 빈 쪽으로 본다
//!
//! 빈 host 문단에 매달린 자리차지(`wrap=위아래`, `vert=문단`) 표들은 세로로 쌓여야 한다.
//! 첫 표가 커서 쪽을 넘기면 그 조각은 `PageItem::PartialTable` 로 나가는데, **뒤 표를
//! 배치하는 두 경로가 모두 그 조각을 못 본다.**
//!
//! ```text
//!   ① try_typeset_empty_para_float_table
//!        raw_top 은 para_start_height 기준 → 이어지는 쪽에서는 0.0
//!        실측: cur_h=415.7 인데 raw_top=0.0, lane_bottom 696.0 ≤ available 710.6 → 배치
//!
//!   ② #2813 통짜-배치 구제 (typeset_block_table_inner)
//!        "저장 앵커 줄이 스택 아래이고 본문 안이면 한글이 통째로 이 쪽에 놨다"
//!        실측: bounds=(655.8, 669.8) — 그런데 이 좌표는 **문단이 시작한 쪽**의 것이다
//!        → 구제가 잘못 발동해 advance_column_or_new_page 를 막는다
//! ```
//!
//! ①만 고치면 ②가 받아 같은 자리에 놓고, ②만 고치면 ①이 먼저 놓는다. **둘 다** 고쳐야
//! 닫힌다(실측: `lane` 단독 겹침 1, `guard` 단독 겹침 1, 둘 다 0).
//!
//! ## 수정 역적용 실측 (red → green)
//!
//! `git apply -R` 로 `typeset.rs` hunk 57줄만 되돌리고 이 파일의 5개 시험을 돌린 결과다
//! (`samples/issue6795/…-cyber-university-application.hwp`, 쪽 인덱스는 0-based).
//!
//! ```text
//!   수정 전  idx 30  pi=113 ci=0  y=143.6..560.1
//!                    pi=113 ci=1  y=158.2..854.2  → 같은 쪽, 548.0 × 401.9px 겹침
//!            idx 31  pi=114 ci=0                  (형제 표가 없어 한 쪽씩 앞당겨진다)
//!            idx 32  pi=121 ci=0
//!   수정 후  idx 30  pi=113 ci=0 단독 (y=143.6..560.1)
//!            idx 31  pi=113 ci=1 단독 (y=143.6..839.6, 본문 143.6..854.2 안)
//!            idx 32  pi=114 ci=0 단독
//! ```
//!
//! 역적용 판에서 아래 다섯 중 **넷이 실패**하고, `#2813` 음성 통제군만 통과한다 —
//! 가드가 대상 형상만 잡고 통제군을 건드리지 않는다는 증거다.
//!
//! 한/글 **2020** 오라클(`lastSavedWith.product = null`, `version 6.7.6.1002` → 저장소
//! 정책 §3.5.1 의 2022 이하 버킷)도 같은 순서다 — 인쇄 쪽번호 `- 27 -` 조각,
//! `- 28 -` `현장실사 … 위원회 심의결과` 단독, `- 29 -` `XIV. 종합의견`. 총 45쪽.
//!
//! ## 지키는 계약
//!
//! 대상을 `pi`/`ci` 로 직접 집는다 — 표 개수나 인접 상자 관계만 보면 대상 표가 사라지거나
//! 다른 쪽으로 가도 통과한다.
//!
//! ⚠ 기각한 안 둘. `is_deferred_coanchored_rowbreak_table` 의 `vertical_offset > 0` 을
//! `>= 0` 으로 넓히면 겹침은 사라지지만 표가 `pi=114` 뒤로 가 **쪽 순서가 뒤집힌다**.
//! 문단 기준 높이(`para_start_height`)를 쪽 넘김 뒤 다시 잡는 안은 ②가 막아 효과가 0이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6795/1341000-201100013-cyber-university-application.hwp";

/// `#2813` 통짜-배치 구제가 **살아 있어야 하는** 통제군 — 이 fixture 의 쪽에는 자기 문단의
/// continuation 조각이 없으므로 새 가드가 발동하면 안 된다.
const CONTROL_SAMPLE: &str = "samples/issue2813/dangjik_dutylog.hwpx";

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// `Column` 의 **직계** 표만 `(pi, ci, y0, y1, x0, x1)` 로 모은다 — 칸 안의 중첩 표는
/// 세지 않는다.
type TableBox = (Option<usize>, Option<usize>, f64, f64, f64, f64);

fn column_tables(node: &RenderNode, in_column: bool, out: &mut Vec<TableBox>) {
    if in_column {
        if let RenderNodeType::Table(table) = &node.node_type {
            out.push((
                table.para_index,
                table.control_index,
                node.bbox.y,
                node.bbox.y + node.bbox.height,
                node.bbox.x,
                node.bbox.x + node.bbox.width,
            ));
            return;
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        column_tables(child, in_column, out);
    }
}

fn body_bounds(node: &RenderNode) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children.iter().find_map(body_bounds)
}

fn page_tables(document: &HwpDocument, page: u32) -> Vec<TableBox> {
    let tree = document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"));
    let mut out = Vec::new();
    column_tables(&tree.root, false, &mut out);
    out
}

/// 괘선 두께·반올림을 넘는 실질 겹침만 센다.
const TOLERANCE_PX: f64 = 8.0;

/// 조각이 차지한 쪽에는 형제 표가 함께 오지 않는다 — 겹침의 직접 계약.
#[test]
fn split_fragment_page_holds_only_the_fragment() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");

    let fragment_page = page_tables(&document, 30);
    assert_eq!(
        fragment_page.len(),
        1,
        "쪽 idx 30 에는 조각 하나만 있어야 한다 — 회귀 시 `pi=113 ci=1` 이 함께 와 \
         548.0 × 401.9px 겹친다. got {fragment_page:?}"
    );
    let (pi, ci, y0, y1, ..) = fragment_page[0];
    assert_eq!(
        (pi, ci),
        (Some(113), Some(0)),
        "쪽 idx 30 의 표는 27×10 표의 마지막 조각(pi=113 ci=0)이어야 한다"
    );
    assert!(
        (y0 - 143.6).abs() < TOLERANCE_PX && (y1 - 560.1).abs() < TOLERANCE_PX,
        "조각 상자가 실측(y=143.6..560.1)에서 벗어났다 — got {y0:.1}..{y1:.1}"
    );
}

/// 형제 표는 **자기 쪽 상단**에 단독으로, 본문 안에 놓인다.
#[test]
fn split_float_sibling_gets_its_own_page_inside_the_body() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");

    let tree = document
        .build_page_render_tree(31)
        .expect("쪽 idx 31 render tree");
    let (body_top, body_bottom) = body_bounds(&tree.root).expect("쪽 idx 31 Body");
    let mut tables = Vec::new();
    column_tables(&tree.root, false, &mut tables);

    assert_eq!(
        tables.len(),
        1,
        "쪽 idx 31 에는 형제 표 하나만 있어야 한다. got {tables:?}"
    );
    let (pi, ci, y0, y1, ..) = tables[0];
    assert_eq!(
        (pi, ci),
        (Some(113), Some(1)),
        "쪽 idx 31 의 표는 19×6 형제 표(pi=113 ci=1)여야 한다 — 회귀 시 이 표가 idx 30 조각 위에 \
         겹쳐 그려지고 32쪽에는 오지 않는다"
    );
    assert!(
        (y0 - body_top).abs() < TOLERANCE_PX,
        "형제 표는 본문 상단({body_top:.1})에서 시작해야 한다 — got {y0:.1}"
    );
    assert!(
        y1 <= body_bottom + TOLERANCE_PX,
        "형제 표 하단({y1:.1})이 본문 하한({body_bottom:.1})을 넘었다 — \
         겹침을 넘침으로 바꾸기만 한 판이다"
    );
}

/// 겹침만 없애고 표를 뒤로 미루는 안(`vertical_offset >= 0` 완화)을 막는다 — 문서 순서가
/// 한/글 2020 오라클과 같아야 한다.
#[test]
fn split_float_sibling_keeps_document_order() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");

    let next = page_tables(&document, 32);
    assert_eq!(
        next.len(),
        1,
        "쪽 idx 32 에는 다음 문단의 표 하나만 있어야 한다. got {next:?}"
    );
    assert_eq!(
        (next[0].0, next[0].1),
        (Some(114), Some(0)),
        "쪽 idx 32 의 표는 `XIV. 종합의견`(pi=114 ci=0)이어야 한다 — 기각한 `>= 0` 완화 판은 \
         여기에 pi=114 를 앞세우고 pi=113 ci=1 을 뒤로 보내 쪽 순서를 뒤집는다"
    );
}

/// 어느 쪽에서도 자리차지 표 두 장이 겹치지 않는다 — 형상이 바뀌어도 남는 상위 계약.
#[test]
fn no_page_stacks_two_float_tables_on_top_of_each_other() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    let last = document.page_count().saturating_sub(1).min(34);

    for page in 28..=last {
        let tables = page_tables(&document, page);
        for i in 0..tables.len() {
            for j in (i + 1)..tables.len() {
                let (api, aci, ay0, ay1, ax0, ax1) = tables[i];
                let (bpi, bci, by0, by1, bx0, bx1) = tables[j];
                let w = ax1.min(bx1) - ax0.max(bx0);
                let h = ay1.min(by1) - ay0.max(by0);
                assert!(
                    w <= TOLERANCE_PX || h <= TOLERANCE_PX,
                    "쪽 idx {page} 에서 자리차지 표 두 장이 {w:.1} × {h:.1}px 겹쳤다 — \
                     pi={api:?} ci={aci:?} vs pi={bpi:?} ci={bci:?}"
                );
            }
        }
    }
}

/// **음성 통제군** — `#2813` 통짜-배치 구제는 그대로 살아 있어야 한다.
///
/// 새 가드는 "이 쪽이 자기 문단의 continuation 조각으로 시작할 때"만 구제를 끈다.
/// 이 fixture 의 스택은 조각 없이 1쪽에 통째로 앉으므로 가드가 발동하면 안 되고,
/// 발동하면 쪽수가 2 → 3 으로 늘어난다(`#2813` 원 회귀).
#[test]
fn issue_2813_whole_placement_rescue_still_applies() {
    let document = HwpDocument::from_bytes(&read(CONTROL_SAMPLE)).expect("통제군 로드");
    assert_eq!(
        document.page_count(),
        2,
        "#2813 스택은 1쪽에 통째로 앉아 2쪽이어야 한다 — 새 가드가 조각 없는 쪽까지 \
         끄면 3쪽으로 과분할된다"
    );
}
