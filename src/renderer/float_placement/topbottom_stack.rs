//! 글이 있는 칸 문단에 달린 자리차지 그림 여럿의 겹침 없는 자리를 측정·정렬·배치가 함께 쓴다.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    shape::{TextWrap, VertAlign, VertRelTo},
};

pub(crate) struct CellTextHostTopBottomStack {
    /// (control 번호, 문단 위끝 기준 그림 위끝 HWPUNIT).
    pub tops_hu: Vec<(usize, i32)>,
    /// 문단 위끝부터 글줄·그림 가운데 가장 아래 끝까지(HWPUNIT).
    pub extent_hu: i32,
}

impl CellTextHostTopBottomStack {
    pub(crate) fn top_hu(&self, control: usize) -> Option<i32> {
        self.tops_hu
            .iter()
            .find(|(index, _)| *index == control)
            .map(|(_, top)| *top)
    }
}

/// 칸의 유일한 문단이 글 한 줄과 겹침을 허용하지 않는 문단 기준 자리차지 그림 둘 이상을 가지면,
/// 그림은 문단 위끝에서 자기 양수 세로 오프셋 자리에 놓이고 먼저 놓인 그림과 겹치면 그 아래로
/// 내려간다. 음수 오프셋은 문단 위로 올라가지 않는다. 칸 내용의 끝은 가장 아래 그림 바닥이다.
///
/// 가로로 떨어져 나란히 놓인 그림은 서로 밀지 않는다(#6761). 캡션·바깥 여백·글자처럼 취급·
/// 다른 배치·재조판 줄은 각자의 기존 계약에 남긴다.
pub(crate) fn cell_text_host_topbottom_stack(
    paragraphs: &[Paragraph],
) -> Option<CellTextHostTopBottomStack> {
    let [para] = paragraphs else {
        return None;
    };
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if para.text.trim().is_empty()
        || para.text.contains(['\r', '\n'])
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.vertical_pos != 0
        || line.line_height <= 0
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || para.controls.len() < 2
    {
        return None;
    }
    // (x0, x1, y0, y1)
    let mut placed: Vec<(i64, i64, i64, i64)> = Vec::with_capacity(para.controls.len());
    let mut tops = Vec::with_capacity(para.controls.len());
    for (index, control) in para.controls.iter().enumerate() {
        let Control::Picture(picture) = control else {
            return None;
        };
        let common = &picture.common;
        if common.treat_as_char
            || !common.flow_with_text
            || common.allow_overlap
            || picture.caption.is_some()
            || common.text_wrap != TextWrap::TopAndBottom
            || common.vert_rel_to != VertRelTo::Para
            || common.vert_align != VertAlign::Top
            || common.margin.top != 0
            || common.margin.bottom != 0
            || common.margin.left != 0
            || common.margin.right != 0
        {
            return None;
        }
        let width = i64::from(super::signed_hwpunit(common.width));
        let height = i64::from(super::signed_hwpunit(common.height));
        if width <= 0 || height <= 0 {
            return None;
        }
        let x0 = i64::from(super::signed_hwpunit(common.horizontal_offset));
        let x1 = x0 + width;
        let mut y0 = i64::from(super::signed_hwpunit(common.vertical_offset)).max(0);
        while let Some(lower) = placed
            .iter()
            .filter(|&&(px0, px1, py0, py1)| x0 < px1 && px0 < x1 && y0 < py1 && py0 < y0 + height)
            .map(|&(_, _, _, py1)| py1)
            .max()
        {
            y0 = lower;
        }
        placed.push((x0, x1, y0, y0 + height));
        tops.push((index, i32::try_from(y0).ok()?));
    }
    let extent = placed
        .iter()
        .map(|&(_, _, _, bottom)| bottom)
        .max()?
        .max(i64::from(line.line_height));
    Some(CellTextHostTopBottomStack {
        tops_hu: tops,
        extent_hu: i32::try_from(extent).ok()?,
    })
}
