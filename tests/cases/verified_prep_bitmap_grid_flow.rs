//! PrEP 원본의 차트 소속·뒤 내용과 독립 Print의 격자선 표본 보존.
//! 전쪽 양쪽90%·직접 판독 증거는 bitmap_visible_palette_20261019/README.md를 따른다.
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, ImageNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::BTreeMap;
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn figures<'a>(node: &'a LayerNode, bin_data_id: u16, out: &mut Vec<(BoundingBox, &'a ImageNode)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                figures(child, bin_data_id, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            figures(child, bin_data_id, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image { bbox, image, .. } = op {
                    // 원문 BinData로 선택하며 출력 좌표로 대상을 고르지 않는다.
                    if image.bin_data_id == bin_data_id {
                        out.push((
                            image
                                .transform
                                .effective_image_bbox(&image.paint_bbox(bbox)),
                            image,
                        ));
                    }
                }
            }
        }
    }
}
fn paragraphs(node: &RenderNode, out: &mut BTreeMap<usize, (String, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.cell_context.is_none() {
            if let Some(owner) = run.para_index {
                let entry = out
                    .entry(owner)
                    .or_insert_with(|| (String::new(), node.bbox.y));
                entry.0.push_str(&run.text);
                entry.1 = entry.1.min(node.bbox.y);
            }
        }
    }
    for child in &node.children {
        paragraphs(child, out);
    }
}
// 독립 Print를 같은 96dpi로 이진화한 그림 영역에는 다섯 개의 긴 회색
// 내용 밴드가 남는다. 옅은 선은 일부 pixel center에서 임계값을 넘으므로
// 이 수를 원래 수평선 여덟 개의 완전 검출이나 정확한 위치의 증거로 쓰지 않는다.
// fixed screen 영역 대신 실제 그림 bbox의 너비 비율과 인접 행 관계를 쓴다.
#[cfg(feature = "native-skia")]
pub(crate) fn assert_print_grid_bands(output: &image::RgbaImage, bounds: BoundingBox) {
    let left = bounds.x.ceil().max(0.0) as u32;
    let top = bounds.y.floor().max(0.0) as u32;
    let right = (bounds.x + bounds.width)
        .floor()
        .min(f64::from(output.width())) as u32;
    let bottom = (bounds.y + bounds.height)
        .ceil()
        .min(f64::from(output.height())) as u32;
    assert!(right > left && bottom > top);
    let mut bands = 0;
    let mut previous_qualifies = false;
    for y in top..bottom {
        let ink = (left..right)
            .filter(|&x| {
                let p = output.get_pixel(x, y);
                let rgb = &p.0[..3];
                let low = *rgb.iter().min().unwrap();
                let high = *rgb.iter().max().unwrap();
                let sum: u16 = rgb.iter().map(|&c| u16::from(c)).sum();
                high - low <= 5 && sum > 170 * 3 && sum < 232 * 3
            })
            .count();
        let qualifies = ink as f64 >= f64::from(right - left) * 0.6;
        if qualifies && !previous_qualifies {
            bands += 1;
        }
        previous_qualifies = qualifies;
    }
    assert!(bands >= 5, "독립 Print의 다섯 내용 밴드보다 적다: {bands}");
}
#[test]
fn prep_bitmap_grid_keeps_owned_page_caption_and_following_body() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(
        core.page_count(),
        140,
        "원문 그대로의 새 MCP Print 전체 쪽수"
    );
    let mut found = Vec::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_layer_tree_with_profile(page, RenderProfile::Print)
            .unwrap();
        let mut images = Vec::new();
        figures(&tree.root, 4, &mut images);
        for (bbox, image) in images {
            assert!(
                image.para_index.is_some() && image.control_index.is_some(),
                "원문 소유 보존"
            );
            found.push((page, bbox));
        }
    }
    assert_eq!(found.len(), 1, "차트의 누락·중복 방지");
    let (page, bounds) = found[0];
    assert_eq!(page, 14, "독립 Print의 물리 15쪽");
    // 전140쪽 양쪽90% 이상·15쪽 양쪽98% 이상과 직접 판독 뒤 추가한
    // 독립 Print의 image Do 프레임이다. 캡션/후속 본문 관계와 잉크 검사도 유지한다.
    for (actual, expected) in [
        (bounds.x, 144.7226666667),
        (bounds.y, 98.132),
        (bounds.width, 503.7293333333),
        (bounds.height, 285.448),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 그림2 프레임에서2CSSpx 초과: {actual} vs {expected}"
        );
    }
    let tree = core.build_page_render_tree(page).unwrap();
    let mut paras = BTreeMap::new();
    paragraphs(&tree.root, &mut paras);
    let captions: Vec<_> = paras
        .values()
        .filter(|(t, _)| compact(t).contains("[그림2]성매개감염병6종시계열변화"))
        .collect();
    assert_eq!(captions.len(), 1, "원문 캡션 누락·중복 방지");
    let headings: Vec<_> = paras
        .values()
        .filter(|(t, _)| compact(t) == "2.연구의필요성")
        .collect();
    assert_eq!(headings.len(), 1, "후속 본문 제목 보존");
    assert!(
        bounds.y + bounds.height < captions[0].1 && captions[0].1 < headings[0].1,
        "그림→캡션→후속 본문 순서와 비겹침"
    );
    #[cfg(feature = "native-skia")]
    {
        let bytes = core
            .render_page_png_native_with_profile_and_export_options(
                page,
                RenderProfile::Print,
                &rhwp::document_core::queries::rendering::PngExportOptions {
                    dpi: Some(96.),
                    ..Default::default()
                },
            )
            .unwrap();
        let output = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_print_grid_bands(&output, bounds);
    }
}

// 전140쪽 Native/fresh WASM 모두90% 이상, 72쪽은 두 경로 모두98% 이상인
// source에만 추가한다. 독립 Print96dpi의 image Do 행렬과24개 긴 괘선에서
// 얻은 위치다. stage23 qualification/TSV/review 증거를 함께 보존한다.
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn prep_large_table_preserves_source_owner_print_frame_and_each_vertical_grid_line() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 140);
    fn count(node: &RenderNode) -> usize {
        let own = if let RenderNodeType::Image(image) = &node.node_type {
            if image.bin_data_id == 20 {
                assert_eq!(image.section_index, Some(2));
                assert_eq!(image.para_index, Some(48));
                assert_eq!(image.control_index, Some(0));
                1
            } else {
                0
            }
        } else {
            0
        };
        own + node.children.iter().map(count).sum::<usize>()
    }
    let mut pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        pages.extend(std::iter::repeat_n(page, count(&tree.root)));
    }
    assert_eq!(
        pages,
        [71],
        "원문 표 그림은 독립 Print72쪽에 한 번만 속한다"
    );
    let tree = core
        .build_page_layer_tree_with_profile(71, RenderProfile::Print)
        .unwrap();
    let mut selected = Vec::new();
    figures(&tree.root, 20, &mut selected);
    assert_eq!(selected.len(), 1);
    let (bounds, image) = selected[0];
    assert_eq!(
        (image.section_index, image.para_index, image.control_index),
        (Some(2), Some(48), Some(0))
    );
    for (actual, expected) in [
        (bounds.x, 88.752),
        (bounds.y, 94.2973333333),
        (bounds.width, 648.7706666667),
        (bounds.height, 914.9986666667),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print의 그림 프레임에서2CSSpx 초과: {actual} vs {expected}"
        );
    }
    let bytes = core
        .render_page_png_native_with_profile_and_export_options(
            71,
            RenderProfile::Print,
            &rhwp::document_core::queries::rendering::PngExportOptions {
                dpi: Some(96.),
                ..Default::default()
            },
        )
        .unwrap();
    let output = image::load_from_memory(&bytes).unwrap().to_rgba8();
    // 96dpi 독립 PDF에서 gray<=180이 세로 영역의90% 이상 이어지는24개 선.
    // 인접 두 pixel인 선은 중심을 쓴다. 각 선의±2CSSpx 안에서 실제 잉크를
    // 검사해 missing stroke를 전체 페이지의 높은 평균으로 숨기지 않는다.
    let reference_x: [f64; 24] = [
        114.5, 158., 182., 206., 230., 254., 278., 302., 326., 350., 374., 398., 422., 446., 470.,
        494., 518., 542., 566., 590., 614., 637.5, 673., 707.,
    ];
    let (top, bottom) = (95, 1009);
    assert!(bottom <= output.height());
    for x in reference_x {
        assert!(bounds.x <= x && x <= bounds.x + bounds.width);
        let left = (x - 2.).ceil() as u32;
        let right = (x + 2.).floor() as u32;
        assert!(right < output.width());
        let ink_rows = (top..bottom)
            .filter(|&y| {
                (left..=right).any(|pixel_x| {
                    let p = output.get_pixel(pixel_x, y);
                    p[0].max(p[1]).max(p[2]) <= 180
                })
            })
            .count();
        assert!(
            ink_rows * 10 >= (bottom - top) as usize * 9,
            "독립 Print 괘선 x={x}±2CSSpx의 연속 잉크가 부족하다: {ink_rows}/{}",
            bottom - top
        );
    }
}
