//! 레이아웃 통합 테스트
//!
//! 실제 HWP 파일을 로딩하여 페이지네이션 + 레이아웃 결과를 검증한다.
//! samples/ 디렉토리에 테스트 파일이 없으면 건너뜀.

#[cfg(test)]
mod tests {
    use base64::Engine;
    use resvg::{tiny_skia, usvg};
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    const SKIA_TOLERANT_CHANNEL_DELTA: u8 = 8;
    const SKIA_TOLERANT_MAX_DIFF_PIXELS: usize = 64;
    const SKIA_RASTER_TOLERANT_NEIGHBOR_RADIUS: usize = 1;
    const SKIA_RASTER_TOLERANT_MAX_DIFF_RATIO: f64 = 0.013;
    const SKIA_INK_MASK_WHITE_DELTA: u8 = 25;
    const SKIA_INK_MASK_ALPHA_THRESHOLD: u8 = 8;
    const SKIA_INK_MASK_NEIGHBOR_RADIUS: usize = 1;
    const SKIA_INK_MASK_MAX_DIFF_RATIO: f64 = 0.003;

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
        let svg = normalize_svg_embedded_bitmaps(svg);
        let mut options = usvg::Options::default();
        let fontdb = options.fontdb_mut();
        fontdb.load_system_fonts();
        fontdb.set_sans_serif_family("Noto Sans CJK KR");
        fontdb.set_serif_family("Noto Serif CJK KR");
        fontdb.set_monospace_family("D2Coding");
        let tree = usvg::Tree::from_str(&svg, &options).ok()?;
        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        Some(pixmap)
    }

    fn normalize_svg_embedded_bitmaps(svg: &str) -> String {
        const PREFIX: &str = "href=\"data:image/bmp;base64,";

        let mut normalized = String::with_capacity(svg.len());
        let mut rest = svg;

        while let Some(start) = rest.find(PREFIX) {
            let (before, after_prefix) = rest.split_at(start);
            normalized.push_str(before);

            let after_prefix = &after_prefix[PREFIX.len()..];
            let Some(end) = after_prefix.find('"') else {
                normalized.push_str(rest);
                return normalized;
            };

            let encoded = &after_prefix[..end];
            let replacement = decode_bmp_data_uri_to_png(encoded)
                .unwrap_or_else(|| format!("data:image/bmp;base64,{encoded}"));
            normalized.push_str("href=\"");
            normalized.push_str(&replacement);
            normalized.push('"');
            rest = &after_prefix[end + 1..];
        }

        normalized.push_str(rest);
        normalized
    }

    fn decode_bmp_data_uri_to_png(encoded: &str) -> Option<String> {
        let bmp_bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()?;
        let image =
            image::load_from_memory_with_format(&bmp_bytes, image::ImageFormat::Bmp).ok()?;
        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        image.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        let png_base64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
        Some(format!("data:image/png;base64,{png_base64}"))
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

    fn pixel_max_delta(expected_px: &[u8], actual_px: &[u8]) -> u8 {
        let mut pixel_max_delta = 0u8;
        for channel in 0..4 {
            pixel_max_delta =
                pixel_max_delta.max(expected_px[channel].abs_diff(actual_px[channel]));
        }
        pixel_max_delta
    }

    fn pixel_matches_within_delta(
        expected_px: &[u8],
        actual_px: &[u8],
        ignored_channel_delta: u8,
    ) -> bool {
        pixel_max_delta(expected_px, actual_px) <= ignored_channel_delta
    }

    fn pixel_is_ink(pixel: &[u8], white_delta: u8, alpha_threshold: u8) -> bool {
        pixel[3] > alpha_threshold
            && [
                255u8.saturating_sub(pixel[0]),
                255u8.saturating_sub(pixel[1]),
                255u8.saturating_sub(pixel[2]),
            ]
            .into_iter()
            .max()
            .unwrap_or(0)
                > white_delta
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
            for channel in 0..4 {
                let delta = expected_px[channel].abs_diff(actual_px[channel]);
                total_channel_delta += u64::from(delta);
                max_channel_delta = max_channel_delta.max(delta);
            }

            let pixel_max_delta = pixel_max_delta(expected_px, actual_px);
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

    fn diff_pixmaps_with_neighborhood(
        expected: &tiny_skia::Pixmap,
        actual: &tiny_skia::Pixmap,
        ignored_channel_delta: u8,
        radius: usize,
    ) -> PixmapDiff {
        let total_pixels = (expected.width() as usize) * (expected.height() as usize);
        let mut diff_pixmap = tiny_skia::Pixmap::new(expected.width(), expected.height())
            .expect("diff pixmap 생성 실패");
        let mut diff_pixels = 0usize;
        let mut total_channel_delta = 0u64;
        let mut max_channel_delta = 0u8;
        let width = expected.width() as usize;
        let height = expected.height() as usize;
        let expected_data = expected.data();
        let actual_data = actual.data();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let base = idx * 4;
                let expected_px = &expected_data[base..base + 4];
                let actual_px = &actual_data[base..base + 4];

                for channel in 0..4 {
                    let delta = expected_px[channel].abs_diff(actual_px[channel]);
                    total_channel_delta += u64::from(delta);
                    max_channel_delta = max_channel_delta.max(delta);
                }

                let pixel_max_delta = pixel_max_delta(expected_px, actual_px);
                if pixel_max_delta <= ignored_channel_delta {
                    continue;
                }

                let mut matched = false;
                let min_y = y.saturating_sub(radius);
                let max_y = (y + radius).min(height - 1);
                let min_x = x.saturating_sub(radius);
                let max_x = (x + radius).min(width - 1);

                'search_actual: for ny in min_y..=max_y {
                    for nx in min_x..=max_x {
                        let neighbor_base = (ny * width + nx) * 4;
                        let candidate = &actual_data[neighbor_base..neighbor_base + 4];
                        if pixel_matches_within_delta(expected_px, candidate, ignored_channel_delta)
                        {
                            matched = true;
                            break 'search_actual;
                        }
                    }
                }

                if !matched {
                    'search_expected: for ny in min_y..=max_y {
                        for nx in min_x..=max_x {
                            let neighbor_base = (ny * width + nx) * 4;
                            let candidate = &expected_data[neighbor_base..neighbor_base + 4];
                            if pixel_matches_within_delta(
                                candidate,
                                actual_px,
                                ignored_channel_delta,
                            ) {
                                matched = true;
                                break 'search_expected;
                            }
                        }
                    }
                }

                if matched {
                    continue;
                }

                diff_pixels += 1;
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

    fn diff_ink_masks_with_neighborhood(
        expected: &tiny_skia::Pixmap,
        actual: &tiny_skia::Pixmap,
        white_delta: u8,
        alpha_threshold: u8,
        radius: usize,
    ) -> PixmapDiff {
        let total_pixels = (expected.width() as usize) * (expected.height() as usize);
        let mut diff_pixmap = tiny_skia::Pixmap::new(expected.width(), expected.height())
            .expect("ink mask diff pixmap 생성 실패");
        let mut diff_pixels = 0usize;
        let width = expected.width() as usize;
        let height = expected.height() as usize;
        let expected_data = expected.data();
        let actual_data = actual.data();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let base = idx * 4;
                let expected_px = &expected_data[base..base + 4];
                let actual_px = &actual_data[base..base + 4];
                let expected_ink = pixel_is_ink(expected_px, white_delta, alpha_threshold);
                let actual_ink = pixel_is_ink(actual_px, white_delta, alpha_threshold);

                if expected_ink == actual_ink {
                    continue;
                }

                let min_y = y.saturating_sub(radius);
                let max_y = (y + radius).min(height - 1);
                let min_x = x.saturating_sub(radius);
                let max_x = (x + radius).min(width - 1);
                let mut matched = false;

                if expected_ink && !actual_ink {
                    'search_actual: for ny in min_y..=max_y {
                        for nx in min_x..=max_x {
                            let neighbor_base = (ny * width + nx) * 4;
                            let candidate = &actual_data[neighbor_base..neighbor_base + 4];
                            if pixel_is_ink(candidate, white_delta, alpha_threshold) {
                                matched = true;
                                break 'search_actual;
                            }
                        }
                    }
                } else if actual_ink && !expected_ink {
                    'search_expected: for ny in min_y..=max_y {
                        for nx in min_x..=max_x {
                            let neighbor_base = (ny * width + nx) * 4;
                            let candidate = &expected_data[neighbor_base..neighbor_base + 4];
                            if pixel_is_ink(candidate, white_delta, alpha_threshold) {
                                matched = true;
                                break 'search_expected;
                            }
                        }
                    }
                }

                if matched {
                    continue;
                }

                diff_pixels += 1;
                diff_pixmap.data_mut()[base..base + 4].copy_from_slice(&[
                    if expected_ink { 255 } else { 0 },
                    0,
                    if actual_ink { 255 } else { 0 },
                    255,
                ]);
            }
        }

        PixmapDiff {
            diff_pixels,
            total_pixels,
            max_channel_delta: 0,
            mean_abs_channel_delta: 0.0,
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
    fn compare_skia_png_matches_layer_svg(sample: &str, page_num: u32) -> Result<(), String> {
        let path = Path::new(sample);
        if !path.exists() {
            return Ok(());
        }

        let data = std::fs::read(path).map_err(|err| format!("샘플 읽기 실패: {sample}: {err}"))?;
        let core = crate::document_core::DocumentCore::from_bytes(&data)
            .map_err(|err| format!("문서 파싱 실패: {sample}: {err}"))?;
        if page_num >= core.page_count() {
            return Err(format!(
                "페이지 범위 초과: {sample} requested={} page_count={}",
                page_num,
                core.page_count()
            ));
        }

        let layered_svg = core
            .render_page_svg_layer_native(page_num)
            .map_err(|err| format!("layer SVG 렌더 실패: {sample} p{page_num}: {err}"))?;
        let expected = rasterize_svg(&layered_svg)
            .ok_or_else(|| format!("layer SVG rasterize 실패: {sample} p{page_num}"))?;
        let actual_png = core
            .render_page_png_native(page_num)
            .map_err(|err| format!("Skia PNG 렌더 실패: {sample} p{page_num}: {err}"))?;
        let actual = decode_png(&actual_png)
            .ok_or_else(|| format!("Skia PNG decode 실패: {sample} p{page_num}"))?;

        if (actual.width(), actual.height()) != (expected.width(), expected.height()) {
            return Err(format!(
                "Skia/layer raster 크기 불일치: {sample} p{page_num} expected=({},{}) actual=({},{})",
                expected.width(),
                expected.height(),
                actual.width(),
                actual.height(),
            ));
        }

        let exact_diff = diff_pixmaps(&expected, &actual, 0);
        let raw_tolerant_diff = diff_pixmaps(&expected, &actual, SKIA_TOLERANT_CHANNEL_DELTA);
        let raster_tolerant_diff = diff_pixmaps_with_neighborhood(
            &expected,
            &actual,
            SKIA_TOLERANT_CHANNEL_DELTA,
            SKIA_RASTER_TOLERANT_NEIGHBOR_RADIUS,
        );
        let ink_mask_diff = diff_ink_masks_with_neighborhood(
            &expected,
            &actual,
            SKIA_INK_MASK_WHITE_DELTA,
            SKIA_INK_MASK_ALPHA_THRESHOLD,
            SKIA_INK_MASK_NEIGHBOR_RADIUS,
        );
        let raster_tolerant_ratio =
            raster_tolerant_diff.diff_pixels as f64 / raster_tolerant_diff.total_pixels as f64;
        let ink_mask_ratio = ink_mask_diff.diff_pixels as f64 / ink_mask_diff.total_pixels as f64;

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

        if ink_mask_ratio > SKIA_INK_MASK_MAX_DIFF_RATIO {
            let (expected_path, actual_path, diff_path) =
                exact_paths.expect("tolerant diff가 있으면 exact diff도 있어야 함");
            let tolerant_diff_path = tolerant_paths
                .as_ref()
                .map(|(_, _, path)| path.display().to_string())
                .unwrap_or_else(|| "-".to_string());
            let (_, _, raster_tolerant_diff_path) = save_diff_artifacts(
                "output/skia-diff",
                sample,
                page_num,
                "layer",
                "skia",
                "raster-tolerant-diff",
                &expected,
                &actual,
                &raster_tolerant_diff.diff_pixmap,
            );
            let (_, _, ink_mask_diff_path) = save_diff_artifacts(
                "output/skia-diff",
                sample,
                page_num,
                "layer",
                "skia",
                "ink-mask-diff",
                &expected,
                &actual,
                &ink_mask_diff.diff_pixmap,
            );
            return Err(format!(
                "Skia raster diff 발생: exact={} pixels, tolerant={} pixels (budget={}, ignored_channel_delta<={}), raster_tolerant={} pixels (radius={}, ratio={:.3}%, budget={:.3}%), ink_mask={} pixels (white_delta={}, alpha_threshold={}, radius={}, ratio={:.3}%, budget={:.3}%) (layer: {}, skia: {}, exact diff: {}, tolerant diff: {}, raster tolerant diff: {}, ink mask diff: {})",
                exact_diff.diff_pixels,
                raw_tolerant_diff.diff_pixels,
                SKIA_TOLERANT_MAX_DIFF_PIXELS,
                SKIA_TOLERANT_CHANNEL_DELTA,
                raster_tolerant_diff.diff_pixels,
                SKIA_RASTER_TOLERANT_NEIGHBOR_RADIUS,
                raster_tolerant_ratio * 100.0,
                SKIA_RASTER_TOLERANT_MAX_DIFF_RATIO * 100.0,
                ink_mask_diff.diff_pixels,
                SKIA_INK_MASK_WHITE_DELTA,
                SKIA_INK_MASK_ALPHA_THRESHOLD,
                SKIA_INK_MASK_NEIGHBOR_RADIUS,
                ink_mask_ratio * 100.0,
                SKIA_INK_MASK_MAX_DIFF_RATIO * 100.0,
                expected_path.display(),
                actual_path.display(),
                diff_path.display(),
                tolerant_diff_path,
                raster_tolerant_diff_path.display(),
                ink_mask_diff_path.display(),
            ));
        }

        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    fn assert_skia_png_matches_layer_svg_for_corpus(label: &str, samples: &[(String, Vec<u32>)]) {
        let mut failures = Vec::new();
        let total_pages: usize = samples.iter().map(|(_, pages)| pages.len()).sum();
        let mut completed_pages = 0usize;

        for (sample, pages) in samples {
            for &page_num in pages {
                completed_pages += 1;
                eprintln!(
                    "[skia-corpus:{label}] {completed_pages}/{total_pages} {sample} p{page_num}"
                );
                if let Err(err) = compare_skia_png_matches_layer_svg(sample, page_num) {
                    failures.push(err);
                }
            }
        }

        if !failures.is_empty() {
            panic!(
                "Skia corpus screenshot regression 실패 {}건:\n{}",
                failures.len(),
                failures.join("\n"),
            );
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    fn assert_skia_png_matches_layer_svg(sample: &str, page_num: u32) {
        if let Err(err) = compare_skia_png_matches_layer_svg(sample, page_num) {
            panic!("{err}");
        }
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    fn assert_skia_layer_tree_matches_svg(
        case_name: &str,
        layer_tree: &crate::paint::PageLayerTree,
    ) {
        use crate::renderer::layer_renderer::LayerRenderer;
        use crate::renderer::skia::SkiaLayerRenderer;
        use crate::renderer::svg_layer::SvgLayerRenderer;

        let mut svg_renderer = SvgLayerRenderer::new();
        svg_renderer.render_page(layer_tree);
        let expected = rasterize_svg(svg_renderer.output()).expect("synthetic SVG rasterize 실패");
        let actual_png = SkiaLayerRenderer::new()
            .render_png(layer_tree)
            .expect("synthetic Skia PNG 렌더 실패");
        let actual = decode_png(&actual_png).expect("synthetic Skia PNG decode 실패");
        let tolerant_diff = diff_pixmaps(&expected, &actual, SKIA_TOLERANT_CHANNEL_DELTA);
        let raster_tolerant_diff = diff_pixmaps_with_neighborhood(
            &expected,
            &actual,
            SKIA_TOLERANT_CHANNEL_DELTA,
            SKIA_RASTER_TOLERANT_NEIGHBOR_RADIUS,
        );
        let ink_mask_diff = diff_ink_masks_with_neighborhood(
            &expected,
            &actual,
            SKIA_INK_MASK_WHITE_DELTA,
            SKIA_INK_MASK_ALPHA_THRESHOLD,
            SKIA_INK_MASK_NEIGHBOR_RADIUS,
        );
        let raster_tolerant_ratio =
            raster_tolerant_diff.diff_pixels as f64 / raster_tolerant_diff.total_pixels as f64;
        let ink_mask_ratio = ink_mask_diff.diff_pixels as f64 / ink_mask_diff.total_pixels as f64;

        if ink_mask_ratio > SKIA_INK_MASK_MAX_DIFF_RATIO {
            let exact_diff = diff_pixmaps(&expected, &actual, 0);
            let (expected_path, actual_path, diff_path) = save_diff_artifacts(
                "output/skia-diff",
                case_name,
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
                case_name,
                0,
                "layer",
                "skia",
                "tolerant-diff",
                &expected,
                &actual,
                &tolerant_diff.diff_pixmap,
            );
            let (_, _, raster_tolerant_path) = save_diff_artifacts(
                "output/skia-diff",
                case_name,
                0,
                "layer",
                "skia",
                "raster-tolerant-diff",
                &expected,
                &actual,
                &raster_tolerant_diff.diff_pixmap,
            );
            let (_, _, ink_mask_path) = save_diff_artifacts(
                "output/skia-diff",
                case_name,
                0,
                "layer",
                "skia",
                "ink-mask-diff",
                &expected,
                &actual,
                &ink_mask_diff.diff_pixmap,
            );
            panic!(
                "synthetic Skia raster diff 발생: exact={} tolerant={} (budget={}), raster_tolerant={} (radius={}, ratio={:.3}%, budget={:.3}%), ink_mask={} (white_delta={}, alpha_threshold={}, radius={}, ratio={:.3}%, budget={:.3}%) (layer: {}, skia: {}, exact diff: {}, tolerant diff: {}, raster tolerant diff: {}, ink mask diff: {})",
                exact_diff.diff_pixels,
                tolerant_diff.diff_pixels,
                SKIA_TOLERANT_MAX_DIFF_PIXELS,
                raster_tolerant_diff.diff_pixels,
                SKIA_RASTER_TOLERANT_NEIGHBOR_RADIUS,
                raster_tolerant_ratio * 100.0,
                SKIA_RASTER_TOLERANT_MAX_DIFF_RATIO * 100.0,
                ink_mask_diff.diff_pixels,
                SKIA_INK_MASK_WHITE_DELTA,
                SKIA_INK_MASK_ALPHA_THRESHOLD,
                SKIA_INK_MASK_NEIGHBOR_RADIUS,
                ink_mask_ratio * 100.0,
                SKIA_INK_MASK_MAX_DIFF_RATIO * 100.0,
                expected_path.display(),
                actual_path.display(),
                diff_path.display(),
                tolerant_path.display(),
                raster_tolerant_path.display(),
                ink_mask_path.display(),
            );
        }
    }

    fn synthetic_png_bytes() -> Vec<u8> {
        let mut pixmap = tiny_skia::Pixmap::new(40, 30).expect("synthetic pixmap 생성 실패");
        for y in 0..30usize {
            for x in 0..40usize {
                let (r, g, b) = match (x < 20, y < 15) {
                    (true, true) => (255, 32, 32),
                    (false, true) => (32, 200, 64),
                    (true, false) => (48, 96, 255),
                    (false, false) => (255, 200, 32),
                };
                let base = (y * 40 + x) * 4;
                pixmap.data_mut()[base..base + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
        pixmap.encode_png().expect("synthetic png 인코딩 실패")
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

    #[test]
    fn test_diff_pixmaps_with_neighborhood_ignores_one_pixel_shift() {
        let mut expected = tiny_skia::Pixmap::new(5, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(5, 1).expect("actual pixmap 생성 실패");

        expected
            .data_mut()
            .copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        actual
            .data_mut()
            .copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0]);

        let diff = diff_pixmaps_with_neighborhood(&expected, &actual, 8, 1);
        assert_eq!(diff.diff_pixels, 0);
    }

    #[test]
    fn test_diff_pixmaps_with_neighborhood_preserves_large_shift() {
        let mut expected = tiny_skia::Pixmap::new(10, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(10, 1).expect("actual pixmap 생성 실패");

        expected.data_mut().copy_from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        actual.data_mut().copy_from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0,
            255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        let diff = diff_pixmaps_with_neighborhood(&expected, &actual, 8, 1);
        assert!(diff.diff_pixels > 0);
    }

    #[test]
    fn test_diff_ink_masks_ignores_antialias_coverage_difference() {
        let mut expected = tiny_skia::Pixmap::new(3, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(3, 1).expect("actual pixmap 생성 실패");

        expected
            .data_mut()
            .copy_from_slice(&[255, 255, 255, 255, 235, 235, 235, 255, 0, 0, 0, 255]);
        actual
            .data_mut()
            .copy_from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255]);

        let diff = diff_ink_masks_with_neighborhood(&expected, &actual, 25, 8, 1);
        assert_eq!(diff.diff_pixels, 0);
    }

    #[test]
    fn test_diff_ink_masks_ignores_one_pixel_shift() {
        let mut expected = tiny_skia::Pixmap::new(5, 1).expect("expected pixmap 생성 실패");
        let mut actual = tiny_skia::Pixmap::new(5, 1).expect("actual pixmap 생성 실패");

        expected.data_mut().copy_from_slice(&[
            255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255,
        ]);
        actual.data_mut().copy_from_slice(&[
            255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255,
            255, 255,
        ]);

        let diff = diff_ink_masks_with_neighborhood(&expected, &actual, 25, 8, 1);
        assert_eq!(diff.diff_pixels, 0);
    }

    #[test]
    fn test_diff_ink_masks_preserves_missing_shape() {
        let mut expected = tiny_skia::Pixmap::new(3, 1).expect("expected pixmap 생성 실패");
        let actual = tiny_skia::Pixmap::new(3, 1).expect("actual pixmap 생성 실패");

        expected
            .data_mut()
            .copy_from_slice(&[255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255]);

        let diff = diff_ink_masks_with_neighborhood(&expected, &actual, 25, 8, 1);
        assert!(diff.diff_pixels > 0);
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
        use crate::renderer::render_tree::{
            BoundingBox, PageNode, PageRenderTree, RectangleNode, RenderNode, RenderNodeType,
        };
        use crate::renderer::ShapeStyle;
        use crate::renderer::{GradientFillInfo, PatternFillInfo};

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
                    pattern: Some(PatternFillInfo {
                        pattern_type: 4,
                        pattern_color: 0x003D6FB6,
                        background_color: 0x00D9E7FF,
                    }),
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(90.0, 28.0, 66.0, 52.0),
        ));
        tree.root.children.push(RenderNode::new(
            3,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x00C3D7AF),
                    ..Default::default()
                },
                Some(Box::new(GradientFillInfo {
                    gradient_type: 1,
                    angle: 0,
                    center_x: 50,
                    center_y: 50,
                    colors: vec![0x00EEF4E8, 0x00A9C47F, 0x00839A6B],
                    positions: vec![0.0, 0.65, 1.0],
                })),
            )),
            BoundingBox::new(24.0, 68.0, 64.0, 40.0),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        assert_skia_layer_tree_matches_svg("synthetic-shapes", &layer_tree);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_shape_group_sample() {
        assert_skia_png_matches_layer_svg("samples/shape-group-02.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_table_vpos_sample() {
        assert_skia_png_matches_layer_svg("samples/table-vpos-01.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_img_start_sample() {
        assert_skia_png_matches_layer_svg("samples/img-start-001.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_pic_in_head_sample() {
        assert_skia_png_matches_layer_svg("samples/pic-in-head-02.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_footnote_sample() {
        assert_skia_png_matches_layer_svg("samples/footnote-01.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_endnote_sample() {
        assert_skia_png_matches_layer_svg("samples/endnote-01.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_equation_layout() {
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::svg_render::render_equation_svg;
        use crate::renderer::equation::tokenizer::tokenize;
        use crate::renderer::render_tree::{
            BoundingBox, EquationNode, PageNode, PageRenderTree, RenderNode, RenderNodeType,
        };

        let font_size = 22.0;
        let ast = EqParser::new(tokenize(
            "SUM _{i=1} ^{n} LEFT ( x_i ^2 + y_i ^2 RIGHT ) over SQRT {n}",
        ))
        .parse();
        let layout_box = EqLayout::new(font_size).layout(&ast);
        let svg_content = render_equation_svg(&layout_box, "#000000", font_size);

        let mut tree = PageRenderTree::new(0, 320.0, 140.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 320.0,
            height: 140.0,
            section_index: 0,
        });
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::Equation(EquationNode {
                svg_content,
                layout_box: layout_box.clone(),
                color_str: "#000000".to_string(),
                color: 0x00000000,
                font_size,
                section_index: Some(0),
                para_index: Some(0),
                control_index: Some(0),
                cell_index: None,
                cell_para_index: None,
            }),
            BoundingBox::new(18.0, 24.0, layout_box.width, layout_box.height),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        assert_skia_layer_tree_matches_svg("synthetic-equation-layout", &layer_tree);
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
    fn test_skia_screenshot_matches_layer_svg_for_equation_sample() {
        assert_skia_png_matches_layer_svg("samples/eq-01.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_picture_crop_sample() {
        assert_skia_png_matches_layer_svg("samples/pic-crop-01.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_draw_group_sample() {
        assert_skia_png_matches_layer_svg("samples/draw-group.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_hwp_3_0_hwpml_sample() {
        assert_skia_png_matches_layer_svg("samples/hwp-3.0-HWPML.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_hwpspec_sample() {
        assert_skia_png_matches_layer_svg("samples/hwpspec.hwp", 0);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_representative_sample_corpus() {
        let samples = vec![
            ("samples/lseg-02-mixed.hwp".to_string(), vec![0]),
            ("samples/lseg-03-spacing.hwp".to_string(), vec![0]),
            ("samples/field-01.hwp".to_string(), vec![0]),
            ("samples/form-01.hwp".to_string(), vec![0]),
            ("samples/eq-01.hwp".to_string(), vec![0]),
            ("samples/pic-crop-01.hwp".to_string(), vec![0]),
            ("samples/table-001.hwp".to_string(), vec![0]),
            ("samples/table-complex.hwp".to_string(), vec![0]),
            ("samples/group-drawing-02.hwp".to_string(), vec![0]),
            ("samples/hwp-img-001.hwp".to_string(), vec![0]),
            ("samples/biz_plan.hwp".to_string(), vec![0]),
        ];

        assert_skia_png_matches_layer_svg_for_corpus("representative", &samples);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    #[ignore = "expensive full sample sweep"]
    fn test_skia_screenshot_matches_layer_svg_for_full_sample_corpus() {
        let mut samples = Vec::new();
        for entry in std::fs::read_dir("samples").expect("samples 디렉터리 읽기 실패") {
            let entry = entry.expect("samples 디렉터리 항목 읽기 실패");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let extension = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .unwrap_or_default();
            if !matches!(extension.as_str(), "hwp" | "hwpx") {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if matches!(file_name, "loading-fail-01.hwp") {
                continue;
            }

            samples.push((path.to_string_lossy().to_string(), vec![0]));
        }
        samples.sort_by(|left, right| left.0.cmp(&right.0));

        assert_skia_png_matches_layer_svg_for_corpus("full", &samples);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_form_controls() {
        use crate::model::control::FormType;
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::render_tree::{
            BoundingBox, FormObjectNode, PageNode, PageRenderTree, RenderNode, RenderNodeType,
        };

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
        assert_skia_layer_tree_matches_svg("synthetic-form-controls", &layer_tree);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_page_background_image() {
        use crate::model::style::ImageFillMode;
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::render_tree::{
            BoundingBox, PageBackgroundImage, PageBackgroundNode, PageNode, PageRenderTree,
            RenderNode, RenderNodeType,
        };

        let mut tree = PageRenderTree::new(0, 160.0, 120.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 160.0,
            height: 120.0,
            section_index: 0,
        });
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::PageBackground(PageBackgroundNode {
                background_color: Some(0x00FFFFFF),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: Some(PageBackgroundImage {
                    data: synthetic_png_bytes(),
                    fill_mode: ImageFillMode::FitToSize,
                }),
            }),
            BoundingBox::new(0.0, 0.0, 160.0, 120.0),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        assert_skia_layer_tree_matches_svg("synthetic-page-background-image", &layer_tree);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn test_skia_screenshot_matches_layer_svg_for_synthetic_image_fill_modes() {
        use crate::model::image::ImageEffect;
        use crate::model::style::ImageFillMode;
        use crate::paint::{LayerBuilder, RenderProfile};
        use crate::renderer::render_tree::{
            BoundingBox, ImageNode, PageNode, PageRenderTree, RenderNode, RenderNodeType,
            ShapeTransform,
        };

        let png_bytes = synthetic_png_bytes();
        let mut tree = PageRenderTree::new(0, 240.0, 160.0);
        tree.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 240.0,
            height: 160.0,
            section_index: 0,
        });

        let mut cropped = ImageNode::new(0, Some(png_bytes.clone()));
        cropped.fill_mode = Some(ImageFillMode::FitToSize);
        cropped.crop = Some((20 * 75, 0, 40 * 75, 30 * 75));
        cropped.transform = ShapeTransform::default();
        cropped.effect = ImageEffect::RealPic;
        tree.root.children.push(RenderNode::new(
            1,
            RenderNodeType::Image(cropped),
            BoundingBox::new(16.0, 16.0, 96.0, 72.0),
        ));

        let mut centered = ImageNode::new(0, Some(png_bytes.clone()));
        centered.fill_mode = Some(ImageFillMode::CenterBottom);
        centered.original_size = Some((40.0, 30.0));
        centered.transform = ShapeTransform::default();
        centered.effect = ImageEffect::RealPic;
        tree.root.children.push(RenderNode::new(
            2,
            RenderNodeType::Image(centered),
            BoundingBox::new(132.0, 16.0, 80.0, 72.0),
        ));

        let mut tiled = ImageNode::new(0, Some(png_bytes));
        tiled.fill_mode = Some(ImageFillMode::TileAll);
        tiled.original_size = Some((20.0, 15.0));
        tiled.transform = ShapeTransform::default();
        tiled.effect = ImageEffect::RealPic;
        tree.root.children.push(RenderNode::new(
            3,
            RenderNodeType::Image(tiled),
            BoundingBox::new(16.0, 100.0, 196.0, 44.0),
        ));

        let mut builder = LayerBuilder::new(RenderProfile::Screen);
        let layer_tree = builder.build(&tree);
        assert_skia_layer_tree_matches_svg("synthetic-image-fill-modes", &layer_tree);
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
