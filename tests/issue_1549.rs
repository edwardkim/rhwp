//! Issue #1549: visible host 문단(텍스트=섹션 제목)에 양수 offset co-anchored
//! TopAndBottom float 표가 여러 개 있을 때, host 제목이 표 *아래*로 밀려 렌더되던
//! 회귀를 막는다. 한컴은 제목을 문단 앵커(표 위)에 두고 양수 offset 표를 그 아래에 둔다.
//!
//! fixture: samples/issue1549_multipositive_float_tables.hwpx
//!   = issue1510 구조에서 float 표 offset 을 모두 작은 양수로 narrow
//!     (A=+200, B=+500, C=+800 HWPUNIT) — 표 top 이 제목 라인과 겹치는 multi-positive 케이스.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const HWPX_SAMPLE: &str = "samples/issue1549_multipositive_float_tables.hwpx";
const EMPTY_HOST_SAMPLE: &str = "samples/issue1549_empty_host_float_clamp.hwpx";
const TARGET_PI: usize = 0;
const TITLE_NEEDLE: &str = "MULTI POSITIVE TITLE";
const TARGET_TABLES: [usize; 3] = [2, 3, 4];

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
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

fn find_table_bbox_by_para(root: &RenderNode, para_index: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(para_index) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_table_bbox_by_para(child, para_index) {
            return Some(found);
        }
    }
    None
}

fn find_title_bbox(root: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.para_index.is_some() && run.text.contains(needle) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_title_bbox(child, needle) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_1549_multi_positive_float_host_title_renders_above_tables() {
    let doc = load_doc(HWPX_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (title_top, title_bottom) =
        find_title_bbox(&tree.root, TITLE_NEEDLE).expect("host title text bbox");
    let table_tops: Vec<f64> = TARGET_TABLES
        .iter()
        .map(|&ci| {
            find_table_bbox(&tree.root, ci)
                .unwrap_or_else(|| panic!("table ci={ci} bbox"))
                .0
        })
        .collect();
    let first_table_top = table_tops.iter().cloned().fold(f64::INFINITY, f64::min);

    assert!(
        title_top < first_table_top + 0.5,
        "host title must render at the paragraph anchor, above its co-anchored \
         positive-offset float tables (Hancom places the title above the tables): \
         title_top={title_top:.1}, table_tops={table_tops:?}",
    );
    // 한컴은 제목을 자기 줄에 두고 표를 그 줄 *아래*에 둔다. 표가 제목 줄을 침범하면
    // 안 된다(작은 양수 offset 에서 제목·표가 같은 y 로 밀려 겹치던 회귀 가드).
    assert!(
        first_table_top + 0.5 >= title_bottom,
        "co-anchored float table must clear the host title line, not overlap it: \
         title=({title_top:.1}..{title_bottom:.1}), first_table_top={first_table_top:.1}",
    );
}

/// 제목을 앵커로 되돌리면(위 테스트), 옛 버그가 우연히 제공하던 flow advance 가
/// 사라져 뒤따르는 *빈-host(text 없는)* para float 표가 선행 float 점유밴드 위로
/// 올라와 겹칠 수 있다(작업일지류 실문서 회귀). 빈-host float 도 선행 exclusion
/// 밴드 아래로 클램프되는지 가드한다.
///
/// fixture: 문단0 = visible host 제목 + 양수 offset float 표 A,
///          문단1 = 빈 host(text 0) + 양수 offset float 표 C (A 밴드와 겹치는 자연위치).
#[test]
fn issue_1549_empty_host_following_float_clears_preceding_band() {
    let doc = load_doc(EMPTY_HOST_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, a_bottom) =
        find_table_bbox_by_para(&tree.root, 0).expect("para0 visible-host float table A");
    let (c_top, _) =
        find_table_bbox_by_para(&tree.root, 1).expect("para1 empty-host float table C");

    assert!(
        c_top + 0.5 >= a_bottom,
        "empty-host following float table must clear the preceding float band, \
         not overlap it: a=({a_top:.1}..{a_bottom:.1}), c_top={c_top:.1}",
    );
}
