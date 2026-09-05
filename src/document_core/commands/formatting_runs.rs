//! #6788: 문자 단위 모양 구간 조회와 원자적 문단 복원.
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::{CharShapeRun, Paragraph};
use crate::renderer::composer::{reflow_line_segs, ParagraphBox};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::resolve_styles_for_document;

fn range_error() -> HwpError {
    HwpError::RenderError("글자 모양 구간/범위/ID가 유효하지 않습니다".into())
}

fn validate_range(para: &Paragraph, start: usize, end: usize) -> Result<(), HwpError> {
    if start > end || end > para.char_offsets.len() {
        return Err(range_error());
    }
    Ok(())
}

fn validate_runs(
    para: &Paragraph,
    start: usize,
    end: usize,
    runs: &[CharShapeRun],
    count: usize,
) -> Result<(), HwpError> {
    validate_range(para, start, end)?;
    let mut next = start;
    for run in runs {
        if run.start_offset != next
            || run.end_offset <= next
            || run.end_offset > end
            || run.char_shape_id as usize >= count
        {
            return Err(range_error());
        }
        next = run.end_offset;
    }
    if next != end {
        return Err(range_error());
    }
    Ok(())
}

impl DocumentCore {
    pub fn get_char_shape_runs_native(
        &self,
        sec: usize,
        para: usize,
        start: usize,
        end: usize,
    ) -> Result<String, HwpError> {
        let paragraph = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .ok_or_else(range_error)?;
        validate_range(paragraph, start, end)?;
        serde_json::to_string(&paragraph.char_shape_runs(start, end)).map_err(|_| range_error())
    }

    pub fn get_char_shape_runs_in_cell_by_path_native(
        &mut self,
        sec: usize,
        para: usize,
        path: &[(usize, usize, usize)],
        start: usize,
        end: usize,
    ) -> Result<String, HwpError> {
        let paragraph = self.get_cell_paragraph_mut_by_path(sec, para, path)?;
        validate_range(paragraph, start, end)?;
        serde_json::to_string(&paragraph.char_shape_runs(start, end)).map_err(|_| range_error())
    }

    pub fn set_char_shape_runs_native(
        &mut self,
        sec: usize,
        para: usize,
        start: usize,
        end: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let runs: Vec<CharShapeRun> = serde_json::from_str(json).map_err(|_| range_error())?;
        let paragraph = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .ok_or_else(range_error)?;
        validate_runs(
            paragraph,
            start,
            end,
            &runs,
            self.document.doc_info.char_shapes.len(),
        )?;
        if start == end {
            return Ok("{\"ok\":true}".into());
        }
        let styles = resolve_styles_for_document(&self.document, self.dpi);
        let section = &self.document.sections[sec];
        let columns = Self::find_initial_column_def(&section.paragraphs);
        let layout =
            PageLayoutInfo::from_page_def(&section.section_def.page_def, &columns, self.dpi);
        let width = layout
            .column_areas
            .first()
            .map_or(layout.body_area.width, |a| a.width);
        let available = ParagraphBox::body_for_style(
            width,
            styles.para_styles.get(paragraph.para_shape_id as usize),
            self.dpi,
        );
        let paragraph = &mut self.document.sections[sec].paragraphs[para];
        paragraph.restore_char_shape_runs(start, end, &runs);
        reflow_line_segs(paragraph, available, &styles, self.dpi);
        self.document.sections[sec].raw_stream = None;
        self.rebuild_section_deferred_in_batch(sec);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec,
            para,
            start,
            end,
        });
        Ok("{\"ok\":true}".into())
    }

    pub fn set_char_shape_runs_in_cell_by_path_native(
        &mut self,
        sec: usize,
        para: usize,
        path: &[(usize, usize, usize)],
        start: usize,
        end: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let runs: Vec<CharShapeRun> = serde_json::from_str(json).map_err(|_| range_error())?;
        let count = self.document.doc_info.char_shapes.len();
        let paragraph = self.get_cell_paragraph_mut_by_path(sec, para, path)?;
        validate_runs(paragraph, start, end, &runs, count)?;
        if start == end {
            return Ok("{\"ok\":true}".into());
        }
        paragraph.restore_char_shape_runs(start, end, &runs);
        let &(control, cell, cell_para) = path.last().ok_or_else(range_error)?;
        if path.len() == 1 {
            self.reflow_cell_paragraph(sec, para, control, cell, cell_para);
        } else {
            self.reflow_cell_paragraph_by_path(sec, para, path, cell_para);
        }
        self.mark_cell_control_dirty(sec, para, path[0].0);
        self.document.sections[sec].raw_stream = None;
        self.rebuild_section_deferred_in_batch(sec);
        self.event_log.push(DocumentEvent::CharFormatChanged {
            section: sec,
            para,
            start,
            end,
        });
        Ok("{\"ok\":true}".into())
    }
}
