//! Issue #3738 Stage 8: Center 정렬 셀의 Bottom caption 그림은 그림 본체와
//! caption을 하나의 시각 블록으로 정렬해야 한다.
//!
//! 개인정보를 제거한 실제 HWP의 23쪽 그림 21/22는 1×2 표의 Center 셀에
//! Bottom caption(각 5줄)을 둔다. caption을 제외하고 그림만 중앙 정렬하면
//! 그림과 caption이 약 50px 아래로 밀리고 caption이 다음 본문과 겹친다.
//! 한컴오피스 2020 PDF의 그림 21 caption 첫 줄은 371.37pt = 495.16px(96DPI)다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";
const PAGE_23: u32 = 22;

fn find_picture_21_and_caption(
    node: &RenderNode,
    image_y: &mut Option<f64>,
    caption_y: &mut Option<f64>,
) {
    match &node.node_type {
        RenderNodeType::Image(_)
            if (node.bbox.x - 101.9).abs() < 2.0 && node.bbox.width > 250.0 =>
        {
            *image_y = Some(node.bbox.y);
        }
        RenderNodeType::TextRun(run)
            if node.bbox.x < 110.0
                && node.bbox.y > 450.0
                && run.text.trim_start().starts_with("그림") =>
        {
            *caption_y = Some(node.bbox.y);
        }
        _ => {}
    }
    for child in &node.children {
        find_picture_21_and_caption(child, image_y, caption_y);
    }
}

#[test]
fn hwp_page23_bottom_caption_is_centered_as_one_visual_block() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage8 HWP evidence fixture");
    let tree = doc
        .build_page_render_tree(PAGE_23)
        .expect("render HWP physical page 23");

    let mut image_y = None;
    let mut caption_y = None;
    find_picture_21_and_caption(&tree.root, &mut image_y, &mut caption_y);
    let image_y = image_y.expect("figure 21 image node");
    let caption_y = caption_y.expect("figure 21 caption text node");

    assert!(
        (image_y - 148.3).abs() <= 3.0,
        "그림 21 본체가 Bottom caption을 제외하고 다시 중앙 정렬됨: image_y={image_y:.1} (회귀 전 198.4, 한컴 PDF 정합값 약 148.3)"
    );
    assert!(
        (caption_y - 495.2).abs() <= 3.0,
        "그림 21 caption 첫 줄이 한컴 PDF(371.37pt = 495.16px)와 어긋남: caption_y={caption_y:.1} (회귀 전 544.7)"
    );
}
