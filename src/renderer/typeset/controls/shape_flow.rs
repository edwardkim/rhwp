//! 일반 표 문단에 동반된 그림·도형·수식의 기존 흐름 높이와 이월 조건 조회.
//! 원본 LineSeg 선택과 두 높이의 우선순위를 보존하며 상태를 변경하지 않는다.

use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::hwpunit_to_px;

pub(in crate::renderer::typeset) struct TableHostShapePage {
    pub has_items: bool,
    pub current_height: f64,
}

#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct TableHostShapeFlow {
    pub tac_separate_line_h: Option<f64>,
    pub non_tac_pushdown_h: Option<f64>,
}

pub(super) fn prepare(
    ctrl: &Control,
    para: &Paragraph,
    ctrl_idx: usize,
    dpi: f64,
) -> TableHostShapeFlow {
    // Task #402: 같은 paragraph의 선행 TAC 컨트롤이 있는 TAC 그림은
    // 자기 line_seg에 위치하므로 그 line의 높이를 페이지 누적에 반영해야 함.
    // 누락 시 후속 항목이 페이지 끝을 넘어 그려져 겹침/오버플로 발생 (#402).
    let tac_separate_line_h: Option<f64> = match ctrl {
        Control::Picture(p) if p.common.treat_as_char => Some(()),
        Control::Shape(s) if s.common().treat_as_char => Some(()),
        _ => None,
    }
    .and_then(|_| {
        let prior_tac_count = para
            .controls
            .iter()
            .take(ctrl_idx)
            .filter(|c| match c {
                Control::Table(t) => t.common.treat_as_char,
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            })
            .count();
        if prior_tac_count == 0 {
            return None;
        }
        para.line_segs.get(prior_tac_count).map(|seg| {
            let lh = hwpunit_to_px(seg.line_height, dpi);
            let ls_extra = if seg.line_spacing > 0 {
                hwpunit_to_px(seg.line_spacing, dpi)
            } else {
                0.0
            };
            lh + ls_extra
        })
    });
    // [Issue #1156] 비-TAC 자리차지(TopAndBottom) 객체(차트 OLE 등):
    // 표와 같은 문단에 있으면 종전에는 높이/단 이동 없이 push 만 되어,
    // 한컴처럼 단 끝을 넘는 객체가 다음 단으로 이동하지 못했다.
    // 한컴: 객체가 현재 단 잔여 영역을 넘으면 다음 단 상단으로 이동.
    // (객체 점유 크기 = common 높이 80mm, spec/한컴/HWPX hp:sz 3중 일치)
    use crate::model::shape::{TextWrap, VertRelTo};
    let non_tac_pushdown_h: Option<f64> = if tac_separate_line_h.is_none() {
        match ctrl {
            Control::Picture(p)
                if !p.common.treat_as_char
                    && matches!(p.common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(p.common.vert_rel_to, VertRelTo::Para) =>
            {
                topbottom_pushdown_height(&p.common, dpi).map(|height| {
                    height - hwpunit_to_px(p.common.height as i32, dpi)
                        + crate::renderer::composer::captioned_picture_height_px(p, dpi)
                })
            }
            Control::Shape(s)
                if !s.common().treat_as_char
                    && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
                    && matches!(s.common().vert_rel_to, VertRelTo::Para) =>
            {
                let common = s.common();
                topbottom_pushdown_height(common, dpi).map(|height| {
                    let shape_height = hwpunit_to_px(common.height as i32, dpi);
                    height - shape_height
                        + crate::renderer::composer::captioned_shape_height_px(
                            shape_height,
                            &crate::renderer::composer::shape_caption_for_layout(s),
                            dpi,
                        )
                })
            }
            _ => None,
        }
    } else {
        None
    };

    TableHostShapeFlow {
        tac_separate_line_h,
        non_tac_pushdown_h,
    }
}

impl TableHostShapeFlow {
    /// A sequential paragraph-relative shape owns the frame accepted before
    /// its height advances the cursor. A later saved host line can already
    /// include this exclusion band; it is not another object origin.
    pub(super) fn accepted_shape_frame(
        &self,
        control: &Control,
        origin: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        use crate::model::shape::VertAlign;
        use crate::renderer::float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement};
        let Control::Shape(shape) = control else {
            return None;
        };
        let common = shape.common();
        let height = self.non_tac_pushdown_h?;
        // Nonzero offsets and bottom/center alignment need their own anchor
        // resolution, rather than substituting the sequential flow origin.
        if common.vertical_offset != 0
            || !matches!(common.vert_align, VertAlign::Top | VertAlign::Inside)
        {
            return None;
        }
        Some(ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: origin,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: origin,
            occupied_bottom: origin + height,
        })
    }

    /// TAC의 항목 존재 조건과 float의 단 상단 조건을 구별한다. 예산 조회도 단락 평가한다.
    pub(super) fn needs_advance(
        &self,
        page: TableHostShapePage,
        available_height: impl FnOnce() -> f64,
    ) -> bool {
        if let Some(line_h) = self.tac_separate_line_h {
            // 자기 line이 현재 페이지에 들어가지 않으면 다음 페이지로 분할
            page.has_items && page.current_height + line_h > available_height() + 0.5
        } else if let Some(extra) = self.non_tac_pushdown_h {
            // 비-TAC 자리차지 객체: 현재 단 잔여 부족 + 단 상단 아니면 다음 단/페이지 이동
            let is_column_top = page.current_height < 1.0;
            !is_column_top && page.current_height + extra > available_height() + 0.5
        } else {
            false
        }
    }
}

/// 호스트 위쪽에서 끝나는 개체는 현재 본문 흐름의 물리 띠를 차지하지 않는다.
/// 개체의 출력 위치는 보존하며, 양수 오프셋 등 다른 앵커의 기존 처리는 유지한다.
fn topbottom_pushdown_height(common: &crate::model::shape::CommonObjAttr, dpi: f64) -> Option<f64> {
    let bottom = i64::from(crate::renderer::float_placement::signed_hwpunit(
        common.vertical_offset,
    )) + i64::from(common.height)
        + i64::from(common.margin.bottom);
    if bottom <= 0 {
        return None;
    }
    Some(hwpunit_to_px(common.height as i32, dpi) + hwpunit_to_px(common.margin.bottom as i32, dpi))
}

/// [#8491] 문단 기준·위 정렬·양수 세로 오프셋 비-TAC 개체가 그 문단 **자기 저장 줄들 아래**에서
/// 시작하면, 문단 위끝에서 개체 위끝까지의 거리(px)를 돌려준다.
///
/// 자리차지 개체는 자기 띠와 겹치는 줄만 그 아래로 민다. 저장 사다리의 모든 줄이 개체 띠보다
/// 위에서 끝나면 한/글은 그 줄들을 밀지 않는다(156602236 문단 14: 표 줄 0..3564 HU, 로고 상자
/// 오프셋 4138 HU → 표와 로고 모두 1쪽). 편집·합성 줄이나 쪽 되감김이 있는 사다리는 판정하지 않는다.
pub(super) fn below_own_lines_top_px(ctrl: &Control, para: &Paragraph, dpi: f64) -> Option<f64> {
    use crate::model::shape::{VertAlign, VertRelTo};
    let common = match ctrl {
        Control::Shape(shape) => shape.common(),
        Control::Picture(picture) => &picture.common,
        _ => return None,
    };
    if common.treat_as_char
        || !matches!(common.vert_rel_to, VertRelTo::Para)
        || !matches!(common.vert_align, VertAlign::Top)
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return None;
    }
    let offset = i64::from(crate::renderer::float_placement::signed_hwpunit(
        common.vertical_offset,
    ));
    if offset <= 0 {
        return None;
    }
    let first = para.line_segs.first()?;
    let mut prev_vpos = first.vertical_pos;
    let mut lines_bottom = 0i64;
    for seg in &para.line_segs {
        if crate::renderer::typeset::is_synthetic_line_seg(seg)
            || seg.line_height <= 0
            || seg.vertical_pos < prev_vpos
        {
            return None;
        }
        prev_vpos = seg.vertical_pos;
        lines_bottom = lines_bottom.max(
            i64::from(seg.vertical_pos) - i64::from(first.vertical_pos)
                + i64::from(seg.line_height),
        );
    }
    (offset >= lines_bottom).then(|| hwpunit_to_px(offset as i32, dpi))
}

/// [#8491] 자기 줄들 아래 개체의 배치: 원점은 문단 위끝 + 오프셋, 흐름은 현재 위치에서 더 소비하지 않는다.
pub(super) fn below_own_lines_frame(
    para_top: f64,
    shape_top: f64,
    flow_bottom: f64,
) -> crate::renderer::float_placement::ParagraphFloatPlacement {
    use crate::renderer::float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement};
    ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: para_top,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: shape_top,
        occupied_bottom: flow_bottom,
    }
}
