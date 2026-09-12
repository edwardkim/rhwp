//! [Issue #7047] 글자 없는 떠있는 개체 host 문단이 흐름을 전진시키지 않아, 뒤따르는
//! 개체가 저장 사다리보다 위에 놓이고 제목 표가 글상자 첫 줄과 겹치던 결함의 가드.
//!
//! 제보는 "큰 글꼴 제목 문단 다음 줄의 y 가 제목 줄 높이를 반영하지 않는다" 였지만, 겹치는
//! 두 줄은 같은 흐름의 연속 문단이 아니다 — 하나는 쪽 직속 떠있는 표, 하나는 글상자 안
//! 문단이다. 본문 흐름은 정본과 dy 0.00 으로 일치한다. 실제 원인은 **개체를 매단 빈 문단이
//! 흐름에서 자기 줄을 차지하지 않는 것**이고, 세 관문이 겹쳐 있었다.
//!
//! ① `textless_host_ladder_line_advance` 의 기대값이 `줄높이 + 줄간격` 뿐이었다. 저장
//!    델타에는 **문단 간격**(host 뒤 + 다음 앞)까지 실려 있어서, 줄 높이가 작은 host 만
//!    `델타/기대` 가 1.5 를 넘어 stale 가드에 걸려 판별 불가로 물러났다.
//! ② `#703` 데코레이션(글앞/글뒤) 표 단축은 표만 방출하고 흐름을 0 소비한다. 가시 텍스트가
//!    있는 host 는 보완됐지만 **글자 없는 host** 는 `PageItem` 이 하나도 없어 렌더가 그
//!    문단을 건너뛰고 저장 vpos 보정조차 받지 못했다.
//! ③ 렌더의 두 술어(`para_has_visible_textless_float_shape_item` · `has_overlay_float`)가
//!    Picture/Shape 만 매칭해 **표 host** 는 ②로 항목을 얻어도 사다리 전진을 못 받았다.
//!
//! 셋 중 하나만 고치면 닫히지 않는다 — ③ 을 한쪽 술어에만 넣으면 사다리 질의가 돌지 않아
//! 휴리스틱이 "전진 없음"으로 답해 **오히려 나빠진다**(최대 |dy| 19.6 → 27.6px 실측).
//!
//! 재현체 3쪽 빈 개체 host 일곱 개 전량에서 저장 델타가 정확히 그 합이다.
//!
//! ```text
//!   host      델타 = 줄높이 + 줄간격 + host 뒤간격 + 다음 앞간격
//!   rec#829   1720 =  1100 +  220 +  200 +  200
//!   rec#847   2120 =  1400 +  420 +    0 +  300
//!   rec#947    886 =   450 +  136 +    0 +  300
//!   rec#958    494 =   150 +   44 +    0 +  300
//!   rec#1041  1794 =  1150 +  344 +    0 +  300     (rec#1052 · rec#1092 동형)
//! ```
//!
//! 인과는 돌연변이 검정으로 단독 확정했다 — host 문단에 글자 한 자를 넣어 "빈 host" 분기를
//! 벗어나게 하면 아래 전체가 정확히 그 문단의 저장 델타만큼 내려온다(rec#947 +11.8px =
//! 886 HU, rec#958 +6.6px = 494 HU, 0.01px 일치).
//!
//! 정본(engine 2020 새 PDF)과 3쪽 글자 1,276자를 전량 정합한 최대 |dy| 는
//! **57.9px → 1.79px** 다. 남은 1.7px 이하는 별개 계열이다 — `table_layout.rs` 가 문단
//! 기준 떠있는 표에 `outer_margin_top`(283 HWPUNIT = 1.88px) 을 더하지 않는다. 표별 실측
//! 편차는 1.7 / 1.3 / 0.9px 로 일정하지 않고 그 상한 안에서 흩어지므로, 아래 시험은
//! 상수 일치가 아니라 **부호와 상한**만 잠근다(후속 과제).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue_7047/housing-lease-standard-form.hwp";

/// 3쪽 전폭 글상자 세 개의 상단 y(px). 정본 환산값 123.3 / 501.2 / 858.1 과 0.1px 안에서
/// 같다 — 이 축은 완전히 닫혔다.
const BOX_TOPS: [f64; 3] = [123.4, 501.2, 858.2];

/// 3쪽 떠있는 제목 표 세 개의 상단 y(px).
const TABLE_TOPS: [f64; 3] = [108.4, 487.6, 846.2];

/// 정본(한/글 2020 새 PDF) 제목 표 상단. rhwp 는 셋 다 이보다 조금 위다 — 실측 편차는
/// 1.7 / 1.3 / 0.9px 로 **일정하지 않고**, 문단 기준 떠있는 표가 못 받는
/// `outer_margin_top`(283 HWPUNIT = 1.88px) 안에서 흩어진다.
const TABLE_TOPS_ORACLE: [f64; 3] = [110.1, 488.9, 847.1];

/// 그 편차의 상한 — `outer_margin_top` 283 HWPUNIT. 후속 과제의 크기를 이 값으로 못 박는다.
const TABLE_OM_TOP_LIMIT_PX: f64 = 1.88;

const TOL: f64 = 0.6;

fn page3() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).expect("재현체 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(2)
        .expect("3쪽 render tree")
        .root
}

/// 3쪽 전폭 글상자인가 — 머리의 작은 안내 상자(670×31)와 줄 안 장식은 걸러진다.
fn is_wide_textbox(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Rectangle(_))
        && node.bbox.width >= 600.0
        && node.bbox.height >= 60.0
}

/// 쪽 직속 떠있는 제목 표인가.
fn is_title_table(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Table(_)) && node.bbox.width >= 250.0
}

fn tops(node: &RenderNode, pick: fn(&RenderNode) -> bool, out: &mut Vec<f64>) {
    if pick(node) {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        tops(child, pick, out);
    }
}

fn sorted_tops(root: &RenderNode, pick: fn(&RenderNode) -> bool) -> Vec<f64> {
    let mut v = Vec::new();
    tops(root, pick, &mut v);
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    v
}

/// 이 부분트리 안 `TextLine` 의 (상단, 하단).
fn text_line_bands(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        text_line_bands(child, out);
    }
}

/// 빈 개체 host 가 저장 델타만큼 흐름을 전진시켜야 한다 — 글상자 셋이 정본 자리에 놓인다.
#[test]
fn textless_float_hosts_advance_by_their_stored_ladder_delta() {
    let got = sorted_tops(&page3(), is_wide_textbox);
    assert_eq!(got.len(), 3, "3쪽 전폭 글상자 셋: {got:?}");
    for (have, want) in got.iter().zip(BOX_TOPS) {
        assert!(
            (have - want).abs() < TOL,
            "글상자 상단 {have:.1} != {want} (전체 {got:?})"
        );
    }
}

/// 떠있는 제목 표 셋도 같은 사다리를 따른다.
#[test]
fn the_floating_title_tables_follow_the_same_ladder() {
    let got = sorted_tops(&page3(), is_title_table);
    assert_eq!(got.len(), 3, "3쪽 제목 표 셋: {got:?}");
    for (have, want) in got.iter().zip(TABLE_TOPS) {
        assert!(
            (have - want).abs() < TOL,
            "제목 표 상단 {have:.1} != {want} (전체 {got:?})"
        );
    }
}

/// 표에 남은 편차는 **한 방향**(정본보다 위)이고 `outer_margin_top` 안이어야 한다 — 이
/// 이슈가 남긴 잔여가 그 한 축뿐이라는 상한이다. 부호가 뒤집히거나 상한을 넘으면 다른 축이
/// 섞인 것이다.
#[test]
fn the_remaining_table_gap_stays_within_the_outer_margin() {
    let got = sorted_tops(&page3(), is_title_table);
    assert_eq!(got.len(), 3, "3쪽 제목 표 셋: {got:?}");
    for (have, oracle) in got.iter().zip(TABLE_TOPS_ORACLE) {
        let gap = oracle - have;
        assert!(
            gap > 0.0 && gap <= TABLE_OM_TOP_LIMIT_PX + 0.05,
            "표 상단 편차 {gap:.2}px 이 (0, {TABLE_OM_TOP_LIMIT_PX}] 밖이다 (상단 {have:.1})"
        );
    }
}

/// 제보된 겹침이 사라져야 한다 — 쪽 하단 제목 표의 글줄이 그 아래 글상자 첫 줄을 침범하지
/// 않는다. 수정 전에는 제목 줄 바닥 828.7 이 글상자 첫 줄 상단 821.0 을 7.7px 파고들었다.
#[test]
fn the_title_table_line_no_longer_overlaps_the_textbox_first_line() {
    let root = page3();

    let mut title: Option<(f64, f64)> = None;
    fn scan_titles(node: &RenderNode, out: &mut Option<(f64, f64)>) {
        if is_title_table(node) {
            let mut bands = Vec::new();
            text_line_bands(node, &mut bands);
            if let Some(band) = bands
                .into_iter()
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            {
                if out.map(|cur| band.0 > cur.0).unwrap_or(true) {
                    *out = Some(band);
                }
            }
        }
        for child in &node.children {
            scan_titles(child, out);
        }
    }
    scan_titles(&root, &mut title);
    let (title_top, title_bottom) = title.expect("제목 표 글줄");

    let mut first_below: Option<f64> = None;
    fn scan_boxes(node: &RenderNode, after: f64, out: &mut Option<f64>) {
        if is_wide_textbox(node) {
            let mut bands = Vec::new();
            text_line_bands(node, &mut bands);
            for (top, _) in bands {
                if top > after && out.map(|cur| top < cur).unwrap_or(true) {
                    *out = Some(top);
                }
            }
        }
        for child in &node.children {
            scan_boxes(child, after, out);
        }
    }
    scan_boxes(&root, title_top, &mut first_below);
    let box_line_top = first_below.expect("제목 아래 글상자 첫 글줄");

    assert!(
        box_line_top > title_bottom,
        "제목 줄[{title_top:.1}..{title_bottom:.1}] 이 글상자 첫 줄 {box_line_top:.1} 을 \
         침범한다 (겹침 {:.1}px)",
        title_bottom - box_line_top
    );
}
