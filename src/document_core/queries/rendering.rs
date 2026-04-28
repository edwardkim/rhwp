//! 렌더링/페이지 정보/구성/페이지네이션/페이지 트리 관련 native 메서드

use std::cell::RefCell;
use crate::model::document::Section;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::page::ColumnDef;
use crate::renderer::pagination::{Paginator, PaginationResult};
use crate::renderer::height_measurer::{MeasuredTable, MeasuredSection, HeightMeasurer};
use crate::renderer::layer_renderer::LayerRenderer;
use crate::renderer::layout::LayoutEngine;
use crate::renderer::render_tree::PageRenderTree;
use crate::renderer::svg::SvgRenderer;
use crate::renderer::svg_layer::SvgLayerRenderer;
use crate::renderer::html::HtmlRenderer;
use crate::renderer::canvas::CanvasRenderer;
use crate::renderer::style_resolver::resolve_styles;
use crate::renderer::composer::{compose_section, compose_paragraph, ComposedParagraph};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::paint::{LayerBuilder, PageLayerTree, RenderProfile};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use super::super::helpers::color_ref_to_css;

impl DocumentCore {
    /// 페이지 렌더 트리를 생성하여 반환한다 (native bridge / 외부 렌더러용).
    pub fn build_page_render_tree(&self, page_num: u32) -> Result<PageRenderTree, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        Ok(tree)
    }

    /// 페이지 레이어 트리를 생성하여 반환한다 (native bridge / backend replay용).
    pub fn build_page_layer_tree(&self, page_num: u32) -> Result<PageLayerTree, HwpError> {
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut builder = LayerBuilder::new(RenderProfile::Screen);
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
            let style_css = crate::renderer::svg::generate_font_style(
                &renderer, font_paths,
            );
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
        let tree = self.build_page_tree(page_num)?;
        let _overflows = self.layout_engine.take_overflows();
        let mut renderer = CanvasRenderer::new();
        renderer.render_tree(&tree);
        Ok(renderer.command_count() as u32)
    }

    pub fn get_page_layer_tree_native(&self, page_num: u32) -> Result<String, HwpError> {
        Ok(self.build_page_layer_tree(page_num)?.to_json())
    }

    /// 페이지 정보 (네이티브 에러 타입)
    pub fn get_page_info_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::renderer::hwpunit_to_px;
        let (page_content, _, _) = self.find_page(page_num)?;
        let sec_idx = page_content.section_index;
        let page_def = &self.document.sections[sec_idx].section_def.page_def;
        let ml = hwpunit_to_px(page_def.margin_left as i32, self.dpi);
        let mr = hwpunit_to_px(page_def.margin_right as i32, self.dpi);
        let mt = hwpunit_to_px(page_def.margin_top as i32, self.dpi);
        let mb = hwpunit_to_px(page_def.margin_bottom as i32, self.dpi);
        let mh = hwpunit_to_px(page_def.margin_header as i32, self.dpi);
        let mf = hwpunit_to_px(page_def.margin_footer as i32, self.dpi);
        // 단별 영역 정보
        let cols_json: String = page_content.layout.column_areas.iter()
            .map(|ca| format!("{{\"x\":{:.1},\"width\":{:.1}}}", ca.x, ca.width))
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!(
            "{{\"pageIndex\":{},\"width\":{:.1},\"height\":{:.1},\"sectionIndex\":{},\
            \"marginLeft\":{:.1},\"marginRight\":{:.1},\"marginTop\":{:.1},\"marginBottom\":{:.1},\
            \"marginHeader\":{:.1},\"marginFooter\":{:.1},\"columns\":[{}]}}",
            page_content.page_index,
            page_content.layout.page_width,
            page_content.layout.page_height,
            page_content.section_index,
            ml, mr, mt, mb, mh, mf,
            cols_json,
        ))
    }

    /// 구역 정의(SectionDef)를 JSON으로 반환 (네이티브 에러 타입)
    pub fn get_section_def_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
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
        use super::super::helpers::{json_u16, json_u32, json_bool};

        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let sd = &mut section.section_def;

        if let Some(v) = json_u16(json, "pageNum") { sd.page_num = v; }
        if let Some(v) = json_u16(json, "pageNumType") { sd.page_num_type = v as u8; }
        if let Some(v) = json_u16(json, "pictureNum") { sd.picture_num = v; }
        if let Some(v) = json_u16(json, "tableNum") { sd.table_num = v; }
        if let Some(v) = json_u16(json, "equationNum") { sd.equation_num = v; }
        if let Some(v) = json_u16(json, "columnSpacing") { sd.column_spacing = v as i16; }
        if let Some(v) = json_u32(json, "defaultTabSpacing") { sd.default_tab_spacing = v; }
        if let Some(v) = json_bool(json, "hideHeader") { sd.hide_header = v; }
        if let Some(v) = json_bool(json, "hideFooter") { sd.hide_footer = v; }
        if let Some(v) = json_bool(json, "hideMasterPage") { sd.hide_master_page = v; }
        if let Some(v) = json_bool(json, "hideBorder") { sd.hide_border = v; }
        if let Some(v) = json_bool(json, "hideFill") { sd.hide_fill = v; }
        if let Some(v) = json_bool(json, "hideEmptyLine") { sd.hide_empty_line = v; }

        // flags 비트플래그 재구성 (파서와 동일한 비트 위치)
        let flags = &mut sd.flags;
        fn set_bit(flags: &mut u32, mask: u32, val: bool) {
            if val { *flags |= mask; } else { *flags &= !mask; }
        }
        set_bit(flags, 0x0100, sd.hide_header);      // bit 8
        set_bit(flags, 0x0200, sd.hide_footer);       // bit 9
        set_bit(flags, 0x0004, sd.hide_master_page);  // bit 2 (HWP5 스펙, 첫쪽 바탕쪽 감춤)
        set_bit(flags, 0x0800, sd.hide_border);       // bit 11
        set_bit(flags, 0x1000, sd.hide_fill);         // bit 12
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
        self.composed = self.document.sections.iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.paginate();
        self.page_count()
    }

    /// 구역 정의(SectionDef)를 변경하고 재페이지네이션 (네이티브 에러 타입)
    pub fn set_section_def_native(&mut self, section_idx: usize, json: &str) -> Result<String, HwpError> {
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

    /// 구역의 용지 설정(PageDef)을 HWPUNIT 원본값으로 반환 (네이티브 에러 타입)
    pub fn get_page_def_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
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
            pd.width, pd.height,
            pd.margin_left, pd.margin_right, pd.margin_top, pd.margin_bottom,
            pd.margin_header, pd.margin_footer, pd.margin_gutter,
            pd.landscape, binding,
        ))
    }

    /// 구역의 용지 설정(PageDef)을 변경하고 재페이지네이션 (네이티브 에러 타입)
    pub fn set_page_def_native(&mut self, section_idx: usize, json: &str) -> Result<String, HwpError> {
        use crate::model::page::BindingMethod;

        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 {} 범위 초과", section_idx)))?;
        let pd = &mut section.section_def.page_def;

        use super::super::helpers::{json_u32, json_bool};

        if let Some(v) = json_u32(json, "width") { pd.width = v; }
        if let Some(v) = json_u32(json, "height") { pd.height = v; }
        if let Some(v) = json_u32(json, "marginLeft") { pd.margin_left = v; }
        if let Some(v) = json_u32(json, "marginRight") { pd.margin_right = v; }
        if let Some(v) = json_u32(json, "marginTop") { pd.margin_top = v; }
        if let Some(v) = json_u32(json, "marginBottom") { pd.margin_bottom = v; }
        if let Some(v) = json_u32(json, "marginHeader") { pd.margin_header = v; }
        if let Some(v) = json_u32(json, "marginFooter") { pd.margin_footer = v; }
        if let Some(v) = json_u32(json, "marginGutter") { pd.margin_gutter = v; }
        if let Some(v) = json_bool(json, "landscape") { pd.landscape = v; }
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
        self.composed = self.document.sections.iter()
            .map(|s| compose_section(s))
            .collect();
        self.mark_all_sections_dirty();
        self.paginate();

        let page_count = self.page_count();
        Ok(format!("{{\"ok\":true,\"pageCount\":{}}}", page_count))
    }

    /// 텍스트 레이아웃 정보 (네이티브 에러 타입)
    pub fn get_page_text_layout_native(&self, page_num: u32) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        let tree = self.build_page_tree(page_num)?;

        // 렌더 트리에서 TextRun 노드를 재귀적으로 수집
        fn collect_text_runs(node: &RenderNode, runs: &mut Vec<String>) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let positions = compute_char_positions(&text_run.text, &text_run.style);
                let char_x: Vec<String> = positions.iter()
                    .map(|v| format!("{:.1}", v))
                    .collect();

                let escaped_text = super::super::helpers::json_escape(&text_run.text);

                // 문서 좌표 (편집용)
                let doc_coords = match (text_run.section_index, text_run.para_index, text_run.char_start) {
                    (Some(si), Some(pi), Some(cs)) => {
                        format!(",\"secIdx\":{},\"paraIdx\":{},\"charStart\":{}", si, pi, cs)
                    }
                    _ => String::new(),
                };

                // 표 셀 식별 정보 (편집용)
                let cell_coords = if let Some(ref ctx) = text_run.cell_context {
                    let outer = &ctx.path[0];
                    let path_entries: Vec<String> = ctx.path.iter().map(|e| {
                        format!("{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                            e.control_index, e.cell_index, e.cell_para_index)
                    }).collect();
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
                    !matches!(text_run.style.underline, crate::model::style::UnderlineType::None),
                    text_run.style.strikethrough,
                    color_ref_to_css(text_run.style.color),
                );

                // 글자/문단 모양 ID (서식 툴바용)
                let shape_ids = match (text_run.char_shape_id, text_run.para_shape_id) {
                    (Some(csid), Some(psid)) => format!(",\"charShapeId\":{},\"paraShapeId\":{}", csid, psid),
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
            match &node.node_type {
                RenderNodeType::Table(table_node) => {
                    // 문서 좌표
                    let doc_coords = match (table_node.section_index, table_node.para_index, table_node.control_index) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}", si, pi, ci)
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
                        "{{\"type\":\"table\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"rowCount\":{},\"colCount\":{}{},\"cells\":[{}]}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        table_node.row_count, table_node.col_count,
                        doc_coords,
                        cells.join(",")
                    ));
                    // Table 내부도 탐색 (셀 내 수식 등 수집)
                }
                RenderNodeType::Equation(eq_node) => {
                    let doc_coords = match (eq_node.section_index, eq_node.para_index, eq_node.control_index) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}", si, pi, ci)
                        }
                        _ => String::new(),
                    };
                    let cell_coords = match (eq_node.cell_index, eq_node.cell_para_index) {
                        (Some(ci), Some(cpi)) => {
                            format!(",\"cellIdx\":{},\"cellParaIdx\":{}", ci, cpi)
                        }
                        _ => String::new(),
                    };

                    controls.push(format!(
                        "{{\"type\":\"equation\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}{}}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        doc_coords, cell_coords
                    ));
                    return;
                }
                RenderNodeType::Image(image_node) => {
                    let doc_coords = match (image_node.section_index, image_node.para_index, image_node.control_index) {
                        (Some(si), Some(pi), Some(ci)) => {
                            format!(",\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}", si, pi, ci)
                        }
                        _ => String::new(),
                    };

                    controls.push(format!(
                        "{{\"type\":\"image\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}}}",
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                        doc_coords
                    ));
                    return;
                }
                RenderNodeType::Group(group_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (group_node.section_index, group_node.para_index, group_node.control_index) {
                        controls.push(format!(
                            "{{\"type\":\"group\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci
                        ));
                        return; // 자식 개별 수집하지 않음 — 묶음 전체가 하나의 컨트롤
                    }
                }
                RenderNodeType::Rectangle(rect_node) => {
                    // 문서 좌표가 있는 Rectangle만 shape로 수집 (배경 사각형 제외)
                    if let (Some(si), Some(pi), Some(ci)) = (rect_node.section_index, rect_node.para_index, rect_node.control_index) {
                        controls.push(format!(
                            "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci
                        ));
                        return;
                    }
                }
                RenderNodeType::Line(line_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (line_node.section_index, line_node.para_index, line_node.control_index) {
                        controls.push(format!(
                            "{{\"type\":\"line\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"x1\":{:.1},\"y1\":{:.1},\"x2\":{:.1},\"y2\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            line_node.x1, line_node.y1, line_node.x2, line_node.y2,
                            si, pi, ci
                        ));
                        return;
                    }
                }
                RenderNodeType::Ellipse(ell_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (ell_node.section_index, ell_node.para_index, ell_node.control_index) {
                        controls.push(format!(
                            "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                            node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                            si, pi, ci
                        ));
                        return;
                    }
                }
                RenderNodeType::Path(path_node) => {
                    if let (Some(si), Some(pi), Some(ci)) = (path_node.section_index, path_node.para_index, path_node.control_index) {
                        if let Some((x1, y1, x2, y2)) = path_node.connector_endpoints {
                            // 연결선: 선 선택 방식 (시작/끝 좌표 포함)
                            controls.push(format!(
                                "{{\"type\":\"line\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"x1\":{:.1},\"y1\":{:.1},\"x2\":{:.1},\"y2\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                                node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                                x1, y1, x2, y2,
                                si, pi, ci
                            ));
                        } else {
                            controls.push(format!(
                                "{{\"type\":\"shape\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1},\"secIdx\":{},\"paraIdx\":{},\"controlIdx\":{}}}",
                                node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height,
                                si, pi, ci
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
    pub(crate) fn find_column_def_for_paragraph(paragraphs: &[Paragraph], para_idx: usize) -> ColumnDef {
        let mut last_cd = ColumnDef::default();
        for (i, para) in paragraphs.iter().enumerate() {
            if i > para_idx { break; }
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
        if section_idx >= self.dirty_paragraphs.len() { return; }
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
            let bits = self.dirty_paragraphs[section_idx]
                .get_or_insert_with(|| vec![false; para_count]);
            // 기존 비트맵에 삽입 위치 추가 (후속 인덱스 shift)
            if para_idx <= bits.len() {
                bits.insert(para_idx, true);
            }
            // 비트맵 길이를 문단 수에 맞춤
            bits.resize(para_count, false);
            // 분할 원본 문단도 dirty
            if para_idx > 0 { bits[para_idx - 1] = true; }
        }
        // para_offset 누적 (수렴 감지용)
        while self.para_offset.len() <= section_idx { self.para_offset.push(0); }
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
            let bits = self.dirty_paragraphs[section_idx]
                .get_or_insert_with(|| vec![false; para_count]);
            if para_idx < bits.len() {
                bits.remove(para_idx);
            }
            bits.resize(para_count, false);
            // 병합 결과 문단 dirty
            if para_idx < bits.len() { bits[para_idx] = true; }
            if para_idx > 0 && para_idx - 1 < bits.len() { bits[para_idx - 1] = true; }
        }
        // para_offset 누적 (수렴 감지용)
        while self.para_offset.len() <= section_idx { self.para_offset.push(0); }
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
    pub(crate) fn paginate(&mut self) {
        self.invalidate_page_tree_cache();
        let paginator = Paginator::new(self.dpi);
        let measurer = HeightMeasurer::new(self.dpi);

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
            self.pagination.push(PaginationResult { pages: Vec::new(), wrap_around_paras: Vec::new(), hidden_empty_paras: std::collections::HashSet::new() });
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
            self.measured_sections.push(MeasuredSection { paragraphs: Vec::new(), tables: Vec::new() });
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
                            if is_odd { carry_header_odd = page.active_header.clone(); }
                            else { carry_header_even = page.active_header.clone(); }
                        }
                        if page.active_footer.is_some() {
                            if is_odd { carry_footer_odd = page.active_footer.clone(); }
                            else { carry_footer_even = page.active_footer.clone(); }
                        }
                    }
                }
                continue;
            }

            let composed = if idx < self.composed.len() {
                &self.composed[idx]
            } else {
                continue;
            };

            // 증분 측정: 이전 측정 데이터가 있으면 문단/표 수준 선택적 캐싱
            let measured = if !self.measured_sections[idx].paragraphs.is_empty() {
                let dirty_paras = self.dirty_paragraphs.get(idx)
                    .and_then(|opt| opt.as_deref());
                measurer.measure_section_selective(
                    &section.paragraphs,
                    composed,
                    &self.styles,
                    &self.measured_sections[idx],
                    dirty_paras,
                )
            } else {
                measurer.measure_section(&section.paragraphs, composed, &self.styles)
            };

            let column_def = Self::find_initial_column_def(&section.paragraphs);
            // TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback 가능.
            let use_paginator = std::env::var("RHWP_USE_PAGINATOR").map(|v| v == "1").unwrap_or(false);
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
                    },
                )
            } else {
                use crate::renderer::typeset::TypesetEngine;
                let typesetter = TypesetEngine::new(self.dpi);
                typesetter.typeset_section(
                    &section.paragraphs,
                    composed,
                    &self.styles,
                    &section.section_def.page_def,
                    &column_def,
                    idx,
                    &measured.tables,
                    section.section_def.hide_empty_line,
                )
            };

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
                    if converge_page > 0 && converge_page < new_page_count && converge_page < old_page_count {
                        // 검증: full pagination에서 수렴 이후 페이지가 old+offset와 일치하는지 확인
                        let mut verified = true;
                        let check_end = result.pages.len().min(old_page_count);
                        for pi in converge_page..check_end {
                            let new_page = &result.pages[pi];
                            let old_page = &old_result.pages[pi];
                            let new_items: Vec<usize> = new_page.column_contents.iter()
                                .flat_map(|cc| cc.items.iter().map(|it| it.para_index())).collect();
                            let old_items: Vec<usize> = old_page.column_contents.iter()
                                .flat_map(|cc| cc.items.iter().map(|it| (it.para_index() as i64 + offset as i64) as usize)).collect();
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
                            eprintln!("CONVERGENCE: sec{} page {} 수렴 확인 ({}페이지 재사용 가능)",
                                idx, converge_page + 1, old_page_count - converge_page);
                        }
                    }
                }
            }

            // 바탕쪽 선택 (기본 + 확장)
            if !section.section_def.master_pages.is_empty() {
                use crate::model::header_footer::HeaderFooterApply;
                use crate::renderer::pagination::MasterPageRef;

                let mps = &section.section_def.master_pages;
                // 기본 바탕쪽 (비확장 Both/Odd/Even)
                let mp_both = mps.iter().position(|m| m.apply_to == HeaderFooterApply::Both && !m.is_extension);
                let mp_odd = mps.iter().position(|m| m.apply_to == HeaderFooterApply::Odd && !m.is_extension);
                let mp_even = mps.iter().position(|m| m.apply_to == HeaderFooterApply::Even && !m.is_extension);
                // 확장 바탕쪽 (is_extension=true, 마지막 쪽/임의 쪽)
                let ext_mp_indices: Vec<usize> = mps.iter().enumerate()
                    .filter(|(_, m)| m.is_extension)
                    .map(|(i, _)| i)
                    .collect();

                // 구역 내 페이지 수 (마지막 쪽 판별용)
                let section_page_count = result.pages.len();

                for (page_idx_in_section, page) in result.pages.iter_mut().enumerate() {
                    let is_odd = page.page_number % 2 == 1;
                    let is_last = page_idx_in_section + 1 == section_page_count;
                    let is_first_page = page_idx_in_section == 0;

                    // 첫 쪽 감추기: hide_master_page가 true이면 첫 쪽에서 바탕쪽 미적용
                    if is_first_page && section.section_def.hide_master_page {
                        continue;
                    }

                    // 1. 기본 바탕쪽 선택: Odd/Even > Both
                    let selected = if is_odd {
                        mp_odd.or(mp_both)
                    } else {
                        mp_even.or(mp_both)
                    };
                    page.active_master_page = selected.map(|mi| MasterPageRef {
                        section_index: idx,
                        master_page_index: mi,
                    });

                    // 2. 확장 바탕쪽: 마지막 쪽에 적용
                    // 겹치기(overlap): 기존 바탕쪽 위에 추가
                    // 비겹치기: 기존 바탕쪽 대체
                    if is_last && !ext_mp_indices.is_empty() {
                        let overlap_exts: Vec<usize> = ext_mp_indices.iter()
                            .filter(|&&i| mps[i].overlap)
                            .copied().collect();
                        let replace_exts: Vec<usize> = ext_mp_indices.iter()
                            .filter(|&&i| !mps[i].overlap)
                            .copied().collect();

                        // 대체형 확장이 있으면 active를 대체
                        if let Some(&replace_idx) = replace_exts.last() {
                            page.active_master_page = Some(MasterPageRef {
                                section_index: idx,
                                master_page_index: replace_idx,
                            });
                        }
                        // 겹침형 확장은 extra로 추가
                        if !overlap_exts.is_empty() {
                            page.extra_master_pages = overlap_exts.iter()
                                .map(|&mi| MasterPageRef { section_index: idx, master_page_index: mi })
                                .collect();
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
                                let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: idx };
                                match h.apply_to {
                                    HFA::Both => { carry_header_odd = Some(r.clone()); carry_header_even = Some(r); }
                                    HFA::Odd => { carry_header_odd = Some(r); }
                                    HFA::Even => { carry_header_even = Some(r); }
                                }
                            }
                            Control::Footer(f) => {
                                let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: idx };
                                match f.apply_to {
                                    HFA::Both => { carry_footer_odd = Some(r.clone()); carry_footer_even = Some(r); }
                                    HFA::Odd => { carry_footer_odd = Some(r); }
                                    HFA::Even => { carry_footer_even = Some(r); }
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
                use crate::model::control::{Control, AutoNumberType};
                let has_new_number = section.paragraphs.iter().any(|p|
                    p.controls.iter().any(|c| matches!(c, Control::NewNumber(nn) if nn.number_type == AutoNumberType::Page))
                );
                if !has_new_number {
                    for page in &mut result.pages {
                        page.page_number += carry_last_page_number;
                    }
                }
            }

            // 구역 간 머리말/꼬리말 상속 (쪽번호 보정 이후 실행)
            if idx > 0 {
                let section_has_header = result.pages.iter().any(|p| p.active_header.is_some());
                if !section_has_header {
                    for (pi, page) in result.pages.iter_mut().enumerate() {
                        let is_odd = page.page_number % 2 == 1;
                        page.active_header = if is_odd {
                            carry_header_odd.clone().or_else(|| carry_header_even.clone())
                        } else {
                            carry_header_even.clone().or_else(|| carry_header_odd.clone())
                        };
                    }
                }
                let section_has_footer = result.pages.iter().any(|p| p.active_footer.is_some());
                if !section_has_footer {
                    for (pi, page) in result.pages.iter_mut().enumerate() {
                        let is_odd = page.page_number % 2 == 1;
                        page.active_footer = if is_odd {
                            carry_footer_odd.clone().or_else(|| carry_footer_even.clone())
                        } else {
                            carry_footer_even.clone().or_else(|| carry_footer_odd.clone())
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
                                let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: idx };
                                match h.apply_to { HFA::Both => sec_h_both = Some(r), HFA::Even => sec_h_even = Some(r), HFA::Odd => sec_h_odd = Some(r) }
                            }
                            Control::Footer(f) => {
                                let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: idx };
                                match f.apply_to { HFA::Both => sec_f_both = Some(r), HFA::Even => sec_f_even = Some(r), HFA::Odd => sec_f_odd = Some(r) }
                            }
                            _ => {}
                        }
                    }
                }
                let has_hf = sec_h_odd.is_some() || sec_h_even.is_some() || sec_h_both.is_some()
                    || sec_f_odd.is_some() || sec_f_even.is_some() || sec_f_both.is_some();
                // 머리말/꼬리말이 정의된 문단이 시작되는 페이지를 찾아
                // 그 페이지부터만 적용 (정의 이전 페이지에는 미적용)
                use crate::renderer::pagination::PageItem;
                let hdr_start_page = [&sec_h_odd, &sec_h_even, &sec_h_both].iter()
                    .filter_map(|r| r.as_ref())
                    .map(|r| r.para_index)
                    .min()
                    .and_then(|hdr_pi| {
                        result.pages.iter().position(|p| {
                            p.column_contents.iter().any(|cc| {
                                cc.items.iter().any(|item| {
                                    let pi = match item {
                                        PageItem::FullParagraph { para_index } => *para_index,
                                        PageItem::PartialParagraph { para_index, .. } => *para_index,
                                        PageItem::Table { para_index, .. } => *para_index,
                                        PageItem::PartialTable { para_index, .. } => *para_index,
                                        PageItem::Shape { para_index, .. } => *para_index,
                                    };
                                    pi >= hdr_pi
                                })
                            })
                        })
                    })
                    .unwrap_or(0);
                let ftr_start_page = [&sec_f_odd, &sec_f_even, &sec_f_both].iter()
                    .filter_map(|r| r.as_ref())
                    .map(|r| r.para_index)
                    .min()
                    .and_then(|ftr_pi| {
                        result.pages.iter().position(|p| {
                            p.column_contents.iter().any(|cc| {
                                cc.items.iter().any(|item| {
                                    let pi = match item {
                                        PageItem::FullParagraph { para_index } => *para_index,
                                        PageItem::PartialParagraph { para_index, .. } => *para_index,
                                        PageItem::Table { para_index, .. } => *para_index,
                                        PageItem::PartialTable { para_index, .. } => *para_index,
                                        PageItem::Shape { para_index, .. } => *para_index,
                                    };
                                    pi >= ftr_pi
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
                                    if let Some(Control::Header(h)) = para.controls.get(hdr_ref.control_index) {
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
                                    if let Some(Control::Footer(f)) = para.controls.get(ftr_ref.control_index) {
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
    pub(crate) fn find_page(&self, page_num: u32) -> Result<(&crate::renderer::pagination::PageContent, &[crate::model::paragraph::Paragraph], &[ComposedParagraph]), HwpError> {
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
        use crate::renderer::pagination::PageItem;
        use crate::model::control::Control;
        use crate::renderer::hwpunit_to_px;

        let dpi = self.dpi;
        let mut out = String::new();
        let mut global_page = 0u32;

        for (sec_idx, pr) in self.pagination.iter().enumerate() {
            let paragraphs = self.document.sections.get(sec_idx)
                .map(|s| &s.paragraphs[..]).unwrap_or(&[]);
            let measured = self.measured_sections.get(sec_idx);

            for (local_idx, page) in pr.pages.iter().enumerate() {
                if let Some(pf) = page_filter {
                    if global_page != pf {
                        global_page += 1;
                        continue;
                    }
                }

                out.push_str(&format!("\n=== 페이지 {} (global_idx={}, section={}, page_num={}) ===\n",
                    global_page + 1, global_page, sec_idx, page.page_number));

                // 페이지 레이아웃 정보
                let la = &page.layout;
                out.push_str(&format!("  body_area: x={:.1} y={:.1} w={:.1} h={:.1}\n",
                    la.body_area.x, la.body_area.y, la.body_area.width, la.body_area.height));

                for (col_idx, cc) in page.column_contents.iter().enumerate() {
                    let hwp_used_px = compute_hwp_used_height(cc, paragraphs, dpi);
                    let diff_str = match hwp_used_px {
                        Some(hwp) => format!(", hwp_used≈{:.1}px, diff={:+.1}px", hwp, cc.used_height - hwp),
                        None => String::new(),
                    };
                    out.push_str(&format!("  단 {} (items={}, used={:.1}px{}{})\n",
                        col_idx, cc.items.len(), cc.used_height, diff_str,
                        if cc.zone_y_offset > 0.0 { format!(", zone_y_offset={:.1}", cc.zone_y_offset) } else { String::new() }));

                    for item in &cc.items {
                        match item {
                            PageItem::FullParagraph { para_index } => {
                                let text_preview = paragraphs.get(*para_index)
                                    .map(|p| {
                                        let t: String = p.text.chars()
                                            .filter(|c| *c > '\u{001F}')
                                            .take(40).collect();
                                        if t.is_empty() { "(빈)".to_string() } else { t }
                                    })
                                    .unwrap_or_default();
                                let height = measured.and_then(|m| m.get_measured_paragraph(*para_index))
                                    .map(|mp| {
                                        let sb = mp.spacing_before;
                                        let sa = mp.spacing_after;
                                        let lines: f64 = mp.line_heights.iter().sum();
                                        format!("h={:.1} (sb={:.1} lines={:.1} sa={:.1})", sb + lines + sa, sb, lines, sa)
                                    })
                                    .unwrap_or_default();
                                let vpos_info = format_vpos_range(paragraphs.get(*para_index), None, None);
                                out.push_str(&format!("    FullParagraph  pi={}  {}  {}  \"{}\"\n",
                                    para_index, height, vpos_info, text_preview));
                            }
                            PageItem::PartialParagraph { para_index, start_line, end_line } => {
                                let vpos_info = format_vpos_range(paragraphs.get(*para_index), Some(*start_line), Some(*end_line));
                                out.push_str(&format!("    PartialParagraph  pi={}  lines={}..{}  {}\n",
                                    para_index, start_line, end_line, vpos_info));
                            }
                            PageItem::Table { para_index, control_index } => {
                                let table_info = paragraphs.get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| {
                                        if let Control::Table(t) = c {
                                            let h = hwpunit_to_px(t.common.height as i32, dpi);
                                            let w = hwpunit_to_px(t.common.width as i32, dpi);
                                            format!("{}x{}  {:.1}x{:.1}px  wrap={:?} tac={}",
                                                t.row_count, t.col_count, w, h,
                                                t.common.text_wrap, t.common.treat_as_char)
                                        } else { String::new() }
                                    })
                                    .unwrap_or_default();
                                let vpos_info = format_vpos_range(paragraphs.get(*para_index), None, None);
                                out.push_str(&format!("    Table          pi={} ci={}  {}  {}\n",
                                    para_index, control_index, table_info, vpos_info));
                            }
                            PageItem::PartialTable { para_index, control_index, start_row, end_row, is_continuation, .. } => {
                                let table_info = paragraphs.get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| {
                                        if let Control::Table(t) = c {
                                            format!("{}x{}", t.row_count, t.col_count)
                                        } else { String::new() }
                                    })
                                    .unwrap_or_default();
                                let vpos_info = format_vpos_range(paragraphs.get(*para_index), None, None);
                                out.push_str(&format!("    PartialTable   pi={} ci={}  rows={}..{}  cont={}  {}  {}\n",
                                    para_index, control_index, start_row, end_row, is_continuation, table_info, vpos_info));
                            }
                            PageItem::Shape { para_index, control_index } => {
                                let shape_info = paragraphs.get(*para_index)
                                    .and_then(|p| p.controls.get(*control_index))
                                    .map(|c| {
                                        match c {
                                            Control::Shape(s) => format!("wrap={:?} tac={}", s.common().text_wrap, s.common().treat_as_char),
                                            Control::Picture(p) => format!("그림 tac={}", p.common.treat_as_char),
                                            Control::Equation(_) => "수식".to_string(),
                                            _ => String::new(),
                                        }
                                    })
                                    .unwrap_or_default();
                                let vpos_info = format_vpos_range(paragraphs.get(*para_index), None, None);
                                out.push_str(&format!("    Shape          pi={} ci={}  {}  {}\n",
                                    para_index, control_index, shape_info, vpos_info));
                            }
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
        use crate::renderer::pagination::PageItem;
        use crate::model::style::HeadType;
        use crate::renderer::layout::resolve_numbering_id;

        self.layout_engine.set_show_transparent_borders(self.show_transparent_borders);
        self.layout_engine.set_clip_enabled(self.clip_enabled);
        self.layout_engine.set_show_control_codes(self.show_control_codes);
        // 활성 필드 정보를 레이아웃 엔진에 전달 (안내문 숨김용)
        self.layout_engine.set_active_field(
            self.active_field.as_ref().map(|af| (af.section_idx, af.para_idx, af.control_idx, af.cell_path.clone()))
        );
        let (page_content, paragraphs, composed) = self.find_page(page_num)?;
        // 구역의 각주 모양 정보
        let footnote_shape = if page_content.section_index < self.document.sections.len() {
            &self.document.sections[page_content.section_index].section_def.footnote_shape
        } else {
            &crate::model::footnote::FootnoteShape::default()
        };
        // 활성 바탕쪽 조회
        let active_mp = page_content.active_master_page.as_ref().and_then(|mp_ref| {
            self.document.sections.get(mp_ref.section_index)
                .and_then(|s| s.section_def.master_pages.get(mp_ref.master_page_index))
        });
        // 확장 바탕쪽 조회
        let extra_mps: Vec<&crate::model::header_footer::MasterPage> = page_content.extra_master_pages.iter()
            .filter_map(|mp_ref| {
                self.document.sections.get(mp_ref.section_index)
                    .and_then(|s| s.section_def.master_pages.get(mp_ref.master_page_index))
            })
            .collect();
        let sec_measured = self.measured_tables.get(page_content.section_index)
            .map(|v| v.as_slice()).unwrap_or(&[]);
        // 쪽 테두리/배경 조회
        let page_border_fill = self.document.sections.get(page_content.section_index)
            .map(|s| &s.section_def.page_border_fill);
        let outline_num_id = self.document.sections.get(page_content.section_index)
            .map(|s| s.section_def.outline_numbering_id).unwrap_or(0);

        // 번호 상태 리셋 후, 이 페이지 이전의 번호 문단을 재계산하여 카운터 복원
        // (이전 구역 + 현재 구역의 이전 페이지 모두 포함하여 구역 간 번호 연속 지원)
        self.layout_engine.reset_numbering_state();
        let sec_idx = page_content.section_index;
        let sec_page_offset: usize = self.pagination.iter().take(sec_idx).map(|p| p.pages.len()).sum();
        let local_idx = (page_num as usize).saturating_sub(sec_page_offset);
        {
            let mut replayed_partial: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
            // 이전 구역들의 모든 페이지 replay
            for prev_sec in 0..sec_idx {
                if let Some(pr) = self.pagination.get(prev_sec) {
                    let prev_paras = &self.document.sections[prev_sec].paragraphs;
                    let prev_outline_id = self.document.sections[prev_sec].section_def.outline_numbering_id;
                    for pg in &pr.pages {
                        self.replay_numbering_page(
                            pg, prev_paras, prev_outline_id, &mut replayed_partial, prev_sec,
                        );
                    }
                }
            }
            // 현재 구역의 이전 페이지들 replay
            if let Some(pr) = self.pagination.get(sec_idx) {
                for prev_local in 0..local_idx {
                    if let Some(prev_page) = pr.pages.get(prev_local) {
                        self.replay_numbering_page(
                            prev_page, paragraphs, outline_num_id, &mut replayed_partial, sec_idx,
                        );
                    }
                }
            }
        }

        // 머리말/꼬리말이 상속된 경우 원본 구역의 문단을 조회 (source_section_index 기반)
        let header_paragraphs: &[crate::model::paragraph::Paragraph] =
            page_content.active_header.as_ref()
                .and_then(|hf| self.document.sections.get(hf.source_section_index))
                .map(|s| s.paragraphs.as_slice())
                .unwrap_or(paragraphs);
        let footer_paragraphs: &[crate::model::paragraph::Paragraph] =
            page_content.active_footer.as_ref()
                .and_then(|hf| self.document.sections.get(hf.source_section_index))
                .map(|s| s.paragraphs.as_slice())
                .unwrap_or(paragraphs);

        // 머리말/꼬리말 감추기 세트를 레이아웃 엔진에 전달
        self.layout_engine.set_hidden_header_footer(&self.hidden_header_footer);

        // 총 쪽수·파일 이름을 레이아웃 엔진에 전달 (머리말/꼬리말 필드 치환용)
        let total_pages: u32 = self.pagination.iter().map(|p| p.pages.len() as u32).sum();
        self.layout_engine.set_total_pages(total_pages);
        self.layout_engine.set_file_name(&self.file_name);

        let wrap_around_paras = self.pagination.get(sec_idx)
            .map(|pr| pr.wrap_around_paras.as_slice())
            .unwrap_or(&[]);

        // 빈 줄 감추기 문단 집합을 레이아웃 엔진에 전달
        if let Some(pr) = self.pagination.get(sec_idx) {
            self.layout_engine.set_hidden_empty_paras(&pr.hidden_empty_paras);
        }

        let mut tree = self.layout_engine.build_render_tree(
            page_content,
            paragraphs,
            header_paragraphs,
            footer_paragraphs,
            composed,
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
                &mut tree, Some(*ext_mp),
                &page_content.layout, composed, &self.styles,
                &self.document.bin_data_content,
                page_content.section_index, page_content.page_number,
            );
        }
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
        use crate::renderer::pagination::PageItem;
        use crate::model::style::HeadType;
        for col in &page.column_contents {
            for item in &col.items {
                let pi = match item {
                    PageItem::FullParagraph { para_index } |
                    PageItem::PartialParagraph { para_index, .. } |
                    PageItem::Table { para_index, .. } => {
                        if replayed.insert((sec_idx, *para_index)) { Some(*para_index) } else { None }
                    }
                    _ => None,
                };
                if let Some(pi) = pi {
                    if let Some(para) = paras.get(pi) {
                        if let Some(ps) = self.styles.para_styles.get(para.para_shape_id as usize) {
                            if ps.head_type == HeadType::Outline || ps.head_type == HeadType::Number {
                                let nid = crate::renderer::layout::resolve_numbering_id(
                                    ps.head_type, ps.numbering_id, outline_num_id);
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

            let make_row = |cells: &[String]| -> String {
                format!("| {} |", cells.join(" | "))
            };

            let header = make_row(&grid[0]);
            let separator = format!(
                "| {} |",
                std::iter::repeat("---").take(cols).collect::<Vec<_>>().join(" | ")
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
    use crate::renderer::pagination::PageItem;
    use crate::renderer::hwpunit_to_px;

    // 1) 단 항목 내 첫 vpos-reset 검색
    for item in &cc.items {
        let (para_idx, range_start, range_end) = match item {
            PageItem::FullParagraph { para_index } => {
                let p = match paragraphs.get(*para_index) { Some(p) => p, None => continue };
                (*para_index, 0usize, p.line_segs.len())
            }
            PageItem::PartialParagraph { para_index, start_line, end_line } => {
                let p = match paragraphs.get(*para_index) { Some(p) => p, None => continue };
                (*para_index, *start_line, (*end_line).min(p.line_segs.len()))
            }
            _ => continue,
        };
        let p = match paragraphs.get(para_idx) { Some(p) => p, None => continue };
        // line>0 인 줄 중 vertical_pos==0 첫 줄 찾기
        for i in range_start.max(1)..range_end {
            if let Some(seg) = p.line_segs.get(i) {
                if seg.vertical_pos == 0 {
                    if let Some(prev) = p.line_segs.get(i.saturating_sub(1)) {
                        let bottom_hwpu = prev.vertical_pos + prev.line_height;
                        return Some(hwpunit_to_px(bottom_hwpu, dpi));
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
            if p.line_segs.is_empty() { return None; }
            (*para_index, p.line_segs.len() - 1)
        }
        PageItem::PartialParagraph { para_index, end_line, .. } => {
            let p = paragraphs.get(*para_index)?;
            if p.line_segs.is_empty() { return None; }
            (*para_index, end_line.saturating_sub(1).min(p.line_segs.len() - 1))
        }
        _ => return None,
    };
    let p = paragraphs.get(para_idx)?;
    let seg = p.line_segs.get(line_idx)?;
    let bottom_hwpu = seg.vertical_pos + seg.line_height;
    Some(hwpunit_to_px(bottom_hwpu, dpi))
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
    if p.line_segs.is_empty() { return String::new() };
    let total = p.line_segs.len();
    let s = start_line.unwrap_or(0).min(total.saturating_sub(1));
    let e = end_line.unwrap_or(total - 1).min(total - 1);
    if s > e { return String::new() };

    let first = p.line_segs[s].vertical_pos;
    let last = p.line_segs[e].vertical_pos;
    let mut resets: Vec<usize> = Vec::new();
    for i in s..=e {
        // 문단 첫 줄(line 0)의 vpos는 자연 시작점이므로 제외
        if i > 0 && p.line_segs[i].vertical_pos == 0 {
            resets.push(i);
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
    s_out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;
    use crate::renderer::render_tree::RenderNodeType;

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
}
