//! #7751 실제 WMF의 공통 paint payload와 지연 로딩 계약.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use rhwp::document_core::DocumentCore;
use rhwp::model::image::ImageEffect;
use rhwp::paint::{
    parse_source_image_key, source_image_key, LayerBuilder, LayerNode, LayerNodeKind, PaintOp,
    RenderProfile,
};
use rhwp::renderer::render_tree::{
    BoundingBox, ImageNode, PageRenderTree, RenderNode, RenderNodeType,
};
const INPUT: &str = "samples/2025 행정업무운영 편람(최종).hwp";
fn bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT)).expect("편람 원본")
}
fn source() -> Vec<u8> {
    let document = rhwp::parser::parse_hwp(&bytes()).expect("편람 파싱");
    let content = document
        .bin_data_content
        .iter()
        .find(|entry| entry.id == 376)
        .expect("130쪽 원본 WMF");
    assert!(
        content.extension.eq_ignore_ascii_case("wmf"),
        "원본 그림 형식"
    );
    content.data.load()
}
fn png(svg: &[u8]) -> image::RgbaImage {
    let text = std::str::from_utf8(svg).unwrap();
    let start = text
        .find("data:image/png;base64,")
        .expect("실제 PNG payload")
        + 22;
    let end = start + text[start..].find('"').unwrap();
    let b = base64::engine::general_purpose::STANDARD
        .decode(&text[start..end])
        .unwrap();
    image::load_from_memory(&b).unwrap().to_rgba8()
}

fn svg_root_attrs(bytes: &[u8]) -> std::collections::HashMap<String, String> {
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let quick_xml::events::Event::Start(root) = reader.read_event().expect("SVG 루트") else {
        panic!("실제 WMF의 SVG 루트");
    };
    root.attributes()
        .map(|attr| {
            let attr = attr.expect("SVG 속성");
            (
                attr.key.as_ref().to_owned(),
                attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .unwrap()
                    .into_owned(),
            )
        })
        .collect()
}
fn find_image(node: &LayerNode) -> Option<&[u8]> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children.iter().find_map(find_image),
        LayerNodeKind::ClipRect { child, .. } => find_image(child),
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
            if let PaintOp::Image {
                resolved: Some(p), ..
            } = op
            {
                assert!(p.suppress_effects, "색조 중복 적용 방지");
                Some(p.data.as_slice())
            } else {
                None
            }
        }),
    }
}
fn check(effect: ImageEffect, b: i8, c: i8, expected_white: Option<u8>) {
    let core = DocumentCore::from_bytes(&bytes()).unwrap();
    let mut image = ImageNode::new(376, Some(source()));
    image.effect = effect;
    image.brightness = b;
    image.contrast = c;
    let key = source_image_key(0, &image).unwrap();
    assert_eq!(parse_source_image_key(&key).unwrap().1, 376);
    let (mime, lazy) = core
        .get_source_image_bytes_native(&key)
        .expect("실제 BinData 지연 조회");
    assert_eq!(mime, "image/svg+xml");
    // 이 bbox는 paint 바이트 계약을 실행하는 합성 캔버스이며 실제 문서 배치의 px 핀이 아니다.
    let mut tree = PageRenderTree::new(0, 800.0, 600.0);
    tree.root.children.push(RenderNode::new(
        1,
        RenderNodeType::Image(image),
        BoundingBox::new(10.0, 20.0, 480.0, 355.0),
    ));
    let layers = LayerBuilder::new(RenderProfile::Print).build(&tree);
    let inline = find_image(&layers.root).expect("실제 paint image");
    assert_eq!(inline, lazy, "inline과 source key 조회의 바이트 일치");
    let (_, original) = core.get_source_image_bytes_native("bin:0:376:src").unwrap();
    let original_root = svg_root_attrs(&original);
    let viewport: Vec<f64> = original_root["viewBox"]
        .split_ascii_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    let adjusted_root = svg_root_attrs(inline);
    assert_eq!(
        adjusted_root["width"].parse::<f64>().unwrap(),
        viewport[2],
        "WMF의 원본 창과 crop용 SVG 픽셀 좌표계가 같아야 한다"
    );
    assert_eq!(adjusted_root["height"].parse::<f64>().unwrap(), viewport[3]);
    let src = png(&original);
    let dst = png(inline);
    assert_eq!(
        src.dimensions(),
        dst.dimensions(),
        "색 변환에서 원본 픽셀 크기 보존"
    );
    let mut count = 0;
    let mut black_count = 0;
    for (before, after) in src.pixels().zip(dst.pixels()) {
        if before.0 == [255, 255, 255, 255] {
            if let Some(value) = expected_white {
                assert_eq!(
                    after.0,
                    [value, value, value, 255],
                    "독립 Print 원본 그림의 흰색"
                );
            }
            count += 1;
        }
        if effect == ImageEffect::GrayScale {
            assert_eq!(after[0], after[1]);
            assert_eq!(after[1], after[2]);
        }
        if expected_white.is_some() && before.0 == [0, 0, 0, 255] {
            assert_eq!(after.0, before.0, "Print의 검은 글자 위치·픽셀 보존");
            black_count += 1;
        }
    }
    assert!(count > 1000, "실제 그림의 흰색 영역을 검사");
    if expected_white.is_some() {
        assert!(black_count > 0, "원본에 있는 검은 픽셀을 실제로 검사한다");
    }
    assert!(
        core.get_source_image_bytes_native(&key.replacen("bin:0:", "bin:1:", 1))
            .is_none(),
        "이전 epoch 키 거부"
    );
}
#[test]
fn issue_7751_real_picture_inline_and_lazy_pixels_follow_independent_print() {
    check(ImageEffect::RealPic, -5, 0, Some(249))
}
#[test]
fn issue_7751_grayscale_inline_and_lazy_payload_preserve_equal_channels() {
    // 원본 Print는 실사 효과다. 다른 효과의 기대 색을 구현식으로 만들어 고정하지 않는다.
    check(ImageEffect::GrayScale, -5, 0, None)
}
#[test]
fn issue_7751_distinct_tones_and_effects_have_distinct_keys() {
    let mut image = ImageNode::new(376, Some(source()));
    image.brightness = -5;
    let first = source_image_key(0, &image).unwrap();
    image.brightness = 0;
    image.contrast = -5;
    let second = source_image_key(0, &image).unwrap();
    image.effect = ImageEffect::GrayScale;
    let third = source_image_key(0, &image).unwrap();
    assert_ne!(first, second);
    assert_ne!(second, third);
    for key in [first, second, third] {
        assert!(parse_source_image_key(&key).is_some());
    }
    for key in [
        "bin:0:376:wmftone_-5_0",
        "bin:0:376:wmftone_-5_0_2",
        "bin:0:376:wmftone_-5_0_0_extra",
    ] {
        assert!(parse_source_image_key(key).is_none());
    }
}
