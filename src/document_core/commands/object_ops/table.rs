//! 표/셀 관련 native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
    /// [Task #1151 v7] cell_path JSON → Vec<(controlIdx, cellIdx, cellParaIdx)>.
    /// 4 개 by_path setter/getter (cell picture/shape × set/get) 의 공통 파싱.
    /// 빈 path 면 Err 반환.
    fn parse_cell_path_json(json: &str) -> Result<Vec<(usize, usize, usize)>, HwpError> {
        let path: Vec<(usize, usize, usize)> = serde_json::from_str::<Vec<serde_json::Value>>(json)
            .map_err(|e| HwpError::RenderError(format!("cell_path JSON 파싱 실패: {}", e)))?
            .iter()
            .map(|v| {
                let c = v
                    .get("controlIdx")
                    .or_else(|| v.get("controlIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                let ci = v
                    .get("cellIdx")
                    .or_else(|| v.get("cellIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                let cpi = v
                    .get("cellParaIdx")
                    .or_else(|| v.get("cellParaIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                (c, ci, cpi)
            })
            .collect();
        if path.is_empty() {
            return Err(HwpError::RenderError(
                "cell_path 가 비어있습니다".to_string(),
            ));
        }
        Ok(path)
    }
    /// [Task #1151 v7] section + parent_para_idx + path → target paragraph (mut).
    /// 2 개 set_cell_*_by_path_native (Picture / Shape) 의 공통 traversal.
    /// immutable 버전은 cursor_nav.rs 의 `resolve_paragraph_by_path` 가 담당하며,
    /// [Task #1171] 이후 표 셀과 글상자(Shape text_box, cell_index=0 sentinel) 를 모두
    /// 처리하도록 immutable 짝과 동일하게 맞춘다.
    pub(crate) fn resolve_cell_paragraph_mut<'a>(
        section: &'a mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&'a mut crate::model::paragraph::Paragraph, HwpError> {
        let mut current_para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let ctrl = current_para.controls.get_mut(ctrl_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: controls[{}] 범위 초과", i, ctrl_idx))
            })?;
            current_para = match ctrl {
                crate::model::control::Control::Table(t) => {
                    let cell = t.cells.get_mut(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("경로[{}]: cells[{}] 범위 초과", i, cell_idx))
                    })?;
                    cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: paragraphs[{}] 범위 초과",
                            i, cell_para_idx
                        ))
                    })?
                }
                // [Task #1171] 글상자(Shape text_box) — cell_index=0 sentinel.
                crate::model::control::Control::Shape(shape) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let text_box = get_textbox_from_shape_mut(shape).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                            i, ctrl_idx
                        ))
                    })?;
                    text_box.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과",
                            i, cell_para_idx
                        ))
                    })?
                }
                crate::model::control::Control::Picture(pic) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 그림 캡션의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let caption = pic.caption.as_mut().ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}] 그림에 캡션이 없습니다",
                            i, ctrl_idx
                        ))
                    })?;
                    caption.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: 그림 캡션문단 {} 범위 초과",
                            i, cell_para_idx
                        ))
                    })?
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}] 가 표/글상자/그림 캡션이 아닙니다",
                        i, ctrl_idx
                    )))
                }
            };
        }
        Ok(current_para)
    }
    fn required_cell_height_for_picture(
        cell: &crate::model::table::Cell,
        pic: &crate::model::image::Picture,
    ) -> u32 {
        Self::required_cell_height_for_picture_padding(cell.padding.top, cell.padding.bottom, pic)
    }
    fn required_cell_height_for_picture_padding(
        padding_top: i16,
        padding_bottom: i16,
        pic: &crate::model::image::Picture,
    ) -> u32 {
        let vert_offset = (pic.common.vertical_offset as i32).max(0) as u32;
        let visual_height = if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
            && pic.shape_attr.current_width > 0
            && pic.shape_attr.current_height > 0
        {
            pic.common.height
        } else {
            let (_, height) = Self::picture_rotated_bounds(
                pic.common.width,
                pic.common.height,
                pic.shape_attr.rotation_angle,
            );
            height
        };
        vert_offset
            .saturating_add(visual_height)
            .saturating_add(padding_top.max(0) as u32)
            .saturating_add(padding_bottom.max(0) as u32)
    }
    fn sync_direct_owner_cell_for_picture(
        section: &mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        inner_control_idx: usize,
    ) -> Result<(), HwpError> {
        if path.len() != 1 {
            return Ok(());
        }

        let (table_ctrl_idx, cell_idx, cell_para_idx) = path[0];
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let existing_line_height = para
            .line_segs
            .first()
            .map(|seg| seg.line_height)
            .unwrap_or(0);
        let table = match para.controls.get_mut(table_ctrl_idx) {
            Some(Control::Table(table)) => table,
            _ => return Ok(()),
        };
        let line_height_extra = (existing_line_height - table.common.height as i32).max(0);
        let mut line_seg_update: Option<(i32, i32)> = None;

        let required_height = {
            let cell = table.cells.get(cell_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[0]: cells[{}] 범위 초과", cell_idx))
            })?;
            let cell_para = cell.paragraphs.get(cell_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[0]: paragraphs[{}] 범위 초과", cell_para_idx))
            })?;
            let pic = match cell_para.controls.get(inner_control_idx) {
                Some(Control::Picture(pic)) => pic,
                _ => return Ok(()),
            };
            let take_place_flow_offset = Self::take_place_picture_flow_offset(pic);
            if table.common.treat_as_char {
                if let Some(flow_offset) = take_place_flow_offset {
                    let vertical_pos = if pic.common.flow_with_text {
                        0
                    } else {
                        flow_offset
                    };
                    line_seg_update = Some((vertical_pos, line_height_extra));
                }
            }
            if pic.common.flow_with_text {
                Some(Self::required_cell_height_for_picture(cell, pic))
            } else {
                None
            }
        };

        if let (Some(required_height), Some(cell)) =
            (required_height, table.cells.get_mut(cell_idx))
        {
            let synced_height = required_height.max(MIN_SHAPE_SIZE);
            if cell.height != synced_height {
                cell.height = synced_height;
            }
        }
        table.update_ctrl_dimensions();
        table.dirty = true;
        let new_table_height = table.common.height as i32;
        if let Some((vertical_pos, line_height_extra)) = line_seg_update {
            if let Some(seg) = para.line_segs.first_mut() {
                let line_height = new_table_height
                    .saturating_add(line_height_extra)
                    .max(MIN_SHAPE_SIZE as i32);
                seg.vertical_pos = vertical_pos;
                seg.line_height = line_height;
                seg.text_height = line_height;
                seg.baseline_distance =
                    ((line_height as i64 * 17 + 10) / 20).min(i32::MAX as i64) as i32;
            }
        }
        Ok(())
    }
    fn clamp_direct_owner_cell_picture_offsets(
        section: &mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        inner_control_idx: usize,
        clamp_horz: bool,
        clamp_vert: bool,
    ) -> Result<(), HwpError> {
        if path.len() != 1 || (!clamp_horz && !clamp_vert) {
            return Ok(());
        }

        let (table_ctrl_idx, cell_idx, cell_para_idx) = path[0];
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let table = match para.controls.get_mut(table_ctrl_idx) {
            Some(Control::Table(table)) => table,
            _ => return Ok(()),
        };
        let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
            HwpError::RenderError(format!("경로[0]: cells[{}] 범위 초과", cell_idx))
        })?;

        let inner_width = cell
            .width
            .saturating_sub(cell.padding.left.max(0) as u32)
            .saturating_sub(cell.padding.right.max(0) as u32) as i64;
        let cell_para = cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("경로[0]: paragraphs[{}] 범위 초과", cell_para_idx))
        })?;
        let pic = match cell_para.controls.get_mut(inner_control_idx) {
            Some(Control::Picture(pic)) => pic,
            _ => return Ok(()),
        };

        if !pic.common.flow_with_text {
            return Ok(());
        }

        if clamp_horz {
            let max_horz = (inner_width - pic.common.width as i64)
                .max(0)
                .min(i32::MAX as i64);
            let horz = (pic.common.horizontal_offset as i32).clamp(0, max_horz as i32);
            pic.common.horizontal_offset = horz as u32;
        }
        if clamp_vert {
            let vert = (pic.common.vertical_offset as i32).max(0);
            pic.common.vertical_offset = vert as u32;
        }
        Ok(())
    }
    /// path 의 마지막 엔트리가 글상자(Shape text_box)를 가리키는지 판정한다.
    ///
    /// 표 셀 picture 삽입은 한컴 정합상 parent paragraph 의 sibling floating
    /// picture 로 처리하지만, 글상자 내부 picture 는 text_box paragraph 안에
    /// 실제 Picture control 로 들어가야 한다. `resolve_cell_by_path` 는 마지막
    /// 엔트리가 표일 때만 성공하므로, insert path 에서는 표/글상자를 먼저 구분한다.
    pub(crate) fn cell_path_terminates_at_textbox(
        section: &crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<bool, HwpError> {
        let mut current_para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let ctrl = current_para.controls.get(ctrl_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: controls[{}] 범위 초과", i, ctrl_idx))
            })?;
            match ctrl {
                crate::model::control::Control::Table(table) => {
                    let cell = table.cells.get(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("경로[{}]: cells[{}] 범위 초과", i, cell_idx))
                    })?;
                    if i == path.len() - 1 {
                        return Ok(false);
                    }
                    current_para = cell.paragraphs.get(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: paragraphs[{}] 범위 초과",
                            i, cell_para_idx
                        ))
                    })?;
                }
                crate::model::control::Control::Shape(shape) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let text_box = get_textbox_from_shape(shape.as_ref()).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                            i, ctrl_idx
                        ))
                    })?;
                    if i == path.len() - 1 {
                        return Ok(true);
                    }
                    current_para = text_box.paragraphs.get(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과",
                            i, cell_para_idx
                        ))
                    })?;
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}] 가 표/글상자가 아닙니다",
                        i, ctrl_idx
                    )))
                }
            }
        }

        Err(HwpError::RenderError("경로가 비어있습니다".to_string()))
    }
    /// 커서 위치에 새 표를 삽입한다 (네이티브).
    ///
    /// 1. PageDef에서 편집 영역 폭 계산
    /// 2. 균등 열 폭으로 row_count × col_count 셀 생성
    /// 3. Table + Paragraph 조립
    /// 4. 커서 위치에 삽입 (빈 문단이면 교체, 아니면 분할 후 삽입)
    /// 5. 표 아래에 빈 문단 추가 (HWP 표준)
    pub fn create_table_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        row_count: u16,
        col_count: u16,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{
            BorderFill, BorderLine, BorderLineType, CenterLine, DiagonalLine, Fill,
        };
        use crate::model::table::{Cell, Table, TablePageBreak};

        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={}, 열은 1~256)",
                row_count, col_count
            )));
        }

        // --- 1. 편집 영역 폭 계산 ---
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin_lr: i32 = 283 * 2; // outer_margin left + right (~2mm)
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr)
                .max(7200) as u32;

        // --- 2. 한컴 기본값 기반 셀 생성 (blank_h_saved.hwp 참조) ---
        let col_width = content_width / col_count as u32;
        // 한컴 기본: 셀 패딩 L=510 R=510 T=141 B=141
        let cell_pad = crate::model::Padding {
            left: 510,
            right: 510,
            top: 141,
            bottom: 141,
        };
        // 한컴 기본: 셀 높이 = top + bottom padding (빈 셀 최소 높이)
        let cell_height: u32 = (cell_pad.top + cell_pad.bottom) as u32;
        // 한컴 기본: 행 렌더링 높이 = padding_top + line_height(1000) + padding_bottom
        let rendered_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let total_width = col_width * col_count as u32;
        let total_height = rendered_row_height * row_count as u32;

        // BorderFill: 실선 테두리가 있는 기존 항목 재사용, 없으면 새로 생성
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders
                    .iter()
                    .all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16 // 1-based
            } else {
                // 실선 BorderFill이 없으면 새로 생성
                let solid_border = BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                };
                let new_bf = BorderFill {
                    raw_data: None,
                    attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine {
                        diagonal_type: 1,
                        width: 0,
                        color: 0,
                    },
                    center_line: CenterLine::None,
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16 // 1-based
            }
        };

        // 커서 위치 문단의 속성을 기본값으로 상속 (한컴 동작 일치).
        // 혼합 글자모양 문단에서는 첫 엔트리가 아니라 커서 offset 의 글자모양이 기준이다.
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shape_id_at(char_offset).unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 셀 목록 생성
        let mut cells = Vec::with_capacity((row_count as usize) * (col_count as usize));
        for r in 0..row_count {
            for c in 0..col_count {
                let mut cell = Cell::new_empty(c, r, col_width, cell_height, cell_border_fill_id);
                cell.padding = cell_pad;
                cell.vertical_align = crate::model::table::VerticalAlign::Center; // 한컴 기본값
                                                                                  // 셀 문단 보정: char_count_msb, raw_header_extra, para/char shape
                for cp in &mut cell.paragraphs {
                    cp.char_count_msb = true;
                    cp.para_shape_id = default_para_shape_id;
                    // Cell::new_empty() 의 문단은 char_shapes 가 비어 있고, 저장기는 그것을
                    // charPrIDRef="0" 으로 쓴다. 아래 raw_header_extra 가 n_char_shapes=1 을
                    // 주장하는 것과도 어긋난다. 표를 삽입한 문단의 글자모양을 상속한다.
                    cp.char_shapes = vec![CharShapeRef {
                        start_pos: 0,
                        char_shape_id: default_char_shape_id,
                    }];
                    if cp.raw_header_extra.len() < 10 {
                        let mut rhe = vec![0u8; 10];
                        rhe[0..2].copy_from_slice(&1u16.to_le_bytes()); // n_char_shapes=1
                        rhe[4..6].copy_from_slice(&1u16.to_le_bytes()); // n_line_segs=1
                        cp.raw_header_extra = rhe;
                    }
                    // line_segs 보정: new_empty()의 기본 LineSeg는 line_height=0이므로 항상 교체
                    let seg_w = (col_width as i32) - 141 - 141; // 셀 폭 - 좌우 패딩
                    cp.line_segs = vec![LineSeg {
                        text_start: 0,
                        line_height: 1000,
                        text_height: 1000,
                        baseline_distance: 850,
                        line_spacing: 600,
                        segment_width: seg_w,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }];
                }
                // raw_list_extra: 빈 벡터 (cell.width 필드가 LIST_HEADER에 직렬화됨)
                cell.raw_list_extra = Vec::new();
                cells.push(cell);
            }
        }

        // --- 3. Table 구조체 조립 (한컴 기본 속성값) ---
        let row_sizes: Vec<i16> = (0..row_count).map(|_| col_count as i16).collect();

        // raw_ctrl_data: CommonObjAttr 바이너리 (파서 호환)
        // 바이트 레이아웃: flags(4) + v_offset(4) + h_offset(4) + width(4) + height(4)
        //                 + z_order(4) + margin_l(2) + margin_r(2) + margin_t(2) + margin_b(2)
        //                 + instance_id(4) = 36바이트 (+ 여유 2바이트 = 38)
        // vert=Para(2), horz=Para(3), wrap=TopAndBottom(1)
        // width_criterion=Absolute(4), height_criterion=Absolute(2)
        let flags: u32 = (2 << 3) | (3 << 8) | (4 << 15) | (2 << 18) | (1 << 21);
        let outer_margin: i16 = 283; // ~1mm
        let mut raw_ctrl_data = vec![0u8; 38];
        raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
        // vertical_offset/horizontal_offset/z_order = 0
        raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
            .copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
            .copy_from_slice(&outer_margin.to_le_bytes());
        // instance_id (해시 기반, 비-0 필수)
        let instance_id: u32 = {
            let mut h: u32 = 0x7c150000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            h = h.wrapping_add(total_height.wrapping_mul(0x1b));
            if h == 0 {
                h = 0x7c154b69;
            }
            h
        };
        raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x082A2210, // 한컴 기본값 (blank_h_saved.hwp)
            row_count,
            col_count,
            cell_spacing: 0,
            padding: crate::model::Padding {
                left: 510,
                right: 510,
                top: 141,
                bottom: 141,
            },
            row_sizes,
            border_fill_id: cell_border_fill_id, // 한컴: 표와 셀이 같은 BorderFill 사용
            zones: Vec::new(),
            cells,
            cell_grid: Vec::new(),
            page_break: TablePageBreak::None,
            repeat_header: false,
            caption: None,
            common: crate::model::shape::CommonObjAttr {
                treat_as_char: false,
                text_wrap: crate::model::shape::TextWrap::TopAndBottom,
                vert_rel_to: crate::model::shape::VertRelTo::Para,
                horz_rel_to: crate::model::shape::HorzRelTo::Para,
                vert_align: crate::model::shape::VertAlign::Top,
                horz_align: crate::model::shape::HorzAlign::Left,
                width: total_width,
                height: total_height,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            raw_ctrl_data,
            raw_table_record_attr: 0x00000006, // 한컴 기본값 (bit1=셀분리금지, bit2=repeat_header)
            raw_table_record_extra: vec![0u8; 2],
            dirty: true,
            local_resize_rows: Vec::new(),
            local_resize_cols: Vec::new(),
            local_resize_cell_widths: Vec::new(),
            local_resize_cell_heights: Vec::new(),
        };
        table.rebuild_grid();

        // --- 4. Table을 포함하는 Paragraph 생성 ---
        // para_shape_id: 커서 위치 문단의 값 상속 (한컴 동작 일치)
        let table_para_shape_id = default_para_shape_id;

        let mut table_raw_header_extra = vec![0u8; 10];
        table_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
        table_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());

        let table_para = Paragraph {
            text: String::new(),
            char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1)
            control_mask: 0x00000800,
            char_offsets: vec![],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 0, // 한컴 표준: 표 문단의 segment_width는 0
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            para_shape_id: table_para_shape_id,
            style_id: 0,
            controls: vec![Control::Table(Box::new(table))],
            ctrl_data_records: vec![None],
            has_para_text: true,
            raw_header_extra: table_raw_header_extra,
            char_count_msb: false,
            ..Default::default()
        };

        let make_empty_neighbor_para = || {
            let mut empty_raw_header_extra = vec![0u8; 10];
            empty_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
            empty_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());
            Paragraph {
                text: String::new(),
                char_count: 1,
                char_count_msb: false,
                control_mask: 0,
                para_shape_id: default_para_shape_id,
                style_id: 0,
                char_shapes: vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: default_char_shape_id,
                }],
                line_segs: vec![LineSeg {
                    text_start: 0,
                    line_height: 1000,
                    text_height: 1000,
                    baseline_distance: 850,
                    line_spacing: 600,
                    segment_width: content_width as i32,
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    ..Default::default()
                }],
                has_para_text: false,
                raw_header_extra: empty_raw_header_extra,
                ..Default::default()
            }
        };

        // --- 5. 커서 위치에 삽입 ---
        self.document.sections[section_idx].raw_stream = None;

        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let is_empty_para = para.text.is_empty() && para.controls.is_empty();
        let is_structure_only_empty_para = Self::is_structure_only_empty_paragraph(para);

        let insert_para_idx;
        let table_control_idx;
        if is_empty_para {
            // 빈 문단이면 UI에서 넘어온 offset과 무관하게 현재 줄을 표 host로 사용한다.
            self.document.sections[section_idx].paragraphs[para_idx] = table_para;
            insert_para_idx = para_idx;
            table_control_idx = 0;
        } else if is_structure_only_empty_para {
            // blank2010 첫 문단처럼 SectionDef/ColumnDef만 있는 빈 줄은 구조 컨트롤을
            // 보존하되, 줄 배치는 표 host 문단 기준으로 교체해 표 위 빈 줄을 만들지 않는다.
            let old_para = self.document.sections[section_idx].paragraphs[para_idx].clone();
            let mut merged_para = table_para;
            let table_control = merged_para
                .controls
                .pop()
                .ok_or_else(|| HwpError::RenderError("표 컨트롤 생성 실패".to_string()))?;
            let table_ctrl_data = merged_para.ctrl_data_records.pop().unwrap_or(None);

            merged_para.controls = old_para.controls;
            merged_para.ctrl_data_records = old_para.ctrl_data_records;
            while merged_para.ctrl_data_records.len() < merged_para.controls.len() {
                merged_para.ctrl_data_records.push(None);
            }
            table_control_idx = merged_para.controls.len();
            merged_para.controls.push(table_control);
            merged_para.ctrl_data_records.push(table_ctrl_data);
            merged_para.char_count = merged_para.controls.len() as u32 * 8 + 1;
            merged_para.control_mask = old_para.control_mask | 0x0000_0800;
            merged_para.has_para_text = true;

            self.document.sections[section_idx].paragraphs[para_idx] = merged_para;
            insert_para_idx = para_idx;
        } else if char_offset == 0 && para.controls.is_empty() {
            // 문단 맨 앞이면 바로 앞에 삽입
            self.document.sections[section_idx]
                .paragraphs
                .insert(para_idx, table_para);
            insert_para_idx = para_idx;
            table_control_idx = 0;
        } else {
            // 문단 중간이면 분할 후 삽입
            if char_offset > 0 && !para.text.is_empty() {
                let new_para =
                    self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, new_para);
                // 표 문단은 분할된 뒤에 삽입
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, table_para);
                insert_para_idx = para_idx + 1;
                table_control_idx = 0;
            } else {
                // char_offset == 0이지만 컨트롤이 있는 경우 → 뒤에 삽입
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, table_para);
                insert_para_idx = para_idx + 1;
                table_control_idx = 0;
            }
        }

        // 표 아래에 빈 문단 추가 (HWP 표준, 한컴 blank_h_saved.hwp 참조)
        self.document.sections[section_idx]
            .paragraphs
            .insert(insert_para_idx + 1, make_empty_neighbor_para());

        // --- 6. 스타일 갱신 + 리플로우 + 페이지네이션 ---
        // 새 BorderFill 추가 시 styles.border_styles 갱신이 필요하므로 rebuild_section 사용
        self.rebuild_section(section_idx);

        self.event_log.push(DocumentEvent::TableRowInserted {
            section: section_idx,
            para: insert_para_idx,
            ctrl: table_control_idx,
        });
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{}",
            insert_para_idx, table_control_idx
        )))
    }
    /// 커서 위치에 표를 삽입한다 (확장, JSON 옵션).
    ///
    /// 기본 create_table_native의 확장판으로, treat_as_char(인라인) 등 세부 속성을 지정할 수 있다.
    /// treat_as_char=true인 경우:
    ///   - 별도 문단을 생성하지 않고 기존 문단의 controls에 표를 추가
    ///   - 텍스트 흐름에 8 UTF-16 코드유닛 자리를 삽입
    ///   - 표 아래 빈 문단 미생성
    pub fn create_table_ex_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        row_count: u16,
        col_count: u16,
        treat_as_char: bool,
        col_widths_hu: Option<&[u32]>,
        row_heights_hu: Option<&[u32]>,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{
            BorderFill, BorderLine, BorderLineType, CenterLine, DiagonalLine, Fill,
        };
        use crate::model::table::{Cell, Table, TablePageBreak};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={})",
                row_count, col_count
            )));
        }

        if !treat_as_char {
            return self.create_table_native(
                section_idx,
                para_idx,
                char_offset,
                row_count,
                col_count,
            );
        }

        // ── 인라인 TAC 표 생성 ──

        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin: i16 = 283;
        let outer_margin_lr = (outer_margin * 2) as i32;
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr)
                .max(7200) as u32;

        // 열 폭 결정
        let col_ws: Vec<u32> = if let Some(widths) = col_widths_hu {
            if widths.len() == col_count as usize {
                widths.to_vec()
            } else {
                let w = content_width / col_count as u32;
                vec![w; col_count as usize]
            }
        } else {
            let w = content_width / col_count as u32;
            vec![w; col_count as usize]
        };
        let total_width: u32 = col_ws.iter().sum();

        let cell_pad = crate::model::Padding {
            left: 510,
            right: 510,
            top: 141,
            bottom: 141,
        };
        let min_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let row_heights: Vec<u32> = if let Some(heights) = row_heights_hu {
            if heights.len() == row_count as usize {
                heights.iter().map(|h| (*h).max(min_row_height)).collect()
            } else {
                vec![min_row_height; row_count as usize]
            }
        } else {
            vec![min_row_height; row_count as usize]
        };
        let total_height: u32 = row_heights.iter().sum();

        // BorderFill
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders
                    .iter()
                    .all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16
            } else {
                let solid_border = BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                };
                let new_bf = BorderFill {
                    raw_data: None,
                    attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine {
                        diagonal_type: 1,
                        width: 0,
                        color: 0,
                    },
                    center_line: CenterLine::None,
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16
            }
        };

        // create_table_native 와 동일 — 커서 offset 의 글자모양을 상속 기준으로 쓴다.
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shape_id_at(char_offset).unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 셀 생성
        let mut cells = Vec::with_capacity((row_count as usize) * (col_count as usize));
        for r in 0..row_count {
            let row_height = row_heights[r as usize];
            for c in 0..col_count {
                let col_w = col_ws[c as usize];
                let mut cell = Cell::new_empty(c, r, col_w, row_height, cell_border_fill_id);
                cell.padding = cell_pad;
                cell.vertical_align = crate::model::table::VerticalAlign::Center;
                for cp in &mut cell.paragraphs {
                    cp.char_count_msb = true;
                    cp.para_shape_id = default_para_shape_id;
                    // create_table_native 와 동일 — 빈 char_shapes 는 charPrIDRef="0" 이 된다.
                    // default_char_shape_id 는 여기서 쓰라고 계산된 값이었으나 누락돼 있었다.
                    cp.char_shapes = vec![CharShapeRef {
                        start_pos: 0,
                        char_shape_id: default_char_shape_id,
                    }];
                    if cp.raw_header_extra.len() < 10 {
                        let mut rhe = vec![0u8; 10];
                        rhe[0..2].copy_from_slice(&1u16.to_le_bytes());
                        rhe[4..6].copy_from_slice(&1u16.to_le_bytes());
                        cp.raw_header_extra = rhe;
                    }
                    let seg_w = (col_w as i32) - 141 - 141;
                    let text_height =
                        row_height.saturating_sub((cell_pad.top + cell_pad.bottom) as u32);
                    cp.line_segs = vec![LineSeg {
                        text_start: 0,
                        line_height: text_height as i32,
                        text_height: text_height as i32,
                        baseline_distance: (text_height as f64 * 0.85) as i32,
                        line_spacing: 600,
                        segment_width: seg_w,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }];
                }
                cell.raw_list_extra = Vec::new();
                cells.push(cell);
            }
        }

        // Table 구조체
        let row_sizes: Vec<i16> = (0..row_count).map(|_| col_count as i16).collect();
        // raw_ctrl_data: treat_as_char + vert=Page(0) + horz=Para(3) + wrap=TopAndBottom(1)
        #[allow(clippy::identity_op)]
        let flags: u32 = (1 << 0) /* treat_as_char */
            | (0 << 3) /* vert=Page */
            | (3 << 8) /* horz=Para */
            | (4 << 15) /* width_criterion=Absolute */
            | (2 << 18) /* height_criterion=Absolute */
            | (1 << 21) /* wrap=TopAndBottom */;
        let mut raw_ctrl_data = vec![0u8; 38];
        raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
            .copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
            .copy_from_slice(&outer_margin.to_le_bytes());
        let instance_id: u32 = {
            let mut h: u32 = 0x7c160000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            if h == 0 {
                h = 0x7c164b69;
            }
            h
        };
        raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x04000006,
            row_count,
            col_count,
            cell_spacing: 0,
            padding: cell_pad,
            row_sizes,
            border_fill_id: cell_border_fill_id,
            zones: Vec::new(),
            cells,
            cell_grid: Vec::new(),
            page_break: TablePageBreak::RowBreak,
            repeat_header: false,
            caption: None,
            common: crate::model::shape::CommonObjAttr {
                treat_as_char: true,
                text_wrap: crate::model::shape::TextWrap::TopAndBottom,
                vert_rel_to: crate::model::shape::VertRelTo::Page,
                horz_rel_to: crate::model::shape::HorzRelTo::Para,
                vert_align: crate::model::shape::VertAlign::Top,
                horz_align: crate::model::shape::HorzAlign::Left,
                width: total_width,
                height: total_height,
                ..Default::default()
            },
            outer_margin_left: outer_margin,
            outer_margin_right: outer_margin,
            outer_margin_top: outer_margin,
            outer_margin_bottom: outer_margin,
            raw_ctrl_data,
            raw_table_record_attr: 0x04000006,
            raw_table_record_extra: vec![0u8; 2],
            dirty: true,
            local_resize_rows: Vec::new(),
            local_resize_cols: Vec::new(),
            local_resize_cell_widths: Vec::new(),
            local_resize_cell_heights: Vec::new(),
        };
        table.rebuild_grid();

        // ── 기존 문단에 인라인 삽입 ──
        self.document.sections[section_idx].raw_stream = None;
        let para = &mut self.document.sections[section_idx].paragraphs[para_idx];

        // controls에 표 추가
        let ctrl_idx = para.controls.len();
        para.controls.push(Control::Table(Box::new(table)));
        para.ctrl_data_records.push(None);

        // char_offsets에 8 UTF-16 코드유닛 갭 삽입
        // 확장 제어문자는 8 코드유닛을 차지
        let insert_utf16_pos = if char_offset < para.char_offsets.len() {
            para.char_offsets[char_offset]
        } else if !para.char_offsets.is_empty() {
            let last_idx = para.char_offsets.len() - 1;
            let last_char_len = para
                .text
                .chars()
                .nth(last_idx)
                .map(|c| c.len_utf16() as u32)
                .unwrap_or(1);
            para.char_offsets[last_idx] + last_char_len
        } else {
            0
        };

        // 이후 char_offsets를 8만큼 shift
        for offset in para.char_offsets.iter_mut() {
            if *offset >= insert_utf16_pos {
                *offset += 8;
            }
        }

        // char_count 갱신 (확장 제어문자 8 + 기존)
        para.char_count += 8;

        // LINE_SEG 갱신: 표 높이를 반영
        if let Some(seg) = para.line_segs.first_mut() {
            let new_lh = (total_height as i32).max(seg.line_height);
            if new_lh > seg.line_height {
                seg.line_height = new_lh;
                seg.text_height = new_lh;
                seg.baseline_distance = (new_lh as f64 * 0.85) as i32;
            }
        }

        // rebuild
        self.rebuild_section(section_idx);

        self.event_log.push(DocumentEvent::TableRowInserted {
            section: section_idx,
            para: para_idx,
            ctrl: ctrl_idx,
        });
        // 표 바로 뒤의 논리적 오프셋 계산
        let logical_after = crate::document_core::helpers::text_to_logical_offset(
            &self.document.sections[section_idx].paragraphs[para_idx],
            char_offset,
        ) + 1;
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
            para_idx, ctrl_idx, logical_after
        )))
    }
    /// 표 셀의 page-relative 좌상단 좌표를 HWPUNIT 단위로 계산 (#1151 floating).
    ///
    /// render tree 를 순회하여 cell_path 와 매칭되는 TableCell 노드를 찾고
    /// bbox.x / bbox.y (px) 를 HWPUNIT 으로 환산 (× 75).
    ///
    /// 매칭 실패 / 페이지 미빌드 시 (0, 0) fallback — picture 가 페이지 좌상단에
    /// 떠도 사용자가 드래그로 위치 조정 가능.
    pub(crate) fn compute_cell_page_offset(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path: &[(usize, usize, usize)],
    ) -> (i32, i32) {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        if cell_path.is_empty() {
            return (0, 0);
        }

        fn matches_cell_run(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
        ) -> bool {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                return tr.cell_context.as_ref().is_some_and(|ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path.len() == path.len()
                        && ctx
                            .path
                            .iter()
                            .zip(path.iter())
                            .all(|(a, b)| a.control_index == b.0 && a.cell_index == b.1)
                });
            }
            for child in &node.children {
                if matches!(child.node_type, RenderNodeType::Table(_)) {
                    continue;
                }
                if matches_cell_run(child, parent_para, path) {
                    return true;
                }
            }
            false
        }

        fn find_cell(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
        ) -> Option<(f64, f64)> {
            if let RenderNodeType::Table(_) = node.node_type {
                if matches_cell_run(node, parent_para, path) {
                    let target_cell = path.last().unwrap().1;
                    for child in node.children.iter() {
                        if let RenderNodeType::TableCell(ref tc) = child.node_type {
                            if tc.model_cell_index == Some(target_cell as u32) {
                                return Some((child.bbox.x, child.bbox.y));
                            }
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(found) = find_cell(child, parent_para, path) {
                    return Some(found);
                }
            }
            None
        }

        let total_pages = self.page_count();
        for p in 0..total_pages {
            if let Ok(tree) = self.build_page_tree(p) {
                if let Some((px, py)) = find_cell(&tree.root, parent_para_idx, cell_path) {
                    // px → HWPUNIT (1px = 75 HWPUNIT at 96 DPI 가정).
                    // 단, section_idx 가 의미 있는 단위 정합을 위해 section 자체의
                    // 보정은 호출 측 (Picture.horz/vert_rel_to=Page) 가 처리.
                    let _ = section_idx;
                    return ((px * 75.0) as i32, (py * 75.0) as i32);
                }
            }
        }
        (0, 0)
    }
    /// 표의 모든 셀 bbox를 반환한다 (네이티브).
    pub(crate) fn get_table_cell_bboxes_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        self.get_table_cell_bboxes_from_page(section_idx, parent_para_idx, control_idx, 0)
    }
    /// page_hint부터 탐색하여 표의 셀 bbox를 반환한다 (네이티브).
    /// page_hint에서 못 찾으면 앞쪽도 탐색한다 (페이지 분할된 표 대응).
    pub(crate) fn get_table_cell_bboxes_from_page(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        page_hint: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 렌더 트리에서 해당 표 노드를 찾아 셀 bbox를 수집
        fn find_table_cells(
            node: &RenderNode,
            sec: usize,
            ppi: usize,
            ci: usize,
            page_idx: usize,
            result: &mut Vec<String>,
        ) -> bool {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                if tn.section_index == Some(sec)
                    && tn.para_index == Some(ppi)
                    && tn.control_index == Some(ci)
                {
                    for (_child_idx, child) in node.children.iter().enumerate() {
                        if let RenderNodeType::TableCell(ref cn) = child.node_type {
                            // cellIdx: 모델의 cells 배열에서 (row, col)로 검색한 인덱스
                            let model_cell_idx = cn.model_cell_index.unwrap_or(0) as usize;
                            result.push(format!(
                                "{{\"cellIdx\":{},\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
                                model_cell_idx, cn.row, cn.col, cn.row_span, cn.col_span,
                                page_idx,
                                child.bbox.x, child.bbox.y, child.bbox.width, child.bbox.height
                            ));
                        }
                    }
                    return true; // 찾음
                }
            }
            for child in &node.children {
                if find_table_cells(child, sec, ppi, ci, page_idx, result) {
                    return true;
                }
            }
            false
        }

        let mut cells = Vec::new();
        let total_pages = self.page_count() as usize;
        let start = page_hint.min(total_pages.saturating_sub(1));

        // page_hint부터 뒤쪽 탐색
        let mut found = false;
        for page_num in start..total_pages {
            let tree = self.build_page_tree_cached(page_num as u32)?;
            if find_table_cells(
                &tree.root,
                section_idx,
                parent_para_idx,
                control_idx,
                page_num,
                &mut cells,
            ) {
                found = true;
            } else if found {
                break;
            }
        }

        // page_hint에서 못 찾았으면 앞쪽 탐색 (페이지 분할 표가 hint 이전 페이지에서 시작될 수 있음)
        if !found && start > 0 {
            for page_num in (0..start).rev() {
                let tree = self.build_page_tree_cached(page_num as u32)?;
                if find_table_cells(
                    &tree.root,
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    page_num,
                    &mut cells,
                ) {
                    found = true;
                    // 이 페이지에서 찾음 — hint까지 다시 정방향 탐색하여 누락된 페이지 수집
                    for fwd in (page_num + 1)..=start {
                        let tree2 = self.build_page_tree_cached(fwd as u32)?;
                        if !find_table_cells(
                            &tree2.root,
                            section_idx,
                            parent_para_idx,
                            control_idx,
                            fwd,
                            &mut cells,
                        ) {
                            break;
                        }
                    }
                    break;
                }
            }
        }

        Ok(format!("[{}]", cells.join(",")))
    }

    // ── 글상자(Shape) CRUD ─────────────────────────────────
    /// [Task #1138] 표 셀 내 Shape 속성 조회 (by_path).
    /// [Task #1151 v4] 셀 안 picture 속성 조회 (cell_path 기반).
    /// `get_cell_shape_properties_by_path_native` Picture 버전.
    pub fn get_cell_picture_properties_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        // [Task #1171] 표 셀과 글상자(Shape text_box) 를 모두 처리하는 resolver 사용.
        // (기존 resolve_cell_by_path 는 마지막 세그먼트가 표 셀이어야 했음.)
        let cell_para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &path)?;
        let ctrl = cell_para.controls.get(inner_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
        })?;
        let pic = match ctrl {
            Control::Picture(p) => p,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                ))
            }
        };
        Self::format_picture_properties_json(pic)
    }
    pub fn get_cell_shape_properties_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;
        let last_cell_para_idx = path.last().unwrap().2;
        let cell_para = cell.paragraphs.get(last_cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 문단 {} 범위 초과", last_cell_para_idx))
        })?;
        let ctrl = cell_para.controls.get(inner_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
        })?;
        let shape_ref = match ctrl {
            Control::Shape(s) => s.as_ref(),
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string(),
                ))
            }
        };
        Self::format_shape_props_inner(shape_ref)
    }
    /// [Task #1138] 표 셀 내 Shape 속성 변경 (by_path).
    /// [Task #1151 v4] 셀 안 picture 속성 변경 (cell_path 기반).
    ///
    /// `set_cell_shape_properties_by_path_native` 와 동일 패턴 — 셀 path 순회 후
    /// inner_control_idx 의 Picture 에 대해 `apply_picture_props_inner` 적용.
    /// v2 의 tac 토글 마이그레이션 path 는 본 셀 안 picture path 에서는 적용되지
    /// 않는다 (셀 안 inline picture 는 이미 셀 안 위치에 있고, 한컴은 inline→floating
    /// 자동 변환을 별도 path 로 처리. 본 PR 의 v2 scope 는 floating→inline 만).
    pub fn set_cell_picture_properties_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        use crate::document_core::helpers::{json_bool, json_i32};

        let path = Self::parse_cell_path_json(cell_path_json)?;
        let restrict_change = json_bool(props_json, "restrictInPage");
        let restrict_enabled_by_this_call = restrict_change.unwrap_or(false);
        let clamp_horz =
            restrict_enabled_by_this_call || json_i32(props_json, "horzOffset").is_some();
        let clamp_vert =
            restrict_enabled_by_this_call || json_i32(props_json, "vertOffset").is_some();
        let caption_changed = {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let current_para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            let ctrl = current_para
                .controls
                .get_mut(inner_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
                })?;
            let pic = match ctrl {
                Control::Picture(p) => p,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                    ))
                }
            };
            let had_caption = pic.caption.is_some();
            let caption_created = Self::apply_picture_props_inner(pic, props_json);
            caption_created || (had_caption && pic.caption.is_none())
        };
        if caption_changed {
            crate::parser::assign_auto_numbers(&mut self.document);
        }
        let section = &mut self.document.sections[section_idx];
        Self::clamp_direct_owner_cell_picture_offsets(
            section,
            parent_para_idx,
            &path,
            inner_control_idx,
            clamp_horz,
            clamp_vert,
        )?;
        Self::sync_direct_owner_cell_for_picture(
            section,
            parent_para_idx,
            &path,
            inner_control_idx,
        )?;
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        let outer_table_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_table_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
    }
    /// [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (cell_path 기반).
    pub fn delete_cell_picture_control_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            if inner_control_idx >= para.controls.len() {
                return Err(HwpError::RenderError(format!(
                    "셀 내 컨트롤 {} 범위 초과",
                    inner_control_idx
                )));
            }
            if !matches!(&para.controls[inner_control_idx], Control::Picture(_)) {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                ));
            }

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
                    if ci == inner_control_idx {
                        gap_start = Some(prev_end);
                        break 'outer;
                    }
                    ci += 1;
                    prev_end += 8;
                }
                let char_size: u32 = if text_chars[i] == '\t' {
                    8
                } else if text_chars[i].len_utf16() == 2 {
                    2
                } else {
                    1
                };
                prev_end = offset + char_size;
            }
            if gap_start.is_none() {
                while ci < para.controls.len() {
                    if ci == inner_control_idx {
                        gap_start = Some(prev_end);
                        break;
                    }
                    ci += 1;
                    prev_end += 8;
                }
            }

            if let Some(gs) = gap_start {
                let threshold = gs + 8;
                for offset in para.char_offsets.iter_mut() {
                    if *offset >= threshold {
                        *offset -= 8;
                    }
                }
            }

            para.controls.remove(inner_control_idx);
            if inner_control_idx < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(inner_control_idx);
            }
            if para.char_count >= 8 {
                para.char_count -= 8;
            }
            Self::reflow_paragraph_line_segs_after_control_delete(para, &self.styles, self.dpi);
        }

        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        let outer_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
    }
    pub fn set_cell_shape_properties_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        let caption_changed = {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let current_para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            let ctrl = current_para
                .controls
                .get_mut(inner_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
                })?;
            let shape = match ctrl {
                Control::Shape(s) => s.as_mut(),
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string(),
                    ))
                }
            };
            Self::apply_shape_props_inner(shape, props_json)
        };
        if caption_changed {
            crate::parser::assign_auto_numbers(&mut self.document);
        }
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        let outer_table_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_table_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::paragraph::CharShapeRef;
    use crate::model::table::Table;

    /// 표를 삽입한 문단에 서식을 심고, 그 문서에서 표를 만든다.
    fn core_with_shaped_paragraph() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        let para = &mut core.document.sections[0].paragraphs[0];
        para.para_shape_id = 12;
        para.char_shapes = vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }];
        core
    }

    fn table_of(core: &DocumentCore) -> &Table {
        core.document.sections[0]
            .paragraphs
            .iter()
            .find_map(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::Table(t) => Some(t.as_ref()),
                    _ => None,
                })
            })
            .expect("표 컨트롤")
    }

    /// Cell::new_empty() 의 문단은 char_shapes 가 비어 있고, 저장기는 그것을
    /// charPrIDRef="0" 으로 쓴다 — 새 표의 셀에 글자를 입력하면 문서의 0번
    /// 글자모양이 나온다. 표를 삽입한 문단의 글자모양을 상속해야 한다.
    fn assert_cells_inherit_shape(table: &Table) {
        assert!(!table.cells.is_empty(), "셀이 있어야 한다");
        for cell in &table.cells {
            let para = &cell.paragraphs[0];
            assert_eq!(
                para.para_shape_id, 12,
                "셀 ({},{}) para_shape_id",
                cell.row, cell.col
            );
            assert_eq!(
                para.char_shapes.first().map(|cs| cs.char_shape_id),
                Some(7),
                "셀 ({},{}) char_shapes — 비면 charPrIDRef=0",
                cell.row,
                cell.col
            );
        }
    }

    #[test]
    fn create_table_native_cells_inherit_char_shape() {
        let mut core = core_with_shaped_paragraph();
        core.create_table_native(0, 0, 0, 2, 3).unwrap();
        assert_cells_inherit_shape(table_of(&core));
    }

    #[test]
    fn create_table_ex_native_cells_inherit_char_shape() {
        let mut core = core_with_shaped_paragraph();
        core.create_table_ex_native(0, 0, 0, 2, 3, false, None, None)
            .unwrap();
        assert_cells_inherit_shape(table_of(&core));
    }

    /// 혼합 글자모양 문단: 텍스트 20자, 글자 인덱스 0~9 는 34, 10~ 는 37.
    /// 커서 offset 10 의 글자모양(37)은 첫 엔트리(34)와 다르다 — 첫 엔트리를
    /// 상속 기준으로 쓰는 회귀를 잡는다.
    fn core_with_mixed_shape_paragraph() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "0123456789abcdefghij")
            .unwrap();
        let para = &mut core.document.sections[0].paragraphs[0];
        para.para_shape_id = 12;
        // 컨트롤(SectionDef 등)이 UTF-16 앞자리를 차지하므로 경계는 char_offsets 로 계산.
        let boundary = para.char_offsets[10];
        para.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 34,
            },
            CharShapeRef {
                start_pos: boundary,
                char_shape_id: 37,
            },
        ];
        core
    }

    fn assert_cells_inherit_cursor_shape(table: &Table) {
        assert!(!table.cells.is_empty(), "셀이 있어야 한다");
        for cell in &table.cells {
            let para = &cell.paragraphs[0];
            assert_eq!(
                para.char_shapes.first().map(|cs| cs.char_shape_id),
                Some(37),
                "셀 ({},{}) — 커서 offset 의 글자모양(37)이 아니라 첫 엔트리(34)를 상속",
                cell.row,
                cell.col
            );
        }
    }

    #[test]
    fn create_table_native_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        core.create_table_native(0, 0, 10, 2, 2).unwrap();
        assert_cells_inherit_cursor_shape(table_of(&core));
    }

    #[test]
    fn create_table_ex_native_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        core.create_table_ex_native(0, 0, 10, 2, 2, false, None, None)
            .unwrap();
        assert_cells_inherit_cursor_shape(table_of(&core));
    }

    /// treat_as_char=true 인라인 경로는 create_table_native 로 위임하지 않는
    /// 별도 구현이므로 따로 검증한다.
    #[test]
    fn create_table_ex_native_tac_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        core.create_table_ex_native(0, 0, 10, 2, 2, true, None, None)
            .unwrap();
        assert_cells_inherit_cursor_shape(table_of(&core));
    }
}
