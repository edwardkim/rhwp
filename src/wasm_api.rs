//! WASM ↔ JavaScript 공개 API
//!
//! wasm-bindgen을 통해 JavaScript에서 호출 가능한 API를 정의한다.
//! 주요 API:
//! - `HwpDocument::new(data)` - HWP 파일 로드
//! - `HwpDocument::page_count()` - 페이지 수 조회
//! - `HwpDocument::render_page_svg(page_num)` - SVG로 렌더링
//! - `HwpDocument::render_page_html(page_num)` - HTML로 렌더링

// 하위 호환성: tests.rs에서 super::json_escape 등으로 접근 가능하도록 재내보내기
pub(crate) use crate::document_core::helpers::*;

use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

use crate::document_core::{DocumentCore, DEFAULT_FALLBACK_FONT};
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::page::ColumnDef;
use crate::model::paragraph::Paragraph;
use crate::model::path::{path_from_flat, DocumentPath, PathSegment};
use crate::model::shape::ShapeObject;
use crate::renderer::canvas::CanvasRenderer;
use crate::renderer::composer::{
    compose_paragraph, compose_section, reflow_line_segs, ComposedParagraph,
};
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredSection, MeasuredTable};
use crate::renderer::html::HtmlRenderer;
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::pagination::{PaginationResult, Paginator};
use crate::renderer::render_tree::PageRenderTree;
use crate::renderer::scheduler::{RenderEvent, RenderObserver, RenderScheduler, Viewport};
use crate::renderer::style_resolver::{
    resolve_font_substitution, resolve_styles, ResolvedStyleSet,
};
use crate::renderer::svg::SvgRenderer;
use crate::renderer::DEFAULT_DPI;

impl From<HwpError> for JsValue {
    fn from(err: HwpError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

/// [Task #1161] 클립보드 API 의 cellPath JSON 인자 파싱.
/// 빈 문자열 또는 `"[]"` 면 본문(빈 경로), 그 외에는
/// `[{"controlIndex","cellIndex","cellParaIndex"}, ...]` 를 파싱한다.
fn parse_cell_path_arg(cell_path_json: &str) -> Result<Vec<(usize, usize, usize)>, JsValue> {
    if cell_path_json.is_empty() || cell_path_json == "[]" {
        Ok(Vec::new())
    } else {
        DocumentCore::parse_cell_path(cell_path_json).map_err(JsValue::from)
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const MAX_CANVAS_DIMENSION: f64 = 16_384.0;

#[cfg(any(target_arch = "wasm32", test))]
fn normalize_canvas_scale(
    page_width: f64,
    page_height: f64,
    requested_scale: f64,
) -> Result<f64, &'static str> {
    if !page_width.is_finite()
        || !page_height.is_finite()
        || page_width <= 0.0
        || page_height <= 0.0
    {
        return Err("invalid page dimensions");
    }

    let scale = if requested_scale <= 0.0 || !requested_scale.is_finite() {
        1.0
    } else {
        requested_scale.clamp(0.25, 12.0)
    };

    let scaled_width = page_width * scale;
    let scaled_height = page_height * scale;
    if !scaled_width.is_finite() || !scaled_height.is_finite() {
        return Ok((MAX_CANVAS_DIMENSION / page_width)
            .min(MAX_CANVAS_DIMENSION / page_height)
            .min(scale));
    }

    if scaled_width > MAX_CANVAS_DIMENSION || scaled_height > MAX_CANVAS_DIMENSION {
        Ok((MAX_CANVAS_DIMENSION / page_width)
            .min(MAX_CANVAS_DIMENSION / page_height)
            .min(scale))
    } else {
        Ok(scale)
    }
}

#[cfg(target_arch = "wasm32")]
fn scaled_canvas_extent(page_extent: f64, scale: f64) -> u32 {
    (page_extent * scale).max(1.0).min(MAX_CANVAS_DIMENSION) as u32
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalImageReference {
    key: String,
    bin_data_id: u16,
    original_path: String,
    basename: String,
    extension: String,
    loaded: bool,
}

fn external_path_basename(path: &str) -> &str {
    path.rsplit(|c| c == '/' || c == '\\')
        .find(|part| !part.is_empty())
        .unwrap_or(path)
}

fn external_path_extension(basename: &str) -> String {
    std::path::Path::new(basename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_string()
}

fn parse_external_image_key(key: &str) -> Option<u16> {
    let bin_data_id = key.strip_prefix("binData:")?.parse::<u16>().ok()?;
    (bin_data_id != 0).then_some(bin_data_id)
}

fn collect_external_image_references(document: &Document) -> Vec<ExternalImageReference> {
    let mut references = std::collections::BTreeMap::new();

    for section in &document.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let pic = match ctrl {
                    Control::Picture(pic) => pic,
                    Control::Shape(shape) => match shape.as_ref() {
                        ShapeObject::Picture(pic) => pic,
                        _ => continue,
                    },
                    _ => continue,
                };

                let Some(original_path) = pic.image_attr.external_path.as_ref() else {
                    continue;
                };

                let bin_data_id = pic.image_attr.bin_data_id;
                references.entry(bin_data_id).or_insert_with(|| {
                    let basename = external_path_basename(original_path).to_string();
                    ExternalImageReference {
                        key: format!("binData:{bin_data_id}"),
                        bin_data_id,
                        extension: external_path_extension(&basename),
                        basename,
                        original_path: original_path.clone(),
                        loaded: document.external_image_loaded(bin_data_id),
                    }
                });
            }
        }
    }

    references.into_values().collect()
}

/// WASM에서 사용할 HWP 문서 래퍼
///
/// 도메인 로직은 `DocumentCore`에 구현되어 있으며,
/// `Deref`/`DerefMut`를 통해 투명하게 접근한다.
#[wasm_bindgen]
pub struct HwpDocument {
    core: DocumentCore,
}

impl std::ops::Deref for HwpDocument {
    type Target = DocumentCore;
    fn deref(&self) -> &DocumentCore {
        &self.core
    }
}

impl std::ops::DerefMut for HwpDocument {
    fn deref_mut(&mut self) -> &mut DocumentCore {
        &mut self.core
    }
}

/// 네이티브(비-WASM) 환경용 래퍼 메서드.
///
/// 테스트 및 CLI 환경에서 `HwpDocument::from_bytes()` 등을 직접 호출할 수 있도록 한다.
impl HwpDocument {
    pub fn from_bytes(data: &[u8]) -> Result<HwpDocument, HwpError> {
        DocumentCore::from_bytes(data).map(|core| HwpDocument { core })
    }

    pub fn find_initial_column_def(paragraphs: &[Paragraph]) -> ColumnDef {
        DocumentCore::find_initial_column_def(paragraphs)
    }

    pub fn find_column_def_for_paragraph(paragraphs: &[Paragraph], para_idx: usize) -> ColumnDef {
        DocumentCore::find_column_def_for_paragraph(paragraphs, para_idx)
    }

    fn inject_external_image_by_bin_data_id(
        &mut self,
        bin_data_id: u16,
        data: &[u8],
        display_path: &str,
        fallback_basename: Option<&str>,
    ) -> u32 {
        let Some(reference) = collect_external_image_references(self.document())
            .into_iter()
            .find(|reference| reference.bin_data_id == bin_data_id)
        else {
            return 0;
        };

        if reference.loaded {
            return 0;
        }

        if !self.document_mut().inject_external_image_data(
            bin_data_id,
            data.to_vec(),
            reference.extension.clone(),
        ) {
            return 0;
        }

        let basename = fallback_basename.unwrap_or(&reference.basename);
        let resolved = if display_path.is_empty() {
            format!("/samples/{basename}")
        } else {
            display_path.to_string()
        };
        self.document_mut()
            .update_external_image_display_path(bin_data_id, &resolved);

        1
    }
}

#[wasm_bindgen]
impl HwpDocument {
    /// HWP 파일 바이트를 로드하여 문서 객체를 생성한다.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<HwpDocument, JsValue> {
        DocumentCore::from_bytes(data)
            .map(|core| HwpDocument { core })
            .map_err(|e| e.into())
    }

    /// 빈 문서 생성 (테스트/미리보기용)
    ///
    /// 기본 A4 구역 1개 + 빈 문단 1개를 포함한다. 구역 0개 문서는 모든
    /// 편집/조회 API가 "구역 인덱스 0 범위 초과"로 실패해 사용 불가하므로
    /// 생성 직후 바로 편집 가능한 최소 구조를 보장한다 (#1386).
    #[wasm_bindgen(js_name = createEmpty)]
    pub fn create_empty() -> HwpDocument {
        let mut core = DocumentCore::new_empty();
        let mut section = Section::default();
        // set_document가 styles/composed 재구성 + paginate까지 수행한다.
        section.section_def.page_def = crate::model::page::PageDef::a4_default();
        section.paragraphs.push(Paragraph::new_empty());
        let mut document = Document::default();
        document.sections.push(section);
        core.set_document(document);
        HwpDocument { core }
    }

    /// 내장 템플릿에서 빈 문서를 생성한다.
    ///
    /// saved/blank2010.hwp를 WASM 바이너리에 포함하여 유효한 HWP 문서를 즉시 생성.
    /// DocInfo raw_stream이 온전하므로 FIX-4 워크어라운드와 호환됨.
    #[wasm_bindgen(js_name = createBlankDocument)]
    pub fn create_blank_document(&mut self) -> Result<String, JsValue> {
        self.create_blank_document_native().map_err(|e| e.into())
    }

    /// 문단부호(¶) 표시 여부를 설정한다.
    #[wasm_bindgen(js_name = setShowParagraphMarks)]
    pub fn set_show_paragraph_marks(&mut self, enabled: bool) {
        self.show_paragraph_marks = enabled;
        self.invalidate_page_tree_cache();
    }

    /// 문단부호(¶) 표시 여부를 반환한다.
    #[wasm_bindgen(js_name = getShowParagraphMarks)]
    pub fn get_show_paragraph_marks(&self) -> bool {
        self.show_paragraph_marks
    }

    /// 조판부호 표시 여부를 반환한다.
    #[wasm_bindgen(js_name = getShowControlCodes)]
    pub fn get_show_control_codes(&self) -> bool {
        self.show_control_codes
    }

    /// 조판부호 표시 여부를 설정한다 (개체 마커 + 문단부호 포함).
    #[wasm_bindgen(js_name = setShowControlCodes)]
    pub fn set_show_control_codes(&mut self, enabled: bool) {
        self.show_control_codes = enabled;
        self.invalidate_page_tree_cache();
    }

    /// 투명선 표시 여부를 반환한다.
    #[wasm_bindgen(js_name = getShowTransparentBorders)]
    pub fn get_show_transparent_borders(&self) -> bool {
        self.show_transparent_borders
    }

    /// 투명선 표시 여부를 설정한다.
    #[wasm_bindgen(js_name = setShowTransparentBorders)]
    pub fn set_show_transparent_borders(&mut self, enabled: bool) {
        self.show_transparent_borders = enabled;
        self.invalidate_page_tree_cache();
    }

    #[wasm_bindgen(js_name = setClipEnabled)]
    pub fn set_clip_enabled(&mut self, enabled: bool) {
        self.clip_enabled = enabled;
        self.invalidate_page_tree_cache();
    }

    /// 디버그 오버레이 표시 여부를 설정한다.
    pub fn set_debug_overlay(&mut self, enabled: bool) {
        self.debug_overlay = enabled;
    }

    /// LINE_SEG vpos-reset 강제 분리 적용 여부를 설정한다.
    /// 변경 시 페이지네이션 결과가 달라지므로 모든 섹션을 재페이지네이션한다.
    pub fn set_respect_vpos_reset(&mut self, enabled: bool) {
        if self.respect_vpos_reset != enabled {
            self.respect_vpos_reset = enabled;
            // 모든 섹션 dirty 마킹 후 즉시 재페이지네이션
            for d in self.core.dirty_sections.iter_mut() {
                *d = true;
            }
            self.invalidate_page_tree_cache();
            self.core.paginate();
        }
    }

    /// 총 페이지 수를 반환한다.
    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> u32 {
        self.core.page_count()
    }

    /// 특정 페이지를 SVG 문자열로 렌더링한다.
    #[wasm_bindgen(js_name = renderPageSvg)]
    pub fn render_page_svg(&self, page_num: u32) -> Result<String, JsValue> {
        self.render_page_svg_native(page_num).map_err(|e| e.into())
    }

    /// 특정 페이지를 HTML 문자열로 렌더링한다.
    #[wasm_bindgen(js_name = renderPageHtml)]
    pub fn render_page_html(&self, page_num: u32) -> Result<String, JsValue> {
        self.render_page_html_native(page_num).map_err(|e| e.into())
    }

    /// 특정 페이지를 Canvas 명령 수로 반환한다.
    #[wasm_bindgen(js_name = renderPageCanvas)]
    pub fn render_page_canvas(&self, page_num: u32) -> Result<u32, JsValue> {
        self.render_page_canvas_native(page_num)
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = renderPageCanvasLegacy)]
    pub fn render_page_canvas_legacy(&self, page_num: u32) -> Result<u32, JsValue> {
        self.render_page_canvas_legacy_native(page_num)
            .map_err(|e| e.into())
    }

    /// 특정 페이지를 Canvas 2D에 직접 렌더링한다.
    ///
    /// WASM 환경에서만 사용 가능하다. Canvas 크기는 페이지 크기 × scale로 설정된다.
    /// scale이 0 이하이면 1.0으로 처리한다 (하위호환).
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = renderPageToCanvas)]
    pub fn render_page_to_canvas(
        &self,
        page_num: u32,
        canvas: &HtmlCanvasElement,
        scale: f64,
    ) -> Result<(), JsValue> {
        use crate::renderer::layer_renderer::LayerRenderer;
        use crate::renderer::web_canvas::WebCanvasRenderer;

        let tree = self
            .build_page_layer_tree(page_num)
            .map_err(JsValue::from)?;

        let scale = normalize_canvas_scale(tree.page_width, tree.page_height, scale)
            .map_err(JsValue::from_str)?;

        // 캔버스 크기 = 페이지 크기 × scale
        canvas.set_width(scaled_canvas_extent(tree.page_width, scale));
        canvas.set_height(scaled_canvas_extent(tree.page_height, scale));

        let mut renderer = WebCanvasRenderer::new(canvas)?;
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.set_scale(scale);
        renderer.render_page(&tree).map_err(JsValue::from)?;
        Ok(())
    }

    /// 다층 레이어 필터를 적용한 Canvas 렌더링 (Task #516, Stage 5.2).
    ///
    /// `layer_kind`:
    /// - `"all"` → 모든 PaintOp 렌더 (기본 `renderPageToCanvas` 와 동일)
    /// - `"background"` → page background layer
    /// - `"flow"` → 본문 layer (BehindText / InFrontOfText plane 제외)
    /// - `"behind"` → BehindText overlay layer
    /// - `"front"` → InFrontOfText overlay layer
    ///
    /// 본문 Canvas 와 overlay 컨테이너를 분리하는 다층 layer 아키텍처에서 사용.
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = renderPageToCanvasFiltered)]
    pub fn render_page_to_canvas_filtered(
        &self,
        page_num: u32,
        canvas: &HtmlCanvasElement,
        scale: f64,
        layer_kind: &str,
    ) -> Result<(), JsValue> {
        use crate::model::shape::TextWrap;
        use crate::renderer::layer_renderer::LayerRenderer;
        use crate::renderer::web_canvas::{LayerFilter, WebCanvasRenderer};

        let filter = match layer_kind {
            "all" => LayerFilter::All,
            "background" => LayerFilter::BackgroundOnly,
            "flow" => LayerFilter::FlowOnly,
            "behind" => LayerFilter::WrapOnly(TextWrap::BehindText),
            "front" => LayerFilter::WrapOnly(TextWrap::InFrontOfText),
            _ => {
                return Err(JsValue::from_str(
                    "invalid layer_kind: 'all' | 'background' | 'flow' | 'behind' | 'front'",
                ))
            }
        };

        let tree = self
            .build_page_layer_tree(page_num)
            .map_err(JsValue::from)?;

        let scale = normalize_canvas_scale(tree.page_width, tree.page_height, scale)
            .map_err(JsValue::from_str)?;

        canvas.set_width(scaled_canvas_extent(tree.page_width, scale));
        canvas.set_height(scaled_canvas_extent(tree.page_height, scale));

        let mut renderer = WebCanvasRenderer::new(canvas)?;
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.set_scale(scale);
        renderer.set_layer_filter(filter);
        renderer.render_page(&tree).map_err(JsValue::from)?;
        Ok(())
    }

    /// 특정 페이지를 기존 PageRenderTree 경로로 Canvas 2D에 직접 렌더링한다.
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = renderPageToCanvasLegacy)]
    pub fn render_page_to_canvas_legacy(
        &self,
        page_num: u32,
        canvas: &HtmlCanvasElement,
        scale: f64,
    ) -> Result<(), JsValue> {
        use crate::renderer::web_canvas::WebCanvasRenderer;

        let tree = self
            .build_page_tree_cached(page_num)
            .map_err(|e| JsValue::from(e))?;

        let scale = normalize_canvas_scale(tree.root.bbox.width, tree.root.bbox.height, scale)
            .map_err(JsValue::from_str)?;

        // 캔버스 크기 = 페이지 크기 × scale
        canvas.set_width(scaled_canvas_extent(tree.root.bbox.width, scale));
        canvas.set_height(scaled_canvas_extent(tree.root.bbox.height, scale));

        let mut renderer = WebCanvasRenderer::new(canvas)?;
        renderer.show_paragraph_marks = self.show_paragraph_marks;
        renderer.show_control_codes = self.show_control_codes;
        renderer.set_scale(scale);
        renderer.render_tree(&tree);
        Ok(())
    }

    /// 페이지 렌더 트리를 JSON 문자열로 반환한다.
    #[wasm_bindgen(js_name = getPageRenderTree)]
    pub fn get_page_render_tree(&self, page_num: u32) -> Result<String, JsValue> {
        let tree = self
            .build_page_tree_cached(page_num)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(tree.root.to_json())
    }

    /// 페이지 레이어 트리를 JSON 문자열로 반환한다.
    #[wasm_bindgen(js_name = getPageLayerTree)]
    pub fn get_page_layer_tree(&self, page_num: u32) -> Result<String, JsValue> {
        self.get_page_layer_tree_native(page_num)
            .map_err(|e| e.into())
    }

    /// CanvasKit direct replay 정책 진단을 JSON 문자열로 반환한다.
    ///
    /// `mode` 는 `"default"` 또는 `"compat"` 를 받는다. 빈 문자열은 `"default"` 로 처리한다.
    /// 현재 두 mode 모두 hidden Canvas2D overlay 없이 direct replay required 정책을 따른다.
    /// `compat` 는 API/URL 호환성과 이후 보수적인 direct replay 튜닝을 위해 남겨 둔 선택지다.
    #[wasm_bindgen(js_name = getCanvasKitReplayPlan)]
    pub fn get_canvaskit_replay_plan(&self, page_num: u32, mode: &str) -> Result<String, JsValue> {
        self.get_canvaskit_replay_plan_native(page_num, mode)
            .map_err(|e| e.into())
    }

    /// 페이지 overlay 이미지 정보만 JSON 문자열로 반환한다.
    #[wasm_bindgen(js_name = getPageOverlayImages)]
    pub fn get_page_overlay_images(&self, page_num: u32) -> Result<String, JsValue> {
        self.get_page_overlay_images_native(page_num)
            .map_err(|e| e.into())
    }

    /// 페이지 정보를 JSON 문자열로 반환한다.
    #[wasm_bindgen(js_name = getPageInfo)]
    pub fn get_page_info(&self, page_num: u32) -> Result<String, JsValue> {
        self.get_page_info_native(page_num).map_err(|e| e.into())
    }

    /// 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환한다.
    #[wasm_bindgen(js_name = getPageDef)]
    pub fn get_page_def(&self, section_idx: u32) -> Result<String, JsValue> {
        self.get_page_def_native(section_idx as usize)
            .map_err(|e| e.into())
    }

    /// 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션한다.
    #[wasm_bindgen(js_name = setPageDef)]
    pub fn set_page_def(&mut self, section_idx: u32, json: &str) -> Result<String, JsValue> {
        self.set_page_def_native(section_idx as usize, json)
            .map_err(|e| e.into())
    }

    /// 구역 정의(SectionDef)를 JSON으로 반환한다.
    #[wasm_bindgen(js_name = getSectionDef)]
    pub fn get_section_def(&self, section_idx: u32) -> Result<String, JsValue> {
        self.get_section_def_native(section_idx as usize)
            .map_err(|e| e.into())
    }

    /// 구역 정의(SectionDef)를 변경하고 재페이지네이션한다.
    #[wasm_bindgen(js_name = setSectionDef)]
    pub fn set_section_def(&mut self, section_idx: u32, json: &str) -> Result<String, JsValue> {
        self.set_section_def_native(section_idx as usize, json)
            .map_err(|e| e.into())
    }

    /// 모든 구역의 SectionDef를 일괄 변경하고 재페이지네이션한다.
    #[wasm_bindgen(js_name = setSectionDefAll)]
    pub fn set_section_def_all(&mut self, json: &str) -> Result<String, JsValue> {
        self.set_section_def_all_native(json).map_err(|e| e.into())
    }

    /// 구역의 쪽 테두리/배경 설정을 JSON으로 반환한다.
    #[wasm_bindgen(js_name = getPageBorderFill)]
    pub fn get_page_border_fill(&self, section_idx: u32) -> Result<String, JsValue> {
        self.get_page_border_fill_native(section_idx as usize)
            .map_err(|e| e.into())
    }

    /// 구역의 쪽 테두리/배경 설정을 변경하고 재페이지네이션한다.
    #[wasm_bindgen(js_name = setPageBorderFill)]
    pub fn set_page_border_fill(
        &mut self,
        section_idx: u32,
        json: &str,
    ) -> Result<String, JsValue> {
        self.set_page_border_fill_native(section_idx as usize, json)
            .map_err(|e| e.into())
    }

    /// 현재 구역의 다단 설정을 JSON으로 반환한다.
    #[wasm_bindgen(js_name = getColumnDef)]
    pub fn get_column_def(&self, section_idx: u32) -> Result<String, JsValue> {
        let sec = self
            .core
            .document
            .sections
            .get(section_idx as usize)
            .ok_or_else(|| JsValue::from_str("구역 인덱스 범위 초과"))?;
        let col_def = HwpDocument::find_initial_column_def(&sec.paragraphs);
        let col_type = match col_def.column_type {
            crate::model::page::ColumnType::Normal => 0,
            crate::model::page::ColumnType::Distribute => 1,
            crate::model::page::ColumnType::Parallel => 2,
        };
        Ok(format!(
            "{{\"columnCount\":{},\"columnType\":{},\"sameWidth\":{},\"spacing\":{}}}",
            col_def.column_count, col_type, col_def.same_width, col_def.spacing,
        ))
    }

    /// 문서 정보를 JSON 문자열로 반환한다.
    #[wasm_bindgen(js_name = getDocumentInfo)]
    pub fn get_document_info(&self) -> String {
        self.core.get_document_info()
    }

    /// 특정 페이지의 텍스트 레이아웃 정보를 JSON 문자열로 반환한다.
    ///
    /// 각 TextRun의 위치, 텍스트, 글자별 X 좌표 경계값을 포함한다.
    #[wasm_bindgen(js_name = getPageTextLayout)]
    pub fn get_page_text_layout(&self, page_num: u32) -> Result<String, JsValue> {
        self.get_page_text_layout_native(page_num)
            .map_err(|e| e.into())
    }

    /// 컨트롤(표, 이미지 등) 레이아웃 정보를 반환한다.
    #[wasm_bindgen(js_name = getPageControlLayout)]
    pub fn get_page_control_layout(&self, page_num: u32) -> Result<String, JsValue> {
        self.get_page_control_layout_native(page_num)
            .map_err(|e| e.into())
    }

    /// DPI를 설정한다.
    #[wasm_bindgen(js_name = setDpi)]
    pub fn set_dpi(&mut self, dpi: f64) {
        self.core.set_dpi(dpi);
    }

    /// 파일 이름을 설정한다 (머리말/꼬리말 필드 치환용).
    #[wasm_bindgen(js_name = setFileName)]
    pub fn set_file_name(&mut self, name: &str) {
        if self.core.file_name != name {
            self.core.file_name = name.to_string();
            self.core.invalidate_page_tree_cache();
        }
    }

    /// 현재 DPI를 반환한다.
    #[wasm_bindgen(js_name = getDpi)]
    pub fn get_dpi(&self) -> f64 {
        self.dpi
    }

    /// 대체 폰트 경로를 설정한다.
    #[wasm_bindgen(js_name = setFallbackFont)]
    pub fn set_fallback_font(&mut self, path: &str) {
        self.fallback_font = path.to_string();
    }

    /// 현재 대체 폰트 경로를 반환한다.
    #[wasm_bindgen(js_name = getFallbackFont)]
    pub fn get_fallback_font(&self) -> String {
        self.fallback_font.clone()
    }

    /// 문단에 텍스트를 삽입한다.
    ///
    /// 삽입 후 구역을 재구성하고 재페이지네이션한다.
    /// 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        self.insert_text_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            text,
        )
        .map_err(|e| e.into())
    }

    /// 논리적 오프셋으로 텍스트를 삽입한다.
    ///
    /// logical_offset: 텍스트 문자 + 인라인 컨트롤을 각각 1로 세는 위치.
    /// 예: "abc[표]XYZ" → a(0) b(1) c(2) [표](3) X(4) Y(5) Z(6)
    /// logical_offset=4이면 표 뒤의 X 앞에 삽입.
    /// 반환값: JSON `{"ok":true,"logicalOffset":<new_logical_offset>}`
    #[wasm_bindgen(js_name = insertTextLogical)]
    pub fn insert_text_logical(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        logical_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        let sec = section_idx as usize;
        let pi = para_idx as usize;
        if sec >= self.document.sections.len() || pi >= self.document.sections[sec].paragraphs.len()
        {
            return Err(JsValue::from_str("인덱스 범위 초과"));
        }
        let (text_offset, _) = crate::document_core::helpers::logical_to_text_offset(
            &self.document.sections[sec].paragraphs[pi],
            logical_offset as usize,
        );
        let result = self.insert_text_native(sec, pi, text_offset, text)?;
        // 삽입 후 논리적 오프셋 반환
        let new_text_offset = text_offset + text.chars().count();
        let new_logical = crate::document_core::helpers::text_to_logical_offset(
            &self.document.sections[sec].paragraphs[pi],
            new_text_offset,
        );
        Ok(format!("{{\"ok\":true,\"logicalOffset\":{}}}", new_logical))
    }

    /// 문단의 논리적 길이를 반환한다 (텍스트 문자 + 인라인 컨트롤 수).
    #[wasm_bindgen(js_name = getLogicalLength)]
    pub fn get_logical_length(&self, section_idx: u32, para_idx: u32) -> Result<u32, JsValue> {
        let sec = section_idx as usize;
        let pi = para_idx as usize;
        if sec >= self.document.sections.len() || pi >= self.document.sections[sec].paragraphs.len()
        {
            return Err(JsValue::from_str("인덱스 범위 초과"));
        }
        Ok(crate::document_core::helpers::logical_paragraph_length(
            &self.document.sections[sec].paragraphs[pi],
        ) as u32)
    }

    /// 논리적 오프셋 → 텍스트 오프셋 변환.
    #[wasm_bindgen(js_name = logicalToTextOffset)]
    pub fn logical_to_text_offset(
        &self,
        section_idx: u32,
        para_idx: u32,
        logical_offset: u32,
    ) -> Result<u32, JsValue> {
        let sec = section_idx as usize;
        let pi = para_idx as usize;
        if sec >= self.document.sections.len() || pi >= self.document.sections[sec].paragraphs.len()
        {
            return Err(JsValue::from_str("인덱스 범위 초과"));
        }
        let (text_offset, _) = crate::document_core::helpers::logical_to_text_offset(
            &self.document.sections[sec].paragraphs[pi],
            logical_offset as usize,
        );
        Ok(text_offset as u32)
    }

    /// 텍스트 오프셋 → 논리적 오프셋 변환.
    #[wasm_bindgen(js_name = textToLogicalOffset)]
    pub fn text_to_logical_offset(
        &self,
        section_idx: u32,
        para_idx: u32,
        text_offset: u32,
    ) -> Result<u32, JsValue> {
        let sec = section_idx as usize;
        let pi = para_idx as usize;
        if sec >= self.document.sections.len() || pi >= self.document.sections[sec].paragraphs.len()
        {
            return Err(JsValue::from_str("인덱스 범위 초과"));
        }
        Ok(crate::document_core::helpers::text_to_logical_offset(
            &self.document.sections[sec].paragraphs[pi],
            text_offset as usize,
        ) as u32)
    }

    /// 문단에서 텍스트를 삭제한다.
    ///
    /// 삭제 후 구역을 재구성하고 재페이지네이션한다.
    /// 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
    #[wasm_bindgen(js_name = deleteText)]
    pub fn delete_text(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.delete_text_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 셀 내부 문단에 텍스트를 삽입한다.
    ///
    /// 반환값: JSON `{"ok":true,"charOffset":<new_offset>}`
    #[wasm_bindgen(js_name = insertTextInCell)]
    pub fn insert_text_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        self.insert_text_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            text,
        )
        .map_err(|e| e.into())
    }

    /// `insertTextInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, text: string }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = insertTextInCellEx)]
    pub fn insert_text_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_str, json_u32};
        self.insert_text_in_cell_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            &json_str(options_json, "text").unwrap_or_default(),
        )
        .map_err(|e| e.into())
    }

    /// 표 셀 내부 문단에서 텍스트를 삭제한다.
    ///
    /// 반환값: JSON `{"ok":true,"charOffset":<offset_after_delete>}`
    #[wasm_bindgen(js_name = deleteTextInCell)]
    pub fn delete_text_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.delete_text_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// `deleteTextInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, count }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = deleteTextInCellEx)]
    pub fn delete_text_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.delete_text_in_cell_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_u32(options_json, "count").unwrap_or(0) as usize,
        )
        .map_err(|e| e.into())
    }

    /// 셀 내부 문단을 분할한다 (셀 내 Enter 키).
    ///
    /// 반환값: JSON `{"ok":true,"cellParaIndex":<new_idx>,"charOffset":0}`
    #[wasm_bindgen(js_name = splitParagraphInCell)]
    pub fn split_paragraph_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.split_paragraph_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 셀 내부 문단을 이전 문단에 병합한다 (셀 내 Backspace at start).
    ///
    /// 반환값: JSON `{"ok":true,"cellParaIndex":<prev_idx>,"charOffset":<merge_point>}`
    #[wasm_bindgen(js_name = mergeParagraphInCell)]
    pub fn merge_paragraph_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
    ) -> Result<String, JsValue> {
        self.merge_paragraph_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
        )
        .map_err(|e| e.into())
    }

    // ─── 중첩 표 path 기반 편집 API ──────────────────────────

    #[wasm_bindgen(js_name = insertTextInCellByPath)]
    pub fn insert_text_in_cell_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.insert_text_in_cell_by_path(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
            text,
        )
        .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = deleteTextInCellByPath)]
    pub fn delete_text_in_cell_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.delete_text_in_cell_by_path(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = splitParagraphInCellByPath)]
    pub fn split_paragraph_in_cell_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.split_paragraph_in_cell_by_path(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = mergeParagraphInCellByPath)]
    pub fn merge_paragraph_in_cell_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.merge_paragraph_in_cell_by_path(section_idx as usize, parent_para_idx as usize, &path)
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = getTextInCellByPath)]
    pub fn get_text_in_cell_by_path_api(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.get_text_in_cell_by_path(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    // ─── 머리말/꼬리말 API ──────────────────────────────────

    /// 머리말/꼬리말 조회
    ///
    /// 반환: JSON `{"ok":true,"exists":true/false,...}`
    #[wasm_bindgen(js_name = getHeaderFooter)]
    pub fn get_header_footer(
        &self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
    ) -> Result<String, JsValue> {
        self.get_header_footer_native(section_idx as usize, is_header, apply_to)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 생성 (빈 문단 1개 포함)
    ///
    /// 반환: JSON `{"ok":true,"kind":"header/footer","applyTo":N,...}`
    #[wasm_bindgen(js_name = createHeaderFooter)]
    pub fn create_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
    ) -> Result<String, JsValue> {
        self.create_header_footer_native(section_idx as usize, is_header, apply_to)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내 텍스트 삽입
    ///
    /// 반환: JSON `{"ok":true,"charOffset":<new_offset>}`
    #[wasm_bindgen(js_name = insertTextInHeaderFooter)]
    pub fn insert_text_in_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
        char_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        self.insert_text_in_header_footer_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
            char_offset as usize,
            text,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내 텍스트 삭제
    ///
    /// 반환: JSON `{"ok":true,"charOffset":<offset>}`
    #[wasm_bindgen(js_name = deleteTextInHeaderFooter)]
    pub fn delete_text_in_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.delete_text_in_header_footer_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내 문단 분할 (Enter 키)
    ///
    /// 반환: JSON `{"ok":true,"hfParaIndex":<new_idx>,"charOffset":0}`
    #[wasm_bindgen(js_name = splitParagraphInHeaderFooter)]
    pub fn split_paragraph_in_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.split_paragraph_in_header_footer_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내 문단 병합 (Backspace at start)
    ///
    /// 반환: JSON `{"ok":true,"hfParaIndex":<prev_idx>,"charOffset":<merge_point>}`
    #[wasm_bindgen(js_name = mergeParagraphInHeaderFooter)]
    pub fn merge_paragraph_in_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
    ) -> Result<String, JsValue> {
        self.merge_paragraph_in_header_footer_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 문단 정보 조회
    ///
    /// 반환: JSON `{"ok":true,"paraCount":N,"charCount":N}`
    #[wasm_bindgen(js_name = getHeaderFooterParaInfo)]
    pub fn get_header_footer_para_info(
        &self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_header_footer_para_info_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표에 행을 삽입한다.
    ///
    /// 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
    #[wasm_bindgen(js_name = insertTableRow)]
    pub fn insert_table_row(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        row_idx: u32,
        below: bool,
    ) -> Result<String, JsValue> {
        self.insert_table_row_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            row_idx as u16,
            below,
        )
        .map_err(|e| e.into())
    }

    /// 표에 열을 삽입한다.
    ///
    /// 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
    #[wasm_bindgen(js_name = insertTableColumn)]
    pub fn insert_table_column(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        col_idx: u32,
        right: bool,
    ) -> Result<String, JsValue> {
        self.insert_table_column_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            col_idx as u16,
            right,
        )
        .map_err(|e| e.into())
    }

    /// 표에서 행을 삭제한다.
    ///
    /// 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
    #[wasm_bindgen(js_name = deleteTableRow)]
    pub fn delete_table_row(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        row_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_table_row_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            row_idx as u16,
        )
        .map_err(|e| e.into())
    }

    /// 표에서 열을 삭제한다.
    ///
    /// 반환값: JSON `{"ok":true,"rowCount":<N>,"colCount":<M>}`
    #[wasm_bindgen(js_name = deleteTableColumn)]
    pub fn delete_table_column(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        col_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_table_column_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            col_idx as u16,
        )
        .map_err(|e| e.into())
    }

    /// 표의 셀을 병합한다.
    ///
    /// 반환값: JSON `{"ok":true,"cellCount":<N>}`
    #[wasm_bindgen(js_name = mergeTableCells)]
    pub fn merge_table_cells(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
    ) -> Result<String, JsValue> {
        self.merge_table_cells_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            start_row as u16,
            start_col as u16,
            end_row as u16,
            end_col as u16,
        )
        .map_err(|e| e.into())
    }

    /// `mergeTableCells` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
    /// endRow, endCol }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = mergeTableCellsEx)]
    pub fn merge_table_cells_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.merge_table_cells_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startRow").unwrap_or(0) as u16,
            json_u32(options_json, "startCol").unwrap_or(0) as u16,
            json_u32(options_json, "endRow").unwrap_or(0) as u16,
            json_u32(options_json, "endCol").unwrap_or(0) as u16,
        )
        .map_err(|e| e.into())
    }

    /// 병합된 셀을 나눈다 (split).
    ///
    /// 반환값: JSON `{"ok":true,"cellCount":<N>}`
    #[wasm_bindgen(js_name = splitTableCell)]
    pub fn split_table_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        row: u32,
        col: u32,
    ) -> Result<String, JsValue> {
        self.split_table_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            row as u16,
            col as u16,
        )
        .map_err(|e| e.into())
    }

    /// 셀을 N줄 × M칸으로 분할한다.
    ///
    /// 반환값: JSON `{"ok":true,"cellCount":<N>}`
    #[wasm_bindgen(js_name = splitTableCellInto)]
    pub fn split_table_cell_into(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        row: u32,
        col: u32,
        n_rows: u32,
        m_cols: u32,
        equal_row_height: bool,
        merge_first: bool,
    ) -> Result<String, JsValue> {
        self.split_table_cell_into_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            row as u16,
            col as u16,
            n_rows as u16,
            m_cols as u16,
            equal_row_height,
            merge_first,
        )
        .map_err(|e| e.into())
    }

    /// `splitTableCellInto` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, row, col, nRows, mCols,
    /// equalRowHeight?, mergeFirst? }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = splitTableCellIntoEx)]
    pub fn split_table_cell_into_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_u32};
        self.split_table_cell_into_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "row").unwrap_or(0) as u16,
            json_u32(options_json, "col").unwrap_or(0) as u16,
            json_u32(options_json, "nRows").unwrap_or(1) as u16,
            json_u32(options_json, "mCols").unwrap_or(1) as u16,
            json_bool(options_json, "equalRowHeight").unwrap_or(false),
            json_bool(options_json, "mergeFirst").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    /// 범위 내 셀들을 각각 N줄 × M칸으로 분할한다.
    ///
    /// 반환값: JSON `{"ok":true,"cellCount":<N>}`
    #[wasm_bindgen(js_name = splitTableCellsInRange)]
    pub fn split_table_cells_in_range(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
        n_rows: u32,
        m_cols: u32,
        equal_row_height: bool,
    ) -> Result<String, JsValue> {
        self.split_table_cells_in_range_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            start_row as u16,
            start_col as u16,
            end_row as u16,
            end_col as u16,
            n_rows as u16,
            m_cols as u16,
            equal_row_height,
        )
        .map_err(|e| e.into())
    }

    /// `splitTableCellsInRange` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, startRow, startCol,
    /// endRow, endCol, nRows, mCols, equalRowHeight? }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = splitTableCellsInRangeEx)]
    pub fn split_table_cells_in_range_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_u32};
        self.split_table_cells_in_range_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startRow").unwrap_or(0) as u16,
            json_u32(options_json, "startCol").unwrap_or(0) as u16,
            json_u32(options_json, "endRow").unwrap_or(0) as u16,
            json_u32(options_json, "endCol").unwrap_or(0) as u16,
            json_u32(options_json, "nRows").unwrap_or(1) as u16,
            json_u32(options_json, "mCols").unwrap_or(1) as u16,
            json_bool(options_json, "equalRowHeight").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    /// 선택된 표 셀 범위를 행/열 바꿈 복사용 내부 버퍼에 저장한다.
    ///
    /// 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
    #[wasm_bindgen(js_name = copyTableCellsTransposed)]
    pub fn copy_table_cells_transposed(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
    ) -> Result<String, JsValue> {
        self.copy_table_cells_transposed_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            start_row as u16,
            start_col as u16,
            end_row as u16,
            end_col as u16,
        )
        .map_err(|e| e.into())
    }

    /// 행/열 바꿈 복사 버퍼를 대상 시작 셀부터 붙여넣는다.
    ///
    /// 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
    #[wasm_bindgen(js_name = pasteTableCellsTransposed)]
    pub fn paste_table_cells_transposed(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        start_row: u32,
        start_col: u32,
    ) -> Result<String, JsValue> {
        self.paste_table_cells_transposed_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            start_row as u16,
            start_col as u16,
        )
        .map_err(|e| e.into())
    }

    /// 선택된 전체 표를 제자리에서 전치한다.
    ///
    /// 반환값: JSON `{"ok":true,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
    #[wasm_bindgen(js_name = transposeTableCellsInPlace)]
    pub fn transpose_table_cells_in_place(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.transpose_table_cells_in_place_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 행/열 바꿈 복사 버퍼를 커서 위치에 새 표로 생성해 붙여넣는다.
    ///
    /// 반환값: JSON `{"ok":true,"paraIdx":N,"controlIdx":N,"sourceRows":N,"sourceCols":N,"targetRows":N,"targetCols":N}`
    #[wasm_bindgen(js_name = pasteTableCellsTransposedAsTable)]
    pub fn paste_table_cells_transposed_as_table(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.paste_table_cells_transposed_as_new_table_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 행/열 바꿈 복사 버퍼 보유 여부를 반환한다.
    #[wasm_bindgen(js_name = hasTableTransposeClipboard)]
    pub fn has_table_transpose_clipboard(&self) -> bool {
        self.has_table_transpose_clipboard_native()
    }

    /// 캐럿 위치에서 문단을 분할한다 (Enter 키).
    ///
    /// char_offset 이후의 텍스트가 새 문단으로 이동한다.
    /// 반환값: JSON `{"ok":true,"paraIdx":<new_para_idx>,"charOffset":0}`
    #[wasm_bindgen(js_name = splitParagraph)]
    pub fn split_paragraph(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.split_paragraph_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 강제 쪽 나누기 삽입 (Ctrl+Enter)
    #[wasm_bindgen(js_name = insertPageBreak)]
    pub fn insert_page_break(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.insert_page_break_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 단 나누기 삽입 (Ctrl+Shift+Enter)
    #[wasm_bindgen(js_name = insertColumnBreak)]
    pub fn insert_column_break(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.insert_column_break_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 새 번호 지정 컨트롤 삽입 (쪽 > 새 번호로 시작)
    #[wasm_bindgen(js_name = insertNewNumber)]
    pub fn insert_new_number(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        start_num: u32,
    ) -> Result<String, JsValue> {
        if start_num == 0 || start_num > 65535 {
            return Err(JsValue::from_str("start_num must be 1~65535"));
        }
        self.insert_new_number_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            start_num as u16,
        )
        .map_err(|e| e.into())
    }

    /// 다단 설정 변경
    /// column_type: 0=일반, 1=배분, 2=평행
    /// same_width: 0=다른 너비, 1=같은 너비
    #[wasm_bindgen(js_name = setColumnDef)]
    pub fn set_column_def(
        &mut self,
        section_idx: u32,
        column_count: u32,
        column_type: u32,
        same_width: u32,
        spacing_hu: i32,
    ) -> Result<String, JsValue> {
        self.set_column_def_native(
            section_idx as usize,
            column_count as u16,
            column_type as u8,
            same_width != 0,
            spacing_hu as i16,
        )
        .map_err(|e| e.into())
    }

    /// 현재 문단을 이전 문단에 병합한다 (Backspace at start).
    ///
    /// para_idx의 텍스트가 para_idx-1에 결합되고 para_idx는 삭제된다.
    /// 반환값: JSON `{"ok":true,"paraIdx":<merged_para_idx>,"charOffset":<merge_point>}`
    #[wasm_bindgen(js_name = mergeParagraph)]
    pub fn merge_paragraph(&mut self, section_idx: u32, para_idx: u32) -> Result<String, JsValue> {
        self.merge_paragraph_native(section_idx as usize, para_idx as usize)
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = deleteParagraph)]
    pub fn delete_paragraph(&mut self, section_idx: u32, para_idx: u32) -> Result<String, JsValue> {
        self.delete_paragraph_native(section_idx as usize, para_idx as usize)
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = insertParagraph)]
    pub fn insert_paragraph(&mut self, section_idx: u32, para_idx: u32) -> Result<String, JsValue> {
        self.insert_paragraph_native(section_idx as usize, para_idx as usize)
            .map_err(|e| e.into())
    }

    // ─── Phase 1: 기본 편집 보조 API ───────────────────────────

    /// 구역(Section) 수를 반환한다.
    #[wasm_bindgen(js_name = getSectionCount)]
    pub fn get_section_count(&self) -> u32 {
        self.document.sections.len() as u32
    }

    /// 구역 내 문단 수를 반환한다.
    #[wasm_bindgen(js_name = getParagraphCount)]
    pub fn get_paragraph_count(&self, section_idx: u32) -> Result<u32, JsValue> {
        self.get_paragraph_count_native(section_idx as usize)
            .map(|v| v as u32)
            .map_err(|e| e.into())
    }

    /// 문단의 글자 수(char 개수)를 반환한다.
    #[wasm_bindgen(js_name = getParagraphLength)]
    pub fn get_paragraph_length(&self, section_idx: u32, para_idx: u32) -> Result<u32, JsValue> {
        self.get_paragraph_length_native(section_idx as usize, para_idx as usize)
            .map(|v| v as u32)
            .map_err(|e| e.into())
    }

    /// 문단에 텍스트박스가 있는 Shape 컨트롤이 있으면 해당 control_index를 반환한다.
    /// 없으면 -1을 반환한다.
    #[wasm_bindgen(js_name = getTextBoxControlIndex)]
    pub fn get_textbox_control_index(&self, section_idx: u32, para_idx: u32) -> i32 {
        self.get_textbox_control_index_native(section_idx as usize, para_idx as usize)
    }

    /// 문서 트리에서 다음 편집 가능한 컨트롤/본문을 찾는다.
    /// delta=+1(앞), delta=-1(뒤). ctrl_idx=-1이면 본문 텍스트에서 출발.
    #[wasm_bindgen(js_name = findNextEditableControl)]
    pub fn find_next_editable_control(
        &self,
        section_idx: u32,
        para_idx: u32,
        ctrl_idx: i32,
        delta: i32,
    ) -> String {
        self.find_next_editable_control_native(
            section_idx as usize,
            para_idx as usize,
            ctrl_idx,
            delta,
        )
    }

    /// 커서에서 이전 방향으로 가장 가까운 선택 가능 컨트롤을 찾는다 (F11 키).
    #[wasm_bindgen(js_name = findNearestControlBackward)]
    pub fn find_nearest_control_backward(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> String {
        self.find_nearest_control_backward_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
    }

    /// 현재 위치 이후의 가장 가까운 선택 가능 컨트롤을 찾는다 (Shift+F11).
    #[wasm_bindgen(js_name = findNearestControlForward)]
    pub fn find_nearest_control_forward(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> String {
        self.find_nearest_control_forward_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
    }

    /// 문단 내 컨트롤의 텍스트 위치 배열을 반환한다.
    #[wasm_bindgen(js_name = getControlTextPositions)]
    pub fn get_control_text_positions(&self, section_idx: u32, para_idx: u32) -> String {
        let sections = &self.document.sections;
        if let Some(sec) = sections.get(section_idx as usize) {
            if let Some(para) = sec.paragraphs.get(para_idx as usize) {
                let positions = crate::document_core::find_control_text_positions(para);
                return format!(
                    "[{}]",
                    positions
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
        "[]".to_string()
    }

    /// 문서 트리 DFS 기반 다음/이전 편집 가능 위치를 반환한다.
    /// context_json: NavContextEntry 배열의 JSON (빈 배열 "[]" = body)
    #[wasm_bindgen(js_name = navigateNextEditable)]
    pub fn navigate_next_editable_wasm(
        &self,
        sec: u32,
        para: u32,
        char_offset: u32,
        delta: i32,
        context_json: &str,
    ) -> String {
        let raw_context = DocumentCore::parse_nav_context(context_json);
        // TypeScript에서 ctrl_text_pos=0으로 전달되므로 실제 값으로 보정
        let context = DocumentCore::fix_context_text_positions(
            &self.core.document.sections,
            sec as usize,
            &raw_context,
        );

        // 오버플로우 링크 계산 (캐시됨)
        let overflow_links = self.core.get_overflow_links(sec as usize);

        // 컨텍스트가 있으면 (컨테이너 내부) 렌더링된 마지막 문단 인덱스를 조회
        let max_para = if !context.is_empty() {
            let last = &context[context.len() - 1];
            self.core.last_rendered_para_in_container(
                sec as usize,
                last.parent_para,
                last.ctrl_idx,
                last.cell_idx,
            )
        } else {
            None
        };

        let result = self.core.navigate_next_editable(
            sec as usize,
            para as usize,
            char_offset as usize,
            delta,
            &context,
            max_para,
            &overflow_links,
        );
        DocumentCore::nav_result_to_json(&result)
    }

    /// 문단에서 텍스트 부분 문자열을 반환한다 (Undo용 텍스트 보존).
    #[wasm_bindgen(js_name = getTextRange)]
    pub fn get_text_range(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.get_text_range_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 셀 내 문단 수를 반환한다.
    #[wasm_bindgen(js_name = getCellParagraphCount)]
    pub fn get_cell_paragraph_count(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
    ) -> Result<u32, JsValue> {
        self.get_cell_paragraph_count_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
        )
        .map(|v| v as u32)
        .map_err(|e| e.into())
    }

    /// 표 셀 내 문단의 글자 수를 반환한다.
    #[wasm_bindgen(js_name = getCellParagraphLength)]
    pub fn get_cell_paragraph_length(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
    ) -> Result<u32, JsValue> {
        self.get_cell_paragraph_length_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
        )
        .map(|v| v as u32)
        .map_err(|e| e.into())
    }

    /// 경로 기반: 셀/글상자 내 문단 수를 반환한다 (중첩 표/글상자 지원).
    #[wasm_bindgen(js_name = getCellParagraphCountByPath)]
    pub fn get_cell_paragraph_count_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<u32, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        let count = self
            .resolve_container_para_count_by_path(
                section_idx as usize,
                parent_para_idx as usize,
                &path,
            )
            .map_err(|e| -> JsValue { e.into() })?;
        Ok(count as u32)
    }

    /// 경로 기반: 셀 내 문단의 글자 수를 반환한다 (중첩 표 지원).
    #[wasm_bindgen(js_name = getCellParagraphLengthByPath)]
    pub fn get_cell_paragraph_length_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<u32, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        let para = self
            .resolve_paragraph_by_path(section_idx as usize, parent_para_idx as usize, &path)
            .map_err(|e| -> JsValue { e.into() })?;
        Ok(para.text.chars().count() as u32)
    }

    /// 표 셀의 텍스트 방향을 반환한다 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움).
    #[wasm_bindgen(js_name = getCellTextDirection)]
    pub fn get_cell_text_direction(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
    ) -> Result<u32, JsValue> {
        let para = self
            .document
            .sections
            .get(section_idx as usize)
            .ok_or_else(|| JsValue::from_str("구역 인덱스 범위 초과"))?
            .paragraphs
            .get(parent_para_idx as usize)
            .ok_or_else(|| JsValue::from_str("문단 인덱스 범위 초과"))?;
        match para.controls.get(control_idx as usize) {
            Some(Control::Table(table)) => {
                let cell = table
                    .cells
                    .get(cell_idx as usize)
                    .ok_or_else(|| JsValue::from_str("셀 인덱스 범위 초과"))?;
                Ok(cell.text_direction as u32)
            }
            _ => Ok(0), // 글상자 등은 가로쓰기
        }
    }

    /// 표 셀 내 문단에서 텍스트 부분 문자열을 반환한다.
    #[wasm_bindgen(js_name = getTextInCell)]
    pub fn get_text_in_cell(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.get_text_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// `getTextInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, count }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = getTextInCellEx)]
    pub fn get_text_in_cell_ex(&self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.get_text_in_cell_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_u32(options_json, "count").unwrap_or(0) as usize,
        )
        .map_err(|e| e.into())
    }

    // ─── Phase 1 끝 ─────────────────────────────────────────

    // ─── Phase 2: 커서/히트 테스트 API ──────────────────────────

    /// 커서 위치의 픽셀 좌표를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    #[wasm_bindgen(js_name = getCursorRect)]
    pub fn get_cursor_rect(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 줄 경계 offset을 특정 시각 줄 기준으로 해석한 커서 좌표를 반환한다.
    ///
    /// `at_end=false`이면 lineIndex 줄의 시작, `at_end=true`이면 lineIndex 줄의 끝을 반환한다.
    /// soft-wrap 경계에서는 같은 charOffset이 이전 줄 끝과 다음 줄 시작을 동시에 뜻할 수 있어
    /// Home/End가 이 API로 시각 줄 affinity를 명시한다.
    #[wasm_bindgen(js_name = getCursorRectOnLine)]
    pub fn get_cursor_rect_on_line(
        &self,
        section_idx: u32,
        para_idx: u32,
        line_index: u32,
        at_end: bool,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
    ) -> Result<String, JsValue> {
        let cell_ctx = if parent_para_idx == u32::MAX {
            None
        } else {
            Some((
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
                cell_para_idx as usize,
            ))
        };
        self.get_cursor_rect_on_line_native(
            section_idx as usize,
            para_idx as usize,
            line_index as usize,
            at_end,
            cell_ctx,
        )
        .map_err(|e| e.into())
    }

    /// 페이지 좌표에서 문서 위치를 찾는다.
    ///
    /// 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
    #[wasm_bindgen(js_name = hitTest)]
    pub fn hit_test(&self, page_num: u32, x: f64, y: f64) -> Result<String, JsValue> {
        self.hit_test_native(page_num, x, y).map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내 커서 위치의 픽셀 좌표를 반환한다.
    ///
    /// preferred_page: 선호 페이지 (더블클릭한 페이지). -1이면 첫 번째 발견 페이지 사용.
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    #[wasm_bindgen(js_name = getCursorRectInHeaderFooter)]
    pub fn get_cursor_rect_in_header_footer(
        &self,
        section_idx: u32,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: u32,
        char_offset: u32,
        preferred_page: i32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_in_header_footer_native(
            section_idx as usize,
            is_header,
            apply_to,
            hf_para_idx as usize,
            char_offset as usize,
            preferred_page,
        )
        .map_err(|e| e.into())
    }

    /// 페이지 좌표가 머리말/꼬리말 영역에 해당하는지 판별한다.
    ///
    /// 반환: JSON `{"hit":true/false,"isHeader":bool,"sectionIndex":N,"applyTo":N}`
    #[wasm_bindgen(js_name = hitTestHeaderFooter)]
    pub fn hit_test_header_footer(&self, page_num: u32, x: f64, y: f64) -> Result<String, JsValue> {
        self.hit_test_header_footer_native(page_num, x, y)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 내부 텍스트 히트테스트.
    ///
    /// 편집 모드에서 클릭한 좌표의 문단·문자 위치를 반환.
    /// 반환: JSON `{"hit":true,"paraIndex":N,"charOffset":N,"cursorRect":{...}}`
    #[wasm_bindgen(js_name = hitTestInHeaderFooter)]
    pub fn hit_test_in_header_footer(
        &self,
        page_num: u32,
        is_header: bool,
        x: f64,
        y: f64,
    ) -> Result<String, JsValue> {
        self.hit_test_in_header_footer_native(page_num, is_header, x, y)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 문단의 문단 속성을 조회한다.
    #[wasm_bindgen(js_name = getParaPropertiesInHf)]
    pub fn get_para_properties_in_hf(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Result<String, JsValue> {
        self.get_para_properties_in_hf_native(section_idx, is_header, apply_to, hf_para_idx)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 문단에 문단 서식을 적용한다.
    #[wasm_bindgen(js_name = applyParaFormatInHf)]
    pub fn apply_para_format_in_hf(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_para_format_in_hf_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 문단에 필드 마커를 삽입한다.
    #[wasm_bindgen(js_name = insertFieldInHf)]
    pub fn insert_field_in_hf(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        field_type: u8,
    ) -> Result<String, JsValue> {
        self.insert_field_in_hf_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            char_offset,
            field_type,
        )
        .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 마당(템플릿)을 적용한다.
    #[wasm_bindgen(js_name = applyHfTemplate)]
    pub fn apply_hf_template(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        template_id: u8,
    ) -> Result<String, JsValue> {
        self.apply_hf_template_native(section_idx, is_header, apply_to, template_id)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말을 삭제한다 (컨트롤 자체 제거).
    #[wasm_bindgen(js_name = deleteHeaderFooter)]
    pub fn delete_header_footer(
        &mut self,
        section_idx: u32,
        is_header: bool,
        apply_to: u32,
    ) -> Result<String, JsValue> {
        self.delete_header_footer_native(section_idx as usize, is_header, apply_to as u8)
            .map_err(|e| e.into())
    }

    /// 문서 전체의 머리말/꼬리말 목록을 반환한다.
    #[wasm_bindgen(js_name = getHeaderFooterList)]
    pub fn get_header_footer_list(
        &self,
        current_section_idx: u32,
        current_is_header: bool,
        current_apply_to: u32,
    ) -> Result<String, JsValue> {
        self.get_header_footer_list_native(
            current_section_idx as usize,
            current_is_header,
            current_apply_to as u8,
        )
        .map_err(|e| e.into())
    }

    /// 페이지 단위로 이전/다음 머리말·꼬리말로 이동한다.
    ///
    /// 반환: JSON `{"ok":true,"pageIndex":N,"sectionIdx":N,"isHeader":bool,"applyTo":N}`
    /// 또는 더 이상 이동할 페이지가 없으면 `{"ok":false}`
    #[wasm_bindgen(js_name = navigateHeaderFooterByPage)]
    pub fn navigate_header_footer_by_page(
        &self,
        current_page: u32,
        is_header: bool,
        direction: i32,
    ) -> Result<String, JsValue> {
        self.navigate_header_footer_by_page_native(current_page, is_header, direction)
            .map_err(|e| e.into())
    }

    /// 머리말/꼬리말 감추기를 토글한다 (현재 쪽만).
    ///
    /// 반환: JSON `{"hidden":true/false}` — 토글 후 상태
    #[wasm_bindgen(js_name = toggleHideHeaderFooter)]
    pub fn toggle_hide_header_footer(
        &mut self,
        page_index: u32,
        is_header: bool,
    ) -> Result<String, JsValue> {
        self.toggle_hide_header_footer_native(page_index, is_header)
            .map_err(|e| e.into())
    }

    /// 표 셀 내부 커서 위치의 픽셀 좌표를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    #[wasm_bindgen(js_name = getCursorRectInCell)]
    pub fn get_cursor_rect_in_cell(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    // ─── Phase 3: 커서 이동 API ──────────────────────────────

    /// 문단 내 줄 정보를 반환한다 (커서 수직 이동/Home/End용).
    ///
    /// 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
    #[wasm_bindgen(js_name = getLineInfo)]
    pub fn get_line_info(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_line_info_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 셀 내 문단의 줄 정보를 반환한다.
    ///
    /// 반환: JSON `{"lineIndex":N,"lineCount":N,"charStart":N,"charEnd":N}`
    #[wasm_bindgen(js_name = getLineInfoInCell)]
    pub fn get_line_info_in_cell(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_line_info_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 문서에 저장된 캐럿 위치를 반환한다 (문서 로딩 시 캐럿 자동 배치용).
    ///
    /// 반환: JSON `{"sectionIndex":N,"paragraphIndex":N,"charOffset":N}`
    #[wasm_bindgen(js_name = getCaretPosition)]
    pub fn get_caret_position(&self) -> Result<String, JsValue> {
        self.get_caret_position_native().map_err(|e| e.into())
    }

    /// 표의 행/열/셀 수를 반환한다.
    ///
    /// 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
    #[wasm_bindgen(js_name = getTableDimensions)]
    pub fn get_table_dimensions(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_table_dimensions_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 셀의 행/열/병합 정보를 반환한다.
    ///
    /// 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
    #[wasm_bindgen(js_name = getCellInfo)]
    pub fn get_cell_info(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_cell_info_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 셀 속성을 조회한다.
    ///
    /// 반환: JSON `{width, height, paddingLeft, paddingRight, paddingTop, paddingBottom, applyInnerMargin, verticalAlign, textDirection, isHeader, cellProtect, fieldName, editableInForm, ...borderFill}`
    #[wasm_bindgen(js_name = getCellProperties)]
    pub fn get_cell_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_cell_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 셀 고유 속성을 조회한다.
    ///
    /// cellzone overlay를 합성하지 않고 셀 자체의 borderFill만 반환한다.
    #[wasm_bindgen(js_name = getCellOwnProperties)]
    pub fn get_cell_own_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_cell_own_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 셀 속성을 수정한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = setCellProperties)]
    pub fn set_cell_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        json: &str,
    ) -> Result<String, JsValue> {
        self.set_cell_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            json,
        )
        .map_err(|e| e.into())
    }

    /// 선택 영역을 하나의 셀처럼 취급하는 cellzone 테두리/배경 속성을 적용한다.
    ///
    /// 반환: JSON `{"ok":true,"startRow":...,"borderFillId":...}`
    #[wasm_bindgen(js_name = setCellZoneProperties)]
    pub fn set_cell_zone_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
        json: &str,
    ) -> Result<String, JsValue> {
        self.set_cell_zone_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            start_row as u16,
            start_col as u16,
            end_row as u16,
            end_col as u16,
            json,
        )
        .map_err(|e| e.into())
    }

    /// 여러 셀의 width/height를 한 번에 조절한다 (배치).
    ///
    /// json: `[{"cellIdx":0,"widthDelta":150},{"cellIdx":2,"heightDelta":-100}]`
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = resizeTableCells)]
    pub fn resize_table_cells(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        json: &str,
    ) -> Result<String, JsValue> {
        self.resize_table_cells_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            json,
        )
        .map_err(|e| e.into())
    }

    /// 표의 위치 오프셋(vertical_offset, horizontal_offset)을 이동한다.
    ///
    /// delta_h, delta_v: HWPUNIT 단위 이동량 (양수=오른쪽/아래, 음수=왼쪽/위)
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = moveTableOffset)]
    pub fn move_table_offset(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        delta_h: i32,
        delta_v: i32,
    ) -> Result<String, JsValue> {
        self.move_table_offset_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            delta_h,
            delta_v,
        )
        .map_err(|e| e.into())
    }

    /// 표 속성을 조회한다.
    ///
    /// 반환: JSON `{cellSpacing, paddingLeft, paddingRight, paddingTop, paddingBottom, pageBreak, repeatHeader}`
    #[wasm_bindgen(js_name = getTableProperties)]
    pub fn get_table_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_table_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 속성을 수정한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = setTableProperties)]
    pub fn set_table_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        json: &str,
    ) -> Result<String, JsValue> {
        self.set_table_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            json,
        )
        .map_err(|e| e.into())
    }

    /// 표의 모든 셀 bbox를 반환한다 (F5 셀 선택 모드용).
    ///
    /// 반환: JSON `[{cellIdx, row, col, rowSpan, colSpan, pageIndex, x, y, w, h}, ...]`
    #[wasm_bindgen(js_name = getTableCellBboxes)]
    pub fn get_table_cell_bboxes(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        page_hint: Option<u32>,
    ) -> Result<String, JsValue> {
        self.get_table_cell_bboxes_from_page(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            page_hint.unwrap_or(0) as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 전체의 바운딩박스를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
    #[wasm_bindgen(js_name = getTableBBox)]
    pub fn get_table_bbox(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_table_bbox_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #919] 글상자/도형 컨트롤의 페이지 좌표 바운딩박스를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
    /// studio 의 `isShapeBorderClick` 헬퍼에서 외곽 경계선 클릭 판별에 사용.
    #[wasm_bindgen(js_name = getShapeBBox)]
    pub fn get_shape_bbox(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_shape_bbox_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 컨트롤을 문단에서 삭제한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = deleteTableControl)]
    pub fn delete_table_control(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_table_control_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 커서 위치에 새 표를 삽입한다.
    ///
    /// 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
    #[wasm_bindgen(js_name = createTable)]
    pub fn create_table(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        row_count: u32,
        col_count: u32,
    ) -> Result<String, JsValue> {
        self.create_table_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            row_count as u16,
            col_count as u16,
        )
        .map_err(|e| e.into())
    }

    /// 커서 위치에 표를 삽입한다 (확장, JSON 옵션).
    ///
    /// options JSON: { sectionIdx, paraIdx, charOffset, rowCount, colCount,
    ///                 treatAsChar?: bool, colWidths?: [u32, ...] }
    #[wasm_bindgen(js_name = createTableEx)]
    pub fn create_table_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_u32};
        let section_idx = json_u32(options_json, "sectionIdx").unwrap_or(0) as usize;
        let para_idx = json_u32(options_json, "paraIdx").unwrap_or(0) as usize;
        let char_offset = json_u32(options_json, "charOffset").unwrap_or(0) as usize;
        let row_count = json_u32(options_json, "rowCount").unwrap_or(2) as u16;
        let col_count = json_u32(options_json, "colCount").unwrap_or(2) as u16;
        let treat_as_char = json_bool(options_json, "treatAsChar").unwrap_or(false);
        fn parse_u32_array(json: &str, key: &str) -> Option<Vec<u32>> {
            if let Some(start) = json.find(&format!("\"{}\"", key)) {
                let rest = &json[start..];
                if let Some(arr_start) = rest.find('[') {
                    if let Some(arr_end) = rest[arr_start..].find(']') {
                        let arr_str = &rest[arr_start + 1..arr_start + arr_end];
                        let nums: Vec<u32> = arr_str
                            .split(',')
                            .filter_map(|s| s.trim().parse::<u32>().ok())
                            .collect();
                        if !nums.is_empty() {
                            Some(nums)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        let col_widths = parse_u32_array(options_json, "colWidths");
        let row_heights = parse_u32_array(options_json, "rowHeights");

        self.create_table_ex_native(
            section_idx,
            para_idx,
            char_offset,
            row_count,
            col_count,
            treat_as_char,
            col_widths.as_deref(),
            row_heights.as_deref(),
        )
        .map_err(|e| e.into())
    }

    /// 커서 위치에 그림을 삽입한다.
    ///
    /// image_data: 이미지 바이너리 데이터 (PNG/JPG/GIF/BMP 등)
    /// width, height: HWPUNIT 단위 크기
    /// extension: 파일 확장자 (jpg, png 등)
    ///
    /// 반환:
    /// - 본문 inline: `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
    /// - 셀 floating (#1151): `{"ok":true,"paraIdx":<table_para>,"controlIdx":<new_sibling_idx>}`
    ///
    /// `cell_path_json` 이 빈 문자열 또는 `"[]"` 면 본문 inline 삽입. 그 외에는
    /// 표 셀 영역에 floating picture (한컴 정합) 로 삽입한다.
    /// 예: `[{"controlIndex":0,"cellIndex":2,"cellParaIndex":0}]`
    /// [Task #1151 v8 결함 C] `paper_offset_x_hu / paper_offset_y_hu` 는 사용자가 셀 안에
    /// 클릭/드래그한 위치 (paper-relative HU). studio 의 finishImagePlacement 가 drag 좌표를
    /// 변환하여 전달. JS 측에서 `undefined` 전달 시 (또는 음수) wasm 이 셀 좌상단을 default 사용
    /// — 기존 동작 호환.
    #[wasm_bindgen(js_name = insertPicture)]
    #[allow(clippy::too_many_arguments)]
    pub fn insert_picture(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        cell_path_json: &str,
        image_data: &[u8],
        width: u32,
        height: u32,
        natural_width_px: u32,
        natural_height_px: u32,
        extension: &str,
        description: &str,
        paper_offset_x_hu: Option<i32>,
        paper_offset_y_hu: Option<i32>,
    ) -> Result<String, JsValue> {
        let cell_path: Vec<(usize, usize, usize)> =
            if cell_path_json.is_empty() || cell_path_json == "[]" {
                Vec::new()
            } else {
                DocumentCore::parse_cell_path(cell_path_json).map_err(JsValue::from)?
            };
        self.insert_picture_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            &cell_path,
            image_data,
            width,
            height,
            natural_width_px,
            natural_height_px,
            extension,
            description,
            paper_offset_x_hu,
            paper_offset_y_hu,
        )
        .map_err(|e| e.into())
    }

    /// 커서 위치에 그림을 삽입한다 (확장, options object — #1413).
    ///
    /// positional `insertPicture` 와 동일 동작의 얇은 어댑터. 이미지 바이너리는 별도
    /// `image_data` 인자(Uint8Array)로 받고, 나머지는 JSON options 로 받는다. 필드 추가/
    /// 순서 변경 시 호출부 영향이 작다.
    ///
    /// options JSON 키 (positional 과 동일 의미, camelCase):
    /// `{ sectionIdx, paraIdx, charOffset?, cellPath?: string, width, height,
    ///    naturalWidthPx, naturalHeightPx, extension?, description?,
    ///    paperOffsetXHu?: number|null, paperOffsetYHu?: number|null }`
    /// - `cellPath` 는 cell_path_json 문자열(빈 문자열/`"[]"` 이면 본문 inline).
    /// - 반환값은 `insertPicture` 와 동일.
    #[wasm_bindgen(js_name = insertPictureEx)]
    pub fn insert_picture_ex(
        &mut self,
        options_json: &str,
        image_data: &[u8],
    ) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_i32, json_str, json_u32};
        let section_idx = json_u32(options_json, "sectionIdx").unwrap_or(0);
        let para_idx = json_u32(options_json, "paraIdx").unwrap_or(0);
        let char_offset = json_u32(options_json, "charOffset").unwrap_or(0);
        let cell_path_json = json_str(options_json, "cellPath").unwrap_or_default();
        let width = json_u32(options_json, "width").unwrap_or(0);
        let height = json_u32(options_json, "height").unwrap_or(0);
        let natural_width_px = json_u32(options_json, "naturalWidthPx").unwrap_or(0);
        let natural_height_px = json_u32(options_json, "naturalHeightPx").unwrap_or(0);
        let extension = json_str(options_json, "extension").unwrap_or_default();
        let description = json_str(options_json, "description").unwrap_or_default();
        // paperOffset 은 키 부재 시 None(셀 좌상단 default) — positional 의 Option 동작과 동일.
        let paper_offset_x_hu = json_i32(options_json, "paperOffsetXHu");
        let paper_offset_y_hu = json_i32(options_json, "paperOffsetYHu");

        let cell_path: Vec<(usize, usize, usize)> =
            if cell_path_json.is_empty() || cell_path_json == "[]" {
                Vec::new()
            } else {
                DocumentCore::parse_cell_path(&cell_path_json).map_err(JsValue::from)?
            };
        self.insert_picture_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            &cell_path,
            image_data,
            width,
            height,
            natural_width_px,
            natural_height_px,
            &extension,
            &description,
            paper_offset_x_hu,
            paper_offset_y_hu,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1142] 외부 file path 그림 reference 목록을 구조화된 JSON 배열로 반환한다.
    ///
    /// 반환: JSON 배열 `[{ key, binDataId, originalPath, basename, extension, loaded }, ...]`
    #[wasm_bindgen(js_name = getExternalImageReferences)]
    pub fn get_external_image_references(&self) -> String {
        serde_json::to_string(&collect_external_image_references(self.document()))
            .unwrap_or_else(|_| "[]".to_string())
    }

    /// [Task #741 후속] 외부 file path 그림 영역 영역 영역 영역 basename 목록 영역 반환.
    ///
    /// HWP3 파일 영역 image 영역 영역 절대 경로 영역 저장 영역. WASM 환경 영역 영역 file
    /// system access 부재 영역, JS 영역 영역 영역 영역 fetch 영역 영역 영역 file 영역 load
    /// 영역 후 `injectExternalImage` 영역 영역 영역 inject 영역.
    ///
    /// 반환: JSON 배열 `["oracle.gif", "rdb02.gif", ...]` (중복 제거)
    #[wasm_bindgen(js_name = getExternalImageBasenames)]
    pub fn get_external_image_basenames(&self) -> String {
        use std::collections::BTreeSet;

        let mut names: BTreeSet<String> = BTreeSet::new();
        for reference in collect_external_image_references(self.document()) {
            if !reference.loaded {
                names.insert(reference.basename);
            }
        }
        let arr: Vec<String> = names.into_iter().collect();
        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
    }

    /// [Task #741 후속] 외부 file path 그림 영역 영역 binary data 영역 inject.
    ///
    /// JS 영역 영역 영역 fetch 영역 영역 영역 file 영역 load 영역 후 본 메서드 영역 호출 영역
    /// IR 영역 영역 영역 image binary 영역 영역 → renderer 영역 영역 표시.
    ///
    /// `basename`: 영역 영역 file 영역 영역 (예: "oracle.gif")
    /// `data`: 영역 영역 binary 영역
    /// `display_path`: dialog 영역 영역 영역 영역 표시 영역 영역 path. 빈 문자열 ("") 영역
    ///                 영역 영역 fallback 영역 영역 `/samples/<basename>` 영역 사용. 한컴 viewer
    ///                 정합 영역 영역 OS 영역 절대 경로 영역 영역 (예: "/Users/.../samples/rdb02.gif")
    #[wasm_bindgen(js_name = injectExternalImage)]
    pub fn inject_external_image(
        &mut self,
        basename: &str,
        data: &[u8],
        display_path: &str,
    ) -> u32 {
        use crate::model::control::Control;
        use crate::model::shape::ShapeObject;
        use std::collections::BTreeSet;

        let mut injected: u32 = 0;
        // 영역 외부 image 영역 영역 영역 영역 basename 매칭 영역 영역 id 수집
        let mut targets: BTreeSet<u16> = BTreeSet::new();
        for section in &self.document().sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    let pic = match ctrl {
                        Control::Picture(p) => p,
                        Control::Shape(s) => match s.as_ref() {
                            ShapeObject::Picture(p) => p,
                            _ => continue,
                        },
                        _ => continue,
                    };
                    if let Some(ref path) = pic.image_attr.external_path {
                        let path_basename = path
                            .rsplit(|c| c == '/' || c == '\\')
                            .next()
                            .unwrap_or(path);
                        if path_basename != basename {
                            continue;
                        }
                        let id = pic.image_attr.bin_data_id;
                        if self.document().external_image_loaded(id) {
                            continue;
                        }
                        targets.insert(id);
                    }
                }
            }
        }

        for id in targets {
            injected +=
                self.inject_external_image_by_bin_data_id(id, data, display_path, Some(basename));
        }

        if injected > 0 {
            self.invalidate_page_tree_cache();
        }

        injected
    }

    /// [Task #1143] `getExternalImageReferences()` 의 key로 외부 이미지 bytes를 주입한다.
    ///
    /// 지원 key: `binData:<bin_data_id>`.
    /// 잘못된 key, 존재하지 않는 key, 이미 loaded 상태인 reference는 0을 반환한다.
    #[wasm_bindgen(js_name = injectExternalImageByKey)]
    pub fn inject_external_image_by_key(
        &mut self,
        key: &str,
        data: &[u8],
        display_path: &str,
    ) -> u32 {
        let Some(bin_data_id) = parse_external_image_key(key) else {
            return 0;
        };

        let injected =
            self.inject_external_image_by_bin_data_id(bin_data_id, data, display_path, None);
        if injected > 0 {
            self.invalidate_page_tree_cache();
        }
        injected
    }

    /// 그림 컨트롤의 속성을 조회한다.
    ///
    /// 반환: JSON `{ width, height, treatAsChar, ... }`
    #[wasm_bindgen(js_name = getPictureProperties)]
    pub fn get_picture_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_picture_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #825] 머리말/꼬리말 안 그림의 속성 조회.
    /// path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
    ///       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
    #[wasm_bindgen(js_name = getHeaderFooterPictureProperties)]
    pub fn get_header_footer_picture_properties(
        &self,
        section_idx: u32,
        outer_para_idx: u32,
        outer_control_idx: u32,
        inner_para_idx: u32,
        inner_control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_header_footer_picture_properties_native(
            section_idx as usize,
            outer_para_idx as usize,
            outer_control_idx as usize,
            inner_para_idx as usize,
            inner_control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 그림 컨트롤의 속성을 변경한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = setPictureProperties)]
    pub fn set_picture_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_picture_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// [Task #825] 머리말/꼬리말 안 그림 속성 변경.
    #[wasm_bindgen(js_name = setHeaderFooterPictureProperties)]
    pub fn set_header_footer_picture_properties(
        &mut self,
        section_idx: u32,
        outer_para_idx: u32,
        outer_control_idx: u32,
        inner_para_idx: u32,
        inner_control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_header_footer_picture_properties_native(
            section_idx as usize,
            outer_para_idx as usize,
            outer_control_idx as usize,
            inner_para_idx as usize,
            inner_control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// 그림 컨트롤을 문단에서 삭제한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = deletePictureControl)]
    pub fn delete_picture_control(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_picture_control_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (by_path).
    #[wasm_bindgen(js_name = deleteCellPictureControlByPath)]
    pub fn delete_cell_picture_control_by_path(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        cell_path_json: &str,
        inner_control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_cell_picture_control_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            cell_path_json,
            inner_control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1138] 표 셀 내 Shape(글상자/사각형/도형) 속성 조회 (by_path).
    #[wasm_bindgen(js_name = getCellShapePropertiesByPath)]
    pub fn get_cell_shape_properties_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        cell_path_json: &str,
        inner_control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_cell_shape_properties_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            cell_path_json,
            inner_control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1151 v4] 표 셀 내 Picture 속성 조회 (by_path). Shape 패턴 정합.
    #[wasm_bindgen(js_name = getCellPicturePropertiesByPath)]
    pub fn get_cell_picture_properties_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        cell_path_json: &str,
        inner_control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_cell_picture_properties_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            cell_path_json,
            inner_control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1138] 표 셀 내 Shape 속성 변경 (by_path).
    #[wasm_bindgen(js_name = setCellShapePropertiesByPath)]
    pub fn set_cell_shape_properties_by_path(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        cell_path_json: &str,
        inner_control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_cell_shape_properties_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            cell_path_json,
            inner_control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// [Task #1151 v4] 표 셀 내 Picture 속성 변경 (by_path). Shape 패턴 정합.
    #[wasm_bindgen(js_name = setCellPicturePropertiesByPath)]
    pub fn set_cell_picture_properties_by_path(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        cell_path_json: &str,
        inner_control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_cell_picture_properties_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            cell_path_json,
            inner_control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    // ─── Equation(수식) API ──────────────────────────────

    /// 수식 컨트롤을 문단에서 삭제한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = deleteEquationControl)]
    pub fn delete_equation_control(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_equation_control_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 수식 컨트롤의 속성을 조회한다.
    ///
    /// 반환: JSON `{ script, fontSize, color, baseline, fontName }`
    #[wasm_bindgen(js_name = getEquationProperties)]
    pub fn get_equation_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: i32,
        cell_para_idx: i32,
    ) -> Result<String, JsValue> {
        let ci = if cell_idx >= 0 {
            Some(cell_idx as usize)
        } else {
            None
        };
        let cpi = if cell_para_idx >= 0 {
            Some(cell_para_idx as usize)
        } else {
            None
        };
        self.get_equation_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            ci,
            cpi,
        )
        .map_err(|e| e.into())
    }

    /// 수식 컨트롤의 속성을 변경한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = setEquationProperties)]
    pub fn set_equation_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: i32,
        cell_para_idx: i32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        let ci = if cell_idx >= 0 {
            Some(cell_idx as usize)
        } else {
            None
        };
        let cpi = if cell_para_idx >= 0 {
            Some(cell_para_idx as usize)
        } else {
            None
        };
        self.set_equation_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            ci,
            cpi,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 수식 컨트롤의 속성을 조회한다.
    #[wasm_bindgen(js_name = getNoteEquationProperties)]
    pub fn get_note_equation_properties(
        &self,
        kind: &str,
        section_idx: u32,
        parent_para_idx: u32,
        note_control_idx: u32,
        note_para_idx: u32,
        inner_control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_note_equation_properties_native(
            kind,
            section_idx as usize,
            parent_para_idx as usize,
            note_control_idx as usize,
            note_para_idx as usize,
            inner_control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 수식 컨트롤의 속성을 변경한다.
    #[wasm_bindgen(js_name = setNoteEquationProperties)]
    pub fn set_note_equation_properties(
        &mut self,
        kind: &str,
        section_idx: u32,
        parent_para_idx: u32,
        note_control_idx: u32,
        note_para_idx: u32,
        inner_control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_note_equation_properties_native(
            kind,
            section_idx as usize,
            parent_para_idx as usize,
            note_control_idx as usize,
            note_para_idx as usize,
            inner_control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// `setNoteEquationProperties` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ kind, sectionIdx, parentParaIdx, noteControlIdx, noteParaIdx,
    /// innerControlIdx, props: object }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = setNoteEquationPropertiesEx)]
    pub fn set_note_equation_properties_ex(
        &mut self,
        options_json: &str,
    ) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_object, json_str, json_u32};
        let props_json = json_object(options_json, "props").unwrap_or_else(|| "{}".to_string());
        self.set_note_equation_properties_native(
            &json_str(options_json, "kind").unwrap_or_default(),
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "noteControlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "noteParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "innerControlIdx").unwrap_or(0) as usize,
            &props_json,
        )
        .map_err(|e| e.into())
    }

    /// 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
    ///
    /// 반환: 완전한 `<svg>` 문자열
    #[wasm_bindgen(js_name = renderEquationPreview)]
    pub fn render_equation_preview(
        &self,
        script: &str,
        font_size_hwpunit: u32,
        color: u32,
    ) -> Result<String, JsValue> {
        self.render_equation_preview_native(script, font_size_hwpunit, color)
            .map_err(|e| e.into())
    }

    /// JSON에서 polygonPoints 배열 파싱
    fn parse_polygon_points(json: &str) -> Vec<crate::model::Point> {
        // 간단한 파싱: "polygonPoints":[{"x":1,"y":2},{"x":3,"y":4}]
        let key = "\"polygonPoints\":[";
        if let Some(start) = json.find(key) {
            let rest = &json[start + key.len()..];
            if let Some(end) = rest.find(']') {
                let arr = &rest[..end];
                return arr
                    .split("},")
                    .filter_map(|item| {
                        let item = item.trim().trim_start_matches('{').trim_end_matches('}');
                        let x =
                            crate::document_core::helpers::json_i32(&format!("{{{}}}", item), "x")?;
                        let y =
                            crate::document_core::helpers::json_i32(&format!("{{{}}}", item), "y")?;
                        Some(crate::model::Point { x, y })
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    // ─── Shape(글상자) API ───────────────────────────────

    /// 커서 위치에 글상자(Rectangle + TextBox)를 삽입한다.
    ///
    /// json: `{"sectionIdx":N,"paraIdx":N,"charOffset":N,"width":N,"height":N,
    ///         "horzOffset":N,"vertOffset":N,"treatAsChar":bool,"textWrap":"Square"}`
    /// 반환: JSON `{"ok":true,"paraIdx":<N>,"controlIdx":0}`
    #[wasm_bindgen(js_name = createShapeControl)]
    pub fn create_shape_control(&mut self, json: &str) -> Result<String, JsValue> {
        let sec = json_u32(json, "sectionIdx").unwrap_or(0) as usize;
        let para = json_u32(json, "paraIdx").unwrap_or(0) as usize;
        let offset = json_u32(json, "charOffset").unwrap_or(0) as usize;
        let width = json_u32(json, "width").unwrap_or(8504);
        let height = json_u32(json, "height").unwrap_or(8504);
        let horz_offset = json_u32(json, "horzOffset").unwrap_or(0);
        let vert_offset = json_u32(json, "vertOffset").unwrap_or(0);
        let shape_type = json_str(json, "shapeType").unwrap_or_else(|| "rectangle".to_string());
        // 글상자는 기본적으로 treat_as_char=true (한컴 기본값)
        let default_tac = shape_type == "textbox";
        let treat_as_char = json_bool(json, "treatAsChar").unwrap_or(default_tac);
        let text_wrap = json_str(json, "textWrap").unwrap_or_else(|| "Square".to_string());
        let line_flip_x = json_bool(json, "lineFlipX").unwrap_or(false);
        let line_flip_y = json_bool(json, "lineFlipY").unwrap_or(false);
        // 다각형 꼭짓점: "polygonPoints":[{"x":N,"y":N},...]
        let polygon_points: Vec<crate::model::Point> = if shape_type == "polygon" {
            Self::parse_polygon_points(json)
        } else {
            Vec::new()
        };
        let result = self.create_shape_control_native(
            sec,
            para,
            offset,
            width,
            height,
            horz_offset,
            vert_offset,
            treat_as_char,
            &text_wrap,
            &shape_type,
            line_flip_x,
            line_flip_y,
            &polygon_points,
        )?;

        // 연결선: SubjectID + 제어점 라우팅 설정 (생성 후)
        if shape_type.starts_with("connector-") {
            let ssid = json_u32(json, "startSubjectID").unwrap_or(0);
            let ssidx = json_u32(json, "startSubjectIndex").unwrap_or(0);
            let esid = json_u32(json, "endSubjectID").unwrap_or(0);
            let esidx = json_u32(json, "endSubjectIndex").unwrap_or(0);
            let pi = json_u32(&result, "paraIdx");
            let ci = json_u32(&result, "controlIdx");
            if let (Some(pi), Some(ci)) = (pi, ci) {
                self.update_connector_subject_ids(
                    sec,
                    pi as usize,
                    ci as usize,
                    ssid,
                    ssidx,
                    esid,
                    esidx,
                );
                self.recalculate_connector_routing(sec, pi as usize, ci as usize, ssidx, esidx);
            }
        }

        Ok(result)
    }

    /// Shape(글상자) 속성을 조회한다.
    ///
    /// 반환: JSON `{ width, height, treatAsChar, tbMarginLeft, ... }`
    #[wasm_bindgen(js_name = getShapeProperties)]
    pub fn get_shape_properties(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_shape_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// Shape(글상자) 속성을 변경한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = setShapeProperties)]
    pub fn set_shape_properties(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.set_shape_properties_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// Shape(글상자) 컨트롤을 문단에서 삭제한다.
    ///
    /// 반환: JSON `{"ok":true}`
    #[wasm_bindgen(js_name = deleteShapeControl)]
    pub fn delete_shape_control(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_shape_control_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// Shape z-order 변경
    /// operation: "front" | "back" | "forward" | "backward"
    #[wasm_bindgen(js_name = changeShapeZOrder)]
    pub fn change_shape_z_order(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        operation: &str,
    ) -> Result<String, JsValue> {
        self.change_shape_z_order_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            operation,
        )
        .map_err(|e| e.into())
    }

    /// 선택된 개체들을 하나의 GroupShape로 묶는다.
    /// json: `{"sectionIdx":N, "targets":[{"paraIdx":N,"controlIdx":N},...]}`
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
    #[wasm_bindgen(js_name = groupShapes)]
    pub fn group_shapes(&mut self, json: &str) -> Result<String, JsValue> {
        let sec = json_u32(json, "sectionIdx").unwrap_or(0) as usize;
        // targets 배열 파싱
        let targets: Vec<(usize, usize)> = {
            let mut result = Vec::new();
            // 간단한 JSON 배열 파싱: "targets":[{"paraIdx":N,"controlIdx":N},...]
            if let Some(start) = json.find("\"targets\"") {
                let rest = &json[start..];
                if let Some(arr_start) = rest.find('[') {
                    if let Some(arr_end) = rest.find(']') {
                        let arr = &rest[arr_start + 1..arr_end];
                        // 각 {} 블록에서 paraIdx, controlIdx 추출
                        let mut pos = 0;
                        while let Some(obj_start) = arr[pos..].find('{') {
                            let obj_start = pos + obj_start;
                            if let Some(obj_end) = arr[obj_start..].find('}') {
                                let obj = &arr[obj_start..obj_start + obj_end + 1];
                                let pi = json_u32(obj, "paraIdx").unwrap_or(0) as usize;
                                let ci = json_u32(obj, "controlIdx").unwrap_or(0) as usize;
                                result.push((pi, ci));
                                pos = obj_start + obj_end + 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            result
        };
        self.group_shapes_native(sec, &targets)
            .map_err(|e| e.into())
    }

    /// GroupShape를 풀어 자식 개체들을 개별로 복원한다.
    #[wasm_bindgen(js_name = ungroupShape)]
    pub fn ungroup_shape(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.ungroup_shape_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 직선 끝점 이동 (글로벌 HWPUNIT 좌표)
    #[wasm_bindgen(js_name = moveLineEndpoint)]
    pub fn move_line_endpoint(
        &mut self,
        sec: u32,
        para: u32,
        ci: u32,
        sx: i32,
        sy: i32,
        ex: i32,
        ey: i32,
    ) -> Result<String, JsValue> {
        self.move_line_endpoint_native(sec as usize, para as usize, ci as usize, sx, sy, ex, ey)
            .map_err(|e| e.into())
    }

    /// `moveLineEndpoint` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sec, para, ci, sx, sy, ex, ey }` (좌표는 i32). positional 과 동일 동작.
    #[wasm_bindgen(js_name = moveLineEndpointEx)]
    pub fn move_line_endpoint_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_i32, json_u32};
        self.move_line_endpoint_native(
            json_u32(options_json, "sec").unwrap_or(0) as usize,
            json_u32(options_json, "para").unwrap_or(0) as usize,
            json_u32(options_json, "ci").unwrap_or(0) as usize,
            json_i32(options_json, "sx").unwrap_or(0),
            json_i32(options_json, "sy").unwrap_or(0),
            json_i32(options_json, "ex").unwrap_or(0),
            json_i32(options_json, "ey").unwrap_or(0),
        )
        .map_err(|e| e.into())
    }

    /// 구역 내 모든 연결선의 좌표를 연결된 도형 위치에 맞게 갱신한다.
    #[wasm_bindgen(js_name = updateConnectorsInSection)]
    pub fn update_connectors_in_section_wasm(&mut self, section_idx: u32) {
        self.update_connectors_in_section(section_idx as usize);
    }

    /// 각주를 삽입한다.
    #[wasm_bindgen(js_name = insertFootnote)]
    pub fn insert_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.insert_footnote_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 미주를 삽입한다.
    #[wasm_bindgen(js_name = insertEndnote)]
    pub fn insert_endnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.insert_endnote_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 미주 모양을 조회한다.
    #[wasm_bindgen(js_name = getEndnoteShape)]
    pub fn get_endnote_shape(&self, section_idx: u32) -> Result<String, JsValue> {
        self.get_endnote_shape_native(section_idx as usize)
            .map_err(|e| e.into())
    }

    /// 미주 모양을 적용한다.
    #[wasm_bindgen(js_name = applyEndnoteShape)]
    pub fn apply_endnote_shape(
        &mut self,
        section_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_endnote_shape_native(section_idx as usize, props_json)
            .map_err(|e| e.into())
    }

    /// 수식을 삽입한다.
    #[wasm_bindgen(js_name = insertEquation)]
    pub fn insert_equation(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        script: &str,
        font_size: u32,
        color: u32,
    ) -> Result<String, JsValue> {
        self.insert_equation_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            script,
            font_size,
            color,
        )
        .map_err(|e| e.into())
    }

    /// 각주 정보를 조회한다.
    #[wasm_bindgen(js_name = getFootnoteInfo)]
    pub fn get_footnote_info(
        &self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_footnote_info_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 본문 커서 위치의 각주 마커를 조회한다.
    ///
    /// direction: "backward" 또는 "forward"
    #[wasm_bindgen(js_name = getFootnoteAtCursor)]
    pub fn get_footnote_at_cursor(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        direction: &str,
    ) -> Result<String, JsValue> {
        self.get_footnote_at_cursor_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            direction,
        )
        .map_err(|e| e.into())
    }

    /// 본문 각주 컨트롤을 삭제한다.
    #[wasm_bindgen(js_name = deleteFootnote)]
    pub fn delete_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.delete_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주 내 텍스트를 삽입한다.
    #[wasm_bindgen(js_name = insertTextInFootnote)]
    pub fn insert_text_in_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
        char_offset: u32,
        text: &str,
    ) -> Result<String, JsValue> {
        self.insert_text_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
            char_offset as usize,
            text,
        )
        .map_err(|e| e.into())
    }

    /// 각주 내 텍스트를 삭제한다.
    #[wasm_bindgen(js_name = deleteTextInFootnote)]
    pub fn delete_text_in_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
        char_offset: u32,
        count: u32,
    ) -> Result<String, JsValue> {
        self.delete_text_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
            char_offset as usize,
            count as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주 내 문단을 분할한다 (Enter).
    #[wasm_bindgen(js_name = splitParagraphInFootnote)]
    pub fn split_paragraph_in_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.split_paragraph_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주 내 문단을 병합한다 (Backspace at start).
    #[wasm_bindgen(js_name = mergeParagraphInFootnote)]
    pub fn merge_paragraph_in_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
    ) -> Result<String, JsValue> {
        self.merge_paragraph_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주 영역 히트테스트
    #[wasm_bindgen(js_name = hitTestFootnote)]
    pub fn hit_test_footnote(&self, page_num: u32, x: f64, y: f64) -> Result<String, JsValue> {
        self.hit_test_footnote_native(page_num, x, y)
            .map_err(|e| e.into())
    }

    /// 각주 내부 텍스트 히트테스트
    #[wasm_bindgen(js_name = hitTestInFootnote)]
    pub fn hit_test_in_footnote(&self, page_num: u32, x: f64, y: f64) -> Result<String, JsValue> {
        self.hit_test_in_footnote_native(page_num, x, y)
            .map_err(|e| e.into())
    }

    /// 페이지의 각주 참조 정보
    #[wasm_bindgen(js_name = getPageFootnoteInfo)]
    pub fn get_page_footnote_info(
        &self,
        page_num: u32,
        footnote_index: u32,
    ) -> Result<String, JsValue> {
        self.get_page_footnote_info_native(page_num, footnote_index as usize)
            .map_err(|e| e.into())
    }

    /// 각주 내 커서 렉트 계산
    #[wasm_bindgen(js_name = getCursorRectInFootnote)]
    pub fn get_cursor_rect_in_footnote(
        &self,
        page_num: u32,
        footnote_index: u32,
        fn_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_in_footnote_native(
            page_num,
            footnote_index as usize,
            fn_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 편집 모드 진입 대상 조회
    #[wasm_bindgen(js_name = getNoteEditInfo)]
    pub fn get_note_edit_info(
        &self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_note_edit_info_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 커서 렉트 계산
    #[wasm_bindgen(js_name = getCursorRectInNote)]
    pub fn get_cursor_rect_in_note(
        &self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        note_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_in_note_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            note_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 문단 속성 조회
    #[wasm_bindgen(js_name = getParaPropertiesInFootnote)]
    pub fn get_para_properties_in_footnote(
        &self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
    ) -> Result<String, JsValue> {
        self.get_para_properties_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 문단 속성 적용
    #[wasm_bindgen(js_name = applyParaFormatInFootnote)]
    pub fn apply_para_format_in_footnote(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        control_idx: u32,
        fn_para_idx: u32,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_para_format_in_footnote_native(
            section_idx as usize,
            para_idx as usize,
            control_idx as usize,
            fn_para_idx as usize,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// 본문 인라인 각주 마커 히트테스트
    #[wasm_bindgen(js_name = hitTestBodyFootnoteMarker)]
    pub fn hit_test_body_footnote_marker(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, JsValue> {
        self.hit_test_body_footnote_marker_native(page_num, x, y)
            .map_err(|e| e.into())
    }

    /// 수직 커서 이동 (ArrowUp/Down) — 단일 호출로 줄/문단/표/구역 경계를 모두 처리한다.
    ///
    /// delta: -1=위, +1=아래
    /// preferred_x: 이전 반환값의 preferredX (최초 이동 시 -1.0 전달)
    /// 셀 컨텍스트: 본문이면 모두 0xFFFFFFFF 전달
    ///
    /// 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
    #[wasm_bindgen(js_name = moveVertical)]
    pub fn move_vertical(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        delta: i32,
        preferred_x: f64,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
    ) -> Result<String, JsValue> {
        let cell_ctx = if parent_para_idx == u32::MAX {
            None
        } else {
            Some((
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
                cell_para_idx as usize,
            ))
        };
        self.move_vertical_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            delta,
            preferred_x,
            cell_ctx,
        )
        .map_err(|e| e.into())
    }

    /// `moveVertical` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, paraIdx, charOffset?, delta, preferredX,
    /// parentParaIdx?, controlIdx?, cellIdx?, cellParaIdx? }`. cell 컨텍스트 키가 모두
    /// 생략되면 본문 이동(parentParaIdx=MAX 동작과 동일). positional 과 동일 동작.
    #[wasm_bindgen(js_name = moveVerticalEx)]
    pub fn move_vertical_ex(&self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_f64, json_i32, json_u32};
        // parentParaIdx 부재 시 u32::MAX(본문) — positional 분기와 동일.
        let parent_para_idx = json_u32(options_json, "parentParaIdx").unwrap_or(u32::MAX);
        let cell_ctx = if parent_para_idx == u32::MAX {
            None
        } else {
            Some((
                parent_para_idx as usize,
                json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
                json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
                json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            ))
        };
        self.move_vertical_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "paraIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_i32(options_json, "delta").unwrap_or(0),
            json_f64(options_json, "preferredX").unwrap_or(0.0),
            cell_ctx,
        )
        .map_err(|e| e.into())
    }

    // ─── 필드 API (Task 230) ─────────────────────────────────

    /// 문서 내 모든 필드 목록을 JSON 배열로 반환한다.
    ///
    /// 반환: `[{fieldId, fieldType, name, guide, command, value, location}]`
    #[wasm_bindgen(js_name = getFieldList)]
    pub fn get_field_list(&self) -> String {
        self.get_field_list_json()
    }

    /// field_id로 필드 값을 조회한다.
    ///
    /// 반환: `{ok, value}`
    #[wasm_bindgen(js_name = getFieldValue)]
    pub fn get_field_value(&self, field_id: u32) -> Result<String, JsValue> {
        self.get_field_value_by_id(field_id).map_err(|e| e.into())
    }

    /// 필드 이름으로 값을 조회한다.
    ///
    /// 반환: `{ok, fieldId, value}`
    #[wasm_bindgen(js_name = getFieldValueByName)]
    pub fn get_field_value_by_name_api(&self, name: &str) -> Result<String, JsValue> {
        self.get_field_value_by_name(name).map_err(|e| e.into())
    }

    /// field_id로 필드 값을 설정한다.
    ///
    /// 반환: `{ok, fieldId, oldValue, newValue}`
    #[wasm_bindgen(js_name = setFieldValue)]
    pub fn set_field_value(&mut self, field_id: u32, value: &str) -> Result<String, JsValue> {
        self.set_field_value_by_id(field_id, value)
            .map_err(|e| e.into())
    }

    /// 필드 이름으로 값을 설정한다.
    ///
    /// 반환: `{ok, fieldId, oldValue, newValue}`
    #[wasm_bindgen(js_name = setFieldValueByName)]
    pub fn set_field_value_by_name_api(
        &mut self,
        name: &str,
        value: &str,
    ) -> Result<String, JsValue> {
        self.set_field_value_by_name(name, value)
            .map_err(|e| e.into())
    }

    /// 현재 본문 위치에 ClickHere 누름틀 필드를 삽입한다.
    #[wasm_bindgen(js_name = insertClickHereField)]
    pub fn insert_click_here_field_api(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, JsValue> {
        self.insert_click_here_field_at(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            guide,
            memo,
            name,
            editable,
        )
        .map_err(|e| e.into())
    }

    /// `insertClickHereField` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, paraIdx, charOffset?, guide?, memo?, name?, editable? }`.
    /// positional 과 동일 동작.
    #[wasm_bindgen(js_name = insertClickHereFieldEx)]
    pub fn insert_click_here_field_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_str, json_u32};
        self.insert_click_here_field_at(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "paraIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            &json_str(options_json, "guide").unwrap_or_default(),
            &json_str(options_json, "memo").unwrap_or_default(),
            &json_str(options_json, "name").unwrap_or_default(),
            json_bool(options_json, "editable").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    /// 현재 셀/글상자 위치에 ClickHere 누름틀 필드를 삽입한다.
    #[wasm_bindgen(js_name = insertClickHereFieldInCell)]
    pub fn insert_click_here_field_in_cell_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        is_textbox: bool,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, JsValue> {
        self.insert_click_here_field_at_in_cell(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            is_textbox,
            guide,
            memo,
            name,
            editable,
        )
        .map_err(|e| e.into())
    }

    /// `insertClickHereFieldInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, isTextbox?, guide?, memo?, name?, editable? }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = insertClickHereFieldInCellEx)]
    pub fn insert_click_here_field_in_cell_ex(
        &mut self,
        options_json: &str,
    ) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_str, json_u32};
        self.insert_click_here_field_at_in_cell(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_bool(options_json, "isTextbox").unwrap_or(false),
            &json_str(options_json, "guide").unwrap_or_default(),
            &json_str(options_json, "memo").unwrap_or_default(),
            &json_str(options_json, "name").unwrap_or_default(),
            json_bool(options_json, "editable").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    /// 현재 중첩 표 cellPath 위치에 ClickHere 누름틀 필드를 삽입한다.
    #[wasm_bindgen(js_name = insertClickHereFieldByPath)]
    pub fn insert_click_here_field_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.insert_click_here_field_at_by_path(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
            guide,
            memo,
            name,
            editable,
        )
        .map_err(|e| e.into())
    }

    /// `insertClickHereFieldByPath` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, path: string, charOffset?, guide?,
    /// memo?, name?, editable? }`. `path` 는 cell_path JSON 문자열. positional 과 동일 동작.
    #[wasm_bindgen(js_name = insertClickHereFieldByPathEx)]
    pub fn insert_click_here_field_by_path_ex(
        &mut self,
        options_json: &str,
    ) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_str, json_u32};
        let path_json = json_str(options_json, "path").unwrap_or_default();
        let path = DocumentCore::parse_cell_path(&path_json)?;
        self.insert_click_here_field_at_by_path(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            &path,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            &json_str(options_json, "guide").unwrap_or_default(),
            &json_str(options_json, "memo").unwrap_or_default(),
            &json_str(options_json, "name").unwrap_or_default(),
            json_bool(options_json, "editable").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    // ─────────────────────────────────────────────
    // 양식 개체(Form Object) API
    // ─────────────────────────────────────────────

    /// 페이지 좌표에서 양식 개체를 찾는다.
    ///
    /// 반환: `{found, sec, para, ci, formType, name, value, caption, text, bbox}`
    #[wasm_bindgen(js_name = getFormObjectAt)]
    pub fn get_form_object_at(&self, page_num: u32, x: f64, y: f64) -> Result<String, JsValue> {
        self.core
            .get_form_object_at_native(page_num, x, y)
            .map_err(|e| e.into())
    }

    /// 양식 개체 값을 조회한다.
    ///
    /// 반환: `{ok, formType, name, value, text, caption, enabled}`
    #[wasm_bindgen(js_name = getFormValue)]
    pub fn get_form_value(&self, sec: u32, para: u32, ci: u32) -> Result<String, JsValue> {
        self.core
            .get_form_value_native(sec as usize, para as usize, ci as usize)
            .map_err(|e| e.into())
    }

    /// 양식 개체 값을 설정한다.
    ///
    /// value_json: `{"value":1}` 또는 `{"text":"입력값"}`
    /// 반환: `{ok}`
    #[wasm_bindgen(js_name = setFormValue)]
    pub fn set_form_value(
        &mut self,
        sec: u32,
        para: u32,
        ci: u32,
        value_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .set_form_value_native(sec as usize, para as usize, ci as usize, value_json)
            .map_err(|e| e.into())
    }

    /// 셀 내부 양식 개체 값을 설정한다.
    ///
    /// table_para: 표를 포함한 최상위 문단 인덱스
    /// table_ci: 표 컨트롤 인덱스
    /// cell_idx: 셀 인덱스
    /// cell_para: 셀 내 문단 인덱스
    /// form_ci: 셀 내 양식 컨트롤 인덱스
    /// value_json: `{"value":1}` 또는 `{"text":"입력값"}`
    /// 반환: `{ok}`
    #[wasm_bindgen(js_name = setFormValueInCell)]
    pub fn set_form_value_in_cell(
        &mut self,
        sec: u32,
        table_para: u32,
        table_ci: u32,
        cell_idx: u32,
        cell_para: u32,
        form_ci: u32,
        value_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .set_form_value_in_cell_native(
                sec as usize,
                table_para as usize,
                table_ci as usize,
                cell_idx as usize,
                cell_para as usize,
                form_ci as usize,
                value_json,
            )
            .map_err(|e| e.into())
    }

    /// `setFormValueInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sec, tablePara, tableCi, cellIdx, cellPara, formCi, value: object }`.
    /// positional 과 동일 동작.
    #[wasm_bindgen(js_name = setFormValueInCellEx)]
    pub fn set_form_value_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_object, json_u32};
        let value_json = json_object(options_json, "value").unwrap_or_else(|| "{}".to_string());
        self.core
            .set_form_value_in_cell_native(
                json_u32(options_json, "sec").unwrap_or(0) as usize,
                json_u32(options_json, "tablePara").unwrap_or(0) as usize,
                json_u32(options_json, "tableCi").unwrap_or(0) as usize,
                json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
                json_u32(options_json, "cellPara").unwrap_or(0) as usize,
                json_u32(options_json, "formCi").unwrap_or(0) as usize,
                &value_json,
            )
            .map_err(|e| e.into())
    }

    /// 양식 개체 상세 정보를 반환한다 (properties 포함).
    ///
    /// 반환: `{ok, formType, name, value, text, caption, enabled, width, height, foreColor, backColor, properties}`
    #[wasm_bindgen(js_name = getFormObjectInfo)]
    pub fn get_form_object_info(&self, sec: u32, para: u32, ci: u32) -> Result<String, JsValue> {
        self.core
            .get_form_object_info_native(sec as usize, para as usize, ci as usize)
            .map_err(|e| e.into())
    }

    // ── 검색/치환 API ──

    /// 문서 텍스트 검색
    #[wasm_bindgen(js_name = searchText)]
    pub fn search_text(
        &self,
        query: &str,
        from_sec: u32,
        from_para: u32,
        from_char: u32,
        forward: bool,
        case_sensitive: bool,
    ) -> Result<String, JsValue> {
        self.core
            .search_text_native(
                query,
                from_sec as usize,
                from_para as usize,
                from_char as usize,
                forward,
                case_sensitive,
            )
            .map_err(|e| e.into())
    }

    /// 문서 전체 검색 (모든 매치 반환)
    #[wasm_bindgen(js_name = searchAllText)]
    pub fn search_all_text(
        &self,
        query: &str,
        case_sensitive: bool,
        include_cells: bool,
    ) -> Result<String, JsValue> {
        self.core
            .search_all_text_native(query, case_sensitive, include_cells)
            .map_err(|e| e.into())
    }

    /// 텍스트 치환 (단일)
    #[wasm_bindgen(js_name = replaceText)]
    pub fn replace_text(
        &mut self,
        sec: u32,
        para: u32,
        char_offset: u32,
        length: u32,
        new_text: &str,
    ) -> Result<String, JsValue> {
        self.core
            .replace_text_native(
                sec as usize,
                para as usize,
                char_offset as usize,
                length as usize,
                new_text,
            )
            .map_err(|e| e.into())
    }

    /// 단일 치환 (검색어 기반) — 첫 번째 매치만 교체
    #[wasm_bindgen(js_name = replaceOne)]
    pub fn replace_one(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
    ) -> Result<String, JsValue> {
        self.core
            .replace_one_native(query, new_text, case_sensitive)
            .map_err(|e| e.into())
    }

    /// 전체 치환
    #[wasm_bindgen(js_name = replaceAll)]
    pub fn replace_all(
        &mut self,
        query: &str,
        new_text: &str,
        case_sensitive: bool,
    ) -> Result<String, JsValue> {
        self.core
            .replace_all_native(query, new_text, case_sensitive)
            .map_err(|e| e.into())
    }

    /// 글로벌 쪽 번호에 해당하는 첫 문단 위치 반환
    #[wasm_bindgen(js_name = getPositionOfPage)]
    pub fn get_position_of_page(&self, global_page: u32) -> Result<String, JsValue> {
        self.core
            .get_position_of_page_native(global_page as usize)
            .map_err(|e| e.into())
    }

    /// 위치에 해당하는 글로벌 쪽 번호 반환
    #[wasm_bindgen(js_name = getPageOfPosition)]
    pub fn get_page_of_position(&self, section_idx: u32, para_idx: u32) -> Result<String, JsValue> {
        self.core
            .get_page_of_position_native(section_idx as usize, para_idx as usize)
            .map_err(|e| e.into())
    }

    /// 커서 위치의 필드 범위 정보를 조회한다 (본문 문단).
    ///
    /// 반환: `{inField, fieldId?, startCharIdx?, endCharIdx?, isGuide?, guideName?, editableInForm?}`
    #[wasm_bindgen(js_name = getFieldInfoAt)]
    pub fn get_field_info_at_api(
        &self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> String {
        self.get_field_info_at(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
    }

    /// 커서 위치의 필드 범위 정보를 조회한다 (셀/글상자 내 문단).
    #[wasm_bindgen(js_name = getFieldInfoAtInCell)]
    pub fn get_field_info_at_in_cell_api(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        is_textbox: bool,
    ) -> String {
        self.get_field_info_at_in_cell(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            is_textbox,
        )
    }

    /// `getFieldInfoAtInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
    #[wasm_bindgen(js_name = getFieldInfoAtInCellEx)]
    pub fn get_field_info_at_in_cell_ex(&self, options_json: &str) -> String {
        use crate::document_core::helpers::{json_bool, json_u32};
        self.get_field_info_at_in_cell(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_bool(options_json, "isTextbox").unwrap_or(false),
        )
    }

    /// 커서 위치의 누름틀 필드를 제거한다 (본문 문단).
    #[wasm_bindgen(js_name = removeFieldAt)]
    pub fn remove_field_at_api(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> String {
        match self.remove_field_at(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        ) {
            Ok(s) => s,
            Err(e) => {
                let escaped = e.to_string().replace('\\', "\\\\").replace('"', "\\\"");
                format!("{{\"ok\":false,\"error\":\"{}\"}}", escaped)
            }
        }
    }

    /// 커서 위치의 누름틀 필드를 제거한다 (셀/글상자 내 문단).
    #[wasm_bindgen(js_name = removeFieldAtInCell)]
    pub fn remove_field_at_in_cell_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        is_textbox: bool,
    ) -> String {
        match self.remove_field_at_in_cell(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            is_textbox,
        ) {
            Ok(s) => s,
            Err(e) => {
                let escaped = e.to_string().replace('\\', "\\\\").replace('"', "\\\"");
                format!("{{\"ok\":false,\"error\":\"{}\"}}", escaped)
            }
        }
    }

    /// `removeFieldAtInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, isTextbox? }`. positional 과 동일 동작(String 반환).
    #[wasm_bindgen(js_name = removeFieldAtInCellEx)]
    pub fn remove_field_at_in_cell_ex(&mut self, options_json: &str) -> String {
        use crate::document_core::helpers::{json_bool, json_u32};
        match self.remove_field_at_in_cell(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_bool(options_json, "isTextbox").unwrap_or(false),
        ) {
            Ok(s) => s,
            Err(e) => {
                let escaped = e.to_string().replace('\\', "\\\\").replace('"', "\\\"");
                format!("{{\"ok\":false,\"error\":\"{}\"}}", escaped)
            }
        }
    }

    /// 활성 필드를 설정한다 (본문 문단 — 안내문 숨김용).
    #[wasm_bindgen(js_name = setActiveField)]
    pub fn set_active_field_api(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> bool {
        self.set_active_field(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
    }

    /// 활성 필드를 설정한다 (셀/글상자 내 문단 — 안내문 숨김용).
    /// 변경이 발생하면 true를 반환한다.
    #[wasm_bindgen(js_name = setActiveFieldInCell)]
    pub fn set_active_field_in_cell_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        is_textbox: bool,
    ) -> bool {
        self.set_active_field_in_cell(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            is_textbox,
        )
    }

    /// `setActiveFieldInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, isTextbox? }`. positional 과 동일 동작(bool 반환).
    #[wasm_bindgen(js_name = setActiveFieldInCellEx)]
    pub fn set_active_field_in_cell_ex(&mut self, options_json: &str) -> bool {
        use crate::document_core::helpers::{json_bool, json_u32};
        self.set_active_field_in_cell(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            json_bool(options_json, "isTextbox").unwrap_or(false),
        )
    }

    /// path 기반: 중첩 표 셀의 필드 범위 정보를 조회한다.
    #[wasm_bindgen(js_name = getFieldInfoAtByPath)]
    pub fn get_field_info_at_by_path_api(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
    ) -> String {
        match DocumentCore::parse_cell_path(path_json) {
            Ok(path) => self.get_field_info_at_by_path(
                section_idx as usize,
                parent_para_idx as usize,
                &path,
                char_offset as usize,
            ),
            Err(_) => r#"{"inField":false}"#.to_string(),
        }
    }

    /// path 기반: 중첩 표 셀 내 활성 필드를 설정한다.
    #[wasm_bindgen(js_name = setActiveFieldByPath)]
    pub fn set_active_field_by_path_api(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
    ) -> bool {
        match DocumentCore::parse_cell_path(path_json) {
            Ok(path) => self.set_active_field_by_path(
                section_idx as usize,
                parent_para_idx as usize,
                &path,
                char_offset as usize,
            ),
            Err(_) => false,
        }
    }

    /// 활성 필드를 해제한다 (안내문 다시 표시).
    #[wasm_bindgen(js_name = clearActiveField)]
    pub fn clear_active_field_api(&mut self) {
        self.clear_active_field();
    }

    // ─── 누름틀 속성 조회/수정 API ──────────────────────────────

    /// 누름틀 필드의 속성을 조회한다.
    ///
    /// 반환: JSON `{"ok":true,"guide":"안내문","memo":"메모","name":"이름","editable":true}`
    #[wasm_bindgen(js_name = getClickHereProps)]
    pub fn get_click_here_props(&self, field_id: u32) -> String {
        use crate::model::control::{Control, FieldType};
        // 문서 전체에서 fieldId로 필드 찾기
        for sec in &self.document.sections {
            for para in &sec.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Field(f) = ctrl {
                        if f.field_id == field_id && f.field_type == FieldType::ClickHere {
                            return self.format_click_here_props(f);
                        }
                    }
                }
                // 표/글상자 내부도 탐색
                for ctrl in &para.controls {
                    let paras: Vec<&crate::model::paragraph::Paragraph> = match ctrl {
                        Control::Table(t) => t.cells.iter().flat_map(|c| &c.paragraphs).collect(),
                        Control::Shape(s) => s
                            .drawing()
                            .and_then(|d| d.text_box.as_ref())
                            .map(|tb| tb.paragraphs.iter().collect())
                            .unwrap_or_default(),
                        _ => Vec::new(),
                    };
                    for p in paras {
                        for c in &p.controls {
                            if let Control::Field(f) = c {
                                if f.field_id == field_id && f.field_type == FieldType::ClickHere {
                                    return self.format_click_here_props(f);
                                }
                            }
                        }
                    }
                }
            }
        }
        r#"{"ok":false}"#.to_string()
    }

    /// ClickHere 필드 속성을 JSON으로 포맷한다.
    fn format_click_here_props(&self, f: &crate::model::control::Field) -> String {
        let guide = f.guide_text().unwrap_or("");
        let memo = f.memo_text().unwrap_or("");
        // 필드 이름: ctrl_data_name → command Name: 키 순서
        let name = f
            .ctrl_data_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| f.extract_wstring_value("Name:"))
            .unwrap_or("");
        let editable = f.is_editable_in_form();
        format!(
            "{{\"ok\":true,\"guide\":\"{}\",\"memo\":\"{}\",\"name\":\"{}\",\"editable\":{}}}",
            json_escape(guide),
            json_escape(memo),
            json_escape(name),
            editable,
        )
    }

    /// 누름틀 필드의 속성을 수정한다.
    ///
    /// 반환: JSON `{"ok":true}` 또는 `{"ok":false}`
    #[wasm_bindgen(js_name = updateClickHereProps)]
    pub fn update_click_here_props(
        &mut self,
        field_id: u32,
        guide: &str,
        memo: &str,
        name: &str,
        editable: bool,
    ) -> String {
        use crate::model::control::{Control, Field, FieldType};

        let new_props_bit = if editable { 1u32 } else { 0u32 };

        // 필드를 찾아 수정하고, ctrl_data_records 바이너리도 갱신
        fn update_field_in_para(
            para: &mut crate::model::paragraph::Paragraph,
            field_id: u32,
            guide: &str,
            memo: &str,
            new_props_bit: u32,
            new_name: &str,
        ) -> bool {
            for (ci, ctrl) in para.controls.iter_mut().enumerate() {
                if let Control::Field(f) = ctrl {
                    if f.field_id == field_id && f.field_type == FieldType::ClickHere {
                        // guide/memo가 원본과 동일하면 command 문자열을 보존한다.
                        // 원본 command에는 trailing space 등이 포함될 수 있으므로
                        // 불필요한 재구축을 피해야 한컴 호환성이 유지된다.
                        let orig_guide = f.guide_text().unwrap_or("").to_string();
                        let orig_memo = f.memo_text().unwrap_or("").to_string();
                        if guide != orig_guide || memo != orig_memo {
                            // guide 또는 memo가 변경되었으므로 command 재구축
                            let new_command = Field::build_clickhere_command(guide, memo);
                            f.command = new_command;
                        }
                        // command가 변경되지 않았으면 원본 보존

                        f.properties = (f.properties & !1) | new_props_bit;
                        f.ctrl_data_name = if new_name.is_empty() {
                            None
                        } else {
                            Some(new_name.to_string())
                        };
                        // ctrl_data_records 바이너리 갱신
                        update_ctrl_data_name(&mut para.ctrl_data_records, ci, new_name);
                        return true;
                    }
                }
            }
            false
        }

        /// ctrl_data_records[ci]의 필드 이름 부분을 새 이름으로 재구축
        fn update_ctrl_data_name(records: &mut Vec<Option<Vec<u8>>>, ci: usize, new_name: &str) {
            // records 확장 (인덱스 부족 시)
            while records.len() <= ci {
                records.push(None);
            }
            if let Some(ref mut data) = records[ci] {
                if data.len() >= 12 {
                    // 헤더(10바이트) 보존, 이름 부분 재구축
                    let header = data[..10].to_vec();
                    let name_chars: Vec<u16> = new_name.encode_utf16().collect();
                    let name_len = name_chars.len() as u16;
                    let mut new_data = header;
                    new_data.extend_from_slice(&name_len.to_le_bytes());
                    for ch in &name_chars {
                        new_data.extend_from_slice(&ch.to_le_bytes());
                    }
                    *data = new_data;
                }
            } else {
                // CTRL_DATA가 없었던 경우: 새로 생성
                // 기본 헤더(10바이트) + 이름
                let name_chars: Vec<u16> = new_name.encode_utf16().collect();
                let name_len = name_chars.len() as u16;
                let mut data = vec![0x1Bu8, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00];
                data.extend_from_slice(&name_len.to_le_bytes());
                for ch in &name_chars {
                    data.extend_from_slice(&ch.to_le_bytes());
                }
                records[ci] = Some(data);
            }
        }

        for sec in &mut self.document.sections {
            sec.raw_stream = None;
            for para in &mut sec.paragraphs {
                if update_field_in_para(para, field_id, guide, memo, new_props_bit, name) {
                    self.invalidate_page_tree_cache();
                    return r#"{"ok":true}"#.to_string();
                }
                // 표/글상자 내부
                for ctrl in &mut para.controls {
                    let found = match ctrl {
                        Control::Table(t) => t.cells.iter_mut().any(|c| {
                            c.paragraphs.iter_mut().any(|p| {
                                update_field_in_para(p, field_id, guide, memo, new_props_bit, name)
                            })
                        }),
                        Control::Shape(s) => {
                            if let Some(tb) = s.drawing_mut().and_then(|d| d.text_box.as_mut()) {
                                tb.paragraphs.iter_mut().any(|p| {
                                    update_field_in_para(
                                        p,
                                        field_id,
                                        guide,
                                        memo,
                                        new_props_bit,
                                        name,
                                    )
                                })
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    if found {
                        self.invalidate_page_tree_cache();
                        return r#"{"ok":true}"#.to_string();
                    }
                }
            }
        }
        r#"{"ok":false}"#.to_string()
    }

    // ─── 경로 기반 중첩 표 API ───────────────────────────────

    /// 경로 기반 커서 좌표 조회 (중첩 표용).
    ///
    /// path_json: `[{"controlIndex":N,"cellIndex":N,"cellParaIndex":N}, ...]`
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    #[wasm_bindgen(js_name = getCursorRectByPath)]
    pub fn get_cursor_rect_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_cursor_rect_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            path_json,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 경로 기반 셀 정보 조회 (중첩 표용).
    ///
    /// 반환: JSON `{"row":N,"col":N,"rowSpan":N,"colSpan":N}`
    #[wasm_bindgen(js_name = getCellInfoByPath)]
    pub fn get_cell_info_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<String, JsValue> {
        self.get_cell_info_by_path_native(section_idx as usize, parent_para_idx as usize, path_json)
            .map_err(|e| e.into())
    }

    /// 경로 기반 표 차원 조회 (중첩 표용).
    ///
    /// 반환: JSON `{"rowCount":N,"colCount":N,"cellCount":N}`
    #[wasm_bindgen(js_name = getTableDimensionsByPath)]
    pub fn get_table_dimensions_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<String, JsValue> {
        self.get_table_dimensions_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            path_json,
        )
        .map_err(|e| e.into())
    }

    /// 경로 기반 표 셀 바운딩박스 조회 (중첩 표용).
    ///
    /// 반환: JSON 배열 `[{"cellIdx":N,"row":N,"col":N,...,"x":F,"y":F,"w":F,"h":F}, ...]`
    #[wasm_bindgen(js_name = getTableCellBboxesByPath)]
    pub fn get_table_cell_bboxes_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
    ) -> Result<String, JsValue> {
        self.get_table_cell_bboxes_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            path_json,
        )
        .map_err(|e| e.into())
    }

    /// 경로 기반 수직 커서 이동 (중첩 표용).
    ///
    /// 반환: JSON `{DocumentPosition + CursorRect + preferredX}`
    #[wasm_bindgen(js_name = moveVerticalByPath)]
    pub fn move_vertical_by_path(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        delta: i32,
        preferred_x: f64,
    ) -> Result<String, JsValue> {
        self.move_vertical_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            path_json,
            char_offset as usize,
            delta,
            preferred_x,
        )
        .map_err(|e| e.into())
    }

    // ─── Phase 4: Selection API ──────────────────────────────

    /// 본문 선택 영역의 줄별 사각형을 반환한다.
    ///
    /// 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
    #[wasm_bindgen(js_name = getSelectionRects)]
    pub fn get_selection_rects(
        &self,
        section_idx: u32,
        start_para_idx: u32,
        start_char_offset: u32,
        end_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_selection_rects_native(
            section_idx as usize,
            start_para_idx as usize,
            start_char_offset as usize,
            end_para_idx as usize,
            end_char_offset as usize,
            None,
        )
        .map_err(|e| e.into())
    }

    /// 셀 내 선택 영역의 줄별 사각형을 반환한다.
    ///
    /// 반환: JSON 배열 `[{"pageIndex":N,"x":F,"y":F,"width":F,"height":F}, ...]`
    #[wasm_bindgen(js_name = getSelectionRectsInCell)]
    pub fn get_selection_rects_in_cell(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        start_cell_para_idx: u32,
        start_char_offset: u32,
        end_cell_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_selection_rects_native(
            section_idx as usize,
            start_cell_para_idx as usize,
            start_char_offset as usize,
            end_cell_para_idx as usize,
            end_char_offset as usize,
            Some((
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
            )),
        )
        .map_err(|e| e.into())
    }

    /// `getSelectionRectsInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
    /// startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = getSelectionRectsInCellEx)]
    pub fn get_selection_rects_in_cell_ex(&self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.get_selection_rects_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCharOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "endCharOffset").unwrap_or(0) as usize,
            Some((
                json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
                json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
                json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            )),
        )
        .map_err(|e| e.into())
    }

    /// 각주/미주 내부 선택 영역의 줄별 사각형을 반환한다.
    #[wasm_bindgen(js_name = getSelectionRectsInFootnote)]
    pub fn get_selection_rects_in_footnote(
        &self,
        page_num: u32,
        footnote_index: u32,
        start_fn_para_idx: u32,
        start_char_offset: u32,
        end_fn_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.get_selection_rects_in_footnote_native(
            page_num,
            footnote_index as usize,
            start_fn_para_idx as usize,
            start_char_offset as usize,
            end_fn_para_idx as usize,
            end_char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 본문 선택 영역을 삭제한다.
    ///
    /// 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
    #[wasm_bindgen(js_name = deleteRange)]
    pub fn delete_range(
        &mut self,
        section_idx: u32,
        start_para_idx: u32,
        start_char_offset: u32,
        end_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.delete_range_native(
            section_idx as usize,
            start_para_idx as usize,
            start_char_offset as usize,
            end_para_idx as usize,
            end_char_offset as usize,
            None,
        )
        .map_err(|e| e.into())
    }

    /// 셀 내 선택 영역을 삭제한다.
    ///
    /// 반환: JSON `{"ok":true,"paraIdx":N,"charOffset":N}`
    #[wasm_bindgen(js_name = deleteRangeInCell)]
    pub fn delete_range_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        start_cell_para_idx: u32,
        start_char_offset: u32,
        end_cell_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.delete_range_native(
            section_idx as usize,
            start_cell_para_idx as usize,
            start_char_offset as usize,
            end_cell_para_idx as usize,
            end_char_offset as usize,
            Some((
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
            )),
        )
        .map_err(|e| e.into())
    }

    /// `deleteRangeInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
    /// startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = deleteRangeInCellEx)]
    pub fn delete_range_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.delete_range_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCharOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "endCharOffset").unwrap_or(0) as usize,
            Some((
                json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
                json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
                json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            )),
        )
        .map_err(|e| e.into())
    }

    // ─── Phase 4 끝 ─────────────────────────────────────────

    // ─── Phase 3 끝 ─────────────────────────────────────────

    // ─── Phase 2 끝 ─────────────────────────────────────────

    /// 문서를 HWP 바이너리로 내보낸다.
    ///
    /// Document IR을 HWP 5.0 CFB 바이너리로 직렬화하여 반환한다.
    /// HWPX 출처 문서는 `export_hwp_with_adapter` 를 통해 HWPX→HWP IR 매핑 어댑터를
    /// 자동 적용하여 한컴 호환성과 자기 재로드 페이지 보존을 보장한다 (#178).
    /// HWP 출처는 어댑터가 no-op 이므로 기존 동작과 동일.
    #[wasm_bindgen(js_name = exportHwp)]
    pub fn export_hwp(&mut self) -> Result<Vec<u8>, JsValue> {
        self.export_hwp_with_adapter().map_err(|e| e.into())
    }

    /// Document IR을 HWPX(ZIP+XML)로 직렬화하여 반환한다.
    #[wasm_bindgen(js_name = exportHwpx)]
    pub fn export_hwpx(&self) -> Result<Vec<u8>, JsValue> {
        self.export_hwpx_native().map_err(|e| e.into())
    }

    /// 어댑터 적용 + HWP 직렬화 + 자기 재로드 검증을 수행하고 결과를 JSON 으로 반환한다 (#178).
    ///
    /// 반환 JSON:
    /// ```json
    /// {
    ///   "bytesLen": 678912,
    ///   "pageCountBefore": 9,
    ///   "pageCountAfter": 9,
    ///   "recovered": true
    /// }
    /// ```
    ///
    /// 본 함수는 검증 메타데이터만 반환하며 bytes 자체는 별도 호출 (`exportHwp`) 로 받아야 한다.
    /// 검증과 실제 사용을 분리하여 호출자가 결과에 따라 다른 동작을 취할 수 있도록 한다.
    #[wasm_bindgen(js_name = exportHwpVerify)]
    pub fn export_hwp_verify(&mut self) -> Result<String, JsValue> {
        let v = self.serialize_hwp_with_verify().map_err(JsValue::from)?;
        Ok(format!(
            "{{\"bytesLen\":{},\"pageCountBefore\":{},\"pageCountAfter\":{},\"recovered\":{}}}",
            v.bytes_len, v.page_count_before, v.page_count_after, v.recovered
        ))
    }

    /// 원본 파일 형식을 반환한다 ("hwp" 또는 "hwpx").
    #[wasm_bindgen(js_name = getSourceFormat)]
    pub fn get_source_format(&self) -> String {
        match self.core.source_format {
            crate::parser::FileFormat::Hwpx => "hwpx".to_string(),
            _ => "hwp".to_string(),
        }
    }

    /// HWPX 비표준 감지 경고를 JSON 문자열로 반환한다 (#177).
    ///
    /// ## 반환 형식
    ///
    /// ```json
    /// {
    ///   "count": 3,
    ///   "summary": {
    ///     "lineseg 배열이 비어있음": 1,
    ///     "lineseg 가 미계산 상태 (line_height=0)": 2
    ///   },
    ///   "warnings": [
    ///     {
    ///       "section": 0,
    ///       "paragraph": 5,
    ///       "kind": "LinesegArrayEmpty",
    ///       "cell": null
    ///     },
    ///     {
    ///       "section": 0,
    ///       "paragraph": 10,
    ///       "kind": "LinesegUncomputed",
    ///       "cell": {"ctrl": 0, "row": 0, "col": 1, "innerPara": 0}
    ///     }
    ///   ]
    /// }
    /// ```
    #[wasm_bindgen(js_name = getValidationWarnings)]
    pub fn get_validation_warnings(&self) -> String {
        let report = self.core.validation_report();

        // summary 직렬화 (HashMap 순서 안정화를 위해 키 정렬)
        let mut summary_parts: Vec<String> = Vec::new();
        let mut entries: Vec<(String, usize)> = report.summary().into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in &entries {
            // 경고 메시지는 한국어 고정 문자열이므로 `"` / `\` 만 escape.
            let escaped = k.replace('\\', "\\\\").replace('"', "\\\"");
            summary_parts.push(format!("\"{}\":{}", escaped, v));
        }

        // warnings 직렬화
        let mut warning_parts: Vec<String> = Vec::new();
        for w in &report.warnings {
            let cell_part = match &w.cell_path {
                Some(cp) => format!(
                    r#"{{"ctrl":{},"row":{},"col":{},"innerPara":{}}}"#,
                    cp.table_ctrl_idx, cp.row, cp.col, cp.inner_para_idx,
                ),
                None => "null".to_string(),
            };
            let kind_name = match &w.kind {
                crate::document_core::validation::WarningKind::LinesegArrayEmpty => {
                    "LinesegArrayEmpty"
                }
                crate::document_core::validation::WarningKind::LinesegUncomputed => {
                    "LinesegUncomputed"
                }
                crate::document_core::validation::WarningKind::LinesegTextRunReflow => {
                    "LinesegTextRunReflow"
                }
            };
            warning_parts.push(format!(
                r#"{{"section":{},"paragraph":{},"kind":"{}","cell":{}}}"#,
                w.section_idx, w.paragraph_idx, kind_name, cell_part,
            ));
        }

        format!(
            r#"{{"count":{},"summary":{{{}}},"warnings":[{}]}}"#,
            report.len(),
            summary_parts.join(","),
            warning_parts.join(","),
        )
    }

    /// 사용자 명시 요청에 의한 lineseg 전체 reflow (#177).
    ///
    /// `reflow_zero_height_paragraphs` 의 자동 경로와 달리, "빈 line_segs + text 존재"
    /// 케이스까지 포함해 재계산한다. 반환값은 실제로 reflow 된 문단 개수.
    ///
    /// 호출 이후 렌더 캐시·페이지네이션이 갱신되므로 즉시 렌더링하면 보정된 결과가 보인다.
    #[wasm_bindgen(js_name = reflowLinesegs)]
    pub fn reflow_linesegs(&mut self) -> usize {
        self.core.reflow_linesegs_on_demand()
    }

    /// 배포용(읽기전용) 문서를 편집 가능한 일반 문서로 변환한다.
    ///
    /// 반환값: JSON `{"ok":true,"converted":true}` 또는 `{"ok":true,"converted":false}`
    #[wasm_bindgen(js_name = convertToEditable)]
    pub fn convert_to_editable(&mut self) -> Result<String, JsValue> {
        self.convert_to_editable_native().map_err(|e| e.into())
    }

    /// Batch 모드를 시작한다. 이후 Command 호출 시 paginate()를 건너뛴다.
    #[wasm_bindgen(js_name = beginBatch)]
    pub fn begin_batch(&mut self) -> Result<String, JsValue> {
        self.begin_batch_native().map_err(|e| e.into())
    }

    /// Batch 모드를 종료하고 누적된 이벤트를 반환한다.
    #[wasm_bindgen(js_name = endBatch)]
    pub fn end_batch(&mut self) -> Result<String, JsValue> {
        self.end_batch_native().map_err(|e| e.into())
    }

    /// 현재 이벤트 로그를 JSON으로 반환한다.
    #[wasm_bindgen(js_name = getEventLog)]
    pub fn get_event_log(&self) -> String {
        self.serialize_event_log()
    }

    // ─── Undo/Redo 스냅샷 API ──────────────────────────

    /// Document 스냅샷을 저장하고 ID를 반환한다.
    #[wasm_bindgen(js_name = saveSnapshot)]
    pub fn save_snapshot(&mut self) -> u32 {
        self.save_snapshot_native()
    }

    /// 지정 ID의 스냅샷으로 Document를 복원한다.
    #[wasm_bindgen(js_name = restoreSnapshot)]
    pub fn restore_snapshot(&mut self, id: u32) -> Result<String, JsValue> {
        self.restore_snapshot_native(id).map_err(|e| e.into())
    }

    /// 지정 ID의 스냅샷을 제거하여 메모리를 해제한다.
    #[wasm_bindgen(js_name = discardSnapshot)]
    pub fn discard_snapshot(&mut self, id: u32) {
        self.discard_snapshot_native(id)
    }

    /// 캐럿 위치의 글자 속성을 조회한다.
    ///
    /// 반환값: JSON 객체 (fontFamily, fontSize, bold, italic, underline, strikethrough, textColor 등)
    #[wasm_bindgen(js_name = getCharPropertiesAt)]
    pub fn get_char_properties_at(
        &self,
        sec_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, JsValue> {
        self.get_char_properties_at_native(sec_idx, para_idx, char_offset)
            .map_err(|e| e.into())
    }

    /// 셀 내부 문단의 글자 속성을 조회한다.
    #[wasm_bindgen(js_name = getCellCharPropertiesAt)]
    pub fn get_cell_char_properties_at(
        &self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, JsValue> {
        self.get_cell_char_properties_at_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
        )
        .map_err(|e| e.into())
    }

    /// 캐럿 위치의 문단 속성을 조회한다.
    ///
    /// 반환값: JSON 객체 (alignment, lineSpacing, marginLeft, marginRight, indent 등)
    #[wasm_bindgen(js_name = getParaPropertiesAt)]
    pub fn get_para_properties_at(
        &self,
        sec_idx: usize,
        para_idx: usize,
    ) -> Result<String, JsValue> {
        self.get_para_properties_at_native(sec_idx, para_idx)
            .map_err(|e| e.into())
    }

    /// 셀 내부 문단의 문단 속성을 조회한다.
    #[wasm_bindgen(js_name = getCellParaPropertiesAt)]
    pub fn get_cell_para_properties_at(
        &self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
    ) -> Result<String, JsValue> {
        self.get_cell_para_properties_at_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )
        .map_err(|e| e.into())
    }

    /// 문서에 정의된 스타일 목록을 조회한다.
    ///
    /// 반환값: JSON 배열 [{ id, name, englishName, type, paraShapeId, charShapeId }, ...]
    #[wasm_bindgen(js_name = getStyleList)]
    pub fn get_style_list(&self) -> String {
        let styles = &self.core.document.doc_info.styles;
        let mut items = Vec::new();
        for (i, s) in styles.iter().enumerate() {
            items.push(format!(
                "{{\"id\":{},\"name\":\"{}\",\"englishName\":\"{}\",\"type\":{},\"nextStyleId\":{},\"paraShapeId\":{},\"charShapeId\":{}}}",
                i,
                s.local_name.replace('"', "\\\""),
                s.english_name.replace('"', "\\\""),
                s.style_type,
                s.next_style_id,
                s.para_shape_id,
                s.char_shape_id
            ));
        }
        format!("[{}]", items.join(","))
    }

    /// 특정 스타일의 CharShape/ParaShape 속성을 상세 조회한다.
    ///
    /// 반환값: JSON { charProps: {...}, paraProps: {...} }
    #[wasm_bindgen(js_name = getStyleDetail)]
    pub fn get_style_detail(&self, style_id: u32) -> String {
        let styles = &self.core.document.doc_info.styles;
        let style = match styles.get(style_id as usize) {
            Some(s) => s,
            None => return "{}".to_string(),
        };
        let char_json = self
            .core
            .build_char_properties_json_by_id(style.char_shape_id);

        // 스타일의 기본 ParaShape에 번호 정보가 없으면,
        // 이 스타일을 사용하는 실제 문단의 ParaShape에서 조회
        let effective_psid =
            self.find_effective_para_shape_for_style(style_id, style.para_shape_id);
        let para_json = self.core.build_para_properties_json(effective_psid, 0);
        format!(
            "{{\"charProps\":{},\"paraProps\":{}}}",
            char_json, para_json
        )
    }

    /// 스타일의 실효 ParaShape ID를 찾는다.
    /// 스타일 정의의 ParaShape에 번호 정보가 없으면, 이 스타일을 사용하는 문단에서 조회한다.
    fn find_effective_para_shape_for_style(&self, style_id: u32, base_psid: u16) -> u16 {
        use crate::model::style::HeadType;
        // 기본 ParaShape에 이미 번호 정보가 있으면 그대로 사용
        if let Some(ps) = self
            .core
            .document
            .doc_info
            .para_shapes
            .get(base_psid as usize)
        {
            if ps.head_type != HeadType::None {
                return base_psid;
            }
        }
        // 이 스타일을 사용하는 첫 번째 문단의 para_shape_id에서 번호 정보 탐색
        let sid = style_id as u8;
        for section in &self.core.document.sections {
            for para in &section.paragraphs {
                if para.style_id == sid {
                    if let Some(ps) = self
                        .core
                        .document
                        .doc_info
                        .para_shapes
                        .get(para.para_shape_id as usize)
                    {
                        if ps.head_type != HeadType::None {
                            return para.para_shape_id;
                        }
                    }
                }
            }
        }
        base_psid
    }

    /// 스타일의 메타 정보(이름/영문이름/nextStyleId)를 수정한다.
    ///
    /// json: {"name":"...", "englishName":"...", "nextStyleId":0}
    #[wasm_bindgen(js_name = updateStyle)]
    pub fn update_style(&mut self, style_id: u32, json: &str) -> bool {
        use crate::document_core::helpers::json_i32;
        let styles = &mut self.core.document.doc_info.styles;
        let style = match styles.get_mut(style_id as usize) {
            Some(s) => s,
            None => return false,
        };
        // 이름 파싱
        if let Some(name) = crate::document_core::helpers::json_str(json, "name") {
            style.local_name = name;
        }
        if let Some(en) = crate::document_core::helpers::json_str(json, "englishName") {
            style.english_name = en;
        }
        if let Some(v) = json_i32(json, "nextStyleId") {
            style.next_style_id = v as u8;
        }
        // raw_data 무효화 (수정됨)
        style.raw_data = None;
        true
    }

    /// 스타일의 CharShape/ParaShape를 수정한다.
    ///
    /// charMods/paraMods는 기존 parse_char_shape_mods/parse_para_shape_mods와 동일한 JSON 형식
    #[wasm_bindgen(js_name = updateStyleShapes)]
    pub fn update_style_shapes(
        &mut self,
        style_id: u32,
        char_mods_json: &str,
        para_mods_json: &str,
    ) -> bool {
        let styles = &self.core.document.doc_info.styles;
        let style = match styles.get(style_id as usize) {
            Some(s) => s.clone(),
            None => return false,
        };
        let old_csid = style.char_shape_id as u32;
        let old_psid = style.para_shape_id;
        let style_type = style.style_type;

        // CharShape 수정
        if !char_mods_json.is_empty() && char_mods_json != "{}" {
            let char_mods = crate::document_core::helpers::parse_char_shape_mods(char_mods_json);
            if let Some(cs) = self
                .core
                .document
                .doc_info
                .char_shapes
                .get(style.char_shape_id as usize)
            {
                let new_cs = char_mods.apply_to(cs);
                // 새 CharShape를 추가하고 스타일에 연결
                self.core.document.doc_info.char_shapes.push(new_cs);
                let new_id = (self.core.document.doc_info.char_shapes.len() - 1) as u16;
                self.core.document.doc_info.styles[style_id as usize].char_shape_id = new_id;
            }
        }

        // ParaShape 수정
        if !para_mods_json.is_empty() && para_mods_json != "{}" {
            let para_mods = crate::document_core::helpers::parse_para_shape_mods(para_mods_json);
            if let Some(ps) = self
                .core
                .document
                .doc_info
                .para_shapes
                .get(style.para_shape_id as usize)
            {
                let new_ps = para_mods.apply_to(ps);
                self.core.document.doc_info.para_shapes.push(new_ps);
                let new_id = (self.core.document.doc_info.para_shapes.len() - 1) as u16;
                self.core.document.doc_info.styles[style_id as usize].para_shape_id = new_id;
            }
        }

        // raw_data 무효화
        self.core.document.doc_info.styles[style_id as usize].raw_data = None;
        self.core.document.doc_info.raw_stream_dirty = true;

        let sid = style_id as u8;
        let mut body_targets = Vec::new();
        let mut cell_targets = Vec::new();
        for (sec_idx, section) in self.core.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                if para.style_id == sid {
                    body_targets.push((sec_idx, para_idx));
                }
                for (control_idx, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(table) = ctrl {
                        for (cell_idx, cell) in table.cells.iter().enumerate() {
                            for (cell_para_idx, cpara) in cell.paragraphs.iter().enumerate() {
                                if cpara.style_id == sid {
                                    cell_targets.push((
                                        sec_idx,
                                        para_idx,
                                        control_idx,
                                        cell_idx,
                                        cell_para_idx,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── 스타일 변경을 해당 스타일을 사용하는 모든 문단에 전파 ──
        let updated_style = self.core.document.doc_info.styles[style_id as usize].clone();
        let new_csid = updated_style.char_shape_id as u32;
        let new_psid = updated_style.para_shape_id;

        for (sec_idx, para_idx) in body_targets {
            if let Some(para) = self
                .core
                .document
                .sections
                .get_mut(sec_idx)
                .and_then(|s| s.paragraphs.get_mut(para_idx))
            {
                if style_type == 0 && para.para_shape_id == old_psid {
                    para.para_shape_id = new_psid;
                }
                para.replace_style_char_shape_preserving_overrides(old_csid, new_csid);
            }
            self.core.reflow_body_paragraph(sec_idx, para_idx);
            if let Some(section) = self.core.document.sections.get_mut(sec_idx) {
                section.raw_stream = None;
            }
        }

        for (sec_idx, para_idx, control_idx, cell_idx, cell_para_idx) in cell_targets {
            if let Ok(cpara) = self.core.get_cell_paragraph_mut(
                sec_idx,
                para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            ) {
                if style_type == 0 && cpara.para_shape_id == old_psid {
                    cpara.para_shape_id = new_psid;
                }
                cpara.replace_style_char_shape_preserving_overrides(old_csid, new_csid);
            }
            self.core.reflow_cell_paragraph(
                sec_idx,
                para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            );
            self.core
                .mark_cell_control_dirty(sec_idx, para_idx, control_idx);
            if let Some(section) = self.core.document.sections.get_mut(sec_idx) {
                section.raw_stream = None;
            }
        }

        // 스타일 캐시 무효화 + 전체 리빌드
        let num_sections = self.core.document.sections.len();
        for sec_idx in 0..num_sections {
            self.core.rebuild_section(sec_idx);
        }
        true
    }

    /// 새 스타일을 생성한다.
    ///
    /// json: {"name":"...", "englishName":"...", "type":0, "nextStyleId":0}
    /// 반환값: 새 스타일 ID (0-based)
    #[wasm_bindgen(js_name = createStyle)]
    pub fn create_style(&mut self, json: &str) -> i32 {
        use crate::document_core::helpers::{json_i32, json_str};
        use crate::model::style::Style;

        let name = json_str(json, "name").unwrap_or_default();
        let english_name = json_str(json, "englishName").unwrap_or_default();
        let style_type = json_i32(json, "type").unwrap_or(0) as u8;
        let next_style_id = json_i32(json, "nextStyleId").unwrap_or(0) as u8;

        // 한컴 스타일 추가 흐름은 현재 문단의 모양을 기본값으로 삼는다.
        // 호출자가 base ID를 넘기지 않으면 기존 호환성을 위해 바탕글을 사용한다.
        let base_style = self.core.document.doc_info.styles.first();
        let (fallback_char_shape_id, fallback_para_shape_id) = match base_style {
            Some(s) => (s.char_shape_id, s.para_shape_id),
            None => (0, 0),
        };
        let char_shape_id = json_i32(json, "baseCharShapeId")
            .filter(|id| *id >= 0)
            .map(|id| id as u16)
            .filter(|id| (*id as usize) < self.core.document.doc_info.char_shapes.len())
            .unwrap_or(fallback_char_shape_id);
        let para_shape_id = json_i32(json, "baseParaShapeId")
            .filter(|id| *id >= 0)
            .map(|id| id as u16)
            .filter(|id| (*id as usize) < self.core.document.doc_info.para_shapes.len())
            .unwrap_or(fallback_para_shape_id);

        let new_style = Style {
            raw_data: None,
            local_name: name,
            english_name,
            style_type,
            next_style_id,
            lang_id: 1042, // 한국어 default (HWP5 spec 표 47)
            para_shape_id,
            char_shape_id,
        };
        self.core.document.doc_info.styles.push(new_style);
        self.core.document.doc_info.raw_stream_dirty = true;
        let new_id = (self.core.document.doc_info.styles.len() - 1) as i32;
        // 스타일 캐시 갱신
        self.core.styles = crate::renderer::style_resolver::resolve_styles(
            &self.core.document.doc_info,
            self.core.dpi,
        );
        new_id
    }

    /// 스타일을 삭제한다.
    ///
    /// 바탕글(ID 0)은 삭제할 수 없다.
    /// 삭제된 스타일을 사용 중인 문단은 바탕글(ID 0)로 변경된다.
    #[wasm_bindgen(js_name = deleteStyle)]
    pub fn delete_style(&mut self, style_id: u32) -> bool {
        if style_id == 0 {
            return false; // 바탕글은 삭제 불가
        }
        let styles = &self.core.document.doc_info.styles;
        if style_id as usize >= styles.len() {
            return false;
        }
        let sid = style_id as u8;
        // 해당 스타일을 사용 중인 문단을 바탕글(0)로 변경
        for section in &mut self.core.document.sections {
            for para in &mut section.paragraphs {
                if para.style_id == sid {
                    para.style_id = 0;
                }
            }
        }
        // 스타일 삭제 (인덱스 기반이므로 뒤의 ID가 변경됨에 주의)
        self.core.document.doc_info.styles.remove(style_id as usize);
        // 삭제된 ID보다 큰 style_id를 가진 문단들 보정
        for section in &mut self.core.document.sections {
            for para in &mut section.paragraphs {
                if para.style_id > sid {
                    para.style_id -= 1;
                }
            }
        }
        // next_style_id 보정
        for s in &mut self.core.document.doc_info.styles {
            if s.next_style_id == sid {
                s.next_style_id = 0;
            } else if s.next_style_id > sid {
                s.next_style_id -= 1;
            }
        }
        // 스타일 캐시 갱신
        self.core.styles = crate::renderer::style_resolver::resolve_styles(
            &self.core.document.doc_info,
            self.core.dpi,
        );
        true
    }

    /// 문서에 정의된 문단 번호(Numbering) 목록을 조회한다.
    ///
    /// 반환값: JSON 배열 [{ id, levelFormats: [...] }, ...]
    /// id는 1-based (ParaShape.numbering_id와 동일)
    #[wasm_bindgen(js_name = getNumberingList)]
    pub fn get_numbering_list(&self) -> String {
        let numberings = &self.core.document.doc_info.numberings;
        let mut items = Vec::new();
        for (i, n) in numberings.iter().enumerate() {
            let formats: Vec<String> = n
                .level_formats
                .iter()
                .map(|f| format!("\"{}\"", f.replace('"', "\\\"")))
                .collect();
            items.push(format!(
                "{{\"id\":{},\"levelFormats\":[{}],\"startNumber\":{}}}",
                i + 1,
                formats.join(","),
                n.start_number
            ));
        }
        format!("[{}]", items.join(","))
    }

    /// 문서에 정의된 글머리표(Bullet) 목록을 조회한다.
    ///
    /// 반환값: JSON 배열 [{ id, char }, ...]
    /// id는 1-based (ParaShape.numbering_id와 동일)
    #[wasm_bindgen(js_name = getBulletList)]
    pub fn get_bullet_list(&self) -> String {
        let bullets = &self.core.document.doc_info.bullets;
        let mut items = Vec::new();
        for (i, b) in bullets.iter().enumerate() {
            let mapped = crate::renderer::layout::map_pua_bullet_char(b.bullet_char);
            let raw_code = b.bullet_char as u32;
            items.push(format!(
                "{{\"id\":{},\"char\":\"{}\",\"rawCode\":{}}}",
                i + 1,
                mapped,
                raw_code
            ));
        }
        format!("[{}]", items.join(","))
    }

    /// 문서에 기본 문단 번호 정의가 없으면 생성한다.
    ///
    /// 반환값: Numbering ID (1-based)
    #[wasm_bindgen(js_name = ensureDefaultNumbering)]
    pub fn ensure_default_numbering(&mut self) -> u16 {
        let numberings = &self.core.document.doc_info.numberings;
        if !numberings.is_empty() {
            return 1; // 이미 있으면 첫 번째 반환
        }
        // 기본 7수준 번호 형식 생성 (한컴 기본 패턴)
        use crate::model::style::{Numbering, NumberingHead};
        let mut n = Numbering::default();
        n.level_formats = [
            "^1.".to_string(), // 1.
            "^2)".to_string(), // 가)
            "^3)".to_string(), // (1)
            "^4)".to_string(), // (가)
            "^5)".to_string(), // ①
            "^6)".to_string(), // ㄱ)
            "^7)".to_string(), // a)
        ];
        n.start_number = 1;
        n.level_start_numbers = [1; 7];
        // 수준별 번호 형식 코드 설정
        n.heads[0] = NumberingHead {
            number_format: 0,
            ..Default::default()
        }; // 1,2,3
        n.heads[1] = NumberingHead {
            number_format: 8,
            ..Default::default()
        }; // 가,나,다
        n.heads[2] = NumberingHead {
            number_format: 0,
            ..Default::default()
        }; // 1,2,3
        n.heads[3] = NumberingHead {
            number_format: 8,
            ..Default::default()
        }; // 가,나,다
        n.heads[4] = NumberingHead {
            number_format: 1,
            ..Default::default()
        }; // ①②③
        n.heads[5] = NumberingHead {
            number_format: 10,
            ..Default::default()
        }; // ㄱ,ㄴ,ㄷ
        n.heads[6] = NumberingHead {
            number_format: 5,
            ..Default::default()
        }; // a,b,c
        self.core.document.doc_info.numberings.push(n);
        1
    }

    /// JSON으로 지정된 번호 형식으로 Numbering 정의를 생성한다.
    ///
    /// json: {"levelFormats":["^1.","^2)",...],"numberFormats":[0,8,...],"startNumber":1}
    /// 반환값: Numbering ID (1-based)
    #[wasm_bindgen(js_name = createNumbering)]
    pub fn create_numbering(&mut self, json: &str) -> u16 {
        use crate::document_core::helpers::json_i32;
        use crate::model::style::{Numbering, NumberingHead};

        let mut n = Numbering::default();

        // levelFormats 배열 파싱
        if let Some(arr_start) = json.find("\"levelFormats\"") {
            let rest = &json[arr_start..];
            if let Some(bracket_start) = rest.find('[') {
                if let Some(bracket_end) = rest[bracket_start..].find(']') {
                    let arr_str = &rest[bracket_start + 1..bracket_start + bracket_end];
                    let mut level = 0;
                    for part in arr_str.split(',') {
                        if level >= 7 {
                            break;
                        }
                        let trimmed = part.trim().trim_matches('"');
                        if !trimmed.is_empty() {
                            n.level_formats[level] = trimmed.to_string();
                            level += 1;
                        }
                    }
                }
            }
        }

        // numberFormats 배열 파싱
        if let Some(arr_start) = json.find("\"numberFormats\"") {
            let rest = &json[arr_start..];
            if let Some(bracket_start) = rest.find('[') {
                if let Some(bracket_end) = rest[bracket_start..].find(']') {
                    let arr_str = &rest[bracket_start + 1..bracket_start + bracket_end];
                    let mut level = 0;
                    for part in arr_str.split(',') {
                        if level >= 7 {
                            break;
                        }
                        if let Ok(code) = part.trim().parse::<u8>() {
                            n.heads[level] = NumberingHead {
                                number_format: code,
                                ..Default::default()
                            };
                            level += 1;
                        }
                    }
                }
            }
        }

        n.start_number = json_i32(json, "startNumber").unwrap_or(1) as u16;
        n.level_start_numbers = [n.start_number as u32; 7];
        self.core.document.doc_info.numberings.push(n);
        self.core.document.doc_info.numberings.len() as u16
    }

    /// 특정 문자의 글머리표 정의가 없으면 생성한다.
    ///
    /// 반환값: Bullet ID (1-based)
    #[wasm_bindgen(js_name = ensureDefaultBullet)]
    pub fn ensure_default_bullet(&mut self, bullet_char_str: &str) -> u16 {
        let bullet_ch = bullet_char_str.chars().next().unwrap_or('●');
        // 이미 해당 문자의 Bullet이 있는지 검색
        let bullets = &self.core.document.doc_info.bullets;
        for (i, b) in bullets.iter().enumerate() {
            let mapped = crate::renderer::layout::map_pua_bullet_char(b.bullet_char);
            if mapped == bullet_ch {
                return (i + 1) as u16;
            }
        }
        // 없으면 새로 생성
        use crate::model::style::Bullet;
        let b = Bullet {
            bullet_char: bullet_ch,
            text_distance: 50,
            ..Default::default()
        };
        self.core.document.doc_info.bullets.push(b);
        self.core.document.doc_info.bullets.len() as u16
    }

    /// 특정 문단의 스타일을 조회한다.
    ///
    /// 반환값: JSON { id, name }
    #[wasm_bindgen(js_name = getStyleAt)]
    pub fn get_style_at(&self, sec_idx: u32, para_idx: u32) -> String {
        let sec = sec_idx as usize;
        let para = para_idx as usize;
        let style_id = self
            .core
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .map(|p| p.style_id as usize)
            .unwrap_or(0);
        let name = self
            .core
            .document
            .doc_info
            .styles
            .get(style_id)
            .map(|s| s.local_name.as_str())
            .unwrap_or("");
        format!(
            "{{\"id\":{},\"name\":\"{}\"}}",
            style_id,
            name.replace('"', "\\\"")
        )
    }

    /// 셀 내부 문단의 스타일을 조회한다.
    #[wasm_bindgen(js_name = getCellStyleAt)]
    pub fn get_cell_style_at(
        &self,
        sec_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
    ) -> String {
        let style_id = self
            .core
            .get_cell_paragraph_ref(
                sec_idx as usize,
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
                cell_para_idx as usize,
            )
            .map(|p| p.style_id as usize)
            .unwrap_or(0);
        let name = self
            .core
            .document
            .doc_info
            .styles
            .get(style_id)
            .map(|s| s.local_name.as_str())
            .unwrap_or("");
        format!(
            "{{\"id\":{},\"name\":\"{}\"}}",
            style_id,
            name.replace('"', "\\\"")
        )
    }

    /// 스타일을 적용한다 (본문 문단).
    #[wasm_bindgen(js_name = applyStyle)]
    pub fn apply_style(
        &mut self,
        sec_idx: u32,
        para_idx: u32,
        style_id: u32,
    ) -> Result<String, JsValue> {
        self.core
            .apply_style_native(sec_idx as usize, para_idx as usize, style_id as usize)
            .map_err(|e| e.into())
    }

    /// 스타일을 적용한다 (셀 내 문단).
    #[wasm_bindgen(js_name = applyCellStyle)]
    pub fn apply_cell_style(
        &mut self,
        sec_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        style_id: u32,
    ) -> Result<String, JsValue> {
        self.core
            .apply_cell_style_native(
                sec_idx as usize,
                parent_para_idx as usize,
                control_idx as usize,
                cell_idx as usize,
                cell_para_idx as usize,
                style_id as usize,
            )
            .map_err(|e| e.into())
    }

    /// 표 셀에서 계산식을 실행한다.
    ///
    /// formula: "=SUM(A1:A5)", "=A1+B2*3" 등
    /// write_result: true이면 결과를 셀에 기록
    #[wasm_bindgen(js_name = evaluateTableFormula)]
    pub fn evaluate_table_formula(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        target_row: u32,
        target_col: u32,
        formula: &str,
        write_result: bool,
    ) -> Result<String, JsValue> {
        self.core
            .evaluate_table_formula(
                section_idx as usize,
                parent_para_idx as usize,
                control_idx as usize,
                target_row as usize,
                target_col as usize,
                formula,
                write_result,
            )
            .map_err(|e| e.into())
    }

    /// `evaluateTableFormula` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, targetRow, targetCol,
    /// formula: string, writeResult? }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = evaluateTableFormulaEx)]
    pub fn evaluate_table_formula_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_str, json_u32};
        self.core
            .evaluate_table_formula(
                json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
                json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
                json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
                json_u32(options_json, "targetRow").unwrap_or(0) as usize,
                json_u32(options_json, "targetCol").unwrap_or(0) as usize,
                &json_str(options_json, "formula").unwrap_or_default(),
                json_bool(options_json, "writeResult").unwrap_or(false),
            )
            .map_err(|e| e.into())
    }

    /// 글꼴 이름으로 font_id를 조회하거나 새로 생성한다.
    ///
    /// 한글(0번) 카테고리에서 이름 검색 → 없으면 7개 전체 카테고리에 신규 등록.
    /// 반환값: font_id (u16), 실패 시 -1
    #[wasm_bindgen(js_name = findOrCreateFontId)]
    pub fn find_or_create_font_id(&mut self, name: &str) -> i32 {
        self.find_or_create_font_id_native(name)
    }

    /// 특정 언어 카테고리에서 글꼴 이름으로 ID를 찾거나 등록한다.
    #[wasm_bindgen(js_name = findOrCreateFontIdForLang)]
    pub fn wasm_find_or_create_font_id_for_lang(&mut self, lang: u32, name: &str) -> i32 {
        self.core
            .find_or_create_font_id_for_lang(lang as usize, name)
    }

    /// 글자 서식을 적용한다 (본문 문단).
    #[wasm_bindgen(js_name = applyCharFormat)]
    pub fn apply_char_format(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_char_format_native(sec_idx, para_idx, start_offset, end_offset, props_json)
            .map_err(|e| e.into())
    }

    /// 글자 서식 ID를 직접 복원한다 (본문 문단).
    #[wasm_bindgen(js_name = setCharShapeId)]
    pub fn set_char_shape_id(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        char_shape_id: u32,
    ) -> Result<String, JsValue> {
        self.set_char_shape_id_native(sec_idx, para_idx, start_offset, end_offset, char_shape_id)
            .map_err(|e| e.into())
    }

    /// 글자 서식을 적용한다 (셀 내 문단).
    #[wasm_bindgen(js_name = applyCharFormatInCell)]
    pub fn apply_char_format_in_cell(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_char_format_in_cell_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            start_offset,
            end_offset,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// `applyCharFormatInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// startOffset, endOffset, props: object }`. `props` 는 글자 서식 JSON 객체(positional
    /// 의 props_json 과 동일). positional 과 동일 동작.
    #[wasm_bindgen(js_name = applyCharFormatInCellEx)]
    pub fn apply_char_format_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_object, json_u32};
        let props_json = json_object(options_json, "props").unwrap_or_else(|| "{}".to_string());
        self.apply_char_format_in_cell_native(
            json_u32(options_json, "secIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endOffset").unwrap_or(0) as usize,
            &props_json,
        )
        .map_err(|e| e.into())
    }

    /// 글자 서식 ID를 직접 복원한다 (셀 내 문단).
    #[wasm_bindgen(js_name = setCharShapeIdInCell)]
    pub fn set_char_shape_id_in_cell(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        start_offset: usize,
        end_offset: usize,
        char_shape_id: u32,
    ) -> Result<String, JsValue> {
        self.set_char_shape_id_in_cell_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            start_offset,
            end_offset,
            char_shape_id,
        )
        .map_err(|e| e.into())
    }

    /// `setCharShapeIdInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ secIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// startOffset, endOffset, charShapeId }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = setCharShapeIdInCellEx)]
    pub fn set_char_shape_id_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.set_char_shape_id_in_cell_native(
            json_u32(options_json, "secIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endOffset").unwrap_or(0) as usize,
            json_u32(options_json, "charShapeId").unwrap_or(0),
        )
        .map_err(|e| e.into())
    }

    /// 감추기 설정
    #[wasm_bindgen(js_name = setPageHide)]
    pub fn set_page_hide(
        &mut self,
        sec: u32,
        para: u32,
        hide_header: bool,
        hide_footer: bool,
        hide_master: bool,
        hide_border: bool,
        hide_fill: bool,
        hide_page_num: bool,
    ) -> Result<String, JsValue> {
        self.set_page_hide_native(
            sec as usize,
            para as usize,
            hide_header,
            hide_footer,
            hide_master,
            hide_border,
            hide_fill,
            hide_page_num,
        )
        .map_err(|e| e.into())
    }

    /// `setPageHide` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sec, para, hideHeader?, hideFooter?, hideMaster?, hideBorder?,
    /// hideFill?, hidePageNum? }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = setPageHideEx)]
    pub fn set_page_hide_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_bool, json_u32};
        self.set_page_hide_native(
            json_u32(options_json, "sec").unwrap_or(0) as usize,
            json_u32(options_json, "para").unwrap_or(0) as usize,
            json_bool(options_json, "hideHeader").unwrap_or(false),
            json_bool(options_json, "hideFooter").unwrap_or(false),
            json_bool(options_json, "hideMaster").unwrap_or(false),
            json_bool(options_json, "hideBorder").unwrap_or(false),
            json_bool(options_json, "hideFill").unwrap_or(false),
            json_bool(options_json, "hidePageNum").unwrap_or(false),
        )
        .map_err(|e| e.into())
    }

    /// 감추기 조회
    #[wasm_bindgen(js_name = getPageHide)]
    pub fn get_page_hide(&self, sec: u32, para: u32) -> Result<String, JsValue> {
        self.get_page_hide_native(sec as usize, para as usize)
            .map_err(|e| e.into())
    }

    /// 문단 서식을 적용한다 (본문 문단).
    /// 문단 번호 시작 방식 설정
    #[wasm_bindgen(js_name = setNumberingRestart)]
    pub fn set_numbering_restart(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        mode: u8,
        start_num: u32,
    ) -> Result<String, JsValue> {
        self.set_numbering_restart_native(section_idx as usize, para_idx as usize, mode, start_num)
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = applyParaFormat)]
    pub fn apply_para_format(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_para_format_native(sec_idx, para_idx, props_json)
            .map_err(|e| e.into())
    }

    /// 문단의 paraShapeId를 직접 설정한다.
    #[wasm_bindgen(js_name = setParaShapeId)]
    pub fn set_para_shape_id(
        &mut self,
        sec_idx: usize,
        para_idx: usize,
        para_shape_id: u16,
    ) -> Result<String, JsValue> {
        self.set_para_shape_id_native(sec_idx, para_idx, para_shape_id)
            .map_err(|e| e.into())
    }

    /// 문단 서식을 적용한다 (셀 내 문단).
    #[wasm_bindgen(js_name = applyParaFormatInCell)]
    pub fn apply_para_format_in_cell(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        props_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_para_format_in_cell_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            props_json,
        )
        .map_err(|e| e.into())
    }

    /// 셀 내 문단의 paraShapeId를 직접 설정한다.
    #[wasm_bindgen(js_name = setCellParaShapeId)]
    pub fn set_cell_para_shape_id(
        &mut self,
        sec_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        para_shape_id: u16,
    ) -> Result<String, JsValue> {
        self.set_cell_para_shape_id_native(
            sec_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            para_shape_id,
        )
        .map_err(|e| e.into())
    }

    // =====================================================================
    // 클립보드 API (WASM 바인딩)
    // =====================================================================

    /// 내부 클립보드에 데이터가 있는지 확인한다.
    #[wasm_bindgen(js_name = hasInternalClipboard)]
    pub fn has_internal_clipboard(&self) -> bool {
        self.has_internal_clipboard_native()
    }

    /// 내부 클립보드의 플레인 텍스트를 반환한다.
    #[wasm_bindgen(js_name = getClipboardText)]
    pub fn get_clipboard_text(&self) -> String {
        self.get_clipboard_text_native()
    }

    /// 내부 클립보드를 초기화한다.
    #[wasm_bindgen(js_name = clearClipboard)]
    pub fn clear_clipboard(&mut self) {
        self.clear_clipboard_native()
    }

    /// 선택 영역을 내부 클립보드에 복사한다.
    ///
    /// 반환값: JSON `{"ok":true,"text":"<plain_text>"}`
    #[wasm_bindgen(js_name = copySelection)]
    pub fn copy_selection(
        &mut self,
        section_idx: u32,
        start_para_idx: u32,
        start_char_offset: u32,
        end_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.copy_selection_native(
            section_idx as usize,
            start_para_idx as usize,
            start_char_offset as usize,
            end_para_idx as usize,
            end_char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 표 셀 내부 선택 영역을 내부 클립보드에 복사한다.
    #[wasm_bindgen(js_name = copySelectionInCell)]
    pub fn copy_selection_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        start_cell_para_idx: u32,
        start_char_offset: u32,
        end_cell_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.copy_selection_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            start_cell_para_idx as usize,
            start_char_offset as usize,
            end_cell_para_idx as usize,
            end_char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// `copySelectionInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
    /// startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = copySelectionInCellEx)]
    pub fn copy_selection_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.copy_selection_in_cell_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCharOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "endCharOffset").unwrap_or(0) as usize,
        )
        .map_err(|e| e.into())
    }

    /// 컨트롤 객체(표, 이미지, 도형)를 내부 클립보드에 복사한다.
    ///
    /// [Task #1161] `cell_path_json` 이 빈 문자열/`"[]"` 면 본문, 그 외에는 셀/글상자
    /// 경로(`[{"controlIndex","cellIndex","cellParaIndex"}, ...]`)의 컨트롤을 복사한다.
    #[wasm_bindgen(js_name = copyControl)]
    pub fn copy_control(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        cell_path_json: &str,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        let cell_path = parse_cell_path_arg(cell_path_json)?;
        self.copy_control_native(
            section_idx as usize,
            para_idx as usize,
            &cell_path,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 내부 클립보드에 컨트롤(표/그림/도형)이 포함되어 있는지 확인한다.
    #[wasm_bindgen(js_name = clipboardHasControl)]
    pub fn clipboard_has_control(&self) -> bool {
        self.clipboard_has_control_native()
    }

    /// 내부 클립보드의 컨트롤 객체를 캐럿 위치에 붙여넣는다.
    ///
    /// 반환값: JSON `{"ok":true,"paraIdx":<idx>,"controlIdx":0}`
    #[wasm_bindgen(js_name = pasteControl)]
    pub fn paste_control(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.paste_control_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 내부 클립보드의 내용을 캐럿 위치에 붙여넣는다 (본문 문단).
    ///
    /// 반환값: JSON `{"ok":true,"paraIdx":<idx>,"charOffset":<offset>}`
    #[wasm_bindgen(js_name = pasteInternal)]
    pub fn paste_internal(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.paste_internal_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 내부 클립보드의 내용을 표 셀 내부에 붙여넣는다.
    ///
    /// 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
    #[wasm_bindgen(js_name = pasteInternalInCell)]
    pub fn paste_internal_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        self.paste_internal_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 내부 클립보드의 내용을 cellPath가 가리키는 중첩 표 셀에 붙여넣는다.
    ///
    /// 반환값: JSON `{"ok":true,"cellParaIdx":<idx>,"charOffset":<offset>}`
    #[wasm_bindgen(js_name = pasteInternalInCellByPath)]
    pub fn paste_internal_in_cell_by_path(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.paste_internal_in_cell_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 선택 영역을 HTML 문자열로 변환한다 (본문).
    #[wasm_bindgen(js_name = exportSelectionHtml)]
    pub fn export_selection_html(
        &self,
        section_idx: u32,
        start_para_idx: u32,
        start_char_offset: u32,
        end_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.export_selection_html_native(
            section_idx as usize,
            start_para_idx as usize,
            start_char_offset as usize,
            end_para_idx as usize,
            end_char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// 선택 영역을 HTML 문자열로 변환한다 (셀 내부).
    #[wasm_bindgen(js_name = exportSelectionInCellHtml)]
    pub fn export_selection_in_cell_html(
        &self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        start_cell_para_idx: u32,
        start_char_offset: u32,
        end_cell_para_idx: u32,
        end_char_offset: u32,
    ) -> Result<String, JsValue> {
        self.export_selection_in_cell_html_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            start_cell_para_idx as usize,
            start_char_offset as usize,
            end_cell_para_idx as usize,
            end_char_offset as usize,
        )
        .map_err(|e| e.into())
    }

    /// `exportSelectionInCellHtml` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, startCellParaIdx,
    /// startCharOffset, endCellParaIdx, endCharOffset }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = exportSelectionInCellHtmlEx)]
    pub fn export_selection_in_cell_html_ex(&self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::json_u32;
        self.export_selection_in_cell_html_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "startCharOffset").unwrap_or(0) as usize,
            json_u32(options_json, "endCellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "endCharOffset").unwrap_or(0) as usize,
        )
        .map_err(|e| e.into())
    }

    /// 컨트롤 객체를 HTML 문자열로 변환한다.
    #[wasm_bindgen(js_name = exportControlHtml)]
    pub fn export_control_html(
        &self,
        section_idx: u32,
        para_idx: u32,
        cell_path_json: &str,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        let cell_path = parse_cell_path_arg(cell_path_json)?;
        self.export_control_html_native(
            section_idx as usize,
            para_idx as usize,
            &cell_path,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 컨트롤의 이미지 바이너리 데이터를 반환한다 (Uint8Array).
    #[wasm_bindgen(js_name = getControlImageData)]
    pub fn get_control_image_data(
        &self,
        section_idx: u32,
        para_idx: u32,
        cell_path_json: &str,
        control_idx: u32,
    ) -> Result<Vec<u8>, JsValue> {
        let cell_path = parse_cell_path_arg(cell_path_json)?;
        self.get_control_image_data_native(
            section_idx as usize,
            para_idx as usize,
            &cell_path,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// 컨트롤의 이미지 MIME 타입을 반환한다.
    #[wasm_bindgen(js_name = getControlImageMime)]
    pub fn get_control_image_mime(
        &self,
        section_idx: u32,
        para_idx: u32,
        cell_path_json: &str,
        control_idx: u32,
    ) -> Result<String, JsValue> {
        let cell_path = parse_cell_path_arg(cell_path_json)?;
        self.get_control_image_mime_native(
            section_idx as usize,
            para_idx as usize,
            &cell_path,
            control_idx as usize,
        )
        .map_err(|e| e.into())
    }

    /// HTML 문자열을 파싱하여 캐럿 위치에 삽입한다 (본문).
    #[wasm_bindgen(js_name = pasteHtml)]
    pub fn paste_html(
        &mut self,
        section_idx: u32,
        para_idx: u32,
        char_offset: u32,
        html: &str,
    ) -> Result<String, JsValue> {
        self.paste_html_native(
            section_idx as usize,
            para_idx as usize,
            char_offset as usize,
            html,
        )
        .map_err(|e| e.into())
    }

    /// HTML 문자열을 파싱하여 셀 내부 캐럿 위치에 삽입한다.
    #[wasm_bindgen(js_name = pasteHtmlInCell)]
    pub fn paste_html_in_cell(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        control_idx: u32,
        cell_idx: u32,
        cell_para_idx: u32,
        char_offset: u32,
        html: &str,
    ) -> Result<String, JsValue> {
        self.paste_html_in_cell_native(
            section_idx as usize,
            parent_para_idx as usize,
            control_idx as usize,
            cell_idx as usize,
            cell_para_idx as usize,
            char_offset as usize,
            html,
        )
        .map_err(|e| e.into())
    }

    /// `pasteHtmlInCell` 의 options object 변형 (#1413).
    ///
    /// options JSON 키: `{ sectionIdx, parentParaIdx, controlIdx, cellIdx, cellParaIdx,
    /// charOffset?, html: string }`. positional 과 동일 동작.
    #[wasm_bindgen(js_name = pasteHtmlInCellEx)]
    pub fn paste_html_in_cell_ex(&mut self, options_json: &str) -> Result<String, JsValue> {
        use crate::document_core::helpers::{json_str, json_u32};
        self.paste_html_in_cell_native(
            json_u32(options_json, "sectionIdx").unwrap_or(0) as usize,
            json_u32(options_json, "parentParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "controlIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellIdx").unwrap_or(0) as usize,
            json_u32(options_json, "cellParaIdx").unwrap_or(0) as usize,
            json_u32(options_json, "charOffset").unwrap_or(0) as usize,
            &json_str(options_json, "html").unwrap_or_default(),
        )
        .map_err(|e| e.into())
    }

    /// HTML 문자열을 파싱하여 cellPath가 가리키는 중첩 표 셀에 삽입한다.
    #[wasm_bindgen(js_name = pasteHtmlInCellByPath)]
    pub fn paste_html_in_cell_by_path(
        &mut self,
        section_idx: u32,
        parent_para_idx: u32,
        path_json: &str,
        char_offset: u32,
        html: &str,
    ) -> Result<String, JsValue> {
        let path = DocumentCore::parse_cell_path(path_json)?;
        self.paste_html_in_cell_by_path_native(
            section_idx as usize,
            parent_para_idx as usize,
            &path,
            char_offset as usize,
            html,
        )
        .map_err(|e| e.into())
    }

    /// 문단별 줄 폭 측정 진단 (WASM)
    #[wasm_bindgen(js_name = measureWidthDiagnostic)]
    pub fn measure_width_diagnostic(
        &self,
        section_idx: u32,
        para_idx: u32,
    ) -> Result<String, JsValue> {
        self.measure_width_diagnostic_native(section_idx as usize, para_idx as usize)
            .map_err(|e| e.into())
    }
}

pub(crate) mod event;

/// WASM 뷰어 컨트롤러 (뷰포트 관리 + 스케줄링)
#[wasm_bindgen]
pub struct HwpViewer {
    /// 문서 참조 (소유)
    document: HwpDocument,
    /// 렌더링 스케줄러
    scheduler: RenderScheduler,
}

#[wasm_bindgen]
impl HwpViewer {
    /// 뷰어 생성
    #[wasm_bindgen(constructor)]
    pub fn new(document: HwpDocument) -> Self {
        let page_count = document.page_count();
        let scheduler = RenderScheduler::new(page_count);
        Self {
            document,
            scheduler,
        }
    }

    /// 뷰포트 업데이트 (스크롤/리사이즈 시 호출)
    #[wasm_bindgen(js_name = updateViewport)]
    pub fn update_viewport(&mut self, scroll_x: f64, scroll_y: f64, width: f64, height: f64) {
        let event = RenderEvent::ViewportChanged(Viewport {
            scroll_x,
            scroll_y,
            width,
            height,
            zoom: self.scheduler_zoom(),
        });
        self.scheduler.on_event(&event);
    }

    /// 줌 변경
    #[wasm_bindgen(js_name = setZoom)]
    pub fn set_zoom(&mut self, zoom: f64) {
        let event = RenderEvent::ZoomChanged(zoom);
        self.scheduler.on_event(&event);
    }

    /// 현재 보이는 페이지 목록 반환
    #[wasm_bindgen(js_name = visiblePages)]
    pub fn visible_pages(&self) -> Vec<u32> {
        self.scheduler.visible_pages()
    }

    /// 대기 중인 렌더링 작업 수
    #[wasm_bindgen(js_name = pendingTaskCount)]
    pub fn pending_task_count(&self) -> u32 {
        self.scheduler.pending_count() as u32
    }

    /// 총 페이지 수
    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> u32 {
        self.document.page_count()
    }

    /// 특정 페이지 SVG 렌더링
    #[wasm_bindgen(js_name = renderPageSvg)]
    pub fn render_page_svg(&self, page_num: u32) -> Result<String, JsValue> {
        self.document.render_page_svg(page_num)
    }

    /// 특정 페이지 HTML 렌더링
    #[wasm_bindgen(js_name = renderPageHtml)]
    pub fn render_page_html(&self, page_num: u32) -> Result<String, JsValue> {
        self.document.render_page_html(page_num)
    }
}

impl HwpViewer {
    fn scheduler_zoom(&self) -> f64 {
        1.0
    }
}

#[wasm_bindgen]
impl HwpDocument {
    // ── 책갈피 API ──

    /// 문서 내 모든 책갈피 목록 반환
    #[wasm_bindgen(js_name = getBookmarks)]
    pub fn get_bookmarks(&self) -> Result<String, JsValue> {
        self.core.get_bookmarks_native().map_err(|e| e.into())
    }

    /// 책갈피 추가
    #[wasm_bindgen(js_name = addBookmark)]
    pub fn add_bookmark(
        &mut self,
        sec: u32,
        para: u32,
        char_offset: u32,
        name: &str,
    ) -> Result<String, JsValue> {
        self.core
            .add_bookmark_native(sec as usize, para as usize, char_offset as usize, name)
            .map_err(|e| e.into())
    }

    /// 책갈피 삭제
    #[wasm_bindgen(js_name = deleteBookmark)]
    pub fn delete_bookmark(
        &mut self,
        sec: u32,
        para: u32,
        ctrl_idx: u32,
    ) -> Result<String, JsValue> {
        self.core
            .delete_bookmark_native(sec as usize, para as usize, ctrl_idx as usize)
            .map_err(|e| e.into())
    }

    /// 책갈피 이름 변경
    #[wasm_bindgen(js_name = renameBookmark)]
    pub fn rename_bookmark(
        &mut self,
        sec: u32,
        para: u32,
        ctrl_idx: u32,
        new_name: &str,
    ) -> Result<String, JsValue> {
        self.core
            .rename_bookmark_native(sec as usize, para as usize, ctrl_idx as usize, new_name)
            .map_err(|e| e.into())
    }
}

// ─── 독립 함수 (문서 로드 없이 사용 가능) ───────────────

/// HWP 파일에서 썸네일 이미지만 경량 추출 (전체 파싱 없이)
///
/// 반환: JSON `{ "format": "png"|"gif", "base64": "...", "width": N, "height": N }`
/// PrvImage가 없으면 `null` 반환
#[wasm_bindgen(js_name = extractThumbnail)]
pub fn extract_thumbnail(data: &[u8]) -> JsValue {
    match crate::parser::extract_thumbnail_only(data) {
        Some(result) => {
            let base64 = base64_encode(&result.data);
            let mime = match result.format.as_str() {
                "png" => "image/png",
                "bmp" => "image/bmp",
                "gif" => "image/gif",
                _ => "application/octet-stream",
            };
            let json = format!(
                r#"{{"format":"{}","base64":"{}","dataUri":"data:{};base64,{}","width":{},"height":{}}}"#,
                result.format, base64, mime, base64, result.width, result.height
            );
            JsValue::from_str(&json)
        }
        None => JsValue::NULL,
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests;
