//! 본문 프레임이 줄어들면 저장 쪽 경계 앞 빈 줄도 실제 점유로 이월한다.

use super::*;
use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};

pub(in crate::renderer::typeset) struct SavedEmptyRun {
    first: usize,
    reset: usize,
    page: usize,
}

fn saved_empty(para: &Paragraph) -> Option<&LineSeg> {
    if !para.text.is_empty()
        || !para.controls.is_empty()
        || para.column_type != ColumnBreakType::None
        || para.stored_text_partition_is_dirty()
    {
        return None;
    }
    let [seg] = para.line_segs.as_slice() else {
        return None;
    };
    (seg.line_height > 0 && seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0).then_some(seg)
}

impl TypesetState {
    pub(in crate::renderer::typeset) fn prepare_saved_empty_run(
        &mut self,
        first: usize,
        paragraphs: &[Paragraph],
        dpi: f64,
        measured_note_area: impl FnOnce(&Self) -> Option<f64>,
    ) {
        if self.data.saved_empty_run.is_some()
            || self.data.col_count != 1
            || self.data.hide_empty_line
            || self.data.current_items.is_empty()
        {
            return;
        }
        let mut reset = first;
        let mut last = None;
        while let Some(seg) = paragraphs.get(reset).and_then(saved_empty) {
            if last.is_some_and(|previous: &LineSeg| seg.vertical_pos < previous.vertical_pos) {
                return;
            }
            last = Some(seg);
            reset += 1;
        }
        let (Some(last), Some(after)) = (last, paragraphs.get(reset)) else {
            return;
        };
        if after.column_type != ColumnBreakType::None
            || after.line_segs.first().is_none_or(|seg| {
                seg.vertical_pos != 0 || seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            })
        {
            return;
        }
        // 원본의 쪽 좌표와 재기준한 현재 커서는 다른 축이다. 다음 reset=0이
        // 닫는 원본 프레임에서 이 빈 줄의 잉크가 본문 안이었는지 먼저 확인한다.
        let source_bottom = paragraphs[reset - 1]
            .source_line_seg_vertical_pos
            .as_ref()
            .map_or(Some(last.vertical_pos), |source| {
                (source.len() == 1).then(|| source[0])
            })
            .map(|vpos| crate::renderer::hwpunit_to_px(vpos.saturating_add(last.line_height), dpi));
        // 이미 수용한 본문의 커서가 물리 프레임을 넘었으면 원래 reset의
        // 꼬리 흡수로 다음 쪽의 빈 prefix까지 없애지 않는다. dirty 표식은
        // 저장 줄 재기준에도 생기므로, 아직 프레임 안인 본문 뒤의 빈 줄이
        // 안 들어간다는 사실만으로 원래 쪽 경계를 무효화하지 않는다.
        // sample16의 정상 저장 꼬리와 수식 재구성 뒤 넘친 본문이 이 경계다.
        if self.data.vpos_ladder_dirty
            && !self.data.profile.session_edited()
            && (self.data.profile.hwp5_stored_pagination_layout()
                || self.data.profile.hwpx_stored_layout())
            && source_bottom.is_some_and(|bottom| bottom <= self.base_available_height())
            && self.data.current_height > self.available_height()
        {
            self.data.saved_empty_run = Some(SavedEmptyRun {
                first,
                reset,
                page: self.data.pages.len(),
            });
            return;
        }
        // 저장 빈 줄의 점유와 그 뒤의 reset은 별개다. 같은 저장 프레임에
        // 있는 빈 꼬리가 실제 각주 상자 위에 모두 들어가면, 추정용 안전
        // 여백 때문에 빈 줄만 다음 쪽으로 보내지 않는다. 각주의 원래
        // 조각·소유자는 바꾸지 않고 paint와 같은 선택 조각 높이만 예약한다.
        if !self.data.current_footnote_body_bottom_reserved
            && !self.data.profile.session_edited()
            && (self.data.profile.hwp5_stored_pagination_layout()
                || self.data.profile.hwpx_stored_layout())
            && self.data.vpos_page_base_stored
            && !self.data.vpos_ladder_dirty
        {
            if let Some(base) = self.data.vpos_page_base {
                let first_line = &paragraphs[first].line_segs[0];
                let top = crate::renderer::hwpunit_to_px(
                    first_line.vertical_pos.saturating_sub(base),
                    dpi,
                );
                let bottom = crate::renderer::hwpunit_to_px(
                    last.vertical_pos
                        .saturating_add(last.line_height)
                        .saturating_sub(base),
                    dpi,
                );
                if (top - self.data.current_height).abs() <= dpi / 7200.0 {
                    if let Some(height) = measured_note_area(self) {
                        let capacity = self.base_available_height()
                            - height
                            - self.data.current_zone_y_offset
                            - self.data.current_bottom_fixed_exclusion;
                        if std::env::var_os("RHWP_DIAG_FLOW").is_some() {
                            eprintln!(
                                "DIAG_EMPTY_FRAME first={first} reset={reset} base={base} source_bottom={source_bottom:?} actual_top={top} actual_bottom={bottom} current={} note={height} capacity={capacity}",
                                self.data.current_height
                            );
                        }
                        if height.is_finite()
                            && height > 0.0
                            && top >= 0.0
                            && source_bottom.is_some_and(|bottom| bottom <= capacity)
                        {
                            self.reserve_painted_footnote_area(height);
                            self.data.saved_empty_run = Some(SavedEmptyRun {
                                first,
                                reset,
                                page: self.data.pages.len(),
                            });
                            return;
                        }
                    }
                }
            }
            return;
        }
        if !self.data.current_footnote_body_bottom_reserved {
            return;
        }
        let bottom =
            crate::renderer::hwpunit_to_px(last.vertical_pos.saturating_add(last.line_height), dpi);
        // 원래 본문에는 들어가지만 실제 각주 영역을 예약한 본문에는 안 들어간다.
        // 저장 reset을 앞당겨 숨기지 않고 각 줄을 같은 fit으로 수용한다.
        if bottom <= self.base_available_height() && bottom > self.available_height() {
            self.data.saved_empty_run = Some(SavedEmptyRun {
                first,
                reset,
                page: self.data.pages.len(),
            });
        }
    }

    pub(in crate::renderer::typeset) fn saved_empty_run_contains(&self, para: usize) -> bool {
        self.data
            .saved_empty_run
            .as_ref()
            .is_some_and(|run| run.first <= para && para < run.reset)
    }

    pub(in crate::renderer::typeset) fn consume_saved_empty_run_reset(
        &mut self,
        reset: usize,
        paragraphs: &[Paragraph],
    ) -> bool {
        if self
            .data
            .saved_empty_run
            .as_ref()
            .is_none_or(|run| run.reset != reset)
        {
            return false;
        }
        let run = self.data.saved_empty_run.take().unwrap();
        self.data.pages.len() > run.page
            && !self.data.current_items.is_empty()
            && self.data.current_items.iter().all(|item| {
                let index = match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph {
                        para_index,
                        start_line: 0,
                        end_line: 1,
                    } => *para_index,
                    _ => return false,
                };
                index >= run.first
                    && index < run.reset
                    && paragraphs.get(index).and_then(saved_empty).is_some()
            })
    }
}
