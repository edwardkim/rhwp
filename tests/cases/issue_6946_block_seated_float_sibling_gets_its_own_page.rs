//! [Issue #6946] 빈 host 문단의 자리차지 표 **형제**가, 앞 표가 block 경로로 통째 앉은 쪽에
//! 겹쳐 놓이던 결함의 가드.
//!
//! ## 형상
//!
//! `samples/issue6946/44529-logistics-policy-rule-amendment.hwp` 구역 3 의 문단 0 은
//! 빈 문단(`text_len=0`)이고 비-TAC 자리차지(`wrap=위아래`, `vert=문단`) 표를 둘 단다.
//!
//! ```text
//!   ci=3  24×13  903.3px  (앞쪽 서식)
//!   ci=4  10×1   894.5px  (뒤쪽 행정정보 공동이용 동의서)
//!   본문 918.4px
//! ```
//!
//! ## 원인 — lane 이 block 으로 앉은 앞 형제를 모른다
//!
//! `try_typeset_empty_para_float_table` 은 `ci=3` 의 lane 예약(903.3 + 20.8 = 924.2px)이
//! available(918.4px)을 넘어 거절하고, `typeset_block_table` 이 저장 사다리 fit 으로 7쪽에
//! 앉힌다 — lane 에는 등록되지 않는다. `ci=4` 는 빈 lane 을 믿고 문단 앵커 0 에 앉아
//! 894.5px ≤ 918.4px 로 **같은 쪽**에 배치된다. `dump-pages` 의 `used=924.2` 가 그
//! 결과다(= max(924.2, 894.5)). `#6795` 는 앞 형제가 **쪼개진**(`PartialTable`) 경우만
//! 막았다.
//!
//! ## 정답지 — 한/글 engine 2020
//!
//! `pdf/issue6946/44529-logistics-policy-rule-amendment-hwp-2020.pdf`(원본 `lastSavedWith` 는
//! hancom-office-2010 → 정책 §3.5.1 의 2020 버킷). **13쪽.** 인쇄 `- 7 -` 은 `(앞쪽)` 서식만,
//! `- 8 -` 은 `(뒤쪽) 행정정보 공동이용 동의서` 로 시작한다. 수정 전 rhwp 는 12쪽이고 7쪽에서
//! 두 표가 879.4px 겹친다.
//!
//! ## 지키는 계약
//!
//! 대상을 `pi`/`ci` 로 직접 집는다 — 표 개수만 보면 대상 표가 사라지거나 다른 쪽으로 가도
//! 통과한다. lane 으로 쌓인 형제(`#6795` fixture 의 통제군)와 조각 형제(`#6795`)는 그대로다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6946/44529-logistics-policy-rule-amendment.hwp";

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// `Column` 의 **직계** 표만 `(pi, ci, y0, y1, x0, x1)` 로 모은다.
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

fn page_tables(document: &HwpDocument, page: u32) -> (Vec<TableBox>, (f64, f64)) {
    let tree = document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"));
    let mut out = Vec::new();
    column_tables(&tree.root, false, &mut out);
    let body = body_bounds(&tree.root).unwrap_or_else(|| panic!("쪽 idx {page} Body"));
    (out, body)
}

/// 괘선 두께·반올림을 넘는 실질 겹침만 센다.
const TOLERANCE_PX: f64 = 8.0;

/// 한/글 정답지와 같은 13쪽 — 사라졌던 한 쪽이 돌아온다.
#[test]
fn page_count_matches_hancom_2020_oracle() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    assert_eq!(
        document.page_count(),
        13,
        "한/글 engine 2020 정답지는 13쪽이다 — 회귀 시 앞·뒤쪽 서식이 한 쪽에 겹쳐 12쪽이 된다"
    );
}

/// 앞쪽 서식 쪽(idx 6)에는 `ci=3` 하나만 있다.
#[test]
fn front_form_page_holds_only_the_front_table() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    let (tables, (_, body_bottom)) = page_tables(&document, 6);
    assert_eq!(
        tables.len(),
        1,
        "쪽 idx 6 에는 앞쪽 서식 표 하나만 있어야 한다 — 회귀 시 `pi=0 ci=4` 가 함께 와 \
         879.4px 겹친다. got {tables:?}"
    );
    let (pi, ci, _, y1, ..) = tables[0];
    assert_eq!(
        (pi, ci),
        (Some(0), Some(3)),
        "쪽 idx 6 의 표는 24×13 앞쪽 서식이어야 한다"
    );
    assert!(
        y1 <= body_bottom + TOLERANCE_PX,
        "앞쪽 서식 하단({y1:.1})이 본문 하한({body_bottom:.1})을 넘었다"
    );
}

/// 뒤쪽 동의서(`ci=4`)는 **다음 쪽 본문 상단**에 단독으로, 본문 안에 놓인다.
#[test]
fn back_form_sibling_gets_its_own_page_inside_the_body() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    let (tables, (body_top, body_bottom)) = page_tables(&document, 7);
    let back: Vec<&TableBox> = tables
        .iter()
        .filter(|(pi, ci, ..)| (*pi, *ci) == (Some(0), Some(4)))
        .collect();
    assert_eq!(
        back.len(),
        1,
        "쪽 idx 7 에 뒤쪽 동의서(pi=0 ci=4)가 있어야 한다. got {tables:?}"
    );
    let (_, _, y0, y1, ..) = *back[0];
    assert!(
        (y0 - body_top).abs() < TOLERANCE_PX,
        "뒤쪽 동의서는 본문 상단({body_top:.1})에서 시작해야 한다 — got {y0:.1}"
    );
    assert!(
        y1 <= body_bottom + TOLERANCE_PX,
        "뒤쪽 동의서 하단({y1:.1})이 본문 하한({body_bottom:.1})을 넘었다 — \
         겹침을 넘침으로 바꾸기만 한 판이다"
    );
}

/// 어느 쪽에서도 자리차지 표 두 장이 겹치지 않는다 — 형상이 바뀌어도 남는 상위 계약.
#[test]
fn no_page_stacks_two_float_tables_on_top_of_each_other() {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    for page in 0..document.page_count() {
        let (tables, _) = page_tables(&document, page);
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

/// Synthetic IR contract, not a Hancom oracle: adding an independent small lane table must
/// not make a following block table appear lane-owned. Preserve the real front form
/// unchanged; construct both empty neighbors with internally consistent dimensions.
#[test]
fn unrelated_lane_does_not_hide_a_block_siblings_occupancy() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::{
        control::Control,
        table::{Cell, Table, TablePageBreak},
    };
    let mut core = DocumentCore::from_bytes(&read(SAMPLE)).expect("document");
    let mut document = core.document().clone();
    let para = &mut document.sections[3].paragraphs[0];
    let Control::Table(front) = &para.controls[3] else {
        panic!("front form");
    };
    let empty_neighbor = |height| {
        let mut common = front.common.clone();
        common.height = height;
        let mut table = Table {
            common,
            row_count: 1,
            col_count: 1,
            row_sizes: vec![1], // HWP row_sizes contains cell counts, not heights.
            cells: vec![Cell::new_empty(
                0,
                0,
                front.common.width,
                height,
                front.border_fill_id,
            )],
            page_break: TablePageBreak::None,
            ..Default::default()
        };
        table.rebuild_grid();
        Control::Table(Box::new(table))
    };
    let first = empty_neighbor(300);
    let last = empty_neighbor(30000);
    para.controls[4] = last;
    para.controls.insert(3, first);
    core.set_document(document);

    let mut seen = [0; 3];
    let dump = core.dump_page_items_json(None);
    for page in 0..core.page_count() {
        if dump[page as usize]["section"] != 3 {
            continue;
        }
        let tree = core.build_page_render_tree(page).expect("tree");
        let mut tables = Vec::new();
        column_tables(&tree.root, false, &mut tables);
        let siblings: Vec<_> = tables
            .iter()
            .filter(|t| t.0 == Some(0) && matches!(t.1, Some(3..=5)))
            .collect();
        for table in &siblings {
            seen[table.1.unwrap() - 3] += 1;
        }
        for (i, a) in siblings.iter().enumerate() {
            for b in siblings.iter().skip(i + 1) {
                let width = a.5.min(b.5) - a.4.max(b.4);
                let height = a.3.min(b.3) - a.2.max(b.2);
                assert!(width <= 1.0 || height <= 1.0,
                    "page {page}: unrelated lane allowed sibling overlap {width} x {height}: {a:?} / {b:?}");
            }
        }
    }
    assert_eq!(
        seen,
        [1, 1, 1],
        "all three whole tables must be present once"
    );
}
