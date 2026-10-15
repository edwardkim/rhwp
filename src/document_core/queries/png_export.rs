//! 다쪽 PNG 내보내기의 글꼴 공급 snapshot을 한 번 적재하고 재사용한다.

use crate::document_core::DocumentCore;
use crate::paint::RenderProfile;
use crate::renderer::layer_renderer::{LayerRasterRenderer, RasterRenderOptions};
use crate::renderer::skia::SkiaLayerRenderer;
use crate::HwpError;

use super::rendering::PngExportOptions;

/// 같은 내보내기 명령의 모든 페이지가 동일한 글꼴 공급을 사용한다.
/// 새 명령은 새 snapshot을 만든다. 문서의 내장 글꼴은 매 페이지 레이어의 자원을 소비한다.
pub struct NativePngExporter {
    profile: RenderProfile,
    options: PngExportOptions,
    renderer: SkiaLayerRenderer,
}

impl NativePngExporter {
    /// 옵션 없는 기존 screen PNG의 시스템 글꼴 경로를 유지한다.
    pub fn system_defaults() -> Self {
        Self {
            profile: RenderProfile::Screen,
            options: PngExportOptions::default(),
            renderer: SkiaLayerRenderer::new(),
        }
    }

    pub fn new(profile: RenderProfile, options: PngExportOptions) -> Self {
        let renderer = SkiaLayerRenderer::new().with_font_paths(&options.font_paths);
        Self {
            profile,
            options,
            renderer,
        }
    }

    pub fn render_page(&self, document: &DocumentCore, page_num: u32) -> Result<Vec<u8>, HwpError> {
        let options = &self.options;
        let layer_tree = document.build_page_layer_tree_with_profile(page_num, self.profile)?;

        // 페이지 크기에서 effective scale + max_dimension 결정
        let mut raster_options = RasterRenderOptions::default();

        // VLM 프리셋 적용 (longest edge 한도 + 픽셀 수 한도)
        let mut effective_max_dim: Option<i32> = options.max_dimension;
        let mut effective_max_pixels: Option<u64> = None;
        if let Some(target) = options.vlm_target {
            let (longest_edge, max_pixels) = target.constraints();
            // 명시 max_dimension 이 없으면 프리셋 한도 사용
            effective_max_dim.get_or_insert(longest_edge);
            effective_max_pixels = Some(max_pixels);
        }

        // scale 결정 우선순위:
        // 1. 명시 scale (사용자 직접 지정)
        // 2. max_dimension / VLM 기반 자동 계산
        // 3. --dpi 만 지정 시 scale = dpi / 96.0 (#614)
        // 4. 기본 1.0
        let scale = if let Some(s) = options.scale {
            s
        } else if effective_max_dim.is_some() || effective_max_pixels.is_some() {
            let mut auto_scale: f64 = 1.0;
            // longest edge 한도
            if let Some(max_dim) = effective_max_dim {
                let longest_page_edge = layer_tree.page_width.max(layer_tree.page_height);
                if longest_page_edge > 0.0 {
                    auto_scale = auto_scale.min(max_dim as f64 / longest_page_edge);
                }
            }
            // 픽셀 수 한도 (ceil + 부동소수점 오차 안전 마진 0.5%)
            if let Some(max_pixels) = effective_max_pixels {
                let page_pixels = layer_tree.page_width * layer_tree.page_height;
                if page_pixels > 0.0 {
                    let pixel_scale = (max_pixels as f64 / page_pixels).sqrt() * 0.995;
                    auto_scale = auto_scale.min(pixel_scale);
                }
            }
            // 1.0 초과 시는 페이지가 이미 충분히 작은 것이므로 1.0 으로 cap
            auto_scale.min(1.0).max(0.1)
        } else if let Some(dpi) = options.dpi {
            (dpi / 96.0).max(0.1)
        } else {
            1.0
        };

        raster_options.scale = scale;
        raster_options.dpi = options.dpi;
        if let Some(max_dim) = effective_max_dim {
            raster_options.max_dimension = max_dim;
        }
        if let Some(max_pixels) = effective_max_pixels {
            raster_options.max_pixels = max_pixels;
        }

        let png_bytes = self
            .renderer
            .render_png_with_options(&layer_tree, raster_options)?;

        if let Some(dpi) = options.dpi {
            Ok(super::rendering::inject_png_phys(png_bytes, dpi))
        } else {
            Ok(png_bytes)
        }
    }
}
