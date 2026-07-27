//! Rust 소비자가 사용하던 그림 바이트 공개 API의 source compatibility 계약.

use std::sync::Arc;

use rhwp::model::bin_data::{BinDataBytes, BinDataContent};
use rhwp::model::document::Document;
use rhwp::model::image::ImageEffect;
use rhwp::model::style::ImageFillMode;
use rhwp::paint::{
    LayerNode, LayerOutputOptions, PageLayerTree, RenderProfile, ResolvedImageKind,
    ResolvedImagePayload, ResourceArena, TextSourceTable,
};
use rhwp::renderer::render_tree::{BoundingBox, ImageNode, PageBackgroundImage};

fn exhaustive_legacy_match(bytes: &BinDataBytes) -> usize {
    match bytes {
        BinDataBytes::Loaded(bytes) => bytes.len(),
        BinDataBytes::Lazy { .. } => 0,
    }
}

#[test]
fn public_image_byte_api_keeps_vec_source_contract() {
    let bytes = BinDataBytes::Loaded(vec![1, 2, 3]);
    assert_eq!(exhaustive_legacy_match(&bytes), 3);

    let loaded: Vec<u8> = bytes.load();
    let limited: Option<Vec<u8>> = bytes.load_limited(3);
    assert_eq!(loaded, vec![1, 2, 3]);
    assert_eq!(limited, Some(vec![1, 2, 3]));

    let mut image = ImageNode::new(7, Some(vec![4, 5]));
    let image_data: &mut Option<Vec<u8>> = &mut image.data;
    *image_data = Some(vec![6, 7]);

    let background = PageBackgroundImage {
        data: vec![8, 9],
        fill_mode: ImageFillMode::FitToSize,
        brightness: 0,
        contrast: 0,
        effect: ImageEffect::RealPic,
    };
    let payload = ResolvedImagePayload {
        data: vec![10, 11],
        mime: "image/png",
        kind: ResolvedImageKind::FormatConverted,
        suppress_effects: false,
    };

    assert_eq!(image.data, Some(vec![6, 7]));
    assert_eq!(background.data, vec![8, 9]);
    assert_eq!(payload.data, vec![10, 11]);

    // #3455의 내부 epoch는 기존 공개 struct literal에 필드를 추가하지 않아야 한다.
    let _layer_tree = PageLayerTree {
        page_width: 100.0,
        page_height: 200.0,
        profile: RenderProfile::Screen,
        output_options: LayerOutputOptions::default(),
        root: LayerNode::leaf(BoundingBox::new(0.0, 0.0, 100.0, 200.0), None, Vec::new()),
        resources: ResourceArena::default(),
        text_sources: TextSourceTable::default(),
    };
}

#[test]
fn shared_in_memory_bytes_survive_document_snapshot_without_deep_copy() {
    let mut document = Document::default();
    document.bin_data_content.push(BinDataContent {
        id: 1,
        data: BinDataBytes::from_shared(vec![0x89, b'P', b'N', b'G']),
        extension: "png".to_string(),
    });

    let snapshot = document.clone();
    let original = document.bin_data_content[0].data.load_shared();
    let cloned = snapshot.bin_data_content[0].data.load_shared();

    assert!(Arc::ptr_eq(&original, &cloned));
    assert_eq!(&*original, &[0x89, b'P', b'N', b'G']);
}
