//! [Issue #7063] 글자 없는 자리차지 표 host 의 저장-vpos 후방 스냅이 앞 문단 마지막
//! 줄 상자를 파고든다.
//!
//! `vpos_adjust` 의 저장-vpos 스냅은 목적지에서 현재 문단의 `spacing_before` 를 미리
//! 빼고(#643) 뒤에서 그 문단을 조판할 때 다시 더해 상쇄한다. 그런데 **글자 없는
//! 자리차지 표 host** 는 조판할 글줄이 없어 그 재가산 경로를 타지 않는다(#7203).
//! 저장 사다리 증인(`native_empty_single_topbottom_table_saved_top`)까지 없으면 그
//! 스냅 좌표가 곧 표 윗변이 되어 표가 앞 줄 상자 안으로 들어간다.
//!
//! 실측 — `samples/hwpctl_API_v2.4.hwp` 60쪽(0-기준 59) `pi=1465`:
//!
//! ```text
//!   앞 문단 pi=1464 줄 상자   y=191.73  h=13.33  →  바닥 205.07
//!   흐름 커서(스냅 이전)                              209.07
//!   표 outMargin.top 141HU                          +  1.88
//!   ─────────────────────────────────────────────────────────
//!   기대 윗변                                         210.95
//!   한/글 정본 pdf/hwpctl_API_v2.4-hwp-2020.pdf        210.81   (차 0.14)
//!   수정 전 rhwp                                      202.40   ← 앞 줄을 2.67px 문다
//! ```
//!
//! 정본 좌표는 한/글 PDF(`Hancom PDF 1.3.0.550` · `Hwp 2022`)의 가로 괘선 `y0` 를
//! 96/72 로 환산해 떴다. 같은 문서 자리차지 표 280개 중 앞 줄을 무는 것은 이 형상
//! 4개뿐이고, 저장 사다리 증인이 있는 32건은 이 갈래에 오지 않는다.
//!
//! 이 검사는 정본 없이도 성립하는 불변식을 함께 잠근다 — **자리차지 표의 윗변은
//! 앞 문단 마지막 줄 상자 바닥보다 위일 수 없다.**

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";
/// 0-기준 쪽 번호(문서 60쪽).
const PAGE_INDEX: u32 = 59;
/// 앞 문단(코드 블록 마지막 줄)의 문단 인덱스.
const PREV_PARA: usize = 1464;
/// 빈 host 자리차지 표의 문단 인덱스.
const TABLE_PARA: usize = 1465;
/// 한/글 정본의 표 윗변(px).
const ORACLE_TOP_PX: f64 = 210.81;

fn find_table_top(node: &RenderNode, para_index: usize) -> Option<f64> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(para_index) && t.cell_context.is_none() {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table_top(child, para_index))
}

fn find_table_bottom(node: &RenderNode, para_index: usize) -> Option<f64> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(para_index) && t.cell_context.is_none() {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table_bottom(child, para_index))
}

fn find_line_box_top(node: &RenderNode, para_index: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_line_box_top(child, para_index))
}

fn find_line_box_bottom(node: &RenderNode, para_index: usize) -> Option<f64> {
    let mut bottom: Option<f64> = None;
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            bottom = Some(node.bbox.y + node.bbox.height);
        }
    }
    for child in &node.children {
        if let Some(child_bottom) = find_line_box_bottom(child, para_index) {
            bottom = Some(bottom.map_or(child_bottom, |b: f64| b.max(child_bottom)));
        }
    }
    bottom
}

#[test]
fn issue_7063_empty_float_host_table_starts_below_previous_line_box() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core
        .build_page_render_tree(PAGE_INDEX)
        .expect("60쪽 render tree");

    let prev_bottom =
        find_line_box_bottom(&page.root, PREV_PARA).expect("앞 문단 pi=1464 줄 상자 — 시험 설정");
    let table_top =
        find_table_top(&page.root, TABLE_PARA).expect("자리차지 표 pi=1465 — 시험 설정");

    // 불변식: 자리차지 표는 앞 문단 줄 상자를 파고들 수 없다.
    // 수정 전 202.40 < 205.07 로 2.67px 파고든다.
    assert!(
        table_top >= prev_bottom - 0.05,
        "자리차지 표 윗변({table_top:.2})이 앞 줄 상자 바닥({prev_bottom:.2})보다 위다"
    );

    // 정본 정합: 흐름 커서 209.07 + outMargin.top 1.88 = 210.95 (정본 210.81).
    // 수정 전 202.40 은 −8.41px 이다.
    assert!(
        (table_top - ORACLE_TOP_PX).abs() <= 1.0,
        "자리차지 표 윗변이 한/글 정본({ORACLE_TOP_PX:.2}px) 1px 안이어야 한다 \
         (수정 전 202.40): {table_top:.2}"
    );
}

/// 대조군 — 같은 쪽의 두 번째 빈 host 자리차지 표(`pi=1468`). 같은 형상이 한 쪽에
/// 두 번 나와 "이 표만 맞춘 것이 아님" 을 잠근다. 정본 530.30px.
#[test]
fn issue_7063_second_empty_float_host_table_matches_oracle() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core
        .build_page_render_tree(PAGE_INDEX)
        .expect("60쪽 render tree");

    let prev_bottom =
        find_line_box_bottom(&page.root, 1467).expect("앞 문단 pi=1467 줄 상자 — 시험 설정");
    let table_top = find_table_top(&page.root, 1468).expect("자리차지 표 pi=1468 — 시험 설정");

    assert!(
        table_top >= prev_bottom - 0.05,
        "자리차지 표 윗변({table_top:.2})이 앞 줄 상자 바닥({prev_bottom:.2})보다 위다"
    );
    assert!(
        (table_top - 530.30).abs() <= 1.0,
        "두 번째 표 윗변이 한/글 정본(530.30px) 1px 안이어야 한다 (수정 전 522.20): {table_top:.2}"
    );
}

/// 반례 — 저장 사다리 증인이 있는 자리차지 표(`pi=1172`, 49쪽)는 이 갈래에 오지
/// 않는다. 수정 전에도 정본(208.73px)과 맞으므로 결함 검출 증거가 아니라 **규칙이
/// 빈 host 자리차지 표 전부로 넓어지는 것을 막는 잠금**이다.
#[test]
fn issue_7063_stored_ladder_witness_table_is_untouched() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(48).expect("49쪽 render tree");

    let table_top = find_table_top(&page.root, 1172).expect("자리차지 표 pi=1172 — 시험 설정");
    assert!(
        (table_top - 208.73).abs() <= 1.0,
        "저장 사다리 증인 표는 종전 좌표(208.73px)를 유지해야 한다: {table_top:.2}"
    );
}

/// 표 윗변 보정 뒤의 점유도 잠근다. 빈 host의 표를 내렸더라도 측정이 그만큼
/// 덜 예약하거나 뒤 문단을 되감으면 이 두 문구가 표 안으로 들어간다.
/// PDF 글자 상단은 `pdftotext -bbox-layout` 60쪽의 option/callback을 96/72로
/// 환산했다. render tree 줄 상자는 glyph보다 약 0.8px 위에서 시작한다.
#[test]
fn issue_7063_following_flow_and_page_boundary_match_hancom() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core
        .build_page_render_tree(PAGE_INDEX)
        .expect("60쪽 render tree");

    for (table_pi, empty_line_pi, following_pi, oracle_glyph_top) in
        [(1465, 1466, 1467, 512.43), (1468, 1469, 1470, 601.39)]
    {
        let table_bottom = find_table_bottom(&page.root, table_pi).expect("표 하단");
        let empty_line_top =
            find_line_box_top(&page.root, empty_line_pi).expect("표 뒤 빈 줄 상자");
        let following_top =
            find_line_box_top(&page.root, following_pi).expect("표 뒤 실제 본문 줄");
        assert!(
            empty_line_top >= table_bottom - 0.1 && following_top > empty_line_top,
            "표 {table_pi} 뒤 빈 줄/본문 점유: 표 하단={table_bottom:.2}, \
             빈 줄={empty_line_top:.2}, 본문={following_top:.2}"
        );
        assert!(
            (following_top - oracle_glyph_top).abs() <= 1.5,
            "표 {table_pi} 뒤 본문 시작 {following_top:.2}가 한컴 PDF 글자 상단 \
             {oracle_glyph_top:.2}와 어긋난다"
        );
    }

    // 한컴 PDF는 105쪽이고 다음 쪽 첫 항목은 pi=1477의 표다. 표 높이를
    // 과소/과대 예약해 경계 항목의 소유 쪽을 바꾸는 퇴행을 함께 검출한다.
    assert_eq!(core.page_count(), 105, "한컴 PDF와 쪽수 일치");
    let next_page = core.build_page_render_tree(PAGE_INDEX + 1).expect("61쪽");
    assert!(
        find_table_top(&next_page.root, 1477).is_some(),
        "다음 쪽 첫 표 pi=1477을 61쪽에 보존해야 한다"
    );
}
