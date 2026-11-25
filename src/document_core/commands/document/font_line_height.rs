//! 글꼴에 어울리는 줄 높이를 적용하지 않고 저장된 줄의 세로 치수를 고친다 (#9379).
//!
//! HWP 5.0 문단 모양 속성 1 bit 22(글꼴에 어울리는 줄 높이)가 켜진 문단은 줄 높이를 글자
//! 크기가 아니라 글꼴의 세로 상자(위+아래)로 잡고, 퍼센트 줄 간격도 그 높이에 건다.
//! 옛 한/글(5.0.2.x)이 함초롬 글꼴 문단을 저장할 때는 이 상자를 쓰지 않고 글자 크기를 줄
//! 높이로 적었고, 한/글 2020 은 같은 문서를 열어 상자대로 다시 짠다 — KAF 회의자료의 글 상자
//! 문단 13pt: 저장 줄 높이 1300·줄 간격 1196, 한/글 2020 재저장 1691·1552, Print 줄 피치
//! 32.38pt(= 3238 HU).
//!
//! 한/글 2020 은 글 상자(그리기 개체의 글) 문단만 열 때 다시 짜고, 본문·표 칸 문단은 저장
//! 줄을 그대로 인쇄한다 — 같은 KAF 문서를 한/글 2020 으로 다시 저장하면 2,572 문단 가운데 같은
//! 문단 모양·글꼴(함초롬바탕, bit 22)의 본문·칸 문단 수십 개는 저장 줄 높이(글자 크기)를
//! 그대로 두고 글 상자 문단 3개만 상자 높이로 바뀐다. 그래서 글 상자 안 문단만 고친다.
//!
//! 고치는 조건은 모두 원본 저장값과 확인된 글꼴 세로 상자로 정한다.
//! - 글 상자 안 문단이다(글 상자 안의 표 칸 등 하위 목록 포함).
//! - 문단 모양에 bit 22 가 있고 줄 간격이 퍼센트다.
//! - 줄의 모든 글자(언어 중립 글자 제외)의 글꼴이 확인된 세로 상자를 가진다
//!   (`font_metrics_data::font_line_box_em`). 모르는 글꼴이 하나라도 있으면 문단을 그대로 둔다.
//! - 저장 줄 높이가 그 줄 최대 글자 크기와 같다 — 상자를 적용하지 않고 저장한 줄이다.
//!   상자를 적용해 저장한 줄(한/글 2020 재저장본 등)은 높이가 이미 커서 건드리지 않는다.
//!
//! 줄 나눔 위치(`text_start`)와 가로 치수는 바꾸지 않는다. 문단 안 뒤 줄과 같은 글 상자
//! 목록에서 이어지는 뒤 문단의 저장 세로 위치는 늘어난 만큼 함께 내린다.
use crate::model::control::Control;
use crate::model::document::{DocInfo, Document};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::ShapeObject;
use crate::model::style::LineSpacingType;
use crate::renderer::composer::is_lang_neutral;
use crate::renderer::font_metrics_data::font_line_box_em;
use crate::renderer::style_resolver::detect_lang_category;

const FONT_LINE_HEIGHT: u32 = 1 << 22;

pub(super) fn refresh_stale_font_line_heights(document: &mut Document) {
    let profile = document.layout_profile();
    if !profile.hwp5_stored_pagination_layout() || profile.session_edited() {
        return;
    }
    let doc_info = &document.doc_info;
    for section in &mut document.sections {
        refresh_list(&mut section.paragraphs, doc_info, false);
    }
}

/// `in_text_box`: 이 목록이 글 상자 안에 있다(한/글이 열 때 다시 짜는 범위).
fn refresh_list(paragraphs: &mut [Paragraph], doc_info: &DocInfo, in_text_box: bool) -> bool {
    let mut refreshed = false;
    let mut shift = 0i32;
    let mut previous_end: Option<i32> = None;
    for para in paragraphs {
        for control in &mut para.controls {
            refresh_control(control, doc_info, in_text_box);
        }
        if !in_text_box {
            continue;
        }
        let (Some(first), Some(last)) = (para.line_segs.first(), para.line_segs.last()) else {
            continue;
        };
        let (source_first, source_last) = (first.vertical_pos, last.vertical_pos);
        // 저장 세로 위치가 되돌아가면 새 쪽(또는 새 단)이다. 그 뒤는 앞 문단의 늘어남과 무관하다.
        if previous_end.is_some_and(|end| source_first < end) {
            shift = 0;
        }
        previous_end = Some(source_last);
        if shift != 0 {
            for line in &mut para.line_segs {
                line.vertical_pos += shift;
            }
        }
        if let Some(grown) = refresh_paragraph(para, doc_info) {
            refreshed = true;
            shift += grown;
        }
    }
    refreshed
}

fn refresh_control(control: &mut Control, doc_info: &DocInfo, in_text_box: bool) {
    match control {
        Control::Table(table) => {
            for cell in &mut table.cells {
                refresh_list(&mut cell.paragraphs, doc_info, in_text_box);
            }
        }
        Control::Shape(shape) => refresh_shape(shape, doc_info),
        Control::Header(header) => {
            refresh_list(&mut header.paragraphs, doc_info, false);
        }
        Control::Footer(footer) => {
            refresh_list(&mut footer.paragraphs, doc_info, false);
        }
        Control::Footnote(note) => {
            refresh_list(&mut note.paragraphs, doc_info, false);
        }
        Control::Endnote(note) => {
            refresh_list(&mut note.paragraphs, doc_info, false);
        }
        _ => {}
    }
}

fn refresh_shape(shape: &mut ShapeObject, doc_info: &DocInfo) {
    if let ShapeObject::Group(group) = shape {
        for child in &mut group.children {
            refresh_shape(child, doc_info);
        }
        return;
    }
    if let Some(drawing) = shape.drawing_mut() {
        if let Some(text_box) = drawing.text_box.as_mut() {
            text_box.font_line_heights_refreshed =
                refresh_list(&mut text_box.paragraphs, doc_info, true);
        }
    }
}

/// 줄 하나가 쓰는 글자 크기와 글꼴 세로 상자의 최댓값(HWPUNIT).
struct LineBox {
    size: f64,
    height: f64,
    ascent: f64,
}

fn line_box(para: &Paragraph, doc_info: &DocInfo, start: u32, end: u32) -> Option<LineBox> {
    let in_line = |index: usize| {
        let position = para
            .char_offsets
            .get(index)
            .copied()
            .unwrap_or(index as u32);
        position >= start && position < end
    };
    let chars: Vec<(usize, char)> = para
        .text
        .chars()
        .enumerate()
        .filter(|&(index, ch)| in_line(index) && !ch.is_control() && !is_lang_neutral(ch))
        .collect();
    if chars.is_empty() {
        return None;
    }
    let mut found = LineBox {
        size: 0.0,
        height: 0.0,
        ascent: 0.0,
    };
    for (index, ch) in chars {
        let position = para
            .char_offsets
            .get(index)
            .copied()
            .unwrap_or(index as u32);
        let shape_id = para
            .char_shapes
            .iter()
            .take_while(|run| run.start_pos <= position)
            .last()?
            .char_shape_id;
        let shape = doc_info.char_shapes.get(shape_id as usize)?;
        let lang = detect_lang_category(ch).min(6);
        let size = f64::from(shape.base_size) * f64::from(shape.relative_sizes[lang]) / 100.0;
        let face = doc_info
            .font_faces
            .get(lang)?
            .get(usize::from(shape.font_ids[lang]))?;
        let (ascent, descent) = font_line_box_em(&face.name, shape.bold, shape.italic)?;
        found.size = found.size.max(size);
        found.height = found.height.max(size * (ascent + descent));
        found.ascent = found.ascent.max(size * ascent);
    }
    (found.size > 0.0).then_some(found)
}

/// 퍼센트 줄 간격: 한컴은 4 HWPUNIT 배수로 반올림한다(`compute_line_spacing_hwp` 와 같은 식).
fn percent_spacing(line_height: i32, percent: i32) -> i32 {
    let raw = f64::from(line_height) * f64::from(percent - 100) / 100.0;
    ((raw.abs() / 4.0 + 0.5).floor() * 4.0 * raw.signum()) as i32
}

/// 고친 문단이 늘어난 세로 길이(HWPUNIT). 고치지 않으면 None.
fn refresh_paragraph(para: &mut Paragraph, doc_info: &DocInfo) -> Option<i32> {
    let shape = doc_info.para_shapes.get(para.para_shape_id as usize)?;
    if shape.attr1 & FONT_LINE_HEIGHT == 0
        || shape.line_spacing_type != LineSpacingType::Percent
        || para.line_segs.is_empty()
        || para.stored_text_partition_is_dirty()
    {
        return None;
    }
    let mut lines = para.line_segs.clone();
    for (index, line) in para.line_segs.iter().enumerate() {
        if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0 {
            return None;
        }
        let start = para.line_seg_text_start(index);
        let end = if index + 1 < para.line_segs.len() {
            para.line_seg_text_start(index + 1)
        } else {
            u32::MAX
        };
        let found = line_box(para, doc_info, start, end)?;
        let height = found.height.round() as i32;
        // 상자를 이미 적용한 줄, 또는 글자 크기와 다른 근거로 잡힌 줄(개체 등)은 그대로 둔다.
        if (f64::from(line.line_height) - found.size).abs() > 1.0 || height <= line.line_height {
            return None;
        }
        let fixed = &mut lines[index];
        fixed.line_height = height;
        fixed.text_height = height;
        fixed.baseline_distance = found.ascent.round() as i32;
        fixed.line_spacing = percent_spacing(height, shape.line_spacing);
    }
    let mut grown = 0i32;
    for (fixed, source) in lines.iter_mut().zip(&para.line_segs) {
        fixed.vertical_pos = source.vertical_pos + grown;
        grown +=
            (fixed.line_height + fixed.line_spacing) - (source.line_height + source.line_spacing);
    }
    for (line, fixed) in para.line_segs.iter_mut().zip(lines) {
        *line = fixed;
    }
    Some(grown)
}
