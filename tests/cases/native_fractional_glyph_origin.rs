#![cfg(feature = "native-skia")]

//! 생성 번호의 소수 원점과 일반 본문의 기존 raster 격자를 함께 보호한다.
//! 장치 한 칸보다 가는 raster 획의 최소 굵기(#8125)도 같은 Native raster 격자 계약이라 여기서 검사한다.

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterPixels;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn glyph_at(renderer: &SkiaLayerRenderer, y: f64, fixed_glyph_advances: bool) -> RasterPixels {
    let run = TextRunNode {
        text: "H".into(),
        style: TextStyle {
            font_family: "Noto Sans KR".into(),
            font_size: 32.0,
            color: 0,
            fixed_glyph_advances,
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
    let tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 64.0),
            None,
            vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
        ),
    );
    renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .expect("글리프를 투명 RGBA에 재생한다")
}

fn ink_center(image: &RasterPixels) -> (f64, f64) {
    let mut mass = 0.0;
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
        for (x, pixel) in row[..image.width as usize * 4].chunks_exact(4).enumerate() {
            let weight = f64::from(pixel[3]);
            mass += weight;
            x_sum += x as f64 * weight;
            y_sum += y as f64 * weight;
        }
    }
    assert!(mass > 0.0, "공백/누락을 같은 위치의 잉크로 판정하지 않는다");
    (x_sum / mass, y_sum / mass)
}

#[test]
fn fixed_advance_glyph_preserves_quarter_pixel_baseline_translation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[root.join("ttfs/opensource")]);
    let original = ink_center(&glyph_at(&renderer, 4.1, true));
    let quarter = ink_center(&glyph_at(&renderer, 4.35, true));
    let whole = ink_center(&glyph_at(&renderer, 5.1, true));
    // 입력은 정확히1/4px 이동이다. alpha 양자화만 허용하며, 같은 정수 기준선으로
    // 반올림돼 이동이0이 되는 기존 raster 동작을 검출한다.
    let quarter_shift = quarter.1 - original.1;
    assert!(
        (quarter_shift - 0.25).abs() <= 0.08,
        "소수 기준선 이동이 잉크에 보존돼야 한다: {quarter_shift}"
    );
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}

#[test]
fn ordinary_ttf_keeps_existing_raster_baseline_grid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[root.join("ttfs/opensource")]);
    let original = ink_center(&glyph_at(&renderer, 4.1, false));
    let quarter = ink_center(&glyph_at(&renderer, 4.35, false));
    let whole = ink_center(&glyph_at(&renderer, 5.1, false));
    // 일반 TTF는 기존 화면 격자를 쓴다. 생성 번호의 별도 계약을 모든 본문에
    // 확장해 정상 쪽의 잉크 위치까지 바꾸는 회귀를 검출한다.
    assert!((quarter.1 - original.1).abs() <= 0.02);
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}

/// Native raster 의 장치 한 칸보다 가는 획은 한 칸 굵기로 그린다.
///
/// 한컴 Print PDF 를 raster 로 읽으면 0.1 mm(약 0.38 px) 표 괘선도 장치 한 칸을 채운다.
/// Native Skia 가 선언 굵기 그대로 anti-alias 하면 괘선의 덮임이 38% 안팎에 그쳐 거의 보이지
/// 않는다. `samples/basic/calendar_year.hwp`(한 쪽, 기준 `pdf/basic/calendar_year-2022.pdf`)는
/// 이 차이로 Native 실루엣이 88.25 였고, 한 칸 굵기로 그리면 98.87(fresh WASM 93.80)이다.
///
/// 조판 좌표·선언 굵기는 그대로이고 raster 획 굵기만 바뀌므로, render tree 의 실제 괘선 위치에서
/// 그 괘선을 가로지르는 화소 중 가장 진한 화소가 선 색 대비의 절반 이상인지 본다.
mod subpixel_stroke_hairline {
    use rhwp::document_core::DocumentCore;
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};

    fn thin_horizontal_lines(node: &RenderNode, out: &mut Vec<LineNode>) {
        if let RenderNodeType::Line(line) = &node.node_type {
            let horizontal = (line.y1 - line.y2).abs() < 0.01 && (line.x2 - line.x1).abs() > 20.0;
            if horizontal && line.style.width > 0.0 && line.style.width < 1.0 {
                out.push(line.clone());
            }
        }
        for child in &node.children {
            thin_horizontal_lines(child, out);
        }
    }

    #[test]
    fn subpixel_table_rules_cover_one_device_pixel() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/basic/calendar_year.hwp");
        let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
        let tree = core.build_page_render_tree(0).expect("page");
        let mut lines = Vec::new();
        thin_horizontal_lines(&tree.root, &mut lines);
        assert!(lines.len() >= 10, "가는 가로 괘선 — {}개", lines.len());

        let width = tree.root.bbox.width.round() as u32;
        let pixels = core
            .render_page_pixels_native_at_width(0, width, RenderProfile::Print, &[])
            .expect("raster");
        let scale = f64::from(pixels.width) / tree.root.bbox.width;
        let gray = |x: i32, y: i32| -> f64 {
            let at = y as usize * pixels.stride + x as usize * 4;
            let p = &pixels.bytes[at..at + 3];
            (f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2])) / 3.0
        };

        let mut faint = Vec::new();
        for line in &lines {
            let color = line.style.color;
            let ink = (f64::from((color & 0xff) as u8)
                + f64::from(((color >> 8) & 0xff) as u8)
                + f64::from(((color >> 16) & 0xff) as u8))
                / 3.0;
            let contrast = 255.0 - ink;
            if contrast < 64.0 {
                continue;
            }
            // 괘선 양 끝(교차 괘선)을 피해 가운데 세 지점을 본다.
            for t in [0.3, 0.5, 0.7] {
                let x = ((line.x1 + (line.x2 - line.x1) * t) * scale) as i32;
                let cy = line.y1 * scale;
                let darkest = ((cy - 2.0) as i32..=(cy + 2.0) as i32)
                    .filter(|y| *y >= 0 && *y < pixels.height)
                    .map(|y| 255.0 - gray(x, y))
                    .fold(0.0, f64::max);
                if darkest < contrast * 0.5 {
                    faint.push((line.y1, line.style.width, darkest / contrast));
                }
            }
        }
        assert!(
            faint.is_empty(),
            "장치 한 칸을 덮지 못한 괘선 {}곳 (y, 굵기, 최대 덮임) — {:?}",
            faint.len(),
            &faint[..faint.len().min(6)]
        );
    }

    fn thin_lines(node: &RenderNode, out: &mut Vec<LineNode>) {
        if let RenderNodeType::Line(line) = &node.node_type {
            let axis = (line.y1 - line.y2).abs() < 0.01 || (line.x1 - line.x2).abs() < 0.01;
            let long = (line.x2 - line.x1).hypot(line.y2 - line.y1) > 100.0;
            if axis && long && line.style.width > 0.0 && line.style.width < 1.0 {
                out.push(line.clone());
            }
        }
        for child in &node.children {
            thin_lines(child, out);
        }
    }

    /// 단일 실선의 인쇄 coverage 계획은 점선을 원래 굵기로 둔다. 점선 쪽 테두리도 장치 한 칸을
    /// 덮어야 한다 — `samples/basic/sungeo.hwp` 34쪽(기준 `pdf/basic/sungeo-2022.pdf`)의 점선 쪽
    /// 테두리 네 변은 선언 굵기 그대로면 거의 보이지 않아 Native raw 실루엣이 78.75 였고, 한 칸
    /// 굵기로 그리면 99.58(fresh WASM 99.99)이다.
    ///
    /// 같은 PDF 를 96DPI 로 raster 하면(poppler) 네 변 모두 회색 63 의 내용 화소다. Visual Sweep 의
    /// 내용 화소 판정(최소 채널 < 232)으로 rhwp 도 각 변에 내용 화소가 있는지 본다. 점선의 빈 칸을
    /// 피하려고 선 방향 ±20px 창에서 선을 가로지르는 ±2px 화소 중 가장 진한 값을 쓴다.
    #[test]
    fn subpixel_dashed_page_border_covers_one_device_pixel() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp");
        let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
        let page = 33;
        let tree = core.build_page_render_tree(page).expect("page");
        let mut lines = Vec::new();
        thin_lines(&tree.root, &mut lines);
        assert!(
            lines.len() >= 4,
            "쪽 테두리 네 변의 가는 선 — {}개",
            lines.len()
        );

        let width = tree.root.bbox.width.round() as u32;
        let pixels = core
            .render_page_pixels_native_at_width(page, width, RenderProfile::Print, &[])
            .expect("raster");
        let scale = f64::from(pixels.width) / tree.root.bbox.width;
        let min_channel = |x: i32, y: i32| -> u8 {
            let at = y as usize * pixels.stride + x as usize * 4;
            pixels.bytes[at..at + 3]
                .iter()
                .copied()
                .min()
                .unwrap_or(255)
        };
        let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < pixels.width && y < pixels.height;

        let mut faint = Vec::new();
        for line in &lines {
            let horizontal = (line.y1 - line.y2).abs() < 0.01;
            for t in [0.3, 0.5, 0.7] {
                let along = if horizontal {
                    (line.x1 + (line.x2 - line.x1) * t) * scale
                } else {
                    (line.y1 + (line.y2 - line.y1) * t) * scale
                };
                let across = if horizontal { line.y1 } else { line.x1 } * scale;
                let mut darkest = 255u8;
                for a in (along - 20.0) as i32..=(along + 20.0) as i32 {
                    for c in (across - 2.0) as i32..=(across + 2.0) as i32 {
                        let (x, y) = if horizontal { (a, c) } else { (c, a) };
                        if inside(x, y) {
                            darkest = darkest.min(min_channel(x, y));
                        }
                    }
                }
                if darkest >= 232 {
                    faint.push((horizontal, across / scale, darkest));
                }
            }
        }
        assert!(
            faint.is_empty(),
            "내용 화소가 없는 점선 쪽 테두리 {}곳 (가로, 위치, 가장 진한 최소 채널) — {:?}",
            faint.len(),
            &faint[..faint.len().min(6)]
        );
    }
}
