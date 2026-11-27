//! 저장 줄 없는 칸 문단에 함께 걸린 음수 세로 오프셋 그림 무리.
//!
//! 저장 줄이 없는 글 없는 문단에 문단 기준 자리차지 그림이 여럿 걸리고 그 세로 오프셋이
//! 음수이면, 그 값은 그림에 밀려 내려간 줄 위치를 기준으로 저장된 것이라 지금 배치의 원점
//! 위치를 말해 주지 않는다. 한컴 2024는 음수 오프셋을 문단 원점(0)으로 보고, '겹침 허용
//! 안 함' 그림을 문단 순서대로 놓으면서 앞 그림과 겹치면 그 아래로 내린다. 자리차지 그림에
//! 걸린 줄은 그 그림 아래로 밀린다. 그래서 사진 칸의 위아래 줄이 겹치지 않고 쌓이며,
//! 마지막 빈 줄은 그림 묶음 아래에 놓이고, 칸 세로 정렬은 그 묶음 전체에 적용된다.
//!
//! 종전에는 무리 전체가 같은 문단 원점에 겹쳐 그려져 위 줄 사진이 아래 줄 사진에 덮였고,
//! 행 높이는 그림 높이를 모두 더해 칸이 쪽 밖으로 커졌다.

use super::{
    hwpunit_to_px, signed_hwpunit, Control, LayoutRect, Paragraph, ReflowPictureParagraphFrame,
};
use crate::model::shape::{HorzRelTo, TextWrap, VertAlign, VertRelTo};
use crate::renderer::object_alignment::aligned_horizontal_origin;

struct StackedFloat {
    index: usize,
    left: f64,
    right: f64,
    /// 바깥 여백을 포함한 점유 위끝(문단 원점 기준).
    top: f64,
    outer_height: f64,
    margin_top: f64,
    allow_overlap: bool,
    pushes_line: bool,
    rect: LayoutRect,
}

impl StackedFloat {
    fn bottom(&self) -> f64 {
        self.top + self.outer_height
    }
}

/// 조건에 맞지 않으면 `None` 을 돌려주고 기존 계약을 그대로 쓴다.
pub(super) fn paragraph_frame(
    para: &Paragraph,
    inner_width: f64,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) -> Option<ReflowPictureParagraphFrame> {
    use crate::renderer::layout_frame::ParagraphBox;
    if !crate::renderer::para_has_no_stored_line_segs(para)
        || !para.text.trim().is_empty()
        || para.text.contains(['\r', '\n'])
        || para.controls.len() < 2
    {
        return None;
    }
    let style = styles.para_styles.get(para.para_shape_id as usize);
    let px = |hu: i32| hwpunit_to_px(hu, dpi);
    let mut floats = Vec::with_capacity(para.controls.len());
    let mut has_negative_topbottom = false;
    for (index, control) in para.controls.iter().enumerate() {
        let Control::Picture(picture) = control else {
            return None;
        };
        let c = &picture.common;
        let width = signed_hwpunit(c.width);
        let height = signed_hwpunit(c.height);
        if c.treat_as_char
            || picture.caption.is_some()
            || !c.flow_with_text
            || c.vert_rel_to != VertRelTo::Para
            || c.vert_align != VertAlign::Top
            || !matches!(c.horz_rel_to, HorzRelTo::Column | HorzRelTo::Para)
            || !matches!(c.text_wrap, TextWrap::TopAndBottom | TextWrap::Square)
            || width <= 0
            || height <= 0
        {
            return None;
        }
        let offset = signed_hwpunit(c.vertical_offset);
        has_negative_topbottom |= c.text_wrap == TextWrap::TopAndBottom && offset < 0;
        let reference_left = if c.horz_rel_to == HorzRelTo::Para {
            style.map_or(0.0, |s| s.margin_left)
        } else {
            0.0
        };
        let (width, height) = (px(width), px(height));
        let x = aligned_horizontal_origin(
            reference_left,
            inner_width - reference_left,
            width,
            c.horz_align,
            px(signed_hwpunit(c.horizontal_offset)),
        );
        let margin_top = px(i32::from(c.margin.top));
        floats.push(StackedFloat {
            index,
            left: x - px(i32::from(c.margin.left)),
            right: x + width + px(i32::from(c.margin.right)),
            // 음수 오프셋은 문단 원점 위로 올라가지 않는다.
            top: px(offset.max(0)),
            outer_height: margin_top + height + px(i32::from(c.margin.bottom)),
            margin_top,
            allow_overlap: c.allow_overlap,
            pushes_line: c.text_wrap == TextWrap::TopAndBottom,
            rect: LayoutRect {
                x,
                y: 0.0,
                width,
                height,
            },
        });
    }
    if !has_negative_topbottom {
        return None;
    }

    // 문단의 한 줄(빈 줄 또는 공백 줄)을 기존 줄 나눔으로 만든다.
    let paragraph_box = ParagraphBox::content_for_style(inner_width, style, dpi);
    if !paragraph_box.is_usable() {
        return None;
    }
    let mut reflowed = para.clone();
    reflowed.line_segs.clear();
    crate::renderer::composer::reflow_line_segs(&mut reflowed, paragraph_box, styles, dpi);
    let [line] = reflowed.line_segs.as_slice() else {
        return None;
    };
    let line_height = px(line.line_height).max(0.0);
    let line_spacing = px(line.line_spacing);

    let mut placed: Vec<StackedFloat> = Vec::with_capacity(floats.len());
    for mut float in floats {
        // 두 그림 모두 겹침을 막을 때 앞 그림의 바닥으로 이동한다. 글줄 점유는 textWrap을 따른다.
        while let Some(bottom) = placed
            .iter()
            .filter(|other| {
                !float.allow_overlap
                    && !other.allow_overlap
                    && other.left < float.right
                    && float.left < other.right
                    && other.top < float.bottom()
                    && float.top < other.bottom()
            })
            .map(StackedFloat::bottom)
            .reduce(f64::max)
        {
            float.top = bottom;
        }
        float.rect.y = float.top + float.margin_top;
        placed.push(float);
    }
    // 자리차지 그림에 걸린 줄은 그 아래로 밀린다.
    let mut line_top = 0.0f64;
    while let Some(bottom) = placed
        .iter()
        .filter(|other| {
            other.pushes_line
                && other.top < line_top + line_height.max(f64::EPSILON)
                && line_top < other.bottom()
        })
        .map(StackedFloat::bottom)
        .reduce(f64::max)
    {
        line_top = bottom;
    }
    let content_height = placed
        .iter()
        .map(StackedFloat::bottom)
        .fold(line_top + line_height, f64::max);
    let mut pictures: Vec<(usize, LayoutRect)> = placed
        .into_iter()
        .map(|float| (float.index, float.rect))
        .collect();
    pictures.sort_by_key(|(index, _)| *index);
    Some(ReflowPictureParagraphFrame {
        pictures,
        content_height,
        advance: line_top + line_height + line_spacing,
    })
}
