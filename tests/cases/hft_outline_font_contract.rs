//! Font-only 계약: 생성한 사각형의 소수 원점과 host 프로그램 교체를 검증한다.
//! 실물 문서의 픽셀 정답지나 한컴 일치 증거는 #7902의 전쪽 sweep으로 별도 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::style::Font;

const FAMILY: &str = "RHWP HFT Outline Fixture";
const CFF: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");

#[test]
fn registering_paint_source_preserves_document_and_replaces_unsupported_programs() {
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![
        vec![Font {
            name: FAMILY.into(),
            alt_type: 2,
            ..Default::default()
        }];
        7
    ];
    core.set_document(document);
    let before = core.get_document_info();
    let document_before = format!("{:?}", core.document());
    let saved_before = rhwp::serializer::serialize_document(core.document()).unwrap();
    assert!(!core
        .register_hft_outline_font_native("Absent family", CFF, 0)
        .unwrap());
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    assert!(core
        .register_hft_outline_font_native(FAMILY, ttf, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, ttf, 0)
        .unwrap());
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    // FontFace가 허용하지만 SFNT parser는 읽지 않는 포맷은 원래 painter에 맡긴다.
    let woff = include_bytes!("../../assets/fonts/NotoSansKR-Bold.woff2");
    assert!(core
        .register_hft_outline_font_native(FAMILY, woff, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, woff, 0)
        .unwrap());
    core.clear_text_font_family_native(FAMILY);
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    assert_eq!(
        core.get_document_info(),
        before,
        "paint 등록은 저장 문서·쪽수를 바꾸지 않는다"
    );
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "저장 IR·raw 상태를 보존한다"
    );
    assert_eq!(
        rhwp::serializer::serialize_document(core.document()).unwrap(),
        saved_before,
        "paint registry는 실제 저장 HWP 바이트에 반영되지 않는다"
    );
    assert!(core
        .register_hft_outline_font_native(FAMILY, &[], 0)
        .is_err());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, CFF, 999)
        .unwrap());
}

#[cfg(feature = "native-skia")]
mod native {
    use super::*;
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::raster_pixels::RasterPixels;
    use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::renderer::TextStyle;

    fn paint(y: f64, hft: bool, profile: RenderProfile) -> RasterPixels {
        let run = TextRunNode {
            text: "가".into(),
            style: TextStyle {
                font_family: FAMILY.into(),
                font_size: 32.0,
                source_font_is_hft: hft,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 32.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        };
        let mut tree = PageLayerTree::new(
            64.0,
            64.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                None,
                vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
            ),
        );
        tree.profile = profile;
        SkiaLayerRenderer::new()
            .with_font_paths(&[
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts")
            ])
            .render_pixels_with_options(&tree, RasterRenderOptions::default())
            .unwrap()
    }

    fn center(image: &RasterPixels) -> f64 {
        let mut mass = 0.0;
        let mut weighted = 0.0;
        for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
            for p in row[..image.width as usize * 4].chunks_exact(4) {
                mass += f64::from(p[3]);
                weighted += y as f64 * f64::from(p[3]);
            }
        }
        assert!(mass > 0.0, "실제 사각형 잉크가 있어야 한다");
        weighted / mass
    }

    #[test]
    fn print_hft_contour_preserves_quarter_pixel_baseline() {
        let original = center(&paint(4.1, true, RenderProfile::Print));
        let quarter = center(&paint(4.35, true, RenderProfile::Print));
        let whole = center(&paint(5.1, true, RenderProfile::Print));
        // 글꼴 계산 결과를 기대값으로 복사하지 않고 입력의 1/4px 이동을 검사한다.
        assert!(
            (quarter - original - 0.25).abs() <= 0.08,
            "{}",
            quarter - original
        );
        assert!((whole - original - 1.0).abs() <= 0.02);
    }

    #[test]
    fn screen_and_non_hft_keep_existing_font_raster() {
        for (hft, profile) in [(false, RenderProfile::Print), (true, RenderProfile::Screen)] {
            let original = center(&paint(4.1, hft, profile));
            let quarter = center(&paint(4.35, hft, profile));
            assert!((quarter - original).abs() <= 0.02);
        }
        assert_eq!(
            paint(4.1, true, RenderProfile::Screen),
            paint(4.1, false, RenderProfile::Screen)
        );
    }

    #[test]
    fn split_cff_uses_selected_glyph_outline_in_print_profile() {
        // 첫 face는 가, 같은 family의 두 번째 face는 나만 제공한다.
        let mut second = CFF.to_vec();
        let count = u16::from_be_bytes([second[4], second[5]]) as usize;
        let cmap = (0..count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (second.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(second[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub =
            cmap + u32::from_be_bytes(second[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([second[sub], second[sub + 1]]), 4);
        for offset in [sub + 14, sub + 20] {
            second[offset..offset + 2].copy_from_slice(&('나' as u16).to_be_bytes());
        }
        second[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub('나' as u16).to_be_bytes());
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[CFF.to_vec(), second])
            .unwrap();
        let paint_at = |y| {
            let run = TextRunNode {
                text: "나".into(),
                style: TextStyle {
                    font_family: FAMILY.into(),
                    font_size: 32.0,
                    source_font_is_hft: true,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 32.0,
                field_marker: Default::default(),
                layout_positions: None,
                display_text: None,
            };
            let mut tree = PageLayerTree::new(
                64.0,
                64.0,
                LayerNode::leaf(
                    BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                    None,
                    vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
                ),
            );
            tree.profile = RenderProfile::Print;
            center(
                &renderer
                    .render_pixels_with_options(&tree, RasterRenderOptions::default())
                    .unwrap(),
            )
        };
        let first = paint_at(4.1);
        let quarter = paint_at(4.35);
        let whole = paint_at(5.1);
        assert!(
            first < quarter && quarter < whole,
            "선택된 두 번째 CFF도 소수 원점 이동을 보존해야 한다: {first}, {quarter}, {whole}"
        );
    }
}
