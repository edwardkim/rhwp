use std::borrow::Cow;

use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::page::{ColumnDef, PageDef};

// 본문 폭을 모두 쓰는 절대 단 크기만 저장 사본에서 한컴의 합 32768 비례값으로 바꾼다.
pub(super) fn for_export(section: &Section) -> Cow<'_, Section> {
    let mut body_width = physical_body_width(&section.section_def.page_def);
    let mut result = Cow::Borrowed(section);
    for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
        if let Some(definition) = paragraph.controls.iter().find_map(|control| match control {
            Control::SectionDef(definition) => Some(definition),
            _ => None,
        }) {
            body_width = physical_body_width(&definition.page_def);
        }
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let Control::ColumnDef(definition) = control else {
                continue;
            };
            if let Some(normalized) = body_width.and_then(|width| normalized(definition, width)) {
                result.to_mut().paragraphs[paragraph_index].controls[control_index] =
                    Control::ColumnDef(normalized);
            }
        }
    }
    result
}

fn physical_body_width(page: &PageDef) -> Option<i32> {
    let width = u64::from(if page.landscape {
        page.height
    } else {
        page.width
    });
    let margins =
        u64::from(page.margin_left) + u64::from(page.margin_gutter) + u64::from(page.margin_right);
    // 렌더링용 손상 용지 폴백을 실제 저장 폭으로 굳히지 않는다.
    if page.width == 0 || page.height == 0 || width <= margins {
        return None;
    }
    i32::try_from(width - margins).ok()
}

fn normalized(definition: &ColumnDef, body_width: i32) -> Option<ColumnDef> {
    let count = usize::from(definition.column_count);
    if definition.same_width
        || definition.proportional_widths
        || count < 2
        || definition.widths.len() != count
        || definition.gaps.len() > count
        || definition.widths.iter().any(|&width| width <= 0)
        || definition.gaps.iter().any(|&gap| gap < 0)
    {
        return None;
    }
    let total: i64 = definition
        .widths
        .iter()
        .chain(&definition.gaps)
        .map(|&value| i64::from(value))
        .sum();
    if total != i64::from(body_width) {
        return None;
    }
    let mut result = definition.clone();
    result.widths.clear();
    result.gaps.clear();
    let (mut accumulated, mut previous) = (0, 0);
    for index in 0..count {
        for (is_width, value) in [
            (true, definition.widths[index]),
            (false, definition.gaps.get(index).copied().unwrap_or(0)),
        ] {
            accumulated += i64::from(value);
            let boundary = (accumulated * 32768 + total / 2) / total;
            let scaled = i16::try_from(boundary - previous).ok()?;
            if is_width && scaled == 0 {
                return None;
            }
            if is_width {
                result.widths.push(scaled);
            } else {
                result.gaps.push(scaled);
            }
            previous = boundary;
        }
    }
    result.proportional_widths = true;
    Some(result)
}
