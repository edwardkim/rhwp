//! [Issue #7441] 용지 기준(vert=Paper) 글앞/글뒤 표가 본문 안에 통째로 들어가는데도
//! 행 단위로 잘리고, 잘린 행이 어느 쪽에도 그려지지 않던 결함의 회귀 가드.
//!
//! 근인: overlay 표 잔여 행 컷(#4568, `typeset/controls/decoration_table.rs`)이
//! `anchor_y = 흐름 높이 + vertical_offset` 을 본문 높이에서 뺐다. 문단 기준 표에만 맞는
//! 식이다. 용지 기준 `vertical_offset` 은 용지 맨 위에서 재므로 본문 시작(머리말 여백 +
//! 위 여백)만큼 표가 더 아래에 있다고 오판해 본문 안의 표를 잘랐다. 잘린 행은 다음 쪽
//! 상단에 이어 그려야 하지만, 다음 쪽 맨 위에 그 쪽의 표가 있으면 #4514 겹침 가드가
//! 이어 그릴 자리를 0 으로 막아 행이 통째로 사라졌다.
//!
//! 독립 기준(한컴 출력, 표 하단·글자 위치는 PDF 실측):
//! - `pdf/table-complex-hwp-2020.pdf` 4쪽: 쪽마다 용지 기준 표가 한 쪽에 통째로 있고 표 하단
//!   177.5·182.4·182.9mm < 본문 끝 195mm(용지 210 − 아래 여백 15).
//! - `pdf/table-ipc-hwp-2020.pdf` 10쪽: 표 하단 최대 189.5mm < 본문 끝 195mm.
//!
//! 꼬리말 영역까지 뻗은 용지 기준 표도 한컴은 행을 다음 쪽으로 넘기지 않는다(온새미로 표지 틀,
//! 한컴 2020·2024 PDF). 이 문서는 같은 원본의 시각 일치율이 1쪽에서 90% 미만(표지 제목 글꼴)이라
//! 정식 회귀로 두지 않고 `mydocs/tech/investigations/issue-7441/probes/` 에 근거로 보존한다.
//!
//! 검사는 쪽별 표 소속(모든 행이 앵커 쪽에 있고 다른 쪽에 조각이 없음)과 본문 영역 포함 관계로
//! 한다. 본문 끝은 한컴 도움말의 여백 구조(용지 높이 − 꼬리말 여백 − 아래 여백)로 저장 PageDef
//! 에서 직접 계산한다. 실제 위치·모양은 위 한컴 PDF 의 Visual Sweep 으로 따로 확인한다.
//!
//! 비적용 반례(문단 기준 표는 계속 쪽 하단에서 분할)는
//! `tests/issue_4514_overlay_table_flow.rs` 의 ECR-004 연속 조각 단언이 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const TABLE_COMPLEX: &str = "samples/table-complex.hwp";
const TABLE_IPC: &str = "samples/table-ipc.hwp";

/// 쪽에 그려진 최외곽 표 한 조각.
struct PaintedTable {
    key: (usize, usize, usize),
    row_count: u16,
    rows: BTreeSet<u16>,
    bottom: f64,
}

fn load(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn collect_cells(node: &RenderNode, table: &mut PaintedTable) {
    for child in &node.children {
        match &child.node_type {
            RenderNodeType::TableCell(cell) => {
                let span = cell.row_span.max(1);
                table.rows.extend(cell.row..cell.row + span);
            }
            // 셀 밖 중첩 표는 이 표의 행이 아니다.
            RenderNodeType::Table(_) => {}
            _ => collect_cells(child, table),
        }
    }
}

fn collect_tables(node: &RenderNode, out: &mut Vec<PaintedTable>) {
    match &node.node_type {
        RenderNodeType::Table(tn) if tn.cell_context.is_none() => {
            let mut table = PaintedTable {
                key: (
                    tn.section_index.unwrap_or(usize::MAX),
                    tn.para_index.unwrap_or(usize::MAX),
                    tn.control_index.unwrap_or(usize::MAX),
                ),
                row_count: tn.row_count,
                rows: BTreeSet::new(),
                bottom: node.bbox.y + node.bbox.height,
            };
            collect_cells(node, &mut table);
            out.push(table);
        }
        RenderNodeType::TableCell(_) => {}
        _ => {
            for child in &node.children {
                collect_tables(child, out);
            }
        }
    }
}

fn painted_tables(doc: &DocumentCore, page: u32) -> Vec<PaintedTable> {
    let tree = doc
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree p{}: {e}", page + 1));
    let mut out = Vec::new();
    collect_tables(&tree.root, &mut out);
    out
}

fn page_def(doc: &DocumentCore, section: usize) -> serde_json::Value {
    let json = doc
        .get_page_def_native(section)
        .unwrap_or_else(|e| panic!("page def s{section}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("page def json s{section}: {e}"))
}

fn hu(def: &serde_json::Value, key: &str) -> i64 {
    def[key]
        .as_i64()
        .unwrap_or_else(|| panic!("page def 에 {key} 없음: {def}"))
}

fn hu_to_px(v: i64) -> f64 {
    v as f64 * 96.0 / 7200.0
}

/// 저장 PageDef 로 본 용지 높이(HWPUNIT). 가로 방향이면 width/height 를 바꾼다.
fn paper_height_hu(def: &serde_json::Value) -> i64 {
    if def["landscape"].as_bool().unwrap_or(false) {
        hu(def, "width")
    } else {
        hu(def, "height")
    }
}

/// 본문 끝 = 꼬리말 시작(용지 좌표 px) = 용지 높이 − 꼬리말 여백 − 아래 여백.
fn footer_start_px(def: &serde_json::Value) -> f64 {
    hu_to_px(paper_height_hu(def) - hu(def, "marginFooter") - hu(def, "marginBottom"))
}

/// 쪽마다 최외곽 표의 모든 행이 그 쪽에 그려지고, 표가 본문 끝 안에 있고, 같은 표가 다른 쪽에
/// 잔여 조각으로 다시 나타나지 않는지 본다.
fn assert_every_table_whole_on_one_page(doc: &DocumentCore, label: &str) {
    let mut pages_of: BTreeMap<(usize, usize, usize), Vec<u32>> = BTreeMap::new();
    for page in 0..doc.page_count() {
        for table in painted_tables(doc, page) {
            let missing: Vec<u16> = (0..table.row_count)
                .filter(|r| !table.rows.contains(r))
                .collect();
            assert!(
                missing.is_empty(),
                "{label} p{}: 표 {:?} 의 {}행 중 {:?} 가 이 쪽에 그려지지 않았다. \
                 한컴은 표 전체를 한 쪽에 둔다 (#7441)",
                page + 1,
                table.key,
                table.row_count,
                missing
            );
            let body_bottom = footer_start_px(&page_def(doc, table.key.0));
            assert!(
                table.bottom <= body_bottom + 0.5,
                "{label} p{}: 표 {:?} 하단 {:.1}px 가 본문 끝 {:.1}px 를 넘는다",
                page + 1,
                table.key,
                table.bottom,
                body_bottom
            );
            pages_of.entry(table.key).or_default().push(page);
        }
    }
    for (key, pages) in &pages_of {
        assert_eq!(
            pages.len(),
            1,
            "{label}: 표 {key:?} 가 여러 쪽({pages:?})에 조각으로 나뉘었다 (#7441)"
        );
    }
}

#[test]
fn issue_7441_table_complex_paper_overlay_tables_stay_whole() {
    let doc = load(TABLE_COMPLEX);
    assert_eq!(doc.page_count(), 4, "한컴 기준 PDF 는 4쪽이다");
    assert_every_table_whole_on_one_page(&doc, TABLE_COMPLEX);
}

#[test]
fn issue_7441_table_ipc_paper_overlay_tables_stay_whole() {
    let doc = load(TABLE_IPC);
    assert_eq!(doc.page_count(), 10, "한컴 기준 PDF 는 10쪽이다");
    assert_every_table_whole_on_one_page(&doc, TABLE_IPC);
}
