//! #1019: 쪽 배경의 독립 Print RGB와 별도 채움 그림의 opacity 계약.

use std::path::Path;

use rhwp::renderer::render_tree::{
    REAL_PICTURE_WATERMARK_FILL_OPACITY, REAL_PICTURE_WATERMARK_PAGE_OPACITY,
};
use rhwp::wasm_api::HwpDocument;

fn load_doc(rel_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn assert_realpic_watermark_svg_path(svg: &str, label: &str) {
    assert!(
        svg.contains("data:image/png;base64,"),
        "{label}: RealPic watermark should be emitted as tone-baked PNG"
    );
    assert!(
        !svg.contains("rhwp-img-bc-b-50c70"),
        "{label}: RealPic preset must not use generic brightness/contrast SVG filter"
    );
    assert!(
        !svg.contains("rhwp-realpic-watermark-tone"),
        "{label}: decodeable RealPic preset should bake tone into PNG pixels"
    );
    assert!(
        !svg.contains("data:application/octet-stream"),
        "{label}: image resolver must not fall back to octet-stream"
    );
}

#[test]
fn issue_1019_143_print_rgb_page_background_svg_path() {
    let doc = load_doc("samples/143E433F503322BD33.hwp");
    assert!(
        doc.page_count() >= 1,
        "fixture should have at least one page"
    );

    let svg = doc
        .render_page_svg_native(0)
        .expect("render 143E433F503322BD33.hwp page 1");
    assert_realpic_watermark_svg_path(&svg, "143E433F503322BD33 page 1");

    // 같은 원본의 독립 Print는 RGB를 보정한 불투명 그림을 쓴다.
    // 과거 화면 근사 opacity를 재적용하면 이미 보정한 색을 이중으로 감쇠한다.
    use base64::Engine;
    let input =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp"))
            .unwrap();
    let core = rhwp::DocumentCore::from_bytes(&input).unwrap();
    let originals: Vec<_> = core
        .document()
        .bin_data_content
        .iter()
        .filter_map(|bin| image::load_from_memory(&bin.data.load_shared()).ok())
        // 원본 bitmap의 고유 크기다. 문서의 px 배치 영역으로 개체를 고르지 않는다.
        .filter(|image| image.width() == 512 && image.height() == 512)
        .collect();
    assert_eq!(originals.len(), 1, "원본의 쪽 배경 그림 하나");
    let original = originals[0].to_rgba8();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let backgrounds: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("image"))
        .filter_map(|node| {
            let uri = node
                .attribute(("http://www.w3.org/1999/xlink", "href"))
                .or_else(|| node.attribute("href"))?;
            let png = uri.strip_prefix("data:image/png;base64,")?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(png)
                .unwrap();
            let pixels = image::load_from_memory(&bytes).unwrap().to_rgba8();
            (pixels.dimensions() == original.dimensions()).then_some((node, pixels))
        })
        .collect();
    assert_eq!(backgrounds.len(), 1, "실제 SVG에도 쪽 배경 그림 하나");
    let (node, pixels) = &backgrounds[0];
    for ancestor in node.ancestors().filter(roxmltree::Node::is_element) {
        let opacity: f64 = ancestor
            .attribute("opacity")
            .unwrap_or("1")
            .parse()
            .unwrap();
        assert_eq!(opacity, 1.0, "Print RGB 뒤에 추가 opacity를 곱하지 않는다");
        assert!(
            ancestor.attribute("filter").is_none(),
            "색조를 다시 보정하지 않는다"
        );
    }
    let mut compared = 0;
    for (x, y, pixel) in original.enumerate_pixels() {
        if pixel.0 == [0, 118, 191, 255] {
            // 독립 Print 이미지 객체에서 추출한 대응 RGB. JPEG 오차만 ±2로 허용한다.
            let actual = pixels.get_pixel(x, y);
            for (got, want) in actual.0[..3].iter().zip([197_u8, 255, 255]) {
                assert!(got.abs_diff(want) <= 2, "Print 배경 색조: {actual:?}");
            }
            assert_eq!(actual[3], 255, "색이 있는 원본 표본은 불투명하게 보존한다");
            compared += 1;
        }
    }
    assert!(compared > 0, "대응 원본 표본을 실제 비교해야 한다");
}

#[test]
fn issue_1019_253_empty_realpic_svg_path_pages_1_and_2() {
    let doc = load_doc("samples/253E164F57A1BC6934-empty.hwp");
    assert!(
        doc.page_count() >= 2,
        "fixture should have at least two pages"
    );

    for page in 0..2 {
        let svg = doc.render_page_svg_native(page).unwrap_or_else(|e| {
            panic!("render 253E164F57A1BC6934-empty.hwp page {}: {e}", page + 1)
        });
        assert_realpic_watermark_svg_path(
            &svg,
            &format!("253E164F57A1BC6934-empty page {}", page + 1),
        );
        let page_opacity = format!("opacity=\"{}\"", REAL_PICTURE_WATERMARK_PAGE_OPACITY);
        let fill_opacity = format!("opacity=\"{}\"", REAL_PICTURE_WATERMARK_FILL_OPACITY);
        assert!(
            svg.contains(&page_opacity) || svg.contains(&fill_opacity),
            "253E164F57A1BC6934-empty page {}: RealPic watermark opacity should be applied",
            page + 1
        );
    }
}
