//! Flow reservation helpers for non-inline floating objects.

use std::ops::Range;

use crate::model::control::Control;
use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, TextFlow, TextWrap, VertAlign, VertRelTo,
};
use crate::model::table::{Table, TablePageBreak};
use crate::model::HwpUnit;

use super::hwpunit_to_px;
use super::layout::picture_flow_frame_size_hu;
use super::layout_frame::{FrameExclusion, FrameExclusionPolicy};
use super::page_layout::LayoutRect;

/// A paper/page-anchored side-wrap float that can explain a stored body row's
/// missing right-side width.
///
/// Width alone is deliberately insufficient: a same-width object on another
/// vertical band must not make an unrelated paragraph keep stale narrow rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FloatCarveEvidence {
    pub(crate) width: i32,
    pub(crate) vertical: Range<i32>,
}

impl FloatCarveEvidence {
    /// A stored row may use this evidence only when its missing width matches
    /// and at least one of the paragraph's stored vertical bands intersects the
    /// float. A same-width object elsewhere is not an exclusion provenance.
    pub(crate) fn matches_stored_rows(
        &self,
        missing_width: i32,
        rows: &[crate::model::paragraph::LineSeg],
        tolerance: i32,
    ) -> bool {
        (missing_width - self.width).abs() <= tolerance
            && rows.iter().any(|segment| {
                let top = segment.vertical_pos;
                let bottom = top.saturating_add(segment.line_height.max(1));
                top < self.vertical.end && self.vertical.start < bottom
            })
    }
}

/// Interpret an HWPUNIT value that may have been stored through a signed field.
pub(crate) fn signed_hwpunit(value: HwpUnit) -> i32 {
    value as i32
}

/// Resolve the deliberately small Picture/Square side-wrap subset used by a
/// caller-owned `LayoutFrame`.
///
/// The caller owns both the paragraph-relative anchor and the exclusion's
/// lifetime. This function intentionally has no fallback policy: only an
/// uncaptained, non-TAC Picture with `Square` and the two recovered side-wrap
/// flows has a physical-row representation. Every other object shape remains
/// with its existing owner.
/// [#6175] 용지/쪽 기준 어울림 개체의 흐름 폭과 세로 band(HWPUNIT, 바깥 여백 포함).
///
/// `stored_rows_require_external_geometry` 가 저장 행의 결손 폭과 같은 **세로 band**의
/// 이 값을 함께 대조해, 균일하게 좁은 저장 행의 좁음이 문단 자신의 테두리 inset에서
/// 온 것인지 외부 개체에서 온 것인지 가른다. 셀에서는 #5818 이 같은 혼동을 같은 셀의
/// Square float 실재로 갈랐고, 이것은 그 계약의 본문 판이다.
///
/// 문단 기준 개체는 `resolve_picture_exclusion`의 caller-owned frame이 직접 소유한다.
/// 여기서는 그 frame이 지원하지 않는 용지/쪽 기준 개체만 수집한다.
pub(crate) fn paper_or_page_float_carve_evidence(
    paragraphs: &[crate::model::paragraph::Paragraph],
) -> Vec<FloatCarveEvidence> {
    use crate::model::control::Control;
    let mut evidence = Vec::new();
    for para in paragraphs {
        for control in &para.controls {
            let common = match control {
                Control::Picture(picture) => &picture.common,
                Control::Shape(shape) => shape.common(),
                Control::Table(table) => &table.common,
                _ => continue,
            };
            if common.treat_as_char
                || !matches!(
                    common.text_wrap,
                    TextWrap::Square | TextWrap::Tight | TextWrap::Through
                )
            {
                continue;
            }
            if !matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
                || !matches!(common.vert_align, VertAlign::Top | VertAlign::Inside)
            {
                continue;
            }
            let width = (common.width as i32)
                .saturating_add(i32::from(common.margin.left))
                .saturating_add(i32::from(common.margin.right));
            let height = (common.height as i32)
                .saturating_add(i32::from(common.margin.top))
                .saturating_add(i32::from(common.margin.bottom));
            let top = signed_hwpunit(common.vertical_offset);
            let vertical = top..top.saturating_add(height);
            let candidate = FloatCarveEvidence { width, vertical };
            if width > 0 && height > 0 && !evidence.contains(&candidate) {
                evidence.push(candidate);
            }
        }
    }
    evidence
}

/// [#6202] 용지 기준(`Paper`/`Page`) 어울림 개체를 재는 데 필요한 쪽 원점.
///
/// `Column`/`Para` 기준은 문단 밴드 안에서 끝나지만, 용지 기준은 본문 상자가 용지 안
/// 어디에 있는지를 알아야 한다. 그 값 **두 개**면 공식은 같다 — 기준점만 바뀐다.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PaperOrigin {
    /// 본문 상자의 용지 기준 왼쪽 (HWPUNIT).
    pub(crate) body_left: i32,
    /// 본문 상자의 용지 기준 위쪽 (HWPUNIT).
    pub(crate) body_top: i32,
}

pub(crate) fn resolve_picture_exclusion(
    picture: &Picture,
    column_horizontal: Range<i32>,
    paragraph_horizontal: Range<i32>,
    paragraph_top: i32,
    paper: PaperOrigin,
) -> Option<FrameExclusion> {
    let common = &picture.common;
    // [#6202] 용지 기준 개체도 잰다. 코퍼스 1,997건 표본에서 Square float 개체를
    // 막는 관문은 사실상 `horz_rel`/`vert_rel` 의 `Paper|Page` 뿐이었다(각 57건).
    //
    // 156483689 실측 — 계산이 저장 사다리를 **1 HU 오차로 재현**한다:
    //
    // ```text
    //   그림  w=12482 h=9366  vert=Paper(36844) horz=Paper(42333)
    //   본문  왼쪽 5670 HU · 폭 48188 HU
    //   가로  깎인 줄 오른끝 5670+36664 = 42334   vs  그림 왼쪽 42333   (1 HU)
    //   세로  밴드 36844−5670 .. +9366 = 31174..40540
    //         pi=5 29631..32031 깎임 · pi=6 34431 깎임 · pi=7 에서 40540 지나 전폭 복귀
    // ```
    let paper_relative_horz = matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page);
    let paper_relative_vert = matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page);
    if common.treat_as_char
        // [#6202] 캡션이 붙은 개체도 잰다 — 156483689 실측에서 밴드 높이는 캡션을 뺀
        // 그림 높이(9366 HU) 그대로이고, 저장 사다리의 깎임 끝(40540 HU)과 일치한다.
        || common.text_wrap != TextWrap::Square
        || !matches!(
            common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para | HorzRelTo::Paper | HorzRelTo::Page
        )
        || !matches!(
            common.vert_rel_to,
            VertRelTo::Para | VertRelTo::Paper | VertRelTo::Page
        )
        || !matches!(common.vert_align, VertAlign::Top | VertAlign::Inside)
    {
        return None;
    }
    let policy = match common.text_flow {
        TextFlow::BothSides => FrameExclusionPolicy::BothSides,
        TextFlow::LargestOnly => FrameExclusionPolicy::LargestSide,
        TextFlow::LeftOnly | TextFlow::RightOnly => return None,
    };

    let (width, height) = picture_flow_frame_size_hu(picture);
    if column_horizontal.is_empty() || paragraph_horizontal.is_empty() || width <= 0 || height <= 0
    {
        return None;
    }

    let horizontal_offset = signed_hwpunit(common.horizontal_offset);
    let column_end = column_horizontal.end;
    let reference = match common.horz_rel_to {
        HorzRelTo::Column => column_horizontal,
        HorzRelTo::Para => paragraph_horizontal,
        // 용지 기준은 본문 왼쪽을 빼 컬럼 좌표로 옮긴다 — 이후 공식은 동일하다.
        HorzRelTo::Paper | HorzRelTo::Page => {
            (-paper.body_left)..(column_end.saturating_sub(paper.body_left))
        }
    };
    let reference_width = reference.end.saturating_sub(reference.start);
    let visible_left = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => reference.start.saturating_add(horizontal_offset),
        HorzAlign::Center => reference
            .start
            .saturating_add(
                reference_width
                    .saturating_sub(width)
                    .max(0)
                    .saturating_div(2),
            )
            .saturating_add(horizontal_offset),
        HorzAlign::Right | HorzAlign::Outside => reference
            .end
            .saturating_sub(width)
            .saturating_sub(horizontal_offset),
    };
    let visible_top = if paper_relative_vert {
        // 용지 기준 세로는 문단 앵커가 아니라 본문 위쪽에서 잰다.
        signed_hwpunit(common.vertical_offset).saturating_sub(paper.body_top)
    } else {
        paragraph_top.saturating_add(signed_hwpunit(common.vertical_offset))
    };
    let _ = paper_relative_horz;
    let horizontal = visible_left.saturating_sub(i32::from(common.margin.left))
        ..visible_left
            .saturating_add(width)
            .saturating_add(i32::from(common.margin.right));
    let vertical = visible_top.saturating_sub(i32::from(common.margin.top))
        ..visible_top
            .saturating_add(height)
            .saturating_add(i32::from(common.margin.bottom));

    (!horizontal.is_empty() && !vertical.is_empty()).then_some(FrameExclusion {
        horizontal,
        vertical,
        policy,
    })
}

/// A non-TAC `TopAndBottom` object positioned from its host paragraph.
pub(crate) fn is_para_topbottom_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Para)
}

/// A positive-offset empty host float whose next, generated body paragraph has
/// no stored line-segment anchor. Hancom consumes the empty host's physical row,
/// lays that body paragraph in the remaining gap above the float, then resumes
/// ordinary flow below the float.
///
/// Returns the stored host row's `(vertical_pos, line_height + line_spacing)`.
/// The first coordinate anchors the float; their sum anchors the generated text.
pub(crate) fn empty_offset_float_deferred_text_ladder_hu(
    host: &Paragraph,
    table: &Table,
    following: &Paragraph,
) -> Option<(i32, i32)> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '\u{FFFC}')
    };

    let qualifies = is_para_topbottom_float(&table.common)
        && table.common.flow_with_text
        && signed_hwpunit(table.common.vertical_offset) > 0
        && host.controls.len() == 1
        && !has_non_whitespace_text(host)
        && has_non_whitespace_text(following)
        // HWPX 원본에 linesegarray가 없어도 parser가 계산 segment를 보충한다.
        // 저장 segment가 하나라도 있으면 그 위치를 우선해야 하므로 제외한다.
        && following.line_segs.iter().all(|segment| {
            segment.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        });
    qualifies.then(|| {
        host.line_segs
            .iter()
            .find(|segment| {
                segment.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            .map(|segment| {
                (
                    segment.vertical_pos,
                    segment
                        .line_height
                        .saturating_add(segment.line_spacing)
                        .max(0),
                )
            })
    })?
}

/// [#6366] 원본 HWPX 문단 기준 글앞으로 다행·다열 표가 `flowWithText` 이면
/// 데코레이션 Shape 단축에서 빼 쪽 분할에 참여한다.
///
/// 모든 `flowWithText` 글앞으로/글뒤로 표에 열면 #5918 쪽수가 늘고
/// text-overlap 기준선이 커진다. 한글 6쪽 정합 픽스처
/// (`2700727_animal_facility_standards.hwpx` pi=9)만 연다: 원본 HWPX,
/// 비-TAC, IN_FRONT_OF_TEXT, vert=문단, horz=문단, 40행 이상 6열 이상.
/// #5918 의 4×5·31×7 글앞으로 표는 데코레이션으로 남긴다.
pub(crate) fn original_hwpx_infront_para_flow_paginates(
    original_hwpx: bool,
    table: &Table,
) -> bool {
    original_hwpx
        && !table.common.treat_as_char
        && table.common.flow_with_text
        && matches!(table.common.text_wrap, TextWrap::InFrontOfText)
        && matches!(table.common.vert_rel_to, VertRelTo::Para)
        && matches!(table.common.horz_rel_to, HorzRelTo::Para)
        && table.row_count >= 40
        && table.col_count >= 6
}

/// Stored host-line evidence for the narrow native-HWP RowBreak flow contract (#2439).
///
/// The returned value is the non-synthetic stored line advance in HWPUNIT.  Callers may combine
/// it with the positive object offset for pagination, or with the painted lane bottom and outer
/// bottom for layout.  Keeping the structural predicate here prevents typeset/full/partial layout
/// from drifting apart.  A broad empty-host outer-margin rule is disproven by #2097.
pub(crate) fn native_empty_host_rowbreak_line_advance_hu(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    if !native_hwp5_layout
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || has_non_whitespace_text(para)
        || !matches!(table.common.vert_rel_to, VertRelTo::Para)
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || signed_hwpunit(table.common.vertical_offset) <= 0
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
        || para
            .controls
            .iter()
            .filter(|control| {
                matches!(control, Control::Table(candidate)
                    if is_para_topbottom_float(&candidate.common))
            })
            .count()
            != 1
        || !next_para.is_some_and(|next| has_non_whitespace_text(next) && next.controls.is_empty())
    {
        return None;
    }

    let host_seg = para
        .line_segs
        .iter()
        .find(|seg| seg.tag & 0x80000000 == 0 && seg.line_height > 0)?;
    let advance = host_seg.line_height + host_seg.line_spacing.max(0);
    if advance <= 0 {
        return None;
    }
    // [#2808] 저장 vpos ladder 로 한컴이 host 줄 advance 를 실제 흐름에 계상했는지
    // 검증한다. #2439 재현 문서(기계 반복 양식)는 ladder 가 표 높이를 접고
    // `next.vpos - host.vpos == advance` 로 저장되는 반면(= advance 가 실 흐름 증거),
    // 일반 물리 ladder 문서는 델타가 표 높이+offset 을 이미 포함하므로 advance 를
    // 다시 더하면 이중 계상되어 쪽 경계 한 줄이 +1 로 밀린다 (10k r19 회귀 4건).
    let next_vpos = next_para
        .and_then(|next| {
            next.line_segs
                .iter()
                .find(|seg| seg.tag & 0x80000000 == 0 && seg.line_height > 0)
        })
        .map(|seg| seg.vertical_pos)?;
    if (next_vpos - host_seg.vertical_pos - advance).abs() > 1 {
        return None;
    }
    Some(advance)
}

/// [#6147] 저장 사다리가 "빈 앵커 문단의 줄 하나"만 증언하는 자리차지 밴드의 host 줄 계약.
///
/// 한글은 자리차지(TopAndBottom) 개체를 매단 **빈 앵커 문단**도 개체 아래에 자기 줄
/// 상자(`lh + ls`)를 차지한다. rhwp 는 #1147 이래 이 줄을 일괄 억제해 왔고(빈 앵커 vpos 가
/// 이미 갭을 인코딩한다는 전제), 그래서 밴드 바로 아래 첫 본문 문단이 개체에 딱 붙는다.
///
/// 억제가 옳은 문단과 아닌 문단은 **저장 사다리가 가른다** — `next.vpos - host.vpos` 가
/// 정확히 `lh + max(ls, 0)` 이면 한글이 개체 높이를 접고 host 줄 advance 만 흐름에
/// 계상했다는 뜻이라, 그 줄은 별도로 더해야 할 실 흐름이다(= #1147 의 "vpos 가 이미
/// 갭을 인코딩" 전제가 성립하지 않는 문단). 델타가 개체 높이를 품은 일반 물리 사다리는
/// 등식이 깨져 자연 배제된다 — #2439 의 [#2808] 판별자와 같은 축이다.
///
/// #2439(`native_empty_host_rowbreak_line_advance_hu`)는 이 계약의 **단일 표·양수 offset·
/// RowBreak·native HWP5** 특수형이고, 이 함수는 같은 증거를 HWPX 저장 레이아웃과 다중
/// 자리차지 개체(보도자료 서식의 머리표 2~3개)로 넓힌다. 개체가 여럿이면 마지막 자리차지
/// 개체에서만 계상해 밴드마다 중복 가산되지 않게 한다.
pub(crate) fn stored_empty_anchor_band_host_line_advance_hu(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    // host 글자가 **한 자도 없어야** 한다. 공백 한 칸이라도 있으면 #1147 억제가
    // 애초에 걸리지 않아 조판이 이미 host 줄을 계상하고 있고(156272593 pi=44:
    // `text=" "` → `PartialParagraph` 항목 존재), 여기서 또 더하면 이중 계상이라
    // 쪽이 하나 늘어난다(코퍼스 4,000 표본 유일 회귀). 판정을 #1147 의 조판 술어
    // (`para.text.is_empty()`)와 정확히 같은 축에 둔다.
    if !stored_layout || !para.text.is_empty() {
        return None;
    }
    stored_anchor_band_host_line_from_ladder(para, control_index, next_plain_text_vpos(next_para))
}

/// [#6312] 글이 있는 자리차지 host 문단의 저장 사다리가 host 줄만 증언하면
/// 표 밴드 아래에 그 줄 상자(`lh + ls`)를 계상한다.
///
/// #6147 은 빈 앵커(`text.is_empty()`)에만 발동한다. 글이 있으면 #1147 억제가
/// 꺼져 조판이 host 줄을 이미 계상한다고 가정했는데, `is_current_visible_para_float`
/// 경로는 표 뒤에 줄 높이/줄간격을 건너뛰어 다음 문단이 표에 붙는다
/// (156721992 1쪽: 한글 27.0pt 자리, rhwp 0). 같은 사다리 등식
/// `next.vpos - host.vpos == lh+ls` 가 표 높이를 접었다는 뜻이라 표 높이를
/// 다시 더하지 않는다(#4090 이중 계상 차단과 같은 축).
pub(crate) fn stored_visible_anchor_band_host_line_advance_hu(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    stored_visible_anchor_band_host_line_advance_from_vpos(
        stored_layout,
        para,
        control_index,
        next_plain_text_vpos(next_para),
    )
}

pub(crate) fn stored_visible_anchor_band_host_line_advance_from_vpos(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_plain_text_vpos: Option<i32>,
) -> Option<i32> {
    if !stored_layout || !para_has_non_whitespace_text(para) {
        return None;
    }
    stored_anchor_band_host_line_from_ladder(para, control_index, next_plain_text_vpos)
}

fn next_plain_text_vpos(next_para: Option<&Paragraph>) -> Option<i32> {
    let next = next_para?;
    if !para_has_non_whitespace_text(next) || !next.controls.is_empty() {
        return None;
    }
    next.line_segs
        .iter()
        .enumerate()
        .find(|(_, seg)| seg.tag & 0x8000_0000 == 0 && seg.line_height > 0)
        .map(|(index, seg)| ladder_vpos(next, index, seg.vertical_pos))
}

fn ladder_vpos(paragraph: &Paragraph, index: usize, fallback: i32) -> i32 {
    paragraph
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|source| source.get(index).copied())
        .unwrap_or(fallback)
}

fn stored_anchor_band_host_line_from_ladder(
    para: &Paragraph,
    control_index: usize,
    next_plain_text_vpos: Option<i32>,
) -> Option<i32> {
    // 문단의 가시 개체가 전부 비-TAC 자리차지(vert=문단) float 이어야 한다 — 인라인
    // 내용이 섞이면 host 줄이 그 내용의 줄이지 앵커 줄이 아니다.
    let mut last_float = None;
    for (index, control) in para.controls.iter().enumerate() {
        let common = match control {
            Control::Table(table) => &table.common,
            Control::Picture(picture) => &picture.common,
            Control::Shape(shape) => shape.common(),
            _ => continue,
        };
        if !is_para_topbottom_float(common) {
            return None;
        }
        last_float = Some(index);
    }
    if last_float != Some(control_index) {
        return None;
    }
    // 다음이 개체 없는 일반 본문 문단일 때만 — 앵커 스택(다음도 빈 앵커)의 줄간격은
    // 개체-개체 간격이라 이미 #1133 이 보존한다.
    let next_vpos = next_plain_text_vpos?;

    // [#6312] 사다리 등식은 재조판 좌표가 아니라 원본 저장 vpos 로 본다.
    let (host_index, host_seg) = para
        .line_segs
        .iter()
        .enumerate()
        .find(|(_, seg)| seg.tag & 0x8000_0000 == 0 && seg.line_height > 0)?;
    let advance = host_seg.line_height + host_seg.line_spacing.max(0);
    if advance <= 0 {
        return None;
    }
    let host_vpos = ladder_vpos(para, host_index, host_seg.vertical_pos);
    ((next_vpos - host_vpos - advance).abs() <= 1).then_some(advance)
}

/// 문단에 공백·개체 마커가 아닌 실제 글자가 있는가.
fn para_has_non_whitespace_text(para: &Paragraph) -> bool {
    para.text
        .chars()
        .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
}

/// [#5922] native HWP5 CellBreak 자리차지 표의 연속 조각 바깥 여백 재개방 계약.
///
/// 한글은 다쪽으로 이어지는 CellBreak 조각을 쪽마다 표 바깥 여백(상·하)을 다시
/// 열어 그린다(화성시 별표2 실측: 본문 상단 42.52pt + 0.5mm 여백 = 괘선 44pt).
/// RowBreak 의 #2439 계약과 달리 저장 ladder 증거를 요구할 수 없다 — 거대 표의
/// 저장 vpos ladder 는 표 높이를 접기 때문이다. 대신 구조를 좁힌다: native HWP5,
/// 비-TAC TopAndBottom(vert=문단), 쪽나눔=CellBreak, 빈 host 문단(표 전용).
/// 바깥 여백이 0 이면 더해질 값이 없어 no-op다.
pub(crate) fn native_empty_host_cellbreak_fragment_repeats_outer_margin(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
) -> bool {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    native_hwp5_layout
        && !table.common.treat_as_char
        && is_para_topbottom_float(&table.common)
        && matches!(table.page_break, TablePageBreak::CellBreak)
        && !has_non_whitespace_text(para)
}

/// [#6378] 원본 HWPX 단 기준 RowBreak 자리차지 표의 사방 균등 outMargin (HU).
///
/// `hwp5_stored_pagination_layout` 이 꺼진 원본 HWPX 는 native HWP5 빈-host
/// RowBreak helper(`native_empty_host_physical_outer_box_paint_inset`)가 표
/// 원점에 싣는 바깥 여백을 주지 않는다. 같은 문서 HWP 경로(`tac-img-02`)는
/// 1mm(283HU) 안쪽에 둔다. HWPX XML `pageBreak="CELL"` 도 이 픽스처에서는
/// IR `RowBreak` 로 들어온다. 모든 원본 HWPX 표·연속 block 표(#1133)에
/// 더하면 간격이 3.8px 줄고 글자 겹침 기준선이 커지므로, native helper 와
/// 같은 형상만 연다: 비-TAC TopAndBottom(vert=문단), 단·왼쪽, RowBreak,
/// 다행 1열, 사방 균등 양의 outMargin, 오프셋 0.
pub(crate) fn original_hwpx_column_rowbreak_equal_outer_margin_hu(
    original_hwpx: bool,
    table: &Table,
) -> Option<i32> {
    let declared_height = signed_hwpunit(table.common.height);
    if !original_hwpx
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count <= 1
        || table.col_count != 1
        || table.cells.len() != usize::from(table.row_count)
        || !table.cells.iter().enumerate().all(|(row, cell)| {
            cell.row == row as u16 && cell.col == 0 && cell.row_span == 1 && cell.col_span == 1
        })
        || signed_hwpunit(table.common.width) <= 0
        || declared_height <= 0
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
        || table.caption.is_some()
    {
        return None;
    }
    Some(i32::from(table.outer_margin_left))
}

/// [#5870] 빈 host 자리차지 float 의 저장 사다리가 **물리 공식과 정확히 일치**하는지 —
/// `next.vpos - host.vpos == v_off + outer_top + 선언높이 + outer_bottom` (±2HU) — 를
/// 검증하고, 일치하면 흐름에 더 계상해야 할 여분(`v_off + outer_top + outer_bottom`, HU)을
/// 돌려준다. 한글은 이 형상의 흐름을 그 합만큼 전진시키는데(10645 [별지 제11호서식]
/// 40쪽: 저장 델타 8413 = 1840+140+6293+140 정확 일치), rhwp 의 일반 빈-앵커 계약은
/// 표 높이만 전진시켜 다음 float 가 위로 올라와 겹쳤다(19.7px). 광역 "빈 host 에
/// v_off+outer 가산"은 #2097 이 반증했으므로(82802 pi75 는 outer 를 한 번만 담아 저장 —
/// 이 등식에 걸리지 않는다) 문단 단위 저장 증거를 게이트로 쓴다.
///
/// 추가 조임 두 겹 — ① RowBreak 표 제외: 빈-host RowBreak 는 별도 저장 계약군
/// (#2439·#3931·#3820 rowbreak tail 등)이 흐름·조각을 이미 다뤄 이중 계상이 된다
/// (r 게이트 실측: 3931×3·3930·3820·5699·5801·3565 회귀). ② 호출부는 **다음 문단도
/// 빈 float 표 앵커**일 때만 발동한다 — 이 결함의 실증상이 float 뒤 float 겹침이고,
/// 후속이 텍스트면 저장 vpos 재고정으로 어차피 무결하다. 나머지 구조 조건(빈 host·
/// 단일 표·비합성 lineseg·프로파일)도 호출부가 확인한다.
pub(crate) fn empty_host_physical_ladder_extras_hu(
    table: &Table,
    host_vpos: i32,
    next_vpos: i32,
) -> Option<i64> {
    if matches!(table.page_break, TablePageBreak::RowBreak) {
        return None;
    }
    let extras = i64::from(signed_hwpunit(table.common.vertical_offset).max(0))
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    if extras <= 0 {
        return None;
    }
    let stored_delta = i64::from(next_vpos) - i64::from(host_vpos);
    let physical_delta = extras + i64::from(table.common.height.min(i32::MAX as u32));
    ((stored_delta - physical_delta).abs() <= 2).then_some(extras)
}

/// [#3931] native HWP5 다행 RowBreak 표가 cell 내부 저장 page reset을 갖고,
/// 후속 source 문단도 host anchor 위로 되감기는 빈-host 형상인지 판별한다.
///
/// 반환값은 host의 저장 line advance다. 첫 fragment를 앞선 표 바로 뒤에 그릴 때
/// layout cursor에 남은 이전 host trailing margin이 이 advance 이내면 회수할 수
/// 있다는 구조 증거로 사용한다.
pub(crate) fn native_multirow_internal_reset_rowbreak_anchor_advance_hu(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    if !native_hwp5_layout
        || table.row_count <= 1
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || has_non_whitespace_text(para)
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return None;
    }

    let host_seg = para.line_segs.iter().find(|seg| {
        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && seg.line_height > 0
    })?;
    let next_seg = next_para?
        .line_segs
        .iter()
        .find(|seg| seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)?;
    if next_seg.vertical_pos >= host_seg.vertical_pos {
        return None;
    }

    let has_internal_reset = table.cells.iter().any(|cell| {
        let mut previous_vpos = None;
        for cell_para in &cell.paragraphs {
            for seg in cell_para.line_segs.iter().filter(|seg| {
                seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            }) {
                if previous_vpos.is_some_and(|previous| previous > 0 && seg.vertical_pos <= 0) {
                    return true;
                }
                previous_vpos = Some(seg.vertical_pos);
            }
        }
        false
    });
    if !has_internal_reset {
        return None;
    }

    let advance = host_seg.line_height + host_seg.line_spacing.max(0);
    (advance > 0).then_some(advance)
}

/// Native HWP5가 빈 host의 저장 LINE_SEG 사다리에 표의 outer box 전체를 기록한
/// 경우만 paint origin에 outer-left/top을 복원한다.
///
/// 모든 empty-host 표에 outer margin을 더하는 규칙은 #2097 실물과 충돌한다. 이
/// helper는 표 높이와 위·아래 outer margin의 합이 다음 실제 저장 vpos와 정확히
/// 일치하는 단일 whole-table 형상만 식별한다. Pagination/flow는 이미 이 outer box를
/// 예약하므로 caller는 paint subtree만 이동해야 한다.
pub(crate) fn native_empty_host_physical_outer_box_paint_inset(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> bool {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    let declared_height = signed_hwpunit(table.common.height);
    if !native_hwp5_layout
        || has_non_whitespace_text(para)
        || para.controls.len() != 1
        || !matches!(para.controls.first(), Some(Control::Table(_)))
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count <= 1
        || table.col_count != 1
        || table.cells.len() != usize::from(table.row_count)
        || !table.cells.iter().enumerate().all(|(row, cell)| {
            cell.row == row as u16
                && cell.col == 0
                && cell.row_span == 1
                && cell.col_span == 1
        })
        || signed_hwpunit(table.common.width) <= 0
        || declared_height <= 0
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        // 저장 vpos 사다리는 세로 outer box만 직접 증명한다. p120처럼 네 방향
        // margin이 같은 경우에만 그 증거를 수평 paint inset까지 확장한다.
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
        || table.caption.is_some()
        || next_para.is_some_and(|next| has_non_whitespace_text(next) || !next.controls.is_empty())
    {
        return false;
    }

    fn stored_seg(paragraph: &Paragraph) -> Option<&crate::model::paragraph::LineSeg> {
        paragraph.line_segs.iter().find(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && seg.line_height > 0
        })
    }
    let Some(host_seg) = stored_seg(para) else {
        return false;
    };
    let Some(next_seg) = next_para.and_then(stored_seg) else {
        return false;
    };
    let stored_advance = i64::from(next_seg.vertical_pos) - i64::from(host_seg.vertical_pos);
    let physical_outer_height = i64::from(declared_height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    stored_advance > 0 && (stored_advance - physical_outer_height).abs() <= 1
}

/// Paint-only geometry for the narrow native-HWP5 stored-reset table fragment contract.
///
/// These 1x1 RowBreak tables store the first physical fragment height in
/// `CommonObjAttr::height`, then restart the cell LINE_SEG ladder at `vpos=0` in the next
/// paragraph.  The paginator deliberately keeps its composed trailing line spacing for flow
/// ownership, but the painted first-fragment frame must stop at the stored height.  Both physical
/// fragments also paint inside the equal four-way outer margin already reserved by flow.
///
/// Callers must use this result only to change the paint subtree.  It is not a pagination or flow
/// height contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeStoredResetFragmentPaintGeometry {
    pub(crate) outer_left_hu: i32,
    pub(crate) outer_top_hu: i32,
    /// `Some` only for the first fragment.  A successor receives the same origin inset but keeps
    /// its measured fragment height.
    pub(crate) first_fragment_height_hu: Option<i32>,
}

/// Recognize one physical fragment of a native-HWP5 1x1 stored-reset RowBreak table.
///
/// The predicate intentionally contains no paragraph, table, or fixture identifier.  In
/// particular, the declared head height must be independently proven by the last stored line
/// before the cross-paragraph rewind plus the effective vertical cell padding.  The fragment cut
/// must then meet that exact rewind boundary.
pub(crate) fn native_hwp5_stored_reset_fragment_paint_geometry(
    native_hwp5_layout: bool,
    host_para: &Paragraph,
    table: &Table,
    is_continuation: bool,
    start_cut: &[usize],
    end_cut: &[usize],
) -> Option<NativeStoredResetFragmentPaintGeometry> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    let cell = table.cells.first()?;
    let declared_height_hu = signed_hwpunit(table.common.height);
    if !native_hwp5_layout
        || has_non_whitespace_text(host_para)
        || host_para.controls.len() != 1
        || !matches!(host_para.controls.first(), Some(Control::Table(_)))
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || cell.row != 0
        || cell.col != 0
        || cell.row_span != 1
        || cell.col_span != 1
        || signed_hwpunit(table.common.width) <= 0
        || declared_height_hu <= 0
        || table.caption.is_some()
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
    {
        return None;
    }

    let effective_padding = cell.effective_padding(&table.padding);
    if effective_padding.top < 0 || effective_padding.bottom < 0 {
        return None;
    }

    // Count only real stored text lines.  A composed atom/spacer makes the count diverge from the
    // RowCut and is therefore rejected by the exact fragment-boundary check below.
    let mut previous: Option<(usize, &crate::model::paragraph::LineSeg)> = None;
    let mut stored_lines_before = 0usize;
    let mut reset_witness = None;
    'paragraphs: for (para_index, paragraph) in cell.paragraphs.iter().enumerate() {
        for seg in paragraph.line_segs.iter().filter(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && seg.text_height > 0
        }) {
            if let Some((previous_para_index, previous_seg)) = previous {
                if previous_para_index != para_index
                    && previous_seg.vertical_pos > 0
                    && seg.vertical_pos == 0
                {
                    reset_witness = Some((stored_lines_before, previous_seg));
                    break 'paragraphs;
                }
            }
            stored_lines_before += 1;
            previous = Some((para_index, seg));
        }
    }
    let (reset_unit_end, previous_seg) = reset_witness?;
    let stored_head_height_hu = i64::from(previous_seg.vertical_pos)
        + i64::from(previous_seg.text_height)
        + i64::from(effective_padding.top)
        + i64::from(effective_padding.bottom);
    if (stored_head_height_hu - i64::from(declared_height_hu)).abs() > 1 {
        return None;
    }

    let is_first_fragment = !is_continuation
        && start_cut.is_empty()
        && end_cut.len() == 1
        && end_cut[0] == reset_unit_end;
    let is_final_successor = is_continuation
        && start_cut.len() == 1
        && start_cut[0] == reset_unit_end
        && end_cut.is_empty();
    if !is_first_fragment && !is_final_successor {
        return None;
    }

    Some(NativeStoredResetFragmentPaintGeometry {
        outer_left_hu: i32::from(table.outer_margin_left),
        outer_top_hu: i32::from(table.outer_margin_top),
        first_fragment_height_hu: is_first_fragment.then_some(declared_height_hu),
    })
}

/// [Task #1658 v3] 페이지 하단 고정(vert=쪽·valign=Bottom) 자리차지 개체 (결재/서명 틀).
/// 한글은 이를 본문 하단에 절대배치(겹침 허용)하고 본문 텍스트를 그 위까지만 흐르게
/// 한다(하단 배타 영역) — 문서순 flow 소비 대상이 아니다. #1653 RCA 패턴 B.
pub(crate) fn is_page_bottom_fixed_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Page)
        && matches!(common.vert_align, VertAlign::Bottom)
}

/// Horizontal reference data used by float placement and table layout.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FloatPlacementContext {
    pub col_area: LayoutRect,
    pub body_area: Option<LayoutRect>,
    pub paper_width: Option<f64>,
    pub host_margin_left: f64,
    pub host_margin_right: f64,
}

impl FloatPlacementContext {
    pub(crate) fn new(col_area: LayoutRect) -> Self {
        Self {
            col_area,
            body_area: None,
            paper_width: None,
            host_margin_left: 0.0,
            host_margin_right: 0.0,
        }
    }

    pub(crate) fn with_body_area(mut self, body_area: LayoutRect) -> Self {
        self.body_area = Some(body_area);
        self
    }

    pub(crate) fn with_paper_width(mut self, paper_width: f64) -> Self {
        self.paper_width = Some(paper_width);
        self
    }

    pub(crate) fn with_host_margins(mut self, left: f64, right: f64) -> Self {
        self.host_margin_left = left;
        self.host_margin_right = right;
        self
    }
}

/// Compute the same depth-0 horizontal range used by table layout.
pub(crate) fn horizontal_range(
    common: &CommonObjAttr,
    width_px: f64,
    ctx: FloatPlacementContext,
    dpi: f64,
) -> (f64, f64) {
    let h_offset = hwpunit_to_px(signed_hwpunit(common.horizontal_offset), dpi);
    let col_area = ctx.col_area;
    let (ref_x, ref_w) = match common.horz_rel_to {
        HorzRelTo::Paper => {
            let fallback_paper_w = if width_px > col_area.width {
                col_area.x * 2.0 + width_px
            } else {
                col_area.x * 2.0 + col_area.width
            };
            let paper_w = ctx.paper_width.unwrap_or(fallback_paper_w);
            (0.0, paper_w)
        }
        HorzRelTo::Page => ctx
            .body_area
            .filter(|body| body.width > 0.0)
            .map(|body| (body.x, body.width))
            .unwrap_or((col_area.x, col_area.width)),
        HorzRelTo::Para => (
            col_area.x + ctx.host_margin_left,
            col_area.width - ctx.host_margin_left,
        ),
        HorzRelTo::Column => (col_area.x, col_area.width),
    };

    let x = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
        HorzAlign::Center => ref_x + (ref_w - width_px).max(0.0) / 2.0 + h_offset,
        HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - width_px).max(0.0) - h_offset,
    };
    (x, x + width_px.max(0.0))
}

/// A placed float lane in page/column-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatLane {
    pub x_start: f64,
    pub x_end: f64,
    pub bottom: f64,
}

impl FloatLane {
    fn overlaps_x(&self, x_start: f64, x_end: f64) -> bool {
        ranges_overlap(self.x_start, self.x_end, x_start, x_end)
    }
}

/// Tracks bottom reservations for horizontally independent float lanes.
#[derive(Debug, Default, Clone)]
pub(crate) struct FloatLaneSet {
    lanes: Vec<FloatLane>,
}

impl FloatLaneSet {
    pub(crate) fn new() -> Self {
        Self { lanes: Vec::new() }
    }

    pub(crate) fn clear(&mut self) {
        self.lanes.clear();
    }

    pub(crate) fn lanes(&self) -> &[FloatLane] {
        &self.lanes
    }

    pub(crate) fn pushed_top(&self, x_start: f64, x_end: f64, raw_top: f64) -> f64 {
        self.lanes
            .iter()
            .filter(|lane| lane.overlaps_x(x_start, x_end))
            .fold(raw_top, |top, lane| top.max(lane.bottom))
    }

    pub(crate) fn place(
        &mut self,
        x_start: f64,
        x_end: f64,
        raw_top: f64,
        height: f64,
    ) -> FloatLane {
        let top = self.pushed_top(x_start, x_end, raw_top);
        let lane = FloatLane {
            x_start,
            x_end,
            bottom: top + height.max(0.0),
        };
        self.lanes.push(lane);
        lane
    }

    pub(crate) fn max_bottom(&self) -> f64 {
        self.lanes
            .iter()
            .map(|lane| lane.bottom)
            .fold(0.0, f64::max)
    }
}

pub(crate) fn ranges_overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> bool {
    let a0 = a_start.min(a_end);
    let a1 = a_start.max(a_end);
    let b0 = b_start.min(b_end);
    let b1 = b_start.max(b_end);
    a0 < b1 && b0 < a1
}

/// [#6143] 문단 기준 양수 오프셋이 **쪽 경계에서 이미 소진**됐는지 판정한다.
///
/// 오프셋의 기준점은 앵커 문단이 놓인 자리다. 저장 사다리가 준 앵커 자리에 오프셋을
/// 얹었을 때 표 상단이 이 쪽 바닥에서 최소 조각(`MIN_FRAGMENT_KEEP_PX`)도 남기지 못하는
/// 자리에 떨어진다면, 그 자리는 **앞 쪽 바닥**이고 오프셋은 거기서 이미 쓰였다. 그런
/// 조각을 이 쪽 최상단에서 다시 오프셋만큼 밀면 쪽 상단에 빈 띠가 생기고, 그만큼 조각이
/// 짧아져 표가 한 쪽 더 갈라진다(156555538 9쪽: 앵커 vpos=32514(433.5px) + off=41592
/// (554.6px) = 988.1px 로 가용 990.3px 의 바닥. 한글 17쪽 ↔ rhwp 18쪽).
///
/// 반대로 앵커 자리 + 오프셋이 쪽 안에 여유 있게 들어가면 그 오프셋은 이 쪽에서 유효한
/// 통상적인 미세 이동이므로 그대로 둔다(1342000 교육부 맵 p25: 200.0 + 10.0 ≪ 585.9).
pub(crate) fn para_offset_consumed_by_page_break(
    para: &Paragraph,
    common: &CommonObjAttr,
    available_height: f64,
    dpi: f64,
) -> bool {
    /// 오프셋을 적용한 자리에 이만큼도 안 남으면 그 자리에서는 조각이 시작될 수 없다.
    const MIN_FRAGMENT_KEEP_PX: f64 = 25.0;

    if common.treat_as_char || !matches!(common.vert_rel_to, VertRelTo::Para) {
        return false;
    }
    let offset = signed_hwpunit(common.vertical_offset);
    if offset <= 0 {
        return false;
    }
    if !available_height.is_finite() || available_height <= 0.0 {
        return false;
    }
    let anchor_top = para
        .line_segs
        .first()
        .map(|seg| hwpunit_to_px(seg.vertical_pos, dpi))
        .unwrap_or(0.0);
    anchor_top + hwpunit_to_px(offset, dpi) + MIN_FRAGMENT_KEEP_PX >= available_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::LineSeg;
    use crate::model::shape::{HorzAlign, HorzRelTo, VertAlign};
    use crate::model::table::Cell;
    use crate::renderer::layout_frame::FrameExclusionPolicy;

    fn base_common() -> CommonObjAttr {
        CommonObjAttr {
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Column,
            horz_align: HorzAlign::Left,
            ..Default::default()
        }
    }

    fn supported_picture(flow: TextFlow) -> Picture {
        Picture {
            common: CommonObjAttr {
                width: 300,
                height: 200,
                text_wrap: TextWrap::Square,
                text_flow: flow,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn picture_exclusion_maps_only_recovered_side_wrap_flows() {
        let both_sides = resolve_picture_exclusion(
            &supported_picture(TextFlow::BothSides),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .expect("BothSides is represented by the physical row frame");
        assert_eq!(both_sides.policy, FrameExclusionPolicy::BothSides);

        let largest_only = resolve_picture_exclusion(
            &supported_picture(TextFlow::LargestOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .expect("LargestOnly has a recovered frame policy");
        assert_eq!(largest_only.policy, FrameExclusionPolicy::LargestSide);

        assert!(resolve_picture_exclusion(
            &supported_picture(TextFlow::LeftOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .is_none());
        assert!(resolve_picture_exclusion(
            &supported_picture(TextFlow::RightOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .is_none());
    }

    #[test]
    fn picture_exclusion_uses_flow_frame_for_oversized_current_square_float() {
        let mut picture = supported_picture(TextFlow::BothSides);
        picture.shape_attr.current_width = 900;
        picture.shape_attr.current_height = 800;

        let exclusion =
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 50, PaperOrigin::default())
                .expect("Square side-wrap float has a physical row frame");

        assert_eq!(exclusion.horizontal, 0..300);
        assert_eq!(exclusion.vertical, 50..250);
    }

    #[test]
    fn picture_exclusion_rejects_non_square_or_inline_hosts() {
        let mut picture = supported_picture(TextFlow::BothSides);
        picture.common.treat_as_char = true;
        assert!(
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 0, PaperOrigin::default())
                .is_none()
        );

        picture.common.treat_as_char = false;
        picture.common.text_wrap = TextWrap::TopAndBottom;
        assert!(
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 0, PaperOrigin::default())
                .is_none()
        );
    }

    #[test]
    fn signed_hwpunit_preserves_negative_offsets() {
        assert_eq!(signed_hwpunit((-43892i32) as u32), -43892);
        assert_eq!(signed_hwpunit(51100), 51100);
    }

    #[test]
    fn para_topbottom_float_predicate_requires_non_tac_para_topbottom() {
        let mut common = base_common();
        assert!(is_para_topbottom_float(&common));

        common.treat_as_char = true;
        assert!(!is_para_topbottom_float(&common));

        common.treat_as_char = false;
        common.text_wrap = TextWrap::Square;
        assert!(!is_para_topbottom_float(&common));

        common.text_wrap = TextWrap::TopAndBottom;
        common.vert_rel_to = VertRelTo::Page;
        assert!(!is_para_topbottom_float(&common));
    }

    #[test]
    fn lane_set_does_not_push_non_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        let first = lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(120.0, 200.0, 10.0, 20.0);

        assert_eq!(first.bottom, 50.0);
        assert_eq!(second.bottom, 30.0);
        assert_eq!(lanes.max_bottom(), 50.0);
    }

    #[test]
    fn lane_set_pushes_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(90.0, 160.0, 10.0, 20.0);

        assert_eq!(second.bottom, 70.0);
        assert_eq!(lanes.max_bottom(), 70.0);
    }

    #[test]
    fn horizontal_range_matches_column_right_offset_rule() {
        let mut common = base_common();
        common.horz_align = HorzAlign::Right;
        common.horizontal_offset = 10;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 50.0, ctx, 7200.0);

        assert_eq!(x0, 160.0);
        assert_eq!(x1, 210.0);
    }

    #[test]
    fn horizontal_range_uses_body_area_for_page_relative_objects() {
        let mut common = base_common();
        common.horz_rel_to = HorzRelTo::Page;
        common.horz_align = HorzAlign::Center;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        })
        .with_body_area(LayoutRect {
            x: 40.0,
            y: 0.0,
            width: 300.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 100.0, ctx, 7200.0);

        assert_eq!(x0, 140.0);
        assert_eq!(x1, 240.0);
    }

    fn stored_reset_fragment_candidate() -> (Paragraph, Table) {
        let mut cell = Cell::new_empty(0, 0, 41_954, 2_282, 1);
        cell.paragraphs = vec![
            Paragraph {
                line_segs: vec![
                    LineSeg {
                        vertical_pos: 0,
                        line_height: 2_000,
                        text_height: 1_000,
                        line_spacing: 1_000,
                        ..Default::default()
                    },
                    LineSeg {
                        vertical_pos: 1_000,
                        line_height: 2_000,
                        text_height: 1_000,
                        line_spacing: 1_000,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: 2_000,
                    text_height: 1_000,
                    line_spacing: 1_000,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let table = Table {
            row_count: 1,
            col_count: 1,
            page_break: TablePageBreak::RowBreak,
            padding: crate::model::Padding {
                top: 141,
                bottom: 141,
                ..Default::default()
            },
            common: CommonObjAttr {
                width: 41_954,
                height: 2_282,
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            cells: vec![cell],
            ..Default::default()
        };
        let host = Paragraph {
            controls: vec![Control::Table(Box::new(table.clone()))],
            ..Default::default()
        };
        (host, table)
    }

    #[test]
    fn stored_reset_fragment_geometry_separates_first_paint_height_from_successor_origin() {
        let (host, table) = stored_reset_fragment_candidate();

        assert_eq!(
            native_hwp5_stored_reset_fragment_paint_geometry(true, &host, &table, false, &[], &[2],),
            Some(NativeStoredResetFragmentPaintGeometry {
                outer_left_hu: 283,
                outer_top_hu: 283,
                first_fragment_height_hu: Some(2_282),
            })
        );
        assert_eq!(
            native_hwp5_stored_reset_fragment_paint_geometry(true, &host, &table, true, &[2], &[],),
            Some(NativeStoredResetFragmentPaintGeometry {
                outer_left_hu: 283,
                outer_top_hu: 283,
                first_fragment_height_hu: None,
            })
        );

        // A neighboring cut is not the stored reset boundary.
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &table,
            false,
            &[],
            &[1],
        )
        .is_none());
    }

    #[test]
    fn stored_reset_fragment_geometry_rejects_unproven_neighboring_shapes() {
        let (host, table) = stored_reset_fragment_candidate();

        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            false,
            &host,
            &table,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut visible_host = host.clone();
        visible_host.text = "표 제목".to_string();
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &visible_host,
            &table,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut wrong_declared_height = table.clone();
        wrong_declared_height.common.height += 1_000;
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &wrong_declared_height,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut asymmetric_margin = table.clone();
        asymmetric_margin.outer_margin_right += 1;
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &asymmetric_margin,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut same_paragraph_rewind = table.clone();
        let reset = same_paragraph_rewind.cells[0].paragraphs.remove(1);
        same_paragraph_rewind.cells[0].paragraphs[0]
            .line_segs
            .extend(reset.line_segs);
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &same_paragraph_rewind,
            false,
            &[],
            &[2],
        )
        .is_none());
    }

    fn physical_outer_box_candidate() -> (Paragraph, Table, Paragraph) {
        let table = Table {
            row_count: 6,
            col_count: 1,
            page_break: TablePageBreak::RowBreak,
            common: CommonObjAttr {
                width: 41_954,
                height: 23_790,
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            cells: (0..6)
                .map(|row| Cell::new_empty(0, row, 41_954, 3_965, 1))
                .collect(),
            ..Default::default()
        };
        let host = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 0,
                line_height: 1,
                ..Default::default()
            }],
            controls: vec![Control::Table(Box::new(table.clone()))],
            ..Default::default()
        };
        let next = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 24_356,
                line_height: 1_000,
                ..Default::default()
            }],
            ..Default::default()
        };
        (host, table, next)
    }

    #[test]
    fn physical_outer_box_paint_inset_requires_exact_native_stored_ladder() {
        let (host, table, next) = physical_outer_box_candidate();
        assert!(native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next),
        ));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            false,
            &host,
            &table,
            Some(&next),
        ));

        let mut short = next.clone();
        short.line_segs[0].vertical_pos = 23_790;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&short),
        ));

        let mut mismatched = next.clone();
        mismatched.line_segs[0].vertical_pos += 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&mismatched),
        ));

        let mut synthetic_host = host.clone();
        synthetic_host.line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &synthetic_host,
            &table,
            Some(&next),
        ));

        let mut synthetic_next = next.clone();
        synthetic_next.line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&synthetic_next),
        ));
    }

    #[test]
    fn physical_outer_box_paint_inset_rejects_neighboring_float_contracts() {
        let (host, table, next) = physical_outer_box_candidate();

        let mut positive_offset = table.clone();
        positive_offset.common.vertical_offset = 350;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &positive_offset,
            Some(&next),
        ));

        let mut horizontal_offset = table.clone();
        horizontal_offset.common.horizontal_offset = 350;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &horizontal_offset,
            Some(&next),
        ));

        let mut visible_host = host.clone();
        visible_host.text = "표 제목".to_string();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &visible_host,
            &table,
            Some(&next),
        ));

        let mut two_tables = host.clone();
        two_tables
            .controls
            .push(Control::Table(Box::new(table.clone())));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &two_tables,
            &table,
            Some(&next),
        ));

        let mut next_object_host = next.clone();
        next_object_host
            .controls
            .push(Control::Table(Box::new(table.clone())));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next_object_host),
        ));

        let mut next_visible = next.clone();
        next_visible.text = "다음 본문".to_string();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next_visible),
        ));

        let mut tac = table.clone();
        tac.common.treat_as_char = true;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &tac,
            Some(&next),
        ));

        let mut square = table.clone();
        square.common.text_wrap = TextWrap::Square;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &square,
            Some(&next),
        ));

        let mut page_relative = table.clone();
        page_relative.common.vert_rel_to = VertRelTo::Page;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &page_relative,
            Some(&next),
        ));

        let mut right_aligned = table.clone();
        right_aligned.common.horz_align = HorzAlign::Right;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &right_aligned,
            Some(&next),
        ));

        let mut missing_margin = table.clone();
        missing_margin.outer_margin_left = 0;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &missing_margin,
            Some(&next),
        ));

        let mut asymmetric_margin = table.clone();
        asymmetric_margin.outer_margin_right += 1;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &asymmetric_margin,
            Some(&next),
        ));

        let mut one_by_one = table.clone();
        one_by_one.row_count = 1;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &one_by_one,
            Some(&next),
        ));

        let mut two_columns = table.clone();
        two_columns.col_count = 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &two_columns,
            Some(&next),
        ));

        let mut missing_cells = table.clone();
        missing_cells.cells.clear();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &missing_cells,
            Some(&next),
        ));

        let mut duplicate_row = table.clone();
        duplicate_row.cells[1].row = 0;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &duplicate_row,
            Some(&next),
        ));

        let mut spanning_row = table.clone();
        spanning_row.cells[0].row_span = 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &spanning_row,
            Some(&next),
        ));
    }
}
