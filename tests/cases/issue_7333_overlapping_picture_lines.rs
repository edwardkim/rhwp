//! Issue #7333: 그림만 담은 1×1 셀의 겹친 저장 LINE_SEG를 각각 새 줄로 합산하면
//! 스크린샷 표가 약 두 배로 늘어나 본문·꼬리말을 가리던 회귀를 막는다.
//!
//! `samples/issue7333/aaaaaa.hwp` 40쪽(`pi=523`)은 한컴 2020 PDF에서 616.5px
//! 표 상자 안에 온전히 놓인다. 결함 시 rhwp는 이 표를 1194.6px로 측정했다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue7333/aaaaaa.hwp";
const PAGE_INDEX: u32 = 39;
const PARA_INDEX: u64 = 523;

fn find_table(node: &serde_json::Value) -> Option<(f64, f64)> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Table")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(PARA_INDEX)
    {
        let bbox = node.get("bbox")?;
        return Some((bbox.get("y")?.as_f64()?, bbox.get("h")?.as_f64()?));
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(find_table)
}

#[test]
fn overlapping_picture_lines_occupy_one_declared_table_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    assert_eq!(document.page_count(), 50, "fixture page count");
    let json = document
        .get_page_render_tree(PAGE_INDEX)
        .unwrap_or_else(|error| panic!("40쪽 render tree: {error:?}"));
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let (y, height) = find_table(&tree).expect("40쪽 pi=523 스크린샷 표");

    assert!(
        (height - 616.5).abs() < 1.0,
        "pi=523 표 높이={height:.1}px — 한컴 2020 PDF의 선언 frame 616.5px이어야 한다"
    );
    assert!(
        y + height < 920.0,
        "pi=523 표 bottom={:.1}px — 본문·꼬리말 사이에 들어가야 한다",
        y + height
    );
}
