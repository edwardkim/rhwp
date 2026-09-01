//! Issue #6545: 쪽을 넘어온 미주 문단 꼬리가 저장 사다리를 통째로 잃어, 큰 `vertsize`
//! 수식 줄이 제 몫만큼 전진하지 못하고 다음 문단이 그 위에 겹쳐 그려진다.
//!
//! **첫 걸음에서만 감긴다.** HWPX 미주는 쪽 경계에서 `vertpos=0` 으로 리셋하고 그 뒤
//! 줄들을 새 쪽 좌표계로 적는다. 파서(`normalize_hwpx_note_line_vpos`, #1692)는 note 안의
//! `0` 을 연속줄 아티팩트로 보고 `prev + line_height + line_spacing` 으로 되돌리는데, 뒤
//! 줄들은 새 좌표계 그대로라 **리셋 줄 하나만** 앞 쪽 좌표로 튄다.
//!
//! ```text
//! idx  파일 raw   IR          base+raw
//!   9    54920    1487426     1487426  ✓
//!  10        0   *1505494*    1432506  ✗   ← 1487426 + 17616 + 452 (되돌린 값)
//!  11    18068    1450574     1450574  ✓   ← 여기서만 감김, 이후는 단조
//!  12    19420    1451926     1451926  ✓
//! ```
//!
//! 그 한 걸음 때문에 `endnote_line_vpos_base` 의 단조 검사가 꺼지면 꼬리 **전체**가 저장
//! 배치를 잃고 `line_height`(=`vertsize`) 누적으로 떨어진다. seg11 은 `vertsize 2205` /
//! `textheight 900` 이라 진행량이 `2205+452` 로 부풀어 저장 `900+452` 보다 **1305 HU
//! (13.05pt)** 많다.
//!
//! 여기에 `skip_advance_empty_tac_picture` 가 겹친다 — 앞 줄이 예약한 TAC 개체 높이와
//! 높이가 같은 공백 줄을 유령 사본으로 보고 흐름 전진을 0 으로 만든다. 저장 사다리는 그
//! 줄에 **자기 `vertical_pos`** 를 줬으므로 유령이 아니다.
//!
//! 실측 (`samples/3-09월_교육_통합_2022.hwpx` 23쪽, 미주 `s0:p440:ci0` note12→note13):
//!
//! ```text
//! 저장  seg11 vertpos=18068 vertsize=2205 textheight= 900   '따라서 점 Y가 …'
//!       seg12 vertpos=19420 vertsize=2205 textheight=2205   수식
//!       (다음 문단) vertpos=22077                            '(ⅱ) 두 점 P, Q가 …'
//!       → 진행량 1352 + 2657 = 4009 HU = 40.09pt
//!
//! 한글 2024   248.70 → 288.80 pt   ( 40.10pt )
//! rhwp 종전   248.70 → 275.25 pt   ( 26.55pt ) — 수식 줄과 다음 문단이 같은 띠
//! rhwp 수정   248.70 → 288.83 pt   ( 40.13pt )
//! ```
//!
//! 쪽수는 23/23 으로 전후·한글 모두 같고 넘침 지표도 침묵한다. 그래서 좌표를 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/3-09월_교육_통합_2022.hwpx";
const PAGE_INDEX: u32 = 22; // 0-based — 23쪽

/// 미주 note12 (수식으로 끝나는 꼬리 문단) 과 note13 (그 다음 문단).
const TAIL_PARA: usize = 1175;
const NEXT_PARA: usize = 1176;

/// `따라서 점 Y가 …` 줄 → `(ⅱ) 두 점 P, Q가 …` 줄의 저장 진행량.
/// 1352 + 2657 = 4009 HU = 40.09pt = 53.45px @96dpi. 회귀 시에는 35.40px 였다.
const STORED_ADVANCE_PX: f64 = 53.45;
const TOLERANCE_PX: f64 = 1.5;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 이 문단이 만든 글줄 상단 y 목록 (오름차순).
fn line_tops(nodes: &[&RenderNode], para_index: usize) -> Vec<f64> {
    let mut ys: Vec<f64> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    ys
}

#[test]
fn endnote_page_reset_tail_keeps_stored_ladder() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6545 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p23");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    let tail = line_tops(&nodes, TAIL_PARA);
    let next = line_tops(&nodes, NEXT_PARA);
    assert!(
        tail.len() >= 3,
        "꼬리 문단 줄이 3개 이상이어야 한다 (그림줄·본문줄·수식줄): {tail:?}"
    );
    assert!(!next.is_empty(), "다음 문단 줄을 찾지 못했다");

    // ① 겹침 금지 — 수식 줄(꼬리 마지막)과 다음 문단 첫 줄이 같은 띠에 놓이면 안 된다.
    let equation_top = *tail.last().expect("수식 줄");
    let next_top = next[0];
    assert!(
        next_top > equation_top + 10.0,
        "수식 줄({equation_top:.2}) 위에 다음 문단({next_top:.2})이 겹쳤다 — \
         저장 사다리를 잃으면 두 줄이 같은 y 가 된다"
    );

    // ② 저장 진행량 — '따라서 …' 줄에서 다음 문단 첫 줄까지 4009 HU.
    let text_line_top = tail[tail.len() - 2];
    let advance = next_top - text_line_top;
    assert!(
        (advance - STORED_ADVANCE_PX).abs() <= TOLERANCE_PX,
        "저장 진행량 {STORED_ADVANCE_PX:.2}px(40.09pt) 를 벗어났다: {advance:.2}px \
         (본문줄 {text_line_top:.2} → 다음 문단 {next_top:.2}). \
         회귀 시 35.40px — 수식 줄 몫이 통째로 사라진다"
    );
}
