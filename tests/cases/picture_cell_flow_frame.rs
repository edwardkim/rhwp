//! 검사 유형: 관계형. 줄·문단 소속, 전체 프레임 예약과 내부 표시 기하의 보존을 검사한다.
//! 주석 갱신: 2026-11-14T02:13:49+09:00
//! Native/fresh WASM 공개 31종 최저 99.97213%, 메인터너 HWP/HWPX 대조군 각 100%.
//! 근거: mydocs/pr/assets/jeong_sik_integration_20261114/input-reference-pairs.json.
//! 제품 source 6f8d22ac95의 동일 입력·한컴 2020 Desktop Print·96 DPI·2px raw 비교다.
//! 절대 좌표를 고정하지 않으며 원본 프레임·소속과 반복 저장 전후의 관계를 검증한다.

#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{ImageNode, RenderNode, RenderNodeType};
use serde_json::Value;

fn load(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}
fn visit(node: &RenderNode, images: &mut Vec<RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Image(image) if image.fill_mode.is_none()) {
        images.push(node.clone());
    }
    for child in &node.children {
        visit(child, images);
    }
}
fn images(core: &DocumentCore) -> Vec<RenderNode> {
    let mut result = Vec::new();
    for page in 0..core.page_count() {
        visit(
            &core.build_page_render_tree(page).unwrap().root,
            &mut result,
        );
    }
    result.sort_by_key(|node| payload(node).bin_data_id);
    result
}
fn payload(node: &RenderNode) -> &ImageNode {
    let RenderNodeType::Image(image) = &node.node_type else {
        unreachable!()
    };
    image
}
fn source_ids(controls: &[Control], ids: &mut Vec<u16>) {
    for control in controls {
        match control {
            Control::Picture(p) => ids.push(p.image_attr.bin_data_id),
            Control::Table(t) => {
                for cell in &t.cells {
                    for p in &cell.paragraphs {
                        source_ids(&p.controls, ids);
                    }
                }
            }
            _ => {}
        }
    }
}
#[test]
fn every_public_picture_has_one_rendered_owner() {
    let groups: &[(&str, &[&str])] = &[
        (
            "picture-effect-reflow",
            &[
                "grid-soft-hancom",
                "mixed",
                "top",
                "bottom",
                "short-inline",
                "one-float",
                "positive",
                "following",
                "stale-height",
            ],
        ),
        (
            "inline-picture-cell-flow",
            &[
                "all-inline",
                "no-indent",
                "left",
                "right",
                "center",
                "top",
                "bottom",
                "short",
                "following",
                "stale-height",
                "one-row",
                "positive-indent",
            ],
        ),
        (
            "picture-cell-paragraph-flow",
            &[
                "two-paras",
                "top",
                "center",
                "bottom",
                "three-paras",
                "para-spacing",
                "unequal",
                "compact",
                "wrapped",
                "right",
            ],
        ),
    ];
    for (folder, stems) in groups {
        for stem in *stems {
            let core = load(&format!("mydocs/pr/assets/{folder}/{stem}.hwp"));
            assert_eq!(
                core.page_count(),
                1,
                "동일 입력의 독립 Print: {folder}/{stem}"
            );
            let mut expected = Vec::new();
            for section in &core.document().sections {
                for para in &section.paragraphs {
                    source_ids(&para.controls, &mut expected);
                }
            }
            expected.sort_unstable();
            let actual: Vec<_> = images(&core)
                .iter()
                .map(|node| payload(node).bin_data_id)
                .collect();
            assert!(!expected.is_empty());
            assert_eq!(actual, expected, "그림 누락·중복: {folder}/{stem}");
        }
    }
}
#[test]
fn inline_picture_controls_keep_independent_print_row_membership() {
    let core = load("mydocs/pr/assets/inline-picture-cell-flow/all-inline.hwp");
    let nodes = images(&core);
    assert_eq!(nodes.len(), 4);
    assert!(
        (nodes[0].bbox.y - nodes[1].bbox.y).abs() < 0.01,
        "첫 두 개체는 같은 줄"
    );
    assert!(
        nodes[2].bbox.y >= nodes[1].bbox.y + nodes[1].bbox.height - 0.01,
        "세 번째는 다음 줄"
    );
    assert!(
        nodes[3].bbox.y >= nodes[2].bbox.y + nodes[2].bbox.height - 0.01,
        "네 번째는 그 다음 줄"
    );
    assert!(
        nodes
            .iter()
            .all(|node| payload(node).cell_para_index == Some(0)),
        "하나의 셀 문단 소속"
    );
}
#[test]
fn mixed_float_and_inline_pictures_reserve_separate_whole_frames() {
    let nodes = images(&load("mydocs/pr/assets/picture-effect-reflow/mixed.hwp"));
    assert_eq!(nodes.len(), 4);
    for (index, left) in nodes.iter().enumerate() {
        for right in &nodes[index + 1..] {
            let width = ((left.bbox.x + left.bbox.width).min(right.bbox.x + right.bbox.width)
                - left.bbox.x.max(right.bbox.x))
            .max(0.0);
            let height = ((left.bbox.y + left.bbox.height).min(right.bbox.y + right.bbox.height)
                - left.bbox.y.max(right.bbox.y))
            .max(0.0);
            assert!(
                width * height < 0.01,
                "내부 그림이 작아도 전체 프레임끼리 겹치지 않는다"
            );
        }
    }
}
fn owner_core(extension: &str) -> DocumentCore {
    load(&format!(
        "mydocs/pr/assets/jeong_sik_integration_20261114/input/img_cmp_01.{extension}"
    ))
}
fn owner_contract(core: &DocumentCore) {
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    visit(&tree.root, &mut nodes);
    nodes.sort_by_key(|n| payload(n).control_index);
    assert_eq!(nodes.len(), 2);
    let (first, second) = (&nodes[0], &nodes[1]);
    assert!((first.bbox.width - second.bbox.width).abs() < 0.01);
    assert!((first.bbox.height - second.bbox.height).abs() < 0.01);
    assert!(
        second.bbox.y >= first.bbox.y + first.bbox.height - 0.01,
        "두 번째는 첫 그림의 전체 프레임 아래"
    );
    let a = payload(first).transformed_paint_bbox(&first.bbox);
    let b = payload(second).transformed_paint_bbox(&second.bbox);
    assert!(
        b.width < a.width && b.height < a.height,
        "같은 프레임이어도 둘째 그림의 실제 표시 영역은 작다"
    );
    assert!(
        (b.width / second.bbox.width - 1.0 / 3.0).abs() < 0.001,
        "원본 꼭짓점 폭과 orgSz의 비율"
    );
    assert!((b.height / second.bbox.height - 22500.0 / 120000.0).abs() < 0.001);
    assert_eq!(payload(first).para_index, payload(second).para_index);
    assert_ne!(payload(first).control_index, payload(second).control_index);
}
#[test]
fn source_vertices_and_stored_closing_line_keep_both_body_pictures() {
    for ext in ["hwp", "hwpx"] {
        let core = owner_core(ext);
        assert_eq!(core.page_count(), 1);
        owner_contract(&core);
    }
}
#[test]
fn repeated_hwp_save_preserves_internal_image_and_whole_frame_reservations() {
    for ext in ["hwp", "hwpx"] {
        let mut core = owner_core(ext);
        for _ in 0..3 {
            owner_contract(&core);
            core = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
        }
        owner_contract(&core);
    }
}
fn image_ops(value: &Value, result: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("image") {
                result.push(value.clone());
            }
            for v in map.values() {
                image_ops(v, result);
            }
        }
        Value::Array(values) => {
            for value in values {
                image_ops(value, result);
            }
        }
        _ => {}
    }
}
#[test]
fn print_raster_samples_follow_the_internal_paint_window() {
    let core = load("mydocs/pr/assets/picture-effect-reflow/mixed.hwp");
    let layer = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let value: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(0, RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let mut ops = Vec::new();
    image_ops(&value, &mut ops);
    assert_eq!(ops.len(), 4);
    for op in ops {
        let frame = op["imagePrintFrame"]
            .as_array()
            .expect("실제 JPEG 장치 프레임");
        let width = op["imagePaintBBox"]["width"].as_f64().unwrap();
        let height = op["imagePaintBBox"]["height"].as_f64().unwrap();
        let columns = frame[4].as_f64().unwrap();
        let rows = frame[5].as_f64().unwrap();
        let page_columns = frame[0].as_f64().unwrap() / 12.0;
        let page_rows = frame[1].as_f64().unwrap() / 12.0;
        assert!(
            (columns / page_columns - width / layer.page_width).abs() <= 2.0 / page_columns,
            "내부 번호가 프레임 clip 바깥으로 이동하지 않는다"
        );
        assert!((rows / page_rows - height / layer.page_height).abs() <= 2.0 / page_rows);
    }
}

// HWP5 표 110~116의 효과와 가변 색상 레코드 뒤에 imgDim·alpha가 온다.
// 화면 좌표가 아닌 파일 형식 계약이며 알려진 효과 조합의 저장·재파싱을 확인한다.
fn first_picture(controls: &mut [Control]) -> Option<&mut rhwp::model::image::Picture> {
    for control in controls {
        match control {
            Control::Picture(picture) => return Some(picture),
            Control::Table(table) => {
                for cell in &mut table.cells {
                    for paragraph in &mut cell.paragraphs {
                        if let Some(picture) = first_picture(&mut paragraph.controls) {
                            return Some(picture);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn effect_color_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 8]; // RGB 종류와 RGB 값
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 16]); // 색상 보정 두 항목
    bytes
}

#[test]
fn known_effect_records_keep_dimensions_after_all_effect_combinations() {
    for flags in 0u32..16 {
        let mut document = owner_core("hwp").document().clone();
        let picture = document
            .sections
            .iter_mut()
            .flat_map(|s| &mut s.paragraphs)
            .find_map(|p| first_picture(&mut p.controls))
            .unwrap();
        let mut bytes = vec![0; 5];
        bytes.extend_from_slice(&flags.to_le_bytes());
        if flags & 1 != 0 {
            bytes.extend_from_slice(&[0; 44]);
            bytes.extend(effect_color_bytes());
        }
        if flags & 2 != 0 {
            bytes.extend_from_slice(&[0; 8]);
            bytes.extend(effect_color_bytes());
        }
        if flags & 4 != 0 {
            bytes.extend_from_slice(&0f32.to_le_bytes());
        }
        if flags & 8 != 0 {
            bytes.extend_from_slice(&[0; 56]);
        }
        let effect_end = bytes.len();
        bytes.extend_from_slice(&36000u32.to_le_bytes());
        bytes.extend_from_slice(&48000u32.to_le_bytes());
        bytes.push(69);
        picture.raw_picture_extra = bytes.clone();
        // 파싱된 단독 효과의 의미 IR도 맞춘다. None은 효과를 지우는 편집이다.
        picture.effects.soft_edge_radius = (flags & 4 != 0).then_some(0.0);
        picture.img_dim = (36000, 48000);
        picture.image_attr.transparency = 69;
        let saved = rhwp::serializer::serialize_document(&document).unwrap();
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        let mut again = reopened.document().clone();
        let actual = again
            .sections
            .iter_mut()
            .flat_map(|s| &mut s.paragraphs)
            .find_map(|p| first_picture(&mut p.controls))
            .unwrap();
        assert_eq!(
            actual.img_dim,
            (36000, 48000),
            "효과 {flags}: 원본 이미지 좌표계"
        );
        assert_eq!(
            actual.image_attr.transparency, 69,
            "효과 {flags}: 전체 투명도"
        );
        assert_eq!(
            &actual.raw_picture_extra[5..effect_end],
            &bytes[5..effect_end],
            "효과 레코드를 크기 필드로 덮어쓰지 않는다"
        );
    }
}
