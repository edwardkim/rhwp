//! Issue #1639: 빈 host 문단(비공백 텍스트 없음)에 co-anchored para-relative
//! TopAndBottom float 표가 여러 개 있고, 그 중 후행 표의 vertical_offset 이
//! 음수일 때, vertical_offset 오름차순 정렬이 음수 표를 양수 형제 앞으로 끌어와
//! 문서/배열 순서를 역전시키던 회귀를 막는다.
//!
//! 한컴은 빈 host 라도 음수 offset 이 섞이면 표를 문서/앵커 순서대로 배치한다
//! (#1510: "한글은 앵커/문서 순서대로 배치"). 빈 host 에서 양수 offset 만 있을 때의
//! vertical_offset 재정렬(#986/#1088)은 유지한다.
//!
//! fixture: samples/issue1639_empty_host_negative_offset_float.hwpx
//!   = issue1549_multipositive 구조에서 host 제목 텍스트를 제거(빈 host)하고
//!     후행 표(ci=3) offset 을 음수(-4411 HU = u32 4294962885)로 바꿔, 빈-host
//!     정렬이 배열 순서 [2,3,4] 를 배치 [3,2,4] 로 역전시키는 케이스.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1639_empty_host_negative_offset_float.hwpx";
const POSITIVE_SAMPLE: &str = "samples/issue1639_empty_host_positive_only_float.hwpx";
const TARGET_PI: usize = 0;
const TARGET_TABLES: [usize; 3] = [2, 3, 4];

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn collect_table_order(root: &RenderNode, out: &mut Vec<usize>) {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) {
            if let Some(ci) = table.control_index {
                if TARGET_TABLES.contains(&ci) {
                    out.push(ci);
                }
            }
        }
    }
    for child in &root.children {
        collect_table_order(child, out);
    }
}

fn find_table_bbox(root: &RenderNode, target_ci: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) && table.control_index == Some(target_ci) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_table_bbox(child, target_ci) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_1639_empty_host_negative_offset_float_preserves_document_order() {
    let doc = load_doc(SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let mut order = Vec::new();
    collect_table_order(&tree.root, &mut order);

    assert_eq!(
        order, TARGET_TABLES,
        "co-anchored empty-host float tables with a negative vertical_offset must \
         retain document/control order, not be reordered by the offset sort",
    );
}

#[test]
fn issue_1639_negative_offset_table_does_not_jump_above_preceding_siblings() {
    let doc = load_doc(SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, _) = find_table_bbox(&tree.root, 2).expect("ci=2 (+offset) bbox");
    let (b_top, _) = find_table_bbox(&tree.root, 3).expect("ci=3 (-offset) bbox");
    let (c_top, _) = find_table_bbox(&tree.root, 4).expect("ci=4 (+offset) bbox");

    // 문서 순서(ci=2 → ci=3 → ci=4)대로 위에서 아래로 배치되어야 한다. 음수 offset 인
    // ci=3 이 선행 형제(ci=2) 위로 점프하면 페이지/배치 순서가 역전된다(#1639).
    assert!(
        a_top < b_top,
        "document-first ci=2 must render above the negative-offset ci=3: \
         a_top={a_top:.1}, b_top={b_top:.1}",
    );
    assert!(
        b_top < c_top,
        "ci=3 must render above ci=4 in document order: b_top={b_top:.1}, c_top={c_top:.1}",
    );
}

#[test]
fn issue_1639_positive_only_empty_host_keeps_offset_sort() {
    // 음수가 없는(양수 전용) 빈 host 는 기존 vertical_offset 오름차순 정렬(#986/#1088)을
    // 그대로 유지해야 한다 — 본 수정은 음수가 섞였을 때만 정렬을 끈다. 정렬이 양수까지
    // 무조건 꺼지는 회귀를 막는 가드. fixture: ci=2(+200) / ci=3(+50) / ci=4(+800) →
    // offset 오름차순 정렬 결과 [3, 2, 4](문서/배열 순서 [2,3,4] 와 다름).
    let doc = load_doc(POSITIVE_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let mut order = Vec::new();
    collect_table_order(&tree.root, &mut order);

    assert_eq!(
        order,
        vec![3, 2, 4],
        "positive-only empty-host float tables must remain vertical_offset-sorted (#986/#1088); \
         the negative-offset guard must not disable sorting when no negative offset is present",
    );
}
