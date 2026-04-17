//! 레이아웃 통합 테스트
//!
//! 실제 HWP 파일을 로딩하여 페이지네이션 + 레이아웃 결과를 검증한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use resvg::{tiny_skia, usvg};
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    const SKIA_TOLERANT_CHANNEL_DELTA: u8 = 128;
    const SKIA_TOLERANT_MAX_DIFF_PIXELS: usize = 64;

    fn render_path_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// 테스트용 DocumentCore 생성 헬퍼
    fn load_document(path: &str) -> Option<crate::document_core::DocumentCore> {
        let p = Path::new(path);
        if !p.exists() {
            eprintln!("테스트 파일 없음: {} — 건너뜀", path);
            return None;
        }
        let data = std::fs::read(p).ok()?;
        crate::document_core::DocumentCore::from_bytes(&data).ok()
    }

    fn rasterize_svg(svg: &str) -> Option<tiny_skia::Pixmap> {
        let mut options = usvg::Options::default();
        let fontdb = options.fontdb_mut();
        fontdb.load_system_fonts();
        fontdb.set_sans_serif_family("Noto Sans CJK KR");
        fontdb.set_serif_family("Noto Serif CJK KR");
        fontdb.set_monospace_family("D2Coding");
        let tree = usvg::Tree::from_str(svg, &options).ok()?;
        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        Some(pixmap)
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    fn decode_png(bytes: &[u8]) -> Option<tiny_skia::Pixmap> {
        tiny_skia::Pixmap::decode_png(bytes).ok()
    }

    struct PixmapDiff {
        diff_pixels: usize,
        total_pixels: usize,
        max_channel_delta: u8,
        mean_abs_channel_delta: f64,
        diff_pixmap: tiny_skia::Pixmap,
    }

    fn diff_pixmaps(
        expected: &tiny_skia::Pixmap,
        actual: &tiny_skia::Pixmap,
        ignored_channel_delta: u8,
    ) -> PixmapDiff {
        let total_pixels = (expected.width() as usize) * (expected.height() as usize);
        let mut diff_pixmap = tiny_skia::Pixmap::new(expected.width(), expected.height())
            .expect("diff pixmap 생성 실패");
        let mut diff_pixels = 0usize;
        let mut total_channel_delta = 0u64;
        let mut max_channel_delta = 0u8;

        for (idx, (expected_px, actual_px)) in expected
            .data()
            .chunks_exact(4)
            .zip(actual.data().chunks_exact(4))
            .enumerate()
        {
            let mut pixel_diff = false;
            let mut pixel_max_delta = 0u8;

            for channel in 0..4 {
                let delta = expected_px[channel].abs_diff(actual_px[channel]);
                if delta > 0 {
                    pixel_diff = true;
                }
                total_channel_delta += u64::from(delta);
                pixel_max_delta = pixel_max_delta.max(delta);
                max_channel_delta = max_channel_delta.max(delta);
            }

            if pixel_max_delta > ignored_channel_delta {
                diff_pixels += 1;
                let base = idx * 4;
                diff_pixmap.data_mut()[base..base + 4].copy_from_slice(&[
                    pixel_max_delta.max(32),
                    0,
                    0,
                    255,
                ]);
            }
        }

        let mean_abs_channel_delta = if total_pixels == 0 {
            0.0
        } else {
            total_channel_delta as f64 / (total_pixels as f64 * 4.0)
        };

        PixmapDiff {
            diff_pixels,
            total_pixels,
            max_channel_delta,
            mean_abs_channel_delta,
            diff_pixmap,
        }
    }

    fn save_diff_artifacts(
        output_dir: &str,
        sample: &str,
        page_num: u32,
        expected_name: &str,
        actual_name: &str,
        diff_name: &str,
        expected: &tiny_skia::Pixmap,
        actual: &tiny_skia::Pixmap,
        diff: &tiny_skia::Pixmap,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let output_dir = Path::new(output_dir);
        let _ = std::fs::create_dir_all(output_dir);
        let stem = Path::new(sample)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sample");
        let expected_path = output_dir.join(format!("{stem}-{expected_name}-p{page_num}.png"));
        let actual_path = output_dir.join(format!("{stem}-{actual_name}-p{page_num}.png"));
        let diff_path = output_dir.join(format!("{stem}-{diff_name}-p{page_num}.png"));
        let _ = expected.save_png(&expected_path);
        let _ = actual.save_png(&actual_path);
        let _ = diff.save_png(&diff_path);
        (expected_path, actual_path, diff_path)
    }

    fn assert_layer_svg_pixels_match(sample: &str, page_num: u32) {
        let Some(core) = load_document(sample) else {
            return;
        };
        let legacy = core
            .render_page_svg_legacy_native(page_num)
            .unwrap_or_default();
        let layered = core
            .render_page_svg_layer_native(page_num)
            .unwrap_or_default();
        let legacy_pixmap = rasterize_svg(&legacy).expect("legacy SVG rasterize 실패");
        let layered_pixmap = rasterize_svg(&layered).expect("layer SVG rasterize 실패");

        assert_eq!(
            (layered_pixmap.width(), layered_pixmap.height()),
            (legacy_pixmap.width(), legacy_pixmap.height()),
            "legacy/layer raster 크기가 달라서는 안 됨",
        );

        let diff = diff_pixmaps(&legacy_pixmap, &layered_pixmap, 0);
        if diff.diff_pixels > 0 {
            let (legacy_path, layered_path, diff_path) = save_diff_artifacts(
                "output/layer-svg-diff",
                sample,
                page_num,
                "legacy",
                "layer",
                "diff",
                &legacy_pixmap,
                &layered_pixmap,
                &diff.diff_pixmap,
            );
            panic!(
                "legacy/layer raster diff 발생: {} pixels (legacy: {}, layer: {}, diff: {})",
                diff.diff_pixels,
                legacy_path.display(),
                layered_path.display(),
                diff_path.display(),
            );
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    fn assert_skia_png_matches_layer_svg(sample: &str, page_num: u32) {
        let Some(core) = load_document(sample) else {
            return;
        };
        let layered_svg = core
            .render_page_svg_layer_native(page_num)
            .expect("layer SVG 렌더 실패");
        let expected = rasterize_svg(&layered_svg).expect("layer SVG rasterize 실패");
        let actual_png = core
            .render_page_png_native(page_num)
            .expect("Skia PNG 렌더 실패");
        let actual = decode_png(&actual_png).expect("Skia PNG decode 실패");

        assert_eq!(
            (actual.width(), actual.height()),
            (expected.width(), expected.height()),
            "Skia/layer raster 크기가 달라서는 안 됨",
        );

        let exact_diff = diff_pixmaps(&expected, &actual, 0);
        let raw_tolerant_diff = diff_pixmaps(&expected, &actual, SKIA_TOLERANT_CHANNEL_DELTA);

        let exact_paths = if exact_diff.diff_pixels > 0 {
            Some(save_diff_artifacts(
                "output/skia-diff",
                sample,
                page_num,
                "layer",
                "skia",
                "diff",
                &expected,
                &actual,
                &exact_diff.diff_pixmap,
            ))
        } else {
            None
        };

        let tolerant_paths = if raw_tolerant_diff.diff_pixels > SKIA_TOLERANT_MAX_DIFF_PIXELS {
            Some(save_diff_artifacts(
                "output/skia-diff",
                sample,
                page_num,
                "layer",
                "skia",
                "tolerant-diff",
                &expected,
                &actual,
                &raw_tolerant_diff.diff_pixmap,
            ))
        } else {
            None
        };

        if raw_tolerant_diff.diff_pixels > SKIA_TOLERANT_MAX_DIFF_PIXELS {
            let (expected_path, actual_path, diff_path) =
                exact_paths.expect("tolerant diff가 있으면 exact diff도 있어야 함");
            let tolerant_diff_path = tolerant_paths
                .as_ref()
                .map(|(_, _, path)| path.display().to_string())
                .unwrap_or_else(|| "-".to_string());
            panic!(
                "Skia raster diff 발생: exact={} pixels, tolerant={} pixels (budget={}, ignored_channel_delta<={}), mean_abs_channel_delta={:.3}, max_channel_delta={} (layer: {}, skia: {}, exact diff: {}, tolerant diff: {})",
                exact_diff.diff_pixels,
                raw_tolerant_diff.diff_pixels,
                SKIA_TOLERANT_MAX_DIFF_PIXELS,
                SKIA_TOLERANT_CHANNEL_DELTA,
                exact_diff.mean_abs_channel_delta,
                exact_diff.max_channel_delta,
                expected_path.display(),
                actual_path.display(),
                diff_path.display(),
                tolerant_diff_path,
            );
        }
    }

    #[test]
    fn test_diff_pixmaps_ignores_small_channel_deltas_when_configured() {
        let mut expected = tiny_skia::Pixmap::new(2, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(2, 1).expect("actual pixmap 생성 실패");

        expected
            .data_mut()
            .copy_from_slice(&[10, 20, 30, 255, 80, 90, 100, 255]);
        actual
            .data_mut()
            .copy_from_slice(&[12, 20, 30, 255, 80, 90, 106, 255]);

        let exact = diff_pixmaps(&expected, &actual, 0);
        let tolerant = diff_pixmaps(&expected, &actual, 4);

        assert_eq!(exact.total_pixels, 2);
        assert_eq!(exact.diff_pixels, 2);
        assert_eq!(tolerant.diff_pixels, 1);
        assert_eq!(exact.max_channel_delta, 6);
        assert_eq!(tolerant.max_channel_delta, 6);
    }

    #[test]
    fn test_skia_tolerant_budget_zeroes_passing_diff() {
        let mut expected = tiny_skia::Pixmap::new(4, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(4, 1).expect("actual pixmap 생성 실패");

        expected.data_mut().copy_from_slice(&[
            10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 1, 2, 3, 255,
        ]);
        actual.data_mut().copy_from_slice(&[
            10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 91, 255, 1, 2, 4, 255,
        ]);

        let raw_tolerant = diff_pixmaps(&expected, &actual, 0);
        let budgeted_tolerant = if raw_tolerant.diff_pixels <= 2 {
            0
        } else {
            raw_tolerant.diff_pixels
        };

        assert_eq!(raw_tolerant.diff_pixels, 2);
        assert_eq!(budgeted_tolerant, 0);
    }

    // ─── 페이지 수 검증 ───

    #[test]
    fn test_hwpspec_w_page_count() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let page_count = core.page_count();
        assert!(
            page_count >= 170,
            "hwpspec-w.hwp 페이지 수 170 이상 (실제: {})",
            page_count
        );
    }

    #[test]
    fn test_exam_math_page_count() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        let page_count = core.page_count();
        assert!(
            page_count >= 18,
            "exam_math.hwp 페이지 수 18 이상 (실제: {})",
            page_count
        );
    }

    // ─── 2단 레이아웃 검증 ───

    #[test]
    fn test_exam_math_two_column_layout() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        // 1페이지: 2단 레이아웃이어야 함
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if let Some(page) = result.pages.first() {
                assert!(
                    page.column_contents.len() >= 2,
                    "exam_math.hwp 1페이지는 2단 이상 (실제: {}단)",
                    page.column_contents.len()
                );
            }
        }
    }

    // ─── 머리말 검증 ───

    #[test]
    fn test_exam_math_no_header_on_first_page() {
        let Some(core) = load_document("samples/exam_math_no.hwp") else {
            return;
        };
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if let Some(page) = result.pages.first() {
                assert!(
                    page.active_header.is_none(),
                    "exam_math_no.hwp 1페이지에는 머리말이 없어야 함"
                );
            }
        }
    }

    #[test]
    fn test_exam_math_header_from_second_page() {
        let Some(core) = load_document("samples/exam_math_no.hwp") else {
            return;
        };
        let pages = &core.pagination;
        if let Some(result) = pages.first() {
            if result.pages.len() > 1 {
                let page2 = &result.pages[1];
                assert!(
                    page2.active_header.is_some(),
                    "exam_math_no.hwp 2페이지부터 머리말이 있어야 함"
                );
            }
        }
    }

    // ─── 표 분할(PartialTable) 검증 ───

    #[test]
    fn test_hwpspec_w_table_split() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        use crate::renderer::pagination::PageItem;
        let has_partial_table = core.pagination.iter().any(|result| {
            result.pages.iter().any(|p| {
                p.column_contents.iter().any(|cc| {
                    cc.items
                        .iter()
                        .any(|item| matches!(item, PageItem::PartialTable { .. }))
                })
            })
        });
        assert!(
            has_partial_table,
            "hwpspec-w.hwp에는 페이지 분할된 표(PartialTable)가 있어야 함"
        );
    }

    // ─── SVG 내보내기 검증 ───

    #[test]
    fn test_export_svg_produces_output() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(!svg.is_empty(), "SVG 출력이 비어있으면 안 됨");
        assert!(svg.contains("<svg"), "SVG 출력에 <svg 태그가 있어야 함");
        assert!(svg.contains("</svg>"), "SVG 출력에 </svg> 태그가 있어야 함");
    }

    #[test]
    fn test_export_svg_contains_text() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(svg.contains("<text"), "SVG에 텍스트 요소가 있어야 함");
    }

    // ─── 수식 렌더링 검증 ───

    #[test]
    fn test_equation_svg_content() {
        let Some(core) = load_document("samples/exam_math.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        let has_content = svg.contains("<path") || svg.contains("<text");
        assert!(has_content, "수식 페이지 SVG에 렌더링 요소가 있어야 함");
    }

    // ─── 다중 페이지 렌더링 회귀 테스트 ───

    #[test]
    fn test_hwpspec_w_multi_page_render() {
        let Some(core) = load_document("samples/hwpspec-w.hwp") else {
            return;
        };
        for page_idx in 0..16u32 {
            let svg = core.render_page_svg_native(page_idx).unwrap_or_default();
            assert!(!svg.is_empty(), "페이지 {} SVG가 비어있음", page_idx + 1);
        }
    }

    // ─── 문단 테두리 검증 ───

    #[test]
    fn test_1_3_paragraph_border() {
        let Some(core) = load_document("samples/1-3.hwp") else {
            return;
        };
        let svg = core.render_page_svg_native(0).unwrap_or_default();
        assert!(
            svg.contains("<rect") || svg.contains("<path"),
            "1-3.hwp에 문단 테두리/배경 렌더링 요소가 있어야 함"
        );
    }

    #[test]
    fn test_layer_svg_matches_legacy_for_basic_text_sample() {
        let Some(core) = load_document("samples/lseg-01-basic.hwp") else {
            return;
        };
        let legacy = core.render_page_svg_legacy_native(0).unwrap_or_default();
        let layered = core.render_page_svg_layer_native(0).unwrap_or_default();
        assert_eq!(
            layered, legacy,
            "layer SVG는 기본 텍스트 샘플에서 legacy SVG와 동일해야 함"
        );
    }

    #[test]
    fn test_layer_svg_matches_legacy_for_table_sample() {
        let Some(core) = load_document("samples/hwp_table_test.hwp") else {
            return;
        };
        let legacy = core.render_page_svg_legacy_native(0).unwrap_or_default();
        let layered = core.render_page_svg_layer_native(0).unwrap_or_default();
        assert_eq!(
            layered, legacy,
            "layer SVG는 표 샘플에서 legacy SVG와 동일해야 함"
        );
    }

    #[test]
    fn test_layer_svg_screenshot_matches_legacy_for_basic_text_sample() {
        assert_layer_svg_pixels_match("samples/lseg-01-basic.hwp", 0);
    }

    #[test]
    fn test_layer_svg_screenshot_matches_legacy_for_table_sample() {
        assert_layer_svg_pixels_match("samples/hwp_table_test.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_shapes() {
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::layer_renderer::LayerRenderer;
        use crate::renderer::render_tree::{
            BoundingBox, PageNode, PageRenderTree, RectangleNode, RenderNode, RenderNodeType,
        };
        use crate::renderer::skia::SkiaLayerRenderer;
        use crate::renderer::svg_layer::SvgLayerRenderer;
        use crate::renderer::ShapeStyle;

        let mut tree = PageRenderTree::new(0, 180.0, 120.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 180.0,
            height: 120.0,
            section_index: 0,
        });
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x00F6F0E6),
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(12.0, 12.0, 60.0, 40.0),
        ));
        tree.root.children.push(RenderNode::new(
            2,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x00D9E7FF),
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(90.0, 28.0, 66.0, 52.0),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        let mut svg_renderer = SvgLayerRenderer::new();
        svg_renderer.render_page(&layer_tree);
        let expected = rasterize_svg(svg_renderer.output()).expect("synthetic SVG rasterize 실패");
        let actual_png = SkiaLayerRenderer::new()
            .render_png(&layer_tree)
            .expect("synthetic Skia PNG 렌더 실패");
        let actual = decode_png(&actual_png).expect("synthetic Skia PNG decode 실패");
        let diff = diff_pixmaps(&expected, &actual, 0);

        if diff.diff_pixels > 0 {
            let (expected_path, actual_path, diff_path) = save_diff_artifacts(
                "output/skia-diff",
                "synthetic-shapes",
                0,
                "layer",
                "skia",
                "diff",
                &expected,
                &actual,
                &diff.diff_pixmap,
            );
            panic!(
                "synthetic Skia raster diff 발생: {} pixels (layer: {}, skia: {}, diff: {})",
                diff.diff_pixels,
                expected_path.display(),
                actual_path.display(),
                diff_path.display(),
            );
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_basic_text_sample() {
        assert_skia_png_matches_layer_svg("samples/lseg-01-basic.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_table_sample() {
        assert_skia_png_matches_layer_svg("samples/hwp_table_test.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_form_controls() {
        use crate::model::control::FormType;
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::layer_renderer::LayerRenderer;
        use crate::renderer::render_tree::{
            BoundingBox, FormObjectNode, PageNode, PageRenderTree, RenderNode, RenderNodeType,
        };
        use crate::renderer::skia::SkiaLayerRenderer;
        use crate::renderer::svg_layer::SvgLayerRenderer;

        let mut tree = PageRenderTree::new(0, 320.0, 180.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 320.0,
            height: 180.0,
            section_index: 0,
        });

        let forms = [
            (
                1,
                FormObjectNode {
                    form_type: FormType::PushButton,
                    caption: String::new(),
                    text: String::new(),
                    fore_color: "#000000".to_string(),
                    back_color: "#ffffff".to_string(),
                    value: 0,
                    enabled: true,
                    section_index: 0,
                    para_index: 0,
                    control_index: 0,
                    name: "button".to_string(),
                    cell_location: None,
                },
                BoundingBox::new(20.0, 20.0, 72.0, 24.0),
            ),
            (
                2,
                FormObjectNode {
                    form_type: FormType::CheckBox,
                    caption: String::new(),
                    text: String::new(),
                    fore_color: "#202020".to_string(),
                    back_color: "#ffffff".to_string(),
                    value: 1,
                    enabled: true,
                    section_index: 0,
                    para_index: 0,
                    control_index: 1,
                    name: "check".to_string(),
                    cell_location: None,
                },
                BoundingBox::new(20.0, 60.0, 110.0, 20.0),
            ),
            (
                3,
                FormObjectNode {
                    form_type: FormType::RadioButton,
                    caption: String::new(),
                    text: String::new(),
                    fore_color: "#202020".to_string(),
                    back_color: "#ffffff".to_string(),
                    value: 1,
                    enabled: true,
                    section_index: 0,
                    para_index: 0,
                    control_index: 2,
                    name: "radio".to_string(),
                    cell_location: None,
                },
                BoundingBox::new(20.0, 92.0, 110.0, 20.0),
            ),
            (
                4,
                FormObjectNode {
                    form_type: FormType::ComboBox,
                    caption: String::new(),
                    text: String::new(),
                    fore_color: "#303030".to_string(),
                    back_color: "#ffffff".to_string(),
                    value: 0,
                    enabled: true,
                    section_index: 0,
                    para_index: 0,
                    control_index: 3,
                    name: "combo".to_string(),
                    cell_location: None,
                },
                BoundingBox::new(160.0, 20.0, 110.0, 24.0),
            ),
            (
                5,
                FormObjectNode {
                    form_type: FormType::Edit,
                    caption: String::new(),
                    text: String::new(),
                    fore_color: "#303030".to_string(),
                    back_color: "#ffffff".to_string(),
                    value: 0,
                    enabled: true,
                    section_index: 0,
                    para_index: 0,
                    control_index: 4,
                    name: "edit".to_string(),
                    cell_location: None,
                },
                BoundingBox::new(160.0, 60.0, 110.0, 24.0),
            ),
        ];

        for (node_id, form, bbox) in forms {
            tree.root.children.push(RenderNode::new(
                node_id,
                RenderNodeType::FormObject(form),
                bbox,
            ));
        }

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        let mut svg_renderer = SvgLayerRenderer::new();
        svg_renderer.render_page(&layer_tree);
        let expected = rasterize_svg(svg_renderer.output()).expect("synthetic form SVG rasterize 실패");
        let actual_png = SkiaLayerRenderer::new()
            .render_png(&layer_tree)
            .expect("synthetic form Skia PNG 렌더 실패");
        let actual = decode_png(&actual_png).expect("synthetic form Skia PNG decode 실패");
        let tolerant_diff = diff_pixmaps(&expected, &actual, SKIA_TOLERANT_CHANNEL_DELTA);

        if tolerant_diff.diff_pixels > SKIA_TOLERANT_MAX_DIFF_PIXELS {
            let exact_diff = diff_pixmaps(&expected, &actual, 0);
            let (expected_path, actual_path, diff_path) = save_diff_artifacts(
                "output/skia-diff",
                "synthetic-form-controls",
                0,
                "layer",
                "skia",
                "diff",
                &expected,
                &actual,
                &exact_diff.diff_pixmap,
            );
            let (_, _, tolerant_path) = save_diff_artifacts(
                "output/skia-diff",
                "synthetic-form-controls",
                0,
                "layer",
                "skia",
                "tolerant-diff",
                &expected,
                &actual,
                &tolerant_diff.diff_pixmap,
            );
            panic!(
                "synthetic form Skia raster diff 발생: exact={} tolerant={} (budget={}) (layer: {}, skia: {}, exact diff: {}, tolerant diff: {})",
                exact_diff.diff_pixels,
                tolerant_diff.diff_pixels,
                SKIA_TOLERANT_MAX_DIFF_PIXELS,
                expected_path.display(),
                actual_path.display(),
                diff_path.display(),
                tolerant_path.display(),
            );
        }
    }

    #[test]
    fn test_get_page_layer_tree_native_populates_page_tree_cache() {
        let Some(core) = load_document("samples/lseg-01-basic.hwp") else {
            return;
        };

        assert!(
            core.page_tree_cache.borrow().is_empty(),
            "테스트 시작 시 페이지 트리 캐시는 비어 있어야 함"
        );

        core.get_page_layer_tree_native(0)
            .expect("레이어 트리 직렬화 실패");

        let cache = core.page_tree_cache.borrow();
        assert!(
            !cache.is_empty() && cache[0].is_some(),
            "레이어 트리 조회는 페이지 트리 캐시를 채워야 함"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_render_page_svg_with_fonts_respects_layer_svg_path() {
        let Some(core) = load_document("samples/lseg-01-basic.hwp") else {
            return;
        };
        let _guard = render_path_env_lock()
            .lock()
            .expect("render path env lock 획득 실패");
        std::env::set_var("RHWP_RENDER_PATH", "layer-svg");

        let layered = core
            .render_page_svg_layer_native(0)
            .expect("layer SVG 렌더 실패");
        let embedded = core
            .render_page_svg_with_fonts(0, crate::renderer::svg::FontEmbedMode::Style, &[])
            .expect("폰트 포함 SVG 렌더 실패");

        std::env::remove_var("RHWP_RENDER_PATH");

        let embedded_without_style = if let Some(style_start) = embedded.find("<style>") {
            if let Some(style_end) = embedded.find("</style>") {
                let mut normalized = embedded.clone();
                normalized.replace_range(style_start..style_end + "</style>".len(), "");
                normalized
            } else {
                embedded.clone()
            }
        } else {
            embedded.clone()
        };
        let normalize_svg = |svg: String| {
            svg.lines()
                .map(str::trim_end)
                .filter(|line| !line.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };

        assert_eq!(
            normalize_svg(embedded_without_style),
            normalize_svg(layered),
            "font-embed 경로도 layer-svg 선택을 존중해야 함"
        );
    }
}
