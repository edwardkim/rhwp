//! 완전한 저장 셀의 글줄 원점과 비자리차지 중첩 표의 부동 원점을 구분한다.

use crate::model::control::Control;
use crate::model::paragraph::LineSeg;
use crate::model::table::{Cell, Table, VerticalAlign};
use crate::renderer::px_to_hwpunit;

use super::{ComposedParagraph, LayoutEngine};

impl LayoutEngine {
    /// 표 62의 저장 글줄 위치는 표 80의 원본 셀 내부 좌표를 사용한다.
    /// 부동 표의 시각 점유로 계산한 정렬 원점을 글자에 다시 적용하지 않는다.
    /// 편집·재조판·조각·크기 변경으로 원본 프레임이 없어진 경우에는 적용하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn stored_overlay_text_cell_origin(
        &self,
        table: &Table,
        cell: &Cell,
        composed: &[ComposedParagraph],
        para_index: usize,
        cell_height: f64,
        inner_height: f64,
        content_origin: f64,
    ) -> Option<f64> {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || cell.dirty_flag
            || cell.text_direction != 0
            || self
                .render_normalization
                .borrow()
                .table_text_reflowed(table)
            || i64::from(px_to_hwpunit(cell_height, self.dpi)) != i64::from(cell.height)
            || composed.len() != cell.paragraphs.len()
        {
            return None;
        }
        // 가운데/아래 셀의 0에서 시작한 사다리는 문단 로컬 좌표일 수 있다.
        // 이를 셀 상단의 절대 좌표로 읽으면 공통 정렬 프레임의 여유 공간이
        // 사라진다. 양수 시작 위치의 증거가 없으면 기존 정렬이 원점을 소유한다.
        if cell.vertical_align != VerticalAlign::Top
            && cell.paragraphs.first()?.line_segs.first()?.vertical_pos == 0
        {
            return None;
        }
        let host = cell.paragraphs.get(para_index)?;
        if host.controls.is_empty()
            || !host.controls.iter().all(|control| {
                matches!(control, Control::Table(nested) if self.nested_table_is_overlay(nested))
            })
        {
            return None;
        }

        // 저장 좌표처럼 보이는 값만으로 캐시를 인증하지 않는다. 현재 조판과
        // 원본 줄 수가 같고, 모든 줄이 그대로인 셀 프레임 안에 있어야 한다.
        let bottom = i64::from(px_to_hwpunit(inner_height, self.dpi));
        let mut previous_end = 0_i64;
        for (para, current) in cell.paragraphs.iter().zip(composed) {
            if para.stored_text_partition_is_dirty()
                || para.cell_format_vpos_dirty
                || para.line_segs.is_empty()
                || para.line_segs.len() != current.lines.len()
                // 줄 수가 같아도 재조판 프레임이 새 좌표를 소유할 수 있다.
                // 실제 배치가 소비할 프레임을 원본 저장 줄로 덮지 않는다.
                || current.frame_geometry.is_some()
            {
                return None;
            }
            for line in &para.line_segs {
                let start = i64::from(line.vertical_pos);
                let end = start + i64::from(line.line_height);
                if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.line_height <= 0
                    || line.text_height <= 0
                    || line.text_height > line.line_height
                    || line.baseline_distance < 0
                    || line.baseline_distance > line.line_height
                    || line.segment_width <= 0
                    || start < previous_end
                    || end > bottom
                {
                    return None;
                }
                previous_end = end;
            }
        }
        Some(content_origin)
    }
}
