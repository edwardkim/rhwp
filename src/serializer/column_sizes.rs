use std::borrow::Cow;

use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::page::{ColumnDef, PageDef};

// 본문 폭을 모두 쓰는 절대 단 크기만 저장 사본에서 한컴의 합 32768 비례값으로 바꾼다.
pub(super) fn for_export_with_storage<'a, S: crate::storage::Storage>(
    section: &'a Section,
    storage: &S,
) -> Result<Cow<'a, Section>, S::Error> {
    let mut body_width = physical_body_width(&section.section_def.page_def);
    let mut result: Cow<'a, Section> = Cow::Borrowed(section);
    for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
        storage.bytes(0)?;
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
            let normalized = if let Some(width) = body_width {
                normalized_with_storage(definition, width, storage)?
            } else {
                None
            };
            if let Some(normalized) = normalized {
                if matches!(result, Cow::Borrowed(_)) {
                    result = Cow::Owned(Section {
                        section_def: storage.clone_section_def(&section.section_def)?,
                        paragraphs: storage.try_collect(
                            section
                                .paragraphs
                                .iter()
                                .map(|para| storage.clone_paragraph(para)),
                        )?,
                        trailing_lists: section
                            .trailing_lists
                            .as_ref()
                            .map(|tail| storage.clone_record(tail))
                            .transpose()?,
                        raw_stream: section
                            .raw_stream
                            .as_ref()
                            .map(|raw| storage.copy_slice(raw))
                            .transpose()?,
                        raw_provenance: section.raw_provenance.clone(),
                    });
                }
                result.to_mut().paragraphs[paragraph_index].controls[control_index] =
                    Control::ColumnDef(normalized);
            }
        }
    }
    Ok(result)
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

fn normalized_with_storage<S: crate::storage::Storage>(
    definition: &ColumnDef,
    body_width: i32,
    storage: &S,
) -> Result<Option<ColumnDef>, S::Error> {
    let count = usize::from(definition.column_count);
    if definition.same_width
        || definition.proportional_widths
        || count < 2
        || definition.widths.len() != count
        || definition.gaps.len() > count
        || definition.widths.iter().any(|&width| width <= 0)
        || definition.gaps.iter().any(|&gap| gap < 0)
    {
        return Ok(None);
    }
    let total: i64 = definition
        .widths
        .iter()
        .chain(&definition.gaps)
        .map(|&value| i64::from(value))
        .sum();
    if total != i64::from(body_width) {
        return Ok(None);
    }
    let mut result = storage.clone_record(definition)?;
    result.widths.clear();
    result.gaps.clear();
    storage.reserve(&mut result.gaps, count)?;
    let (mut accumulated, mut previous) = (0, 0);
    for index in 0..count {
        for (is_width, value) in [
            (true, definition.widths[index]),
            (false, definition.gaps.get(index).copied().unwrap_or(0)),
        ] {
            accumulated += i64::from(value);
            let boundary = (accumulated * 32768 + total / 2) / total;
            let Ok(scaled) = i16::try_from(boundary - previous) else {
                return Ok(None);
            };
            if is_width && scaled == 0 {
                return Ok(None);
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
    Ok(Some(result))
}
