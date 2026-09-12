//! [Issue #7047] 빈 개체 host 문단의 흐름 전진을 stale 사다리 가드가 **통째로** 삼키던
//! 결함의 가드.
//!
//! `textless_host_ladder_line_advance` 는 "이 빈 host 문단이 한 줄을 예약했는가" 를 저장
//! `LINE_SEG` 델타로 묻는다. 그 판정의 기대값을 `줄높이 + 줄간격` 으로만 잡았는데, 저장
//! 델타에는 **문단 간격까지** 실려 있다(같은 파일 `ladder_delta_px` 주석도 "저장 델타 =
//! sb+lh+ls 전량" 이라고 적는다).
//!
//! 재현체 3쪽 빈 개체 host 일곱 개 전량 실측 — 델타가 정확히 그 합이다.
//!
//! ```text
//!   host      델타 = 줄높이 + 줄간격 + 문단뒤간격 + 다음문단앞간격
//!   rec#829   1720 =  1100 +  220 +  200 +  200
//!   rec#847   2120 =  1400 +  420 +    0 +  300
//!   rec#947    886 =   450 +  136 +    0 +  300
//!   rec#958    494 =   150 +   44 +    0 +  300
//!   rec#1041  1794 =  1150 +  344 +    0 +  300     (rec#1052 · rec#1092 동형)
//! ```
//!
//! 간격을 빼면 **줄 높이가 작은 host 둘**(450 · 150 HWPUNIT)만 `델타/기대` 가 1.51 · 2.55
//! 로 커져 가드 `delta * 2 > expected * 3` 에 걸린다. 그러면 판별 불가로 물러나고 그 문단이
//! 흐름을 한 픽셀도 전진시키지 않아, 뒤따르는 개체가 저장 사다리보다 그 델타만큼 위에
//! 놓인다. 간격을 넣으면 일곱 개 전부 비율 1.0 이 된다.
//!
//! 돌연변이 검정으로 인과를 단독 확정했다 — host 문단에 글자 한 자를 넣어 "빈 host" 분기를
//! 벗어나게 하면, 아래 전체가 정확히 그 문단의 저장 델타만큼 내려온다(rec#947 +11.8px =
//! 886 HU, rec#958 +6.6px = 494 HU, 0.01px 일치).
//!
//! ⚠ 이 시험이 잠그는 것은 **rec#958 한 host 의 전진**이다. 같은 쪽의 나머지 두 계열은
//! 아직 열려 있다 — `rec#829`(TopAndBottom 도형)은 사다리 질의 대상 wrap 열거 밖이고,
//! `rec#947`(표 host)은 `para_has_visible_textless_float_shape_item` 이 Picture/Shape 만
//! 매칭해 이 경로에 들어오지도 않는다(표 float 레인 · `#2097` 계약). 정본 목표와 남은
//! 격차는 아래 상수에 적어 둔다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue_7047/housing-lease-standard-form.hwp";

/// `rec#958` 의 저장 델타 494 HWPUNIT.
const REC958_STORED_DELTA_PX: f64 = 494.0 * 96.0 / 7200.0;

/// 3쪽 글상자 세 개의 상단 y(px) — 수정 후.
///
/// `글상자C` 는 수정 전 800.2 였다(= 806.8 − 494HU). 위 둘은 `rec#958` 보다 앞이라
/// 움직이지 않아야 한다.
const BOX_A_Y: f64 = 123.4;
const BOX_B_Y: f64 = 467.4;
const BOX_C_Y: f64 = 806.8;

/// 3쪽 제목 표 세 개의 상단 y(px) — 수정 후. `표3` 은 수정 전 807.6 이었다.
const TABLE_1_Y: f64 = 108.4;
const TABLE_2_Y: f64 = 463.8;
const TABLE_3_Y: f64 = 814.1;

/// 정본(한/글 2020, 새로 생성한 PDF) 기준 `글상자C` 상단 — 남은 격차 51.3px 은 위 ⚠ 의
/// 두 계열 몫이다. 이 시험은 그 값을 주장하지 않고, 이 수정이 낸 6.6px 만 잠근다.
const BOX_C_ORACLE_Y: f64 = 858.1;

const TOL: f64 = 0.6;

fn page3() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).expect("재현체 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(2)
        .expect("3쪽 render tree")
        .root
}

/// 폭·높이가 기준 이상인 글상자(Rectangle)·표(Table) 노드의 상단 y 를 오름차순으로 모은다.
///
/// 높이 기준은 3쪽 머리의 작은 안내 글상자(670.5 × 31.3px)와 줄 안 장식 사각형을 걸러낸다.
fn tops(node: &RenderNode, want_table: bool, min_w: f64, min_h: f64, out: &mut Vec<f64>) {
    let hit = match &node.node_type {
        RenderNodeType::Table(_) => want_table,
        RenderNodeType::Rectangle(_) => !want_table,
        _ => false,
    };
    if hit && node.bbox.width >= min_w && node.bbox.height >= min_h {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        tops(child, want_table, min_w, min_h, out);
    }
}

fn sorted_tops(root: &RenderNode, want_table: bool, min_w: f64, min_h: f64) -> Vec<f64> {
    let mut v = Vec::new();
    tops(root, want_table, min_w, min_h, &mut v);
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    v
}

/// 빈 개체 host `rec#958` 이 저장 델타만큼 흐름을 전진시켜야 한다 — 그 아래 글상자가
/// 그만큼 내려온다.
#[test]
fn textless_float_host_advances_by_its_stored_ladder_delta() {
    let root = page3();
    let boxes = sorted_tops(&root, false, 600.0, 60.0);
    assert_eq!(boxes.len(), 3, "3쪽 전폭 글상자 셋: {boxes:?}");
    assert!(
        (boxes[2] - BOX_C_Y).abs() < TOL,
        "글상자C 상단 {:.1} != {BOX_C_Y} (수정 전 {:.1}, 저장 델타 {:.2}px)",
        boxes[2],
        BOX_C_Y - REC958_STORED_DELTA_PX,
        REC958_STORED_DELTA_PX
    );
}

/// `rec#958` 보다 앞에 있는 글상자 둘은 움직이지 않아야 한다.
#[test]
fn the_boxes_anchored_above_that_host_do_not_move() {
    let root = page3();
    let boxes = sorted_tops(&root, false, 600.0, 60.0);
    assert_eq!(boxes.len(), 3, "3쪽 전폭 글상자 셋: {boxes:?}");
    assert!(
        (boxes[0] - BOX_A_Y).abs() < TOL,
        "글상자A 상단 {:.1} != {BOX_A_Y}",
        boxes[0]
    );
    assert!(
        (boxes[1] - BOX_B_Y).abs() < TOL,
        "글상자B 상단 {:.1} != {BOX_B_Y}",
        boxes[1]
    );
}

/// 같은 host 아래 떠있는 제목 표도 같은 델타만큼 내려온다 — 위 둘은 제자리다.
#[test]
fn the_floating_title_tables_follow_the_same_ladder() {
    let root = page3();
    let tables = sorted_tops(&root, true, 250.0, 10.0);
    assert_eq!(tables.len(), 3, "3쪽 제목 표 셋: {tables:?}");
    for (got, want) in tables.iter().zip([TABLE_1_Y, TABLE_2_Y, TABLE_3_Y]) {
        assert!(
            (got - want).abs() < TOL,
            "제목 표 상단 {got:.1} != {want} (전체 {tables:?})"
        );
    }
}

/// 이 수정은 정본 쪽으로 움직인다 — 남은 격차가 수정 전보다 작아야 한다.
#[test]
fn the_correction_moves_toward_the_hangul_oracle() {
    let root = page3();
    let boxes = sorted_tops(&root, false, 600.0, 60.0);
    let before = BOX_C_Y - REC958_STORED_DELTA_PX;
    let gap_before = (BOX_C_ORACLE_Y - before).abs();
    let gap_after = (BOX_C_ORACLE_Y - boxes[2]).abs();
    assert!(
        gap_after < gap_before,
        "정본 격차가 줄지 않았다 — 전 {gap_before:.2}px · 후 {gap_after:.2}px"
    );
    assert!(
        (gap_before - gap_after - REC958_STORED_DELTA_PX).abs() < TOL,
        "줄어든 폭 {:.2}px 이 저장 델타 {:.2}px 과 다르다",
        gap_before - gap_after,
        REC958_STORED_DELTA_PX
    );
}
