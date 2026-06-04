//! Issue #1196: HWPX `pagePr gutterType="LEFT_RIGHT"` 맞쪽 편집 문서에서
//! 최종 쪽번호 홀짝에 따라 좌우 여백이 교대되어야 한다.
//!
//! 재현 문서: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`.
//! #1271 이후 page 4 는 section 1 본문 시작이며, 정답 PDF 기준 짝수쪽인 page 4/6
//! 본문은 홀수쪽 page 5 보다 오른쪽에서 시작해야 한다.

use std::fs;
use std::path::Path;

#[derive(Debug)]
struct BodyArea {
    x: f64,
    width: f64,
}

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let rel = "samples/hwpx/[2027] 온새미로 1 본교재.hwpx";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

fn parse_body_area(dump: &str) -> BodyArea {
    let line = dump
        .lines()
        .find(|line| line.trim_start().starts_with("body_area:"))
        .unwrap_or_else(|| panic!("body_area line not found:\n{dump}"));

    let mut x = None;
    let mut width = None;
    for token in line.split_whitespace() {
        if let Some(value) = token.strip_prefix("x=") {
            x = Some(
                value
                    .parse::<f64>()
                    .unwrap_or_else(|e| panic!("parse body_area x from {line:?}: {e}")),
            );
        } else if let Some(value) = token.strip_prefix("w=") {
            width = Some(
                value
                    .parse::<f64>()
                    .unwrap_or_else(|e| panic!("parse body_area width from {line:?}: {e}")),
            );
        }
    }

    BodyArea {
        x: x.unwrap_or_else(|| panic!("body_area x not found: {line}")),
        width: width.unwrap_or_else(|| panic!("body_area width not found: {line}")),
    }
}

#[test]
fn onsaemiro_left_right_gutter_alternates_body_area_by_page_parity() {
    let doc = load_doc();

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("section=1, page_num=4"),
        "page 4 must remain section 1 body start after #1271:\n{page4}"
    );
    assert!(
        page4.contains("\"강의 01.\""),
        "page 4 should start the main body content:\n{page4}"
    );

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        page5.contains("section=1, page_num=5"),
        "page 5 should keep final page_num=5:\n{page5}"
    );

    let page6 = doc.dump_page_items(Some(5));
    assert!(
        page6.contains("section=1, page_num=6"),
        "page 6 should keep final page_num=6:\n{page6}"
    );

    let page4_body = parse_body_area(&page4);
    let page5_body = parse_body_area(&page5);
    let page6_body = parse_body_area(&page6);

    assert!(
        page4_body.x > page5_body.x + 40.0,
        "Duplex even page 4 should start to the right of odd page 5. \
         page4={page4_body:?}, page5={page5_body:?}"
    );
    assert!(
        page6_body.x > page5_body.x + 40.0,
        "Duplex even page 6 should start to the right of odd page 5. \
         page6={page6_body:?}, page5={page5_body:?}"
    );
    assert!(
        (page4_body.x - page6_body.x).abs() < 0.1,
        "even pages should share the same body x. page4={page4_body:?}, page6={page6_body:?}"
    );
    assert!(
        (page4_body.width - page5_body.width).abs() < 0.1
            && (page6_body.width - page5_body.width).abs() < 0.1,
        "left/right gutter swap should preserve body width. \
         page4={page4_body:?}, page5={page5_body:?}, page6={page6_body:?}"
    );
}
