//! 렌더링/페이지 정보/구성/페이지네이션/페이지 트리 관련 native 메서드

use super::super::helpers::color_ref_to_css;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::page::ColumnDef;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::paint::{LayerBuilder, LayerOutputOptions, PageLayerTree, RenderProfile};
use crate::renderer::canvas::CanvasRenderer;
use crate::renderer::composer::{compose_paragraph, compose_section, ComposedParagraph};
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredSection, MeasuredTable};
use crate::renderer::html::HtmlRenderer;
use crate::renderer::layer_renderer::LayerRenderer;
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::pagination::{MasterPageRef, PageContent, PaginationResult, Paginator};
use crate::renderer::render_tree::PageRenderTree;
use crate::renderer::style_resolver::resolve_styles;
use crate::renderer::svg::SvgRenderer;
use crate::renderer::svg_layer::SvgLayerRenderer;
use std::cell::RefCell;
use std::fmt::Write as _;

fn uses_hwp3_origin_page_tolerance(document: &Document) -> bool {
    let total_paragraphs: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    if total_paragraphs <= 50 {
        return false;
    }

    let para_shape_ratio = document.doc_info.para_shapes.len() as f64 / total_paragraphs as f64;
    let char_shape_ratio = document.doc_info.char_shapes.len() as f64 / total_paragraphs as f64;
    para_shape_ratio < 0.05 && char_shape_ratio < 0.15
}

fn uses_hwp3_origin_flow_spacing_before(document: &Document) -> bool {
    // HWP3-origin HWP5 변환본은 parser 단계에서 ParaShape spacing 계열을 절반으로
    // 정규화하므로, 본문 흐름 계산에서는 원래 spacing_before를 복원한다.
    // 원본 HWP3는 HWP3 parser가 만든 spacing 값을 기준으로 삼아 여기서 재확대하지 않는다.
    document.is_hwp3_variant
}

fn should_insert_hwp3_title_filler_page(
    hwp3_origin_page_tolerance: bool,
    section_index: usize,
    section: &Section,
    carry_last_page_number: u32,
) -> bool {
    if !hwp3_origin_page_tolerance
        || section_index == 0
        || carry_last_page_number == 0
        || carry_last_page_number.is_multiple_of(2)
    {
        return false;
    }

    let Some(first_para) = section.paragraphs.first() else {
        return false;
    };

    let title_section_flags = section.section_def.flags & 0x3 == 0x3
        || first_para.controls.iter().any(|control| {
            matches!(
                control,
                Control::SectionDef(section_def) if section_def.flags & 0x3 == 0x3
            )
        });
    let has_first_page_hide = first_para.controls.iter().any(|control| {
        matches!(
            control,
            Control::PageHide(hide) if hide.hide_header && hide.hide_page_num
        )
    });
    let has_full_page_figure = first_para.controls.iter().any(|control| match control {
        Control::Picture(picture) => {
            !picture.common.treat_as_char
                && picture.common.width >= section.section_def.page_def.width * 9 / 10
                && picture.common.height >= section.section_def.page_def.height * 9 / 10
        }
        Control::Shape(shape) => {
            let common = shape.common();
            !common.treat_as_char
                && common.width >= section.section_def.page_def.width * 9 / 10
                && common.height >= section.section_def.page_def.height * 9 / 10
        }
        _ => false,
    });
    let has_early_explicit_page_break = section.paragraphs.iter().take(40).any(|para| {
        matches!(
            para.column_type,
            ColumnBreakType::Page | ColumnBreakType::Section
        )
    });

    title_section_flags
        && has_first_page_hide
        && has_full_page_figure
        && has_early_explicit_page_break
}

fn insert_hwp3_title_filler_page(result: &mut PaginationResult, section_index: usize) {
    let Some(first_page) = result.pages.first() else {
        return;
    };

    let blank = PageContent {
        page_index: 0,
        page_number: 1,
        section_index,
        layout: first_page.layout.clone(),
        column_contents: Vec::new(),
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    for (page_idx, page) in result.pages.iter_mut().enumerate() {
        page.page_index = (page_idx + 1) as u32;
        page.page_number = page.page_number.saturating_add(1);
    }
    result.pages.insert(0, blank);
}

fn assign_master_pages_for_section(
    result: &mut PaginationResult,
    section_index: usize,
    section: &Section,
    carry_master_odd: &Option<MasterPageRef>,
    carry_master_even: &Option<MasterPageRef>,
) {
    use crate::model::header_footer::HeaderFooterApply;

    for page in &mut result.pages {
        page.active_master_page = None;
        page.extra_master_pages.clear();
    }

    let mps = &section.section_def.master_pages;
    if mps.is_empty() {
        return;
    }

    let base_mp_indices: Vec<usize> = mps
        .iter()
        .enumerate()
        .filter(|(_, m)| !m.is_extension)
        .map(|(i, _)| i)
        .collect();
    let single_base_mp = if base_mp_indices.len() == 1 {
        base_mp_indices.first().copied()
    } else {
        None
    };
    let mp_both = base_mp_indices
        .iter()
        .copied()
        .find(|&i| mps[i].apply_to == HeaderFooterApply::Both);
    let mp_odd = base_mp_indices
        .iter()
        .copied()
        .find(|&i| mps[i].apply_to == HeaderFooterApply::Odd);
    let mp_even = base_mp_indices
        .iter()
        .copied()
        .find(|&i| mps[i].apply_to == HeaderFooterApply::Even);
    let ext_mp_indices: Vec<usize> = mps
        .iter()
        .enumerate()
        .filter(|(_, m)| m.is_extension)
        .map(|(i, _)| i)
        .collect();

    let section_page_count = result.pages.len();
    for (page_idx_in_section, page) in result.pages.iter_mut().enumerate() {
        let is_last = page_idx_in_section + 1 == section_page_count;
        let is_first_page = page_idx_in_section == 0;

        if is_first_page && section.section_def.hide_master_page {
            continue;
        }

        let selected = if page.page_number % 2 == 1 {
            mp_odd
                .or(mp_both)
                .map(|mi| MasterPageRef {
                    section_index,
                    master_page_index: mi,
                })
                .or_else(|| carry_master_odd.clone())
                .or_else(|| {
                    single_base_mp.map(|mi| MasterPageRef {
                        section_index,
                        master_page_index: mi,
                    })
                })
        } else {
            mp_even
                .or(mp_both)
                .map(|mi| MasterPageRef {
                    section_index,
                    master_page_index: mi,
                })
                .or_else(|| carry_master_even.clone())
                .or_else(|| {
                    single_base_mp.map(|mi| MasterPageRef {
                        section_index,
                        master_page_index: mi,
                    })
                })
        };
        page.active_master_page = selected;

        if is_last && !ext_mp_indices.is_empty() {
            let replace_exts: Vec<usize> = ext_mp_indices
                .iter()
                .filter(|&&i| !mps[i].overlap || mps[i].replace_base)
                .copied()
                .collect();
            let overlap_exts: Vec<usize> = ext_mp_indices
                .iter()
                .filter(|&&i| mps[i].overlap && !mps[i].replace_base)
                .copied()
                .collect();

            if let Some(&replace_idx) = replace_exts.last() {
                page.active_master_page = Some(MasterPageRef {
                    section_index,
                    master_page_index: replace_idx,
                });
            }

            let active_apply = page
                .active_master_page
                .as_ref()
                .and_then(|mp_ref| {
                    if mp_ref.section_index == section_index {
                        mps.get(mp_ref.master_page_index)
                    } else {
                        None
                    }
                })
                .map(|m| m.apply_to);
            let mut remaining_overlap_exts: Vec<usize> = Vec::new();
            for &i in &overlap_exts {
                if Some(mps[i].apply_to) == active_apply {
                    page.active_master_page = Some(MasterPageRef {
                        section_index,
                        master_page_index: i,
                    });
                } else {
                    remaining_overlap_exts.push(i);
                }
            }
            if !remaining_overlap_exts.is_empty() {
                page.extra_master_pages = remaining_overlap_exts
                    .iter()
                    .map(|&mi| MasterPageRef {
                        section_index,
                        master_page_index: mi,
                    })
                    .collect();
            }
        }
    }
}

fn update_master_page_carry_from_section(
    section_index: usize,
    section: &Section,
    carry_master_odd: &mut Option<MasterPageRef>,
    carry_master_even: &mut Option<MasterPageRef>,
) {
    use crate::model::header_footer::HeaderFooterApply;

    for (master_page_index, master_page) in section.section_def.master_pages.iter().enumerate() {
        if master_page.is_extension {
            continue;
        }
        let master_ref = MasterPageRef {
            section_index,
            master_page_index,
        };
        match master_page.apply_to {
            HeaderFooterApply::Both => {
                *carry_master_odd = Some(master_ref.clone());
                *carry_master_even = Some(master_ref);
            }
            HeaderFooterApply::Odd => *carry_master_odd = Some(master_ref),
            HeaderFooterApply::Even => *carry_master_even = Some(master_ref),
        }
    }
}

fn apply_page_number_layouts_for_section(result: &mut PaginationResult, section: &Section) {
    let page_def = &section.section_def.page_def;
    for page in &mut result.pages {
        page.layout
            .apply_page_number_margins(page_def, page.page_number);
        for column in &mut page.column_contents {
            if let Some(zone_layout) = &mut column.zone_layout {
                zone_layout.apply_page_number_margins(page_def, page.page_number);
            }
        }
    }
}

impl DocumentCore {
    /// 페이지 렌더 트리를 생성하여 반환한다 (native bridge / 외부 렌더러용).
    pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        Ok(tree)
    }

    /// 페이지 레이어 트리를 생성하여 반환한다 (native bridge / backend replay용).
    pub fn build_page_layer_tree(&self, page_num: u32) -> Result<PageLayerTree, HwpError> {
        let tree = self.build_page_tree_cached(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let output_options = LayerOutputOptions {
            show_paragraph_marks: self.show_paragraph_marks,
            show_control_codes: self.show_control_codes,
            show_transparent_borders: self.show_transparent_borders,
            clip_enabled: self.clip_enabled,
            debug_overlay: self.debug_overlay,
        };
        let mut builder =
            LayerBuilder::new(RenderProfile::Screen).with_output_options(output_options);
        Ok(builder.build(&tree))
    }

    /// 바이너리 데이터를 0-based `bin_data_content` 인덱스로 반환한다.
    pub fn get_bin_data(&self, index: usize) -> Option<&[u8]> {
        self.document
            .bin_data_content
            .get(index)
            .map(|b| b.data.as_slice())
    }

    pub fn render_page_svg_native(&self, page_num: u32) -> Result<String, HwpError> {
        if matches!(
            std::env::var("RHWP_RENDER_PATH").ok().as_deref(),
            Some("layer-svg")
        ) {
            return self.render_page_svg_layer_native(page_num);
        }
        self.render_page_svg_legacy_native(page_num)
    }

    pub fn render_page_svg_legacy_native(&self, page_num: u32) -> Result<String, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut renderer = SvgRenderer::new();
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.debug_overlay = self.debug_overlay;
        renderer.render_tree(&tree);
        Ok(renderer.output().to_string())
    }

    pub fn render_page_svg_layer_native(&self, page_num: u32) -> Result<String, HwpError> {
        let layer_tree = self.build_page_layer_tree(page_num)?;
        let mut renderer = SvgLayerRenderer::new();
        renderer.inner_mut().show_paragraph_marks = self.show_paragraph_marks;
        renderer.inner_mut().show_control_codes = self.show_control_codes;
        renderer.inner_mut().debug_overlay = self.debug_overlay;
        renderer.render_page(&layer_tree)?;
        Ok(renderer.output().to_string())
    }

    /// PDF export for one page, implemented as the current compatibility path:
    /// SVG render output -> svg2pdf.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_page_pdf_native(&self, page_num: u32) -> Result<Vec<u8>, HwpError> {
        self.render_pages_pdf_native(&[page_num])
    }

    /// PDF export for an explicit 0-based page selection.
    ///
    /// This keeps PDF as a native/export API surface rather than a WASM render path.
    /// The implementation intentionally reuses the SVG compatibility output until a
    /// direct/vector PDF backend is introduced.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_pages_pdf_native(&self, page_nums: &[u32]) -> Result<Vec<u8>, HwpError> {
        if page_nums.is_empty() {
            return Err(HwpError::RenderError(
                "PDF export requires at least one page".to_string(),
            ));
        }

        let mut svg_pages = Vec::with_capacity(page_nums.len());
        for &page_num in page_nums {
            svg_pages.push(self.render_page_svg_native(page_num)?);
        }
        crate::renderer::pdf::svgs_to_pdf(&svg_pages).map_err(HwpError::RenderError)
    }

    /// PDF export for the full document.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_document_pdf_native(&self) -> Result<Vec<u8>, HwpError> {
        let pages: Vec<u32> = (0..self.page_count()).collect();
        self.render_pages_pdf_native(&pages)
    }

    /// SVG 렌더링 (폰트 임베딩 옵션 포함)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_page_svg_with_fonts(
        &self,
        page_num: u32,
        font_embed_mode: crate::renderer::svg::FontEmbedMode,
        font_paths: &[std::path::PathBuf],
    ) -> Result<String, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut renderer = SvgRenderer::new();
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.debug_overlay = self.debug_overlay;
        renderer.font_embed_mode = font_embed_mode;
        renderer.font_paths = font_paths.to_vec();
        renderer.render_tree(&tree);

        // 폰트 임베딩 후처리
        let mut svg = renderer.output().to_string();
        if font_embed_mode != crate::renderer::svg::FontEmbedMode::None {
            let style_css = crate::renderer::svg::generate_font_style(&renderer, font_paths);
            if !style_css.is_empty() {
                // <svg ...> 직후에 <style> 삽입
                if let Some(pos) = svg.find('>') {
                    let insert = format!("\n<style>\n{}</style>\n", style_css);
                    svg.insert_str(pos + 1, &insert);
                }
            }
        }
        Ok(svg)
    }

    /// HTML 렌더링 (네이티브 에러 타입)
    pub fn render_page_html_native(&self, page_num: u32) -> Result<String, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut renderer = HtmlRenderer::new();
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.render_tree(&tree);
        Ok(renderer.output().to_string())
    }

    /// Canvas 렌더링 (네이티브 에러 타입)
    pub fn render_page_canvas_native(&self, page_num: u32) -> Result<u32, HwpError> {
        let tree = self.build_page_layer_tree(page_num)?;
        let mut renderer = CanvasRenderer::new();
        renderer.render_page(&tree)?;
        Ok(renderer.command_count() as u32)
    }

    pub fn render_page_canvas_legacy_native(&self, page_num: u32) -> Result<u32, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut renderer = CanvasRenderer::new();
        renderer.render_tree(&tree);
        Ok(renderer.command_count() as u32)
    }

    pub fn get_canvaskit_replay_plan_native(
        &self,
        page_num: u32,
        mode: &str,
    ) -> Result<String, HwpError> {
        use crate::renderer::canvaskit_policy::{
            analyze_canvaskit_replay_plan, CanvasKitReplayMode,
        };

        let mode = CanvasKitReplayMode::from_str(mode).ok_or_else(|| {
            HwpError::RenderError(format!(
                "지원하지 않는 CanvasKit replay mode입니다: {mode}. allowed modes: default, compat"
            ))
        })?;
        let tree = self.build_page_layer_tree(page_num)?;
        let plan = analyze_canvaskit_replay_plan(&tree, mode);
        serde_json::to_string(&plan).map_err(|error| {
            HwpError::RenderError(format!(
                "CanvasKit replay plan JSON 직렬화에 실패했습니다: {error}"
            ))
        })
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    pub fn render_page_png_native(&self, page_num: u32) -> Result<Vec<u8>, HwpError> {
        use crate::renderer::layer_renderer::LayerRasterRenderer;
        use crate::renderer::skia::SkiaLayerRenderer;

        let layer_tree = self.build_page_layer_tree(page_num)?;
        SkiaLayerRenderer::new().render_png(&layer_tree)
    }

    /// 사용자 지정 폰트 경로를 포함한 PNG 렌더링. SVG 의 `--font-path` 와 동일 패턴.
    /// ttfs 디렉토리의 한컴 전용 폰트 (HY견명조 등) 가 시스템 fontconfig 에 없을 때 사용.
    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    pub fn render_page_png_native_with_fonts(
        &self,
        page_num: u32,
        font_paths: &[std::path::PathBuf],
    ) -> Result<Vec<u8>, HwpError> {
        use crate::renderer::layer_renderer::LayerRasterRenderer;
        use crate::renderer::skia::SkiaLayerRenderer;

        let layer_tree = self.build_page_layer_tree(page_num)?;
        SkiaLayerRenderer::new()
            .with_font_paths(font_paths)
            .render_png(&layer_tree)
    }

    /// 옵션 (scale / max-dimension / VLM 프리셋 / font_paths) 적용 PNG 렌더링.
    /// AI 파이프라인 + VLM (Vision-Language Model) 연동 사용 사례용.
    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    pub fn render_page_png_native_with_export_options(
        &self,
        page_num: u32,
        options: &PngExportOptions,
    ) -> Result<Vec<u8>, HwpError> {
        use crate::renderer::layer_renderer::{LayerRasterRenderer, RasterRenderOptions};
        use crate::renderer::skia::SkiaLayerRenderer;

        let layer_tree = self.build_page_layer_tree(page_num)?;

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

        let png_bytes = SkiaLayerRenderer::new()
            .with_font_paths(&options.font_paths)
            .render_png_with_options(&layer_tree, raster_options)?;

        if let Some(dpi) = options.dpi {
            Ok(inject_png_phys(png_bytes, dpi))
        } else {
            Ok(png_bytes)
        }
    }
}

/// PNG 바이트에 pHYs chunk 를 삽입한다 (IHDR 직후, 첫 IDAT 직전).
/// pHYs chunk: 4-byte X ppm + 4-byte Y ppm + 1-byte unit(1=meter).
#[cfg(not(target_arch = "wasm32"))]
fn inject_png_phys(png: Vec<u8>, dpi: f64) -> Vec<u8> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const IHDR_DATA_LEN: usize = 13;

    let ppm = (dpi / 0.0254).round() as u32;
    if png.len() < 8 || png[..8] != PNG_SIGNATURE {
        return png;
    }
    let pos = 8; // signature 이후
    if pos + 8 > png.len() {
        return png;
    }
    let ihdr_len =
        u32::from_be_bytes([png[pos], png[pos + 1], png[pos + 2], png[pos + 3]]) as usize;
    if ihdr_len != IHDR_DATA_LEN || &png[pos + 4..pos + 8] != b"IHDR" {
        return png;
    }
    let Some(ihdr_end) = pos.checked_add(4 + 4 + ihdr_len + 4) else {
        return png;
    };
    if ihdr_end > png.len() {
        return png;
    }

    // pHYs chunk 구성 (9 bytes data)
    let mut phys_data = Vec::with_capacity(9);
    phys_data.extend_from_slice(&ppm.to_be_bytes()); // X pixels per unit
    phys_data.extend_from_slice(&ppm.to_be_bytes()); // Y pixels per unit
    phys_data.push(1); // unit = meter

    let phys_type = b"pHYs";
    let mut phys_chunk = Vec::with_capacity(4 + 4 + 9 + 4);
    phys_chunk.extend_from_slice(&(9u32).to_be_bytes()); // length
    phys_chunk.extend_from_slice(phys_type);
    phys_chunk.extend_from_slice(&phys_data);
    // CRC: type + data
    let mut crc_input = Vec::with_capacity(4 + 9);
    crc_input.extend_from_slice(phys_type);
    crc_input.extend_from_slice(&phys_data);
    let crc = png_crc32(&crc_input);
    phys_chunk.extend_from_slice(&crc.to_be_bytes());

    let mut result = Vec::with_capacity(png.len() + phys_chunk.len());
    result.extend_from_slice(&png[..ihdr_end]);
    result.extend_from_slice(&phys_chunk);
    result.extend_from_slice(&png[ihdr_end..]);
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn png_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// PNG 내보내기 옵션 (export-png CLI / API 통합).
///
/// AI 파이프라인 + VLM (Vision-Language Model) 연동 사용 사례용.
/// `vlm_target` 프리셋 → `max_dimension` → `scale` 순으로 우선순위.
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[derive(Debug, Clone, Default)]
pub struct PngExportOptions {
    /// 명시 배율 (scale). None 이면 max_dimension 기반 자동 계산.
    pub scale: Option<f64>,
    /// 한 변 최대 픽셀 (longest edge). VLM 입력 한도용.
    pub max_dimension: Option<i32>,
    /// VLM 프리셋. Claude / 향후 GPT-4V / Gemini / Qwen-VL / LLaVA 확장 (이슈 #613).
    pub vlm_target: Option<VlmTarget>,
    /// DPI 메타데이터. PNG pHYs chunk 에 기록. 실제 래스터 픽셀 수에 영향 없음.
    /// `scale` 미지정 시 `scale = dpi / 96.0` 자동 계산.
    pub dpi: Option<f64>,
    /// 사용자 지정 폰트 디렉토리 (ttfs 등). SVG 의 `--font-path` 와 동일.
    pub font_paths: Vec<std::path::PathBuf>,
}

/// VLM (Vision-Language Model) 입력 사양 프리셋.
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VlmTarget {
    /// Claude Vision (Anthropic): longest edge ≤1568 px, ≤1.15 MP.
    Claude,
    /// GPT-4V low detail (OpenAI): 512×512 고정.
    Gpt4vLow,
    /// GPT-4V high detail (OpenAI): 768×2000 tile 기반.
    Gpt4vHigh,
    /// Gemini (Google): longest edge ≤3072 px.
    Gemini,
    /// Qwen-VL (Alibaba): longest edge ≤2240 px, 28×28 patch 기반.
    QwenVl,
    /// LLaVA / 기타 OSS (CLIP backbone): longest edge ≤672 px.
    Llava,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
impl VlmTarget {
    /// (longest_edge, max_pixels) 한도 반환.
    pub fn constraints(&self) -> (i32, u64) {
        match self {
            VlmTarget::Claude => (1568, 1_150_000),
            VlmTarget::Gpt4vLow => (512, 262_144),
            VlmTarget::Gpt4vHigh => (2000, 1_536_000),
            VlmTarget::Gemini => (3072, 9_437_184),
            VlmTarget::QwenVl => (2240, 5_017_600),
            VlmTarget::Llava => (672, 451_584),
        }
    }

    /// CLI 옵션 문자열 → VlmTarget 변환.
    /// 하이픈/밑줄 정규화 후 매칭. `gpt4v`/`qwen` 등 축약 별칭도 허용.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "claude" => Some(VlmTarget::Claude),
            "gpt4v_low" => Some(VlmTarget::Gpt4vLow),
            "gpt4v_high" | "gpt4v" => Some(VlmTarget::Gpt4vHigh),
            "gemini" => Some(VlmTarget::Gemini),
            "qwen_vl" | "qwen" => Some(VlmTarget::QwenVl),
            "llava" => Some(VlmTarget::Llava),
            _ => None,
        }
    }

    pub fn all_names() -> &'static str {
        "claude, gpt4v-low, gpt4v-high (또는 gpt4v), gemini, qwen-vl (또는 qwen), llava"
    }
}

impl DocumentCore {
    pub fn get_page_layer_tree_native(&self, page_num: u32) -> Result<String, HwpError> {
        Ok(self.build_page_layer_tree(page_num)?.to_json())
    }

    /// 페이지 overlay 이미지 정보만 작은 JSON으로 반환한다.
    ///
    /// Studio는 BehindText/InFrontOfText 그림 overlay 계산을 위해 전체 PageLayerTree JSON을
    /// 파싱할 필요가 없다. 특히 그림이 본문 layer에만 있는 페이지에서는 빈 overlay 배열과
    /// imageCount만 반환하여 입력 중 대용량 JSON 직렬화/파싱을 피한다.
    pub fn get_page_overlay_images_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::model::image::ImageEffect;
        use crate::model::shape::TextWrap;
        use crate::paint::{
            LayerNode, LayerNodeKind, PaintOp, ResolvedImageKind, ResolvedImagePayload,
        };
        use crate::renderer::render_tree::{BoundingBox, ImageNode};
        use base64::Engine;

        fn effect_str(value: ImageEffect) -> &'static str {
            match value {
                ImageEffect::RealPic => "realPic",
                ImageEffect::GrayScale => "grayScale",
                ImageEffect::BlackWhite => "blackWhite",
                ImageEffect::Pattern8x8 => "pattern8x8",
            }
        }

        fn wrap_str(value: TextWrap) -> &'static str {
            match value {
                TextWrap::BehindText => "behindText",
                TextWrap::InFrontOfText => "inFrontOfText",
                _ => "flow",
            }
        }

        fn write_json_str(buf: &mut String, value: &str) {
            buf.push('"');
            buf.push_str(&crate::document_core::helpers::json_escape(value));
            buf.push('"');
        }

        fn write_bbox(buf: &mut String, bbox: BoundingBox) {
            let _ = write!(
                buf,
                "{{\"x\":{:.3},\"y\":{:.3},\"width\":{:.3},\"height\":{:.3}}}",
                bbox.x, bbox.y, bbox.width, bbox.height
            );
        }

        fn write_overlay_image(
            buf: &mut String,
            bbox: BoundingBox,
            image: &ImageNode,
            resolved: Option<&ResolvedImagePayload>,
            wrap: TextWrap,
        ) {
            if !buf.is_empty() {
                buf.push(',');
            }

            let mut mime = "application/octet-stream";
            let mut base64_data = String::new();
            let mut baked_watermark = false;
            if let Some(payload) = resolved {
                mime = payload.mime;
                base64_data = base64::engine::general_purpose::STANDARD.encode(&payload.data);
                baked_watermark = matches!(payload.kind, ResolvedImageKind::BakedWatermark);
            } else if let Some(data) = &image.data {
                let detected = crate::renderer::svg::detect_image_mime_type(data);
                let (final_mime, final_data): (&str, std::borrow::Cow<[u8]>) =
                    if detected == "image/x-pcx" {
                        match crate::renderer::svg::pcx_bytes_to_png_bytes(data) {
                            Some(png) => ("image/png", std::borrow::Cow::Owned(png)),
                            None => (detected, std::borrow::Cow::Borrowed(data.as_slice())),
                        }
                    } else if detected == "image/bmp" {
                        match crate::renderer::svg::bmp_bytes_to_png_bytes(data) {
                            Some(png) => ("image/png", std::borrow::Cow::Owned(png)),
                            None => (detected, std::borrow::Cow::Borrowed(data.as_slice())),
                        }
                    } else {
                        (detected, std::borrow::Cow::Borrowed(data.as_slice()))
                    };
                mime = final_mime;
                base64_data = base64::engine::general_purpose::STANDARD.encode(&*final_data);
            }

            buf.push('{');
            buf.push_str("\"bbox\":");
            write_bbox(buf, bbox);
            buf.push_str(",\"mime\":");
            write_json_str(buf, mime);
            buf.push_str(",\"base64\":");
            write_json_str(buf, &base64_data);
            buf.push_str(",\"effect\":");
            write_json_str(buf, effect_str(image.effect));
            let _ = write!(
                buf,
                ",\"brightness\":{},\"contrast\":{},\"wrap\":",
                image.brightness, image.contrast
            );
            write_json_str(buf, wrap_str(wrap));
            let opacity = image.opacity.clamp(0.0, 1.0);
            if opacity < 1.0 {
                let _ = write!(buf, ",\"opacity\":{:.6}", opacity);
            }

            if let Some((left, top, right, bottom)) = image.crop {
                let _ = write!(
                    buf,
                    ",\"crop\":{{\"left\":{},\"top\":{},\"right\":{},\"bottom\":{}}}",
                    left, top, right, bottom
                );
            }

            let attr = crate::model::image::ImageAttr {
                brightness: image.brightness,
                contrast: image.contrast,
                effect: image.effect,
                bin_data_id: image.bin_data_id,
                transparency: 0,
                external_path: None,
            };
            if let Some(preset) = attr.watermark_preset() {
                let _ = write!(buf, ",\"watermark\":{{\"preset\":\"{}\"}}", preset);
            }
            if baked_watermark {
                buf.push_str(",\"bakedWatermark\":true");
            }

            let _ = write!(
                buf,
                ",\"transform\":{{\"rotation\":{:.3},\"horzFlip\":{},\"vertFlip\":{}}}}}",
                image.transform.rotation, image.transform.horz_flip, image.transform.vert_flip
            );
        }

        fn collect(
            node: &LayerNode,
            behind: &mut String,
            front: &mut String,
            image_count: &mut usize,
        ) {
            match &node.kind {
                LayerNodeKind::Group { children, .. } => {
                    for child in children {
                        collect(child, behind, front, image_count);
                    }
                }
                LayerNodeKind::ClipRect { child, .. } => collect(child, behind, front, image_count),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        match op {
                            PaintOp::Image {
                                bbox,
                                image,
                                resolved,
                            } => {
                                *image_count += 1;
                                match image.text_wrap {
                                    Some(TextWrap::BehindText) => {
                                        write_overlay_image(
                                            behind,
                                            *bbox,
                                            image,
                                            resolved.as_deref(),
                                            TextWrap::BehindText,
                                        );
                                    }
                                    Some(TextWrap::InFrontOfText) => {
                                        write_overlay_image(
                                            front,
                                            *bbox,
                                            image,
                                            resolved.as_deref(),
                                            TextWrap::InFrontOfText,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            // OLE/차트 미리보기 등은 RawSvg 로 emit 되며 web_canvas 의 draw_image
                            // 경로(IMAGE_CACHE 비동기 디코드)를 그대로 탄다. scheduleReRender 재시도
                            // 발화를 위해 image_count 에 포함한다.
                            PaintOp::RawSvg { .. } => {
                                *image_count += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let tree = self.build_page_layer_tree(page_num)?;
        let mut behind = String::new();
        let mut front = String::new();
        let mut image_count = 0usize;
        collect(&tree.root, &mut behind, &mut front, &mut image_count);

        Ok(format!(
            "{{\"behind\":[{}],\"front\":[{}],\"imageCount\":{}}}",
            behind, front, image_count
        ))
    }

    /// 페이지 정보 (네이티브 에러 타입)
    pub fn get_page_info_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::model::page::PageBorderBasis;
        use crate::renderer::hwpunit_to_px;
        use crate::renderer::layout::body_page_border_outset;
        let (page_content, _, _) = self.find_page(page_num)?;
        let sec_idx = page_content.section_index;
        let section_def = &self.document.sections[sec_idx].section_def;
        let page_def = &section_def.page_def;
        let page_border_fill = &section_def.page_border_fill;
        let ml = hwpunit_to_px(page_def.margin_left as i32, self.dpi);
        let mr = hwpunit_to_px(page_def.margin_right as i32, self.dpi);
        let mt = hwpunit_to_px(page_def.margin_top as i32, self.dpi);
        let mb = hwpunit_to_px(page_def.margin_bottom as i32, self.dpi);
        let mh = hwpunit_to_px(page_def.margin_header as i32, self.dpi);
        let mf = hwpunit_to_px(page_def.margin_footer as i32, self.dpi);
        let pbf_left = hwpunit_to_px(page_border_fill.spacing_left as i32, self.dpi);
        let pbf_right = hwpunit_to_px(page_border_fill.spacing_right as i32, self.dpi);
        let pbf_top = hwpunit_to_px(page_border_fill.spacing_top as i32, self.dpi);
        let pbf_bottom = hwpunit_to_px(page_border_fill.spacing_bottom as i32, self.dpi);
        let body_top = mt + mh;
        let body_bottom_margin = mb + mf;
        let visual_outsets = if matches!(page_border_fill.basis, PageBorderBasis::BodyBased)
            && page_border_fill.border_fill_id > 0
        {
            self.document
                .doc_info
                .border_fills
                .get((page_border_fill.border_fill_id - 1) as usize)
                .map(|bf| {
                    (
                        body_page_border_outset(&bf.borders[0]),
                        body_page_border_outset(&bf.borders[1]),
                        body_page_border_outset(&bf.borders[2]),
                        body_page_border_outset(&bf.borders[3]),
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0, 0.0))
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };
        let (out_l, out_r, out_t, _out_b) = visual_outsets;
        let (page_border_left, page_border_right, page_border_top, page_border_bottom) =
            match page_border_fill.basis {
                PageBorderBasis::PaperBased => (pbf_left, pbf_right, pbf_top, pbf_bottom),
                PageBorderBasis::BodyBased => (
                    (ml - pbf_left - out_l).max(0.0),
                    (mr - pbf_right - out_r).max(0.0),
                    (body_top - pbf_top - out_t).max(0.0),
                    (body_bottom_margin - pbf_bottom).max(0.0),
                ),
            };
        // 단별 영역 정보
        let cols_json: String = page_content
            .layout
            .column_areas
            .iter()
            .map(|ca| format!("{{\"x\":{:.1},\"width\":{:.1}}}", ca.x, ca.width))
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!(
            "{{\"pageIndex\":{},\"width\":{:.1},\"height\":{:.1},\"sectionIndex\":{},\
            \"marginLeft\":{:.1},\"marginRight\":{:.1},\"marginTop\":{:.1},\"marginBottom\":{:.1},\
            \"marginHeader\":{:.1},\"marginFooter\":{:.1},\
            \"pageBorderLeft\":{:.1},\"pageBorderRight\":{:.1},\"pageBorderTop\":{:.1},\"pageBorderBottom\":{:.1},\
            \"columns\":[{}]}}",
            page_content.page_index,
            page_content.layout.page_width,
            page_content.layout.page_height,
            page_content.section_index,
            ml,
            mr,
            mt,
            mb,
            mh,
            mf,
            page_border_left,
            page_border_right,
            page_border_top,
            page_border_bottom,
            cols_json,
        ))
    }

    /// 구역 정의(SectionDef)를 JSON으로 반환 (네이티브 에러 타입)
    pub fn get_section_def_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let sd = &section.section_def;
        Ok(format!(
            "{{\"pageNum\":{},\"pageNumType\":{},\"pictureNum\":{},\"tableNum\":{},\"equationNum\":{},\
            \"columnSpacing\":{},\"defaultTabSpacing\":{},\
            \"hideHeader\":{},\"hideFooter\":{},\"hideMasterPage\":{},\
            \"hideBorder\":{},\"hideFill\":{},\"hideEmptyLine\":{}}}",
            sd.page_num, sd.page_num_type,
            sd.picture_num, sd.table_num, sd.equation_num,
            sd.column_spacing, sd.default_tab_spacing,
            sd.hide_header, sd.hide_footer, sd.hide_master_page,
            sd.hide_border, sd.hide_fill, sd.hide_empty_line,
        ))
    }

    /// 단일 구역의 SectionDef 필드를 JSON에서 업데이트 (재조판 없이)
    fn apply_section_def_json(&mut self, section_idx: usize, json: &str) -> Result<(), HwpError> {
        use super::super::helpers::{json_bool, json_u16, json_u32};

        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let sd = &mut section.section_def;

        if let Some(v) = json_u16(json, "pageNum") {
            sd.page_num = v;
        }
        if let Some(v) = json_u16(json, "pageNumType") {
            sd.page_num_type = v as u8;
        }
        if let Some(v) = json_u16(json, "pictureNum") {
            sd.picture_num = v;
        }
        if let Some(v) = json_u16(json, "tableNum") {
            sd.table_num = v;
        }
        if let Some(v) = json_u16(json, "equationNum") {
            sd.equation_num = v;
        }
        if let Some(v) = json_u16(json, "columnSpacing") {
            sd.column_spacing = v as i16;
        }
        if let Some(v) = json_u32(json, "defaultTabSpacing") {
            sd.default_tab_spacing = v;
        }
        if let Some(v) = json_bool(json, "hideHeader") {
            sd.hide_header = v;
        }
        if let Some(v) = json_bool(json, "hideFooter") {
            sd.hide_footer = v;
        }
        if let Some(v) = json_bool(json, "hideMasterPage") {
            sd.hide_master_page = v;
        }
        if let Some(v) = json_bool(json, "hideBorder") {
            sd.hide_border = v;
        }
        if let Some(v) = json_bool(json, "hideFill") {
            sd.hide_fill = v;
        }
        if let Some(v) = json_bool(json, "hideEmptyLine") {
            sd.hide_empty_line = v;
        }

        // flags 비트플래그 재구성 (파서와 동일한 비트 위치)
        let flags = &mut sd.flags;
        fn set_bit(flags: &mut u32, mask: u32, val: bool) {
            if val {
                *flags |= mask;
            } else {
                *flags &= !mask;
            }
        }
        set_bit(flags, 0x0001, sd.hide_header); // bit 0
        set_bit(flags, 0x0002, sd.hide_footer); // bit 1
        set_bit(flags, 0x0004, sd.hide_master_page); // bit 2 (HWP5 스펙, 첫쪽 바탕쪽 감춤)
        set_bit(flags, 0x0008, sd.hide_border); // bit 3
        set_bit(flags, 0x0010, sd.hide_fill); // bit 4
        set_bit(flags, 0x00080000, sd.hide_empty_line); // bit 19
                                                        // bit 20-21: 쪽 번호 종류
        *flags &= !0x00300000; // clear bits 20-21
        *flags |= ((sd.page_num_type as u32) & 0x03) << 20;

        // paragraphs[0]의 Control::SectionDef에도 동기화
        let updated_sd = section.section_def.clone();
        if let Some(para) = section.paragraphs.get_mut(0) {
            for ctrl in &mut para.controls {
                if let Control::SectionDef(ref mut s) = ctrl {
                    s.page_num = updated_sd.page_num;
                    s.page_num_type = updated_sd.page_num_type;
                    s.picture_num = updated_sd.picture_num;
                    s.table_num = updated_sd.table_num;
                    s.equation_num = updated_sd.equation_num;
                    s.column_spacing = updated_sd.column_spacing;
                    s.default_tab_spacing = updated_sd.default_tab_spacing;
                    s.hide_header = updated_sd.hide_header;
                    s.hide_footer = updated_sd.hide_footer;
                    s.hide_master_page = updated_sd.hide_master_page;
                    s.hide_border = updated_sd.hide_border;
                    s.hide_fill = updated_sd.hide_fill;
                    s.hide_empty_line = updated_sd.hide_empty_line;
                    s.flags = updated_sd.flags;
                    break;
                }
            }
        }

        // raw_stream 무효화
        section.raw_stream = None;
        Ok(())
    }

    /// 재조판 + 재페이지네이션 수행
    fn recompose_and_paginate(&mut self) -> u32 {
        self.composed = self
            .document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.paginate();
        self.page_count()
    }

    /// 구역 정의(SectionDef)를 변경하고 재페이지네이션 (네이티브 에러 타입)
    pub fn set_section_def_native(
        &mut self,
        section_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.apply_section_def_json(section_idx, json)?;
        let page_count = self.recompose_and_paginate();
        Ok(format!("{{\"ok\":true,\"pageCount\":{}}}", page_count))
    }

    /// 모든 구역의 SectionDef를 일괄 변경하고 재페이지네이션 (네이티브 에러 타입)
    pub fn set_section_def_all_native(&mut self, json: &str) -> Result<String, HwpError> {
        let count = self.document.sections.len();
        for idx in 0..count {
            self.apply_section_def_json(idx, json)?;
        }
        let page_count = self.recompose_and_paginate();
        Ok(format!("{{\"ok\":true,\"pageCount\":{}}}", page_count))
    }

    /// 구역의 쪽 테두리/배경 설정을 JSON으로 반환한다.
    pub fn get_page_border_fill_native(&self, section_idx: usize) -> Result<String, HwpError> {
        use crate::document_core::helpers::{border_line_type_to_u8_val, color_ref_to_css};
        use crate::model::page::PageBorderUiBasis;
        use crate::model::style::FillType;

        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let pbf = &section.section_def.page_border_fill;
        let basis = match pbf.ui_basis {
            PageBorderUiBasis::Paper => "paper",
            PageBorderUiBasis::Page => "page",
        };
        let fill_area = match (pbf.attr >> 3) & 0x03 {
            1 => "page",
            2 => "border",
            _ => "paper",
        };
        let mut border_json = concat!(
            "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
            "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
            "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
            "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
            "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
        )
        .to_string();

        if pbf.border_fill_id > 0 {
            if let Some(bf) = self
                .document
                .doc_info
                .border_fills
                .get((pbf.border_fill_id - 1) as usize)
            {
                let dir_names = ["Left", "Right", "Top", "Bottom"];
                let borders = bf
                    .borders
                    .iter()
                    .enumerate()
                    .map(|(idx, border)| {
                        format!(
                            "\"border{}\":{{\"type\":{},\"width\":{},\"color\":\"{}\"}}",
                            dir_names[idx],
                            border_line_type_to_u8_val(border.line_type),
                            border.width,
                            color_ref_to_css(border.color)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let (fill_type, fill_color, pattern_color, pattern_type) =
                    match (&bf.fill.fill_type, &bf.fill.solid) {
                        (FillType::Solid, Some(solid)) => (
                            "solid",
                            color_ref_to_css(solid.background_color),
                            color_ref_to_css(solid.pattern_color),
                            solid.pattern_type,
                        ),
                        _ => ("none", "#ffffff".to_string(), "#000000".to_string(), 0),
                    };
                border_json = format!(
                    "{},\"fillType\":\"{}\",\"fillColor\":\"{}\",\"patternColor\":\"{}\",\"patternType\":{}",
                    borders, fill_type, fill_color, pattern_color, pattern_type
                );
            }
        }

        Ok(format!(
            "{{\"attr\":{},\"basis\":\"{}\",\"spacingLeft\":{},\"spacingRight\":{},\"spacingTop\":{},\"spacingBottom\":{},\
            \"borderFillId\":{},\"headerInside\":{},\"footerInside\":{},\"fillArea\":\"{}\",\
            \"hideBorder\":{},\"hideFill\":{},{} }}",
            pbf.attr,
            basis,
            pbf.spacing_left,
            pbf.spacing_right,
            pbf.spacing_top,
            pbf.spacing_bottom,
            pbf.border_fill_id,
            (pbf.attr & 0x02) != 0,
            (pbf.attr & 0x04) != 0,
            fill_area,
            section.section_def.hide_border,
            section.section_def.hide_fill,
            border_json,
        ))
    }

    /// 구역의 쪽 테두리/배경 설정을 JSON에서 업데이트하고 재조판한다.
    pub fn set_page_border_fill_native(
        &mut self,
        section_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        use crate::document_core::helpers::{json_bool, json_i16, json_str, json_u32};
        use crate::model::page::{PageBorderBasis, PageBorderUiBasis};

        let border_fill_id = self.create_border_fill_from_json(json);
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let sd = &mut section.section_def;
        let pbf = &mut sd.page_border_fill;

        if let Some(v) = json_i16(json, "spacingLeft") {
            pbf.spacing_left = v;
        }
        if let Some(v) = json_i16(json, "spacingRight") {
            pbf.spacing_right = v;
        }
        if let Some(v) = json_i16(json, "spacingTop") {
            pbf.spacing_top = v;
        }
        if let Some(v) = json_i16(json, "spacingBottom") {
            pbf.spacing_bottom = v;
        }

        let mut attr = pbf.attr;
        if let Some(basis) = json_str(json, "basis") {
            if basis == "paper" {
                pbf.ui_basis = PageBorderUiBasis::Paper;
                pbf.basis = PageBorderBasis::PaperBased;
                attr &= !0x01;
            } else {
                pbf.ui_basis = PageBorderUiBasis::Page;
                pbf.basis = PageBorderBasis::BodyBased;
                attr |= 0x01;
            }
        }
        if let Some(v) = json_bool(json, "headerInside") {
            if v {
                attr |= 0x02;
            } else {
                attr &= !0x02;
            }
        }
        if let Some(v) = json_bool(json, "footerInside") {
            if v {
                attr |= 0x04;
            } else {
                attr &= !0x04;
            }
        }
        if let Some(area) = json_str(json, "fillArea") {
            attr &= !(0x03 << 3);
            attr |= match area.as_str() {
                "page" => 0x01 << 3,
                "border" => 0x02 << 3,
                _ => 0,
            };
        }
        pbf.attr = attr;
        pbf.border_fill_id = border_fill_id;

        let apply_page = json_str(json, "applyPage").unwrap_or_else(|| "all".to_string());
        let hide_first = apply_page == "exceptFirst";
        if let Some(v) = json_bool(json, "hideBorder") {
            sd.hide_border = v;
        } else {
            sd.hide_border = hide_first;
        }
        if let Some(v) = json_bool(json, "hideFill") {
            sd.hide_fill = v;
        } else {
            sd.hide_fill = hide_first;
        }

        let flags = &mut sd.flags;
        if pbf.ui_basis == PageBorderUiBasis::Page {
            pbf.attr |= 0x01;
        } else {
            pbf.attr &= !0x01;
        }
        if sd.hide_border {
            *flags |= 0x0008;
        } else {
            *flags &= !0x0008;
        }
        if sd.hide_fill {
            *flags |= 0x0010;
        } else {
            *flags &= !0x0010;
        }
        if let Some(raw) = json_u32(json, "attr") {
            pbf.attr = (pbf.attr & 0x0000_001f) | (raw & !0x0000_001f);
        }

        let updated_sd = sd.clone();
        if let Some(para) = section.paragraphs.get_mut(0) {
            for ctrl in &mut para.controls {
                if let Control::SectionDef(ref mut ctrl_sd) = ctrl {
                    ctrl_sd.page_border_fill = updated_sd.page_border_fill.clone();
                    ctrl_sd.hide_border = updated_sd.hide_border;
                    ctrl_sd.hide_fill = updated_sd.hide_fill;
                    ctrl_sd.flags = updated_sd.flags;
                    break;
                }
            }
        }

        section.raw_stream = None;
        let page_count = self.recompose_and_paginate();
        Ok(format!("{{\"ok\":true,\"pageCount\":{}}}", page_count))
    }

    /// 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환 (네이티브 에러 타입)
    pub fn get_page_def_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let pd = &section.section_def.page_def;
        let binding: u8 = match pd.binding {
            crate::model::page::BindingMethod::SingleSided => 0,
            crate::model::page::BindingMethod::DuplexSided => 1,
            crate::model::page::BindingMethod::TopFlip => 2,
        };
        Ok(format!(
            "{{\"width\":{},\"height\":{},\
            \"marginLeft\":{},\"marginRight\":{},\"marginTop\":{},\"marginBottom\":{},\
            \"marginHeader\":{},\"marginFooter\":{},\"marginGutter\":{},\
            \"landscape\":{},\"binding\":{}}}",
            pd.width,
            pd.height,
            pd.margin_left,
            pd.margin_right,
            pd.margin_top,
            pd.margin_bottom,
            pd.margin_header,
            pd.margin_footer,
            pd.margin_gutter,
            pd.landscape,
            binding,
        ))
    }

    /// 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션 (네이티브 에러 타입)
    pub fn set_page_def_native(
        &mut self,
        section_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        use crate::model::page::BindingMethod;

        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let pd = &mut section.section_def.page_def;

        use super::super::helpers::{json_bool, json_u32};

        if let Some(v) = json_u32(json, "width") {
            pd.width = v;
        }
        if let Some(v) = json_u32(json, "height") {
            pd.height = v;
        }
        if let Some(v) = json_u32(json, "marginLeft") {
            pd.margin_left = v;
        }
        if let Some(v) = json_u32(json, "marginRight") {
            pd.margin_right = v;
        }
        if let Some(v) = json_u32(json, "marginTop") {
            pd.margin_top = v;
        }
        if let Some(v) = json_u32(json, "marginBottom") {
            pd.margin_bottom = v;
        }
        if let Some(v) = json_u32(json, "marginHeader") {
            pd.margin_header = v;
        }
        if let Some(v) = json_u32(json, "marginFooter") {
            pd.margin_footer = v;
        }
        if let Some(v) = json_u32(json, "marginGutter") {
            pd.margin_gutter = v;
        }
        if let Some(v) = json_bool(json, "landscape") {
            pd.landscape = v;
        }
        if let Some(v) = json_u32(json, "binding") {
            pd.binding = match v {
                1 => BindingMethod::DuplexSided,
                2 => BindingMethod::TopFlip,
                _ => BindingMethod::SingleSided,
            };
        }

        // FIX 1: attr 비트플래그 재구성 (bit 0 = landscape, bit 1-2 = binding)
        pd.attr = (pd.attr & !0x07)
            | (if pd.landscape { 1 } else { 0 })
            | (match pd.binding {
                BindingMethod::SingleSided => 0u32,
                BindingMethod::DuplexSided => 1u32 << 1,
                BindingMethod::TopFlip => 2u32 << 1,
            });

        // FIX 2: paragraphs[0]의 Control::SectionDef에도 page_def 동기화
        let updated_page_def = section.section_def.page_def.clone();
        if let Some(para) = section.paragraphs.get_mut(0) {
            for ctrl in &mut para.controls {
                if let Control::SectionDef(ref mut sd) = ctrl {
                    sd.page_def = updated_page_def;
                    break;
                }
            }
        }

        // FIX 3: raw_stream 무효화 → 직렬화 시 모델에서 재구성
        section.raw_stream = None;

        // 재조판 + 재페이지네이션
        self.composed = self
            .document
            .sections
            .iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.paginate();

        let page_count = self.page_count();
        Ok(format!("{{\"ok\":true,\"pageCount\":{}}}", page_count))
    }

    /// 텍스트 레이아웃 정보 (네이티브 에러 타입)
    pub fn get_page_text_layout_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree(page_num)?;

        // 렌더 트리에서 TextRun 노드를 재귀적으로 수집
        fn collect_text_runs(node: &RenderNode, runs: &mut Vec<String>) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let positions = compute_char_positions(&text_run.text, &text_run.style);
                let char_x: Vec<String> = positions.iter().map(|v| format!("{:.1}", v)).collect();

                let escaped_text = super::super::helpers::json_escape(&text_run.text);

                // 문서 좌표 (편집용)
                let doc_coords = match (
                    text_run.section_index,
                    text_run.para_index,
                    text_run.char_start,
                ) {
                    (Some(si), Some(pi), Some(cs)) => {
                        format!(",\"secIdx\":{},\"paraIdx\":{},\"charStart\":{}", si, pi, cs)
                    }
                    _ => String::new(),
                };

                // 표 셀 식별 정보 (편집용)
                let cell_coords = if let Some(ref ctx) = text_run.cell_context {
                    let outer = &ctx.path[0];
                    let path_entries: Vec<String> = ctx
                        .path
                        .iter()
                        .map(|e| {
                            format!(
                                "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                                e.control_index, e.cell_index, e.cell_para_index
                            )
                        })
                        .collect();
                    format!(",\"parentParaIdx\":{},\"controlIdx\":{},\"cellIdx\":{},\"cellParaIdx\":{},\"cellPath\":[{}]",
                        ctx.parent_para_index, outer.control_index, outer.cell_index, outer.cell_para_index,
                        path_entries.join(","))
                } else {
                    String::new()
                };

                let escaped_font = super::super::helpers::json_escape(&text_run.style.font_family);
                let font_info = format!(
                    ",\"fontFamily\":\"{}\",\"fontSize\":{:.1},\"bold\":{},\"italic\":{},\"ratio\":{:.2},\"letterSpacing\":{:.1}",
                    escaped_font,
                    text_run.style.font_size,
                    text_run.style.bold,
                    text_run.style.italic,
                    text_run.style.ratio,
                    text_run.style.letter_spacing,
                );

                // 서식 툴바용 추가 속성
                let format_info = format!(
                    ",\"underline\":{},\"strikethrough\":{},\"textColor\":\"{}\"",
                    !matches!(
                        text_run.style.underline,
                        crate::model::style::UnderlineType::None
                    ),
                    text_run.style.strikethrough,
                    color_ref_to_css(text_run.style.color),
                );

                // 글자/문단 모양 ID (서식 툴바용)
                let shape_ids = match (text_run.char_shape_id, text_run.para_shape_id) {
                    (Some(csid), Some(psid)) => {
                        format!(",\"charShapeId\":{},\"paraShapeId\":{}", csid, psid)
                    }
                    (Some(csid), None) => format!(",\"charShapeId\":{}", csid),
                    (None, Some(psid)) => format!(",\"paraShapeId\":{}", psid),
                    _ => String::new(),
                };

                runs.push(format!(
                    "{{\"text\":\"{}\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"charX\":[{}]{}{}{}{}{}}}",
                    escaped_text,
                    node.bbox.x,
                    node.bbox.y,
                    node.bbox.width,
                    node.bbox.height,
                    char_x.join(","),
                    font_info,
                    format_info,
                    shape_ids,
                    doc_coords,
                    cell_coords,
                ));
            }
            for child in &node.children {
                collect_text_runs(child, runs);
            }
        }

        let mut runs = Vec::new();
        collect_text_runs(&tree.root, &mut runs);

        Ok(format!("{{\"runs\":[{}]}}", runs.join(",")))
    }

    /// 컨트롤(표, 이미지 등) 레이아웃 정보 (네이티브 에러 타입)
    pub fn get_page_control_layout_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let tree = self.build_page_tree_cached(page_num)?;

        // 렌더 트리에서 Table, Image 노드를 재귀적으로 수집
        fn collect_controls(node: &RenderNode, controls: &mut Vec<String>) {
            // [Task #1280 v2] 컨트롤별 plane/zOrder/stableIndex 노출 — 렌더 정렬키
            // `paper_node_sort_key`(layout.rs)를 그대로 재사용해 프런트 히트테스트가
            // 겹침 시 "최상단 개체"를 선택할 수 있게 한다. inline(layer=None) 노드는
            // (plane=2, z=0, stable=node.id) 폴백으로 렌더 정렬과 동일.
            let (plane, z_order, stable_index) =
                crate::renderer::layout::LayoutEngine::paper_node_sort_key(node);
            // wrap 은 이미지뿐 아니라 shape/line/group/path 에도 노출(히트테스트 plane/wrap 일관성).
            // 이미지는 자체 wrap_str(image_node.text_wrap)을 방출하므로 중복 방지로 제외한다.
            let wrap_extra = if matches!(node.node_type, RenderNodeType::Image(_)) {
                ""
            } else {
                match node.layer.and_then(|l| l.text_wrap) {
                    Some(crate::model::shape::TextWrap::BehindText) => ",\"wrap\":\"behindText\"",
                    Some(crate::model::shape::TextWrap::InFrontOfText) => {
                        ",\"wrap\":\"inFrontOfText\""
                    }
                    Some(crate::model::shape::TextWrap::Square) => ",\"wrap\":\"square\"",
                    Some(crate::model::shape::TextWrap::Tight) => ",\"wrap\":\"tight\"",
                    Some(crate::model::shape::TextWrap::Through) => ",\"wrap\":\"through\"",
                    Some(crate::model::shape::TextWrap::TopAndBottom) => {
                        ",\"wrap\":\"topAndBottom\""
                    }
                    None => "",
                }
            };
            let layer_str = format!(
                ",\"plane\":{},\"zOrder\":{},\"stableIndex\":{}{}",
                plane, z_order, stable_index, wrap_extra
            );
            match &node.node_type {
                RenderNodeType::Table(table_node) => {
                    // 문서 좌표
                    let doc_coords = match (
                        table_node.section_index,
                        table_node.para_index,
                        table_node.control_index,
                    ) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(
                                ",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}",
                                si, pi, ci
                            )
                        }
                        _ => String::new(),
                    };

                    // 셀 정보 수집
                    let mut cells = Vec::new();
                    for (cell_idx, child) in node.children.iter().enumerate() {
                        if let RenderNodeType::TableCell(cell_node) = &child.node_type {
                            cells.push(format!(
                                "{{\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"cellIdx\":{}}}",
                                child.bbox.x, child.bbox.y, child.bbox.width, child.bbox.height,
                                cell_node.row, cell_node.col, cell_node.row_span, cell_node.col_span,
                                cell_idx
                            ));
                        }
                    }

                    controls.push(format!(
                        "{{\"type\":\"table\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"rowCount\":{},\"colCount\":{}{}{},\"cells\":[{}]}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        table_node.row_count, table_node.col_count,
                        doc_coords, layer_str,
                        cells.join(",")
                    ));
                    // Table 내부도 탐색 (셀 내 수식 등 수집)
                }
                RenderNodeType::Equation(eq_node) => {
                    let doc_coords = match (
                        eq_node.section_index,
                        eq_node.para_index,
                        eq_node.control_index,
                    ) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(
                                ",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}",
                                si, pi, ci
                            )
                        }
                        _ => String::new(),
                    };
                    let cell_coords = match (eq_node.cell_index, eq_node.cell_para_index) {
                        (Some(ci), Some(cpi)) => {
                            format!(",\"cellIdx\":{},\"cellParaIdx\":{}", ci, cpi)
                        }
                        _ => String::new(),
                    };
                    let note_ref = eq_node.note_ref.as_ref().map_or_else(String::new, |r| {
                        format!(
                            ",\"noteRef\":{{\"kind\":\"{}\",\"sectionIdx\":{},\"paraIdx\":{},\"controlIdx\":{},\"noteParaIdx\":{},\"innerControlIdx\":{}}}",
                            r.kind,
                            r.section_index,
                            r.para_index,
                            r.control_index,
                            r.note_para_index,
                            r.inner_control_index
                        )
                    });

                    controls.push(format!(
                        "{{\"type\":\"equation\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}{}{}{}}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        doc_coords, cell_coords, note_ref, layer_str
                    ));
                    return;
                }
                RenderNodeType::Image(image_node) => {
                    let doc_coords = match (
                        image_node.section_index,
                        image_node.para_index,
                        image_node.control_index,
                    ) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(
                                ",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}",
                                si, pi, ci
                            )
                        }
                        _ => String::new(),
                    };
                    // Task #516 결함 3: hit-test 정합 (옵션 3-C) — wrap 모드 노출.
                    // BehindText 그림은 텍스트 영역 위에서는 hit-test 후순위 처리.
                    let wrap_str = match image_node.text_wrap {
                        Some(crate::model::shape::TextWrap::BehindText) => {
                            ",\"wrap\":\"behindText\""
                        }
                        Some(crate::model::shape::TextWrap::InFrontOfText) => {
                            ",\"wrap\":\"inFrontOfText\""
                        }
                        Some(crate::model::shape::TextWrap::Square) => ",\"wrap\":\"square\"",
                        Some(crate::model::shape::TextWrap::Tight) => ",\"wrap\":\"tight\"",
                        Some(crate::model::shape::TextWrap::Through) => ",\"wrap\":\"through\"",
                        Some(crate::model::shape::TextWrap::TopAndBottom) => {
                            ",\"wrap\":\"topAndBottom\""
                        }
                        None => "",
                    };
                    // [Task #825] 머리말/꼬리말 그림 marker — rhwp-studio findPictureAtClick
                    // 이 secIdx 부재로 필터링하지 않도록 hf 정보 포함.
                    let hf_str = match &image_node.header_footer_ref {
                        Some(r) => {
                            let kind = match r.kind {
                                crate::renderer::render_tree::HeaderFooterKind::Header => "header",
                                crate::renderer::render_tree::HeaderFooterKind::Footer => "footer",
                            };
                            format!(
                                ",\"headerFooter\":{{\"kind\":\"{}\",\"outerParaIdx\":{},\"outerControlIdx\":{}}}",
                                kind, r.outer_para_index, r.outer_control_index
                            )
                        }
                        None => String::new(),
                    };
                    // [Task #1151 v4] 셀 안 inline picture 의 cellIdx/cellParaIdx /
                    // outerTableControlIdx 전달. RectangleNode 처리 (라인 1533-1546)
                    // 와 동일 패턴. 셀 외부 picture 는 cell_index 등이 None → 빈 문자열
                    // → 기존 JSON 출력 그대로 (회귀 0).
                    let cell_str = match (image_node.cell_index, image_node.cell_para_index) {
                        (Some(cei), Some(cpi)) => {
                            format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi)
                        }
                        _ => String::new(),
                    };
                    let outer_table_str = match image_node.outer_table_control_index {
                        Some(otci) => format!(",\"outerTableControlIdx\":{}", otci),
                        None => String::new(),
                    };
                    // [Task #1161] 전체 다단계 cellPath (중첩 표/글상자). TextRun 방출
                    // (cell_coords, 라인 1313-1327) 과 동일 포맷. 단일 레벨 스칼라는 위
                    // cell_str/outer_table_str 로 하위호환 유지. 본문 picture 는 빈 문자열.
                    let cell_path_str = match &image_node.cell_context {
                        Some(ctx) => {
                            let entries: Vec<String> = ctx
                                .path
                                .iter()
                                .map(|e| {
                                    format!(
                                        "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                                        e.control_index, e.cell_index, e.cell_para_index
                                    )
                                })
                                .collect();
                            format!(
                                ",\"parentParaIdx\":{},\"cellPath\":[{}]",
                                ctx.parent_para_index,
                                entries.join(",")
                            )
                        }
                        None => String::new(),
                    };
                    controls.push(format!(
                        "{{\"type\":\"image\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}{}{}{}{}{}{}}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        doc_coords, wrap_str, hf_str, cell_str, outer_table_str, cell_path_str, layer_str
                    ));
                    return;
                }
                RenderNodeType::Group(group_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (
                        group_node.section_index,
                        group_node.para_index,
                        group_node.control_index,
                    ) {
                        controls.push(format!(
                            "{{\"type\":\"group\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci, layer_str
                        ));
                        return; // 자식 개별 수집하지 않음 — 묶음 전체가 하나의 컨트롤
                    }
                }
                RenderNodeType::Rectangle(rect_node) => {
                    // 문서 좌표가 있는 Rectangle만 shape로 수집 (배경 사각형 제외)
                    if let (Some(si), Some(pi), Some(ci)) = (
                        rect_node.section_index,
                        rect_node.para_index,
                        rect_node.control_index,
                    ) {
                        // [Task #1138] 표 셀 안 사각형: cellIdx/cellParaIdx/outerTableControlIdx
                        let cell_str = match (rect_node.cell_index, rect_node.cell_para_index) {
                            (Some(cei), Some(cpi)) => {
                                format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi)
                            }
                            _ => String::new(),
                        };
                        let outer_table_str = match rect_node.outer_table_control_index {
                            Some(otci) => format!(",\"outerTableControlIdx\":{}", otci),
                            None => String::new(),
                        };
                        controls.push(format!(
                            "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}{}{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci, cell_str, outer_table_str, layer_str
                        ));
                        // [Task #1171] return 하지 않고 자식으로 재귀 — 사각형 글상자(text_box)
                        // 안 중첩 picture/도형이 cellPath(cell_index=0 sentinel)로 수집되도록 한다.
                        // 장식 노드(테두리 등)는 section/para/control 좌표가 None 이라 컨트롤로
                        // 방출되지 않는다(좌표 가드). Table 핸들러와 동일하게 자식 탐색.
                    }
                }
                RenderNodeType::Line(line_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (
                        line_node.section_index,
                        line_node.para_index,
                        line_node.control_index,
                    ) {
                        // [Task #1138] 표 셀 안 직선
                        let cell_str = match (line_node.cell_index, line_node.cell_para_index) {
                            (Some(cei), Some(cpi)) => {
                                format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi)
                            }
                            _ => String::new(),
                        };
                        let outer_table_str = match line_node.outer_table_control_index {
                            Some(otci) => format!(",\"outerTableControlIdx\":{}", otci),
                            None => String::new(),
                        };
                        controls.push(format!(
                            "{{\"type\":\"line\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"x1\":{:.1},\"y1\":{:.1},\"x2\":{:.1},\"y2\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}{}{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            line_node.x1, line_node.y1, line_node.x2, line_node.y2,
                            si, pi, ci, cell_str, outer_table_str, layer_str
                        ));
                        return;
                    }
                }
                RenderNodeType::Ellipse(ell_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (
                        ell_node.section_index,
                        ell_node.para_index,
                        ell_node.control_index,
                    ) {
                        // [Task #1138] 표 셀 안 타원
                        let cell_str = match (ell_node.cell_index, ell_node.cell_para_index) {
                            (Some(cei), Some(cpi)) => {
                                format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi)
                            }
                            _ => String::new(),
                        };
                        let outer_table_str = match ell_node.outer_table_control_index {
                            Some(otci) => format!(",\"outerTableControlIdx\":{}", otci),
                            None => String::new(),
                        };
                        controls.push(format!(
                            "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}{}{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci, cell_str, outer_table_str, layer_str
                        ));
                        return;
                    }
                }
                RenderNodeType::Path(path_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (
                        path_node.section_index,
                        path_node.para_index,
                        path_node.control_index,
                    ) {
                        // [Task #1138] 표 셀 안 path (다각형/곡선/연결선)
                        let cell_str = match (path_node.cell_index, path_node.cell_para_index) {
                            (Some(cei), Some(cpi)) => {
                                format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi)
                            }
                            _ => String::new(),
                        };
                        let outer_table_str = match path_node.outer_table_control_index {
                            Some(otci) => format!(",\"outerTableControlIdx\":{}", otci),
                            None => String::new(),
                        };
                        if let Some((x1, y1, x2, y2)) = path_node.connector_endpoints {
                            // 연결선: 선 선택 방식 (시작/끝 좌표 포함)
                            controls.push(format!(
                                "{{\"type\":\"line\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"x1\":{:.1},\"y1\":{:.1},\"x2\":{:.1},\"y2\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}{}{}}}",
                                node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                                x1, y1, x2, y2,
                                si, pi, ci, cell_str, outer_table_str, layer_str
                            ));
                        } else {
                            controls.push(format!(
                                "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}{}{}{}}}",
                                node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                                si, pi, ci, cell_str, outer_table_str, layer_str
                            ));
                        }
                        return;
                    }
                }
                _ => {}
            }
            for child in &node.children {
                collect_controls(child, controls);
            }
        }

        let mut controls = Vec::new();
        collect_controls(&tree.root, &mut controls);

        Ok(format!("{{\"controls\":[{}]}}", controls.join(",")))
    }

    /// 구역의 문단들에서 초기 ColumnDef를 추출한다.
    pub(crate) fn find_initial_column_def(paragraphs: &[Paragraph]) -> ColumnDef {
        for para in paragraphs {
            for ctrl in &para.controls {
                if let Control::ColumnDef(cd) = ctrl {
                    return cd.clone();
                }
            }
        }
        ColumnDef::default()
    }

    /// 특정 문단에 적용되는 ColumnDef를 찾는다.
    /// 문단 순서대로 탐색하여 para_idx 이하에서 가장 마지막 ColumnDef를 반환한다.
    /// (한 구역 내에서 다단↔단일단 전환이 여러 번 일어날 수 있음)
    pub(crate) fn find_column_def_for_paragraph(
        paragraphs: &[Paragraph],
        para_idx: usize,
    ) -> ColumnDef {
        let mut last_cd = ColumnDef::default();
        for (i, para) in paragraphs.iter().enumerate() {
            if i > para_idx {
                break;
            }
            for ctrl in &para.controls {
                if let Control::ColumnDef(cd) = ctrl {
                    last_cd = cd.clone();
                }
            }
        }
        last_cd
    }

    /// 구역을 재조판하고 dirty로 표시한다.
    pub(crate) fn recompose_section(&mut self, section_idx: usize) {
        self.invalidate_page_tree_cache();
        self.composed[section_idx] = compose_section(&self.document.sections[section_idx]);
        if section_idx < self.dirty_sections.len() {
            self.dirty_sections[section_idx] = true;
        }
        // 전체 문단 dirty (모두 재측정 필요)
        if section_idx < self.dirty_paragraphs.len() {
            self.dirty_paragraphs[section_idx] = None;
        }
    }

    /// 구역을 dirty로 표시만 한다 (재조판 없이).
    /// 셀 내부 편집처럼 composed 데이터가 불변인 경우 사용.
    pub(crate) fn mark_section_dirty(&mut self, section_idx: usize) {
        if section_idx < self.dirty_sections.len() {
            self.dirty_sections[section_idx] = true;
        }
        // 문단 dirty는 건드리지 않음 (셀 편집 시 문단 재측정 불필요)
    }

    /// 문단 dirty 비트 설정
    pub(crate) fn mark_paragraph_dirty(&mut self, section_idx: usize, para_idx: usize) {
        if section_idx >= self.dirty_paragraphs.len() {
            return;
        }
        let para_count = self.document.sections[section_idx].paragraphs.len();
        match &mut self.dirty_paragraphs[section_idx] {
            None => {
                // 전체 dirty 상태에서 선택적으로 전환:
                // 전체 dirty인데 특정 문단만 dirty로 설정하면 안 됨 → 이미 전체 dirty이므로 유지
                // (이 경우는 recompose_section 후 recompose_paragraph 호출 시 발생 불가)
            }
            Some(bits) => {
                bits.resize(para_count, false);
                if para_idx < bits.len() {
                    bits[para_idx] = true;
                }
            }
        }
    }

    /// 단일 문단만 재조판한다.
    pub(crate) fn recompose_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        self.invalidate_page_tree_cache();
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        self.composed[section_idx][para_idx] = compose_paragraph(para);
        self.mark_section_dirty(section_idx);
        self.mark_paragraph_dirty(section_idx, para_idx);
    }

    /// composed 벡터에 새 문단 항목을 삽입한다 (문단 분할/붙여넣기 후).
    pub(crate) fn insert_composed_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        self.invalidate_page_tree_cache();
        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let composed = compose_paragraph(para);
        self.composed[section_idx].insert(para_idx, composed);
        self.mark_section_dirty(section_idx);
        // measured_sections: 인덱스 조정으로 기존 측정값 재사용 (전체 재측정 회피)
        if section_idx < self.measured_sections.len() {
            self.measured_sections[section_idx].shift_for_insert(para_idx);
        }
        // dirty_paragraphs: 삽입된 문단과 분할 원본만 dirty 표시
        if section_idx < self.dirty_paragraphs.len() {
            let para_count = self.document.sections[section_idx].paragraphs.len();
            let bits =
                self.dirty_paragraphs[section_idx].get_or_insert_with(|| vec![false; para_count]);
            // 기존 비트맵에 삽입 위치 추가 (후속 인덱스 shift)
            if para_idx <= bits.len() {
                bits.insert(para_idx, true);
            }
            // 비트맵 길이를 문단 수에 맞춤
            bits.resize(para_count, false);
            // 분할 원본 문단도 dirty
            if para_idx > 0 {
                bits[para_idx - 1] = true;
            }
        }
        // para_offset 누적 (수렴 감지용)
        while self.para_offset.len() <= section_idx {
            self.para_offset.push(0);
        }
        self.para_offset[section_idx] += 1;
    }

    /// composed 벡터에서 문단 항목을 제거한다 (문단 병합/삭제 후).
    pub(crate) fn remove_composed_paragraph(&mut self, section_idx: usize, para_idx: usize) {
        self.invalidate_page_tree_cache();
        if para_idx < self.composed[section_idx].len() {
            self.composed[section_idx].remove(para_idx);
        }
        self.mark_section_dirty(section_idx);
        // measured_sections: 인덱스 조정으로 기존 측정값 재사용
        if section_idx < self.measured_sections.len() {
            self.measured_sections[section_idx].shift_for_remove(para_idx);
        }
        // dirty_paragraphs: 병합 대상 문단만 dirty 표시
        if section_idx < self.dirty_paragraphs.len() {
            let para_count = self.document.sections[section_idx].paragraphs.len();
            let bits =
                self.dirty_paragraphs[section_idx].get_or_insert_with(|| vec![false; para_count]);
            if para_idx < bits.len() {
                bits.remove(para_idx);
            }
            bits.resize(para_count, false);
            // 병합 결과 문단 dirty
            if para_idx < bits.len() {
                bits[para_idx] = true;
            }
            if para_idx > 0 && para_idx - 1 < bits.len() {
                bits[para_idx - 1] = true;
            }
        }
        // para_offset 누적 (수렴 감지용)
        while self.para_offset.len() <= section_idx {
            self.para_offset.push(0);
        }
        self.para_offset[section_idx] -= 1;
    }

    /// 모든 구역을 dirty로 표시한다.
    pub(crate) fn mark_all_sections_dirty(&mut self) {
        for d in &mut self.dirty_sections {
            *d = true;
        }
    }

    /// Batch 모드가 아닐 때만 paginate를 실행한다.
    /// Command 메서드에서 self.paginate() 대신 호출한다.
    pub(crate) fn paginate_if_needed(&mut self) {
        if !self.batch_mode {
            self.paginate();
        }
    }

    /// 모든 구역을 페이지로 분할한다 (dirty 구역만 재처리, 증분 표 측정).
    ///
    /// [Task #1046] 사후 reflow(A) 접근은 페이지네이터↔렌더러 측정 드리프트로 인해
    /// overflow 를 줄이지 못함이 확인되어(stage3 findings) 단일 패스로 둔다. 근본 해소는
    /// 측정 통일(B). `paginate_pass` 의 `force_break_before` 훅과 `LayoutOverflow` 의
    /// section_index/is_first_in_column 계측은 측정 통일 작업의 진단·후속용으로 유지한다.
    pub(crate) fn paginate(&mut self) {
        let sec_count = self.document.sections.len().max(1);
        let empty_breaks: Vec<std::collections::HashSet<usize>> =
            vec![std::collections::HashSet::new(); sec_count];
        self.paginate_pass(&empty_breaks);
    }

    fn paginate_pass(&mut self, force_breaks: &[std::collections::HashSet<usize>]) {
        self.invalidate_page_tree_cache();
        let paginator = Paginator::new(self.dpi);
        let hwp3_origin_flow_spacing_before = uses_hwp3_origin_flow_spacing_before(&self.document);
        let is_hwpx_source = matches!(self.source_format, crate::parser::FileFormat::Hwpx);
        let measurer = HeightMeasurer::new(self.dpi)
            .with_hwp3_variant(self.document.is_hwp3_variant)
            .with_hwp3_origin_flow_spacing_before(hwp3_origin_flow_spacing_before);

        if self.document.sections.is_empty() {
            self.pagination.clear();
            self.measured_tables.clear();
            self.measured_sections.clear();
            let default_section = Section::default();
            let empty_composed: Vec<ComposedParagraph> = Vec::new();
            let measured = measurer.measure_section(
                &default_section.paragraphs,
                &empty_composed,
                &self.styles,
                None,
            );
            let result = paginator.paginate_with_measured(
                &default_section.paragraphs,
                &measured,
                &default_section.section_def.page_def,
                &ColumnDef::default(),
                0,
                &self.styles.para_styles,
            );
            self.pagination.push(result);
            self.measured_tables.push(measured.tables.clone());
            self.measured_sections.push(measured);
            return;
        }

        // 벡터 크기 동기화
        let sec_count = self.document.sections.len();
        while self.pagination.len() < sec_count {
            self.pagination.push(PaginationResult {
                pages: Vec::new(),
                wrap_around_paras: Vec::new(),
                hidden_empty_paras: std::collections::HashSet::new(),
                pre_emitted_host_paras: std::collections::HashSet::new(),
                endnotes: Vec::new(),
                endnote_paragraphs: Vec::new(),
                endnote_para_sources: Vec::new(),
                endnote_between_notes_hu: 0,
                endnote_separator_above_hu: 0,
                endnote_separator_below_hu: 0,
            });
        }
        self.pagination.truncate(sec_count);
        while self.para_column_map.len() < sec_count {
            self.para_column_map.push(Vec::new());
        }
        self.para_column_map.truncate(sec_count);
        while self.measured_tables.len() < sec_count {
            self.measured_tables.push(Vec::new());
        }
        self.measured_tables.truncate(sec_count);
        self.dirty_sections.resize(sec_count, true);
        self.dirty_sections.truncate(sec_count);
        while self.measured_sections.len() < sec_count {
            self.measured_sections.push(MeasuredSection {
                paragraphs: Vec::new(),
                tables: Vec::new(),
            });
        }
        self.measured_sections.truncate(sec_count);
        while self.dirty_paragraphs.len() < sec_count {
            self.dirty_paragraphs.push(None);
        }
        self.dirty_paragraphs.truncate(sec_count);

        // 구역 간 쪽번호 위치/번호 상속
        let mut carry_page_number_pos: Option<crate::model::control::PageNumberPos> = None;
        let mut carry_last_page_number: u32 = 0; // 이전 구역의 마지막 쪽번호

        // 구역 간 머리말/꼬리말 상속 (이전 구역의 홀수/짝수 페이지별 carry)
        use crate::renderer::pagination::HeaderFooterRef;
        let mut carry_header_odd: Option<HeaderFooterRef> = None;
        let mut carry_header_even: Option<HeaderFooterRef> = None;
        let mut carry_footer_odd: Option<HeaderFooterRef> = None;
        let mut carry_footer_even: Option<HeaderFooterRef> = None;
        let mut carry_master_odd: Option<MasterPageRef> = None;
        let mut carry_master_even: Option<MasterPageRef> = None;

        // [Task #1046] reflow force-break hint (구역별). reflow 루프(paginate)가 누적해 전달.
        let empty_breaks: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (idx, section) in self.document.sections.iter().enumerate() {
            if !self.dirty_sections[idx] {
                // dirty가 아닌 구역에서도 carry를 업데이트
                if let Some(pages) = self.pagination.get(idx) {
                    if let Some(last) = pages.pages.last() {
                        if last.page_number_pos.is_some() {
                            carry_page_number_pos = last.page_number_pos.clone();
                        }
                        carry_last_page_number = last.page_number;
                    }
                    // 머리말/꼬리말 carry 업데이트 (not-dirty 구역)
                    for page in &pages.pages {
                        let is_odd = page.page_number % 2 == 1;
                        if page.active_header.is_some() {
                            if is_odd {
                                carry_header_odd = page.active_header.clone();
                            } else {
                                carry_header_even = page.active_header.clone();
                            }
                        }
                        if page.active_footer.is_some() {
                            if is_odd {
                                carry_footer_odd = page.active_footer.clone();
                            } else {
                                carry_footer_even = page.active_footer.clone();
                            }
                        }
                    }
                }
                update_master_page_carry_from_section(
                    idx,
                    section,
                    &mut carry_master_odd,
                    &mut carry_master_even,
                );
                continue;
            }

            let composed = if idx < self.composed.len() {
                &self.composed[idx]
            } else {
                continue;
            };

            // 증분 측정: 이전 측정 데이터가 있으면 문단/표 수준 선택적 캐싱
            let measured = if !self.measured_sections[idx].paragraphs.is_empty() {
                let dirty_paras = self
                    .dirty_paragraphs
                    .get(idx)
                    .and_then(|opt| opt.as_deref());
                let column_def_sel = Self::find_initial_column_def(&section.paragraphs);
                let layout_sel = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                    &section.section_def.page_def,
                    &column_def_sel,
                    self.dpi,
                );
                let col_w_sel = layout_sel
                    .column_areas
                    .first()
                    .map(|a| a.width)
                    .unwrap_or(layout_sel.body_area.width);
                measurer.measure_section_selective(
                    &section.paragraphs,
                    composed,
                    &self.styles,
                    &self.measured_sections[idx],
                    dirty_paras,
                    Some(col_w_sel),
                )
            } else {
                let column_def_pre = Self::find_initial_column_def(&section.paragraphs);
                let layout_pre = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                    &section.section_def.page_def,
                    &column_def_pre,
                    self.dpi,
                );
                let col_w_pre = layout_pre
                    .column_areas
                    .first()
                    .map(|a| a.width)
                    .unwrap_or(layout_pre.body_area.width);
                measurer.measure_section(
                    &section.paragraphs,
                    composed,
                    &self.styles,
                    Some(col_w_pre),
                )
            };

            let hwp3_origin_page_tolerance =
                self.document.is_hwp3_variant || uses_hwp3_origin_page_tolerance(&self.document);
            let column_def = Self::find_initial_column_def(&section.paragraphs);
            // TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback 가능.
            let use_paginator = std::env::var("RHWP_USE_PAGINATOR")
                .map(|v| v == "1")
                .unwrap_or(false);
            let mut result = if use_paginator {
                paginator.paginate_with_measured_opts(
                    &section.paragraphs,
                    &measured,
                    &section.section_def.page_def,
                    &column_def,
                    idx,
                    &self.styles.para_styles,
                    crate::renderer::pagination::PaginationOpts {
                        hide_empty_line: section.section_def.hide_empty_line,
                        respect_vpos_reset: self.respect_vpos_reset,
                        is_hwp3_variant: self.document.is_hwp3_variant,
                        footnote_shape: Some(section.section_def.footnote_shape.clone()),
                    },
                )
            } else {
                use crate::renderer::typeset::TypesetEngine;
                let typesetter = TypesetEngine::new(self.dpi);
                typesetter.typeset_section_with_variant(
                    &section.paragraphs,
                    composed,
                    &self.styles,
                    &section.section_def.page_def,
                    &column_def,
                    idx,
                    &measured.tables,
                    section.section_def.hide_empty_line,
                    self.document.is_hwp3_variant,
                    hwp3_origin_flow_spacing_before,
                    hwp3_origin_page_tolerance,
                    Some(&section.section_def.footnote_shape),
                    Some(&section.section_def.endnote_shape),
                    force_breaks.get(idx).unwrap_or(&empty_breaks),
                    is_hwpx_source,
                )
            };

            // [Task #1086 Stage 3] HWP3 변환본의 표지성 구역은 한컴이 직전
            // 구역이 홀수 쪽에서 끝날 때 빈 쪽을 하나 삽입해 다음 구역 제목을
            // 홀수 쪽으로 맞춘다(hwpspec.hwp Part II). 본문 조판 규칙을 넓히지
            // 않도록 첫 문단의 표지/감추기/쪽나누기 패턴이 모두 맞는 경우만 보정한다.
            if should_insert_hwp3_title_filler_page(
                hwp3_origin_page_tolerance,
                idx,
                section,
                carry_last_page_number,
            ) {
                insert_hwp3_title_filler_page(&mut result, idx);
            }

            self.measured_tables[idx] = measured.tables.clone();
            self.measured_sections[idx] = measured;

            // ── 수렴 감지: 이전 페이지네이션과 비교하여 변경 흡수 지점 탐색 ──
            // 판단 후 검증: 수렴으로 복사한 결과를 full pagination과 비교하여 정합성 확인
            let offset = self.para_offset.get(idx).copied().unwrap_or(0);
            if offset != 0 {
                let old_result = &self.pagination[idx];
                if let Some(converge_page) = result.find_convergence(old_result, offset) {
                    let new_page_count = result.pages.len();
                    let old_page_count = old_result.pages.len();
                    if converge_page > 0
                        && converge_page < new_page_count
                        && converge_page < old_page_count
                    {
                        // 검증: full pagination에서 수렴 이후 페이지가 old+offset와 일치하는지 확인
                        let mut verified = true;
                        let check_end = result.pages.len().min(old_page_count);
                        for pi in converge_page..check_end {
                            let new_page = &result.pages[pi];
                            let old_page = &old_result.pages[pi];
                            let new_items: Vec<usize> = new_page
                                .column_contents
                                .iter()
                                .flat_map(|cc| cc.items.iter().map(|it| it.para_index()))
                                .collect();
                            let old_items: Vec<usize> = old_page
                                .column_contents
                                .iter()
                                .flat_map(|cc| {
                                    cc.items
                                        .iter()
                                        .map(|it| (it.para_index() as i64 + offset as i64) as usize)
                                })
                                .collect();
                            if new_items != old_items {
                                eprintln!("CONVERGENCE_VERIFY_FAIL: sec{} page {} new={:?} old+offset={:?}",
                                    idx, pi + 1, new_items, old_items);
                                verified = false;
                                break;
                            }
                        }
                        if result.pages.len() != old_page_count {
                            verified = false;
                        }

                        if verified {
                            eprintln!(
                                "CONVERGENCE: sec{} page {} 수렴 확인 ({}페이지 재사용 가능)",
                                idx,
                                converge_page + 1,
                                old_page_count - converge_page
                            );
                        }
                    }
                }
            }

            // 머리말/꼬리말 carry 업데이트:
            // 페이지 active_header뿐 아니라 구역에 정의된 머리말 컨트롤도 carry에 반영
            // (짝수 페이지가 없어도 Even 머리말이 다음 구역에 상속되어야 함)
            {
                use crate::model::header_footer::HeaderFooterApply as HFA;
                for (pi, para) in section.paragraphs.iter().enumerate() {
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        match ctrl {
                            Control::Header(h) => {
                                let r = HeaderFooterRef {
                                    para_index: pi,
                                    control_index: ci,
                                    source_section_index: idx,
                                };
                                match h.apply_to {
                                    HFA::Both => {
                                        carry_header_odd = Some(r.clone());
                                        carry_header_even = Some(r);
                                    }
                                    HFA::Odd => {
                                        carry_header_odd = Some(r);
                                    }
                                    HFA::Even => {
                                        carry_header_even = Some(r);
                                    }
                                }
                            }
                            Control::Footer(f) => {
                                let r = HeaderFooterRef {
                                    para_index: pi,
                                    control_index: ci,
                                    source_section_index: idx,
                                };
                                match f.apply_to {
                                    HFA::Both => {
                                        carry_footer_odd = Some(r.clone());
                                        carry_footer_even = Some(r);
                                    }
                                    HFA::Odd => {
                                        carry_footer_odd = Some(r);
                                    }
                                    HFA::Even => {
                                        carry_footer_even = Some(r);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 구역 간 page_number_pos 상속: 이 구역에 PageNumberPos가 없으면 이전 구역 값 사용
            let section_has_own_pnp = result.pages.iter().any(|p| p.page_number_pos.is_some());
            if !section_has_own_pnp {
                if let Some(ref prev_pnp) = carry_page_number_pos {
                    for page in &mut result.pages {
                        page.page_number_pos = Some(prev_pnp.clone());
                    }
                }
            }

            // 구역 간 쪽번호 연속: NewNumber(Page) 컨트롤이 없으면 이전 구역에서 이어짐
            if idx > 0 && carry_last_page_number > 0 {
                use crate::model::control::{AutoNumberType, Control};
                let has_new_number = section.paragraphs.iter().any(|p|
                    p.controls.iter().any(|c| matches!(c, Control::NewNumber(nn) if nn.number_type == AutoNumberType::Page))
                );
                if !has_new_number {
                    for page in &mut result.pages {
                        page.page_number += carry_last_page_number;
                    }
                }
            }

            // 맞쪽 편집 layout 은 구역 간 쪽번호 carry 이후 최종 page_number 기준으로 갱신한다.
            apply_page_number_layouts_for_section(&mut result, section);

            // 바탕쪽 선택은 구역 간 쪽번호 carry 보정 이후 최종 page_number 기준으로 수행한다.
            assign_master_pages_for_section(
                &mut result,
                idx,
                section,
                &carry_master_odd,
                &carry_master_even,
            );
            update_master_page_carry_from_section(
                idx,
                section,
                &mut carry_master_odd,
                &mut carry_master_even,
            );

            // 구역 간 머리말/꼬리말 상속 (쪽번호 보정 이후 실행)
            if idx > 0 {
                let section_has_header = result.pages.iter().any(|p| p.active_header.is_some());
                if !section_has_header {
                    for (pi, page) in result.pages.iter_mut().enumerate() {
                        let is_odd = page.page_number % 2 == 1;
                        page.active_header = if is_odd {
                            carry_header_odd
                                .clone()
                                .or_else(|| carry_header_even.clone())
                        } else {
                            carry_header_even
                                .clone()
                                .or_else(|| carry_header_odd.clone())
                        };
                    }
                }
                let section_has_footer = result.pages.iter().any(|p| p.active_footer.is_some());
                if !section_has_footer {
                    for (pi, page) in result.pages.iter_mut().enumerate() {
                        let is_odd = page.page_number % 2 == 1;
                        page.active_footer = if is_odd {
                            carry_footer_odd
                                .clone()
                                .or_else(|| carry_footer_even.clone())
                        } else {
                            carry_footer_even
                                .clone()
                                .or_else(|| carry_footer_odd.clone())
                        };
                    }
                }
            }
            // 첫 쪽 감추기: hide_header/hide_footer가 true이면 구역 첫 쪽의 머리말/꼬리말 제거
            if section.section_def.hide_header {
                if let Some(first_page) = result.pages.first_mut() {
                    first_page.active_header = None;
                }
            }
            if section.section_def.hide_footer {
                if let Some(first_page) = result.pages.first_mut() {
                    first_page.active_footer = None;
                }
            }

            // carry 업데이트
            if let Some(last) = result.pages.last() {
                if last.page_number_pos.is_some() {
                    carry_page_number_pos = last.page_number_pos.clone();
                }
                carry_last_page_number = last.page_number;
            }

            // 같은 구역 내 머리말/꼬리말 보정:
            // pagination에서 머리말이 누락된 페이지에 구역의 머리말 컨트롤을 직접 할당
            {
                use crate::model::header_footer::HeaderFooterApply as HFA;
                let mut sec_h_odd: Option<HeaderFooterRef> = None;
                let mut sec_h_even: Option<HeaderFooterRef> = None;
                let mut sec_h_both: Option<HeaderFooterRef> = None;
                let mut sec_f_odd: Option<HeaderFooterRef> = None;
                let mut sec_f_even: Option<HeaderFooterRef> = None;
                let mut sec_f_both: Option<HeaderFooterRef> = None;
                for (pi, para) in section.paragraphs.iter().enumerate() {
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        match ctrl {
                            Control::Header(h) => {
                                let r = HeaderFooterRef {
                                    para_index: pi,
                                    control_index: ci,
                                    source_section_index: idx,
                                };
                                match h.apply_to {
                                    HFA::Both => sec_h_both = Some(r),
                                    HFA::Even => sec_h_even = Some(r),
                                    HFA::Odd => sec_h_odd = Some(r),
                                }
                            }
                            Control::Footer(f) => {
                                let r = HeaderFooterRef {
                                    para_index: pi,
                                    control_index: ci,
                                    source_section_index: idx,
                                };
                                match f.apply_to {
                                    HFA::Both => sec_f_both = Some(r),
                                    HFA::Even => sec_f_even = Some(r),
                                    HFA::Odd => sec_f_odd = Some(r),
                                }
                            }
                            _ => {}
                        }
                    }
                }
                let has_hf = sec_h_odd.is_some()
                    || sec_h_even.is_some()
                    || sec_h_both.is_some()
                    || sec_f_odd.is_some()
                    || sec_f_even.is_some()
                    || sec_f_both.is_some();
                // 머리말/꼬리말이 정의된 문단이 시작되는 페이지를 찾아
                // 그 페이지부터만 적용 (정의 이전 페이지에는 미적용)
                use crate::renderer::pagination::PageItem;
                let hdr_start_page = [&sec_h_odd, &sec_h_even, &sec_h_both]
                    .iter()
                    .filter_map(|r| r.as_ref())
                    .map(|r| r.para_index)
                    .min()
                    .and_then(|hdr_pi| {
                        result.pages.iter().position(|p| {
                            p.column_contents.iter().any(|cc| {
                                cc.items.iter().any(|item| {
                                    let pi = match item {
                                        PageItem::FullParagraph { para_index } => Some(*para_index),
                                        PageItem::PartialParagraph { para_index, .. } => {
                                            Some(*para_index)
                                        }
                                        PageItem::Table { para_index, .. } => Some(*para_index),
                                        PageItem::PartialTable { para_index, .. } => {
                                            Some(*para_index)
                                        }
                                        PageItem::Shape { para_index, .. } => Some(*para_index),
                                        PageItem::EndnoteSeparator { .. } => None,
                                    };
                                    pi.is_some_and(|pi| pi >= hdr_pi)
                                })
                            })
                        })
                    })
                    .unwrap_or(0);
                let ftr_start_page = [&sec_f_odd, &sec_f_even, &sec_f_both]
                    .iter()
                    .filter_map(|r| r.as_ref())
                    .map(|r| r.para_index)
                    .min()
                    .and_then(|ftr_pi| {
                        result.pages.iter().position(|p| {
                            p.column_contents.iter().any(|cc| {
                                cc.items.iter().any(|item| {
                                    let pi = match item {
                                        PageItem::FullParagraph { para_index } => Some(*para_index),
                                        PageItem::PartialParagraph { para_index, .. } => {
                                            Some(*para_index)
                                        }
                                        PageItem::Table { para_index, .. } => Some(*para_index),
                                        PageItem::PartialTable { para_index, .. } => {
                                            Some(*para_index)
                                        }
                                        PageItem::Shape { para_index, .. } => Some(*para_index),
                                        PageItem::EndnoteSeparator { .. } => None,
                                    };
                                    pi.is_some_and(|pi| pi >= ftr_pi)
                                })
                            })
                        })
                    })
                    .unwrap_or(0);
                if has_hf {
                    for (page_idx, page) in result.pages.iter_mut().enumerate() {
                        let is_odd = page.page_number % 2 == 1;
                        if page.active_header.is_none() && page_idx >= hdr_start_page {
                            page.active_header = if is_odd {
                                sec_h_odd.clone().or_else(|| sec_h_both.clone())
                            } else {
                                sec_h_even.clone().or_else(|| sec_h_both.clone())
                            };
                        } else {
                            // pagination에서 할당된 머리말의 apply_to가 현재 페이지 홀짝과 맞지 않으면 교체
                            if let Some(ref hdr_ref) = page.active_header {
                                if let Some(para) = section.paragraphs.get(hdr_ref.para_index) {
                                    if let Some(Control::Header(h)) =
                                        para.controls.get(hdr_ref.control_index)
                                    {
                                        let correct = match h.apply_to {
                                            HFA::Both => true,
                                            HFA::Odd => is_odd,
                                            HFA::Even => !is_odd,
                                        };
                                        if !correct {
                                            page.active_header = if is_odd {
                                                sec_h_odd.clone().or_else(|| sec_h_both.clone())
                                            } else {
                                                sec_h_even.clone().or_else(|| sec_h_both.clone())
                                            };
                                        }
                                    }
                                }
                            }
                        }
                        if page.active_footer.is_none() && page_idx >= ftr_start_page {
                            page.active_footer = if is_odd {
                                sec_f_odd.clone().or_else(|| sec_f_both.clone())
                            } else {
                                sec_f_even.clone().or_else(|| sec_f_both.clone())
                            };
                        } else {
                            if let Some(ref ftr_ref) = page.active_footer {
                                if let Some(para) = section.paragraphs.get(ftr_ref.para_index) {
                                    if let Some(Control::Footer(f)) =
                                        para.controls.get(ftr_ref.control_index)
                                    {
                                        let correct = match f.apply_to {
                                            HFA::Both => true,
                                            HFA::Odd => is_odd,
                                            HFA::Even => !is_odd,
                                        };
                                        if !correct {
                                            page.active_footer = if is_odd {
                                                sec_f_odd.clone().or_else(|| sec_f_both.clone())
                                            } else {
                                                sec_f_even.clone().or_else(|| sec_f_both.clone())
                                            };
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // 같은 구역 머리말/꼬리말 보정이 첫쪽 감춤을 되돌리지 않도록 마지막에 재적용한다.
            if let Some(first_page) = result.pages.first_mut() {
                if section.section_def.hide_header {
                    first_page.active_header = None;
                }
                if section.section_def.hide_footer {
                    first_page.active_footer = None;
                }
            }
            self.pagination[idx] = result;
            self.dirty_sections[idx] = false;
            // 문단 dirty 비트맵 초기화 (모든 문단 clean)
            let para_count = section.paragraphs.len();
            self.dirty_paragraphs[idx] = Some(vec![false; para_count]);

            // 문단→단 인덱스 매핑 구축
            {
                use crate::renderer::pagination::PageItem;
                let mut col_map = vec![0u16; para_count];
                for page in &self.pagination[idx].pages {
                    for col_content in &page.column_contents {
                        let ci = col_content.column_index;
                        for item in &col_content.items {
                            let pi = match item {
                                PageItem::FullParagraph { para_index } => *para_index,
                                PageItem::PartialParagraph { para_index, .. } => *para_index,
                                PageItem::Table { para_index, .. } => *para_index,
                                PageItem::PartialTable { para_index, .. } => *para_index,
                                PageItem::Shape { para_index, .. } => *para_index,
                                PageItem::EndnoteSeparator { .. } => usize::MAX,
                            };
                            if pi < col_map.len() {
                                col_map[pi] = ci;
                            }
                        }
                    }
                }
                self.para_column_map[idx] = col_map;
            }
        }

        // para_offset 리셋 (수렴 감지 완료)
        for off in &mut self.para_offset {
            *off = 0;
        }

        // 표 dirty 플래그 초기화
        for section in &mut self.document.sections {
            for para in &mut section.paragraphs {
                for ctrl in &mut para.controls {
                    if let Control::Table(table) = ctrl {
                        table.dirty = false;
                    }
                }
            }
        }
    }

    /// 글로벌 페이지 번호로 해당 페이지와 문단/구성 목록을 찾는다.
    pub(crate) fn find_page(
        &self,
        page_num: u32,
    ) -> Result<
        (
            &crate::renderer::pagination::PageContent,
            &[crate::model::paragraph::Paragraph],
            &[ComposedParagraph],
        ),
        HwpError,
    > {
        let mut offset = 0u32;
        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            if (page_num as usize) < offset as usize + pr.pages.len() {
                let local_idx = (page_num - offset) as usize;
                let paragraphs = if sec_idx < self.document.sections.len() {
                    &self.document.sections[sec_idx].paragraphs[..]
                } else {
                    &[][..]
                };
                let composed = if sec_idx < self.composed.len() {
                    &self.composed[sec_idx][..]
                } else {
                    &[][..]
                };
                return Ok((&pr.pages[local_idx], paragraphs, composed));
            }
            offset += pr.pages.len() as u32;
        }
        Err(HwpError::PageOutOfRange(page_num))
    }

    /// 페이지네이션 결과를 텍스트로 덤프 (디버깅용).
    /// page_filter: None이면 전체, Some(n)이면 해당 페이지만.
    pub fn dump_page_items(&self, page_filter: Option<u32>) -> String {
        use crate::model::control::Control;
        use crate::renderer::hwpunit_to_px;
        use crate::renderer::pagination::PageItem;

        let dpi = self.dpi;
        let mut out = String::new();
        let mut global_page = 0u32;

        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            let body_paragraphs = self
                .document
                .sections
                .get(sec_idx)
                .map(|s| &s.paragraphs[..])
                .unwrap_or(&[]);
            // [Task #1082] 미주 paragraphs 를 본문 뒤에 합쳐 인덱싱 정합.
            // 미주 PageItem 의 para_index = body_paragraphs.len() + 미주 로컬 인덱스
            // (typeset.rs en_para_idx 와 동일). 합치지 않으면 미주 항목이 out-of-bounds 로
            // 본문이 빈 것처럼 표시된다(시험지 다단 미주 디버깅 차단).
            let body_len = body_paragraphs.len();
            let combined: Vec<Paragraph>;
            let paragraphs: &[Paragraph] = if pr.endnote_paragraphs.is_empty() {
                body_paragraphs
            } else {
                combined = body_paragraphs
                    .iter()
                    .chain(pr.endnote_paragraphs.iter())
                    .cloned()
                    .collect();
                &combined
            };
            let measured = self.measured_sections.get(sec_idx);

            // [Task #1700] cc.items 에 없는 문단(어울림 흡수 / 빈 줄 감춤)을 페이지 PI 로 표면화
            // 하기 위한 사전 패스. 한글(OLE)은 이들을 본문 문단으로 카운트하므로 문단→페이지
            // 매핑이 정합된다. 레이아웃/렌더 트리는 변경하지 않으므로 기하·페이지 수·시각 불변.
            // 1) items 문단 → 표시 페이지(global+1) 매핑 — **마지막** 출현 페이지(다중 페이지 표는
            //    끝 페이지에 빈 문단이 따라오므로 last-page 가 한글과 정합).
            let sec_start_page = global_page;
            let mut item_disp_page: std::collections::HashMap<usize, u32> =
                std::collections::HashMap::new();
            // [#1705] 표의 첫(앵커) 출현 페이지 — 어울림 빈 문단이 표 옆 wrap zone 에 있을 때 사용.
            let mut item_first_page: std::collections::HashMap<usize, u32> =
                std::collections::HashMap::new();
            for (li, page) in pr.pages.iter().enumerate() {
                let disp = sec_start_page + li as u32 + 1;
                for cc in &page.column_contents {
                    for item in &cc.items {
                        let pidx = match item {
                            PageItem::FullParagraph { para_index }
                            | PageItem::PartialParagraph { para_index, .. }
                            | PageItem::Table { para_index, .. }
                            | PageItem::PartialTable { para_index, .. }
                            | PageItem::Shape { para_index, .. } => Some(*para_index),
                            _ => None,
                        };
                        if let Some(p) = pidx {
                            item_disp_page.insert(p, disp); // 마지막(끝) 페이지
                            item_first_page.entry(p).or_insert(disp); // 첫(앵커) 페이지
                        }
                    }
                }
            }
            // 2) 표면화 대상을 페이지별로 그룹화: (para_index, is_wrap, table_pi).
            //    - 어울림(wrap-around): 앵커 표(table_para_index)의 페이지에 귀속.
            //    - 빈 줄 감춤(hidden_empty_paras): 직전 item 문단(대개 표)의 페이지에 귀속.
            let mut extra_by_page: std::collections::HashMap<u32, Vec<(usize, bool, usize)>> =
                std::collections::HashMap::new();
            let mut emitted_extra: std::collections::HashSet<usize> =
                std::collections::HashSet::new();
            for page in pr.pages.iter() {
                for cc in &page.column_contents {
                    for wp in &cc.wrap_around_paras {
                        if !emitted_extra.insert(wp.para_index) {
                            continue;
                        }
                        // [#1705] 어울림(floating) 표의 트레일링 빈 문단 귀속:
                        //   line_seg 폭(sw)이 좁으면(표 옆 wrap zone) 표의 **첫(앵커) 페이지**,
                        //   전체 폭이면(표 아래로 흐름) 표의 **마지막 페이지**.
                        //   한글은 wrap zone 문단을 앵커 페이지에, 전체폭 문단을 표 끝 페이지에 둔다.
                        let sw_px = paragraphs
                            .get(wp.para_index)
                            .and_then(|p| p.line_segs.first())
                            .map(|s| hwpunit_to_px(s.segment_width as i32, dpi))
                            .unwrap_or(0.0);
                        let body_w = page.layout.body_area.width;
                        let is_wrap_zone = sw_px > 0.0 && sw_px < body_w * 0.9;
                        let disp = if is_wrap_zone {
                            item_first_page.get(&wp.table_para_index)
                        } else {
                            item_disp_page.get(&wp.table_para_index)
                        }
                        .copied()
                        .unwrap_or(sec_start_page + 1);
                        extra_by_page.entry(disp).or_default().push((
                            wp.para_index,
                            true,
                            wp.table_para_index,
                        ));
                    }
                }
            }
            let mut hidden_sorted: Vec<usize> = pr
                .hidden_empty_paras
                .iter()
                .copied()
                .filter(|p| !item_disp_page.contains_key(p) && !emitted_extra.contains(p))
                .collect();
            hidden_sorted.sort_unstable();
            for hidx in hidden_sorted {
                let mut disp = sec_start_page + 1;
                for j in (0..hidx).rev() {
                    if let Some(d) = item_disp_page.get(&j) {
                        disp = *d;
                        break;
                    }
                }
                extra_by_page
                    .entry(disp)
                    .or_default()
                    .push((hidx, false, 0));
                emitted_extra.insert(hidx);
            }

            for (local_idx, page) in pr.pages.iter().enumerate() {
                if let Some(pf) = page_filter {
                    if global_page != pf {
                        global_page += 1;
                        continue;
                    }
                }

                out.push_str(&format!(
                    "\n=== 페이지 {} (global_idx={}, section={}, page_num={}) ===\n",
                    global_page + 1,
                    global_page,
                    sec_idx,
                    page.page_number
                ));

                // 페이지 레이아웃 정보
                let la = &page.layout;
                out.push_str(&format!(
                    "  body_area: x={:.1} y={:.1} w={:.1} h={:.1}\n",
                    la.body_area.x, la.body_area.y, la.body_area.width, la.body_area.height
                ));

                for (col_idx, cc) in page.column_contents.iter().enumerate() {
                    let hwp_used_px = compute_hwp_used_height(cc, paragraphs, dpi);
                    let diff_str = match hwp_used_px {
                        Some(hwp) => format!(
                            ", hwp_used≈{:.1}px, diff={:+.1}px",
                            hwp,
                            cc.used_height - hwp
                        ),
                        None => String::new(),
                    };
                    out.push_str(&format!(
                        "  단 {} (items={}, used={:.1}px{}{})\n",
                        col_idx,
                        cc.items.len(),
                        cc.used_height,
                        diff_str,
                        if cc.zone_y_offset > 0.0 {
                            format!(", zone_y_offset={:.1}", cc.zone_y_offset)
                        } else {
                            String::new()
                        }
                    ));

                    for item in &cc.items {
                        let endnote_source_info = |para_index: usize| -> String {
                            if para_index < body_len {
                                return String::new();
                            }
                            let local = para_index - body_len;
                            pr.endnote_para_sources
                                .get(local)
                                .map(|src| {
                                    format!(
                                        " src=s{}:p{}:ci{}:note{}",
                                        src.section_index,
                                        src.para_index,
                                        src.control_index,
                                        src.note_para_index
                                    )
                                })
                                .unwrap_or_default()
                        };

                        match item {
                            PageItem::FullParagraph { para_index } => {
                                let text_preview = paragraphs
                                    .get(*para_index)
                                    .map(|p| {
                                        let t: String = p
                                            .text
                                            .chars()
                                            .filter(|c| *c > '\u{001F}')
                                            .take(40)
                                            .collect();
                                        if t.is_empty() {
                                            "(빈)".to_string()
                                        } else {
                                            t
                                        }
                                    })
                                    .unwrap_or_default();
                                let height = measured
                                    .and_then(|m| m.get_measured_paragraph(*para_index))
                                    .map(|mp| {
                                        let sb = mp.spacing_before;
                                        let sa = mp.spacing_after;
                                        let lh_sum: f64 = mp.line_heights.iter().sum();
                                        let ls_sum: f64 = mp.line_spacings.iter().sum();
                                        let lines = lh_sum + ls_sum;
                                        format!(
                                            "h={:.1} (sb={:.1} lines={:.1} lh={:.1} ls={:.1} sa={:.1})",
                                            sb + lines + sa,
                                            sb,
                                            lines,
                                            lh_sum,
                                            ls_sum,
                                            sa
                                        )
                                    })
                                    .unwrap_or_default();
                                let vpos_info =
                                    format_vpos_range(paragraphs.get(*para_index), None, None);
                                let line_seg_info =
                                    format_line_seg_brief(paragraphs.get(*para_index));
                                let source_info = endnote_source_info(*para_index);
                                let kind = if *para_index >= body_len {
                                    "FullParagraph[미주]"
                                } else {
                                    "FullParagraph"
                                };
                                out.push_str(&format!(
                                    "    {}  pi={}  {}  {}{}  {}  \"{}\"\n",
                                    kind,
                                    para_index,
                                    height,
                                    vpos_info,
                                    source_info,
                                    line_seg_info,
                                    text_preview
                                ));
                            }
                            PageItem::PartialParagraph {
                                para_index,
                                start_line,
                                end_line,
                            } => {
                                let vpos_info = format_vpos_range(
                                    paragraphs.get(*para_index),
                                    Some(*start_line),
                                    Some(*end_line),
                                );
                                let line_seg_info =
                                    format_line_seg_brief(paragraphs.get(*para_index));
                                let source_info = endnote_source_info(*para_index);
                                out.push_str(&format!(
                                    "    PartialParagraph  pi={}  lines={}..{}  {}{}  {}\n",
                                    para_index,
                                    start_line,
                                    end_line,
                                    vpos_info,
                                    source_info,
                                    line_seg_info
                                ));
                            }
                            PageItem::Table {
                                para_index,
                                control_index,
                            } => {
                                let table_info = paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| {
                                        if let Control::Table(t) = c {
                                            let h = hwpunit_to_px(t.common.height as i32, dpi);
                                            let w = hwpunit_to_px(t.common.width as i32, dpi);
                                            format!(
                                                "{}x{}  {:.1}x{:.1}px  wrap={:?} tac={}",
                                                t.row_count,
                                                t.col_count,
                                                w,
                                                h,
                                                t.common.text_wrap,
                                                t.common.treat_as_char
                                            )
                                        } else {
                                            String::new()
                                        }
                                    })
                                    .unwrap_or_default();
                                let vpos_info =
                                    format_vpos_range(paragraphs.get(*para_index), None, None);
                                let line_seg_info =
                                    format_line_seg_brief(paragraphs.get(*para_index));
                                let source_info = endnote_source_info(*para_index);
                                out.push_str(&format!(
                                    "    Table          pi={} ci={}  {}  {}{}  {}\n",
                                    para_index,
                                    control_index,
                                    table_info,
                                    vpos_info,
                                    source_info,
                                    line_seg_info
                                ));
                            }
                            PageItem::PartialTable {
                                para_index,
                                control_index,
                                start_row,
                                end_row,
                                is_continuation,
                                start_cut,
                                end_cut,
                                ..
                            } => {
                                let table_info = paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| {
                                        if let Control::Table(t) = c {
                                            format!("{}x{}", t.row_count, t.col_count)
                                        } else {
                                            String::new()
                                        }
                                    })
                                    .unwrap_or_default();
                                let vpos_info =
                                    format_vpos_range(paragraphs.get(*para_index), None, None);
                                let line_seg_info =
                                    format_line_seg_brief(paragraphs.get(*para_index));
                                let source_info = endnote_source_info(*para_index);
                                // [Task #993] 분할 표 진단 정보 — 행 컷이 비어 있지
                                // 않으면 셀 내 분할(컷 = 셀별 소비 유닛 수).
                                let split_info = if !start_cut.is_empty() || !end_cut.is_empty() {
                                    format!("  start_cut={:?} end_cut={:?}", start_cut, end_cut)
                                } else {
                                    String::new()
                                };
                                out.push_str(&format!("    PartialTable   pi={} ci={}  rows={}..{}  cont={}  {}  {}{}  {}{}\n",
                                    para_index, control_index, start_row, end_row, is_continuation, table_info, vpos_info, source_info, line_seg_info, split_info));
                            }
                            PageItem::Shape {
                                para_index,
                                control_index,
                            } => {
                                let shape_info = paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| match c {
                                        Control::Shape(s) => format!(
                                            "wrap={:?} tac={}",
                                            s.common().text_wrap,
                                            s.common().treat_as_char
                                        ),
                                        Control::Picture(p) => {
                                            format!("그림 tac={}", p.common.treat_as_char)
                                        }
                                        Control::Equation(_) => "수식".to_string(),
                                        _ => String::new(),
                                    })
                                    .unwrap_or_default();
                                let vpos_info =
                                    format_vpos_range(paragraphs.get(*para_index), None, None);
                                let line_seg_info =
                                    format_line_seg_brief(paragraphs.get(*para_index));
                                let source_info = endnote_source_info(*para_index);
                                out.push_str(&format!(
                                    "    Shape          pi={} ci={}  {}  {}{}  {}\n",
                                    para_index,
                                    control_index,
                                    shape_info,
                                    vpos_info,
                                    source_info,
                                    line_seg_info
                                ));
                            }
                            PageItem::EndnoteSeparator {
                                separator_length,
                                margin_above,
                                margin_below,
                                line_width,
                                color,
                                ..
                            } => {
                                out.push_str(&format!(
                                    "    EndnoteSeparator len={} above={} below={} width={} color=#{:06x}\n",
                                    separator_length, margin_above, margin_below, line_width, color & 0x00ff_ffff
                                ));
                            }
                        }
                    }
                }

                // [Task #1700] cc.items 에 없는 문단(어울림 / 빈 줄 감춤) 표면화.
                let disp_page = global_page + 1;
                if let Some(list) = extra_by_page.get(&disp_page) {
                    for (pidx, is_wrap, tpi) in list {
                        let preview = paragraphs
                            .get(*pidx)
                            .map(|p| {
                                let t: String = p
                                    .text
                                    .chars()
                                    .filter(|c| *c > '\u{001F}')
                                    .take(40)
                                    .collect();
                                if t.is_empty() {
                                    "(빈)".to_string()
                                } else {
                                    t
                                }
                            })
                            .unwrap_or_default();
                        if *is_wrap {
                            out.push_str(&format!(
                                "    WrapAroundPara  pi={}  table_pi={}  \"{}\"\n",
                                pidx, tpi, preview
                            ));
                        } else {
                            out.push_str(&format!(
                                "    HiddenEmptyPara  pi={}  \"{}\"\n",
                                pidx, preview
                            ));
                        }
                    }
                }

                global_page += 1;
            }
        }

        out
    }

    /// 페이지 렌더 트리 캐시 무효화 (from_page 이상 페이지만 무효화).
    pub(crate) fn invalidate_page_tree_cache_from(&self, from_page: u32) {
        let mut cache = self.page_tree_cache.borrow_mut();
        let from = from_page as usize;
        for i in from..cache.len() {
            cache[i] = None;
        }
    }

    /// 페이지 렌더 트리 캐시 전체 무효화.
    pub(crate) fn invalidate_page_tree_cache(&self) {
        self.page_tree_cache.borrow_mut().clear();
    }

    /// 캐시된 페이지 렌더 트리를 반환한다 (캐시 미스 시 빌드 후 캐시).
    pub(crate) fn build_page_tree_cached(&self, page_num: u32) -> Result<PageRenderTree, HwpError> {
        let idx = page_num as usize;

        // 캐시 크기 확보 + 히트 확인
        {
            let mut cache = self.page_tree_cache.borrow_mut();
            if cache.len() <= idx {
                cache.resize_with(idx + 1, || None);
            }
            if let Some(ref tree) = cache[idx] {
                return Ok(tree.clone());
            }
        }

        // 캐시 미스 → 빌드
        let tree = self.build_page_tree(page_num)?;
        let cloned = tree.clone();

        {
            let mut cache = self.page_tree_cache.borrow_mut();
            if cache.len() <= idx {
                cache.resize_with(idx + 1, || None);
            }
            cache[idx] = Some(cloned);
        }

        Ok(tree)
    }

    /// 페이지 렌더 트리를 빌드한다.
    pub(crate) fn build_page_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError> {
        use crate::model::style::HeadType;
        use crate::renderer::layout::resolve_numbering_id;
        use crate::renderer::pagination::PageItem;

        self.layout_engine
            .set_show_transparent_borders(self.show_transparent_borders);
        self.layout_engine.set_clip_enabled(self.clip_enabled);
        self.layout_engine
            .set_show_control_codes(self.show_control_codes);
        self.layout_engine
            .set_hwp3_variant(self.document.is_hwp3_variant);
        self.layout_engine.set_hwp3_origin_flow_spacing_before(
            uses_hwp3_origin_flow_spacing_before(&self.document),
        );
        self.layout_engine.set_hwpx_source(matches!(
            self.source_format,
            crate::parser::FileFormat::Hwpx
        ));
        self.layout_engine.set_hwpx_page_preview(
            self.document
                .hwpx_aux_entry("Preview/PrvImage.png")
                .filter(|_| matches!(self.source_format, crate::parser::FileFormat::Hwpx)),
        );
        // 활성 필드 정보를 레이아웃 엔진에 전달 (안내문 숨김용)
        self.layout_engine
            .set_active_field(self.active_field.as_ref().map(|af| {
                (
                    af.section_idx,
                    af.para_idx,
                    af.control_idx,
                    af.cell_path.clone(),
                )
            }));
        let (page_content, paragraphs, composed) = self.find_page(page_num)?;
        // 구역의 각주 모양 정보
        let footnote_shape = if page_content.section_index < self.document.sections.len() {
            &self.document.sections[page_content.section_index]
                .section_def
                .footnote_shape
        } else {
            &crate::model::footnote::FootnoteShape::default()
        };
        // 활성 바탕쪽 조회
        let active_mp = page_content.active_master_page.as_ref().and_then(|mp_ref| {
            self.document
                .sections
                .get(mp_ref.section_index)
                .and_then(|s| s.section_def.master_pages.get(mp_ref.master_page_index))
        });
        // 확장 바탕쪽 조회
        let extra_mps: Vec<&crate::model::header_footer::MasterPage> = page_content
            .extra_master_pages
            .iter()
            .filter_map(|mp_ref| {
                self.document
                    .sections
                    .get(mp_ref.section_index)
                    .and_then(|s| s.section_def.master_pages.get(mp_ref.master_page_index))
            })
            .collect();
        let sec_measured = self
            .measured_tables
            .get(page_content.section_index)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        // 쪽 테두리/배경 조회
        let page_border_fill = self
            .document
            .sections
            .get(page_content.section_index)
            .map(|s| &s.section_def.page_border_fill);
        let outline_num_id = self
            .document
            .sections
            .get(page_content.section_index)
            .map(|s| s.section_def.outline_numbering_id)
            .unwrap_or(0);

        // 번호 상태 리셋 후, 이 페이지 이전의 번호 문단을 재계산하여 카운터 복원
        // (이전 구역 + 현재 구역의 이전 페이지 모두 포함하여 구역 간 번호 연속 지원)
        self.layout_engine.reset_numbering_state();
        let sec_idx = page_content.section_index;
        let sec_page_offset: usize = self
            .pagination
            .iter()
            .take(sec_idx)
            .map(|p| p.pages.len())
            .sum();
        let local_idx = (page_num as usize).saturating_sub(sec_page_offset);
        {
            let mut replayed_partial: std::collections::HashSet<(usize, usize)> =
                std::collections::HashSet::new();
            // 이전 구역들의 모든 페이지 replay
            for prev_sec in 0..sec_idx {
                if let Some(pr) = self.pagination.get(prev_sec) {
                    let prev_paras = &self.document.sections[prev_sec].paragraphs;
                    let prev_outline_id = self.document.sections[prev_sec]
                        .section_def
                        .outline_numbering_id;
                    for pg in &pr.pages {
                        self.replay_numbering_page(
                            pg,
                            prev_paras,
                            prev_outline_id,
                            &mut replayed_partial,
                            prev_sec,
                        );
                    }
                }
            }
            // 현재 구역의 이전 페이지들 replay
            if let Some(pr) = self.pagination.get(sec_idx) {
                for prev_local in 0..local_idx {
                    if let Some(prev_page) = pr.pages.get(prev_local) {
                        self.replay_numbering_page(
                            prev_page,
                            paragraphs,
                            outline_num_id,
                            &mut replayed_partial,
                            sec_idx,
                        );
                    }
                }
            }
        }

        // 머리말/꼬리말이 상속된 경우 원본 구역의 문단을 조회 (source_section_index 기반)
        let header_paragraphs: &[crate::model::paragraph::Paragraph] = page_content
            .active_header
            .as_ref()
            .and_then(|hf| self.document.sections.get(hf.source_section_index))
            .map(|s| s.paragraphs.as_slice())
            .unwrap_or(paragraphs);
        let footer_paragraphs: &[crate::model::paragraph::Paragraph] = page_content
            .active_footer
            .as_ref()
            .and_then(|hf| self.document.sections.get(hf.source_section_index))
            .map(|s| s.paragraphs.as_slice())
            .unwrap_or(paragraphs);

        // 머리말/꼬리말 감추기 세트를 레이아웃 엔진에 전달
        self.layout_engine
            .set_hidden_header_footer(&self.hidden_header_footer);

        // 총 쪽수·파일 이름을 레이아웃 엔진에 전달 (머리말/꼬리말 필드 치환용)
        let total_pages: u32 = self.pagination.iter().map(|p| p.pages.len() as u32).sum();
        self.layout_engine.set_total_pages(total_pages);
        self.layout_engine.set_file_name(&self.file_name);

        let wrap_around_paras = self
            .pagination
            .get(sec_idx)
            .map(|pr| pr.wrap_around_paras.as_slice())
            .unwrap_or(&[]);

        // 빈 줄 감추기 문단 집합을 레이아웃 엔진에 전달
        if let Some(pr) = self.pagination.get(sec_idx) {
            self.layout_engine
                .set_hidden_empty_paras(&pr.hidden_empty_paras);
            // [Task #1755] pre-emit 된 host 문단 → fragment 쪽 host 렌더 억제.
            self.layout_engine
                .set_pre_emitted_host_paras(&pr.pre_emitted_host_paras);
            self.layout_engine
                .set_endnote_para_sources(paragraphs.len(), &pr.endnote_para_sources);
            // 섹션 미주 모양의 정규화 여백 전달 → HeightCursor min-gap 및 renderer overflow 판정.
            self.layout_engine.set_endnote_shape_margins_hu(
                pr.endnote_separator_above_hu,
                pr.endnote_between_notes_hu,
                pr.endnote_separator_below_hu,
            );
        }

        // [Task #836] 미주 paragraphs를 본문 paragraphs 뒤에 합쳐서 전달
        // endnote para_index = paragraphs.len() + idx → combined에서 접근 가능
        let en_paras = self
            .pagination
            .get(sec_idx)
            .map(|pr| pr.endnote_paragraphs.as_slice())
            .unwrap_or(&[]);
        let combined_paragraphs: Vec<Paragraph>;
        let combined_composed: Vec<crate::renderer::composer::ComposedParagraph>;
        let (render_paragraphs, render_composed): (
            &[Paragraph],
            &[crate::renderer::composer::ComposedParagraph],
        ) = if en_paras.is_empty() {
            (paragraphs, composed)
        } else {
            combined_paragraphs = paragraphs.iter().chain(en_paras.iter()).cloned().collect();
            let en_composed: Vec<_> = en_paras
                .iter()
                .map(|p| crate::renderer::composer::compose_paragraph(p))
                .collect();
            combined_composed = composed
                .iter()
                .cloned()
                .chain(en_composed.into_iter())
                .collect();
            (&combined_paragraphs, &combined_composed)
        };

        let mut tree = self.layout_engine.build_render_tree(
            page_content,
            render_paragraphs,
            header_paragraphs,
            footer_paragraphs,
            render_composed,
            &self.styles,
            footnote_shape,
            &self.document.bin_data_content,
            active_mp,
            sec_measured,
            page_border_fill,
            outline_num_id,
            wrap_around_paras,
        );
        // 확장 바탕쪽 추가 렌더링
        for ext_mp in &extra_mps {
            self.layout_engine.build_master_page_into(
                &mut tree,
                Some(*ext_mp),
                &page_content.layout,
                composed,
                &self.styles,
                &self.document.bin_data_content,
                page_content.section_index,
                page_content.page_number,
            );
        }
        // Task #1154: 동일 bin_data_id Pic 컨트롤이 수직으로 인접 겹쳐 그려질 때
        // 두 그림의 미세한 세로 스케일 차이로 인한 잔상(이중 라인) 제거.
        // build 직후 1회만 적용 — 모든 렌더러(SVG/Canvas/Skia/HTML/Layer) 공통.
        tree.clip_overlapping_same_bin_images();
        Ok(tree)
    }

    /// 한 페이지의 번호 문단을 replay하여 카운터를 전진시킨다.
    fn replay_numbering_page(
        &self,
        page: &crate::renderer::pagination::PageContent,
        paras: &[crate::model::paragraph::Paragraph],
        outline_num_id: u16,
        replayed: &mut std::collections::HashSet<(usize, usize)>,
        sec_idx: usize,
    ) {
        use crate::model::style::HeadType;
        use crate::renderer::pagination::PageItem;
        for col in &page.column_contents {
            for item in &col.items {
                let pi = match item {
                    PageItem::FullParagraph { para_index }
                    | PageItem::PartialParagraph { para_index, .. }
                    | PageItem::Table { para_index, .. } => {
                        if replayed.insert((sec_idx, *para_index)) {
                            Some(*para_index)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(pi) = pi {
                    if let Some(para) = paras.get(pi) {
                        if let Some(ps) = self.styles.para_styles.get(para.para_shape_id as usize) {
                            if ps.head_type == HeadType::Outline || ps.head_type == HeadType::Number
                            {
                                let nid = crate::renderer::layout::resolve_numbering_id(
                                    ps.head_type,
                                    ps.numbering_id,
                                    outline_num_id,
                                );
                                if nid > 0 {
                                    self.layout_engine.advance_numbering(nid, ps.para_level);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn rebuild_section(&mut self, section_idx: usize) {
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
        self.recompose_section(section_idx);
        self.paginate();
    }

    /// 페이지에 렌더된 텍스트를 줄 단위로 추출한다.
    ///
    /// TextLine 노드의 시각적 순서를 유지하며 TextRun/각주 마커/양식 텍스트를 합친다.
    pub fn extract_page_text_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn collect_line_text(node: &RenderNode, out: &mut String, has_token: &mut bool) {
            match &node.node_type {
                RenderNodeType::TextRun(tr) => {
                    out.push_str(&tr.text);
                    *has_token = true;
                }
                RenderNodeType::FootnoteMarker(marker) => {
                    out.push_str(&marker.text);
                    *has_token = true;
                }
                RenderNodeType::FormObject(form) => {
                    if !form.text.is_empty() {
                        out.push_str(&form.text);
                        *has_token = true;
                    } else if !form.caption.is_empty() {
                        out.push_str(&form.caption);
                        *has_token = true;
                    }
                }
                _ => {}
            }

            for child in &node.children {
                collect_line_text(child, out, has_token);
            }
        }

        fn collect_page_lines(node: &RenderNode, lines: &mut Vec<String>) {
            if matches!(node.node_type, RenderNodeType::TextLine(_)) {
                let mut line = String::new();
                let mut has_token = false;

                for child in &node.children {
                    collect_line_text(child, &mut line, &mut has_token);
                }

                if has_token {
                    lines.push(line);
                }
                return;
            }

            for child in &node.children {
                collect_page_lines(child, lines);
            }
        }

        let tree = self.build_page_tree(page_num)?;
        let mut lines = Vec::new();
        collect_page_lines(&tree.root, &mut lines);
        Ok(lines.join("\n"))
    }

    /// 페이지를 Markdown으로 추출한다.
    ///
    /// - 일반 텍스트는 줄 단위로 추출
    /// - 표 컨트롤은 Markdown table(`| ... |`)로 변환
    /// - 표/이미지는 내부 텍스트 중복 추출을 방지하기 위해 하위 순회를 생략
    /// - 이미지는 `[[RHWP_IMAGE:n]]` 토큰으로 출력되며,
    ///   반환 벡터에 같은 순서로 `(sec_idx, para_idx, control_idx, bin_data_id)`가 담긴다.
    ///   (sec/para/ctrl은 없을 수 있으며, 이 경우 bin_data_id로 추출한다)
    pub fn extract_page_markdown_with_images_native(
        &self,
        page_num: u32,
    ) -> Result<
        (
            String,
            Vec<(Option<usize>, Option<usize>, Option<usize>, u16)>,
        ),
        HwpError,
    > {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        #[derive(Debug)]
        enum MarkdownItem {
            Line(String),
            Table(String),
            Image {
                sec_idx: Option<usize>,
                para_idx: Option<usize>,
                control_idx: Option<usize>,
                bin_data_id: u16,
            },
        }

        fn collect_line_text(node: &RenderNode, out: &mut String, has_token: &mut bool) {
            match &node.node_type {
                RenderNodeType::TextRun(tr) => {
                    out.push_str(&tr.text);
                    *has_token = true;
                }
                RenderNodeType::FootnoteMarker(marker) => {
                    out.push_str(&marker.text);
                    *has_token = true;
                }
                RenderNodeType::FormObject(form) => {
                    if !form.text.is_empty() {
                        out.push_str(&form.text);
                        *has_token = true;
                    } else if !form.caption.is_empty() {
                        out.push_str(&form.caption);
                        *has_token = true;
                    }
                }
                _ => {}
            }

            for child in &node.children {
                collect_line_text(child, out, has_token);
            }
        }

        fn markdown_escape_cell(s: &str) -> String {
            s.replace('|', "\\|")
                .replace('\r', "")
                .replace('\n', " ")
                .trim()
                .to_string()
        }

        fn table_cell_text(cell: &crate::model::table::Cell) -> String {
            let mut parts: Vec<String> = Vec::new();
            for para in &cell.paragraphs {
                let txt = para.text.trim();
                if !txt.is_empty() {
                    parts.push(markdown_escape_cell(txt));
                }
            }
            parts.join(" <br> ")
        }

        fn table_to_markdown(table: &crate::model::table::Table) -> String {
            let rows = table.row_count as usize;
            let cols = table.col_count as usize;
            if rows == 0 || cols == 0 {
                return String::new();
            }

            let mut grid = vec![vec![String::new(); cols]; rows];

            for cell in &table.cells {
                let r = cell.row as usize;
                let c = cell.col as usize;
                if r >= rows || c >= cols {
                    continue;
                }
                grid[r][c] = table_cell_text(cell);
            }

            let make_row = |cells: &[String]| -> String { format!("| {} |", cells.join(" | ")) };

            let header = make_row(&grid[0]);
            let separator = format!(
                "| {} |",
                std::iter::repeat("---")
                    .take(cols)
                    .collect::<Vec<_>>()
                    .join(" | ")
            );

            let mut lines = vec![header, separator];
            for row in grid.iter().skip(1) {
                lines.push(make_row(row));
            }
            lines.join("\n")
        }

        fn lookup_table<'a>(
            doc: &'a crate::model::document::Document,
            sec_idx: usize,
            para_idx: usize,
            ctrl_idx: usize,
        ) -> Option<&'a crate::model::table::Table> {
            let para = doc.sections.get(sec_idx)?.paragraphs.get(para_idx)?;
            match para.controls.get(ctrl_idx)? {
                crate::model::control::Control::Table(table) => Some(table.as_ref()),
                _ => None,
            }
        }

        fn collect_markdown_items(
            node: &RenderNode,
            doc: &crate::model::document::Document,
            items: &mut Vec<MarkdownItem>,
        ) {
            fn collect_images_only(node: &RenderNode, items: &mut Vec<MarkdownItem>) {
                if let RenderNodeType::Image(image_node) = &node.node_type {
                    items.push(MarkdownItem::Image {
                        sec_idx: image_node.section_index,
                        para_idx: image_node.para_index,
                        control_idx: image_node.control_index,
                        bin_data_id: image_node.bin_data_id,
                    });
                    return;
                }
                for child in &node.children {
                    collect_images_only(child, items);
                }
            }

            match &node.node_type {
                RenderNodeType::Table(table_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (
                        table_node.section_index,
                        table_node.para_index,
                        table_node.control_index,
                    ) {
                        if let Some(table) = lookup_table(doc, si, pi, ci) {
                            let md = table_to_markdown(table);
                            if !md.is_empty() {
                                items.push(MarkdownItem::Table(md));
                            }
                            // 표 텍스트는 table_to_markdown으로 처리하고,
                            // 표 내부 Image 노드만 별도로 수집한다.
                            for child in &node.children {
                                collect_images_only(child, items);
                            }
                            return;
                        }
                    }
                }
                RenderNodeType::Image(image_node) => {
                    items.push(MarkdownItem::Image {
                        sec_idx: image_node.section_index,
                        para_idx: image_node.para_index,
                        control_idx: image_node.control_index,
                        bin_data_id: image_node.bin_data_id,
                    });
                    return;
                }
                RenderNodeType::TextLine(_) => {
                    let mut line = String::new();
                    let mut has_token = false;
                    for child in &node.children {
                        collect_line_text(child, &mut line, &mut has_token);
                    }
                    if has_token {
                        items.push(MarkdownItem::Line(line));
                    }
                    return;
                }
                _ => {}
            }

            for child in &node.children {
                collect_markdown_items(child, doc, items);
            }
        }

        let tree = self.build_page_tree(page_num)?;
        let mut items: Vec<MarkdownItem> = Vec::new();
        collect_markdown_items(&tree.root, &self.document, &mut items);

        let mut out = String::new();
        let mut images: Vec<(Option<usize>, Option<usize>, Option<usize>, u16)> = Vec::new();
        for item in items {
            match item {
                MarkdownItem::Line(line) => {
                    out.push_str(&line);
                    out.push('\n');
                }
                MarkdownItem::Table(table_md) => {
                    if !out.is_empty() && !out.ends_with("\n\n") {
                        out.push('\n');
                    }
                    out.push_str(&table_md);
                    out.push_str("\n\n");
                }
                MarkdownItem::Image {
                    sec_idx,
                    para_idx,
                    control_idx,
                    bin_data_id,
                } => {
                    if !out.is_empty() && !out.ends_with("\n\n") {
                        out.push('\n');
                    }
                    let image_idx = images.len() + 1;
                    out.push_str(&format!("[[RHWP_IMAGE:{}]]\n\n", image_idx));
                    images.push((sec_idx, para_idx, control_idx, bin_data_id));
                }
            }
        }

        Ok((out.trim_end().to_string(), images))
    }

    /// 페이지를 Markdown으로 추출한다 (이미지 토큰 포함).
    pub fn extract_page_markdown_native(&self, page_num: u32) -> Result<String, HwpError> {
        self.extract_page_markdown_with_images_native(page_num)
            .map(|(md, _)| md)
    }

    // =====================================================================
    // 클립보드 API (내부)
    // =====================================================================
}

/// 단이 HWP 원본 layout 에서 사용했을 높이를 LINE_SEG vpos 기준으로 추정 (px).
///
/// 알고리즘:
/// 1. 단의 항목들을 순회하며 첫 vpos-reset (line>0, vertical_pos==0) 발견 지점을 찾는다.
/// 2. reset 발견: reset 직전 줄의 vpos + line_height (HWP 가 단을 끊은 위치)
/// 3. reset 미발견: 단의 마지막 처리 줄 (FullParagraph: 마지막, PartialParagraph: end_line-1)
/// 4. Table/Shape 만 있는 항목은 추정 불가 (None)
fn compute_hwp_used_height(
    cc: &crate::renderer::pagination::ColumnContent,
    paragraphs: &[Paragraph],
    dpi: f64,
) -> Option<f64> {
    use crate::renderer::hwpunit_to_px;
    use crate::renderer::pagination::PageItem;

    // [Task #1612] vpos 는 다페이지에서 누적값(예: page2 vpos≈66568~127587 HU)이므로, per-page
    // `used_height` 와 비교하려면 이 페이지 시작 오프셋을 빼야 한다. 차감하지 않으면 페이지마다
    // diff 가 ~수백px 누적 증가해 "대형 표 과소측정" 으로 오판된다(#1612). 페이지 첫 문단 항목의
    // top vpos 를 기준선으로 삼는다(문단 항목이 없으면 0).
    let base_top: i32 =
        cc.items
            .iter()
            .find_map(|item| {
                let (para_index, line) = match item {
                    PageItem::FullParagraph { para_index } => (*para_index, 0usize),
                    PageItem::PartialParagraph {
                        para_index,
                        start_line,
                        ..
                    } => (*para_index, *start_line),
                    PageItem::Table { para_index, .. }
                    | PageItem::PartialTable { para_index, .. } => (*para_index, 0usize),
                    _ => return None,
                };
                paragraphs
                    .get(para_index)?
                    .line_segs
                    .get(line)
                    .map(|s| s.vertical_pos)
            })
            .unwrap_or(0);

    // 1) 단 항목 내 첫 vpos-reset 검색
    for item in &cc.items {
        let (para_idx, range_start, range_end) = match item {
            PageItem::FullParagraph { para_index } => {
                let p = match paragraphs.get(*para_index) {
                    Some(p) => p,
                    None => continue,
                };
                (*para_index, 0usize, p.line_segs.len())
            }
            PageItem::PartialParagraph {
                para_index,
                start_line,
                end_line,
            } => {
                let p = match paragraphs.get(*para_index) {
                    Some(p) => p,
                    None => continue,
                };
                (*para_index, *start_line, (*end_line).min(p.line_segs.len()))
            }
            _ => continue,
        };
        let p = match paragraphs.get(para_idx) {
            Some(p) => p,
            None => continue,
        };
        // line>0 인 줄 중 vertical_pos==0 첫 줄 찾기
        for i in range_start.max(1)..range_end {
            if let Some(seg) = p.line_segs.get(i) {
                if seg.vertical_pos == 0 {
                    if let Some(prev) = p.line_segs.get(i.saturating_sub(1)) {
                        let bottom_hwpu = prev.vertical_pos + prev.line_height + prev.line_spacing;
                        return Some(hwpunit_to_px((bottom_hwpu - base_top).max(0), dpi));
                    }
                }
            }
        }
    }

    // 2) reset 미발견: 단 마지막 항목의 마지막 줄
    let last_item = cc.items.last()?;
    let (para_idx, line_idx) = match last_item {
        PageItem::FullParagraph { para_index } => {
            let p = paragraphs.get(*para_index)?;
            if p.line_segs.is_empty() {
                return None;
            }
            (*para_index, p.line_segs.len() - 1)
        }
        PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        } => {
            let p = paragraphs.get(*para_index)?;
            if p.line_segs.is_empty() {
                return None;
            }
            (
                *para_index,
                end_line.saturating_sub(1).min(p.line_segs.len() - 1),
            )
        }
        _ => return None,
    };
    let p = paragraphs.get(para_idx)?;
    let seg = p.line_segs.get(line_idx)?;
    let bottom_hwpu = seg.vertical_pos + seg.line_height + seg.line_spacing;
    Some(hwpunit_to_px((bottom_hwpu - base_top).max(0), dpi))
}

/// LINE_SEG vertical_pos 범위를 문자열로 포맷.
///
/// `start_line..end_line` 가 None 이면 문단 전체 범위.
/// `vertical_pos == 0` 인 줄(문단 첫 줄 제외)을 vpos-reset 으로 마킹.
fn format_vpos_range(
    para: Option<&Paragraph>,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> String {
    let Some(p) = para else { return String::new() };
    if p.line_segs.is_empty() {
        return String::new();
    };
    let total = p.line_segs.len();
    let s = start_line.unwrap_or(0).min(total.saturating_sub(1));
    let end_exclusive = end_line.unwrap_or(total).min(total);
    if s >= end_exclusive {
        return String::new();
    };
    let e = end_exclusive - 1;

    let first = p.line_segs[s].vertical_pos;
    let last = p.line_segs[e].vertical_pos;
    let mut resets: Vec<usize> = Vec::new();
    let mut rewinds: Vec<usize> = Vec::new();
    for i in s..=e {
        // 문단 첫 줄(line 0)의 vpos는 자연 시작점이므로 제외
        if i > 0 && p.line_segs[i].vertical_pos == 0 {
            resets.push(i);
        }
        if i > s && p.line_segs[i].vertical_pos < p.line_segs[i - 1].vertical_pos {
            rewinds.push(i);
        }
    }
    let mut s_out = if s == e {
        format!("vpos={}", first)
    } else {
        format!("vpos={}..{}", first, last)
    };
    for r in resets {
        s_out.push_str(&format!(" [vpos-reset@line{}]", r));
    }
    for r in rewinds {
        s_out.push_str(&format!(" [vpos-rewind@line{}]", r));
    }
    s_out
}

/// dump-pages에서 line_seg 계약을 빠르게 대조하기 위한 한 줄 요약.
fn format_line_seg_brief(para: Option<&Paragraph>) -> String {
    let Some(p) = para else {
        return String::new();
    };
    if p.line_segs.is_empty() {
        return "ls=0".to_string();
    }
    let first = &p.line_segs[0];
    let last = p.line_segs.last().unwrap_or(first);
    if p.line_segs.len() == 1 {
        format!(
            "ls=1[ts={} lh={} th={} bl={} gap={} cs={} sw={}]",
            first.text_start,
            first.line_height,
            first.text_height,
            first.baseline_distance,
            first.line_spacing,
            first.column_start,
            first.segment_width
        )
    } else {
        format!(
            "ls={}[first ts={} lh={} th={} bl={} gap={} cs={} sw={}; last ts={} lh={} th={} bl={} gap={} cs={} sw={}]",
            p.line_segs.len(),
            first.text_start,
            first.line_height,
            first.text_height,
            first.baseline_distance,
            first.line_spacing,
            first.column_start,
            first.segment_width,
            last.text_start,
            last.line_height,
            last.text_height,
            last.baseline_distance,
            last.line_spacing,
            last.column_start,
            last.segment_width
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;
    use crate::renderer::render_tree::RenderNodeType;

    /// [Task #1612] `compute_hwp_used_height` 는 per-page 높이를 내야 한다. vpos 는
    /// 다페이지에서 누적값이므로 페이지 시작 오프셋을 차감하지 않으면 page N>1 에서
    /// 누적값(수백~수천px)을 반환해 dump-pages diff 가 "대형 표 과소측정" 으로 오판된다.
    #[test]
    fn task1612_hwp_used_height_is_per_page_not_cumulative() {
        use crate::model::paragraph::{LineSeg, Paragraph};
        use crate::renderer::pagination::{ColumnContent, PageItem};

        let seg = |vpos: i32| LineSeg {
            vertical_pos: vpos,
            line_height: 1700,
            line_spacing: 0,
            ..Default::default()
        };
        // para 0 = page-1 콘텐츠(vpos 0..), para 1 = page-2 콘텐츠(vpos 누적값 66568..).
        let p0 = Paragraph {
            line_segs: vec![seg(0), seg(1700)],
            ..Default::default()
        };
        let p1 = Paragraph {
            line_segs: vec![seg(66568), seg(68268)],
            ..Default::default()
        };
        let paragraphs = vec![p0, p1];

        // "page 2" 단: para 1 만 포함.
        let cc = ColumnContent {
            column_index: 0,
            start_height: 0.0,
            endnote_flow: false,
            items: vec![PageItem::FullParagraph { para_index: 1 }],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: vec![],
            used_height: 0.0,
            wrap_anchors: std::collections::HashMap::new(),
        };

        let h = compute_hwp_used_height(&cc, &paragraphs, 96.0).expect("값이 있어야 함");
        // per-page: (68268+1700) − 66568 = 3400 HU ≈ 45.3px. 누적이면 ≈ 933px.
        assert!(
            h < 100.0,
            "per-page 높이(누적 vpos 차감)여야 하는데 {h:.1}px (누적값 미차감 의심)"
        );
    }

    #[test]
    fn build_page_render_tree_exposes_public_page_tree() {
        let mut core = DocumentCore::new_empty();
        core.paginate();

        let tree = core
            .build_page_render_tree(0)
            .expect("empty document should expose page render tree");

        match tree.root.node_type {
            RenderNodeType::Page(page) => {
                assert_eq!(page.page_index, 0);
            }
            other => panic!("root should be Page node, got {:?}", other),
        }
    }

    /// [Task #1280 v2] 레이아웃 쿼리가 컨트롤별 plane/zOrder/stableIndex 를 노출하고,
    /// 렌더 정렬키(`paper_node_sort_key`)와 동일하게 한컴 권위 샘플
    /// (글상자=InFrontOfText plane 3, 이미지=Square plane 2)을 산출하는지 검증.
    /// 이미지가 글상자 뒤(plane 2 < 3)로 가는 한컴 정합의 근거.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn task1280_v2_control_layout_exposes_plane_z_order_stable_index() {
        // "type":"<name>" 다음 처음 나오는 "<key>": 정수값 추출(plane 은 컨트롤 객체 끝에 부착).
        fn field_after_type(json: &str, type_name: &str, key: &str) -> Option<i64> {
            let tstart = json.find(&format!("\"type\":\"{}\"", type_name))?;
            let needle = format!("\"{}\":", key);
            let kstart = json[tstart..].find(&needle)? + tstart + needle.len();
            let kend = json[kstart..].find(|c| c == ',' || c == '}')? + kstart;
            json[kstart..kend].trim().parse().ok()
        }

        let data = std::fs::read("samples/textbox-under-image.hwp")
            .expect("read samples/textbox-under-image.hwp");
        let mut core = DocumentCore::from_bytes(&data).expect("load textbox-under-image.hwp");
        core.paginate();
        let json = core
            .get_page_control_layout_native(0)
            .expect("control layout for page 0");

        // 신규 필드가 노출됨
        assert!(json.contains("\"plane\":"), "plane 필드 누락: {json}");
        assert!(json.contains("\"zOrder\":"), "zOrder 필드 누락: {json}");
        assert!(
            json.contains("\"stableIndex\":"),
            "stableIndex 필드 누락: {json}"
        );

        // 권위 샘플: 글상자(shape)=InFrontOfText plane 3, 이미지=Square plane 2.
        let shape_plane = field_after_type(&json, "shape", "plane")
            .unwrap_or_else(|| panic!("shape plane 추출 실패: {json}"));
        let image_plane = field_after_type(&json, "image", "plane")
            .unwrap_or_else(|| panic!("image plane 추출 실패: {json}"));
        assert_eq!(shape_plane, 3, "글상자는 InFrontOfText(plane 3): {json}");
        assert_eq!(image_plane, 2, "이미지는 Square(plane 2): {json}");
        // 핵심 불변식: 이미지가 글상자 뒤(plane 작음) — 한컴 동일.
        assert!(
            image_plane < shape_plane,
            "이미지가 글상자 뒤여야 함(plane {image_plane} < {shape_plane}): {json}"
        );
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn render_page_png_native_uses_document_core_skia_layer_path() {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::page::PageDef;
        use crate::model::paragraph::Paragraph;

        let mut document = Document::default();
        document.sections.push(Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,
                    height: 84188,
                    margin_left: 8504,
                    margin_right: 8504,
                    margin_top: 5668,
                    margin_bottom: 4252,
                    margin_header: 4252,
                    margin_footer: 4252,
                    ..Default::default()
                },
                ..Default::default()
            },
            paragraphs: vec![Paragraph::default()],
            raw_stream: None,
        });

        let mut core = DocumentCore::new_empty();
        core.set_document(document);

        let png = core
            .render_page_png_native(0)
            .expect("empty document should render through native Skia");

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        let decoded = image::load_from_memory(&png).expect("decode native Skia PNG");
        assert!(decoded.width() > 0);
        assert!(decoded.height() > 0);
    }

    #[test]
    fn get_bin_data_returns_zero_based_content_slice() {
        let mut core = DocumentCore::new_empty();
        core.document.bin_data_content.push(BinDataContent {
            id: 1,
            data: vec![0x01, 0x02, 0x03],
            extension: "png".to_string(),
        });
        core.document.bin_data_content.push(BinDataContent {
            id: 2,
            data: vec![0xAA, 0xBB],
            extension: "jpg".to_string(),
        });

        assert_eq!(core.get_bin_data(0), Some(&[0x01, 0x02, 0x03][..]));
        assert_eq!(core.get_bin_data(1), Some(&[0xAA, 0xBB][..]));
        assert_eq!(core.get_bin_data(2), None);
    }

    #[test]
    fn master_page_selection_uses_final_carried_page_number_parity() {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::header_footer::{HeaderFooterApply, MasterPage};
        use crate::model::page::PageDef;
        use crate::model::paragraph::Paragraph;

        fn a4_section(section_def: SectionDef) -> Section {
            Section {
                section_def,
                paragraphs: vec![Paragraph::default()],
                raw_stream: None,
            }
        }

        let page_def = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5668,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        };

        let mut document = Document::default();
        document.sections.push(a4_section(SectionDef {
            page_def: page_def.clone(),
            ..Default::default()
        }));
        document.sections.push(a4_section(SectionDef {
            page_def,
            master_pages: vec![
                MasterPage {
                    apply_to: HeaderFooterApply::Odd,
                    ..Default::default()
                },
                MasterPage {
                    apply_to: HeaderFooterApply::Even,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));

        let mut core = DocumentCore::new_empty();
        core.set_document(document);
        core.paginate();

        let page = core
            .pagination
            .get(1)
            .and_then(|result| result.pages.first())
            .expect("section 1 first page");
        assert_eq!(
            page.page_number, 2,
            "section 1 first page should carry section 0 page number"
        );
        let active = page
            .active_master_page
            .as_ref()
            .expect("section 1 page should have an active master page");
        assert_eq!(
            active.master_page_index, 1,
            "final page_number=2 must select the Even master page, not the section-local Odd page"
        );
    }

    #[test]
    fn missing_even_master_page_inherits_previous_even_master() {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::header_footer::{HeaderFooterApply, MasterPage};
        use crate::model::page::PageDef;
        use crate::model::paragraph::Paragraph;

        fn a4_section(section_def: SectionDef) -> Section {
            Section {
                section_def,
                paragraphs: vec![Paragraph::default()],
                raw_stream: None,
            }
        }

        let page_def = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5668,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        };

        let mut document = Document::default();
        document.sections.push(a4_section(SectionDef {
            page_def: page_def.clone(),
            master_pages: vec![MasterPage {
                apply_to: HeaderFooterApply::Even,
                ..Default::default()
            }],
            ..Default::default()
        }));
        document.sections.push(a4_section(SectionDef {
            page_def,
            master_pages: vec![MasterPage {
                apply_to: HeaderFooterApply::Odd,
                ..Default::default()
            }],
            ..Default::default()
        }));

        let mut core = DocumentCore::new_empty();
        core.set_document(document);
        core.paginate();

        let page = core
            .pagination
            .get(1)
            .and_then(|result| result.pages.first())
            .expect("section 1 first page");
        assert_eq!(page.page_number, 2);
        let active = page
            .active_master_page
            .as_ref()
            .expect("section 1 even page should inherit previous even master page");
        assert_eq!(active.section_index, 0);
        assert_eq!(active.master_page_index, 0);
    }

    #[test]
    fn single_base_master_page_applies_when_matching_parity_is_absent() {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::header_footer::{HeaderFooterApply, MasterPage};
        use crate::model::page::PageDef;
        use crate::model::paragraph::Paragraph;

        fn a4_section(section_def: SectionDef) -> Section {
            Section {
                section_def,
                paragraphs: vec![Paragraph::default()],
                raw_stream: None,
            }
        }

        let page_def = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5668,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        };

        let mut document = Document::default();
        document.sections.push(a4_section(SectionDef {
            page_def: page_def.clone(),
            ..Default::default()
        }));
        document.sections.push(a4_section(SectionDef {
            page_def,
            master_pages: vec![MasterPage {
                apply_to: HeaderFooterApply::Odd,
                ..Default::default()
            }],
            ..Default::default()
        }));

        let mut core = DocumentCore::new_empty();
        core.set_document(document);
        core.paginate();

        let page = core
            .pagination
            .get(1)
            .and_then(|result| result.pages.first())
            .expect("section 1 first page");
        assert_eq!(page.page_number, 2);
        let active = page
            .active_master_page
            .as_ref()
            .expect("single base master should apply to carried even page");
        assert_eq!(active.master_page_index, 0);
    }

    #[test]
    fn page_border_fill_api_updates_basis_spacing_and_border() {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::paragraph::Paragraph;

        let mut core = DocumentCore::new_empty();
        let mut document = Document::default();
        document.sections.push(Section {
            section_def: SectionDef::default(),
            paragraphs: vec![Paragraph::default()],
            raw_stream: None,
        });
        core.set_document(document);
        core.paginate();

        let result = core
            .set_page_border_fill_native(
                0,
                r##"{"basis":"page","spacingLeft":567,"spacingRight":567,"spacingTop":567,"spacingBottom":567,
                "borderLeft":{"type":1,"width":1,"color":"#000000"},
                "borderRight":{"type":1,"width":1,"color":"#000000"},
                "borderTop":{"type":1,"width":1,"color":"#000000"},
                "borderBottom":{"type":1,"width":1,"color":"#000000"},
                "fillType":"solid","fillColor":"#ffffff","patternColor":"#000000","patternType":0,
                "fillArea":"paper","applyPage":"all"}"##,
            )
            .expect("page border fill should update");
        assert!(result.contains("\"ok\":true"));

        let json = core
            .get_page_border_fill_native(0)
            .expect("page border fill should be queryable");
        assert!(json.contains("\"basis\":\"page\""));
        assert!(json.contains("\"spacingTop\":567"));
        assert!(json.contains("\"borderFillId\":"));
        assert!(json.contains("\"borderTop\":{\"type\":1"));

        let info = core
            .get_page_info_native(0)
            .expect("page info should reflect updated page border");
        assert!(info.contains("\"pageBorderTop\":"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn page_border_fill_sample_basis_matches_hancom_ui() {
        use crate::model::page::PageBorderBasis;

        fn json_number(json: &str, key: &str) -> f64 {
            let needle = format!("\"{}\":", key);
            let start = json
                .find(&needle)
                .unwrap_or_else(|| panic!("{key} missing in {json}"))
                + needle.len();
            let end = json[start..]
                .find(|c| c == ',' || c == '}')
                .map(|idx| start + idx)
                .unwrap_or(json.len());
            json[start..end].trim().parse().unwrap()
        }

        fn assert_basis(path: &str, expected: &str, expected_render_basis: PageBorderBasis) -> f64 {
            let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
            let core =
                DocumentCore::from_bytes(&data).unwrap_or_else(|e| panic!("load {path}: {e}"));
            let json = core
                .get_page_border_fill_native(0)
                .unwrap_or_else(|e| panic!("query {path}: {e}"));
            assert!(
                json.contains(&format!("\"basis\":\"{}\"", expected)),
                "{path} expected basis={expected}, got {json}"
            );
            assert_eq!(
                core.document.sections[0].section_def.page_border_fill.basis,
                expected_render_basis
            );
            let info = core
                .get_page_info_native(0)
                .unwrap_or_else(|e| panic!("page info {path}: {e}"));
            json_number(&info, "pageBorderTop")
        }

        let paper_hwp_top =
            assert_basis("samples/종이기준.hwp", "paper", PageBorderBasis::PaperBased);
        assert_basis(
            "samples/종이기준.hwpx",
            "paper",
            PageBorderBasis::PaperBased,
        );
        let page_hwp_top = assert_basis("samples/쪽기준.hwp", "page", PageBorderBasis::BodyBased);
        assert_basis("samples/쪽기준.hwpx", "page", PageBorderBasis::BodyBased);
        let sample16_top = assert_basis(
            "samples/hwp3-sample16-hwp5.hwp",
            "page",
            PageBorderBasis::BodyBased,
        );
        assert!(
            page_hwp_top > paper_hwp_top,
            "쪽 기준 border top should be inside paper 기준: paper={paper_hwp_top}, page={page_hwp_top}"
        );
        assert!(
            (sample16_top - 49.2).abs() < 0.2,
            "sample16 page-basis UI should use body top minus spacing and double-line outset: top={sample16_top}"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn inject_png_phys_inserts_after_ihdr() {
        // 최소 PNG: 8-byte signature + IHDR chunk (13 bytes data)
        let mut png = Vec::new();
        png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // signature
                                                                                  // IHDR: length=13
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&[0u8; 13]); // dummy IHDR data
        let ihdr_crc = super::png_crc32(&{
            let mut v = Vec::new();
            v.extend_from_slice(b"IHDR");
            v.extend_from_slice(&[0u8; 13]);
            v
        });
        png.extend_from_slice(&ihdr_crc.to_be_bytes());
        let ihdr_end = png.len(); // 8 + 4 + 4 + 13 + 4 = 33
                                  // IDAT dummy
        png.extend_from_slice(&4u32.to_be_bytes());
        png.extend_from_slice(b"IDAT");
        png.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let idat_crc = super::png_crc32(&{
            let mut v = Vec::new();
            v.extend_from_slice(b"IDAT");
            v.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
            v
        });
        png.extend_from_slice(&idat_crc.to_be_bytes());

        let result = super::inject_png_phys(png.clone(), 300.0);

        // pHYs chunk 삽입 확인: IHDR 직후에 pHYs 가 위치
        assert_eq!(&result[ihdr_end + 4..ihdr_end + 8], b"pHYs");
        // pHYs data length = 9
        let phys_len = u32::from_be_bytes([
            result[ihdr_end],
            result[ihdr_end + 1],
            result[ihdr_end + 2],
            result[ihdr_end + 3],
        ]);
        assert_eq!(phys_len, 9);
        // 300 DPI → 11811 ppm (300 / 0.0254 = 11811.02...)
        let ppm = u32::from_be_bytes([
            result[ihdr_end + 8],
            result[ihdr_end + 9],
            result[ihdr_end + 10],
            result[ihdr_end + 11],
        ]);
        assert_eq!(ppm, 11811);
        // unit = 1 (meter)
        assert_eq!(result[ihdr_end + 16], 1);
        // IDAT 는 pHYs 뒤에 보존
        let phys_chunk_size = 4 + 4 + 9 + 4; // 21
        let idat_pos = ihdr_end + phys_chunk_size;
        assert_eq!(&result[idat_pos + 4..idat_pos + 8], b"IDAT");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn png_crc32_known_value() {
        let crc = super::png_crc32(b"IHDR");
        assert_eq!(crc, 0xA8A1_AE0A);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn vlm_target_from_str_all_variants() {
        use super::VlmTarget;
        assert_eq!(VlmTarget::from_str("claude"), Some(VlmTarget::Claude));
        assert_eq!(VlmTarget::from_str("gpt4v-low"), Some(VlmTarget::Gpt4vLow));
        assert_eq!(VlmTarget::from_str("gpt4v_low"), Some(VlmTarget::Gpt4vLow));
        assert_eq!(
            VlmTarget::from_str("gpt4v-high"),
            Some(VlmTarget::Gpt4vHigh)
        );
        assert_eq!(VlmTarget::from_str("gpt4v"), Some(VlmTarget::Gpt4vHigh));
        assert_eq!(VlmTarget::from_str("gemini"), Some(VlmTarget::Gemini));
        assert_eq!(VlmTarget::from_str("qwen-vl"), Some(VlmTarget::QwenVl));
        assert_eq!(VlmTarget::from_str("qwen_vl"), Some(VlmTarget::QwenVl));
        assert_eq!(VlmTarget::from_str("qwen"), Some(VlmTarget::QwenVl));
        assert_eq!(VlmTarget::from_str("llava"), Some(VlmTarget::Llava));
        assert_eq!(VlmTarget::from_str("CLAUDE"), Some(VlmTarget::Claude));
        assert_eq!(VlmTarget::from_str("unknown"), None);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    #[test]
    fn vlm_target_constraints_are_sane() {
        use super::VlmTarget;
        let targets = [
            VlmTarget::Claude,
            VlmTarget::Gpt4vLow,
            VlmTarget::Gpt4vHigh,
            VlmTarget::Gemini,
            VlmTarget::QwenVl,
            VlmTarget::Llava,
        ];
        for t in &targets {
            let (edge, pixels) = t.constraints();
            assert!(edge > 0, "{:?} edge should be positive", t);
            assert!(pixels > 0, "{:?} pixels should be positive", t);
            assert!(
                (edge as u64) * (edge as u64) >= pixels,
                "{:?} max_pixels should be reachable within edge",
                t
            );
        }
    }
}
