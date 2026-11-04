//! 편집으로 높이가 바뀐 글자처럼 취급 표의 크기와 그 표가 놓인 본문 저장 줄을 맞춘다.
//!
//! 한컴은 칸 높이(cellSz)는 그대로 두고 표 높이(sz)를 행 내용으로 다시 정한다. 표가 놓인
//! 저장 줄도 같은 차이만큼 키우거나 줄이고, 그 뒤 줄과 뒤 문단의 vpos를 같은 차이만큼
//! 옮긴다. 표 앞 줄(구역·단 정의만 든 줄 등)과 문단 사이 간격은 저장값 그대로 둔다.
//! 저장 줄이 옛 높이로 남으면 그 줄을 쓰는 배치가 표를 거절하고, 다시 열면 뒤 문단이
//! 옛 줄 끝으로 올라붙는다. 높이는 쪽 나누기가 잰 값을 쓰고, 줄을 고치면 쪽을 다시 나눈다.
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::composer::paragraph_flow_end;
use crate::renderer::layout::control_line_seg_index;
use crate::renderer::px_to_hwpunit;

/// px 왕복 반올림 오차는 높이 변화로 보지 않는다.
const ROUNDING_HU: i64 = 10;

/// 직전 쪽 나누기 측정과 비교할 표: (구역, 문단, 컨트롤, 직전 측정 높이 px).
pub(crate) type TacHostRowCandidate = (usize, usize, usize, f64);

impl DocumentCore {
    /// 이번 쪽 나누기에서 다시 잴, 바뀐 문단의 글자처럼 취급 표와 직전 측정 높이.
    /// 구역 전체를 다시 재는 경우(불러오기 등)는 편집한 표를 가릴 수 없어 넣지 않는다.
    pub(crate) fn tac_host_row_candidates(&self) -> Vec<TacHostRowCandidate> {
        let mut candidates = Vec::new();
        for (section_idx, dirty) in self.dirty_paragraphs.iter().enumerate() {
            let (Some(dirty), Some(section), Some(measured)) = (
                dirty.as_deref(),
                self.document.sections.get(section_idx),
                self.measured_sections.get(section_idx),
            ) else {
                continue;
            };
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                if !dirty.get(para_idx).copied().unwrap_or(false) {
                    continue;
                }
                for (control_idx, control) in para.controls.iter().enumerate() {
                    if !matches!(control, Control::Table(table) if table.common.treat_as_char) {
                        continue;
                    }
                    if let Some(previous) = measured.get_measured_table(para_idx, control_idx) {
                        candidates.push((
                            section_idx,
                            para_idx,
                            control_idx,
                            previous.total_height,
                        ));
                    }
                }
            }
        }
        candidates
    }

    /// 쪽 나누기가 방금 잰 높이가 직전과 달라진 표의 저장 줄을 맞춘다. 바꾼 것이 있으면 true다.
    /// 쪽 나누기와 같은 측정값을 쓰므로, 다시 쪽을 나누면 저장 줄 경로가 표를 받아들인다.
    pub(crate) fn sync_tac_host_rows(&mut self, candidates: &[TacHostRowCandidate]) -> bool {
        let rounding = crate::renderer::hwpunit_to_px(ROUNDING_HU as i32, self.dpi);
        let mut changed = false;
        for &(section_idx, para_idx, control_idx, previous) in candidates {
            let Some(current) = self
                .measured_sections
                .get(section_idx)
                .and_then(|section| section.get_measured_table(para_idx, control_idx))
                .map(|table| table.total_height)
            else {
                continue;
            };
            if (current - previous).abs() <= rounding {
                continue;
            }
            changed |= self.sync_tac_host_row(section_idx, para_idx, control_idx, current);
        }
        changed
    }

    fn sync_tac_host_row(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        measured_px: f64,
    ) -> bool {
        let Some(para) = self
            .document
            .sections
            .get(section_idx)
            .and_then(|section| section.paragraphs.get(para_idx))
        else {
            return false;
        };
        let Some(Control::Table(table)) = para.controls.get(control_idx) else {
            return false;
        };
        if !table.common.treat_as_char || table.caption.is_some() || table.common.height == 0 {
            return false;
        }
        // 표 외곽 상자 전체가 그 줄의 글자 높이일 때만 표가 그 저장 줄을 소유한다.
        let Some(row) = control_line_seg_index(para, control_idx) else {
            return false;
        };
        let Some(seg) = para.line_segs.get(row) else {
            return false;
        };
        let declared = i64::from(table.common.height);
        let old_band =
            declared + i64::from(table.outer_margin_top) + i64::from(table.outer_margin_bottom);
        if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || i64::from(seg.text_height) != old_band
        {
            return false;
        }
        let content = i64::from(px_to_hwpunit(measured_px, self.dpi));
        let delta = content - declared;
        if delta.abs() <= ROUNDING_HU || content <= 0 {
            return false;
        }
        let (Ok(content), Ok(delta), Ok(old_band)) = (
            u32::try_from(content),
            i32::try_from(delta),
            i32::try_from(old_band),
        ) else {
            return false;
        };
        let new_band = old_band.saturating_add(delta);
        let old_end = paragraph_flow_end(para);

        let paragraphs = &mut self.document.sections[section_idx].paragraphs;
        let para = &mut paragraphs[para_idx];
        if let Some(Control::Table(table)) = para.controls.get_mut(control_idx) {
            table.sync_ctrl_height(content);
        }
        for (index, seg) in para.line_segs.iter_mut().enumerate() {
            // 한컴은 문단의 모든 줄 line_height에 가장 높은 줄 높이를 적는다.
            if seg.line_height == old_band {
                seg.line_height = new_band;
            }
            if index == row {
                seg.text_height = new_band;
                seg.baseline_distance = ((i64::from(seg.baseline_distance) * i64::from(new_band)
                    + i64::from(old_band) / 2)
                    / i64::from(old_band)) as i32;
            } else if index > row {
                seg.vertical_pos = seg.vertical_pos.saturating_add(delta);
            }
        }
        shift_source_positions(para, row + 1, delta);
        // 뒤 문단은 같은 저장 흐름 안에서만 옮긴다. 쪽·단 시작으로 vpos가 줄어드는 문단부터는
        // 그 자리의 저장 좌표를 그대로 둔다.
        let mut previous_end = old_end;
        let mut shifted = Vec::new();
        for (index, next) in paragraphs.iter_mut().enumerate().skip(para_idx + 1) {
            let Some(first) = next.line_segs.first() else {
                continue;
            };
            if previous_end.is_some_and(|end| first.vertical_pos < end) {
                break;
            }
            previous_end = paragraph_flow_end(next);
            for seg in &mut next.line_segs {
                seg.vertical_pos = seg.vertical_pos.saturating_add(delta);
            }
            shift_source_positions(next, 0, delta);
            shifted.push(index);
        }
        self.recompose_paragraph(section_idx, para_idx);
        self.mark_paragraph_dirty(section_idx, para_idx);
        for index in shifted {
            self.mark_paragraph_dirty(section_idx, index);
        }
        self.mark_section_dirty(section_idx);
        true
    }
}

/// HWPX 저장이 되돌려 쓰는 원본 vpos도 같은 줄만큼 옮긴다.
fn shift_source_positions(para: &mut Paragraph, from: usize, delta: i32) {
    if let Some(positions) = para.source_line_seg_vertical_pos.as_mut() {
        for position in positions.iter_mut().skip(from) {
            *position = position.saturating_add(delta);
        }
    }
}
