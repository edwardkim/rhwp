//! 표/셀 CRUD + 속성 조회·수정 관련 native 메서드

use super::super::helpers::{
    border_line_type_to_u8_val, color_ref_to_css, json_u32, navigate_path_to_table,
};
use crate::document_core::{DocumentCore, TableTransposeClipboard};
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::path::{path_from_flat, PathSegment};
use crate::model::shape::common_obj_offsets;

impl DocumentCore {
    pub(crate) fn get_table_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&mut crate::model::table::Table, HwpError> {
        let path = path_from_flat(parent_para_idx, control_idx);
        self.get_table_by_path(section_idx, &path)
    }

    /// DocumentPath를 사용하여 임의 깊이의 표에 대한 가변 참조를 얻는다.
    pub(crate) fn get_table_by_path(
        &mut self,
        section_idx: usize,
        path: &[PathSegment],
    ) -> Result<&mut crate::model::table::Table, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        navigate_path_to_table(&mut section.paragraphs, path)
    }

    /// 표에 행을 삽입한다 (네이티브).
    pub fn insert_table_row_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        row_idx: u16,
        below: bool,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .insert_row(row_idx, below)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let row_count = table.row_count;
        let col_count = table.col_count;

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableRowInserted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"rowCount\":{},\"colCount\":{}",
            row_count, col_count
        )))
    }

    /// 표에 열을 삽입한다 (네이티브).
    pub fn insert_table_column_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        col_idx: u16,
        right: bool,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .insert_column(col_idx, right)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let row_count = table.row_count;
        let col_count = table.col_count;

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableColumnInserted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"rowCount\":{},\"colCount\":{}",
            row_count, col_count
        )))
    }

    /// 표에서 행을 삭제한다 (네이티브).
    pub fn delete_table_row_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        row_idx: u16,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .delete_row(row_idx)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let row_count = table.row_count;
        let col_count = table.col_count;

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableRowDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"rowCount\":{},\"colCount\":{}",
            row_count, col_count
        )))
    }

    /// 표에서 열을 삭제한다 (네이티브).
    pub fn delete_table_column_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        col_idx: u16,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .delete_column(col_idx)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let row_count = table.row_count;
        let col_count = table.col_count;

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableColumnDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"rowCount\":{},\"colCount\":{}",
            row_count, col_count
        )))
    }

    /// 표 셀을 병합한다 (네이티브).
    pub fn merge_table_cells_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .merge_cells(start_row, start_col, end_row, end_col)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let cell_count = table.cells.len();

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellsMerged {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellCount\":{}",
            cell_count
        )))
    }

    pub fn split_table_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        row: u16,
        col: u16,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .split_cell(row, col)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let cell_count = table.cells.len();

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellSplit {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellCount\":{}",
            cell_count
        )))
    }

    /// 셀을 N줄 × M칸으로 분할한다 (네이티브).
    pub fn split_table_cell_into_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        row: u16,
        col: u16,
        n_rows: u16,
        m_cols: u16,
        equal_row_height: bool,
        merge_first: bool,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .split_cell_into(row, col, n_rows, m_cols, equal_row_height, merge_first)
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let cell_count = table.cells.len();

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellSplit {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellCount\":{}",
            cell_count
        )))
    }

    /// 범위 내 셀들을 각각 N줄 × M칸으로 분할한다 (네이티브).
    pub fn split_table_cells_in_range_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
        n_rows: u16,
        m_cols: u16,
        equal_row_height: bool,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .split_cells_in_range(
                start_row,
                start_col,
                end_row,
                end_col,
                n_rows,
                m_cols,
                equal_row_height,
            )
            .map_err(|e| HwpError::RenderError(e))?;
        table.dirty = true;
        let cell_count = table.cells.len();

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::CellSplit {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"cellCount\":{}",
            cell_count
        )))
    }

    /// 선택된 셀 범위를 행/열 바꿈 복사용 내부 버퍼에 저장한다.
    pub fn copy_table_cells_transposed_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> Result<String, HwpError> {
        let data = {
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            table
                .copy_transpose_range(start_row, start_col, end_row, end_col)
                .map_err(HwpError::RenderError)?
        };
        let source_rows = data.source_rows;
        let source_cols = data.source_cols;
        self.table_transpose_clipboard = Some(TableTransposeClipboard { data });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"sourceRows\":{},\"sourceCols\":{},\"targetRows\":{},\"targetCols\":{}",
            source_rows, source_cols, source_cols, source_rows
        )))
    }

    /// 행/열 바꿈 복사 버퍼를 대상 시작 셀부터 정적 붙여넣기한다.
    pub fn paste_table_cells_transposed_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        start_row: u16,
        start_col: u16,
    ) -> Result<String, HwpError> {
        let data = self
            .table_transpose_clipboard
            .as_ref()
            .ok_or_else(|| HwpError::RenderError("행/열 바꿈 복사 데이터가 없습니다".to_string()))?
            .data
            .clone();

        let source_rows = data.source_rows;
        let source_cols = data.source_cols;
        let changed_cells = {
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            table
                .paste_transposed_cells(start_row, start_col, &data)
                .map_err(HwpError::RenderError)?
        };

        self.document.sections[section_idx].raw_stream = None;
        for (cell_idx, para_count) in changed_cells {
            for cell_para_idx in 0..para_count {
                self.reflow_cell_paragraph(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                );
            }
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableCellsTransposed {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"sourceRows\":{},\"sourceCols\":{},\"targetRows\":{},\"targetCols\":{}",
            source_rows, source_cols, source_cols, source_rows
        )))
    }

    /// 선택된 전체 표의 행/열을 제자리에서 바꾼다.
    pub fn transpose_table_cells_in_place_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let (source_rows, source_cols, changed_cells) = {
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            let source_rows = table.row_count;
            let source_cols = table.col_count;
            let changed_cells = table
                .transpose_unmerged_table_in_place()
                .map_err(HwpError::RenderError)?;
            (source_rows, source_cols, changed_cells)
        };

        self.document.sections[section_idx].raw_stream = None;
        for (cell_idx, para_count) in changed_cells {
            for cell_para_idx in 0..para_count {
                self.reflow_cell_paragraph(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                );
            }
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableCellsTransposed {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"sourceRows\":{},\"sourceCols\":{},\"targetRows\":{},\"targetCols\":{}",
            source_rows, source_cols, source_cols, source_rows
        )))
    }

    /// 행/열 바꿈 복사 버퍼를 커서 위치에 새 표로 생성해 붙여넣는다.
    pub fn paste_table_cells_transposed_as_new_table_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let data = self
            .table_transpose_clipboard
            .as_ref()
            .ok_or_else(|| HwpError::RenderError("행/열 바꿈 복사 데이터가 없습니다".to_string()))?
            .data
            .clone();

        let source_rows = data.source_rows;
        let source_cols = data.source_cols;
        let target_rows = source_cols;
        let target_cols = source_rows;
        if target_rows == 0 || target_cols == 0 {
            return Err(HwpError::RenderError(
                "행/열 바꿈 복사 데이터가 비어 있습니다".to_string(),
            ));
        }

        let create_json =
            self.create_table_native(section_idx, para_idx, char_offset, target_rows, target_cols)?;
        let table_para_idx = json_u32(&create_json, "paraIdx")
            .ok_or_else(|| HwpError::RenderError("표 생성 결과 paraIdx 누락".to_string()))?
            as usize;
        let table_control_idx = json_u32(&create_json, "controlIdx")
            .ok_or_else(|| HwpError::RenderError("표 생성 결과 controlIdx 누락".to_string()))?
            as usize;

        self.paste_table_cells_transposed_native(
            section_idx,
            table_para_idx,
            table_control_idx,
            0,
            0,
        )?;

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"sourceRows\":{},\"sourceCols\":{},\"targetRows\":{},\"targetCols\":{}",
            table_para_idx, table_control_idx, source_rows, source_cols, target_rows, target_cols
        )))
    }

    /// 행/열 바꿈 복사 버퍼 보유 여부.
    pub fn has_table_transpose_clipboard_native(&self) -> bool {
        self.table_transpose_clipboard.is_some()
    }

    pub(crate) fn get_table_dimensions_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;

        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                ))
            }
        };

        Ok(format!(
            "{{\"rowCount\":{},\"colCount\":{},\"cellCount\":{}}}",
            table.row_count,
            table.col_count,
            table.cells.len()
        ))
    }

    /// 표 셀의 행/열/병합 정보를 반환한다 (네이티브).
    pub(crate) fn get_cell_info_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;

        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                ))
            }
        };

        let cell = table.cells.get(cell_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "셀 인덱스 {} 범위 초과 (총 {}개)",
                cell_idx,
                table.cells.len()
            ))
        })?;

        Ok(format!(
            "{{\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{}}}",
            cell.row, cell.col, cell.row_span, cell.col_span
        ))
    }

    /// 셀 속성을 조회한다 (네이티브).
    /// border_fill_id로 BorderFill을 조회하여 JSON 부분 문자열을 생성한다.
    /// 반환 형식: "borderFillId":N,"borderLeft":{...},...,"fillType":"...","fillColor":"..."
    pub(crate) fn build_border_fill_json_by_id(&self, bf_id: u16) -> String {
        if bf_id == 0 {
            return concat!(
                "\"borderFillId\":0,",
                "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0,",
                "\"diagonalLine\":0,\"diagonalSlash\":0,\"diagonalBackSlash\":0,",
                "\"diagonalWidth\":0,\"diagonalColor\":\"#000000\",\"centerLine\":\"NONE\""
            ).to_string();
        }
        let bf = self
            .document
            .doc_info
            .border_fills
            .get((bf_id - 1) as usize);
        match bf {
            Some(bf) => {
                use crate::model::style::{CenterLine, FillType};
                let dir_names = ["Left", "Right", "Top", "Bottom"];
                let borders_json: Vec<String> = bf.borders.iter().enumerate().map(|(i, b)| {
                    format!(
                        "\"border{}\":{{\"type\":{},\"width\":{},\"color\":\"{}\"}}",
                        dir_names[i],
                        border_line_type_to_u8_val(b.line_type),
                        b.width,
                        color_ref_to_css(b.color),
                    )
                }).collect();
                let (fill_type_str, fill_color, pat_color, pat_type) = match &bf.fill.solid {
                    Some(sf) if bf.fill.fill_type == FillType::Solid => {
                        ("solid", color_ref_to_css(sf.background_color),
                         color_ref_to_css(sf.pattern_color), sf.pattern_type)
                    }
                    _ => ("none", "#ffffff".to_string(), "#000000".to_string(), 0),
                };
                let mut diagonal_slash = (bf.attr >> 2) & 0x07;
                let mut diagonal_backslash = (bf.attr >> 5) & 0x07;
                let mut center_line = if bf.center_line != CenterLine::None {
                    bf.center_line
                } else {
                    CenterLine::from_hwp_attr(bf.attr)
                };
                if center_line != CenterLine::None {
                    diagonal_slash = 0;
                    diagonal_backslash = 0;
                } else if diagonal_slash != 0 || diagonal_backslash != 0 {
                    center_line = CenterLine::None;
                }
                format!(
                    "\"borderFillId\":{},{},\"fillType\":\"{}\",\"fillColor\":\"{}\",\"patternColor\":\"{}\",\"patternType\":{},\"diagonalLine\":{},\"diagonalSlash\":{},\"diagonalBackSlash\":{},\"diagonalWidth\":{},\"diagonalColor\":\"{}\",\"centerLine\":\"{}\"",
                    bf_id,
                    borders_json.join(","),
                    fill_type_str, fill_color, pat_color, pat_type,
                    bf.diagonal.diagonal_type,
                    diagonal_slash,
                    diagonal_backslash,
                    bf.diagonal.width,
                    color_ref_to_css(bf.diagonal.color),
                    center_line.as_hwpx(),
                )
            }
            None => {
                concat!(
                    "\"borderFillId\":0,",
                    "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0,",
                    "\"diagonalLine\":0,\"diagonalSlash\":0,\"diagonalBackSlash\":0,",
                    "\"diagonalWidth\":0,\"diagonalColor\":\"#000000\",\"centerLine\":\"NONE\""
                ).to_string()
            }
        }
    }

    /// UI 조회에서는 셀 고유 값보다 cellzone overlay가 실제 표시 상태에 가깝다.
    fn cell_effective_border_fill_id(
        table: &crate::model::table::Table,
        cell_idx: usize,
    ) -> Option<u16> {
        let cell = table.cells.get(cell_idx)?;
        let row = cell.row;
        let col = cell.col;
        let zone_border_fill_id = table
            .zones
            .iter()
            .rev()
            .find(|zone| {
                zone.border_fill_id > 0
                    && zone.start_row <= row
                    && row <= zone.end_row
                    && zone.start_col <= col
                    && col <= zone.end_col
            })
            .map(|zone| zone.border_fill_id);

        Some(zone_border_fill_id.unwrap_or(cell.border_fill_id))
    }

    pub(crate) fn get_cell_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<String, HwpError> {
        self.get_cell_properties_with_border_mode(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            true,
        )
    }

    pub(crate) fn get_cell_own_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<String, HwpError> {
        self.get_cell_properties_with_border_mode(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            false,
        )
    }

    fn get_cell_properties_with_border_mode(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        use_effective_border_fill: bool,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;

        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                ))
            }
        };

        let cell = table
            .cells
            .get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx)))?;

        let va = match cell.vertical_align {
            crate::model::table::VerticalAlign::Top => 0,
            crate::model::table::VerticalAlign::Center => 1,
            crate::model::table::VerticalAlign::Bottom => 2,
        };

        let border_fill_id = if use_effective_border_fill {
            Self::cell_effective_border_fill_id(table, cell_idx).unwrap_or(cell.border_fill_id)
        } else {
            cell.border_fill_id
        };
        let bf_json = self.build_border_fill_json_by_id(border_fill_id);

        Ok(format!(
            "{{\"width\":{},\"height\":{},\"paddingLeft\":{},\"paddingRight\":{},\"paddingTop\":{},\"paddingBottom\":{},\"applyInnerMargin\":{},\"verticalAlign\":{},\"textDirection\":{},\"isHeader\":{},\"cellProtect\":{},\"fieldName\":{},\"editableInForm\":{},{}}}",
            cell.width, cell.height,
            cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
            cell.apply_inner_margin,
            va, cell.text_direction, cell.is_header, cell.cell_protect(),
            json_escape(cell.field_name.as_deref().unwrap_or("")),
            cell.editable_in_form(),
            bf_json,
        ))
    }

    /// 셀 속성을 수정한다 (네이티브).
    pub(crate) fn set_cell_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).unwrap_or(serde_json::Value::Null);
        let obj = parsed.as_object();
        let top_u32 = |key: &str| -> Option<u32> {
            obj.and_then(|m| m.get(key))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
        };
        let top_u8 = |key: &str| -> Option<u8> { top_u32(key).map(|v| v as u8) };
        let top_i16 = |key: &str| -> Option<i16> {
            obj.and_then(|m| m.get(key))
                .and_then(|v| v.as_i64())
                .map(|v| v as i16)
        };
        let top_bool =
            |key: &str| -> Option<bool> { obj.and_then(|m| m.get(key)).and_then(|v| v.as_bool()) };
        let top_str = |key: &str| -> Option<String> {
            obj.and_then(|m| m.get(key))
                .and_then(|v| v.as_str())
                .map(ToOwned::to_owned)
        };

        let has_border_fill_change = json.contains("\"borderLeft\"")
            || json.contains("\"fillType\"")
            || json.contains("\"diagonalLine\"")
            || json.contains("\"diagonalSlash\"")
            || json.contains("\"diagonalBackSlash\"")
            || json.contains("\"diagonalWidth\"")
            || json.contains("\"diagonalColor\"")
            || json.contains("\"centerLine\"");
        let cell_border_fill_json = if has_border_fill_change {
            Some(self.normalize_cell_border_fill_json_for_edit(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                json,
            ))
        } else {
            None
        };

        let (needs_reflow, reflow_para_count) = {
            let mut needs_reflow = false;
            let mut size_changed = false;
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            let direct_border_fill_id = if has_border_fill_change {
                None
            } else {
                top_u32("borderFillId").map(|v| v as u16).and_then(|bf_id| {
                    table.cells.get(cell_idx).and_then(|cell| {
                        if Self::cell_is_covered_by_zone_border_fill(table, cell, bf_id) {
                            None
                        } else {
                            Some(bf_id)
                        }
                    })
                })
            };
            let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
                HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx))
            })?;

            if let Some(v) = top_u32("width") {
                needs_reflow |= cell.width != v;
                size_changed |= cell.width != v;
                cell.width = v;
            }
            if let Some(v) = top_u32("height") {
                size_changed |= cell.height != v;
                cell.height = v;
            }
            if let Some(v) = top_i16("paddingLeft") {
                needs_reflow |= cell.padding.left != v;
                cell.padding.left = v;
            }
            if let Some(v) = top_i16("paddingRight") {
                needs_reflow |= cell.padding.right != v;
                cell.padding.right = v;
            }
            if let Some(v) = top_i16("paddingTop") {
                cell.padding.top = v;
            }
            if let Some(v) = top_i16("paddingBottom") {
                cell.padding.bottom = v;
            }
            if let Some(v) = top_bool("applyInnerMargin") {
                needs_reflow |= cell.apply_inner_margin != v;
                cell.set_apply_inner_margin(v);
            }
            if let Some(v) = top_u8("verticalAlign") {
                cell.vertical_align = match v {
                    1 => crate::model::table::VerticalAlign::Center,
                    2 => crate::model::table::VerticalAlign::Bottom,
                    _ => crate::model::table::VerticalAlign::Top,
                };
            }
            if let Some(v) = top_u8("textDirection") {
                cell.text_direction = v;
            }
            if let Some(v) = top_bool("isHeader") {
                cell.set_header(v);
            }
            if let Some(v) = top_bool("cellProtect") {
                cell.set_cell_protect(v);
            }
            if let Some(v) = top_bool("editableInForm") {
                cell.set_editable_in_form(v);
            }
            if let Some(v) = top_str("fieldName") {
                cell.field_name = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = direct_border_fill_id {
                cell.border_fill_id = v;
            }
            if size_changed {
                table.update_ctrl_dimensions();
            }
            table.dirty = true;
            (needs_reflow, table.cells[cell_idx].paragraphs.len())
        };

        if needs_reflow {
            let para_count = reflow_para_count;
            for cell_para_idx in 0..para_count {
                self.reflow_cell_paragraph(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                );
            }
        }

        if has_border_fill_change {
            let border_fill_json = cell_border_fill_json.as_deref().unwrap_or(json);
            let new_bf_id = self.create_border_fill_from_json(border_fill_json);
            let new_bf_has_cell_diagonal = self
                .document
                .doc_info
                .border_fills
                .get((new_bf_id as usize).saturating_sub(1))
                .is_some_and(Self::border_fill_has_cell_diagonal);
            let cell_diagonal_bf_ids: Vec<u16> = self
                .document
                .doc_info
                .border_fills
                .iter()
                .enumerate()
                .filter_map(|(idx, bf)| {
                    Self::border_fill_has_cell_diagonal(bf).then_some((idx + 1) as u16)
                })
                .collect();

            // 새 BorderFill의 테두리 데이터 복사 (이웃 셀 갱신용)
            let new_borders = {
                let bf_idx = (new_bf_id as usize).saturating_sub(1);
                self.document
                    .doc_info
                    .border_fills
                    .get(bf_idx)
                    .map(|bf| bf.borders)
                    .unwrap_or_default()
            };

            // 대상 셀 정보 추출 + border_fill_id 변경
            let (target_row, target_col, target_col_span, target_row_span) = {
                let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
                let (row, col, col_span, row_span) = {
                    let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx))
                    })?;
                    cell.border_fill_id = new_bf_id;
                    (cell.row, cell.col, cell.col_span, cell.row_span)
                };
                Self::sync_cellzone_origin_cell_diagonal_override(
                    table,
                    row,
                    col,
                    new_bf_id,
                    new_bf_has_cell_diagonal,
                    &cell_diagonal_bf_ids,
                );
                (
                    row as usize,
                    col as usize,
                    col_span as usize,
                    row_span as usize,
                )
            };

            // 이웃 셀의 공유 엣지 테두리를 갱신
            // borders 배열: [좌(0), 우(1), 상(2), 하(3)]
            self.update_neighbor_borders(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                target_row,
                target_col,
                target_col_span,
                target_row_span,
                &new_borders,
            );
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok("{\"ok\":true}".to_string())
    }

    fn border_fill_has_cell_diagonal(bf: &crate::model::style::BorderFill) -> bool {
        let center_line = if bf.center_line != crate::model::style::CenterLine::None {
            bf.center_line
        } else {
            crate::model::style::CenterLine::from_hwp_attr(bf.attr)
        };
        if center_line != crate::model::style::CenterLine::None {
            return false;
        }

        let slash = (bf.attr >> 2) & 0x07;
        let backslash = (bf.attr >> 5) & 0x07;
        bf.diagonal.diagonal_type != 0 && (slash != 0 || backslash != 0)
    }

    fn sync_cellzone_origin_cell_diagonal_override(
        table: &mut crate::model::table::Table,
        row: u16,
        col: u16,
        new_bf_id: u16,
        new_bf_has_cell_diagonal: bool,
        cell_diagonal_bf_ids: &[u16],
    ) {
        let has_large_origin_diagonal_zone = table.zones.iter().any(|zone| {
            zone.start_row == row
                && zone.start_col == col
                && (zone.end_row > row || zone.end_col > col)
                && cell_diagonal_bf_ids.contains(&zone.border_fill_id)
        });
        if !has_large_origin_diagonal_zone {
            return;
        }

        if new_bf_has_cell_diagonal {
            if let Some(zone) = table.zones.iter_mut().find(|zone| {
                zone.start_row == row
                    && zone.start_col == col
                    && zone.end_row == row
                    && zone.end_col == col
            }) {
                zone.border_fill_id = new_bf_id;
            } else {
                table.zones.push(crate::model::table::TableZone {
                    start_col: col,
                    start_row: row,
                    end_col: col,
                    end_row: row,
                    border_fill_id: new_bf_id,
                });
            }
        } else {
            table.zones.retain(|zone| {
                !(zone.start_row == row
                    && zone.start_col == col
                    && zone.end_row == row
                    && zone.end_col == col
                    && cell_diagonal_bf_ids.contains(&zone.border_fill_id))
            });
        }
    }

    fn normalize_cell_border_fill_json_for_edit(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        json: &str,
    ) -> String {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(json) else {
            return json.to_string();
        };
        let Some(obj) = value.as_object_mut() else {
            return json.to_string();
        };
        let incoming_bf_id = obj
            .get("borderFillId")
            .and_then(|v| v.as_u64())
            .map(|v| v as u16)
            .unwrap_or(0);
        if incoming_bf_id == 0 {
            return json.to_string();
        }

        let Some(table) = self
            .document
            .sections
            .get(section_idx)
            .and_then(|section| section.paragraphs.get(parent_para_idx))
            .and_then(|para| match para.controls.get(control_idx) {
                Some(Control::Table(table)) => Some(table),
                _ => None,
            })
        else {
            return json.to_string();
        };
        let Some(cell) = table.cells.get(cell_idx) else {
            return json.to_string();
        };
        if cell.border_fill_id == incoming_bf_id
            || !Self::cell_is_covered_by_zone_border_fill(table, cell, incoming_bf_id)
        {
            return json.to_string();
        }

        let incoming_idx = (incoming_bf_id as usize).saturating_sub(1);
        let own_idx = (cell.border_fill_id as usize).saturating_sub(1);
        let zone_bf = self.document.doc_info.border_fills.get(incoming_idx);
        let own_bf = self.document.doc_info.border_fills.get(own_idx);
        let incoming_borders_are_zone = zone_bf
            .map(|bf| Self::json_borders_match_border_fill(obj, bf))
            .unwrap_or(false);

        obj.insert(
            "borderFillId".to_string(),
            serde_json::Value::Number(serde_json::Number::from(cell.border_fill_id)),
        );
        if incoming_borders_are_zone {
            if let Some(bf) = own_bf {
                Self::write_border_json_from_border_fill(obj, bf);
            }
        }

        serde_json::to_string(&value).unwrap_or_else(|_| json.to_string())
    }

    fn cell_is_covered_by_zone_border_fill(
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        border_fill_id: u16,
    ) -> bool {
        let cell_start_row = cell.row;
        let cell_end_row = cell.row.saturating_add(cell.row_span).saturating_sub(1);
        let cell_start_col = cell.col;
        let cell_end_col = cell.col.saturating_add(cell.col_span).saturating_sub(1);
        table.zones.iter().any(|zone| {
            zone.border_fill_id == border_fill_id
                && cell_start_row <= zone.end_row
                && cell_end_row >= zone.start_row
                && cell_start_col <= zone.end_col
                && cell_end_col >= zone.start_col
        })
    }

    fn json_borders_match_border_fill(
        obj: &serde_json::Map<String, serde_json::Value>,
        bf: &crate::model::style::BorderFill,
    ) -> bool {
        const KEYS: [&str; 4] = ["borderLeft", "borderRight", "borderTop", "borderBottom"];
        KEYS.iter().enumerate().all(|(idx, key)| {
            let Some(border) = obj.get(*key).and_then(|v| v.as_object()) else {
                return false;
            };
            let line = bf.borders[idx];
            let type_matches = border.get("type").and_then(|v| v.as_i64()).map(|v| v as u8)
                == Some(border_line_type_to_u8_val(line.line_type));
            let width_matches = border
                .get("width")
                .and_then(|v| v.as_i64())
                .map(|v| v as u8)
                == Some(line.width);
            let color_matches = border
                .get("color")
                .and_then(|v| v.as_str())
                .map(|v| v.eq_ignore_ascii_case(&color_ref_to_css(line.color)))
                .unwrap_or(false);
            type_matches && width_matches && color_matches
        })
    }

    fn write_border_json_from_border_fill(
        obj: &mut serde_json::Map<String, serde_json::Value>,
        bf: &crate::model::style::BorderFill,
    ) {
        const KEYS: [&str; 4] = ["borderLeft", "borderRight", "borderTop", "borderBottom"];
        for (idx, key) in KEYS.iter().enumerate() {
            let line = bf.borders[idx];
            obj.insert(
                (*key).to_string(),
                serde_json::json!({
                    "type": border_line_type_to_u8_val(line.line_type),
                    "width": line.width,
                    "color": color_ref_to_css(line.color),
                }),
            );
        }
    }

    /// 선택 영역을 하나의 셀처럼 취급하는 cellzone 테두리/배경 속성을 적용한다.
    pub(crate) fn set_cell_zone_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
        json: &str,
    ) -> Result<String, HwpError> {
        let cellzone_json = Self::strip_center_line_for_cellzone_json(json);
        let new_bf_id = self.create_border_fill_from_json(&cellzone_json);
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        if table.row_count == 0 || table.col_count == 0 {
            return Err(HwpError::RenderError(
                "빈 표에는 cellzone을 적용할 수 없습니다".to_string(),
            ));
        }

        let max_row = table.row_count.saturating_sub(1);
        let max_col = table.col_count.saturating_sub(1);
        let sr = start_row.min(end_row).min(max_row);
        let er = start_row.max(end_row).min(max_row);
        let sc = start_col.min(end_col).min(max_col);
        let ec = start_col.max(end_col).min(max_col);

        if let Some(zone) = table.zones.iter_mut().find(|zone| {
            zone.start_row == sr && zone.end_row == er && zone.start_col == sc && zone.end_col == ec
        }) {
            zone.border_fill_id = new_bf_id;
        } else {
            table.zones.push(crate::model::table::TableZone {
                start_col: sc,
                start_row: sr,
                end_col: ec,
                end_row: er,
                border_fill_id: new_bf_id,
            });
        }
        table.dirty = true;

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(format!(
            "{{\"ok\":true,\"startRow\":{},\"startCol\":{},\"endRow\":{},\"endCol\":{},\"borderFillId\":{}}}",
            sr, sc, er, ec, new_bf_id
        ))
    }

    fn strip_center_line_for_cellzone_json(json: &str) -> String {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(json) else {
            return json.to_string();
        };
        let Some(obj) = value.as_object_mut() else {
            return json.to_string();
        };
        obj.insert(
            "centerLine".to_string(),
            serde_json::Value::String("NONE".to_string()),
        );
        serde_json::to_string(&value).unwrap_or_else(|_| json.to_string())
    }

    /// 셀 테두리 변경 시 이웃 셀의 공유 엣지 테두리를 동기화한다.
    ///
    /// HWP 표에서 인접한 두 셀은 같은 엣지를 공유한다.
    /// 한쪽 셀의 테두리만 변경하면 merge_border 우선순위에 의해
    /// 변경이 반영되지 않을 수 있으므로, 이웃 셀의 대응 테두리도 함께 갱신한다.
    ///
    /// borders 배열: [좌(0), 우(1), 상(2), 하(3)]
    fn update_neighbor_borders(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        skip_cell_idx: usize,
        target_row: usize,
        target_col: usize,
        target_col_span: usize,
        target_row_span: usize,
        new_borders: &[crate::model::style::BorderLine; 4],
    ) {
        use crate::model::style::BorderLine;

        // 1단계: 이웃 셀 탐색 — (셀 인덱스, old_bf_id, 갱신할 방향, 새 테두리)
        let mut updates: Vec<(usize, u16, usize, BorderLine)> = Vec::new();
        {
            let table = match self.get_table_mut(section_idx, parent_para_idx, control_idx) {
                Ok(t) => t,
                Err(_) => return,
            };
            for (ci, cell) in table.cells.iter().enumerate() {
                if ci == skip_cell_idx {
                    continue;
                }
                let cr = cell.row as usize;
                let cc = cell.col as usize;
                let cs = cell.col_span as usize;
                let rs = cell.row_span as usize;
                let bf = cell.border_fill_id;

                // 대상 셀의 좌측 엣지 공유 → 이웃 우측
                if cc + cs == target_col
                    && cr < target_row + target_row_span
                    && cr + rs > target_row
                {
                    updates.push((ci, bf, 1, new_borders[0]));
                }
                // 대상 셀의 우측 엣지 공유 → 이웃 좌측
                if cc == target_col + target_col_span
                    && cr < target_row + target_row_span
                    && cr + rs > target_row
                {
                    updates.push((ci, bf, 0, new_borders[1]));
                }
                // 대상 셀의 상측 엣지 공유 → 이웃 하측
                if cr + rs == target_row
                    && cc < target_col + target_col_span
                    && cc + cs > target_col
                {
                    updates.push((ci, bf, 3, new_borders[2]));
                }
                // 대상 셀의 하측 엣지 공유 → 이웃 상측
                if cr == target_row + target_row_span
                    && cc < target_col + target_col_span
                    && cc + cs > target_col
                {
                    updates.push((ci, bf, 2, new_borders[3]));
                }
            }
        } // table borrow 해제

        // 2단계: 각 이웃 셀의 BorderFill 복제 + 해당 방향만 교체
        for (ci, old_bf_id, dir, new_border) in updates {
            if old_bf_id == 0 {
                continue;
            }
            let bf_idx = (old_bf_id as usize) - 1;
            if bf_idx >= self.document.doc_info.border_fills.len() {
                continue;
            }

            let mut new_bf = self.document.doc_info.border_fills[bf_idx].clone();
            new_bf.borders[dir] = new_border;

            // 동일한 BorderFill 검색/추가
            let bf_id = {
                use super::super::helpers::border_fills_equal;
                let found = self
                    .document
                    .doc_info
                    .border_fills
                    .iter()
                    .enumerate()
                    .find(|(_, existing)| border_fills_equal(existing, &new_bf))
                    .map(|(i, _)| (i + 1) as u16);
                match found {
                    Some(id) => id,
                    None => {
                        self.document.doc_info.border_fills.push(new_bf);
                        self.document.doc_info.border_fills.len() as u16
                    }
                }
            };

            let table = match self.get_table_mut(section_idx, parent_para_idx, control_idx) {
                Ok(t) => t,
                Err(_) => return,
            };
            table.cells[ci].border_fill_id = bf_id;
        }

        // 스타일 재계산
        self.styles =
            crate::renderer::style_resolver::resolve_styles(&self.document.doc_info, self.dpi);
    }

    /// 여러 셀의 width/height를 한 번에 조절한다 (네이티브).
    ///
    /// json 형식: `[{"cellIdx":0,"widthDelta":150},{"cellIdx":2,"heightDelta":-100}]`
    pub(crate) fn resize_table_cells_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        const MIN_CELL_SIZE: u32 = 200; // 최소 셀 크기 (HWPUNIT)

        // JSON 배열을 수동 파싱: [{"cellIdx":N,"widthDelta":D,"heightDelta":D}, ...]
        let trimmed = json.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(HwpError::RenderError("잘못된 JSON 배열 형식".to_string()));
        }
        let inner = &trimmed[1..trimmed.len() - 1];

        // 각 {} 객체를 추출
        struct CellUpdate {
            cell_idx: usize,
            width_delta: i32,
            height_delta: i32,
            local_resize: bool,
            render_width: Option<u32>,
            render_height: Option<u32>,
        }
        let mut updates: Vec<CellUpdate> = Vec::new();
        let mut force_local_resize = false;

        let mut depth = 0i32;
        let mut start = 0usize;
        for (i, ch) in inner.char_indices() {
            match ch {
                '{' => {
                    if depth == 0 {
                        start = i;
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let obj = &inner[start..=i];
                        // cellIdx 파싱
                        let cell_idx = Self::parse_json_i32(obj, "cellIdx").unwrap_or(-1);
                        if cell_idx < 0 {
                            continue;
                        }
                        let width_delta = Self::parse_json_i32(obj, "widthDelta").unwrap_or(0);
                        let height_delta = Self::parse_json_i32(obj, "heightDelta").unwrap_or(0);
                        let local_resize = obj.contains("\"localResize\":true")
                            || obj.contains("\"localResize\": true");
                        force_local_resize |= local_resize;
                        let render_width = Self::parse_json_i32(obj, "renderWidth")
                            .and_then(|v| (v > 0).then_some(v as u32));
                        let render_height = Self::parse_json_i32(obj, "renderHeight")
                            .and_then(|v| (v > 0).then_some(v as u32));
                        updates.push(CellUpdate {
                            cell_idx: cell_idx as usize,
                            width_delta,
                            height_delta,
                            local_resize,
                            render_width,
                            render_height,
                        });
                    }
                }
                _ => {}
            }
        }

        if updates.is_empty() {
            return Ok("{\"ok\":true}".to_string());
        }

        // 셀 업데이트 적용
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        let original_width = table.common.width;
        let original_height = table.common.height;
        let original_row_height_sum: u32 = table.get_row_heights().iter().sum();
        let mut applied_width_delta: i64 = 0;
        let mut applied_height_delta: i64 = 0;
        let mut width_delta_by_row = std::collections::BTreeMap::<u16, (usize, i64)>::new();
        let mut height_delta_by_col = std::collections::BTreeMap::<u16, (usize, i64)>::new();
        let mut local_resize_rows = std::collections::BTreeSet::<u16>::new();
        let mut local_resize_cols = std::collections::BTreeSet::<u16>::new();
        for upd in &updates {
            if let Some(cell) = table.cells.get_mut(upd.cell_idx) {
                if upd.width_delta != 0 {
                    let old_w = cell.width;
                    let new_w =
                        (cell.width as i32 + upd.width_delta).max(MIN_CELL_SIZE as i32) as u32;
                    cell.width = new_w;
                    let actual_delta = new_w as i64 - old_w as i64;
                    applied_width_delta += actual_delta;
                    let entry = width_delta_by_row.entry(cell.row).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += actual_delta;
                }
                if upd.height_delta != 0 {
                    let old_h = cell.height;
                    let new_h =
                        (cell.height as i32 + upd.height_delta).max(MIN_CELL_SIZE as i32) as u32;
                    cell.height = new_h;
                    let actual_delta = new_h as i64 - old_h as i64;
                    applied_height_delta += actual_delta;
                    let entry = height_delta_by_col.entry(cell.col).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += actual_delta;
                }
            }
            if upd.local_resize {
                if let Some(width) = upd.render_width {
                    if let Some(cell) = table.cells.get(upd.cell_idx) {
                        local_resize_rows.insert(cell.row);
                    }
                    if let Some((_, existing)) = table
                        .local_resize_cell_widths
                        .iter_mut()
                        .find(|(idx, _)| *idx == upd.cell_idx)
                    {
                        *existing = width;
                    } else {
                        table.local_resize_cell_widths.push((upd.cell_idx, width));
                    }
                }
                if let Some(height) = upd.render_height {
                    if let Some(cell) = table.cells.get(upd.cell_idx) {
                        local_resize_cols.insert(cell.col);
                    }
                    if let Some((_, existing)) = table
                        .local_resize_cell_heights
                        .iter_mut()
                        .find(|(idx, _)| *idx == upd.cell_idx)
                    {
                        *existing = height;
                    } else {
                        table.local_resize_cell_heights.push((upd.cell_idx, height));
                    }
                }
            }
        }
        for row in local_resize_rows {
            if !table.local_resize_rows.contains(&row) {
                table.local_resize_rows.push(row);
            }
        }
        for col in local_resize_cols {
            if !table.local_resize_cols.contains(&col) {
                table.local_resize_cols.push(col);
            }
        }
        for (row, (count, delta_sum)) in width_delta_by_row {
            if count >= 2
                && (delta_sum == 0 || force_local_resize)
                && !table.local_resize_rows.contains(&row)
            {
                table.local_resize_rows.push(row);
            }
        }
        for (col, (count, delta_sum)) in height_delta_by_col {
            if count >= 2
                && (delta_sum == 0 || force_local_resize)
                && !table.local_resize_cols.contains(&col)
            {
                table.local_resize_cols.push(col);
            }
        }
        table.update_ctrl_dimensions();
        if updates.iter().any(|u| u.height_delta != 0)
            && !force_local_resize
            && original_height > original_row_height_sum
            && table.row_count > 1
        {
            // 여러 행 표에서 일부 행을 조절할 때만 생성 표의 표시 height 여유분을 보존한다.
            // 1행 표는 조절한 셀 높이가 곧 표 높이라는 기존 TAC 전환 회귀 규칙을 유지해야 한다.
            let resized_row_height_sum: u32 = table.get_row_heights().iter().sum();
            let row_height_delta = resized_row_height_sum as i64 - original_row_height_sum as i64;
            let adjusted_height = if row_height_delta >= 0 {
                original_height.saturating_add(row_height_delta.min(u32::MAX as i64) as u32)
            } else {
                original_height.saturating_sub((-row_height_delta).min(u32::MAX as i64) as u32)
            }
            .max(resized_row_height_sum);
            table.common.height = adjusted_height;
            if table.raw_ctrl_data.len() >= common_obj_offsets::HEIGHT.end {
                table.raw_ctrl_data[common_obj_offsets::HEIGHT]
                    .copy_from_slice(&adjusted_height.to_le_bytes());
            }
        }
        if applied_width_delta == 0
            || (force_local_resize && updates.iter().any(|u| u.width_delta != 0))
        {
            table.common.width = original_width;
            if table.raw_ctrl_data.len() >= common_obj_offsets::WIDTH.end {
                table.raw_ctrl_data[common_obj_offsets::WIDTH]
                    .copy_from_slice(&original_width.to_le_bytes());
            }
        }
        if applied_height_delta == 0
            || (force_local_resize && updates.iter().any(|u| u.height_delta != 0))
        {
            table.common.height = original_height;
            if table.raw_ctrl_data.len() >= common_obj_offsets::HEIGHT.end {
                table.raw_ctrl_data[common_obj_offsets::HEIGHT]
                    .copy_from_slice(&original_height.to_le_bytes());
            }
        }
        table.dirty = true;

        // 너비가 변경된 셀의 모든 문단에 대해 line_segs 재계산 (텍스트 리플로우)
        let reflow_cells: Vec<(usize, usize)> = {
            let para = &self.document.sections[section_idx].paragraphs[parent_para_idx];
            if let Some(Control::Table(table)) = para.controls.get(control_idx) {
                updates
                    .iter()
                    .filter(|u| u.width_delta != 0)
                    .filter_map(|u| {
                        let pc = table.cells.get(u.cell_idx)?.paragraphs.len();
                        Some((u.cell_idx, pc))
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };
        for (cell_idx, para_count) in reflow_cells {
            for cell_para_idx in 0..para_count {
                self.reflow_cell_paragraph(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                );
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok("{\"ok\":true}".to_string())
    }

    /// 표의 열별 폭(HWPUNIT)을 절대값으로 설정한다 (네이티브).
    ///
    /// `widths.len()` 은 표의 열 수와 같아야 한다. `insert_table_column` 과 달리
    /// 표 전체 폭이 입력한 폭들의 합이 되므로, 페이지를 넘지 않게 하려면
    /// 합을 본문 폭 이하로 전달하거나 `fit_table_to_page_native` 를 쓴다.
    pub fn set_table_column_widths_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        widths: Vec<u32>,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        table
            .set_column_widths(&widths)
            .map_err(HwpError::RenderError)?;
        table.dirty = true;
        let col_count = table.col_count;
        let total: u32 = table.get_column_widths().iter().sum();

        // 폭이 바뀐 셀의 모든 문단을 재배치(line_segs 재계산)한다.
        let reflow: Vec<(usize, usize)> = {
            let para = &self.document.sections[section_idx].paragraphs[parent_para_idx];
            if let Some(Control::Table(t)) = para.controls.get(control_idx) {
                t.cells
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.paragraphs.len()))
                    .collect()
            } else {
                Vec::new()
            }
        };
        for (cell_idx, para_count) in reflow {
            for cell_para_idx in 0..para_count {
                self.reflow_cell_paragraph(
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    cell_idx,
                    cell_para_idx,
                );
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"colCount\":{},\"tableWidth\":{}",
            col_count, total
        )))
    }

    /// 표를 본문(페이지 텍스트) 폭에 맞춰 비례 축소한다 (네이티브).
    ///
    /// 표의 열 폭 합이 본문 폭(페이지 본문 영역 폭 − 표 바깥 좌우 여백)을 넘으면
    /// 각 열을 같은 비율로 줄여 표가 페이지를 넘지 않게 한다. 이미 본문 폭 이하이면
    /// 변경하지 않는다(축소 전용).
    pub fn fit_table_to_page_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        const MIN_COL: u32 = 200; // 최소 열 폭 (HWPUNIT)

        // 현재 열 폭과 표 바깥 좌우 여백을 읽는다.
        let (widths, outer_lr) = {
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            let outer = table.outer_margin_left as i64 + table.outer_margin_right as i64;
            (table.get_column_widths(), outer.max(0) as u32)
        };
        let total: u32 = widths.iter().sum();

        // 본문(텍스트) 폭 = 페이지 본문 영역 폭 − 표 바깥 좌우 여백.
        let page_def = &self.document.sections[section_idx].section_def.page_def;
        let body = crate::model::page::PageAreas::from_page_def(page_def).body_area;
        let body_w = (body.right - body.left).max(0) as u32;
        let target = body_w.saturating_sub(outer_lr);

        if total == 0 || target == 0 || total <= target {
            // 이미 페이지 폭 안에 들어옴 — 변경 없음.
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"colCount\":{},\"tableWidth\":{},\"pageContentWidth\":{},\"changed\":false",
                widths.len(),
                total,
                target
            )));
        }

        // 비례 축소(내림) 후 잔여분을 마지막 열에 더해 합이 정확히 target 이 되게 한다.
        let mut new_w: Vec<u32> = widths
            .iter()
            .map(|&w| ((w as u64 * target as u64) / total as u64) as u32)
            .collect();
        let assigned: u64 = new_w.iter().map(|&w| w as u64).sum();
        let remainder = target as u64 - assigned; // 내림이므로 항상 >= 0
        if let Some(last) = new_w.last_mut() {
            *last = (*last as u64 + remainder) as u32;
        }
        for w in &mut new_w {
            if *w < MIN_COL {
                *w = MIN_COL;
            }
        }

        self.set_table_column_widths_native(section_idx, parent_para_idx, control_idx, new_w)?;

        let new_total: u32 = {
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            table.get_column_widths().iter().sum()
        };
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"colCount\":{},\"tableWidth\":{},\"pageContentWidth\":{},\"changed\":true",
            widths.len(),
            new_total,
            target
        )))
    }

    /// JSON 객체 내 정수 키 값을 파싱하는 헬퍼.
    pub(crate) fn parse_json_i32(json: &str, key: &str) -> Option<i32> {
        let pattern = format!("\"{}\":", key);
        let start = json.find(&pattern)? + pattern.len();
        let rest = json[start..].trim_start();
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(rest.len());
        if end == 0 {
            return None;
        }
        rest[..end].parse().ok()
    }

    /// 표 위치 오프셋을 이동한다 (네이티브).
    ///
    /// treat_as_char(본문배치) 표의 경우, v_offset이 현재 줄 높이를 넘으면
    /// 다음/이전 문단으로 표를 이동시킨다 (문단 간 이동).
    pub(crate) fn move_table_offset_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        delta_h: i32,
        delta_v: i32,
    ) -> Result<String, HwpError> {
        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;

        // CommonObjAttr 바이트 레이아웃: flags/v_offset/h_offset
        while table.raw_ctrl_data.len() < common_obj_offsets::H_OFFSET.end {
            table.raw_ctrl_data.push(0);
        }

        let is_treat_as_char = (table.attr & 0x01) != 0;

        // vertical_offset: CommonObjAttr::V_OFFSET (i32 LE)
        let mut new_v = if delta_v != 0 {
            let cur_v = i32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::V_OFFSET]
                    .try_into()
                    .unwrap(),
            );
            let nv = cur_v.wrapping_add(delta_v);
            table.raw_ctrl_data[common_obj_offsets::V_OFFSET].copy_from_slice(&nv.to_le_bytes());
            table.common.vertical_offset = nv as u32;
            nv
        } else {
            i32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::V_OFFSET]
                    .try_into()
                    .unwrap(),
            )
        };

        // horizontal_offset: CommonObjAttr::H_OFFSET (i32 LE)
        if delta_h != 0 {
            let cur_h = i32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::H_OFFSET]
                    .try_into()
                    .unwrap(),
            );
            let new_h = cur_h.wrapping_add(delta_h);
            table.raw_ctrl_data[common_obj_offsets::H_OFFSET].copy_from_slice(&new_h.to_le_bytes());
            table.common.horizontal_offset = new_h as u32;
        }

        // treat_as_char 표: 문단 경계를 넘으면 문단 이동 (다중 경계 루프)
        let mut result_ppi = parent_para_idx;
        if is_treat_as_char && delta_v != 0 {
            let para_count = self.document.sections[section_idx].paragraphs.len();

            // 아래로: v_offset >= line_height이면 반복적으로 다음 문단과 교환
            while result_ppi + 1 < para_count {
                let lh = self.document.sections[section_idx].paragraphs[result_ppi]
                    .line_segs
                    .first()
                    .map(|ls| ls.line_height)
                    .unwrap_or(1000);
                if new_v < lh {
                    break;
                }
                new_v -= lh;
                self.document.sections[section_idx]
                    .paragraphs
                    .swap(result_ppi, result_ppi + 1);
                result_ppi += 1;
            }

            // 위로: v_offset < 0이면 반복적으로 이전 문단과 교환
            while new_v < 0 && result_ppi > 0 {
                let prev_lh = self.document.sections[section_idx].paragraphs[result_ppi - 1]
                    .line_segs
                    .first()
                    .map(|ls| ls.line_height)
                    .unwrap_or(1000);
                new_v += prev_lh;
                self.document.sections[section_idx]
                    .paragraphs
                    .swap(result_ppi - 1, result_ppi);
                result_ppi -= 1;
            }

            // 최종 v_offset 갱신
            if result_ppi != parent_para_idx {
                let tbl = self.get_table_mut(section_idx, result_ppi, control_idx)?;
                tbl.raw_ctrl_data[common_obj_offsets::V_OFFSET]
                    .copy_from_slice(&new_v.to_le_bytes());
                tbl.common.vertical_offset = new_v as u32;
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(format!(
            "{{\"ok\":true,\"ppi\":{},\"ci\":{}}}",
            result_ppi, control_idx
        ))
    }

    /// 표 속성을 조회한다 (네이티브).
    pub(crate) fn get_table_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;

        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                ))
            }
        };

        let pb = match table.page_break {
            crate::model::table::TablePageBreak::None => 0,
            crate::model::table::TablePageBreak::CellBreak => 1,
            crate::model::table::TablePageBreak::RowBreak => 2,
        };

        let bf_json = self.build_border_fill_json_by_id(table.border_fill_id);

        // raw_ctrl_data에서 표 크기 & 바깥 여백 추출 (parse_common_obj_attr 정합)
        // [0..4]=flags, [4..8]=v_offset, [8..12]=h_offset, [12..16]=width, [16..20]=height
        let rd = &table.raw_ctrl_data;
        let table_width = if rd.len() >= common_obj_offsets::WIDTH.end {
            u32::from_le_bytes(rd[common_obj_offsets::WIDTH].try_into().unwrap())
        } else {
            0
        };
        let table_height = if rd.len() >= common_obj_offsets::HEIGHT.end {
            u32::from_le_bytes(rd[common_obj_offsets::HEIGHT].try_into().unwrap())
        } else {
            0
        };
        // outer_margin: [24..32] (parse_common_obj_attr 정합)
        // [20..24]=z_order, [24..26]=left, [26..28]=right, [28..30]=top, [30..32]=bottom
        let outer_left = if rd.len() >= common_obj_offsets::MARGIN_LEFT.end {
            i16::from_le_bytes(rd[common_obj_offsets::MARGIN_LEFT].try_into().unwrap())
        } else {
            0
        };
        let outer_right = if rd.len() >= common_obj_offsets::MARGIN_RIGHT.end {
            i16::from_le_bytes(rd[common_obj_offsets::MARGIN_RIGHT].try_into().unwrap())
        } else {
            0
        };
        let outer_top = if rd.len() >= common_obj_offsets::MARGIN_TOP.end {
            i16::from_le_bytes(rd[common_obj_offsets::MARGIN_TOP].try_into().unwrap())
        } else {
            0
        };
        let outer_bottom = if rd.len() >= common_obj_offsets::MARGIN_BOTTOM.end {
            i16::from_le_bytes(rd[common_obj_offsets::MARGIN_BOTTOM].try_into().unwrap())
        } else {
            0
        };

        // 캡션 정보
        let caption_json = if let Some(ref cap) = table.caption {
            let dir = match cap.direction {
                crate::model::shape::CaptionDirection::Left => 0,
                crate::model::shape::CaptionDirection::Right => 1,
                crate::model::shape::CaptionDirection::Top => 2,
                crate::model::shape::CaptionDirection::Bottom => 3,
            };
            let va = match cap.vert_align {
                crate::model::shape::CaptionVertAlign::Top => 0,
                crate::model::shape::CaptionVertAlign::Center => 1,
                crate::model::shape::CaptionVertAlign::Bottom => 2,
            };
            format!(",\"captionDirection\":{},\"captionVertAlign\":{},\"captionWidth\":{},\"captionSpacing\":{},\"hasCaption\":true",
                dir, va, cap.width, cap.spacing)
        } else {
            ",\"hasCaption\":false".to_string()
        };

        // HWPX: common 필드에서 직접 읽기. HWP: attr 비트 연산 (common에도 동일하게 파싱됨)
        let treat_as_char = table.common.treat_as_char;
        let text_wrap = match table.common.text_wrap {
            crate::model::shape::TextWrap::Square => "Square",
            crate::model::shape::TextWrap::Tight => "Square",
            crate::model::shape::TextWrap::Through => "Square",
            crate::model::shape::TextWrap::TopAndBottom => "TopAndBottom",
            crate::model::shape::TextWrap::BehindText => "BehindText",
            crate::model::shape::TextWrap::InFrontOfText => "InFrontOfText",
        };
        let vert_rel_to = match table.common.vert_rel_to {
            crate::model::shape::VertRelTo::Paper => "Paper",
            crate::model::shape::VertRelTo::Page => "Page",
            crate::model::shape::VertRelTo::Para => "Para",
        };
        let vert_align = match table.common.vert_align {
            crate::model::shape::VertAlign::Top => "Top",
            crate::model::shape::VertAlign::Center => "Center",
            crate::model::shape::VertAlign::Bottom => "Bottom",
            crate::model::shape::VertAlign::Inside => "Inside",
            crate::model::shape::VertAlign::Outside => "Outside",
        };
        let horz_rel_to = match table.common.horz_rel_to {
            crate::model::shape::HorzRelTo::Paper => "Paper",
            crate::model::shape::HorzRelTo::Page => "Page",
            crate::model::shape::HorzRelTo::Column => "Column",
            crate::model::shape::HorzRelTo::Para => "Para",
        };
        let horz_align = match table.common.horz_align {
            crate::model::shape::HorzAlign::Left => "Left",
            crate::model::shape::HorzAlign::Center => "Center",
            crate::model::shape::HorzAlign::Right => "Right",
            crate::model::shape::HorzAlign::Inside => "Inside",
            crate::model::shape::HorzAlign::Outside => "Outside",
        };
        // CommonObjAttr: flags/v_offset/h_offset
        let vert_offset = if rd.len() >= common_obj_offsets::V_OFFSET.end {
            i32::from_le_bytes(rd[common_obj_offsets::V_OFFSET].try_into().unwrap())
        } else {
            0
        };
        let horz_offset = if rd.len() >= common_obj_offsets::H_OFFSET.end {
            i32::from_le_bytes(rd[common_obj_offsets::H_OFFSET].try_into().unwrap())
        } else {
            0
        };
        let restrict_in_page = (table.attr >> 13) & 0x01 != 0;
        let allow_overlap = (table.attr >> 14) & 0x01 != 0;
        // prevent_page_break: CommonObjAttr::PREVENT_PAGE_BREAK
        let keep_with_anchor = if rd.len() >= common_obj_offsets::PREVENT_PAGE_BREAK.end {
            i32::from_le_bytes(
                rd[common_obj_offsets::PREVENT_PAGE_BREAK]
                    .try_into()
                    .unwrap(),
            ) != 0
        } else {
            false
        };

        Ok(format!(
            "{{\"cellSpacing\":{},\"paddingLeft\":{},\"paddingRight\":{},\"paddingTop\":{},\"paddingBottom\":{},\"pageBreak\":{},\"repeatHeader\":{},{},\"tableWidth\":{},\"tableHeight\":{},\"outerLeft\":{},\"outerRight\":{},\"outerTop\":{},\"outerBottom\":{}{},\"treatAsChar\":{},\"textWrap\":\"{}\",\"vertRelTo\":\"{}\",\"vertAlign\":\"{}\",\"horzRelTo\":\"{}\",\"horzAlign\":\"{}\",\"vertOffset\":{},\"horzOffset\":{},\"restrictInPage\":{},\"allowOverlap\":{},\"keepWithAnchor\":{}}}",
            table.cell_spacing,
            table.padding.left, table.padding.right, table.padding.top, table.padding.bottom,
            pb, table.repeat_header,
            bf_json,
            table_width, table_height,
            outer_left, outer_right, outer_top, outer_bottom,
            caption_json,
            treat_as_char,
            text_wrap, vert_rel_to, vert_align, horz_rel_to, horz_align,
            vert_offset, horz_offset,
            restrict_in_page, allow_overlap, keep_with_anchor,
        ))
    }

    /// 표 속성을 수정한다 (네이티브).
    pub(crate) fn set_table_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        use super::super::helpers::{json_bool, json_i16, json_i32, json_str, json_u32, json_u8};

        let caption_style = self
            .document
            .doc_info
            .styles
            .iter()
            .position(|s| s.english_name == "Caption" || s.local_name == "캡션")
            .and_then(|idx| self.document.doc_info.styles.get(idx).map(|s| (idx, s)));
        let (caption_style_id, caption_para_shape_id, caption_char_shape_id) = caption_style
            .map(|(idx, s)| (idx as u8, s.para_shape_id, s.char_shape_id as u32))
            .unwrap_or((0, 0, 0));

        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;

        if let Some(v) = json_i16(json, "cellSpacing") {
            table.cell_spacing = v;
        }
        if let Some(v) = json_i16(json, "paddingLeft") {
            table.padding.left = v;
        }
        if let Some(v) = json_i16(json, "paddingRight") {
            table.padding.right = v;
        }
        if let Some(v) = json_i16(json, "paddingTop") {
            table.padding.top = v;
        }
        if let Some(v) = json_i16(json, "paddingBottom") {
            table.padding.bottom = v;
        }
        if let Some(v) = json_u8(json, "pageBreak") {
            table.page_break = match v {
                1 => crate::model::table::TablePageBreak::CellBreak,
                2 => crate::model::table::TablePageBreak::RowBreak,
                _ => crate::model::table::TablePageBreak::None,
            };
        }
        if let Some(v) = json_bool(json, "repeatHeader") {
            table.repeat_header = v;
        }
        if let Some(v) = json_bool(json, "treatAsChar") {
            if v {
                table.attr |= 0x01;
            } else {
                table.attr &= !0x01;
            }
            table.common.treat_as_char = v;
        }

        // 위치 속성: attr 비트 필드
        if let Some(v) = json_str(json, "textWrap") {
            let bits: u32 = match v.as_str() {
                "Square" => 0,
                "TopAndBottom" => 1,
                "BehindText" => 2,
                "InFrontOfText" => 3,
                _ => 0,
            };
            table.attr = (table.attr & !(0x07 << 21)) | (bits << 21);
            table.common.text_wrap = match bits {
                1 => crate::model::shape::TextWrap::TopAndBottom,
                2 => crate::model::shape::TextWrap::BehindText,
                3 => crate::model::shape::TextWrap::InFrontOfText,
                _ => crate::model::shape::TextWrap::Square,
            };
        }
        if let Some(v) = json_str(json, "vertRelTo") {
            let bits: u32 = match v.as_str() {
                "Paper" => 0,
                "Page" => 1,
                "Para" => 2,
                _ => 0,
            };
            table.attr = (table.attr & !(0x03 << 3)) | (bits << 3);
            table.common.vert_rel_to = match bits {
                1 => crate::model::shape::VertRelTo::Page,
                2 => crate::model::shape::VertRelTo::Para,
                _ => crate::model::shape::VertRelTo::Paper,
            };
        }
        if let Some(v) = json_str(json, "vertAlign") {
            let bits: u32 = match v.as_str() {
                "Top" => 0,
                "Center" => 1,
                "Bottom" => 2,
                "Inside" => 3,
                "Outside" => 4,
                _ => 0,
            };
            table.attr = (table.attr & !(0x07 << 5)) | (bits << 5);
            table.common.vert_align = match bits {
                1 => crate::model::shape::VertAlign::Center,
                2 => crate::model::shape::VertAlign::Bottom,
                3 => crate::model::shape::VertAlign::Inside,
                4 => crate::model::shape::VertAlign::Outside,
                _ => crate::model::shape::VertAlign::Top,
            };
        }
        if let Some(v) = json_str(json, "horzRelTo") {
            let bits: u32 = match v.as_str() {
                "Paper" => 0,
                "Page" => 1,
                "Column" => 2,
                "Para" => 3,
                _ => 0,
            };
            table.attr = (table.attr & !(0x03 << 8)) | (bits << 8);
            table.common.horz_rel_to = match bits {
                1 => crate::model::shape::HorzRelTo::Page,
                2 => crate::model::shape::HorzRelTo::Column,
                3 => crate::model::shape::HorzRelTo::Para,
                _ => crate::model::shape::HorzRelTo::Paper,
            };
        }
        if let Some(v) = json_str(json, "horzAlign") {
            let bits: u32 = match v.as_str() {
                "Left" => 0,
                "Center" => 1,
                "Right" => 2,
                "Inside" => 3,
                "Outside" => 4,
                _ => 0,
            };
            table.attr = (table.attr & !(0x07 << 10)) | (bits << 10);
            table.common.horz_align = match bits {
                1 => crate::model::shape::HorzAlign::Center,
                2 => crate::model::shape::HorzAlign::Right,
                3 => crate::model::shape::HorzAlign::Inside,
                4 => crate::model::shape::HorzAlign::Outside,
                _ => crate::model::shape::HorzAlign::Left,
            };
        }
        table.common.attr = table.attr;
        // 위치 오프셋: CommonObjAttr [0..4]=flags, [4..8]=v_offset, [8..12]=h_offset
        while table.raw_ctrl_data.len() < common_obj_offsets::H_OFFSET.end {
            table.raw_ctrl_data.push(0);
        }
        if let Some(v) = json_i32(json, "vertOffset") {
            table.raw_ctrl_data[common_obj_offsets::V_OFFSET].copy_from_slice(&v.to_le_bytes());
            table.common.vertical_offset = v as u32;
        }
        if let Some(v) = json_i32(json, "horzOffset") {
            table.raw_ctrl_data[common_obj_offsets::H_OFFSET].copy_from_slice(&v.to_le_bytes());
            table.common.horizontal_offset = v as u32;
        }
        // restrictInPage → attr bit 13
        if let Some(v) = json_bool(json, "restrictInPage") {
            if v {
                table.attr |= 1 << 13;
                table.common.flow_with_text = true;
            } else {
                table.attr &= !(1 << 13);
                table.common.flow_with_text = false;
            }
            table.common.attr = table.attr;
        }
        // allowOverlap → attr bit 14
        if let Some(v) = json_bool(json, "allowOverlap") {
            if v {
                table.attr |= 1 << 14;
                table.common.allow_overlap = true;
            } else {
                table.attr &= !(1 << 14);
                table.common.allow_overlap = false;
            }
            table.common.attr = table.attr;
        }
        // attr 비트 변경을 raw_ctrl_data FLAGS(0..4)에도 반영. HWP5 직렬화기
        // (serialize_table)는 raw_ctrl_data 가 있으면 그대로 기록하므로, 여기
        // 반영하지 않으면 글자처럼 취급/배치/기준/정렬/쪽영역제한/겹침 변경이
        // 저장 파일에서 통째로 유실되고 재로드 시 원복된다. V_OFFSET/H_OFFSET/
        // PREVENT_PAGE_BREAK/MARGIN_* 패치와 동일 규칙 (미변경 시에는 파싱
        // 원본 attr 를 그대로 다시 쓰는 항등 연산이라 무해).
        table.raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&table.attr.to_le_bytes());
        // keepWithAnchor → prevent_page_break
        // CommonObjAttr::PREVENT_PAGE_BREAK (parse_common_obj_attr 정합)
        if let Some(v) = json_bool(json, "keepWithAnchor") {
            while table.raw_ctrl_data.len() < common_obj_offsets::PREVENT_PAGE_BREAK.end {
                table.raw_ctrl_data.push(0);
            }
            let val: i32 = if v { 1 } else { 0 };
            table.raw_ctrl_data[common_obj_offsets::PREVENT_PAGE_BREAK]
                .copy_from_slice(&val.to_le_bytes());
            table.common.prevent_page_break = val;
        }

        // 바깥 여백 (CommonObjAttr margin ranges, parse_common_obj_attr 정합)
        if table.raw_ctrl_data.len() >= common_obj_offsets::MARGIN_BOTTOM.end {
            if let Some(v) = json_i16(json, "outerLeft") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_LEFT]
                    .copy_from_slice(&v.to_le_bytes());
                table.common.margin.left = v;
            }
            if let Some(v) = json_i16(json, "outerRight") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
                    .copy_from_slice(&v.to_le_bytes());
                table.common.margin.right = v;
            }
            if let Some(v) = json_i16(json, "outerTop") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_TOP]
                    .copy_from_slice(&v.to_le_bytes());
                table.common.margin.top = v;
            }
            if let Some(v) = json_i16(json, "outerBottom") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
                    .copy_from_slice(&v.to_le_bytes());
                table.common.margin.bottom = v;
            }
        }

        // 캡션 생성/수정
        let mut caption_created = false;
        let mut caption_changed = false;
        if let Some(has_cap) = json_bool(json, "hasCaption") {
            if has_cap && table.caption.is_none() {
                let mut cap = crate::model::shape::Caption::default();
                let an = crate::model::control::AutoNumber {
                    number_type: crate::model::control::AutoNumberType::Table,
                    ..Default::default()
                };
                let mut cap_para = crate::model::paragraph::Paragraph::new_empty();
                // 한컴 표 캡션은 AutoNumber 앞에 "표" 접두어를 함께 표시한다.
                cap_para.text = "표  ".to_string();
                cap_para.char_count = 13;
                cap_para.char_count_msb = true;
                cap_para.control_mask = 1u32 << 0x12;
                cap_para.char_offsets = vec![0, 1, 2, 11];
                cap_para.style_id = caption_style_id;
                cap_para.para_shape_id = caption_para_shape_id;
                cap_para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
                    start_pos: 0,
                    char_shape_id: caption_char_shape_id,
                }];
                cap_para
                    .controls
                    .push(crate::model::control::Control::AutoNumber(an));
                cap_para.ctrl_data_records.push(None);
                // max_width = 표 전체 폭 (열 폭 합산)
                let total_width: u32 = table
                    .cells
                    .iter()
                    .filter(|c| c.row == 0)
                    .map(|c| c.width as u32)
                    .sum();
                cap.max_width = total_width;
                // LineSeg의 segment_width를 표 폭으로 설정 (텍스트 레이아웃 폭)
                if let Some(ls) = cap_para.line_segs.first_mut() {
                    ls.segment_width = total_width as i32;
                }
                cap.paragraphs.push(cap_para);
                cap.width = 8504; // 기본 캡션 크기 약 30mm
                cap.direction = crate::model::shape::CaptionDirection::Bottom;
                cap.spacing = 850; // 약 3mm
                table.caption = Some(cap);
                caption_created = true;
                // attr bit 29: 캡션 존재 플래그 (한컴 호환성)
                table.attr |= 1 << 29;
                table.common.attr = table.attr;
                table.raw_table_record_attr = table.attr;
            } else if !has_cap && table.caption.is_some() {
                table.caption = None;
                table.attr &= !(1 << 29);
                table.common.attr = table.attr;
                table.raw_table_record_attr = table.attr;
                caption_changed = true;
            }
        }
        // 캡션 속성 수정
        if let Some(ref mut cap) = table.caption {
            if let Some(v) = json_u8(json, "captionDirection") {
                cap.direction = match v {
                    0 => crate::model::shape::CaptionDirection::Left,
                    1 => crate::model::shape::CaptionDirection::Right,
                    2 => crate::model::shape::CaptionDirection::Top,
                    _ => crate::model::shape::CaptionDirection::Bottom,
                };
                caption_changed = true;
            }
            if let Some(v) = json_i16(json, "captionSpacing") {
                cap.spacing = v;
                caption_changed = true;
            }
            if let Some(v) = json_u32(json, "captionWidth") {
                cap.width = v;
                caption_changed = true;
            }
            if let Some(v) = json_u8(json, "captionVertAlign") {
                cap.vert_align = match v {
                    1 => crate::model::shape::CaptionVertAlign::Center,
                    2 => crate::model::shape::CaptionVertAlign::Bottom,
                    _ => crate::model::shape::CaptionVertAlign::Top,
                };
                caption_changed = true;
            }
        }
        if caption_changed || caption_created {
            table.dirty = true;
        }

        // BorderFill 변경 — 표 테두리/배경/대각선 변경 시 모든 셀에도 동일 적용
        // (HWP 렌더링은 cell.border_fill_id를 사용, table.border_fill_id는 페이지 분할용)
        let has_border_fill_change = json.contains("\"borderLeft\"")
            || json.contains("\"fillType\"")
            || json.contains("\"diagonalLine\"")
            || json.contains("\"diagonalSlash\"")
            || json.contains("\"diagonalBackSlash\"")
            || json.contains("\"diagonalWidth\"")
            || json.contains("\"diagonalColor\"")
            || json.contains("\"centerLine\"");
        if has_border_fill_change {
            let new_bf_id = self.create_border_fill_from_json(json);
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            table.border_fill_id = new_bf_id;
            for cell in &mut table.cells {
                cell.border_fill_id = new_bf_id;
            }
            table.dirty = true;
        }

        // 캡션 생성/수정/삭제 후에는 문서 전체 AutoNumber를 다시 배정한다.
        // 중간 표 캡션 삭제 시 남은 표 번호가 한컴처럼 1부터 이어지도록 보장한다.
        if caption_created || caption_changed {
            crate::parser::assign_auto_numbers(&mut self.document);
            if let Some(crate::model::control::Control::Table(ref mut tbl)) =
                self.document.sections[section_idx].paragraphs[parent_para_idx]
                    .controls
                    .get_mut(control_idx)
            {
                if let Some(ref mut cap) = tbl.caption {
                    let available_width_hu = if matches!(
                        cap.direction,
                        crate::model::shape::CaptionDirection::Left
                            | crate::model::shape::CaptionDirection::Right
                    ) {
                        cap.width
                    } else {
                        cap.max_width
                    };
                    let available_width_px =
                        crate::renderer::hwpunit_to_px(available_width_hu as i32, self.dpi);
                    crate::renderer::composer::reflow_line_segs(
                        &mut cap.paragraphs[0],
                        available_width_px,
                        &self.styles,
                        self.dpi,
                    );
                }
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        if caption_created {
            let char_offset = {
                let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
                table.caption.as_ref().map_or(0, |c| {
                    c.paragraphs.first().map_or(0, |p| p.text.chars().count())
                })
            };
            Ok(format!(
                "{{\"ok\":true,\"captionCharOffset\":{}}}",
                char_offset
            ))
        } else {
            Ok("{\"ok\":true}".to_string())
        }
    }

    /// 표 전체의 바운딩박스를 반환한다 (네이티브).
    pub(crate) fn get_table_bbox_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 해당 문단에 표 컨트롤이 실제로 있는지 사전 확인 (전체 페이지 순회 방지)
        let has_table = self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(parent_para_idx))
            .and_then(|p| p.controls.get(control_idx))
            .map(|c| matches!(c, Control::Table(_)))
            .unwrap_or(false);
        if !has_table {
            return Err(HwpError::RenderError(format!(
                "표 노드를 찾을 수 없습니다 (sec={}, ppi={}, ci={})",
                section_idx, parent_para_idx, control_idx
            )));
        }

        fn find_table_bbox(
            node: &RenderNode,
            sec: usize,
            ppi: usize,
            ci: usize,
            page_idx: usize,
        ) -> Option<String> {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                if tn.section_index == Some(sec)
                    && tn.para_index == Some(ppi)
                    && tn.control_index == Some(ci)
                {
                    return Some(format!(
                        "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"width\":{:.1},\"height\":{:.1}}}",
                        page_idx,
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height
                    ));
                }
            }
            for child in &node.children {
                if let Some(result) = find_table_bbox(child, sec, ppi, ci, page_idx) {
                    return Some(result);
                }
            }
            None
        }

        let total_pages = self.page_count() as usize;
        for page_num in 0..total_pages {
            let tree = self.build_page_tree_cached(page_num as u32)?;
            if let Some(result) = find_table_bbox(
                &tree.root,
                section_idx,
                parent_para_idx,
                control_idx,
                page_num,
            ) {
                return Ok(result);
            }
        }

        Err(HwpError::RenderError(format!(
            "표 노드를 찾을 수 없습니다 (sec={}, ppi={}, ci={})",
            section_idx, parent_para_idx, control_idx
        )))
    }

    /// [Task #919] 글상자/도형 컨트롤의 페이지 좌표 바운딩박스를 반환한다 (네이티브).
    ///
    /// render_tree 의 Rectangle/Ellipse/Path 노드 중 (sec, ppi, ci) 매칭되는 것을 찾아
    /// bbox 를 반환. `getTableBBox` 동등 패턴. studio 의 `isShapeBorderClick` 에서 사용.
    pub(crate) fn get_shape_bbox_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 해당 문단에 Shape 컨트롤이 실제로 있는지 사전 확인
        let has_shape = self
            .document
            .sections
            .get(section_idx)
            .and_then(|s| s.paragraphs.get(parent_para_idx))
            .and_then(|p| p.controls.get(control_idx))
            .map(|c| matches!(c, Control::Shape(_)))
            .unwrap_or(false);
        if !has_shape {
            return Err(HwpError::RenderError(format!(
                "글상자/도형 노드를 찾을 수 없습니다 (sec={}, ppi={}, ci={})",
                section_idx, parent_para_idx, control_idx
            )));
        }

        fn find_shape_bbox(
            node: &RenderNode,
            sec: usize,
            ppi: usize,
            ci: usize,
            page_idx: usize,
        ) -> Option<String> {
            let meta: Option<(Option<usize>, Option<usize>, Option<usize>)> = match &node.node_type
            {
                RenderNodeType::Rectangle(r) => {
                    Some((r.section_index, r.para_index, r.control_index))
                }
                RenderNodeType::Ellipse(e) => {
                    Some((e.section_index, e.para_index, e.control_index))
                }
                RenderNodeType::Path(p) => Some((p.section_index, p.para_index, p.control_index)),
                _ => None,
            };
            if let Some((Some(si), Some(pi), Some(cidx))) = meta {
                if si == sec && pi == ppi && cidx == ci {
                    return Some(format!(
                        "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"width\":{:.1},\"height\":{:.1}}}",
                        page_idx,
                        node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height
                    ));
                }
            }
            for child in &node.children {
                if let Some(result) = find_shape_bbox(child, sec, ppi, ci, page_idx) {
                    return Some(result);
                }
            }
            None
        }

        let total_pages = self.page_count() as usize;
        for page_num in 0..total_pages {
            let tree = self.build_page_tree_cached(page_num as u32)?;
            if let Some(result) = find_shape_bbox(
                &tree.root,
                section_idx,
                parent_para_idx,
                control_idx,
                page_num,
            ) {
                return Ok(result);
            }
        }

        Err(HwpError::RenderError(format!(
            "글상자/도형 노드를 찾을 수 없습니다 (sec={}, ppi={}, ci={})",
            section_idx, parent_para_idx, control_idx
        )))
    }

    /// 표 컨트롤을 문단에서 삭제한다 (네이티브).
    ///
    /// 확장 컨트롤은 para.text에 포함되지 않고 char_offsets 간의 갭(8 code unit)에 배치된다.
    /// 컨트롤 제거 시 해당 갭을 닫기 위해 후속 char_offsets를 8씩 감소시킨다.
    pub fn delete_table_control_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        {
            let section = &mut self.document.sections[section_idx];
            if parent_para_idx >= section.paragraphs.len() {
                return Err(HwpError::RenderError(format!(
                    "부모 문단 인덱스 {} 범위 초과",
                    parent_para_idx
                )));
            }
            let para = &mut section.paragraphs[parent_para_idx];
            if control_idx >= para.controls.len() {
                return Err(HwpError::RenderError(format!(
                    "컨트롤 인덱스 {} 범위 초과",
                    control_idx
                )));
            }
            // 표 컨트롤인지 확인
            if !matches!(
                &para.controls[control_idx],
                crate::model::control::Control::Table(_)
            ) {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 표가 아닙니다".to_string(),
                ));
            }

            // 컨트롤이 차지하는 갭의 시작 위치를 찾아 char_offsets 조정
            // serialize_para_text와 동일한 로직으로 control_idx번째 컨트롤의 위치를 찾는다
            let text_chars: Vec<char> = para.text.chars().collect();
            let mut ci = 0usize;
            let mut prev_end: u32 = 0;
            let mut gap_start: Option<u32> = None;
            'outer: for i in 0..text_chars.len() {
                let offset = if i < para.char_offsets.len() {
                    para.char_offsets[i]
                } else {
                    prev_end
                };
                while prev_end + 8 <= offset && ci < para.controls.len() {
                    if ci == control_idx {
                        gap_start = Some(prev_end);
                        break 'outer;
                    }
                    ci += 1;
                    prev_end += 8;
                }
                // 문자 크기 산정
                let char_size: u32 = if text_chars[i] == '\t' {
                    8
                } else if text_chars[i].len_utf16() == 2 {
                    2
                } else {
                    1
                };
                prev_end = offset + char_size;
            }
            // 텍스트 뒤에 배치된 컨트롤 (남은 컨트롤)
            if gap_start.is_none() {
                while ci < para.controls.len() {
                    if ci == control_idx {
                        gap_start = Some(prev_end);
                        break;
                    }
                    ci += 1;
                    prev_end += 8;
                }
            }

            // char_offsets 조정: 컨트롤 이후의 모든 offset을 8 감소
            if let Some(gs) = gap_start {
                let threshold = gs + 8;
                for offset in para.char_offsets.iter_mut() {
                    if *offset >= threshold {
                        *offset -= 8;
                    }
                }
            }

            // 컨트롤 및 대응하는 ctrl_data_record 제거
            para.controls.remove(control_idx);
            if control_idx < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(control_idx);
            }

            // char_count 갱신 (확장 컨트롤 = 8 code unit)
            if para.char_count >= 8 {
                para.char_count -= 8;
            }

            section.raw_stream = None;
        }

        self.reflow_paragraph(section_idx, parent_para_idx);
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            parent_para_idx,
        );
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TableColumnDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 표 셀에서 계산식을 실행하고 결과를 반환한다.
    ///
    /// # Arguments
    /// * `section_idx` - 구역 인덱스
    /// * `parent_para_idx` - 표가 포함된 문단 인덱스
    /// * `control_idx` - 표 컨트롤 인덱스
    /// * `target_row` - 계산식이 입력될 셀 행 (0-based)
    /// * `target_col` - 계산식이 입력될 셀 열 (0-based)
    /// * `formula` - 계산식 문자열 (예: "=SUM(A1:A5)")
    /// * `write_result` - true이면 결과를 셀에 기록
    pub fn evaluate_table_formula(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        target_row: usize,
        target_col: usize,
        formula: &str,
        write_result: bool,
    ) -> Result<String, HwpError> {
        // 표 가져오기
        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 초과".into()))?;
        let para = section
            .paragraphs
            .get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 초과".into()))?;
        let table = match para.controls.get(control_idx) {
            Some(Control::Table(t)) => t,
            _ => return Err(HwpError::RenderError("표 컨트롤이 아님".into())),
        };

        let row_count = table.row_count as usize;
        let col_count = table.col_count as usize;

        // 셀 값 조회 함수: 셀의 첫 문단 텍스트를 숫자로 파싱
        let cells = &table.cells;
        let get_cell = |col: usize, row: usize| -> Option<f64> {
            let idx = row * col_count + col;
            cells
                .get(idx)
                .and_then(|cell| cell.paragraphs.first())
                .and_then(|p| parse_cell_number(&p.text))
        };

        let ctx = crate::document_core::table_calc::TableContext {
            row_count,
            col_count,
            current_row: target_row,
            current_col: target_col,
        };

        let result = crate::document_core::table_calc::evaluate_formula(formula, &ctx, &get_cell)
            .map_err(|e| HwpError::RenderError(format!("계산식 오류: {}", e)))?;

        // 결과를 셀에 기록
        if write_result {
            let cell_idx = target_row * col_count + target_col;
            let section_mut = self.document.sections.get_mut(section_idx).unwrap();
            let para_mut = section_mut.paragraphs.get_mut(parent_para_idx).unwrap();
            if let Some(Control::Table(ref mut t)) = para_mut.controls.get_mut(control_idx) {
                if let Some(cell) = t.cells.get_mut(cell_idx) {
                    if let Some(cell_para) = cell.paragraphs.first_mut() {
                        // 정수이면 정수로, 아니면 소수점 표시
                        let text = if result == result.trunc() && result.abs() < 1e15 {
                            format!("{}", result as i64)
                        } else {
                            format!("{}", result)
                        };
                        cell_para.text = text;
                        let new_len = cell_para.text.chars().count();
                        cell_para.char_offsets = (0..new_len).map(|i| i as u32).collect();
                    }
                }
            }
            // raw_stream 무효화
            if let Some(sec) = self.document.sections.get_mut(section_idx) {
                sec.raw_stream = None;
            }
            self.recompose_section(section_idx);
        }

        Ok(format!(
            "{{\"ok\":true,\"result\":{},\"formula\":{}}}",
            result,
            json_escape(formula)
        ))
    }
}

/// 셀 텍스트에서 숫자를 추출한다 (콤마 제거, 공백 무시).
fn parse_cell_number(text: &str) -> Option<f64> {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

fn json_escape(s: &str) -> String {
    let mut r = String::with_capacity(s.len() + 2);
    r.push('"');
    for c in s.chars() {
        match c {
            '"' => r.push_str("\\\""),
            '\\' => r.push_str("\\\\"),
            _ => r.push(c),
        }
    }
    r.push('"');
    r
}

#[cfg(test)]
mod tests {
    use crate::model::shape::common_obj_offsets;
    use crate::parser::control::parse_common_obj_attr;

    #[test]
    fn raw_ctrl_data_offsets_match_parser() {
        // CommonObjAttr layout: [0..4]=flags, [4..8]=v_offset, [8..12]=h_offset, [12..16]=width
        let mut data = vec![0u8; 36];
        let flags: u32 = (2 << 3) | (3 << 8) | (1 << 21); // vert=Para, horz=Para, wrap=TopAndBottom
        data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
        data[common_obj_offsets::V_OFFSET].copy_from_slice(&42_u32.to_le_bytes());
        data[common_obj_offsets::H_OFFSET].copy_from_slice(&99_u32.to_le_bytes());
        data[common_obj_offsets::WIDTH].copy_from_slice(&5000_u32.to_le_bytes());
        data[common_obj_offsets::HEIGHT].copy_from_slice(&3000_u32.to_le_bytes());

        assert_eq!(
            common_obj_offsets::MIN_LEN,
            common_obj_offsets::INSTANCE_ID.end
        );
        assert_eq!(
            common_obj_offsets::MIN_LEN_WITH_PREVENT_PAGE_BREAK,
            common_obj_offsets::PREVENT_PAGE_BREAK.end
        );

        let common = parse_common_obj_attr(&data);
        assert_eq!(
            common.vertical_offset, 42,
            "v_offset must be at bytes [4..8]"
        );
        assert_eq!(
            common.horizontal_offset, 99,
            "h_offset must be at bytes [8..12]"
        );
        assert_eq!(common.width, 5000);
        assert_eq!(common.height, 3000);
    }

    #[test]
    fn update_ctrl_dimensions_writes_correct_slots() {
        use crate::model::table::{Cell, Table};

        let mut tbl = Table::default();
        tbl.col_count = 2;
        tbl.row_count = 1;
        tbl.cells = vec![
            Cell {
                row: 0,
                col: 0,
                col_span: 1,
                row_span: 1,
                width: 5000,
                height: 3000,
                ..Default::default()
            },
            Cell {
                row: 0,
                col: 1,
                col_span: 1,
                row_span: 1,
                width: 4000,
                height: 3000,
                ..Default::default()
            },
        ];
        tbl.raw_ctrl_data = vec![0u8; 36];

        tbl.update_ctrl_dimensions();

        let common = parse_common_obj_attr(&tbl.raw_ctrl_data);
        assert_eq!(common.width, 9000, "width at [12..16]");
        assert_eq!(common.height, 3000, "height at [16..20]");
        assert_eq!(common.horizontal_offset, 0, "h_offset at [8..12] untouched");
    }
}

#[cfg(test)]
mod table_attr_save_roundtrip_tests {
    //! 표 배치 속성(attr 비트) 변경의 HWP5 저장 유실 회귀 테스트.
    //!
    //! set_table_properties_native 는 글자처럼 취급/배치/기준/정렬/제한/겹침을
    //! table.attr/common 에만 반영하고 raw_ctrl_data FLAGS(0..4)를 패치하지
    //! 않았다. HWP5 직렬화기는 raw_ctrl_data 를 그대로 기록하므로(HWP5 에서
    //! 파싱된 표는 raw 가 항상 보존됨) 변경이 저장 파일에서 통째로 유실되고
    //! 재로드 시 원복됐다. 화면(getter)은 common 필드로 정상 표시되어
    //! "소리 없는" 유실이었다.

    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::shape::{TextWrap, VertRelTo};

    const SAMPLE: &str = "samples/calc-cell.hwp";

    fn load() -> DocumentCore {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
        DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {SAMPLE}: {e}"))
    }

    fn find_first_table(core: &DocumentCore) -> (usize, usize) {
        for (pi, para) in core.document().sections[0].paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if matches!(ctrl, Control::Table(_)) {
                    return (pi, ci);
                }
            }
        }
        panic!("{SAMPLE}: 표 컨트롤이 필요함");
    }

    fn table_attrs(core: &DocumentCore, pi: usize, ci: usize) -> (bool, TextWrap, bool, VertRelTo) {
        match &core.document().sections[0].paragraphs[pi].controls[ci] {
            Control::Table(t) => (
                t.common.treat_as_char,
                t.common.text_wrap,
                t.common.allow_overlap,
                t.common.vert_rel_to,
            ),
            _ => unreachable!(),
        }
    }

    #[test]
    fn table_attr_changes_survive_hwp_save_roundtrip() {
        let mut core = load();
        let (pi, ci) = find_first_table(&core);
        let (orig_tac, orig_wrap, _, _) = table_attrs(&core, pi, ci);

        // 파싱 원본과 반드시 달라지는 값으로 변경
        let new_tac = !orig_tac;
        let new_wrap = if matches!(orig_wrap, TextWrap::TopAndBottom) {
            "Square"
        } else {
            "TopAndBottom"
        };
        let json = format!(
            r#"{{"treatAsChar":{new_tac},"textWrap":"{new_wrap}","vertRelTo":"Para","allowOverlap":true}}"#
        );
        core.set_table_properties_native(0, pi, ci, &json)
            .expect("set_table_properties_native");

        // 메모리(IR) 반영 확인
        let (mem_tac, mem_wrap, mem_overlap, mem_vrel) = table_attrs(&core, pi, ci);
        assert_eq!(mem_tac, new_tac);
        assert_eq!(format!("{mem_wrap:?}"), new_wrap);
        assert!(mem_overlap);
        assert!(matches!(mem_vrel, VertRelTo::Para));

        // HWP5 저장 → 재로드 후에도 보존되어야 한다.
        // (수정 전에는 raw_ctrl_data FLAGS 가 파싱 원본 그대로 기록되어 전부 원복)
        let saved = core.export_hwp_with_adapter().expect("export_hwp");
        let reloaded = DocumentCore::from_bytes(&saved).expect("재로드");
        let (pi2, ci2) = find_first_table(&reloaded);
        let (tac, wrap, overlap, vrel) = table_attrs(&reloaded, pi2, ci2);
        assert_eq!(tac, new_tac, "treatAsChar 변경이 HWP5 저장에서 유실됨");
        assert_eq!(
            format!("{wrap:?}"),
            new_wrap,
            "textWrap 변경이 HWP5 저장에서 유실됨"
        );
        assert!(overlap, "allowOverlap 변경이 HWP5 저장에서 유실됨");
        assert!(
            matches!(vrel, VertRelTo::Para),
            "vertRelTo 변경이 HWP5 저장에서 유실됨 (실제: {vrel:?})"
        );
    }
}
