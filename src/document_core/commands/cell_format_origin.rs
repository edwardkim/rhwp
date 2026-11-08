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
        self.restamp_cell_format_origin_by_path(
            section,
            parent,
            &[(control, cell, index)],
            old_shape_id,
            new_shape_id,
        )
    }

    pub(super) fn restamp_cell_format_origin_by_path(
        &mut self,
        section: usize,
        parent: usize,
        path: &[(usize, usize, usize)],
        old_shape_id: u16,
        new_shape_id: u16,
    ) -> Result<(), HwpError> {
        let Some(&(_, _, index)) = path.last() else {
            return Err(HwpError::RenderError("빈 셀 경로".into()));
        };
        let sibling_path = |index| {
            let mut sibling = path.to_vec();
            sibling.last_mut().unwrap().2 = index;
            sibling
        };
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
        // 평면 셀 API는 표/그림 캡션의 특수 셀 인덱스도 해석한다.
        // 중첩 경로 resolver로 바꾸면 기존 캡션의 앞 간격 편집이 실패한다.
        let resolve = |path: &[(usize, usize, usize)]| {
            if let &[(control, cell, index)] = path {
                self.get_cell_paragraph_ref(section, parent, control, cell, index)
                    .ok_or_else(|| HwpError::RenderError("셀 문단을 찾을 수 없음".into()))
            } else {
                self.resolve_paragraph_by_path(section, parent, path)
            }
        };
        let paragraph = resolve(path)?;
        let resets = index.checked_sub(1).and_then(|previous| {
            resolve(&sibling_path(previous))
                .ok()
                .map(|previous| cell_vpos_resets(previous, paragraph))
        });
        let next_reset = resolve(&sibling_path(index + 1))
            .ok()
            .map(|next| cell_vpos_resets(paragraph, next));
        let stored = !crate::renderer::para_has_no_stored_line_segs(paragraph);
        let old_lead =
            crate::renderer::cell_first_para_stored_lead(paragraph, old_spacing, self.dpi);
        let delta = crate::renderer::px_to_hwpunit(new_spacing - old_lead, self.dpi);
        let paragraph = if let &[(control, cell, index)] = path {
            self.get_cell_paragraph_mut(section, parent, control, cell, index)?
        } else {
            self.get_cell_paragraph_mut_by_path(section, parent, path)?
        };
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
            let next = if let &[(control, cell, index)] = path {
                self.get_cell_paragraph_mut(section, parent, control, cell, index + 1)?
            } else {
                self.get_cell_paragraph_mut_by_path(section, parent, &sibling_path(index + 1))?
            };
            next.cell_vpos_reset = Some(reset);
        }
        Ok(())
    }
}
