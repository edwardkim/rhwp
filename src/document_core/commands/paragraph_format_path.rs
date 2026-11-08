//! 중첩 셀의 문단 모양 조회·적용·ID 복원. UI 좌표는 전체 셀 경로로 해석한다.

use super::formatting::{cell_para_shape_affects_vpos, marker_shape_changed};
use crate::document_core::helpers::{
    build_tab_def_from_json, json_has_border_keys, json_has_tab_keys, parse_json_i16_array,
    parse_para_shape_mods,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use crate::renderer::composer::restamp_indentation;

impl DocumentCore {
    /// 본문 문단의 그 컨트롤이 머리말·꼬리말인가. 한 단계 경로의 셀 번호 API는 머리말·꼬리말을
    /// 모르므로, 그 경로는 셀 경로 구현이 맡는다.
    pub(crate) fn is_header_footer_control(
        &self,
        section: usize,
        parent: usize,
        control: usize,
    ) -> bool {
        self.document
            .sections
            .get(section)
            .and_then(|s| s.paragraphs.get(parent))
            .and_then(|p| p.controls.get(control))
            .is_some_and(|c| c.header_footer_paragraphs().is_some())
    }

    pub fn get_cell_para_properties_at_by_path(
        &self,
        section: usize,
        parent: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<String, HwpError> {
        if let [entry] = path {
            if !self.is_header_footer_control(section, parent, entry.0) {
                return self.get_cell_para_properties_at_native(
                    section, parent, entry.0, entry.1, entry.2,
                );
            }
        }
        if path.is_empty() {
            return Err(HwpError::RenderError("빈 셀 경로".into()));
        }
        let paragraph = self.resolve_paragraph_by_path(section, parent, path)?;
        Ok(self.build_para_properties_json(paragraph.para_shape_id, section))
    }

    pub fn apply_para_format_in_cell_by_path(
        &mut self,
        section: usize,
        parent: usize,
        path: &[(usize, usize, usize)],
        props_json: &str,
    ) -> Result<String, HwpError> {
        if let [entry] = path {
            if !self.is_header_footer_control(section, parent, entry.0) {
                return self.apply_para_format_in_cell_native(
                    section, parent, entry.0, entry.1, entry.2, props_json,
                );
            }
        }
        // 경로 검증을 서식표 변경보다 먼저 한다. 잘못된 대상은 문서를 바꾸지 않는다.
        if path.is_empty() {
            return Err(HwpError::RenderError("빈 셀 경로".into()));
        }
        let base_id = self
            .resolve_paragraph_by_path(section, parent, path)?
            .para_shape_id;
        let mut mods = parse_para_shape_mods(props_json);
        if json_has_tab_keys(props_json) {
            let base_tab = self
                .document
                .doc_info
                .para_shapes
                .get(base_id as usize)
                .map_or(0, |shape| shape.tab_def_id);
            let tab =
                build_tab_def_from_json(props_json, base_tab, &self.document.doc_info.tab_defs);
            mods.tab_def_id = Some(self.document.find_or_create_tab_def(tab));
        }
        if json_has_border_keys(props_json) {
            mods.border_fill_id = Some(self.create_border_fill_from_json(props_json));
        }
        if let Some(spacing) = parse_json_i16_array(props_json, "borderSpacing", 4) {
            mods.border_spacing = Some([spacing[0], spacing[1], spacing[2], spacing[3]]);
        }
        let new_id = self.document.find_or_create_para_shape(base_id, &mods);
        self.set_cell_para_shape_id_by_path(section, parent, path, new_id)
    }

    /// Undo/Redo는 UI용 px 값을 재적용하지 않고 기존 문단 모양 ID를 복원한다.
    pub fn set_cell_para_shape_id_by_path(
        &mut self,
        section: usize,
        parent: usize,
        path: &[(usize, usize, usize)],
        shape_id: u16,
    ) -> Result<String, HwpError> {
        if let [entry] = path {
            if !self.is_header_footer_control(section, parent, entry.0) {
                return self.set_cell_para_shape_id_native(
                    section, parent, entry.0, entry.1, entry.2, shape_id,
                );
            }
        }
        let Some(&(outer_control, _, _)) = path.first() else {
            return Err(HwpError::RenderError("빈 셀 경로".into()));
        };
        let new_shape = self
            .document
            .doc_info
            .para_shapes
            .get(shape_id as usize)
            .ok_or_else(|| HwpError::RenderError(format!("문단 모양 ID {shape_id} 범위 초과")))?;
        let old_id = self
            .resolve_paragraph_by_path(section, parent, path)?
            .para_shape_id;
        let affects_vpos = self
            .document
            .doc_info
            .para_shapes
            .get(old_id as usize)
            .is_none_or(|old| cell_para_shape_affects_vpos(old, new_shape));
        let marker_changed = self
            .document
            .doc_info
            .para_shapes
            .get(old_id as usize)
            .is_none_or(|old| marker_shape_changed(old, new_shape));
        let (old_indent, new_indent) = self.para_shape_indents(old_id, shape_id);
        let inner_para = path.last().unwrap().2;
        // 머리말·꼬리말 문단은 셀 첫 문단의 저장 줄 원점을 쓰지 않는다. 머리말 문단 모양 바꾸기
        // (`apply_para_format_in_hf_native`)처럼 그 문단만 다시 짠다.
        let header_footer = path.len() == 1;
        if !header_footer {
            self.restamp_cell_format_origin_by_path(section, parent, path, old_id, shape_id)?;
        }
        let paragraph = self.get_cell_paragraph_mut_by_path(section, parent, path)?;
        paragraph.para_shape_id = shape_id;
        restamp_indentation(&mut paragraph.line_segs, old_indent, new_indent);
        if marker_changed {
            paragraph.invalidate_layout_inputs();
        }
        if header_footer {
            self.reflow_cell_paragraph_by_path(section, parent, path, inner_para);
        } else {
            paragraph.cell_format_vpos_dirty |= affects_vpos;
            if affects_vpos {
                self.reflow_cell_paragraph_by_path(section, parent, path, inner_para);
            }
            self.pending_cell_format_vpos |= affects_vpos;
        }
        self.mark_cell_control_dirty(section, parent, outer_control);
        self.document.sections[section].raw_stream = None;
        self.rebuild_section_deferred_in_batch(section);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section,
            para: parent,
        });
        Ok("{\"ok\":true}".into())
    }
}
