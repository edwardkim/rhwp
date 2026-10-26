//! #8153: 셀의 글자처럼 취급한 그림은 표의 원본 단 참조 축을 보존한다.
//! 독립 한컴 2024 Print 139쪽의 원래 bitmap만 비교하므로 시스템 글꼴은 필요 없다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;

fn isolate_picture(node: &mut LayerNode, found: &mut Vec<u16>) {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                isolate_picture(child, found);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            isolate_picture(child, found);
        }
        LayerNodeKind::Leaf { ops } => ops.retain(|op| {
            let PaintOp::Image { image, .. } = op else {
                return false;
            };
            if image.cell_context.is_some() {
                found.push(image.bin_data_id);
            }
            image.bin_data_id == 384
        }),
    }
}

fn is_ink(pixel: &image::Rgb<u8>) -> bool {
    pixel.0.into_iter().min().unwrap() < 232
}

fn verify(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let mut tree = core
        .build_page_layer_tree_with_profile(138, RenderProfile::Print)
        .unwrap();
    let mut found = Vec::new();
    isolate_picture(&mut tree.root, &mut found);
    assert_eq!(found, [383, 384], "셀 그림의 쪽 소속·내용 순서·누락·중복");
    tree.text_sources.entries.clear();
    let bytes = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, Default::default())
        .unwrap()
        .bytes;
    let raster = image::load_from_memory(&bytes).unwrap().into_rgba8();
    // 다른 paint를 분리한 PNG의 투명 종이는 PDF와 같은 흰색으로 합성한다.
    // alpha를 버리면 새로 드러난 빈 가장자리가 검정으로 바뀐다.
    let mut opaque =
        image::RgbaImage::from_pixel(raster.width(), raster.height(), image::Rgba([255; 4]));
    image::imageops::overlay(&mut opaque, &raster, 0, 0);
    let actual = image::DynamicImage::ImageRgba8(opaque).into_rgb8();
    let actual = image::imageops::crop_imm(&actual, 139, 658, 447, 195).to_image();
    let expected = image::load_from_memory(include_bytes!(
        "../fixtures/issue8153/handbook_p139_picture_hancom2024_96dpi.png"
    ))
    .unwrap()
    .into_rgb8();
    assert_eq!(actual.dimensions(), expected.dimensions());
    // 독립 PDF의 동일 영역에서 2px 실루엣을 직접 대조한다. 위치의
    // 소수 px를 고정하거나 제품 계산값을 기대값으로 재인용하지 않는다.
    for (name, source, target) in [
        ("현재→Print", &actual, &expected),
        ("Print→현재", &expected, &actual),
    ] {
        let mut unmatched = 0;
        for (x, y, pixel) in source.enumerate_pixels() {
            if !is_ink(pixel) {
                continue;
            }
            let nearby = (y.saturating_sub(2)..=(y + 2).min(target.height() - 1)).any(|yy| {
                (x.saturating_sub(2)..=(x + 2).min(target.width() - 1))
                    .any(|xx| is_ink(target.get_pixel(xx, yy)))
            });
            unmatched += usize::from(!nearby);
        }
        assert_eq!(unmatched, 0, "{extension} {name}: 그림의 2px 실루엣");
    }
}

#[test]
fn hwp_inline_cell_picture_preserves_the_independent_print_footprint() {
    verify("hwp");
}

#[test]
fn hwpx_inline_cell_picture_preserves_the_independent_print_footprint() {
    verify("hwpx");
}
