//! 수식 줄 캐시가 바뀐 내용 맞춤 글상자의 파생 높이와 뒤 문단 원점.

use crate::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use crate::renderer::style_resolver::ResolvedStyleSet;

fn content_end(paragraphs: &[Paragraph]) -> Option<i32> {
    paragraphs
        .iter()
        .flat_map(|p| &p.line_segs)
        .map(|line| line.vertical_pos.saturating_add(line.line_height))
        .max()
}

fn normalize_host(paragraph: &mut Paragraph, styles: &ResolvedStyleSet) -> Option<i32> {
    if !paragraph.text.trim().is_empty()
        || paragraph.controls.len() != 1
        || paragraph.line_segs.len() != 1
    {
        return None;
    }
    let Control::Shape(shape) = &mut paragraph.controls[0] else {
        return None;
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
        return None;
    };
    let common = &rectangle.common;
    let geometry = &rectangle.drawing.shape_attr;
    let extent_scale = |scale: f64, current: u32, original: u32| {
        // 현재 크기는 정수 HWPUNIT, 저장 변환은 실수다. 내용 끝과 같은
        // 1 HWPUNIT 저장 반올림을 extent 좌표계에서 확인한다.
        let scaled_extent = scale * f64::from(original);
        original > 0
            && scale > 0.0
            && (scaled_extent - f64::from(current)).abs()
                <= 1.0 + f64::EPSILON * scaled_extent.abs().max(1.0) * 4.0
    };
    if !common.treat_as_char
        || common.size_protect
        || geometry.rotation_angle != 0
        || geometry.horz_flip
        || geometry.vert_flip
        || geometry.render_b != 0.0
        || geometry.render_c != 0.0
        || !extent_scale(
            geometry.render_sx,
            geometry.current_width,
            geometry.original_width,
        )
        || !extent_scale(
            geometry.render_sy,
            geometry.current_height,
            geometry.original_height,
        )
    {
        return None;
    }
    let textbox = rectangle.drawing.text_box.as_ref()?;
    if textbox.list_attr & 7 != 0
        || textbox.paragraphs.iter().any(|p| {
            p.controls.iter().any(|c| {
                matches!(
                    c,
                    Control::Table(_) | Control::Picture(_) | Control::Shape(_)
                )
            })
        })
    {
        return None;
    }
    let old_end = content_end(&textbox.paragraphs)?;
    let margins = i32::from(textbox.margin_top) + i32::from(textbox.margin_bottom);
    let old_height = i32::try_from(geometry.current_height).ok()?;
    let old_total = old_end.saturating_add(margins);
    let host = &paragraph.line_segs[0];
    // 고정 크기의 여백을 가진 상자는 줄 높이 변화만으로 축소하지 않는다.
    // 현재 크기와 host 줄이 저장 내용 끝+여백에서 만들어진 경우만 재발행한다.
    // 1 HWPUNIT 차이는 정수 저장 반올림이며 px 허용 범위가 아니다.
    if (old_height - old_total).abs() > 1
        || (host.line_height - old_height).abs() > 1
        || common.height >= old_total.max(0) as u32
    {
        return None;
    }
    let projected = crate::renderer::composer::stored_row_projection::project_equation_rows(
        &textbox.paragraphs,
        styles,
    )?;
    let delta = content_end(&projected)?.saturating_sub(old_end);
    let new_height = old_height.saturating_add(delta);
    if delta == 0 || new_height <= 0 || new_height < common.height as i32 {
        return None;
    }
    rectangle.drawing.text_box.as_mut()?.paragraphs = projected;
    rectangle.drawing.shape_attr.current_height = new_height as u32;
    rectangle.drawing.shape_attr.rotation_center.y = new_height / 2;
    rectangle.drawing.shape_attr.render_sy +=
        f64::from(delta) / f64::from(rectangle.drawing.shape_attr.original_height);
    let host = &mut paragraph.line_segs[0];
    let old_line_height = host.line_height;
    host.line_height = host.line_height.saturating_add(delta);
    host.text_height = host.text_height.saturating_add(delta);
    host.baseline_distance = (i64::from(host.baseline_distance) * i64::from(host.line_height)
        / i64::from(old_line_height)) as i32;
    Some(delta)
}

pub(crate) fn normalize(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
) -> Option<Vec<Paragraph>> {
    if !original.iter().any(|paragraph| {
        paragraph.controls.iter().any(|control| {
            matches!(control, Control::Shape(shape)
                if matches!(shape.as_ref(), ShapeObject::Rectangle(rectangle)
                    if rectangle.common.treat_as_char && rectangle.drawing.text_box.is_some()))
        })
    }) {
        return None;
    }
    let mut result = original.to_vec();
    let mut shift = 0_i32;
    let mut previous_vpos = None;
    let mut changed = false;
    for paragraph in &mut result {
        for line in &mut paragraph.line_segs {
            let source_vpos = line.vertical_pos;
            if previous_vpos.is_some_and(|previous| source_vpos < previous) {
                shift = 0;
            }
            previous_vpos = Some(source_vpos);
            line.vertical_pos = source_vpos.saturating_add(shift);
        }
        if let Some(delta) = normalize_host(paragraph, styles) {
            shift = shift.saturating_add(delta);
            changed = true;
        }
    }
    changed.then_some(result)
}
