//! #7751: 실제 저장 색조의 원본 속성·inline/lazy 바이트·DOM용 표시 속성 계약.
//!
//! 원본 HWP SHA-256: e85d3a4b22ac9ce0515a2bf5702aafbc2259a3562e086fd523b7a05b005dd015.
//! 독립 한컴2024 Print: pdf/issue7751/hancom-color-probe-2024.pdf,
//! SHA-256: 8b4420d8caa04c822f47e67fc9edff4b3e447d64c15cac487b4bbd26e1ea6269.
//! 13단계 동일 소스 Native/fresh WASM 전1쪽 raw100%와 실제 DOM RGB를 확인한 뒤 등록했다.
//! 공개 API는 수정 전 B8 lazy RGB80/100/120으로 FAIL, 수정 후100/120/140으로 PASS다.
#![cfg(not(target_arch = "wasm32"))]
use base64::Engine;
use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

fn image_ops<'a>(node: &'a LayerNode, out: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                image_ops(child, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            image_ops(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            out.extend(ops.iter().filter(|op| matches!(op, PaintOp::Image { .. })))
        }
    }
}

#[test]
fn normal_picture_tone_keeps_source_attributes_and_shares_inline_lazy_pixels() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(
        root.join("tests/fixtures/issue_7751_color_controls/hancom-color-probe-2024.hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    let layer = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Screen)
        .unwrap();
    let mut original = vec![];
    image_ops(&layer.root, &mut original);
    assert_eq!(original.len(), 10);
    // 원본 속성은 실제 한컴 저장본과 pdf/issue7751/README.md에서 확인했다.
    let attributes: [(i8, i8, u8); 10] = [
        (0, 0, 0),
        (8, 0, 0),
        (0, 8, 0),
        (20, 0, 0),
        (0, 20, 0),
        (20, 20, 0),
        (-20, 0, 0),
        (0, -20, 0),
        (0, 50, 1),
        (10, 0, 2),
    ];
    for (op, expected) in original.iter().zip(attributes) {
        let PaintOp::Image { image, .. } = op else {
            unreachable!()
        };
        assert_eq!(
            (image.brightness, image.contrast, image.effect as u8),
            expected,
            "표시 정규화로 원본 속성을 바꾸지 않는다"
        );
    }
    // RGB(80,100,120) 타일의 실제 한컴 Print 관측값. 색조 식을 재계산하지 않는다.
    let expected: [[u8; 3]; 10] = [
        [80, 100, 120],
        [100, 120, 140],
        [76, 97, 119],
        [131, 151, 171],
        [68, 93, 118],
        [125, 150, 175],
        [29, 49, 69],
        [89, 105, 121],
        [64, 64, 64],
        [0, 0, 0],
    ];
    let narrow: Value =
        serde_json::from_str(&core.get_page_flow_image_ops_native(0).unwrap()).unwrap();
    assert_eq!(narrow["cacheable"], true);
    let ops = narrow["images"].as_array().unwrap();
    assert_eq!(ops.len(), 10);
    let mut keys = HashSet::new();
    for (op, expected) in ops.iter().zip(expected) {
        let key = op["sourceImageKey"].as_str().unwrap();
        keys.insert(key);
        let (mime, bytes) = core.get_source_image_bytes_native(key).unwrap();
        assert_eq!(mime, op["mime"].as_str().unwrap());
        let png = image::load_from_memory(&bytes).unwrap().to_rgba8();
        let pixel = png.get_pixel(png.width() * 15 / 22, png.height() / 2);
        for channel in 0..3 {
            assert!(
                (i16::from(pixel[channel]) - i16::from(expected[channel])).abs() <= 1,
                "lazy 그림 {key}: 실제{pixel:?}, 독립 Print{expected:?}"
            );
        }
        assert_eq!(pixel[3], 255);
        assert_eq!(
            op["effect"], "realPic",
            "DOM이 bake 후 효과를 중복하지 않는다"
        );
        assert_eq!(op["brightness"], 0);
        assert_eq!(op["contrast"], 0);
    }
    assert_eq!(
        keys.len(),
        10,
        "같은 BinData의 서로 다른 색조 신원을 분리한다"
    );
    // 공개 layer JSON의 inline과 omit-bytes mode도 같은 키의 같은 픽셀이다.
    let inline: Value = serde_json::from_str(&layer.to_json()).unwrap();
    fn check_inline(node: &Value, core: &DocumentCore, count: &mut usize) {
        if node.get("type").and_then(Value::as_str) == Some("image") {
            let key = node["sourceImageKey"].as_str().unwrap();
            let (mime, bytes) = core.get_source_image_bytes_native(key).unwrap();
            assert_eq!(node["mime"], mime);
            assert_eq!(
                base64::engine::general_purpose::STANDARD
                    .decode(node["base64"].as_str().unwrap())
                    .unwrap(),
                bytes
            );
            *count += 1;
        }
        match node {
            Value::Object(values) => {
                for value in values.values() {
                    check_inline(value, core, count)
                }
            }
            Value::Array(values) => {
                for value in values {
                    check_inline(value, core, count)
                }
            }
            _ => {}
        }
    }
    let mut count = 0;
    check_inline(&inline, &core, &mut count);
    assert_eq!(count, 10);
    for invalid in [
        "bin:999:1:src",
        "bin:0:1:pictone_20_0_99",
        "bin:0:1:pictone_101_0_0",
        "bin:0:1:pictone_20_0_0_extra",
    ] {
        assert!(
            core.get_source_image_bytes_native(invalid).is_none(),
            "오래된 epoch/잘못된 variant: {invalid}"
        );
    }
}
