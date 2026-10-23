//! 글 없는 개체 host의 줄과 별도로 출력한 개체의 실제 소유를 구분한다.

use crate::model::{control::Control, paragraph::Paragraph, shape::TextWrap};
use crate::renderer::pagination::{PageContent, PageItem};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(super) struct TextlessShapeHost {
    pub(super) ladder_advance: Option<bool>,
    /// 별도 출력된 TAC의 음수 줄간격은 유효한 저장 사다리가 증언해야 한다.
    /// 그 밖의 TAC는 일반 문단이 개체를 담는 줄 상자의 전진도 소유한다.
    pub(super) separate_inline_object: bool,
}

pub(super) fn control_has_visible_inline_content(control: &Control) -> bool {
    match control {
        Control::Picture(picture) => picture.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        Control::Table(table) => table.common.treat_as_char,
        Control::Equation(equation) => equation.common.treat_as_char,
        Control::Form(form) => form.common.treat_as_char,
        _ => false,
    }
}

pub(super) fn classify(
    page: &PageContent,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    para_index: usize,
    allow_stored_inline_advance: bool,
) -> Option<TextlessShapeHost> {
    let para = paragraphs.get(para_index)?;
    if super::para_has_visible_text(para) {
        return None;
    }
    let separate = |control_index| {
        page.column_contents.iter().any(|column| {
            column.items.iter().any(|item| {
                matches!(item, PageItem::Shape { para_index: owner, control_index: control }
                    if *owner == para_index && *control == control_index)
            })
        })
    };
    if para.controls.iter().enumerate().any(|(index, control)| {
        control_has_visible_inline_content(control)
            && (!matches!(control, Control::Picture(_) | Control::Shape(_)) || !separate(index))
    }) {
        return None;
    }
    let mut found = false;
    let mut ladder_candidate = false;
    let mut separate_inline_object = false;
    for (index, control) in para.controls.iter().enumerate() {
        if !separate(index) {
            continue;
        }
        let common = match control {
            Control::Picture(picture) => &picture.common,
            Control::Shape(shape) => shape.common(),
            // 장식 표만 Shape 경로를 소유한다. 줄 안 표는 별도 Table 소유자를 유지한다.
            Control::Table(table) if !table.common.treat_as_char => &table.common,
            _ => continue,
        };
        found = true;
        separate_inline_object |= common.treat_as_char;
        ladder_candidate |= common.treat_as_char
            || matches!(
                common.text_wrap,
                TextWrap::InFrontOfText
                    | TextWrap::BehindText
                    | TextWrap::Square
                    | TextWrap::Tight
                    | TextWrap::Through
                    | TextWrap::TopAndBottom
            );
    }
    let ladder_advance = ladder_candidate
        .then(|| {
            super::textless_host_ladder_line_advance(
                paragraphs,
                styles,
                dpi,
                para_index,
                separate_inline_object,
            )
        })
        .flatten();
    // Shape 항목의 출력 소유만으로 TAC의 줄 상자까지 부동 개체로 바꾸지 않는다.
    // 원본의 음수 줄간격과 다음 원점이 닫히는 경우만 그 짧은 전진을 공유한다.
    // 음수가 아닌 간격·저장 증거가 없는 TAC는 일반 줄 배치를 따른다.
    if separate_inline_object
        && (!allow_stored_inline_advance
            || para.line_segs.iter().any(|line| line.line_spacing >= 0)
            || ladder_advance.is_none())
    {
        return None;
    }
    found.then_some(TextlessShapeHost {
        ladder_advance,
        separate_inline_object,
    })
}
