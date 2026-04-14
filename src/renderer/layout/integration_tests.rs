//! 레이아웃 통합 테스트
//!
//! 실제 HWP 파일을 로딩하여 페이지네이션 + 레이아웃 결과를 검증한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use resvg::{tiny_skia, usvg};
    use std::path::Path;

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
        options.fontdb_mut().load_system_fonts();
        let tree = usvg::Tree::from_str(svg, &options).ok()?;
        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        Some(pixmap)
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

        let mut diff_pixels = 0usize;
        let mut diff = tiny_skia::Pixmap::new(legacy_pixmap.width(), legacy_pixmap.height())
            .expect("diff pixmap 생성 실패");
        for (idx, (legacy_px, layered_px)) in legacy_pixmap
            .data()
            .chunks_exact(4)
            .zip(layered_pixmap.data().chunks_exact(4))
            .enumerate()
        {
            if legacy_px != layered_px {
                diff_pixels += 1;
                let base = idx * 4;
                diff.data_mut()[base..base + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }

        if diff_pixels > 0 {
            let output_dir = Path::new("output/layer-svg-diff");
            let _ = std::fs::create_dir_all(output_dir);
            let stem = Path::new(sample)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("sample");
            let legacy_path = output_dir.join(format!("{stem}-legacy-p{page_num}.png"));
            let layered_path = output_dir.join(format!("{stem}-layer-p{page_num}.png"));
            let diff_path = output_dir.join(format!("{stem}-diff-p{page_num}.png"));
            let _ = legacy_pixmap.save_png(&legacy_path);
            let _ = layered_pixmap.save_png(&layered_path);
            let _ = diff.save_png(&diff_path);
            panic!(
                "legacy/layer raster diff 발생: {} pixels (legacy: {}, layer: {}, diff: {})",
                diff_pixels,
                legacy_path.display(),
                layered_path.display(),
                diff_path.display(),
            );
        }
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
}
