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
    /// 쪽 수용 판정용 자기 줄 높이. 줄 뒤 간격은 다음 줄까지의 거리라서 쪽 끝 줄의
    /// 수용 판정에 넣지 않는다(일반 문단 `height_for_fit` 과 같은 계약, Task #359).
    pub tac_separate_line_fit_h: Option<f64>,
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
    let tac_separate_line: Option<(f64, f64)> = match ctrl {
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
            (lh + ls_extra, lh)
        })
    });
    let tac_separate_line_h = tac_separate_line.map(|(advance, _)| advance);
    let tac_separate_line_fit_h = tac_separate_line.map(|(_, fit)| fit);
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
        tac_separate_line_fit_h,
        non_tac_pushdown_h,
    }
}

impl TableHostShapeFlow {
    /// [#8435] 앵커 줄을 밀어낸 개체는 저장 사다리가 앵커 줄 자리(`origin`)를 이미
    /// 확정했다. 세로 오프셋도 그 자리 기준이므로 별도 앵커 해석 없이 상자를 만든다.
    pub(super) fn pushed_anchor_shape_frame(
        &self,
        control: &Control,
        origin: f64,
        dpi: f64,
    ) -> Option<crate::renderer::float_placement::ParagraphFloatPlacement> {
        use crate::renderer::float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement};
        let Control::Shape(shape) = control else {
            return None;
        };
        let height = self.non_tac_pushdown_h?;
        let top = origin
            + hwpunit_to_px(
                crate::renderer::float_placement::signed_hwpunit(shape.common().vertical_offset),
                dpi,
            );
        Some(ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: origin,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top + height,
        })
    }

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
        if let Some(advance_h) = self.tac_separate_line_h {
            // 자기 line이 현재 페이지에 들어가지 않으면 다음 페이지로 분할.
            // 쪽 끝 줄은 줄 높이까지만 들어가면 된다 — 저장 vpos+lh 가 본문 높이 이하인
            // 줄을 한컴은 같은 쪽에 둔다(156578788 줄1 66269+3736=70005 ≤ 70020).
            // 자기 줄 판정은 `tac_separate_line_h` 하나가 켜고 끈다(호출자가 이를 비우면
            // 수용 판정도 함께 꺼진다).
            let line_h = self.tac_separate_line_fit_h.unwrap_or(advance_h);
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

/// 개체의 제어 문자가 실린 저장 줄이 앞 줄보다 위(쪽 되감김)에서 시작하는가.
/// 저장 사다리가 그 줄을 다음 쪽 첫머리에 둔다는 뜻이다. 편집·합성 줄은 판정하지 않는다.
pub(super) fn anchor_line_rewinds_to_next_page(para: &Paragraph, ctrl_idx: usize) -> bool {
    if para.stored_text_partition_is_dirty() || para.cell_format_vpos_dirty {
        return false;
    }
    let Some(line) =
        crate::renderer::layout::control_line_seg_index(para, ctrl_idx).filter(|&line| line > 0)
    else {
        return false;
    };
    let (Some(prev), Some(own)) = (para.line_segs.get(line - 1), para.line_segs.get(line)) else {
        return false;
    };
    let stored = |seg: &crate::model::paragraph::LineSeg| {
        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
    };
    stored(prev) && stored(own) && own.vertical_pos < prev.vertical_pos
}

/// 현재 쪽이 아직 앵커 줄 **앞** 줄의 쪽인가 — 이 문단의 앞 줄 개체가 있고 앵커 줄
/// 이후의 개체는 아직 없다. 같은 줄의 다른 개체가 먼저 다음 쪽을 열었으면 그 쪽은
/// 떠나는 쪽이 아니다(156719599: 같은 줄 TAC 표가 먼저 2쪽에 놓인다).
pub(super) fn page_still_holds_lines_before_anchor(
    para_idx: usize,
    para: &Paragraph,
    ctrl_idx: usize,
    items: &[crate::renderer::pagination::PageItem],
) -> bool {
    use crate::renderer::pagination::PageItem;
    let Some(anchor_line) = crate::renderer::layout::control_line_seg_index(para, ctrl_idx) else {
        return false;
    };
    let mut before = false;
    for item in items {
        let control = match item {
            PageItem::Table {
                para_index,
                control_index,
            }
            | PageItem::PartialTable {
                para_index,
                control_index,
                ..
            }
            | PageItem::Shape {
                para_index,
                control_index,
            } if *para_index == para_idx => *control_index,
            _ => continue,
        };
        match crate::renderer::layout::control_line_seg_index(para, control) {
            Some(line) if line < anchor_line => before = true,
            _ => return false,
        }
    }
    before
}

/// 개체의 제어 문자가 실린 저장 줄이 이미 이 쪽의 글줄 항목(전체·부분 문단)으로 놓였는가.
/// 그 줄의 높이는 글줄 항목이 소비했다.
pub(super) fn own_line_already_placed(
    para_idx: usize,
    para: &Paragraph,
    ctrl_idx: usize,
    items: &[crate::renderer::pagination::PageItem],
) -> bool {
    use crate::renderer::pagination::PageItem;
    let Some(line) = crate::renderer::layout::control_line_seg_index(para, ctrl_idx) else {
        return false;
    };
    items.iter().any(|item| match item {
        PageItem::PartialParagraph {
            para_index,
            start_line,
            end_line,
        } => *para_index == para_idx && (*start_line..*end_line).contains(&line),
        PageItem::FullParagraph { para_index } => *para_index == para_idx,
        _ => false,
    })
}

/// 문단 기준(세로=문단) 비-TAC 개체의 위끝: 문단 위끝에서 세로 오프셋만큼(px). 위 정렬만 다룬다.
pub(super) fn para_offset_top_px(ctrl: &Control, dpi: f64) -> Option<f64> {
    use crate::model::shape::{VertAlign, VertRelTo};
    let common = match ctrl {
        Control::Shape(shape) => shape.common(),
        Control::Picture(picture) => &picture.common,
        _ => return None,
    };
    if common.treat_as_char
        || !matches!(common.vert_rel_to, VertRelTo::Para)
        || !matches!(common.vert_align, VertAlign::Top)
    {
        return None;
    }
    let offset = crate::renderer::float_placement::signed_hwpunit(common.vertical_offset);
    (offset > 0).then(|| hwpunit_to_px(offset, dpi))
}

/// 이 쪽 본문 아래로 오프셋된 개체의 배치: 원점은 문단 위끝, 흐름은 현재 위치에서 더 소비하지 않는다.
pub(super) fn offpage_shape_frame(
    para_top: f64,
    offset_top: f64,
    flow_bottom: f64,
) -> crate::renderer::float_placement::ParagraphFloatPlacement {
    use crate::renderer::float_placement::{ParagraphFloatFlow, ParagraphFloatPlacement};
    ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: para_top,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: para_top + offset_top,
        occupied_bottom: flow_bottom,
    }
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
