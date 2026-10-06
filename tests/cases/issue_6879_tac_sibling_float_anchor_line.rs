//! [Issue #6879] 한 문단의 **TAC 라벨 표 위로 자리차지 float 이 올라가** '붙임' 라벨이
//! 쪽 맨 아래로 밀리고 내용 표와 겹치던 결함의 가드.
//!
//! ## 형상
//!
//! 문단 73 에 표가 둘 달렸다 — `ci=0` 은 글자처럼 취급(TAC) 라벨, `ci=1` 은 비-TAC
//! 자리차지 float 이다. 저장 사다리가 둘의 자리를 적어 두었다.
//!
//! ```text
//!   줄0  vertpos = 0      vertsize = 3580 (47.7px)   textpos = 0   ← TAC 라벨이 타는 줄
//!   줄1  vertpos = 4060   vertsize = 1500            textpos = 8   ← float 앵커
//!   float vOff = 2512 (33.5px)
//! ```
//!
//! `textpos=8` 은 첫 인라인 컨트롤(8 슬롯) 다음이라 **float 의 제어 문자는 줄1** 이다.
//!
//! ## 왜 뒤집혔나 — `#5807` 판별식이 문단 첫 줄 기준이었다
//!
//! ```text
//!   문단 상단 기준(결함)  2512 < TAC 줄 높이 3580  → "겹침" → float 을 TAC 앞에 놓는다
//!   앵커 줄 기준(정답)    4060 + 2512 = 6572 ≥ 3580 → 비겹침 → TAC 가 문단 상단에 남는다
//! ```
//!
//! 오판으로 `should_sort_para_float_tables` 가 꺼지면 정렬 tiebreak(비-TAC 0, TAC 1)이
//! float 을 앞세운다. 그 뒤 라벨은 갈 곳을 잃고 흐름 끝(909.1)으로 밀린다.
//!
//! ## 수정 전 → 후
//!
//! ```text
//!   전   ci=1 y=128.0 (h 810.9)  ·  ci=0 y=909.1 (h 40.2)   → 29.8px 겹침
//!   후   ci=0 y= 98.2 .. 138.4   ·  ci=1 y=182.1
//! ```
//!
//! ## 정본
//!
//! `pdf/156767332-broadcast-revenue-attachment-2020.pdf` (engine 2020 — 저장 제품
//! `hancom-office-2022`, 규약 §3.5.1 의 2020 버킷). 라벨 글자 y=107.5 · 그림 185.6/576.5
//! 이고 rhwp 는 라벨 98.2..138.4 · 그림 184.0/575.4 로 **1.6px 안**에서 맞는다. 총 8쪽.
//!
//! ## 곁가지 — 글자 없는 문단의 앵커 사상
//!
//! 이 문단은 `hp:t` 가 하나도 없어 `char_offsets` 가 비고, `control_text_positions` 의
//! char 축(0,1)을 저장 `text_start` 의 HWP5 UTF-16 축(0,8)에 사상할 수 없다. 인라인
//! 개체가 축을 정확히 8유닛씩 차지하므로 **앞선 인라인 개체 수 × 8** 로 잡는다. 이
//! 폴백이 없으면 앵커 오프셋이 0 이 되어 수정 전체가 무효가 된다(실측: 좌표 불변).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6879/156767332-broadcast-revenue-attachment.hwp";
/// 대상 쪽 (0-based). 인쇄 쪽번호 `- 7 -`.
const PAGE: u32 = 6;
/// 형제 표가 달린 문단.
const HOST_PARA: usize = 73;

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()))
}

fn document() -> HwpDocument {
    HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드")
}

fn page_tree(document: &HwpDocument, page: u32) -> RenderNode {
    document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"))
        .root
}

/// `Column` 직계 표를 `(ci, y0, y1)` 로 모은다 — 칸 안 중첩 표는 세지 않는다.
fn host_tables(node: &RenderNode, in_column: bool, out: &mut Vec<(usize, f64, f64)>) {
    if in_column {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(HOST_PARA) {
                if let Some(ci) = table.control_index {
                    out.push((ci, node.bbox.y, node.bbox.y + node.bbox.height));
                }
            }
            return;
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        host_tables(child, in_column, out);
    }
}

fn images(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        images(child, out);
    }
}

fn sorted_host_tables(document: &HwpDocument) -> Vec<(usize, f64, f64)> {
    let mut tables = Vec::new();
    host_tables(&page_tree(document, PAGE), false, &mut tables);
    tables.sort_by(|a, b| a.1.total_cmp(&b.1));
    tables
}

/// TAC 라벨이 자리차지 float **앞**에 온다 — 저장 차례 그대로.
#[test]
fn tac_label_precedes_the_float_sibling() {
    let document = document();
    let tables = sorted_host_tables(&document);
    assert_eq!(
        tables.len(),
        2,
        "문단 {HOST_PARA} 의 형제 표 둘이 7쪽에 있어야 한다 — got {tables:?}"
    );
    assert_eq!(
        (tables[0].0, tables[1].0),
        (0, 1),
        "위에서부터 TAC 라벨(ci=0) → 자리차지 float(ci=1) 이어야 한다 — \
         회귀 시 float 이 y=128.0, 라벨이 y=909.1 로 뒤집힌다. got {tables:?}"
    );
}

/// 라벨은 쪽 상단에, float 은 그 아래에 — 둘이 겹치지 않는다.
#[test]
fn label_stays_at_the_page_top_without_overlapping_the_float() {
    let document = document();
    let tables = sorted_host_tables(&document);
    let (_, label_top, label_bottom) = tables[0];
    let (_, float_top, _) = tables[1];

    assert!(
        (90.0..=115.0).contains(&label_top),
        "라벨은 쪽 상단(정본 글자 107.5)이어야 한다 — got {label_top:.1}"
    );
    assert!(
        float_top >= label_bottom - 0.5,
        "float 이 라벨 아래에서 시작해야 한다 — 라벨 {label_top:.1}..{label_bottom:.1}, \
         float {float_top:.1} (회귀 시 29.8px 겹친다)"
    );
}

/// float 의 세로 원점은 **앵커 줄** 기준이다 — 정본 그림 위치로 잠근다.
#[test]
fn float_origin_follows_the_anchor_line() {
    let document = document();
    let mut ys = Vec::new();
    images(&page_tree(&document, PAGE), &mut ys);
    ys.sort_by(f64::total_cmp);
    assert_eq!(
        ys.len(),
        2,
        "7쪽 내용 표 안에 그림 둘이 있어야 한다 — got {ys:?}"
    );

    // 정본(engine 2020) 185.6 / 576.5. 절대 좌표를 3px 허용으로 잠근다 —
    // 문단 첫 줄 기준이면 각각 130 / 521 근방으로 55px 위에 온다.
    assert!(
        (182.6..=188.6).contains(&ys[0]),
        "첫 그림이 정본 185.6 근방이어야 한다 — got {:.1}",
        ys[0]
    );
    assert!(
        (573.5..=579.5).contains(&ys[1]),
        "둘째 그림이 정본 576.5 근방이어야 한다 — got {:.1}",
        ys[1]
    );
}

/// 정본과 같은 8쪽이다.
#[test]
fn page_count_matches_the_2020_oracle() {
    assert_eq!(
        document().page_count(),
        8,
        "정본(engine 2020)은 8쪽이다 — pdf/156767332-broadcast-revenue-attachment-2020.pdf"
    );
}
