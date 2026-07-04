//! Issue #1535: PR #1518 후속. visible host 문단에 co-anchored para-relative
//! TopAndBottom float 표가 여러 개 있을 때, 선행 float 표가 점유한 세로 영역을
//! 후행 float 표가 무시하고 그 위에 겹쳐 배치되던 회귀를 막는다.
//!
//! 재현 fixture(`issue1535_coanchored_float_exclusion.hwpx`)는 같은 host 문단에
//! 양수 vertical_offset 표 A(offset 16996) 와 B(offset 18000)를 둔다. B 의 선언
//! 위치(host + offset)는 A 가 차지한 영역 안에서 시작하므로, A 아래로 밀려
//! 내려가야 한다(겹침 금지). 수정 전에는 B 가 A 영역 안(y≈376, A=[362,437])에
//! 그려져 텍스트가 겹쳤다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/issue1535_coanchored_float_exclusion.hwpx";
const TARGET_PI: usize = 0;
const TABLE_A: usize = 2;
const TABLE_B: usize = 3;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
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
fn issue_1535_later_visible_float_table_does_not_overlap_earlier_float_zone() {
    let doc = load_doc(SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, a_bottom) = find_table_bbox(&tree.root, TABLE_A).expect("A table bbox");
    let (b_top, _) = find_table_bbox(&tree.root, TABLE_B).expect("B table bbox");

    assert!(
        a_bottom > a_top,
        "table A should have positive height: a_top={a_top:.1}, a_bottom={a_bottom:.1}",
    );
    assert!(
        b_top + 0.5 >= a_bottom,
        "co-anchored float table B (offset inside A's occupied zone) must be pushed below A, \
         not overlapped onto it: a=[{a_top:.1},{a_bottom:.1}], b_top={b_top:.1}",
    );
}
