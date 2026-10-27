//! 저장 쪽 프레임 안에서 빈 앵커 그림의 내용·물리 점유 소유를 정한다.

use super::{CellUnit, LayoutEngine};
use crate::model::control::Control;
use crate::model::shape::{TextWrap, VertRelTo};
use crate::renderer::hwpunit_to_px;

impl LayoutEngine {
    pub(super) fn delay_empty_anchor_topandbottom_flow_units_before_hard_break(
        &self,
        units: Vec<CellUnit>,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> Vec<CellUnit> {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
        {
            return units;
        }
        // 지연은 같은 저장 프레임 안에서만 허용한다. 단조 재조판 사다리의
        // 먼 명시적 쪽 나눔을 현재 앵커 프레임의 끝으로 추측하지 않는다.
        let source_boundaries: Vec<bool> = units
            .iter()
            .map(|unit| same_cell_stored_frame_boundary(cell, unit))
            .collect();
        let mut has_future_stored_frame = vec![false; units.len()];
        let mut seen_stored_frame = false;
        for idx in (0..units.len()).rev() {
            has_future_stored_frame[idx] = seen_stored_frame;
            if source_boundaries[idx] {
                seen_stored_frame = true;
            }
        }

        // [#7840] 빈 앵커의 host 글줄은 제자리에 남고 그림 띠만 뒤로 미룬다. 저장 사다리가
        // 다음 문단을 그 띠 높이만큼만 전진시켰으면(다음 문단 vpos − 앵커 줄 vpos == 띠)
        // host 글줄은 띠 안에 놓인 것이다. 이미 계상한 host 글줄 높이를 띠에서 빼야 같은
        // 띠를 두 번 세지 않는다 — 156465345 4쪽: 그림 18298HU 앵커가 글줄 22.03px +
        // 띠 243.97px 로 계상돼 그림이 다음 쪽으로 밀리고 한/글에 없는 쪽 하나가 생겼다.
        let stored_profile = (self.profile.get().hwp5_stored_pagination_layout()
            || self.profile.get().hwpx_stored_layout())
            && !self.profile.get().session_edited();
        let mut charged_host_line_h = std::collections::HashMap::<usize, f64>::new();
        for unit in units.iter().filter(|unit| unit.vis_start < unit.vis_end) {
            *charged_host_line_h.entry(unit.para_idx).or_default() += unit.height;
        }
        let stored_band_owns_host_line = |para_idx: usize, band: f64| -> bool {
            stored_profile
                && cell
                    .paragraphs
                    .get(para_idx)
                    .and_then(|para| para.line_segs.first())
                    .zip(
                        cell.paragraphs
                            .get(para_idx + 1)
                            .and_then(|next| next.line_segs.first()),
                    )
                    .is_some_and(|(line, next)| {
                        let synthetic =
                            crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
                        line.tag & synthetic == 0
                            && next.tag & synthetic == 0
                            && (hwpunit_to_px(
                                next.vertical_pos.saturating_sub(line.vertical_pos),
                                self.dpi,
                            ) - band)
                                .abs()
                                <= 1.0
                    })
        };
        let mut reordered = Vec::with_capacity(units.len());
        let mut pending = Vec::new();
        for (idx, mut unit) in units.into_iter().enumerate() {
            if has_future_stored_frame[idx]
                && Self::is_delayable_empty_anchor_topandbottom_flow_unit(cell, &unit)
            {
                // 다른 문단의 편집 여부와 관계없이 원래 앵커의 그림 띠를 소비한다.
                if let Some(para) = cell.paragraphs.get(unit.para_idx) {
                    if let Some(height) =
                        crate::renderer::float_placement::deferred_picture_band_height(
                            para, self.dpi,
                        )
                    {
                        let height = if stored_band_owns_host_line(unit.para_idx, height) {
                            (height
                                - charged_host_line_h
                                    .get(&unit.para_idx)
                                    .copied()
                                    .unwrap_or(0.0))
                            .max(0.0)
                        } else {
                            height
                        };
                        unit.height = height;
                        unit.non_inline_control_range = Some((0, para.controls.len() - 1));
                    }
                }
                pending.push(unit);
                continue;
            }
            // 빈 문단도 새 저장 프레임의 첫 줄일 수 있다.
            // 보이는 글자 유무로 앞 프레임의 그림 점유를 뒤로 넘기지 않는다.
            if source_boundaries[idx] && !pending.is_empty() {
                reordered.append(&mut pending);
            }
            reordered.push(unit);
        }
        reordered.append(&mut pending);
        reordered
    }

    fn is_delayable_empty_anchor_topandbottom_flow_unit(
        cell: &crate::model::table::Cell,
        unit: &CellUnit,
    ) -> bool {
        if !Self::is_non_inline_control_flow_unit(unit) {
            return false;
        }
        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        para.text.trim().is_empty()
            && para.controls.iter().any(|control| match control {
                Control::Picture(pic) => {
                    !pic.common.treat_as_char
                        && pic.common.flow_with_text
                        && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(pic.common.vert_rel_to, VertRelTo::Para)
                }
                Control::Shape(shape) => {
                    let common = shape.common();
                    !common.treat_as_char
                        && common.flow_with_text
                        && matches!(common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(common.vert_rel_to, VertRelTo::Para)
                }
                _ => false,
            })
    }
}

/// 자식에서 전파된 reset은 부모 셀의 저장 프레임을 닫지 않는다.
/// 같은 셀의 인접 원본 줄이 실제로 되감긴 경계만 사용한다. 그림 띠가
/// 다음 글줄 앞을 소유한 양수 원점 재시작은 hard break로만 발행된다.
/// 그 경계도 이미 선택된 컷 앞에서 그림을 닫되, 새 저장 줄 경계로 승격하지 않는다.
fn same_cell_stored_frame_boundary(cell: &crate::model::table::Cell, unit: &CellUnit) -> bool {
    use crate::model::paragraph::LineSeg;
    if !unit.stored_frame_break_before && !unit.hard_break_before {
        return false;
    }
    let Some(para) = cell.paragraphs.get(unit.para_idx) else {
        return false;
    };
    let Some(current) = para.line_segs.get(unit.vis_start) else {
        return false;
    };
    let previous = if unit.vis_start > 0 {
        para.line_segs.get(unit.vis_start - 1)
    } else {
        cell.paragraphs[..unit.para_idx]
            .iter()
            .rev()
            .find_map(|paragraph| paragraph.line_segs.last())
    };
    previous.is_some_and(|previous| {
        (previous.tag | current.tag) & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && current.vertical_pos >= 0
            && current.vertical_pos < previous.vertical_pos
    })
}
