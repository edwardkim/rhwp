//! 독립 한컴 Print로 확인한 셀 그림의 쪽/표 그리기 소유와 편집 주소.
//! Native257/fresh074의 전쪽 raw100, PDF CTM·같은 좌표 확대 판독이 선행했다.
//! tests/fixtures/pr8061_cell_picture_layer/expected.json과 정식 증거 README를 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUTS: &[(&str, &[u8])] = &[
    (
        "explicit-geometry",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/explicit-geometry.hwpx"),
    ),
    (
        "no-paper",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/no-paper.hwpx"),
    ),
    (
        "picture-z2",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/picture-z2.hwpx"),
    ),
    (
        "picture-follow-cell",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/picture-follow-cell.hwpx"),
    ),
    (
        "floating-page-order",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/floating-page-order.hwpx"),
    ),
    (
        "floating-picture-z2",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/floating-picture-z2.hwpx"),
    ),
    (
        "floating-follow-cell",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/floating-follow-cell.hwpx"),
    ),
    (
        "bordered-page-picture",
        include_bytes!("../fixtures/pr8061_cell_picture_layer/bordered-page-picture.hwpx"),
    ),
];

fn load(key: &str) -> (DocumentCore, Value) {
    let expected: Value = serde_json::from_str(include_str!(
        "../fixtures/pr8061_cell_picture_layer/expected.json"
    ))
    .unwrap();
    assert_eq!(expected["dpi"], 96);
    assert_eq!(expected["compat"], "2022");
    assert_eq!(expected["profile"], "print");
    let record = expected["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == key)
        .unwrap()
        .clone();
    assert_eq!(record["native_raw"], "100.00000");
    assert_eq!(record["fresh_wasm_raw"], "100.00000");
    let bytes = INPUTS.iter().find(|(name, _)| *name == key).unwrap().1;
    let actual: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        actual,
        record["input_sha256"].as_str().unwrap(),
        "독립 Print와 같은 입력"
    );
    let pdf = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(record["pdf"].as_str().unwrap()),
    )
    .unwrap();
    let actual: String = Sha256::digest(pdf)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        actual,
        record["pdf_sha256"].as_str().unwrap(),
        "기대 기하의 독립 Print"
    );
    let mut core = DocumentCore::from_bytes(bytes).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 1);
    (core, record)
}

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

fn assert_case(key: &str) {
    let (core, expected) = load(key);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let images: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .collect();
    assert_eq!(images.len(), 1, "{key}: bitmap 누락·중복 없음");
    let image = images[0];
    let reference = expected["image_ctm_bbox_px"].as_array().unwrap();
    for ((name, actual), value) in ["x", "y", "width", "height"]
        .into_iter()
        .zip([
            image.bbox.x,
            image.bbox.y,
            image.bbox.width,
            image.bbox.height,
        ])
        .zip(reference)
    {
        let independent = value.as_f64().unwrap();
        assert!(
            (actual - independent).abs() <= 2.0,
            "{key} {name}: actual={actual}, independent Print={independent}"
        );
    }
    let cell = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::TableCell(_)))
        .unwrap();
    // 동일한 원본 좌표도 중심/너비 연산 뒤에는 약 1e-14px 달라질 수 있다.
    // 관계는 원본의 정수 HWPUNIT 격자에서, 독립 Print 좌표는 위 ±2px로 검사한다.
    let source_unit = |px: f64| (px * 7200.0 / 96.0).round() as i64;
    assert_eq!(
        (source_unit(image.bbox.x), source_unit(image.bbox.y)),
        (source_unit(cell.bbox.x), source_unit(cell.bbox.y)),
        "{key}: 원본의 0 offset 그림이 셀에서 시작한다: image={:?}, cell={:?}",
        image.bbox,
        cell.bbox
    );
    assert!(
        source_unit(image.bbox.x + image.bbox.width) <= source_unit(cell.bbox.x + cell.bbox.width)
            && source_unit(image.bbox.y + image.bbox.height)
                <= source_unit(cell.bbox.y + cell.bbox.height),
        "{key}: 그림은 선언된 셀 안에 포함된다"
    );
    let text: String = nodes
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                Some(run.text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(text.trim(), "PHOTO", "{key}: 셀 본문 보존");
    let source = &core.document().sections[0].paragraphs[0];
    let table_index = source
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    let layout: Value =
        serde_json::from_str(&core.get_page_control_layout_native(0).unwrap()).unwrap();
    let records: Vec<_> = layout["controls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["type"] == "image")
        .collect();
    assert_eq!(records.len(), 1, "{key}: 공개 조회도 같은 그림 하나");
    assert_eq!(records[0]["parentParaIdx"], 0);
    assert_eq!(
        records[0]["cellPath"],
        serde_json::json!([{"controlIndex":table_index,"cellIndex":0,"cellParaIndex":0}]),
        "{key}: 쪽 그리기로 옮겨도 셀의 편집 주소 유지"
    );
    let svg = core.render_page_svg_layer_native(0).unwrap();
    let bitmap = svg.find("<image").unwrap();
    let photo = svg
        .find(">P<")
        .or_else(|| svg.find(">PHOTO<"))
        .expect("셀 본문 표시");
    assert!(bitmap < photo, "{key}: 그림은 셀 본문 아래");
    if expected["has_paper_shape"].as_bool().unwrap() {
        let paper = svg.find("fill=\"#00ff00\"").unwrap();
        if expected["picture_below_paper"].as_bool().unwrap() {
            assert!(
                bitmap < paper,
                "{key}: 독립 Print처럼 쪽 도형이 그림을 가린다"
            );
        } else {
            assert!(
                paper < bitmap,
                "{key}: 독립 Print처럼 그림이 쪽 도형 위에 있다"
            );
        }
    } else {
        assert!(
            !svg.contains("fill=\"#00ff00\""),
            "{key}: 대조군에 쪽 도형을 추가하지 않는다"
        );
    }
    if expected["has_blue_border"].as_bool().unwrap() {
        let border = svg.find("stroke=\"#0000ff\"").expect("그림의 테두리");
        assert!(bitmap < border, "같은 그림의 bitmap·테두리 내부 순서 보존");
    }
}

#[test]
fn verified_cell_picture_page_shape_occludes_independent_picture() {
    assert_case("explicit-geometry");
}
#[test]
fn verified_cell_picture_without_occluder_remains_visible() {
    assert_case("no-paper");
}
#[test]
fn verified_cell_picture_higher_page_z_order_remains_above_shape() {
    assert_case("picture-z2");
}
#[test]
fn verified_cell_picture_following_text_inherits_cell_paint() {
    assert_case("picture-follow-cell");
}
#[test]
fn verified_cell_picture_floating_owner_preserves_page_order() {
    assert_case("floating-page-order");
}
#[test]
fn verified_cell_picture_floating_higher_z_preserves_page_order() {
    assert_case("floating-picture-z2");
}
#[test]
fn verified_cell_picture_floating_text_flow_preserves_owner_plane() {
    assert_case("floating-follow-cell");
}
#[test]
fn verified_cell_picture_bitmap_precedes_its_own_border() {
    assert_case("bordered-page-picture");
}

#[cfg(feature = "native-skia")]
#[test]
fn verified_cell_picture_print_pixels_match_independent_occlusion() {
    use rhwp::document_core::queries::rendering::PngExportOptions;
    use rhwp::paint::RenderProfile;
    for (key, _) in INPUTS {
        let (core, expected) = load(key);
        let bytes = core
            .render_page_png_native_with_profile_and_export_options(
                0,
                RenderProfile::Print,
                &PngExportOptions {
                    scale: Some(1.0),
                    dpi: Some(96.0),
                    ..Default::default()
                },
            )
            .unwrap();
        let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
        let point = expected["probe_xy_px"].as_array().unwrap();
        let actual = image.get_pixel(
            point[0].as_u64().unwrap() as u32,
            point[1].as_u64().unwrap() as u32,
        );
        let rgb = expected["probe_rgb"].as_array().unwrap();
        let reference = [
            rgb[0].as_u64().unwrap() as u8,
            rgb[1].as_u64().unwrap() as u8,
            rgb[2].as_u64().unwrap() as u8,
        ];
        assert_eq!(
            actual.0, reference,
            "{key}: raw 실루엣이 같은 잘못된 가림 순서도 검출한다"
        );
    }
}
