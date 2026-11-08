//! 구역 나누기·합치기 — 한/글의 [쪽-구역 나누기]와 구역 경계 지우기.
//!
//! 한/글 12.30(Mac)에서 확인한 동작:
//! - 새 구역은 새 쪽에서 시작하고 캐럿은 그 처음에 선다. 문단 처음이면 그 문단부터, 문단 끝이면
//!   다음 문단부터 새 구역이다. 문단 가운데, 구역 첫 문단의 처음, 구역 마지막 문단의 끝에서는
//!   문단을 나눈다(앞 구역에 빈 문단이 남거나 새 구역이 빈 문단으로 시작한다).
//! - 새 구역은 앞 구역의 용지·각주/미주 모양·쪽 테두리·바탕쪽·단 모양과 홀수/짝수 시작을 잇고,
//!   사용자 시작 쪽 번호와 그림·표·수식 시작 번호는 '이어서'로 둔다.
//! - 구역 경계를 지우면 뒤 구역은 앞 구역의 모양(용지·단 등)을 따른다.

use super::super::helpers::{json_ok_with, logical_paragraph_length};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::page::ColumnDef;
use crate::model::paragraph::{ColumnBreakType, Paragraph};

/// 구역 첫 문단의 문단 머리 나누기 바이트. 한/글은 구역(0x01)과 다단(0x02)을 함께 적는다.
const SECTION_START_BREAK: u8 = 0x03;
/// `SectionDef`·`ColumnDef` 제어 문자(코드 2)의 control mask 비트.
const SECTION_COLUMN_DEF_MASK: u32 = 1 << 0x0002;
const EXTENDED_CONTROL_UNITS: u32 = 8;

impl DocumentCore {
    /// 본문 캐럿 자리에서 구역을 나눈다. 돌려주는 `section`·`para`는 새 구역의 첫 문단이다.
    pub fn split_section_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            ))
        })?;
        let para = section.paragraphs.get(para_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과 (총 {}개)",
                para_idx,
                section.paragraphs.len()
            ))
        })?;
        if section.trailing_lists.is_some() {
            return Err(unsupported_tail());
        }
        let length = logical_paragraph_length(para);
        let char_offset = char_offset.min(length);
        let last = para_idx + 1 == section.paragraphs.len();
        // 쪽·단 나누기가 걸린 문단의 처음에서는 한/글처럼 문단을 나눠, 나누기를 지닌 빈 문단을 앞
        // 구역에 남긴다(한/글 12.30에서 확인: 쪽이 하나 는다). HWP 3.0의 자연 쪽 경계에서 합성한
        // 쪽 나누기는 사용자가 넣은 나누기가 아니다.
        let broken = match para.column_type {
            ColumnBreakType::Page => !para.page_break_synthesized,
            ColumnBreakType::Column => true,
            _ => false,
        };
        let first_moved = if char_offset == 0 && para_idx > 0 && !broken {
            para_idx
        } else if char_offset == length && length > 0 && !last {
            para_idx + 1
        } else {
            self.split_paragraph_native(section_idx, para_idx, char_offset, None)?;
            para_idx + 1
        };

        self.flush_cell_format_vpos();
        let tail_sealed = crate::model::section_tail::validate_preservation(&self.document).is_ok();
        let section = &mut self.document.sections[section_idx];
        let column_def = Self::find_column_def_for_paragraph(&section.paragraphs, first_moved - 1);
        // 구역 중간의 구역 정의(HWPX 한 파일의 여러 secPr 등)가 있으면 그 자리에서 쓰이는 정의를 잇는다.
        // 첫 문단의 정의는 구역 정의(`section_def`)가 주인이다.
        let mut section_def = section.paragraphs[1..first_moved]
            .iter()
            .rev()
            .flat_map(|para| para.controls.iter().rev())
            .find_map(|control| match control {
                Control::SectionDef(def) => Some((**def).clone()),
                _ => None,
            })
            .unwrap_or_else(|| section.section_def.clone());
        section_def.page_num = 0;
        section_def.picture_num = 0;
        section_def.table_num = 0;
        section_def.equation_num = 0;
        let mut paragraphs = section.paragraphs.split_off(first_moved);
        section.raw_stream = None;
        section.raw_provenance = None;
        // 읽을 때 구역 정의는 구역 안 마지막 정의를 따른다. 옮긴 쪽에 정의가 있었으면 남은 구역은
        // 제 첫 문단의 정의를 되찾는다.
        let moved_def = paragraphs.iter().any(|para| {
            para.controls
                .iter()
                .any(|c| matches!(c, Control::SectionDef(_)))
        });
        if moved_def {
            if let Some(def) = section.paragraphs[0].controls.iter().find_map(|c| match c {
                Control::SectionDef(def) => Some((**def).clone()),
                _ => None,
            }) {
                section.section_def = def;
            }
        }

        let first = &mut paragraphs[0];
        let leading: Vec<_> = first
            .controls
            .iter()
            .take_while(|control| matches!(control, Control::SectionDef(_) | Control::ColumnDef(_)))
            .collect();
        let own_section_def = leading.iter().find_map(|control| match control {
            Control::SectionDef(def) => Some((**def).clone()),
            _ => None,
        });
        let has_column_def = leading
            .iter()
            .any(|control| matches!(control, Control::ColumnDef(_)));
        // 다단 설정 나누기로 시작하는 문단은 제 단 정의를 그대로 새 구역의 단 정의로 쓴다.
        if !has_column_def {
            let column_def = if column_def.column_count == 0 {
                // 단 정의가 없는 구역은 한 단이다.
                ColumnDef {
                    column_count: 1,
                    same_width: true,
                    ..ColumnDef::default()
                }
            } else {
                column_def
            };
            let at = usize::from(own_section_def.is_some());
            insert_leading_control_at(first, at, Control::ColumnDef(column_def));
        }
        // 이미 구역 정의로 시작하는 문단은 그 정의를 새 구역의 정의로 올린다.
        let section_def = match own_section_def {
            Some(def) => def,
            None => {
                insert_leading_control_at(
                    first,
                    0,
                    Control::SectionDef(Box::new(section_def.clone())),
                );
                section_def
            }
        };
        first.column_type = ColumnBreakType::Section;
        first.raw_break_type = SECTION_START_BREAK;
        first.page_break_synthesized = false;
        first.invalidate_layout_inputs();
        // 새 구역은 새 쪽 맨 위에서 시작한다.
        self.reconnect_moved_lines(&mut paragraphs, 0, true);

        let new_section_idx = section_idx + 1;
        self.document.sections.insert(
            new_section_idx,
            Section {
                section_def,
                paragraphs,
                trailing_lists: None,
                raw_stream: None,
                raw_provenance: None,
            },
        );
        if tail_sealed {
            self.document.seal_section_tail_provenance();
        }
        self.rebuild_after_section_change();
        Ok(json_ok_with(&format!(
            "\"section\":{},\"para\":0,\"charOffset\":0",
            new_section_idx
        )))
    }

    /// `section_idx` 구역을 앞 구역에 합친다. 합친 문단은 앞 구역의 모양을 따른다.
    /// 돌려주는 `para`는 앞 구역에서 옮겨 온 첫 문단의 번호다.
    pub fn merge_section_native(&mut self, section_idx: usize) -> Result<String, HwpError> {
        let count = self.document.sections.len();
        if section_idx == 0 || section_idx >= count {
            return Err(HwpError::RenderError(format!(
                "합칠 구역 인덱스 {} 범위 오류 (총 {}개)",
                section_idx, count
            )));
        }
        let previous = section_idx - 1;
        if self.document.sections[previous].trailing_lists.is_some()
            || self.document.sections[section_idx].trailing_lists.is_some()
        {
            return Err(unsupported_tail());
        }
        if self.document.sections[section_idx].paragraphs.is_empty() {
            return Err(HwpError::RenderError(format!(
                "합칠 구역 {}에 문단이 없습니다",
                section_idx
            )));
        }
        self.flush_cell_format_vpos();
        // 뒤 구역 글은 앞 구역 폭으로 다시 접어야 한다. 접는 폭이 다르면 저장 줄 나눔을 버린다.
        let rewrap = (self.body_wrap_width(section_idx) - self.body_wrap_width(previous)).abs()
            > f64::EPSILON;
        let tail_sealed = crate::model::section_tail::validate_preservation(&self.document).is_ok();
        let mut paragraphs = self.document.sections.remove(section_idx).paragraphs;
        if rewrap {
            for para in &mut paragraphs {
                para.invalidate_layout_inputs();
            }
        }
        if let Some(first) = paragraphs.first_mut() {
            while matches!(
                first.controls.first(),
                Some(Control::SectionDef(_) | Control::ColumnDef(_))
            ) {
                first.remove_control(0);
            }
            if matches!(
                first.column_type,
                ColumnBreakType::Section | ColumnBreakType::MultiColumn
            ) {
                first.column_type = ColumnBreakType::None;
            }
            first.raw_break_type = 0;
            first.invalidate_layout_inputs();
        }
        let first_moved = self.document.sections[previous].paragraphs.len();
        let mut merged = std::mem::take(&mut self.document.sections[previous].paragraphs);
        merged.append(&mut paragraphs);
        // 뒤 구역의 첫 줄은 쪽 맨 위(0)에 저장돼 있다. 쪽 바뀜으로 읽히지 않게 앞 글 흐름에 잇는다.
        self.reconnect_moved_lines(&mut merged, first_moved, false);
        let target = &mut self.document.sections[previous];
        target.paragraphs = merged;
        target.raw_stream = None;
        target.raw_provenance = None;
        if tail_sealed {
            self.document.seal_section_tail_provenance();
        }
        self.rebuild_after_section_change();
        Ok(json_ok_with(&format!(
            "\"section\":{},\"para\":{}",
            previous, first_moved
        )))
    }

    /// 옮긴 `start` 문단부터 저장 줄 위치(쪽 안 좌표)를 다시 잇는다. `page_top`이면 `start` 문단의
    /// 줄을 새 쪽 맨 위(0)부터 차례로 다시 쌓고, 원래 쪽이 바뀌던 자리(저장 끝보다 앞선 시작)는 그대로
    /// 둔다. 아니면 `start` 문단의 저장 위치를 쪽 바뀜으로 읽지 않고 앞 글에 잇는다.
    ///
    /// 파일에서 읽은 원래 좌표(`source_line_seg_vertical_pos`, HWPX 저장이 쓰는 값)가 있으면 같은
    /// 규칙으로 그 좌표도 옮긴다. 메모리의 좌표는 구역 누적일 수 있어 저장에 쓰면 안 된다(#5847).
    fn reconnect_moved_lines(&self, paragraphs: &mut [Paragraph], start: usize, page_top: bool) {
        let swap_source = |paragraphs: &mut [Paragraph]| {
            for para in paragraphs.iter_mut() {
                let Some(source) = para.source_line_seg_vertical_pos.as_mut() else {
                    continue;
                };
                if source.len() == para.line_segs.len() {
                    for (seg, position) in para.line_segs.iter_mut().zip(source.iter_mut()) {
                        std::mem::swap(&mut seg.vertical_pos, position);
                    }
                }
            }
        };
        if paragraphs[start..]
            .iter()
            .any(|para| para.source_line_seg_vertical_pos.is_some())
        {
            // 앞 글도 파일 좌표로 맞춰 둔다. 바뀌는 것은 `start`부터다.
            swap_source(paragraphs);
            self.relink_lines(paragraphs, start, page_top);
            swap_source(paragraphs);
        }
        self.relink_lines(paragraphs, start, page_top);
    }

    fn relink_lines(&self, paragraphs: &mut [Paragraph], start: usize, page_top: bool) {
        let stored_end = crate::renderer::composer::paragraph_flow_end(&paragraphs[start]);
        if page_top {
            // 문단이 쪽을 넘어 이어졌어도 새 쪽에서는 한 흐름이다. 줄 전진량은 조판의 vpos 체인과 같다.
            let mut top = 0i32;
            for seg in &mut paragraphs[start].line_segs {
                seg.vertical_pos = top;
                let height = if seg.line_height > seg.text_height && seg.text_height > 0 {
                    seg.text_height
                } else {
                    seg.line_height
                };
                top = top.saturating_add(height.saturating_add(seg.line_spacing));
            }
        }
        crate::renderer::composer::recalculate_section_vpos(
            paragraphs,
            start,
            (!page_top).then(|| start..start + 1),
            stored_end.filter(|_| page_top),
            &self.styles,
            self.dpi,
            self.document.layout_profile().hwp3_layout(),
        );
    }

    /// 구역 수가 바뀌면 구역 번호로 맞춘 파생 상태를 모두 다시 만든다. 주석 번호는 그대로 둔다
    /// (새 구역은 앞 구역의 번호 매기기를 이어받는다).
    fn rebuild_after_section_change(&mut self) {
        self.rebuild_derived_state();
    }
}

fn unsupported_tail() -> HwpError {
    HwpError::RenderError(
        "바탕쪽·메모 목록을 원본 그대로 저장하는 구역은 아직 나누거나 합칠 수 없습니다".to_string(),
    )
}

/// 문단 맨 앞 `at`번째 컨트롤 자리(앞의 선행 정의 뒤)에 확장 제어 문자 하나를 넣고 그 8유닛만큼
/// 문단 좌표를 민다. 선행 정의는 글자보다 앞이므로 넣는 자리는 언제나 첫 글자 앞이다.
fn insert_leading_control_at(para: &mut Paragraph, at: usize, control: Control) {
    para.shift_for_inline_control_insert(at, 0);
    para.controls.insert(at, control);
    if para.ctrl_data_records.len() >= at && !para.ctrl_data_records.is_empty() {
        para.ctrl_data_records.insert(at, None);
    }
    para.char_count += EXTENDED_CONTROL_UNITS;
    para.control_mask |= SECTION_COLUMN_DEF_MASK;
}
