//! #7235: NONE 그림 채우기는 원본 종횡비를 유지하며 소유 영역에 맞춘다.
//! source 이미지의 고유 픽셀 치수로 세 대상을 식별하고, 실제 render tree의
//! 소유 상자와 SVG viewport/정렬 계약을 비교한다. 과거 절대 위치 핀은 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use base64::Engine;
use rhwp::{
    document_core::DocumentCore,
    model::style::ImageFillMode,
    renderer::render_tree::{RenderNode, RenderNodeType},
};
use std::{io::Cursor, path::Path};
const FIXTURE: &str = "samples/issue7235/156086935_none_image_fill.hwp";
fn dimensions(data: &[u8]) -> Option<(u32, u32)> {
    image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}
fn matching_nodes<'a>(node: &'a RenderNode, size: (u32, u32), out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.fill_mode == Some(ImageFillMode::None)
            && image.data.as_deref().and_then(dimensions) == Some(size)
        {
            out.push(node);
        }
    }
    for child in &node.children {
        matching_nodes(child, size, out);
    }
}
fn assert_owner_viewport(page: u32, size: (u32, u32)) {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(page).unwrap();
    let mut owners = Vec::new();
    matching_nodes(&tree.root, size, &mut owners);
    assert_eq!(owners.len(), 1, "unique source image {size:?}");
    let owner = &owners[0].bbox;
    let svg = core.render_page_svg_native(page).unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let tags: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("image"))
        .filter(|n| {
            n.attribute("href")
                .or_else(|| n.attribute(("http://www.w3.org/1999/xlink", "href")))
                .and_then(|url| url.split_once(','))
                .and_then(|(_, data)| base64::engine::general_purpose::STANDARD.decode(data).ok())
                .as_deref()
                .and_then(dimensions)
                == Some(size)
        })
        .collect();
    assert_eq!(tags.len(), 1, "source image must paint once");
    let node = tags[0];
    assert_eq!(
        node.attribute("preserveAspectRatio"),
        Some("xMidYMid meet"),
        "SVG must contain and center the source pixels"
    );
    for (name, expected) in [
        ("x", owner.x),
        ("y", owner.y),
        ("width", owner.width),
        ("height", owner.height),
    ] {
        let actual = node.attribute(name).unwrap().parse::<f64>().unwrap();
        assert!(
            (actual - expected).abs() < 0.02,
            "{size:?} viewport {name}: {actual} must follow its owner {expected}"
        );
    }
}
#[test]
fn none_fill_in_table_cell_fits_cell_keeping_aspect() {
    assert_owner_viewport(0, (454, 152));
    assert_owner_viewport(8, (1009, 383));
}
#[test]
fn none_fill_in_shape_fits_shape_keeping_aspect() {
    assert_owner_viewport(0, (1500, 166));
}
