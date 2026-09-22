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

fn find_table(node: &serde_json::Value, para_index: u64) -> Option<(f64, f64)> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Table")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
    {
        let bbox = node.get("bbox")?;
        return Some((bbox.get("y")?.as_f64()?, bbox.get("h")?.as_f64()?));
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|child| find_table(child, para_index))
}

fn rectangles(node: &serde_json::Value, out: &mut Vec<(f64, f64, f64, f64)>) {
    if node.get("type").and_then(|value| value.as_str()) == Some("Rect") {
        if let Some(bbox) = node.get("bbox") {
            if let (Some(x), Some(y), Some(width), Some(height)) = (
                bbox.get("x").and_then(|value| value.as_f64()),
                bbox.get("y").and_then(|value| value.as_f64()),
                bbox.get("w").and_then(|value| value.as_f64()),
                bbox.get("h").and_then(|value| value.as_f64()),
            ) {
                out.push((x, y, width, height));
            }
        }
    }
    if let Some(children) = node.get("children").and_then(|value| value.as_array()) {
        for child in children {
            rectangles(child, out);
        }
    }
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
    let (y, height) = find_table(&tree, PARA_INDEX).expect("40쪽 pi=523 스크린샷 표");

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

#[test]
fn auxiliary_cell_width_does_not_pull_following_table_over_screenshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    let json = document
        .get_page_render_tree(8)
        .unwrap_or_else(|error| panic!("9쪽 render tree: {error:?}"));
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let (screenshot_y, screenshot_height) = find_table(&tree, 155).expect("9쪽 pi=155 스크린샷 표");
    let (following_y, following_height) = find_table(&tree, 157).expect("9쪽 pi=157 후속 정보 표");

    assert!(
        following_height < 180.0,
        "pi=157 표 높이={following_height:.1}px — 행 보조폭으로 재줄바꿈해 커지면 안 된다"
    );
    assert!(
        following_y >= screenshot_y + screenshot_height + 20.0,
        "pi=157 표 top={following_y:.1}px, pi=155 bottom={:.1}px — 다음 표가 스크린샷과 겹쳤다",
        screenshot_y + screenshot_height
    );
}

#[test]
fn in_front_decoration_keeps_character_table_on_its_saved_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    let json = document
        .get_page_render_tree(13)
        .unwrap_or_else(|error| panic!("14쪽 render tree: {error:?}"));
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let (y, height) = find_table(&tree, 220).expect("14쪽 pi=220 스크린샷 표");

    assert!(
        (y - 346.8).abs() < 1.0,
        "pi=220 표 top={y:.1}px — InFrontOfText 장식 뒤 TAC 표의 저장 줄 위치를 유지해야 한다"
    );
    assert!(
        (height - 426.4).abs() < 1.0,
        "pi=220 표 높이={height:.1}px — 스크린샷 frame 높이를 유지해야 한다"
    );

    let mut rects = Vec::new();
    rectangles(&tree, &mut rects);
    let callout = rects
        .iter()
        .find(|(_, _, width, height)| (width - 22.9).abs() < 0.5 && (height - 21.3).abs() < 0.5)
        .expect("14쪽 번호 1 사각형 주석");
    assert!(
        (callout.1 - 353.9).abs() < 1.0,
        "pi=220 번호 1 주석 y={:.1}px — 주석은 첫 저장 줄에 남아야 한다",
        callout.1
    );
}
