//! 표/셀 CRUD + 속성 조회·수정 관련 native 메서드

use super::super::helpers::{border_line_type_to_u8_val, color_ref_to_css, navigate_path_to_table};
use crate::document_core::DocumentCore;
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
                "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
            ).to_string();
        }
        let bf = self
            .document
            .doc_info
            .border_fills
            .get((bf_id - 1) as usize);
        match bf {
            Some(bf) => {
                use crate::model::style::FillType;
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
                format!(
                    "\"borderFillId\":{},{},\"fillType\":\"{}\",\"fillColor\":\"{}\",\"patternColor\":\"{}\",\"patternType\":{}",
                    bf_id,
                    borders_json.join(","),
                    fill_type_str, fill_color, pat_color, pat_type,
                )
            }
            None => {
                concat!(
                    "\"borderFillId\":0,",
                    "\"borderLeft\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderRight\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderTop\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"borderBottom\":{\"type\":0,\"width\":0,\"color\":\"#000000\"},",
                    "\"fillType\":\"none\",\"fillColor\":\"#ffffff\",\"patternColor\":\"#000000\",\"patternType\":0"
                ).to_string()
            }
        }
    }

    pub(crate) fn get_cell_properties_native(
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

        let cell = table
            .cells
            .get(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx)))?;

        let va = match cell.vertical_align {
            crate::model::table::VerticalAlign::Top => 0,
            crate::model::table::VerticalAlign::Center => 1,
            crate::model::table::VerticalAlign::Bottom => 2,
        };

        let bf_json = self.build_border_fill_json_by_id(cell.border_fill_id);

        // 셀 보호 (list_header_width_ref bit 1)
        let cell_protect = (cell.list_header_width_ref & 0x02) != 0;

        Ok(format!(
            "{{\"width\":{},\"height\":{},\"paddingLeft\":{},\"paddingRight\":{},\"paddingTop\":{},\"paddingBottom\":{},\"verticalAlign\":{},\"textDirection\":{},\"isHeader\":{},\"cellProtect\":{},{}}}",
            cell.width, cell.height,
            cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
            va, cell.text_direction, cell.is_header, cell_protect,
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
        use super::super::helpers::{json_bool, json_i16, json_u32, json_u8};

        let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
        let cell = table
            .cells
            .get_mut(cell_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx)))?;

        if let Some(v) = json_u32(json, "width") {
            cell.width = v;
        }
        if let Some(v) = json_u32(json, "height") {
            cell.height = v;
        }
        if let Some(v) = json_i16(json, "paddingLeft") {
            cell.padding.left = v;
        }
        if let Some(v) = json_i16(json, "paddingRight") {
            cell.padding.right = v;
        }
        if let Some(v) = json_i16(json, "paddingTop") {
            cell.padding.top = v;
        }
        if let Some(v) = json_i16(json, "paddingBottom") {
            cell.padding.bottom = v;
        }
        if let Some(v) = json_u8(json, "verticalAlign") {
            cell.vertical_align = match v {
                1 => crate::model::table::VerticalAlign::Center,
                2 => crate::model::table::VerticalAlign::Bottom,
                _ => crate::model::table::VerticalAlign::Top,
            };
        }
        if let Some(v) = json_u8(json, "textDirection") {
            cell.text_direction = v;
        }
        if let Some(v) = json_bool(json, "isHeader") {
            cell.is_header = v;
            if v {
                cell.list_header_width_ref |= 0x04;
            } else {
                cell.list_header_width_ref &= !0x04;
            }
        }
        if let Some(v) = json_bool(json, "cellProtect") {
            if v {
                cell.list_header_width_ref |= 0x02;
            } else {
                cell.list_header_width_ref &= !0x02;
            }
        }

        // BorderFill 변경: borderLeft 등이 포함된 경우 create_border_fill_from_json으로 처리
        let has_border = json.contains("\"borderLeft\"");
        if has_border {
            let new_bf_id = self.create_border_fill_from_json(json);

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
                let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
                    HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", cell_idx))
                })?;
                cell.border_fill_id = new_bf_id;
                (
                    cell.row as usize,
                    cell.col as usize,
                    cell.col_span as usize,
                    cell.row_span as usize,
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
        }
        let mut updates: Vec<CellUpdate> = Vec::new();

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
                        updates.push(CellUpdate {
                            cell_idx: cell_idx as usize,
                            width_delta,
                            height_delta,
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
        for upd in &updates {
            if let Some(cell) = table.cells.get_mut(upd.cell_idx) {
                if upd.width_delta != 0 {
                    let new_w =
                        (cell.width as i32 + upd.width_delta).max(MIN_CELL_SIZE as i32) as u32;
                    cell.width = new_w;
                }
                if upd.height_delta != 0 {
                    let new_h =
                        (cell.height as i32 + upd.height_delta).max(MIN_CELL_SIZE as i32) as u32;
                    cell.height = new_h;
                }
            }
        }
        table.update_ctrl_dimensions();
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
        }
        if let Some(v) = json_str(json, "vertRelTo") {
            let bits: u32 = match v.as_str() {
                "Paper" => 0,
                "Page" => 1,
                "Para" => 2,
                _ => 0,
            };
            table.attr = (table.attr & !(0x03 << 3)) | (bits << 3);
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
        }
        // 위치 오프셋: CommonObjAttr [0..4]=flags, [4..8]=v_offset, [8..12]=h_offset
        while table.raw_ctrl_data.len() < common_obj_offsets::H_OFFSET.end {
            table.raw_ctrl_data.push(0);
        }
        if let Some(v) = json_i32(json, "vertOffset") {
            table.raw_ctrl_data[common_obj_offsets::V_OFFSET].copy_from_slice(&v.to_le_bytes());
        }
        if let Some(v) = json_i32(json, "horzOffset") {
            table.raw_ctrl_data[common_obj_offsets::H_OFFSET].copy_from_slice(&v.to_le_bytes());
        }
        // restrictInPage → attr bit 13
        if let Some(v) = json_bool(json, "restrictInPage") {
            if v {
                table.attr |= 1 << 13;
            } else {
                table.attr &= !(1 << 13);
            }
        }
        // allowOverlap → attr bit 14
        if let Some(v) = json_bool(json, "allowOverlap") {
            if v {
                table.attr |= 1 << 14;
            } else {
                table.attr &= !(1 << 14);
            }
        }
        // keepWithAnchor → prevent_page_break
        // CommonObjAttr::PREVENT_PAGE_BREAK (parse_common_obj_attr 정합)
        if let Some(v) = json_bool(json, "keepWithAnchor") {
            while table.raw_ctrl_data.len() < common_obj_offsets::PREVENT_PAGE_BREAK.end {
                table.raw_ctrl_data.push(0);
            }
            let val: i32 = if v { 1 } else { 0 };
            table.raw_ctrl_data[common_obj_offsets::PREVENT_PAGE_BREAK]
                .copy_from_slice(&val.to_le_bytes());
        }

        // 바깥 여백 (CommonObjAttr margin ranges, parse_common_obj_attr 정합)
        if table.raw_ctrl_data.len() >= common_obj_offsets::MARGIN_BOTTOM.end {
            if let Some(v) = json_i16(json, "outerLeft") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_LEFT]
                    .copy_from_slice(&v.to_le_bytes());
            }
            if let Some(v) = json_i16(json, "outerRight") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
                    .copy_from_slice(&v.to_le_bytes());
            }
            if let Some(v) = json_i16(json, "outerTop") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_TOP]
                    .copy_from_slice(&v.to_le_bytes());
            }
            if let Some(v) = json_i16(json, "outerBottom") {
                table.raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
                    .copy_from_slice(&v.to_le_bytes());
            }
        }

        // 캡션 생성/수정
        let mut caption_created = false;
        if let Some(has_cap) = json_bool(json, "hasCaption") {
            if has_cap && table.caption.is_none() {
                let mut cap = crate::model::shape::Caption::default();
                let an = crate::model::control::AutoNumber {
                    number_type: crate::model::control::AutoNumberType::Table,
                    ..Default::default()
                };
                let mut cap_para = crate::model::paragraph::Paragraph::new_empty();
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
                table.caption.as_mut().unwrap().paragraphs[0]
                    .controls
                    .push(crate::model::control::Control::AutoNumber(an));
                // attr bit 29: 캡션 존재 플래그 (한컴 호환성)
                table.attr |= 1 << 29;
            }
        }
        // 캡션 속성 수정
        let mut caption_changed = false;
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

        // BorderFill 변경 — 표 테두리 변경 시 모든 셀에도 동일 적용
        // (HWP 렌더링은 cell.border_fill_id를 사용, table.border_fill_id는 페이지 분할용)
        let has_border = json.contains("\"borderLeft\"");
        if has_border {
            let new_bf_id = self.create_border_fill_from_json(json);
            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            table.border_fill_id = new_bf_id;
            for cell in &mut table.cells {
                cell.border_fill_id = new_bf_id;
            }
            table.dirty = true;
        }

        // 캡션 생성 시 AutoNumber 번호 확정 + 텍스트에 직접 삽입
        if caption_created {
            crate::parser::assign_auto_numbers(&mut self.document);
            // AutoNumber에서 할당된 번호를 가져와 텍스트에 직접 포함
            // (모델 텍스트와 렌더링 텍스트를 일치시켜 캐럿 위치 정확성 보장)
            let assigned_num = {
                let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
                let para = &table.caption.as_ref().unwrap().paragraphs[0];
                para.controls
                    .iter()
                    .find_map(|c| {
                        if let crate::model::control::Control::AutoNumber(an) = c {
                            Some(an.assigned_number)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(1)
            };
            let num_str = format!("{}", assigned_num);
            // "표 N " 형태의 텍스트 직접 생성 (AutoNumber 치환 불필요)
            let caption_text = format!("표 {} ", num_str);
            let char_count_text: u32 = caption_text.chars().count() as u32;

            let table = self.get_table_mut(section_idx, parent_para_idx, control_idx)?;
            let cap_max_width = table.caption.as_ref().map(|c| c.max_width).unwrap_or(0);
            let para = &mut table.caption.as_mut().unwrap().paragraphs[0];
            para.text = caption_text;
            // char_offsets: 순차적 (AutoNumber 갭 없음, 모델=렌더링 일치)
            para.char_offsets = (0..char_count_text).collect();
            para.char_count = char_count_text + 1; // 텍스트 + 끝마커
                                                   // AutoNumber 컨트롤 제거 (번호가 텍스트에 직접 포함됨)
            para.controls.clear();
            // char_shapes: 전체 텍스트에 기본 스타일(0) 적용
            para.char_shapes = vec![crate::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }];
            // line_segs의 segment_width를 표 폭으로 설정
            if let Some(ls) = para.line_segs.first_mut() {
                ls.segment_width = cap_max_width as i32;
            }
            // 직접 접근으로 borrow 분리하여 reflow_line_segs 호출
            if let Some(crate::model::control::Control::Table(ref mut tbl)) =
                self.document.sections[section_idx].paragraphs[parent_para_idx]
                    .controls
                    .get_mut(control_idx)
            {
                if let Some(ref mut cap) = tbl.caption {
                    let available_width_px =
                        crate::renderer::hwpunit_to_px(cap.max_width as i32, self.dpi);
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
