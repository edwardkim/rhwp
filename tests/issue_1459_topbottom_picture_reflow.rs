//! Issue #1459: 같은 문단의 TAC 그림과 자리차지 그림 혼합 배치.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
struct ImageRender {
    control_index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    opacity: f64,
}

fn collect_images(node: &RenderNode, out: &mut Vec<ImageRender>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if let Some(control_index) = img.control_index {
            out.push(ImageRender {
                control_index,
                x: node.bbox.x,
                y: node.bbox.y,
                width: node.bbox.width,
                height: node.bbox.height,
                opacity: img.opacity,
            });
        }
    }
    for child in &node.children {
        collect_images(child, out);
    }
}

fn load_fixture(path: &str) -> DocumentCore {
    let bytes = load_fixture_bytes(path);
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {path}: {e}"))
}

fn load_fixture_bytes(path: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(repo_root).join(path);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn parse_json(label: &str, json: &str) -> Value {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("parse {label} json `{json}`: {e}"))
}

#[test]
fn topbottom_second_picture_flows_before_tac_picture() {
    for path in [
        "samples/투명도0-50-2nd그림글차처럼off.hwp",
        "samples/투명도0-50-2nd그림글차처럼off.hwpx",
    ] {
        let core = load_fixture(path);
        let para = &core.document().sections[0].paragraphs[0];
        let mut tac_picture = None;
        let mut topbottom_picture = None;
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Picture(pic) = ctrl {
                if pic.common.treat_as_char {
                    tac_picture = Some((ci, pic.image_attr.clamped_transparency()));
                } else if matches!(pic.common.text_wrap, TextWrap::TopAndBottom) {
                    topbottom_picture = Some((ci, pic.image_attr.clamped_transparency()));
                }
            }
        }
        let (tac_ci, tac_transparency) =
            tac_picture.unwrap_or_else(|| panic!("{path}: TAC 그림 누락"));
        let (topbottom_ci, topbottom_transparency) =
            topbottom_picture.unwrap_or_else(|| panic!("{path}: 자리차지 그림 누락"));
        assert_eq!(
            tac_transparency, 0,
            "{path}: 첫 TAC 그림은 투명도 0이어야 함"
        );
        assert_eq!(
            topbottom_transparency, 50,
            "{path}: 두 번째 자리차지 그림은 투명도 50이어야 함"
        );

        let tree = core
            .build_page_render_tree(0)
            .unwrap_or_else(|e| panic!("render tree {path}: {e}"));
        let mut images = Vec::new();
        collect_images(&tree.root, &mut images);

        let topbottom = images
            .iter()
            .find(|img| img.control_index == topbottom_ci)
            .unwrap_or_else(|| panic!("{path}: 자리차지 그림 ImageNode 누락: {images:?}"));
        let tac = images
            .iter()
            .find(|img| img.control_index == tac_ci)
            .unwrap_or_else(|| panic!("{path}: TAC 그림 ImageNode 누락: {images:?}"));

        assert!(
            topbottom.y < tac.y,
            "{path}: 한컴처럼 투명도 50 자리차지 그림이 먼저 흐르고 TAC 그림이 아래에 있어야 함: topbottom={topbottom:?}, tac={tac:?}, all={images:?}"
        );
        let vertical_gap = tac.y - (topbottom.y + topbottom.height);
        assert!(
            vertical_gap.abs() <= 2.0,
            "{path}: TAC 그림은 자리차지 그림 bbox 바로 다음 줄에 이어져야 함: gap={vertical_gap:.2}, topbottom={topbottom:?}, tac={tac:?}, all={images:?}"
        );
        assert!(
            topbottom.opacity < tac.opacity,
            "{path}: 자리차지 그림은 투명도 50%, TAC 그림은 투명도 0% 렌더여야 함: topbottom={topbottom:?}, tac={tac:?}"
        );
    }
}

#[test]
fn turning_first_picture_off_reflows_remaining_tac_picture() {
    let mut core = load_fixture("samples/투명도0-50.hwp");
    let (first_ci, second_ci, first_reserved_hu, second_height_hu) = {
        let para = &core.document().sections[0].paragraphs[0];
        let picture_indices = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(ci, ctrl)| matches!(ctrl, Control::Picture(_)).then_some(ci))
            .collect::<Vec<_>>();
        assert!(
            picture_indices.len() >= 2,
            "투명도0-50.hwp에는 같은 문단의 그림 2개가 필요함: {picture_indices:?}"
        );
        let first_ci = picture_indices[0];
        let second_ci = picture_indices[1];
        let first = match &para.controls[first_ci] {
            Control::Picture(pic) => pic.as_ref(),
            _ => unreachable!(),
        };
        let second = match &para.controls[second_ci] {
            Control::Picture(pic) => pic.as_ref(),
            _ => unreachable!(),
        };
        assert!(first.common.treat_as_char, "원본 첫 그림은 TAC여야 함");
        assert!(second.common.treat_as_char, "원본 둘째 그림은 TAC여야 함");
        (
            first_ci,
            second_ci,
            first.common.height as i32
                + first.common.margin.top as i32
                + first.common.margin.bottom as i32,
            second.common.height as i32,
        )
    };

    core.set_picture_properties_native(
        0,
        0,
        first_ci,
        r#"{"treatAsChar":false,"textWrap":"TopAndBottom"}"#,
    )
    .expect("첫 번째 그림 TAC 해제");

    let para = &core.document().sections[0].paragraphs[0];
    let first = match &para.controls[first_ci] {
        Control::Picture(pic) => pic.as_ref(),
        _ => unreachable!(),
    };
    let second = match &para.controls[second_ci] {
        Control::Picture(pic) => pic.as_ref(),
        _ => unreachable!(),
    };
    assert!(
        !first.common.treat_as_char,
        "첫 그림은 자리차지 그림으로 바뀌어야 함"
    );
    assert!(
        matches!(first.common.text_wrap, TextWrap::TopAndBottom),
        "첫 그림은 자리차지 방식이어야 함"
    );
    assert!(second.common.treat_as_char, "둘째 그림은 여전히 TAC여야 함");
    assert_eq!(
        para.line_segs.len(),
        1,
        "빈 그림 문단에서 TAC 해제된 그림은 inline 줄에서 빠지고 남은 TAC 줄만 남아야 함"
    );
    assert_eq!(
        para.line_segs[0].vertical_pos, first_reserved_hu,
        "자리차지 그림 예약 높이는 남은 TAC 줄의 vertical_pos에 반영되어야 함"
    );
    assert_eq!(
        para.line_segs[0].line_height, second_height_hu,
        "남은 TAC 줄 높이는 둘째 그림 높이를 따라야 함"
    );

    let tree = core
        .build_page_render_tree(0)
        .expect("속성 변경 후 render tree");
    let mut images = Vec::new();
    collect_images(&tree.root, &mut images);
    let topbottom = images
        .iter()
        .find(|img| img.control_index == first_ci)
        .unwrap_or_else(|| panic!("첫 자리차지 그림 ImageNode 누락: {images:?}"));
    let tac = images
        .iter()
        .find(|img| img.control_index == second_ci)
        .unwrap_or_else(|| panic!("둘째 TAC 그림 ImageNode 누락: {images:?}"));

    assert!(
        topbottom.y < tac.y,
        "한컴처럼 첫 자리차지 그림이 먼저 렌더되고 둘째 TAC 그림이 아래에 이어져야 함: topbottom={topbottom:?}, tac={tac:?}, all={images:?}"
    );
    let vertical_gap = tac.y - (topbottom.y + topbottom.height);
    assert!(
        vertical_gap.abs() <= 2.0,
        "속성 변경 직후에도 둘째 TAC 그림은 첫 자리차지 그림 바로 다음 줄에 이어져야 함: gap={vertical_gap:.2}, topbottom={topbottom:?}, tac={tac:?}, all={images:?}"
    );
}

#[test]
fn non_tac_topbottom_picture_is_not_caret_stop() {
    for path in [
        "samples/투명도0-50-2nd그림글차처럼off.hwp",
        "samples/투명도0-50-2nd그림글차처럼off.hwpx",
    ] {
        let bytes = load_fixture_bytes(path);
        let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {path}: {e}"));
        let para = &core.document().sections[0].paragraphs[0];
        let mut tac_ci = None;
        let mut topbottom_ci = None;
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Picture(pic) = ctrl {
                if pic.common.treat_as_char {
                    tac_ci = Some(ci);
                } else if matches!(pic.common.text_wrap, TextWrap::TopAndBottom) {
                    topbottom_ci = Some(ci);
                }
            }
        }
        let tac_ci = tac_ci.unwrap_or_else(|| panic!("{path}: TAC 그림 누락"));
        let topbottom_ci = topbottom_ci.unwrap_or_else(|| panic!("{path}: 자리차지 그림 누락"));

        let tree = core
            .build_page_render_tree(0)
            .unwrap_or_else(|e| panic!("render tree {path}: {e}"));
        let mut images = Vec::new();
        collect_images(&tree.root, &mut images);
        let topbottom = images
            .iter()
            .find(|img| img.control_index == topbottom_ci)
            .unwrap_or_else(|| panic!("{path}: 자리차지 그림 ImageNode 누락: {images:?}"));
        let tac = images
            .iter()
            .find(|img| img.control_index == tac_ci)
            .unwrap_or_else(|| panic!("{path}: TAC 그림 ImageNode 누락: {images:?}"));
        let topbottom_bottom = topbottom.y + topbottom.height;

        let mut doc =
            HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("wasm {path}: {e}"));
        doc.set_show_paragraph_marks(true);

        let before_tac = parse_json(
            "before TAC cursor rect",
            &doc.get_cursor_rect_native(0, 0, 0)
                .unwrap_or_else(|e| panic!("{path}: cursor 0: {e}")),
        );
        let after_tac = parse_json(
            "after TAC cursor rect",
            &doc.get_cursor_rect_native(0, 0, 1)
                .unwrap_or_else(|e| panic!("{path}: cursor 1: {e}")),
        );
        for (label, rect) in [("before", &before_tac), ("after", &after_tac)] {
            let y = rect["y"].as_f64().unwrap();
            assert!(
                y > topbottom_bottom,
                "{path}: {label} TAC 커서가 비TAC 자리차지 그림 줄에 잡히면 안 됨: rect={rect}, topbottom={topbottom:?}, tac={tac:?}"
            );
            assert!(
                y <= tac.y + tac.height,
                "{path}: {label} TAC 커서는 TAC 그림 bbox 기준선에 있어야 함: rect={rect}, topbottom={topbottom:?}, tac={tac:?}"
            );
        }

        let hit = parse_json(
            "topbottom image hit",
            &doc.hit_test(
                0,
                topbottom.x + topbottom.width / 2.0,
                topbottom.y + topbottom.height / 2.0,
            )
            .unwrap_or_else(|e| panic!("{path}: hit topbottom image: {e:?}")),
        );
        if let Some(rect) = hit.get("cursorRect") {
            let y = rect["y"].as_f64().unwrap();
            assert!(
                y > topbottom_bottom,
                "{path}: 비TAC 자리차지 그림 클릭이 해당 그림의 문자 커서로 변환되면 안 됨: hit={hit}, topbottom={topbottom:?}, tac={tac:?}"
            );
        }
    }
}
