//! #6852: HWPX 5쪽의 일반 사각형을 글상자 마스크로 추정해
//! 원본 실선을 지우지 않는지 확인한다.
//! HWP 원본의 픽셀 고정 검사 다섯 개는 전쪽 시각 기준 미달로 #7445에 이관했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const HWPX: &str = "samples/hwpx/156160455-social-pig-farm-income.hwpx";

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("원본 fixture 필수");
    DocumentCore::from_bytes(&bytes).expect("원본 파싱")
}

fn check_rectangles(core: &DocumentCore, foreground_stroke: bool) {
    assert_eq!(core.page_count(), 11, "원본 쪽수 보존");
    let svg = core.render_page_svg_native(4).expect("5쪽 SVG");
    let xml = roxmltree::Document::parse(&svg).expect("SVG XML");
    let coord =
        |node: roxmltree::Node<'_, '_>, name| node.attribute(name).unwrap().parse::<f64>().unwrap();
    let boxes: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("rect"))
        .filter(|node| {
            !node
                .ancestors()
                .any(|parent| parent.has_tag_name("clipPath"))
        })
        .filter(|node| {
            node.attribute("x").is_some()
                && node.attribute("y").is_some()
                && (75.0..80.0).contains(&coord(*node, "x"))
                && (129.0..133.0).contains(&coord(*node, "y"))
                && (16.0..16.1).contains(&coord(*node, "width"))
                && (17.0..17.1).contains(&coord(*node, "height"))
        })
        .collect();
    assert_eq!(boxes.len(), 2, "제목 앞 사각형 2개를 정확히 식별");
    let shadow = boxes[0];
    let foreground = boxes[1];
    assert_eq!(shadow.attribute("fill"), Some("#282828"));
    assert_eq!(shadow.attribute("stroke"), Some("#000000"));
    assert!((coord(shadow, "stroke-width") - 0.5).abs() < 1e-8);
    for (node, x, y) in [
        (shadow, 78.10666666666667, 131.28),
        (foreground, 75.58666666666667, 129.4933333333333),
    ] {
        assert!((coord(node, "x") - x).abs() < 1e-6, "x 유지");
        assert!((coord(node, "y") - y).abs() < 1e-6, "y 유지");
    }
    assert_eq!(
        foreground.attribute("stroke"),
        foreground_stroke.then_some("#000000"),
        "앞쪽 사각형의 명시된 선을 글상자 보정으로 제거해서는 안 된다"
    );
    if foreground_stroke {
        assert!((coord(foreground, "stroke-width") - 0.5).abs() < 1e-8);
    }
}

#[test]
fn original_hwpx_foreground_and_shadow_keep_solid_strokes() {
    check_rectangles(&load(HWPX), true);
}
