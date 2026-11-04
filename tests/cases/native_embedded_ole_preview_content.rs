//! 독립 Print의 노란 3행·5열 미리보기와 숫자 보존. 실루엣 면적으로 내용 누락을 숨기지 않는다.
//! mydocs/pr/assets/ole_preview_content_20261021/README.md
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::{
    document_core::DocumentCore,
    paint::RenderProfile,
    renderer::render_tree::{RenderNode, RenderNodeType},
};
use std::path::Path;

fn raw_previews(node: &RenderNode) -> Vec<&RenderNode> {
    let mut found = Vec::new();
    if matches!(node.node_type, RenderNodeType::RawSvg(_)) {
        found.push(node);
    }
    for child in &node.children {
        found.extend(raw_previews(child));
    }
    found
}

fn assert_visible_table(core: &DocumentCore) {
    assert_eq!(core.page_count(), 1, "독립 Print의 전체 쪽수와 내용 소속");
    let tree = core.build_page_render_tree(0).unwrap();
    let previews = raw_previews(&tree.root);
    assert_eq!(previews.len(), 1, "원본 OLE 미리보기의 누락·중복");
    let preview = previews[0];
    let RenderNodeType::RawSvg(raw) = &preview.node_type else {
        unreachable!();
    };
    assert!(raw.svg.contains("xlink:href=\"data:image/svg+xml;base64,"));
    let png = core
        .render_page_png_native_with_profile_and_export_options(
            0,
            RenderProfile::Print,
            &Default::default(),
        )
        .unwrap();
    let image = image::load_from_memory(&png).unwrap().to_rgb8();
    let frame = preview.bbox;
    // 같은 완료 페이지의 양쪽 raw100%·직접 판독 후 독립 PDF의 틀만 ±2px로 보호한다.
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../mydocs/pr/assets/ole_preview_content_20261021/independent_coordinates.json"
    ))
    .unwrap();
    for (index, actual) in [frame.x, frame.y, frame.width, frame.height]
        .into_iter()
        .enumerate()
    {
        let expected = contract["expected_frame_css96"][index].as_f64().unwrap();
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print의 OLE 틀: {actual} 대 {expected}"
        );
    }
    let sx = image.width() as f64 / tree.root.bbox.width;
    let sy = image.height() as f64 / tree.root.bbox.height;
    // 독립 Print에서 확인한 표의 행·열 구획으로 자기 OLE 틀 내부를 검사한다.
    // 페이지의 절대 px 영역이나 전체 SVG 해시로 대상을 선택하지 않는다.
    for row in 0..3 {
        for column in 0..5 {
            let left = ((frame.x + frame.width * column as f64 / 5.0) * sx).ceil() as u32;
            let right = ((frame.x + frame.width * (column + 1) as f64 / 5.0) * sx).floor() as u32;
            let top = ((frame.y + frame.height * row as f64 / 3.0) * sy).ceil() as u32;
            let bottom = ((frame.y + frame.height * (row + 1) as f64 / 3.0) * sy).floor() as u32;
            let mut yellow = 0;
            let mut ink = 0;
            let mut total = 0;
            for y in top..bottom {
                for x in left..right {
                    let [r, g, b] = image.get_pixel(x, y).0;
                    yellow += usize::from(r > 200 && g > 200 && b < 80);
                    ink += usize::from(r < 100 && g < 100 && b < 100);
                    total += 1;
                }
            }
            assert!(
                yellow * 100 > total * 85,
                "셀({row},{column})의 노란 원본 바탕 보존: {yellow}/{total}"
            );
            assert!(ink > 0, "셀({row},{column})의 숫자 누락");
        }
    }
}

fn check(relative: &str) {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_shape_properties_native(0, 0, 2, r#"{"hasCaption":true,"captionDirection":"Right","captionVertAlign":"Bottom","captionWidth":8504,"captionSpacing":850,"captionIncludeMargin":true}"#).unwrap();
    assert_visible_table(&core);
    for saved in [
        core.export_hwp_with_adapter_snapshot().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_visible_table(&DocumentCore::from_bytes(&saved).unwrap());
    }
}

#[test]
fn hwp_ole_caption_and_both_saved_formats_keep_all_preview_cells_visible() {
    check("samples/한셀OLE.hwp");
}

#[test]
fn hwpx_ole_caption_and_both_saved_formats_keep_all_preview_cells_visible() {
    check("samples/한셀OLE.hwpx");
}
