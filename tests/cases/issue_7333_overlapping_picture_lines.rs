//! Issue #7333: 그림만 담은 1×1 셀의 겹친 저장 LINE_SEG를 각각 새 줄로 합산하면
//! 스크린샷 표가 약 두 배로 늘어나 본문·꼬리말을 가리던 회귀를 막는다.
//!
//! `samples/issue7333/aaaaaa.hwp` 40쪽(`pi=523`)은 한컴 2020 PDF에서 616.5px
//! 표 상자 안에 온전히 놓인다. 결함 시 rhwp는 이 표를 1194.6px로 측정했다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;

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

fn image_bboxes_for_para(
    node: &serde_json::Value,
    para_index: u64,
    out: &mut Vec<(f64, f64, f64, f64)>,
) {
    if node.get("type").and_then(|value| value.as_str()) == Some("Image")
        && node.get("pi").and_then(|value| value.as_u64()) == Some(para_index)
    {
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
            image_bboxes_for_para(child, para_index, out);
        }
    }
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
        (y - 350.6).abs() < 1.0,
        "pi=220 표 top={y:.1}px — 전체 대역 TAC 표는 저장 줄 위 여백 뒤에 놓여야 한다"
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
fn full_band_empty_tac_tables_use_their_saved_line_top() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    // 한컴 2020 PDF의 바깥 표선 상단. 세 표 모두 두 번째 LINE_SEG의 text_height가
    // 표 높이와 outer margin 대역을 함께 저장한 빈 문단이다.
    for (page_index, para_index, expected_y) in
        [(13, 220, 350.6), (21, 329, 327.2), (22, 341, 295.2)]
    {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let (y, _) = find_table(&tree, para_index)
            .unwrap_or_else(|| panic!("{}쪽 pi={para_index} 전체 대역 TAC 표", page_index + 1));
        assert!(
            (y - expected_y).abs() < 1.0,
            "{}쪽 pi={para_index} 표 top={y:.1}px — 한컴 2020 PDF의 저장 줄 기준 y={expected_y:.1}px와 맞아야 한다",
            page_index + 1
        );
    }
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
fn full_band_tac_table_consumes_the_saved_following_line_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    // PDF에서 측정한 표 뒤 첫 가시 본문 줄. 네 full-band carrier의 다음 저장
    // vpos는 각각 `owner.vpos + owner.text_height + owner.line_spacing`와 같다.
    // Stage 8은 표 top만 복원해 이 줄들을 8.8~10.4px 위로 남겼다.
    for (page_index, para_index, expected_y) in [
        (13, 222, 815.5),
        (21, 331, 804.9),
        (22, 344, 826.0),
        (32, 454, 802.9),
    ] {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let y = find_text_line_y(&tree, para_index)
            .unwrap_or_else(|| panic!("{}쪽 pi={para_index} 표 뒤 첫 본문 줄", page_index + 1));
        assert!(
            (y - expected_y).abs() < 1.5,
            "{}쪽 pi={para_index} 본문 y={y:.1}px — full-band TAC의 저장 후행 간격을 한 번 소비해야 한다 (PDF {expected_y:.1}px)",
            page_index + 1
        );
    }
}

#[test]
fn cell_screenshot_uses_its_own_saved_line_after_callout_shapes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {SAMPLE}: {error}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("parse {SAMPLE}: {error}"));

    // 각 셀은 InFrontOfText 주석 도형 뒤에 그림을 둔다. 그림의 empty-control
    // stream position은 두 번째 저장 LINE_SEG(vpos=1600HU)를 가리킨다. 한컴 2020
    // PDF의 스크린샷 상단을 96dpi로 측정한 값이다.
    for (page_index, para_index, control_index, expected_y) in [
        (39, 523, 4, 318.6),
        (40, 532, 7, 369.8),
        (41, 539, 6, 383.1),
        (42, 549, 7, 395.4),
        (43, 557, 6, 408.7),
        (46, 586, 6, 408.7),
    ] {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let (_, y, _, _) = find_image_bbox(&tree, para_index, control_index).unwrap_or_else(|| {
            panic!(
                "{}쪽 pi={para_index} ci={control_index} 셀 스크린샷",
                page_index + 1
            )
        });
        assert!(
            (y - expected_y).abs() < 1.0,
            "{}쪽 pi={para_index} ci={control_index} 그림 y={y:.1}px — 두 번째 저장 줄과 PDF {expected_y:.1}px를 써야 한다",
            page_index + 1
        );
    }
}

#[test]
fn cell_callout_with_stale_negative_offset_stays_on_its_picture_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");

    // 27쪽 pi=383의 첫 번호 도형은 HWP5에 -1275HU offset으로 저장됐지만, 뒤의
    // inline screenshot이 지정하는 셀 줄에 붙어야 한다. 한컴 2020 PDF의 빨간
    // 번호 사각형 top은 96dpi raster에서 약 y=350px이다.
    let json = document.get_page_render_tree(26).expect("27쪽 render tree");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
    let mut rects = Vec::new();
    rectangles(&tree, &mut rects);
    let callout = rects
        .iter()
        .find(|(x, _, width, height)| {
            (x - 260.4).abs() < 1.0 && (width - 26.7).abs() < 1.0 && (height - 25.0).abs() < 1.0
        })
        .expect("27쪽 첫 번호 주석 사각형");
    assert!(
        (callout.1 - 350.8).abs() < 1.0,
        "27쪽 번호 1 주석 y={:.1}px — 뒤 inline 그림의 저장 줄(PDF y≈350.8px)에 붙어야 한다",
        callout.1
    );
}

#[test]
fn stored_multi_picture_rows_keep_their_middle_row_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");

    // p37/pi=495와 p38/pi=503의 세 번째 셀은 빈 문단에 TopAndBottom flow 그림 두 장을
    // 저장한다. 이를 가로 band의 max로만 재면 해당 행이 약 12px 줄고 마지막 행에
    // 높이가 몰린다. 한컴 2020 PDF에서 다음 아이콘들은 각각 y=363.5/376.5px이다.
    for (page_index, para_index, expected_x, expected_y, expected_width) in
        [(36, 495, 100.3, 363.5, 28.8), (37, 503, 82.4, 376.5, 35.2)]
    {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 render tree: {error:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");
        let mut images = Vec::new();
        image_bboxes_for_para(&tree, para_index, &mut images);
        let (_, y, _, _) = images
            .iter()
            .find(|(x, _, width, _)| {
                (x - expected_x).abs() < 0.2 && (width - expected_width).abs() < 0.2
            })
            .unwrap_or_else(|| {
                panic!(
                    "{}쪽 pi={para_index} 중간 행 아이콘 x={expected_x:.1} w={expected_width:.1}",
                    page_index + 1
                )
            });
        assert!(
            (y - expected_y).abs() < 1.0,
            "{}쪽 pi={para_index} 중간 행 아이콘 y={y:.1}px — 한컴 2020 PDF {expected_y:.1}px와 맞아야 한다",
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
fn connector_paths_preserve_saved_direction_and_endpoint_marker() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");

    // 19쪽의 빨간 연결선은 HWP5 renderingInfo에 y 음수 축척이 있고 끝 화살표를
    // 가진다. path로 그리는 연결선도 일반 LineNode처럼 그 축척과 끝 marker를
    // 보존해야 한다. 이 경우 한컴 2020 PDF는 왼쪽 위→오른쪽 아래 방향이다.
    let Control::Shape(shape) = &document.document().sections[0].paragraphs[286].controls[0] else {
        panic!("19쪽 주석 연결선 control");
    };
    let ShapeObject::Line(line) = shape.as_ref() else {
        panic!("19쪽 주석은 직선");
    };
    assert!(
        line.connector
            .as_ref()
            .is_some_and(|connector| !connector.control_points.is_empty()),
        "19쪽 주석은 제어점을 가진 연결선이어야 한다"
    );
    assert_eq!(
        (line.drawing.border_line.attr >> 16) & 0x3f,
        1,
        "19쪽 연결선의 끝은 HWP Arrow여야 한다"
    );

    let svg = document
        .render_page_svg_native(18)
        .expect("19쪽 SVG 렌더링");
    let marker_line = svg
        .lines()
        .filter(|tag| tag.contains("<line") && tag.contains("stroke=\"none\""))
        .find(|tag| tag.contains("marker-end"))
        .expect("19쪽 연결선의 SVG 끝 marker");
    let (x1, y1, x2, y2) = (
        svg_numeric_attr(marker_line, "x1").expect("x1"),
        svg_numeric_attr(marker_line, "y1").expect("y1"),
        svg_numeric_attr(marker_line, "x2").expect("x2"),
        svg_numeric_attr(marker_line, "y2").expect("y2"),
    );
    assert!(
        x2 > x1 && y2 > y1,
        "19쪽 연결선 방향=({x1:.1}, {y1:.1})→({x2:.1}, {y2:.1}) — PDF처럼 오른쪽 아래 끝에 arrow가 있어야 한다"
    );
}

#[test]
fn screenshot_table_direction_lines_keep_their_arrow_markers() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read fixture");
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse fixture");

    // 41·43쪽의 스크린샷 표에는 직선처럼 보이는 방향 연결선이 있다. 파서는 이를
    // 제어점 연결선으로 보존하므로 SVG도 경로 본문만 그리지 말고 marker를 내야 한다.
    for (page_index, table_para_index) in [(40, 532), (42, 549)] {
        let Control::Table(table) =
            &document.document().sections[0].paragraphs[table_para_index].controls[0]
        else {
            panic!("{}쪽 스크린샷 표", page_index + 1);
        };
        let has_connector = table.cells[0].paragraphs[0].controls.iter().any(|control| {
            matches!(control, Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Line(line) if line.connector.is_some()))
        });
        assert!(has_connector, "{}쪽 방향 연결선 파싱", page_index + 1);
        let svg = document
            .render_page_svg_native(page_index)
            .unwrap_or_else(|error| panic!("{}쪽 SVG: {error:?}", page_index + 1));
        assert!(
            svg.lines().any(|tag| tag.contains("<line")
                && tag.contains("stroke=\"none\"")
                && tag.contains("marker-")),
            "{}쪽 연결선 SVG marker",
            page_index + 1
        );
    }
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
