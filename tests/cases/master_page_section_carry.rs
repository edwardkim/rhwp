#![cfg(not(target_arch = "wasm32"))]
//! 바탕쪽이 없는 구역은 앞 구역의 바탕쪽을 잇는다. 한/글은 구역을 나눈 뒤 바탕쪽을 따로
//! 만들지 않은 구역에도 앞 구역의 바탕쪽을 그린다. 구역 사이의 carry 는 있었지만
//! `assign_master_pages_for_section` 이 바탕쪽 없는 구역에서 carry 를 보기 전에 끝났다.
//!
//! 표본은 A4 두 구역 문서다(한글 2024 저장, 글은 채움 글자). 첫 구역에만 바탕쪽(용지
//! 기준 아래 정렬 1칸 표)이 있고 둘째 구역은 `masterPageCnt="0"` 이다. 한글 2024 Print
//! PDF 는 두 쪽 모두 같은 자리에 그 표를 그린다. 고치기 전 rhwp 는 둘째 쪽에 그리지 않았다.
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/master-page-section-carry.hwpx";

fn master_page_table(tree: &Value) -> Option<(f64, f64)> {
    fn table(node: &Value) -> Option<&Value> {
        if node["type"] == "Table" {
            return Some(node);
        }
        node["children"]
            .as_array()
            .into_iter()
            .flatten()
            .find_map(table)
    }
    let master = tree["children"]
        .as_array()?
        .iter()
        .find(|child| child["type"] == "MasterPage")?;
    let b = &table(master)?["bbox"];
    Some((b["x"].as_f64()?, b["y"].as_f64()?))
}

#[test]
fn section_without_master_pages_draws_previous_section_master_page() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(doc.page_count(), 2, "독립 한글 출력과 같은 쪽 수");

    // 표본의 전제: 첫 구역에만 바탕쪽이 있다.
    let sections = &doc.document().sections;
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].section_def.master_pages.len(), 1);
    assert!(sections[1].section_def.master_pages.is_empty());

    let bands: Vec<_> = (0..2)
        .map(|page| {
            let tree: Value =
                serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
            master_page_table(&tree)
        })
        .collect();
    let first = bands[0].expect("첫 구역 쪽의 바탕쪽 표");
    let second = bands[1].expect("바탕쪽 없는 둘째 구역 쪽도 앞 구역 바탕쪽 표를 그린다");
    assert!(
        (first.1 - second.1).abs() < 0.5,
        "두 쪽의 바탕쪽 표 위끝 {:.1}, {:.1} 이 같다",
        first.1,
        second.1
    );
}
