//! [#8679] 자기 자리차지 표 띠에 걸친 저장 호스트 줄.
//!
//! 자리차지(글 위아래) 표의 세로 띠에는 글줄이 들어가지 않는다. 문단 기준 표의 양수
//! 세로 오프셋은 앵커 줄(표 제어 문자가 실린 저장 줄)에서 잰다. 앵커 줄 뒤의 저장 줄이
//! 이 띠와 겹치면 저장 사다리가 자리차지 계약과 모순된다. 한컴은 그 줄을 띠의 바깥
//! 끝으로 내리고, 같은 저장 프레임의 뒤 줄·뒤 문단을 같은 양만큼 민다.
//!
//! 근거: 결재문서 36493460 1쪽 문단 1. 원본 저장 줄 1(vpos 2200, 높이 300)이 제목 표
//! 띠 2441..4723 위끝에 걸친다. 한컴 2022로 다시 저장하면 그 줄은 4723(띠 끝)이 되고
//! 뒤 문단 vpos 가 모두 +480 이 된다. 한컴 PDF 의 뒤 표 괘선도 같은 양만큼 아래에 있다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    shape::{TextWrap, VertAlign, VertRelTo},
};

/// 띠에 걸친 첫 저장 줄을 띠 끝으로 옮긴 결과.
pub(crate) struct BandCrossingShift {
    /// 띠에 걸친 첫 저장 줄. 이 줄과 뒤 줄이 함께 내려간다.
    pub first_line: usize,
    /// 그 줄을 띠 끝으로 옮기는 양(HU).
    pub line_shift: i32,
    /// 문단 저장 프레임의 끝이 늘어난 양(HU). 같은 저장 프레임의 뒤 문단이 이만큼 내려간다.
    /// 프레임 끝은 마지막 줄 끝(줄 간격 포함)과 띠 끝 가운데 아래쪽이다.
    pub frame_shift: i32,
}

/// 띠에 걸친 첫 저장 줄과 그 줄·뒤 사다리를 옮기는 양.
///
/// 걸친 줄은 표 위끝보다 위에서 시작해 띠 안으로 들어간 줄이다. 띠 안에서 시작한 저장
/// 줄은 한컴이 표 뒤에 둔 위치다(연구용역보고서 1.658: 띠 끝 500HU 앞에서 시작한 저장 줄을
/// 한컴 PDF 도 그 자리에 그린다). 그 위치는 바꾸지 않는다.
///
/// 띠는 저장 사다리가 직접 증언해야 한다. 같은 저장 프레임의 다음 문단 첫 줄이 띠 끝에서
/// 시작하고, 띠 안에서 시작하는 호스트 줄이 없어야 한다. 쪽에 들어가지 않아 표가 다음 쪽으로
/// 넘어간 문단은 뒤 줄이 띠 자리에 그대로 저장된다(로드맵 연구 3.5: 줄 4 가 띠 안 49500).
///
/// 표가 앵커 줄과 겹쳐 시작하면 앵커 줄 자체가 표 뒤로 가는 다른 계약이므로 제외한다.
/// 합성·재구성 줄이나 쪽 리셋이 섞인 사다리는 원본 저장 위치의 증거가 아니다.
/// 걸친 줄을 만드는 표가 둘 이상이면 어느 띠가 줄을 소유하는지 정하지 않는다.
pub(crate) fn band_crossing_line_shift(
    para: &Paragraph,
    next_frame_vpos: Option<i32>,
) -> Option<BandCrossingShift> {
    let lines = &para.line_segs;
    if lines.len() < 2
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || lines.iter().any(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0
        })
        || lines
            .windows(2)
            .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
    {
        return None;
    }
    let mut found = None;
    for (control_index, control) in para.controls.iter().enumerate() {
        let Control::Table(table) = control else {
            continue;
        };
        let common = &table.common;
        let v_off = crate::renderer::float_placement::signed_hwpunit(common.vertical_offset);
        if common.treat_as_char
            || common.text_wrap != TextWrap::TopAndBottom
            || common.vert_rel_to != VertRelTo::Para
            || common.vert_align != VertAlign::Top
            || v_off <= 0
            || common.height == 0
            || common.height > i32::MAX as u32
        {
            continue;
        }
        // 표 배치와 같은 앵커 줄 판정(#6860)을 쓴다. 세로 오프셋의 기준이 그 줄이다.
        let stored: Vec<&LineSeg> = lines.iter().collect();
        let Some(anchor_index) =
            crate::renderer::layout::stored_float_anchor_line_top(para, control_index, &stored)
                .and_then(|top| lines.iter().rposition(|line| line.vertical_pos == top))
        else {
            continue;
        };
        let anchor = &lines[anchor_index];
        let band_top = i64::from(anchor.vertical_pos) + i64::from(v_off);
        if i64::from(anchor.vertical_pos) + i64::from(anchor.line_height) > band_top {
            continue;
        }
        let band_bottom = band_top
            + i64::from(table.outer_margin_top)
            + i64::from(common.height)
            + i64::from(table.outer_margin_bottom);
        if next_frame_vpos.map(i64::from) != Some(band_bottom)
            || lines
                .iter()
                .any(|line| (band_top..band_bottom).contains(&i64::from(line.vertical_pos)))
        {
            continue;
        }
        let Some(index) = (anchor_index + 1..lines.len()).find(|&index| {
            let line = &lines[index];
            i64::from(line.vertical_pos) < band_top
                && i64::from(line.vertical_pos) + i64::from(line.line_height) > band_top
        }) else {
            continue;
        };
        let line_shift = band_bottom - i64::from(lines[index].vertical_pos);
        let last = lines.last()?;
        let last_end = i64::from(last.vertical_pos)
            + i64::from(last.line_height)
            + i64::from(last.line_spacing.max(0));
        let frame_shift = (last_end + line_shift).max(band_bottom) - last_end.max(band_bottom);
        if found.is_some() {
            return None;
        }
        found = Some(BandCrossingShift {
            first_line: index,
            line_shift: i32::try_from(line_shift).ok()?,
            frame_shift: i32::try_from(frame_shift).ok()?,
        });
    }
    found.filter(|shift| shift.line_shift > 0)
}
