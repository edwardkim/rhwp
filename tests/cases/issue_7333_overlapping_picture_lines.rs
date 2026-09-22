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

fn find_image_bbox(
    node: &serde_json::Value,
    para_index: u64,
    control_index: u64,
) -> Option<(f64, f64, f64, f64)> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Image")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
        && node.get("ci").and_then(|value| value.as_u64()) == Some(control_index)
    {
        let bbox = node.get("bbox")?;
        return Some((
            bbox.get("x")?.as_f64()?,
            bbox.get("y")?.as_f64()?,
            bbox.get("w")?.as_f64()?,
            bbox.get("h")?.as_f64()?,
        ));
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|child| find_image_bbox(child, para_index, control_index))
}

fn find_text_line_y(node: &serde_json::Value, para_index: u64) -> Option<f64> {
    if node.get("type").and_then(|value| value.as_str()) == Some("TextLine")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
    {
        return node.get("bbox")?.get("y")?.as_f64();
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|child| find_text_line_y(child, para_index))
}

fn find_table_cell_first_line_y(node: &serde_json::Value, para_index: u64) -> Option<f64> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Table")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
    {
        return node
            .get("children")?
            .as_array()?
            .iter()
            .find(|child| child.get("type").and_then(|value| value.as_str()) == Some("Cell"))?
            .get("children")?
            .as_array()?
            .iter()
            .find(|child| child.get("type").and_then(|value| value.as_str()) == Some("TextLine"))?
            .get("bbox")?
            .get("y")?
            .as_f64();
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|child| find_table_cell_first_line_y(child, para_index))
}

fn find_table_cell_image_bbox(
    node: &serde_json::Value,
    para_index: u64,
) -> Option<(f64, f64, f64, f64)> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Table")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
    {
        let cell = node
            .get("children")?
            .as_array()?
            .iter()
            .find(|child| child.get("type").and_then(|value| value.as_str()) == Some("Cell"))?;
        let image =
            cell.get("children")?.as_array()?.iter().find(|child| {
                child.get("type").and_then(|value| value.as_str()) == Some("Image")
            })?;
        let bbox = image.get("bbox")?;
        return Some((
            bbox.get("x")?.as_f64()?,
            bbox.get("y")?.as_f64()?,
            bbox.get("w")?.as_f64()?,
            bbox.get("h")?.as_f64()?,
        ));
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|child| find_table_cell_image_bbox(child, para_index))
}

fn find_footer_logo_frame_y(node: &serde_json::Value) -> Option<f64> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Footer") {
        return node
            .get("children")
            .and_then(|value| value.as_array())?
            .iter()
            .find_map(find_footer_logo_frame_y);
    }
    if node.get("type").and_then(|value| value.as_str()) == Some("Image") {
        let bbox = node.get("bbox")?;
        let width = bbox.get("w")?.as_f64()?;
        let height = bbox.get("h")?.as_f64()?;
        if (165.0..171.0).contains(&width) && (46.0..51.0).contains(&height) {
            return bbox.get("y")?.as_f64();
        }
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(find_footer_logo_frame_y)
}

fn find_footer_rule_y(node: &serde_json::Value) -> Option<f64> {
    if node.get("type").and_then(|value| value.as_str()) == Some("Footer") {
        return node
            .get("children")
            .and_then(|value| value.as_array())?
            .iter()
            .find_map(find_footer_rule_y);
    }
    if node.get("type").and_then(|value| value.as_str()) == Some("Image") {
        let bbox = node.get("bbox")?;
        let width = bbox.get("w")?.as_f64()?;
        let height = bbox.get("h")?.as_f64()?;
        if (620.0..640.0).contains(&width) && height < 6.0 {
            return bbox.get("y")?.as_f64();
        }
    }
    node.get("children")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(find_footer_rule_y)
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

#[test]
fn fixed_line_spacing_after_in_front_decoration_table_is_not_reserved_twice() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    // 한컴 2020 PDF에서 측정한 표 뒤 첫 본문 줄: p13, p16~21.
    for (page_index, para_index, expected_y) in [
        (12, 209, 853.5),
        (15, 253, 835.4),
        (16, 265, 814.5),
        (17, 277, 816.8),
        (18, 288, 662.5),
        (19, 301, 685.2),
        (20, 315, 729.0),
    ] {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let y = find_text_line_y(&tree, para_index)
            .unwrap_or_else(|| panic!("{}쪽 pi={para_index} 표 뒤 첫 본문 줄", page_index + 1));
        assert!(
            (y - expected_y).abs() < 1.0,
            "{}쪽 pi={para_index} 본문 y={y:.1}px — 음수 저장 줄간격을 표 예약에 중복 계상하면 안 된다",
            page_index + 1
        );
    }
}

#[test]
fn bottom_aligned_mixed_footer_uses_the_saved_grid_leading() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    for page_index in [1, 30] {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let y = find_footer_rule_y(&tree)
            .unwrap_or_else(|| panic!("{}쪽 footer 파란 rule", page_index + 1));
        assert!(
            (y - 1002.7).abs() < 1.0,
            "{}쪽 footer rule y={y:.1}px — 한컴 2020 PDF의 y=1003px 기준과 맞아야 한다",
            page_index + 1
        );
        let logo_frame_y = find_footer_logo_frame_y(&tree)
            .unwrap_or_else(|| panic!("{}쪽 footer 로고 frame", page_index + 1));
        assert!(
            (logo_frame_y - 1027.1).abs() < 1.0,
            "{}쪽 footer 로고 frame y={logo_frame_y:.1}px — Paper 기준 그림도 HWP5 저장 grid의 452HU leading을 반영해야 한다",
            page_index + 1
        );
    }
}

fn svg_numeric_attr(tag: &str, name: &str) -> Option<f64> {
    let (_, value) = tag.split_once(&format!("{name}=\""))?;
    value.split_once('"')?.0.parse().ok()
}

#[test]
fn page_31_signed_line_transform_preserves_arrow_direction() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");
    let svg = document
        .render_page_svg_native(30)
        .expect("31쪽 SVG 렌더링");

    // 31쪽 둘째 빨간 화살표는 HWP5 renderingInfo의 x=-0.896, tx=7508 변환을
    // 쓴다. 한컴 2020 PDF에서는 오른쪽 위에서 왼쪽 아래로 진행한다. 절댓값
    // scale만 적용하면 방향이 반대로 된다.
    let (x1, y1, x2, y2) = svg
        .lines()
        .filter(|tag| {
            tag.contains("<line")
                && tag.contains("stroke=\"#ff0000\"")
                && tag.contains("marker-end")
        })
        .filter_map(|tag| {
            Some((
                svg_numeric_attr(tag, "x1")?,
                svg_numeric_attr(tag, "y1")?,
                svg_numeric_attr(tag, "x2")?,
                svg_numeric_attr(tag, "y2")?,
            ))
        })
        .find(|(_, y1, _, y2)| y2 - y1 > 150.0)
        .expect("31쪽의 긴 빨간 대각선 화살표");

    assert!(
        x1 > x2 && y1 < y2,
        "31쪽 화살표 방향=({x1:.1}, {y1:.1})→({x2:.1}, {y2:.1}) — PDF처럼 오른쪽 위에서 왼쪽 아래여야 한다"
    );
    assert!(
        (x1 - 542.0).abs() < 2.0
            && (y1 - 484.2).abs() < 2.0
            && (x2 - 445.9).abs() < 2.0
            && (y2 - 701.2).abs() < 2.0,
        "31쪽 화살표 끝점=({x1:.1}, {y1:.1})→({x2:.1}, {y2:.1}) — 저장 renderingInfo frame을 유지해야 한다"
    );
}

#[test]
fn page_31_inline_screenshot_keeps_its_saved_frame_before_cell_clip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");
    let json = document.get_page_render_tree(30).expect("31쪽 render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let (_, image_y, width, height) =
        find_table_cell_image_bbox(&tree, 430).expect("31쪽 p430 표 셀의 스크린샷 그림");
    let line_y = find_table_cell_first_line_y(&tree, 430).expect("31쪽 p430 빈 셀의 저장 줄 위쪽");
    let picture = match &document.document().sections[0].paragraphs[430].controls[5] {
        rhwp::model::control::Control::Table(table) => {
            match &table.cells[0].paragraphs[0].controls[0] {
                rhwp::model::control::Control::Picture(picture) => picture,
                other => panic!("p430 table picture expected, got {other:?}"),
            }
        }
        other => panic!("p430 table expected, got {other:?}"),
    };
    assert_eq!(
        (picture.common.width, picture.common.height),
        (48190, 38370),
        "31쪽 스크린샷의 HWP5 저장 frame"
    );

    // HWP5 저장 frame=48190×38370 HU. TableCell은 화면 끝에서 clip하지만,
    // 그림 자체를 cell 안쪽 폭 45884 HU로 축소해서는 안 된다.
    assert!(
        (width - 642.5).abs() < 1.0 && (height - 511.6).abs() < 1.0,
        "31쪽 스크린샷 frame={width:.1}×{height:.1}px — 저장 크기를 유지한 뒤 셀 경계에서 clip해야 한다"
    );
    assert!(
        (image_y - line_y).abs() < 1.0,
        "31쪽 스크린샷 y={image_y:.1}px, 셀 저장 줄 y={line_y:.1}px — leading을 다시 더하면 주석 도형과 어긋난다"
    );
}
