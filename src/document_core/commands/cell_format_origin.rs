//! 셀 서식 변경의 저장 조각 원점 갱신. 줄 측정과 배치는 같은 LineSeg를 소비한다.

use super::text_editing::cell_vpos_resets;
use crate::document_core::DocumentCore;
use crate::error::HwpError;

impl DocumentCore {
    /// 첫 저장 줄에는 문단 앞 간격이 이미 포함돼 있다. 서식을 바꿀 때 그
    /// 차이만 원점에 반영하고, 나머지 문단은 기존 vpos 사다리에서 재배치한다.
    /// 조각 경계는 좌표를 바꾸기 전에 보존해 새 앞 간격을 분할 신호로 읽지 않는다.
    pub(super) fn restamp_cell_format_origin(
        &mut self,
        location: (usize, usize, usize, usize, usize),
        old_shape_id: u16,
        new_shape_id: u16,
    ) -> Result<(), HwpError> {
        let (section, parent, control, cell, index) = location;
        let before = |id: u16| {
            self.document
                .doc_info
                .para_shapes
                .get(id as usize)
                .map_or(0, |shape| shape.spacing_before)
        };
        // 문단 모양의 간격은 2배 HWPUNIT이다. 셀 첫 문단의 lead는
        // cell_first_para_stored_lead와 같은 스케일이며 문단 간 흐름 간격과 구분한다.
        let old_spacing = crate::renderer::hwpunit_to_px(before(old_shape_id), self.dpi) / 2.0;
        let new_spacing = crate::renderer::hwpunit_to_px(before(new_shape_id), self.dpi) / 2.0;
        if old_spacing == new_spacing {
            return Ok(());
        }
        let paragraph = self
            .get_cell_paragraph_ref(section, parent, control, cell, index)
            .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".to_string()))?;
        let resets = index.checked_sub(1).and_then(|previous| {
            self.get_cell_paragraph_ref(section, parent, control, cell, previous)
                .map(|previous| cell_vpos_resets(previous, paragraph))
        });
        let next_reset = self
            .get_cell_paragraph_ref(section, parent, control, cell, index + 1)
            .map(|next| cell_vpos_resets(paragraph, next));
        let stored = !crate::renderer::para_has_no_stored_line_segs(paragraph);
        let old_lead =
            crate::renderer::cell_first_para_stored_lead(paragraph, old_spacing, self.dpi);
        let delta = crate::renderer::px_to_hwpunit(new_spacing - old_lead, self.dpi);
        let paragraph = self.get_cell_paragraph_mut(section, parent, control, cell, index)?;
        if let Some(reset) = resets {
            paragraph.cell_vpos_reset = Some(reset);
        }
        // 이어받기 조각의 원점은 앞 간격을 새로 품는 셀 첫 문단이 아니다.
        // 그 원점은 보존하고 이후 문단의 간격은 vpos 사다리에서 소비한다.
        if stored && index == 0 {
            for segment in &mut paragraph.line_segs {
                segment.vertical_pos += delta;
            }
        }
        if let Some(reset) = next_reset {
            self.get_cell_paragraph_mut(section, parent, control, cell, index + 1)?
                .cell_vpos_reset = Some(reset);
        }
        Ok(())
    }
}
