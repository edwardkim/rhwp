//! Memo (HWP `MEMO/` field) reading and editing.
//!
//! A memo is a field control whose command starts with `MEMO/` and whose body
//! paragraphs are stored in the model (`Field::memo_paragraphs`). HWP5 keeps the
//! body in a list after the section's last body paragraph; HWPX keeps it in the
//! field's `subList`. This module exposes one document-order list and the edits
//! the editor needs: insert, set text, remove.
//!
//! The field command is the HWP5 shape:
//! `MEMO/<memoShapeIDRef>/<Number>/<id1>/<id2>/<Author>/\;;`.
//! Hancom writes the control record with `%unk` and the opening field character
//! with `%%me`; the serializer keeps that split for every memo field.

use std::collections::{HashMap, HashSet};

use crate::document_core::helpers::json_escape;
use crate::document_core::queries::field_query::{shape_lists, FieldLocation, NestedEntry};
use crate::document_core::DocumentCore;
use crate::model::control::{Control, Field, FieldType, Parameter};
use crate::model::document::Document;
use crate::model::paragraph::{FieldRange, Paragraph};
use crate::parser::tags;

/// Maximum memo text size in UTF-16 code units.
pub const MEMO_TEXT_MAX_UNITS: usize = 100_000;
/// Maximum memo author size in UTF-16 code units.
pub const MEMO_AUTHOR_MAX_UNITS: usize = 64;

/// What went wrong with a memo request. The wasm layer turns this into the
/// `{ok:false,code,message}` envelope the app expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoErrorCode {
    NotFound,
    InvalidRange,
    Overlap,
    UnsupportedPlace,
    ReadOnly,
    TextTooLong,
    InvalidAuthor,
    InsertFailed,
    RemoveFailed,
}

impl MemoErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoErrorCode::NotFound => "memo_not_found",
            MemoErrorCode::InvalidRange => "memo_invalid_range",
            MemoErrorCode::Overlap => "memo_overlap",
            MemoErrorCode::UnsupportedPlace => "memo_unsupported_place",
            MemoErrorCode::ReadOnly => "memo_read_only",
            MemoErrorCode::TextTooLong => "memo_text_too_long",
            MemoErrorCode::InvalidAuthor => "memo_invalid_author",
            MemoErrorCode::InsertFailed => "memo_insert_failed",
            MemoErrorCode::RemoveFailed => "memo_remove_failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoError {
    pub code: MemoErrorCode,
    pub message: String,
}

impl MemoError {
    pub fn new(code: MemoErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"ok\":false,\"code\":\"{}\",\"message\":\"{}\"}}",
            self.code.as_str(),
            json_escape(&self.message)
        )
    }

    fn not_found(id: u32) -> Self {
        Self::new(MemoErrorCode::NotFound, format!("메모 {} 없음", id))
    }

    fn invalid_range(message: impl Into<String>) -> Self {
        Self::new(MemoErrorCode::InvalidRange, message)
    }

    fn overlap() -> Self {
        Self::new(MemoErrorCode::Overlap, "이미 메모가 있는 범위")
    }

    fn unsupported_place(place: &str) -> Self {
        Self::new(
            MemoErrorCode::UnsupportedPlace,
            format!("{place} 안의 메모는 고칠 수 없음"),
        )
    }

    fn read_only(reason: &str) -> Self {
        Self::new(
            MemoErrorCode::ReadOnly,
            format!("메모를 고칠 수 없음: {reason}"),
        )
    }

    fn text_too_long() -> Self {
        Self::new(MemoErrorCode::TextTooLong, "메모 글자가 너무 김")
    }

    fn invalid_author() -> Self {
        Self::new(MemoErrorCode::InvalidAuthor, "메모 작성자 이름이 너무 김")
    }

    fn insert_failed(message: impl Into<String>) -> Self {
        Self::new(MemoErrorCode::InsertFailed, message)
    }

    fn remove_failed(message: impl Into<String>) -> Self {
        Self::new(MemoErrorCode::RemoveFailed, message)
    }
}

/// One memo found in the document, in document order.
#[derive(Debug, Clone)]
struct MemoHit {
    id: u32,
    place: &'static str,
    /// Present only where the API can edit (body, table cell, text box).
    location: Option<FieldLocation>,
    control_idx: usize,
    field: Field,
    /// Same-paragraph range. A multi-paragraph memo has none; its anchor is
    /// `anchor_start` in the paragraph that holds the control.
    range: Option<(usize, usize)>,
    anchor_start: usize,
    section_index: usize,
    para_index: Option<usize>,
}

pub(crate) fn is_memo_field(field: &Field) -> bool {
    field.field_type == FieldType::Memo || field.command.starts_with("MEMO/")
}

/// Memo numbers are document-wide; the next one is one past the largest.
pub(crate) fn max_memo_number(doc: &Document) -> u32 {
    fn walk(paragraphs: &[Paragraph], max: &mut u32) {
        for para in paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Field(field) if is_memo_field(field) => {
                        *max = (*max).max(memo_number(field));
                    }
                    Control::Table(table) => {
                        for cell in &table.cells {
                            walk(&cell.paragraphs, max);
                        }
                    }
                    Control::Shape(shape) => {
                        for (_, list) in shape_lists(shape) {
                            walk(list, max);
                        }
                    }
                    Control::Header(h) => walk(&h.paragraphs, max),
                    Control::Footer(f) => walk(&f.paragraphs, max),
                    Control::Footnote(note) => walk(&note.paragraphs, max),
                    Control::Endnote(note) => walk(&note.paragraphs, max),
                    _ => {}
                }
            }
        }
    }

    let mut max = 0;
    for section in &doc.sections {
        walk(&section.paragraphs, &mut max);
    }
    max
}

/// The memo number a field names, from the trailing slot or else the command.
pub(crate) fn memo_number(field: &Field) -> u32 {
    if field.memo_index != 0 {
        return field.memo_index;
    }
    field
        .command
        .split('/')
        .nth(2)
        .and_then(|token| token.parse().ok())
        .unwrap_or(0)
}

fn command_tokens(field: &Field) -> Vec<&str> {
    field.command.split('/').collect()
}

fn memo_shape_ref(field: &Field) -> &str {
    command_tokens(field).get(1).copied().unwrap_or("65535")
}

fn memo_author(field: &Field) -> String {
    parameter_string(field, "Author")
        .or_else(|| command_tokens(field).get(5).map(|s| (*s).to_string()))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn memo_date(field: &Field) -> Option<String> {
    parameter_string(field, "CreateDateTime").or_else(|| parameter_string(field, "Date"))
}

fn parameter_string(field: &Field, name: &str) -> Option<String> {
    field.parameters.items.iter().find_map(|item| match item {
        Parameter::String {
            name: Some(key),
            value,
            ..
        } if key == name => Some(value.clone()),
        _ => None,
    })
}

fn memo_text(field: &Field) -> String {
    field
        .memo_paragraphs
        .iter()
        .map(|para| para.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Rewrite a memo's command after an identity change. The shape and author are
/// kept; the number and the two instance ids are new.
fn rewritten_memo_command(field: &Field, number: u32) -> String {
    let shape = memo_shape_ref(field).parse().unwrap_or(65535);
    memo_command(shape, number, field.field_id, &memo_author(field))
}

/// Carry a new memo number into the HWPX parameters. The HWPX writer prefers
/// the parameters over the command, so a renumbered memo must not keep the
/// copied `Number`, `ID` and `Command`; the verbatim original is dropped.
fn set_memo_parameters(field: &mut Field, number: u32) {
    field.raw_parameters_xml = None;
    for item in &mut field.parameters.items {
        match item {
            Parameter::Integer {
                name: Some(key),
                value,
            } if key == "Number" => *value = number as i64,
            Parameter::String {
                name: Some(key),
                value,
                ..
            } => match key.as_str() {
                "Number" => *value = number.to_string(),
                "ID" => *value = format!("memo{number}"),
                "Command" => *value = field.command.clone(),
                _ => {}
            },
            _ => {}
        }
    }
}

/// Give every memo in a detached copy a fresh document-wide number. Used after
/// clipboard identities are remapped; the body travels with the field.
pub(crate) fn renumber_memos(paragraphs: &mut [Paragraph], mut next: u32) -> u32 {
    let mut numbers = HashMap::new();

    fn walk(paragraphs: &mut [Paragraph], next: &mut u32, numbers: &mut HashMap<u32, u32>) {
        for para in paragraphs {
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Field(field) if is_memo_field(field) => {
                        *next = (*next).max(1);
                        field.memo_index = *next;
                        field.command = rewritten_memo_command(field, *next);
                        set_memo_parameters(field, *next);
                        numbers.insert(field.field_id, *next);
                        *next = next.saturating_add(1).max(1);
                        walk(&mut field.memo_paragraphs, next, numbers);
                    }
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            walk(&mut cell.paragraphs, next, numbers);
                        }
                    }
                    Control::Shape(shape) => walk_shape(shape, next, numbers),
                    Control::Header(header) => walk(&mut header.paragraphs, next, numbers),
                    Control::Footer(footer) => walk(&mut footer.paragraphs, next, numbers),
                    Control::Footnote(note) => walk(&mut note.paragraphs, next, numbers),
                    Control::Endnote(note) => walk(&mut note.paragraphs, next, numbers),
                    _ => {}
                }
            }
        }
    }

    fn walk_shape(
        shape: &mut crate::model::shape::ShapeObject,
        next: &mut u32,
        numbers: &mut HashMap<u32, u32>,
    ) {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(text_box) = &mut drawing.text_box {
                walk(&mut text_box.paragraphs, next, numbers);
            }
            if let Some(caption) = &mut drawing.caption {
                walk(&mut caption.paragraphs, next, numbers);
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &mut group.children {
                walk_shape(child, next, numbers);
            }
            if let Some(caption) = &mut group.caption {
                walk(&mut caption.paragraphs, next, numbers);
            }
        }
    }

    fn update_ends(paragraphs: &mut [Paragraph], numbers: &HashMap<u32, u32>) {
        for para in paragraphs {
            for end in &mut para.orphan_field_ends {
                if let Some(number) = numbers.get(&end.begin_id_ref) {
                    end.memo_index = *number;
                    if end.begin_ctrl_id == 0 {
                        end.begin_ctrl_id = tags::FIELD_MEMO;
                    }
                }
            }
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Field(field) => update_ends(&mut field.memo_paragraphs, numbers),
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            update_ends(&mut cell.paragraphs, numbers);
                        }
                    }
                    Control::Shape(shape) => update_shape(shape, numbers),
                    Control::Header(header) => update_ends(&mut header.paragraphs, numbers),
                    Control::Footer(footer) => update_ends(&mut footer.paragraphs, numbers),
                    Control::Footnote(note) => update_ends(&mut note.paragraphs, numbers),
                    Control::Endnote(note) => update_ends(&mut note.paragraphs, numbers),
                    _ => {}
                }
            }
        }
    }

    fn update_shape(shape: &mut crate::model::shape::ShapeObject, numbers: &HashMap<u32, u32>) {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(text_box) = &mut drawing.text_box {
                update_ends(&mut text_box.paragraphs, numbers);
            }
            if let Some(caption) = &mut drawing.caption {
                update_ends(&mut caption.paragraphs, numbers);
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &mut group.children {
                update_shape(child, numbers);
            }
            if let Some(caption) = &mut group.caption {
                update_ends(&mut caption.paragraphs, numbers);
            }
        }
    }

    walk(paragraphs, &mut next, &mut numbers);
    update_ends(paragraphs, &numbers);
    next
}

/// A clipboard can hold a memo's opening paragraph without the paragraph that
/// closes it (or the closing paragraph alone). Such a fragment would paste a
/// dangling field; drop the field and keep the visible text instead.
pub(crate) fn prune_incomplete_memos(paragraphs: &mut [Paragraph]) -> Result<(), MemoError> {
    let mut memo_ids = HashSet::new();
    let mut closed = HashSet::new();

    fn scan(paragraphs: &[Paragraph], memo_ids: &mut HashSet<u32>, closed: &mut HashSet<u32>) {
        for para in paragraphs {
            for range in &para.field_ranges {
                if let Some(Control::Field(field)) = para.controls.get(range.control_idx) {
                    if is_memo_field(field) {
                        memo_ids.insert(field.field_id);
                        closed.insert(field.field_id);
                    }
                }
            }
            for ctrl in &para.controls {
                match ctrl {
                    Control::Field(field) if is_memo_field(field) => {
                        memo_ids.insert(field.field_id);
                    }
                    Control::Field(field) => scan(&field.memo_paragraphs, memo_ids, closed),
                    Control::Table(table) => {
                        for cell in &table.cells {
                            scan(&cell.paragraphs, memo_ids, closed);
                        }
                    }
                    Control::Shape(shape) => scan_shape(shape, memo_ids, closed),
                    Control::Header(header) => scan(&header.paragraphs, memo_ids, closed),
                    Control::Footer(footer) => scan(&footer.paragraphs, memo_ids, closed),
                    Control::Footnote(note) => scan(&note.paragraphs, memo_ids, closed),
                    Control::Endnote(note) => scan(&note.paragraphs, memo_ids, closed),
                    _ => {}
                }
            }
            for end in &para.orphan_field_ends {
                if end.begin_ctrl_id == tags::FIELD_MEMO || end.memo_index != 0 {
                    closed.insert(end.begin_id_ref);
                }
            }
        }
    }

    fn scan_shape(
        shape: &crate::model::shape::ShapeObject,
        memo_ids: &mut HashSet<u32>,
        closed: &mut HashSet<u32>,
    ) {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing() {
            if let Some(text_box) = &drawing.text_box {
                scan(&text_box.paragraphs, memo_ids, closed);
            }
            if let Some(caption) = &drawing.caption {
                scan(&caption.paragraphs, memo_ids, closed);
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &group.children {
                scan_shape(child, memo_ids, closed);
            }
            if let Some(caption) = &group.caption {
                scan(&caption.paragraphs, memo_ids, closed);
            }
        }
    }

    fn drop_fields(paragraphs: &mut [Paragraph], closed: &HashSet<u32>) -> Result<(), MemoError> {
        for para in paragraphs {
            let drop: Vec<usize> = para
                .controls
                .iter()
                .enumerate()
                .filter_map(|(index, ctrl)| match ctrl {
                    Control::Field(field)
                        if is_memo_field(field) && !closed.contains(&field.field_id) =>
                    {
                        Some(index)
                    }
                    _ => None,
                })
                .collect();
            for index in drop.into_iter().rev() {
                remove_begin_slot(para, index)?;
            }
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Field(field) => drop_fields(&mut field.memo_paragraphs, closed)?,
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            drop_fields(&mut cell.paragraphs, closed)?;
                        }
                    }
                    Control::Shape(shape) => drop_shape_fields(shape, closed)?,
                    Control::Header(header) => drop_fields(&mut header.paragraphs, closed)?,
                    Control::Footer(footer) => drop_fields(&mut footer.paragraphs, closed)?,
                    Control::Footnote(note) => drop_fields(&mut note.paragraphs, closed)?,
                    Control::Endnote(note) => drop_fields(&mut note.paragraphs, closed)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn drop_shape_fields(
        shape: &mut crate::model::shape::ShapeObject,
        closed: &HashSet<u32>,
    ) -> Result<(), MemoError> {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(text_box) = &mut drawing.text_box {
                drop_fields(&mut text_box.paragraphs, closed)?;
            }
            if let Some(caption) = &mut drawing.caption {
                drop_fields(&mut caption.paragraphs, closed)?;
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &mut group.children {
                drop_shape_fields(child, closed)?;
            }
            if let Some(caption) = &mut group.caption {
                drop_fields(&mut caption.paragraphs, closed)?;
            }
        }
        Ok(())
    }

    fn drop_ends(paragraphs: &mut [Paragraph], memo_ids: &HashSet<u32>) -> Result<(), MemoError> {
        for para in paragraphs {
            let drop: Vec<usize> = para
                .orphan_field_ends
                .iter()
                .enumerate()
                .filter_map(|(index, end)| {
                    let memo_end = end.begin_ctrl_id == tags::FIELD_MEMO || end.memo_index != 0;
                    (memo_end && !memo_ids.contains(&end.begin_id_ref)).then_some(index)
                })
                .collect();
            for index in drop.into_iter().rev() {
                remove_orphan_end_slot(para, index)?;
            }
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Field(field) => drop_ends(&mut field.memo_paragraphs, memo_ids)?,
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            drop_ends(&mut cell.paragraphs, memo_ids)?;
                        }
                    }
                    Control::Shape(shape) => drop_shape_ends(shape, memo_ids)?,
                    Control::Header(header) => drop_ends(&mut header.paragraphs, memo_ids)?,
                    Control::Footer(footer) => drop_ends(&mut footer.paragraphs, memo_ids)?,
                    Control::Footnote(note) => drop_ends(&mut note.paragraphs, memo_ids)?,
                    Control::Endnote(note) => drop_ends(&mut note.paragraphs, memo_ids)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn drop_shape_ends(
        shape: &mut crate::model::shape::ShapeObject,
        memo_ids: &HashSet<u32>,
    ) -> Result<(), MemoError> {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(text_box) = &mut drawing.text_box {
                drop_ends(&mut text_box.paragraphs, memo_ids)?;
            }
            if let Some(caption) = &mut drawing.caption {
                drop_ends(&mut caption.paragraphs, memo_ids)?;
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &mut group.children {
                drop_shape_ends(child, memo_ids)?;
            }
            if let Some(caption) = &mut group.caption {
                drop_ends(&mut caption.paragraphs, memo_ids)?;
            }
        }
        Ok(())
    }

    scan(paragraphs, &mut memo_ids, &mut closed);
    drop_fields(paragraphs, &closed)?;
    // The dropped fields are no longer present; an end that pointed at one is
    // now dangling too.
    let dangling: Vec<u32> = memo_ids.difference(&closed).copied().collect();
    for id in dangling {
        memo_ids.remove(&id);
    }
    drop_ends(paragraphs, &memo_ids)?;
    Ok(())
}

/// Delete text, but drop a memo whose whole non-empty anchor disappears in this
/// edit. A memo whose anchor is only shortened stays. Memos that were already
/// zero-length anchors are untouched.
pub(crate) fn delete_text_at_keeping_memos(
    para: &mut Paragraph,
    char_offset: usize,
    count: usize,
) -> usize {
    if count == 0 {
        return 0;
    }
    let mut anchored_memos: Vec<(usize, u32)> = Vec::new();
    for range in &para.field_ranges {
        if range.start_char_idx >= range.end_char_idx {
            continue;
        }
        if let Some(Control::Field(field)) = para.controls.get(range.control_idx) {
            if is_memo_field(field) {
                anchored_memos.push((range.control_idx, field.field_id));
            }
        }
    }
    let deleted = para.delete_text_at(char_offset, count);
    if deleted == 0 {
        return 0;
    }

    // Later controls first: removing one shifts the indices after it.
    anchored_memos.sort_by(|a, b| b.0.cmp(&a.0));
    for (control_idx, field_id) in anchored_memos {
        let Some(range_idx) = memo_field_range_index(para, control_idx) else {
            continue;
        };
        let collapsed = {
            let range = &para.field_ranges[range_idx];
            range.start_char_idx == range.end_char_idx
        };
        if !collapsed {
            continue;
        }
        let still_memo = matches!(
            para.controls.get(control_idx),
            Some(Control::Field(field))
                if is_memo_field(field) && (field_id == 0 || field.field_id == field_id)
        );
        if !still_memo {
            continue;
        }
        if remove_memo_field_in_para(para, control_idx, range_idx).is_err() {
            // The raw slot layout was already unusual; rebuild the paragraph
            // without the field rather than leave a zero-length memo behind.
            drop_memo_control_fallback(para, control_idx);
        }
    }
    deleted
}

/// Replace text in one paragraph. Non-empty text keeps memo fields and lets the
/// insertion expand their ranges; empty text is a deletion and may drop them.
pub(crate) fn replace_text_at_keeping_memos(
    para: &mut Paragraph,
    char_offset: usize,
    count: usize,
    new_text: &str,
) -> (usize, usize) {
    if new_text.is_empty() {
        (delete_text_at_keeping_memos(para, char_offset, count), 0)
    } else {
        let deleted = para.delete_text_at(char_offset, count);
        let inserted = para.insert_text_at(char_offset, new_text);
        (deleted, inserted)
    }
}

fn drop_memo_control_fallback(para: &mut Paragraph, control_idx: usize) {
    para.field_ranges
        .retain(|range| range.control_idx != control_idx);
    for range in &mut para.field_ranges {
        if range.control_idx > control_idx {
            range.control_idx -= 1;
        }
    }
    if control_idx < para.controls.len() {
        para.controls.remove(control_idx);
    }
    if control_idx < para.ctrl_data_records.len() {
        para.ctrl_data_records.remove(control_idx);
    }
    let positions = para.control_text_positions();
    para.rebuild_char_offsets(&positions);
    para.invalidate_layout_inputs();
}

/// Drop memo fields that arrived through foreign/HTML paste: their HWPX body is
/// not part of that clipboard, so keeping the anchor would leave a dangling
/// field. The visible text stays.
pub(crate) fn drop_memo_fields(paragraphs: &mut [Paragraph]) {
    fn walk(paragraphs: &mut [Paragraph]) {
        for para in paragraphs {
            let remove: Vec<bool> = para
                .controls
                .iter()
                .map(|ctrl| matches!(ctrl, Control::Field(field) if is_memo_field(field)))
                .collect();
            if remove.iter().any(|drop| *drop) {
                let old_controls = std::mem::take(&mut para.controls);
                let old_records = std::mem::take(&mut para.ctrl_data_records);
                let mut index_map = vec![None; old_controls.len()];
                let mut controls = Vec::new();
                let mut records = Vec::new();
                for (index, control) in old_controls.into_iter().enumerate() {
                    if remove.get(index).copied().unwrap_or(false) {
                        continue;
                    }
                    index_map[index] = Some(controls.len());
                    controls.push(control);
                    records.push(old_records.get(index).cloned().flatten());
                }
                para.controls = controls;
                para.ctrl_data_records = records;
                para.field_ranges = para
                    .field_ranges
                    .drain(..)
                    .filter_map(|mut range| {
                        range.control_idx = index_map.get(range.control_idx).copied().flatten()?;
                        Some(range)
                    })
                    .collect();
                let positions = para.control_text_positions();
                para.rebuild_char_offsets(&positions);
                para.invalidate_layout_inputs();
            }
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            walk(&mut cell.paragraphs);
                        }
                    }
                    Control::Shape(shape) => walk_shape(shape),
                    Control::Header(header) => walk(&mut header.paragraphs),
                    Control::Footer(footer) => walk(&mut footer.paragraphs),
                    Control::Footnote(note) => walk(&mut note.paragraphs),
                    Control::Endnote(note) => walk(&mut note.paragraphs),
                    _ => {}
                }
            }
        }
    }

    fn walk_shape(shape: &mut crate::model::shape::ShapeObject) {
        use crate::model::shape::ShapeObject;
        if let Some(drawing) = shape.drawing_mut() {
            if let Some(text_box) = &mut drawing.text_box {
                walk(&mut text_box.paragraphs);
            }
            if let Some(caption) = &mut drawing.caption {
                walk(&mut caption.paragraphs);
            }
        }
        if let ShapeObject::Group(group) = shape {
            for child in &mut group.children {
                walk_shape(child);
            }
            if let Some(caption) = &mut group.caption {
                walk(&mut caption.paragraphs);
            }
        }
    }

    walk(paragraphs);
}

/// Plain text with no control or object: every memo paragraph must have only
/// characters, and all paragraphs must share one paragraph and character shape.
fn memo_readonly_reason(paragraphs: &[Paragraph]) -> Option<&'static str> {
    let mut object = false;
    let mut para_shape: Option<u16> = None;
    let mut char_shape: Option<u32> = None;
    let mut formatted = false;

    for para in paragraphs {
        if !para.controls.is_empty()
            || !para.field_ranges.is_empty()
            || !para.orphan_field_ends.is_empty()
            || para.text.contains('\u{fffc}')
        {
            object = true;
        }
        match para_shape {
            Some(id) if id != para.para_shape_id => formatted = true,
            None => para_shape = Some(para.para_shape_id),
            _ => {}
        }
        for shape in &para.char_shapes {
            match char_shape {
                Some(id) if id != shape.char_shape_id => formatted = true,
                None => char_shape = Some(shape.char_shape_id),
                _ => {}
            }
        }
    }

    if object {
        Some("object")
    } else if formatted {
        Some("formatted")
    } else {
        None
    }
}

fn effective_id(field: &Field, location: &FieldLocation, control_idx: usize) -> u32 {
    if field.field_id != 0 {
        return field.field_id;
    }
    // Broken files can carry field id 0. Keep a stable address for this session
    // without inventing an id in the document until the memo is edited.
    let mut parts = vec![
        location.section_index,
        location.para_index,
        control_idx,
        0x4d45_4d4f,
    ];
    for entry in &location.nested_path {
        match entry {
            NestedEntry::TableCell {
                control_index,
                cell_index,
                para_index,
            } => parts.extend_from_slice(&[*control_index, *cell_index, *para_index]),
            NestedEntry::TextBox {
                control_index,
                para_index,
            } => parts.extend_from_slice(&[*control_index, 0, *para_index]),
        }
    }
    let mut hash: u32 = 2_166_136_261;
    for part in parts {
        for byte in (part as u64).to_le_bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16_777_619);
        }
    }
    hash | 1
}

fn unsupported_id(field: &Field, section_index: usize, control_idx: usize, ordinal: usize) -> u32 {
    if field.field_id != 0 {
        return field.field_id;
    }
    let mut hash: u32 = 2_166_136_261;
    for part in [section_index, control_idx, ordinal, 0x554e_5355] {
        for byte in (part as u64).to_le_bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16_777_619);
        }
    }
    hash | 1
}

fn hits_in_paragraph(
    para: &Paragraph,
    location: Option<&FieldLocation>,
    place: &'static str,
    section_index: usize,
    out: &mut Vec<MemoHit>,
) {
    for (range_idx, range) in para.field_ranges.iter().enumerate() {
        let Some(Control::Field(field)) = para.controls.get(range.control_idx) else {
            continue;
        };
        if !is_memo_field(field) {
            continue;
        }
        let (id, para_index) = match location {
            Some(location) => (
                effective_id(field, location, range.control_idx),
                Some(location.para_index),
            ),
            None => (
                unsupported_id(field, section_index, range.control_idx, range_idx),
                None,
            ),
        };
        out.push(MemoHit {
            id,
            place,
            location: location.cloned(),
            control_idx: range.control_idx,
            field: field.clone(),
            range: Some((range.start_char_idx, range.end_char_idx)),
            anchor_start: range.start_char_idx,
            section_index,
            para_index,
        });
    }

    // A memo can span paragraphs. HWP5 then has no field range in the anchor
    // paragraph, only the field begin (and an orphan end later). Report it at
    // the begin character so the body and author are still reachable.
    let positions = para.control_text_positions();
    for (control_idx, ctrl) in para.controls.iter().enumerate() {
        let Control::Field(field) = ctrl else {
            continue;
        };
        if !is_memo_field(field) {
            continue;
        }
        if para
            .field_ranges
            .iter()
            .any(|range| range.control_idx == control_idx)
        {
            continue;
        }
        let anchor_start = positions.get(control_idx).copied().unwrap_or(0);
        let (id, para_index) = match location {
            Some(location) => (
                effective_id(field, location, control_idx),
                Some(location.para_index),
            ),
            None => (
                unsupported_id(field, section_index, control_idx, anchor_start),
                None,
            ),
        };
        out.push(MemoHit {
            id,
            place,
            location: location.cloned(),
            control_idx,
            field: field.clone(),
            range: None,
            anchor_start,
            section_index,
            para_index,
        });
    }
}

fn hits_in_supported(
    paragraphs: &[Paragraph],
    base: &FieldLocation,
    place: &'static str,
    out: &mut Vec<MemoHit>,
) {
    for para in paragraphs {
        hits_in_paragraph(para, Some(base), place, base.section_index, out);
        for (control_idx, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Table(table) => {
                    for (cell_index, cell) in table.cells.iter().enumerate() {
                        for (para_index, cell_para) in cell.paragraphs.iter().enumerate() {
                            let mut location = base.clone();
                            location.nested_path.push(NestedEntry::TableCell {
                                control_index: control_idx,
                                cell_index,
                                para_index,
                            });
                            hits_in_supported(
                                std::slice::from_ref(cell_para),
                                &location,
                                "tableCell",
                                out,
                            );
                        }
                    }
                }
                Control::Shape(shape) => {
                    for (node, list) in shape_lists(shape) {
                        if node == 0 {
                            for (para_index, box_para) in list.iter().enumerate() {
                                let mut location = base.clone();
                                location.nested_path.push(NestedEntry::TextBox {
                                    control_index: control_idx,
                                    para_index,
                                });
                                hits_in_supported(
                                    std::slice::from_ref(box_para),
                                    &location,
                                    "textBox",
                                    out,
                                );
                            }
                        } else {
                            hits_in_any(list, base.section_index, "unknown", out);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Memos the API cannot address (header/footer/notes/captions): still reported,
/// but read-only.
fn hits_in_any(
    paragraphs: &[Paragraph],
    section_index: usize,
    place: &'static str,
    out: &mut Vec<MemoHit>,
) {
    for para in paragraphs {
        hits_in_paragraph(para, None, place, section_index, out);
        for ctrl in &para.controls {
            match ctrl {
                Control::Table(table) => {
                    for cell in &table.cells {
                        hits_in_any(&cell.paragraphs, section_index, place, out);
                    }
                }
                Control::Shape(shape) => {
                    for (_, list) in shape_lists(shape) {
                        hits_in_any(list, section_index, place, out);
                    }
                }
                Control::Header(header) => {
                    hits_in_any(&header.paragraphs, section_index, "header", out)
                }
                Control::Footer(footer) => {
                    hits_in_any(&footer.paragraphs, section_index, "footer", out)
                }
                Control::Footnote(note) => {
                    hits_in_any(&note.paragraphs, section_index, "footnote", out)
                }
                Control::Endnote(note) => {
                    hits_in_any(&note.paragraphs, section_index, "endnote", out)
                }
                _ => {}
            }
        }
    }
}

impl DocumentCore {
    fn document_protected(&self) -> bool {
        self.document.header.encrypted || self.document.header.distribution
    }

    fn memo_hits(&self) -> Vec<MemoHit> {
        let mut hits = Vec::new();
        for (section_index, section) in self.document.sections.iter().enumerate() {
            for (para_index, para) in section.paragraphs.iter().enumerate() {
                let location = FieldLocation {
                    section_index,
                    para_index,
                    nested_path: Vec::new(),
                };
                hits_in_supported(std::slice::from_ref(para), &location, "body", &mut hits);
                for ctrl in &para.controls {
                    match ctrl {
                        Control::Header(header) => {
                            hits_in_any(&header.paragraphs, section_index, "header", &mut hits)
                        }
                        Control::Footer(footer) => {
                            hits_in_any(&footer.paragraphs, section_index, "footer", &mut hits)
                        }
                        Control::Footnote(note) => {
                            hits_in_any(&note.paragraphs, section_index, "footnote", &mut hits)
                        }
                        Control::Endnote(note) => {
                            hits_in_any(&note.paragraphs, section_index, "endnote", &mut hits)
                        }
                        _ => {}
                    }
                }
            }
        }
        // A paragraph can hold a range-less multi-paragraph memo at its opening
        // character and a ranged memo later in the text. The walker sees controls
        // before ranges in that case, so sort by the document position itself.
        hits.sort_by(|a, b| {
            a.section_index
                .cmp(&b.section_index)
                .then_with(|| {
                    a.para_index
                        .unwrap_or(usize::MAX)
                        .cmp(&b.para_index.unwrap_or(usize::MAX))
                })
                .then_with(|| a.anchor_start.cmp(&b.anchor_start))
                .then_with(|| memo_number(&a.field).cmp(&memo_number(&b.field)))
                .then_with(|| a.id.cmp(&b.id))
        });
        hits
    }

    /// `getMemoList`: JSON array in document order.
    pub fn get_memo_list_json(&self) -> String {
        let protected = self.document_protected();
        let entries: Vec<String> = self
            .memo_hits()
            .iter()
            .map(|hit| {
                let Some(location) = &hit.location else {
                    return memo_json(hit, protected, "unsupported_place", 0);
                };
                let page = self
                    .find_pages_for_paragraph(location.section_index, location.para_index)
                    .ok()
                    .and_then(|pages| pages.first().copied())
                    .map(|page| page + 1)
                    .unwrap_or(0);
                let reason = memo_readonly_reason(&hit.field.memo_paragraphs).unwrap_or("");
                memo_json(hit, protected, reason, page)
            })
            .collect();
        format!("[{}]", entries.join(","))
    }

    fn memo_hit(&self, id: u32) -> Result<MemoHit, MemoError> {
        self.memo_hits()
            .into_iter()
            .find(|hit| hit.id == id)
            .ok_or_else(|| MemoError::not_found(id))
    }

    fn memo_paragraph_mut(
        &mut self,
        location: &FieldLocation,
    ) -> Result<&mut Paragraph, MemoError> {
        if location.nested_path.is_empty() {
            return self
                .document
                .sections
                .get_mut(location.section_index)
                .and_then(|section| section.paragraphs.get_mut(location.para_index))
                .ok_or_else(|| MemoError::insert_failed("문단 위치를 찾을 수 없음"));
        }
        let path: Vec<(usize, usize, usize)> = location
            .nested_path
            .iter()
            .map(|entry| match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => (*control_index, *cell_index, *para_index),
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => (*control_index, 0, *para_index),
            })
            .collect();
        self.get_cell_paragraph_mut_by_path(location.section_index, location.para_index, &path)
            .map_err(|_| MemoError::insert_failed("문단 위치를 찾을 수 없음"))
    }

    fn memo_paragraphs_mut(
        &mut self,
        location: &FieldLocation,
    ) -> Result<&mut Vec<Paragraph>, MemoError> {
        if location.nested_path.is_empty() {
            return self
                .document
                .sections
                .get_mut(location.section_index)
                .map(|section| &mut section.paragraphs)
                .ok_or_else(|| MemoError::remove_failed("구역 위치를 찾을 수 없음"));
        }
        let path = Self::flat_path(location);
        self.get_cell_paragraphs_mut_by_path(location.section_index, location.para_index, &path)
            .map_err(|_| MemoError::remove_failed("문단 위치를 찾을 수 없음"))
    }

    /// A memo that spans paragraphs has no same-paragraph field range. Remove the
    /// opening slot in the anchor paragraph and the orphan closing slot later.
    fn remove_multi_paragraph_memo(
        &mut self,
        location: &FieldLocation,
        control_idx: usize,
        hit: &MemoHit,
    ) -> Result<(), MemoError> {
        let declared_id = if hit.field.field_id != 0 {
            hit.field.field_id
        } else {
            hit.id
        };
        let anchor_index = location
            .nested_path
            .last()
            .map(|entry| match entry {
                NestedEntry::TableCell { para_index, .. }
                | NestedEntry::TextBox { para_index, .. } => *para_index,
            })
            .unwrap_or(location.para_index);
        let paragraphs = self.memo_paragraphs_mut(location)?;
        let end = paragraphs
            .iter()
            .enumerate()
            .find_map(|(para_index, para)| {
                para.orphan_field_ends
                    .iter()
                    .position(|end| end.begin_id_ref == declared_id)
                    .map(|end_index| (para_index, end_index))
            })
            .ok_or_else(|| MemoError::remove_failed("메모 끝 표시를 찾을 수 없음"))?;

        remove_begin_slot(
            paragraphs
                .get_mut(anchor_index)
                .ok_or_else(|| MemoError::remove_failed("메모 문단을 찾을 수 없음"))?,
            control_idx,
        )?;
        remove_orphan_end_slot(
            paragraphs
                .get_mut(end.0)
                .ok_or_else(|| MemoError::remove_failed("메모 끝 문단을 찾을 수 없음"))?,
            end.1,
        )?;
        Ok(())
    }

    fn flat_path(location: &FieldLocation) -> Vec<(usize, usize, usize)> {
        location
            .nested_path
            .iter()
            .map(|entry| match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => (*control_index, *cell_index, *para_index),
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => (*control_index, 0, *para_index),
            })
            .collect()
    }

    /// Insert a memo over a one-paragraph range. `start == end` makes a
    /// zero-length anchor.
    pub fn insert_memo_at(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        start_char_idx: usize,
        end_char_idx: usize,
        author: &str,
        text: &str,
    ) -> Result<String, MemoError> {
        if self.document_protected() {
            return Err(MemoError::read_only("protected"));
        }
        let author = sanitize_author(author)?;
        check_text_size(text)?;
        let (number, field_id) = self.next_memo_identity()?;

        let section = self
            .document
            .sections
            .get(section_idx)
            .ok_or_else(|| MemoError::invalid_range("구역 위치를 찾을 수 없음"))?;
        let shape_ref = if section.section_def.memo_shape_id != 0 {
            section.section_def.memo_shape_id as u32
        } else {
            65535
        };
        let template = section
            .paragraphs
            .get(para_idx)
            .ok_or_else(|| MemoError::invalid_range("문단 위치를 찾을 수 없음"))?
            .clone();

        let mut field = new_memo_field(field_id, number, shape_ref, &author, text, &template)?;
        // A new memo is an editable field like Hancom's (editable + dirty bits).
        field.properties |= 0x8001;

        let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
            self.document
                .sections
                .get(section_idx)
                .and_then(|section| section.paragraphs.get(para_idx))
                .ok_or_else(|| MemoError::invalid_range("문단 위치를 찾을 수 없음"))?,
        );
        {
            let section = self
                .document
                .sections
                .get_mut(section_idx)
                .ok_or_else(|| MemoError::invalid_range("구역 위치를 찾을 수 없음"))?;
            section.raw_stream = None;
            let para = section
                .paragraphs
                .get_mut(para_idx)
                .ok_or_else(|| MemoError::invalid_range("문단 위치를 찾을 수 없음"))?;
            insert_memo_field_in_para(para, start_char_idx, end_char_idx, field)?;
        }

        self.reflow_paragraph(section_idx, para_idx);
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            None,
            stored_end_for_reset,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );
        self.recompose_paragraph(section_idx, para_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        Ok(format!(
            "{{\"ok\":true,\"id\":{},\"index\":{}}}",
            field_id, number
        ))
    }

    /// Insert a memo in a table cell or text box addressed by a cell path.
    pub fn insert_memo_by_path(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        path: &[(usize, usize, usize)],
        start_char_idx: usize,
        end_char_idx: usize,
        author: &str,
        text: &str,
    ) -> Result<String, MemoError> {
        if path.is_empty() {
            return Err(MemoError::invalid_range("cellPath가 비어 있음"));
        }
        if self.document_protected() {
            return Err(MemoError::read_only("protected"));
        }
        let author = sanitize_author(author)?;
        check_text_size(text)?;
        let (number, field_id) = self.next_memo_identity()?;
        let shape_ref = self
            .document
            .sections
            .get(section_idx)
            .map(|section| section.section_def.memo_shape_id)
            .unwrap_or(0);
        let shape_ref = if shape_ref != 0 {
            shape_ref as u32
        } else {
            65535
        };
        let template = self
            .get_cell_paragraph_mut_by_path(section_idx, para_idx, path)
            .map(|para| para.clone())
            .map_err(|_| MemoError::invalid_range("문단 위치를 찾을 수 없음"))?;
        let mut field = new_memo_field(field_id, number, shape_ref, &author, text, &template)?;
        field.properties |= 0x8001;

        {
            let para = self
                .get_cell_paragraph_mut_by_path(section_idx, para_idx, path)
                .map_err(|_| MemoError::invalid_range("문단 위치를 찾을 수 없음"))?;
            insert_memo_field_in_para(para, start_char_idx, end_char_idx, field)?;
        }
        let inner_para = path.last().map(|entry| entry.2).unwrap_or(0);
        self.reflow_cell_paragraph_by_path(section_idx, para_idx, path, inner_para);
        self.recalculate_cell_paragraph_vpos_by_path(section_idx, para_idx, path, inner_para, None);
        self.mark_cell_control_dirty(section_idx, para_idx, path[0].0);
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        Ok(format!(
            "{{\"ok\":true,\"id\":{},\"index\":{}}}",
            field_id, number
        ))
    }

    /// Replace a memo's body paragraphs. Paragraph/character shapes are kept
    /// from the first body paragraph; lines split on `\n`.
    pub fn set_memo_text(&mut self, id: u32, text: &str) -> Result<String, MemoError> {
        check_text_size(text)?;
        let hit = self.memo_hit(id)?;
        if self.document_protected() {
            return Err(MemoError::read_only("protected"));
        }
        let location = hit
            .location
            .clone()
            .ok_or_else(|| MemoError::unsupported_place(hit.place))?;
        let reason = memo_readonly_reason(&hit.field.memo_paragraphs);
        if let Some(reason) = reason {
            return Err(MemoError::read_only(reason));
        }
        let template = hit
            .field
            .memo_paragraphs
            .first()
            .cloned()
            .unwrap_or_else(|| {
                self.memo_paragraph_mut(&location)
                    .map(|para| para.clone())
                    .unwrap_or_default()
            });

        let paragraphs = memo_paragraphs_from_text(text, &template)?;
        let stored_end_for_reset = self
            .document
            .sections
            .get(location.section_index)
            .and_then(|section| section.paragraphs.get(location.para_index))
            .map(crate::renderer::composer::paragraph_flow_end)
            .ok_or_else(|| MemoError::not_found(id))?;
        {
            let para = self.memo_paragraph_mut(&location)?;
            let field = match para.controls.get_mut(hit.control_idx) {
                Some(Control::Field(field)) if is_memo_field(field) => field,
                _ => return Err(MemoError::not_found(id)),
            };
            field.memo_paragraphs = paragraphs;
            // Keep the session address stable for files that carried id 0.
            if field.field_id == 0 {
                field.field_id = id;
            }
        }
        if location.nested_path.is_empty() {
            self.reflow_paragraph(location.section_index, location.para_index);
            let hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[location.section_index].paragraphs,
                location.para_index,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                hwp3_layout,
            );
            self.recompose_paragraph(location.section_index, location.para_index);
        } else {
            let path = Self::flat_path(&location);
            let inner_para = path.last().map(|entry| entry.2).unwrap_or(0);
            self.reflow_cell_paragraph_by_path(
                location.section_index,
                location.para_index,
                &path,
                inner_para,
            );
            self.recalculate_cell_paragraph_vpos_by_path(
                location.section_index,
                location.para_index,
                &path,
                inner_para,
                None,
            );
            self.mark_cell_control_dirty(location.section_index, location.para_index, path[0].0);
            self.recompose_section(location.section_index);
        }
        if let Some(section) = self.document.sections.get_mut(location.section_index) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(location.section_index);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// Remove a memo's field and body, keeping the anchor text.
    pub fn remove_memo(&mut self, id: u32) -> Result<String, MemoError> {
        let hit = self.memo_hit(id)?;
        if self.document_protected() {
            return Err(MemoError::read_only("protected"));
        }
        let location = hit
            .location
            .clone()
            .ok_or_else(|| MemoError::unsupported_place(hit.place))?;
        let stored_end_for_reset = self
            .document
            .sections
            .get(location.section_index)
            .and_then(|section| section.paragraphs.get(location.para_index))
            .map(crate::renderer::composer::paragraph_flow_end)
            .ok_or_else(|| MemoError::remove_failed("문단 위치를 찾을 수 없음"))?;

        if hit.range.is_some() {
            let para = self.memo_paragraph_mut(&location)?;
            let control_idx = memo_field_range_index(para, hit.control_idx)
                .ok_or_else(|| MemoError::remove_failed("메모 필드 범위를 찾을 수 없음"))?;
            remove_memo_field_in_para(para, hit.control_idx, control_idx)?;
        } else {
            self.remove_multi_paragraph_memo(&location, hit.control_idx, &hit)?;
        }
        if location.nested_path.is_empty() {
            self.reflow_paragraph(location.section_index, location.para_index);
            let hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[location.section_index].paragraphs,
                location.para_index,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                hwp3_layout,
            );
            self.recompose_section(location.section_index);
        } else {
            let path = Self::flat_path(&location);
            let inner_para = path.last().map(|entry| entry.2).unwrap_or(0);
            self.reflow_cell_paragraph_by_path(
                location.section_index,
                location.para_index,
                &path,
                inner_para,
            );
            self.recalculate_cell_paragraph_vpos_by_path(
                location.section_index,
                location.para_index,
                &path,
                inner_para,
                None,
            );
            self.mark_cell_control_dirty(location.section_index, location.para_index, path[0].0);
            self.recompose_section(location.section_index);
        }
        if let Some(section) = self.document.sections.get_mut(location.section_index) {
            section.raw_stream = None;
        }
        self.mark_section_dirty(location.section_index);
        self.paginate();
        self.invalidate_page_tree_cache();
        Ok(r#"{"ok":true}"#.to_string())
    }

    fn next_memo_identity(&self) -> Result<(u32, u32), MemoError> {
        let number = max_memo_number(&self.document).saturating_add(1).max(1);
        Ok((number, self.next_click_here_field_id()))
    }
}

fn memo_json(hit: &MemoHit, protected: bool, reason: &str, page: u32) -> String {
    let (editable, readonly_reason) = if protected {
        (false, "protected")
    } else if !reason.is_empty() {
        (false, reason)
    } else {
        (true, "")
    };
    let field = &hit.field;
    let date = memo_date(field)
        .map(|value| format!("\"{}\"", json_escape(&value)))
        .unwrap_or_else(|| "null".to_string());
    let path = hit
        .location
        .as_ref()
        .map(|location| {
            if location.nested_path.is_empty() {
                "null".to_string()
            } else {
                let entries: Vec<String> = location
                    .nested_path
                    .iter()
                    .map(|entry| match entry {
                        NestedEntry::TableCell {
                            control_index,
                            cell_index,
                            para_index,
                        } => format!(
                            "{{\"type\":\"cell\",\"controlIndex\":{},\"cellIndex\":{},\"paraIndex\":{}}}",
                            control_index, cell_index, para_index
                        ),
                        NestedEntry::TextBox {
                            control_index,
                            para_index,
                        } => format!(
                            "{{\"type\":\"textbox\",\"controlIndex\":{},\"paraIndex\":{}}}",
                            control_index, para_index
                        ),
                    })
                    .collect();
                format!("[{}]", entries.join(","))
            }
        })
        .unwrap_or_else(|| "null".to_string());
    let (para_index, start, end) = match &hit.location {
        Some(location) => {
            let (start, end) = hit.range.unwrap_or((hit.anchor_start, hit.anchor_start));
            (
                location.para_index.to_string(),
                start.to_string(),
                end.to_string(),
            )
        }
        None => ("null".to_string(), "null".to_string(), "null".to_string()),
    };
    format!(
        "{{\"id\":{},\"index\":{},\"author\":\"{}\",\"date\":{},\"text\":\"{}\",\"place\":\"{}\",\"sectionIndex\":{},\"paraIndex\":{},\"startCharIdx\":{},\"endCharIdx\":{},\"path\":{},\"page\":{},\"editable\":{},\"readOnlyReason\":{}}}",
        hit.id,
        memo_number(field),
        json_escape(&memo_author(field)),
        date,
        json_escape(&memo_text(field)),
        hit.place,
        hit.section_index,
        para_index,
        start,
        end,
        path,
        page,
        editable,
        if readonly_reason.is_empty() {
            "null".to_string()
        } else {
            format!("\"{}\"", readonly_reason)
        },
    )
}

/// Read the `{author,text}` argument of the insert entries. A missing member is
/// empty; anything that is not an object of strings is rejected.
pub(crate) fn parse_memo_input(json: &str) -> Result<(String, String), MemoError> {
    let invalid = || MemoError::insert_failed("메모 입력은 {author,text} 문자열 객체여야 함");
    let value: serde_json::Value = serde_json::from_str(json).map_err(|_| invalid())?;
    let object = value.as_object().ok_or_else(invalid)?;
    let member = |key: &str| match object.get(key) {
        None => Ok(String::new()),
        Some(serde_json::Value::String(text)) => Ok(text.clone()),
        Some(_) => Err(invalid()),
    };
    Ok((member("author")?, member("text")?))
}

/// Trim, drop `/`, `\`, `;` and control characters, cap at 64 UTF-16 units.
pub(crate) fn sanitize_author(author: &str) -> Result<String, MemoError> {
    let mut out = String::new();
    for ch in author.trim().chars() {
        if ch == '/' || ch == '\\' || ch == ';' || ch.is_control() {
            continue;
        }
        out.push(ch);
    }
    if out.encode_utf16().count() > MEMO_AUTHOR_MAX_UNITS {
        return Err(MemoError::invalid_author());
    }
    Ok(out)
}

fn check_text_size(text: &str) -> Result<(), MemoError> {
    if text.encode_utf16().count() > MEMO_TEXT_MAX_UNITS {
        return Err(MemoError::text_too_long());
    }
    Ok(())
}

/// `MEMO/<memoShapeIDRef>/<Number>/<id1>/<id2>/<Author>/\;;`
pub(crate) fn memo_command(shape_ref: u32, number: u32, field_id: u32, author: &str) -> String {
    let id2 = field_id
        .wrapping_mul(2_654_435_761)
        .wrapping_add(number)
        .max(1);
    format!(
        "MEMO/{}/{}/{}/{}/{}/\\;;",
        shape_ref, number, field_id, id2, author
    )
}

fn new_memo_field(
    field_id: u32,
    number: u32,
    shape_ref: u32,
    author: &str,
    text: &str,
    template: &Paragraph,
) -> Result<Field, MemoError> {
    Ok(Field {
        field_type: FieldType::Memo,
        command: memo_command(shape_ref, number, field_id, author),
        properties: 0x8001,
        extra_properties: 0,
        field_id,
        ctrl_id: tags::FIELD_UNKNOWN,
        instance_id: None,
        raw_type: None,
        ctrl_data_name: None,
        memo_index: number,
        memo_paragraphs: memo_paragraphs_from_text(text, template)?,
        memo_text_direction: None,
        raw_parameters_xml: None,
        parameters: Default::default(),
        guide_residue: None,
        hyperlink_format: None,
    })
}

fn memo_paragraphs_from_text(
    text: &str,
    template: &Paragraph,
) -> Result<Vec<Paragraph>, MemoError> {
    let mut paragraphs = Vec::new();
    for line in text.split('\n') {
        let mut para = Paragraph::new_empty_like(template);
        if !line.is_empty() {
            para.insert_text_at(0, line);
            para.has_para_text = true;
        } else {
            para.has_para_text = false;
        }
        paragraphs.push(para);
    }
    if paragraphs.is_empty() {
        paragraphs.push(Paragraph::new_empty());
    }
    Ok(paragraphs)
}

fn memo_overlaps(para: &Paragraph, start: usize, end: usize) -> bool {
    para.field_ranges.iter().any(|range| {
        let Some(Control::Field(field)) = para.controls.get(range.control_idx) else {
            return false;
        };
        if !is_memo_field(field) {
            return false;
        }
        let (other_start, other_end) = (range.start_char_idx, range.end_char_idx);
        if start == end {
            // A zero-length anchor may sit on a boundary, not inside another memo.
            other_start < start && start < other_end
        } else {
            start < other_end && other_start < end
        }
    })
}

fn memo_field_range_index(para: &Paragraph, control_idx: usize) -> Option<usize> {
    para.field_ranges
        .iter()
        .position(|range| range.control_idx == control_idx)
}

fn raw_gaps(para: &Paragraph) -> Result<Vec<(u32, u32)>, MemoError> {
    crate::document_core::queries::field_query::raw_slot_gaps(para)
        .map_err(|_| MemoError::insert_failed("메모 자리의 원시 위치를 확인할 수 없음"))
}

fn insert_memo_field_in_para(
    para: &mut Paragraph,
    start: usize,
    end: usize,
    field: Field,
) -> Result<(), MemoError> {
    let text_len = para.text.chars().count();
    if start > end {
        return Err(MemoError::invalid_range("시작이 끝보다 뒤"));
    }
    if end > text_len {
        return Err(MemoError::invalid_range("메모 범위가 문단 끝을 넘음"));
    }
    if memo_overlaps(para, start, end) {
        return Err(MemoError::overlap());
    }
    let gaps = raw_gaps(para)?;
    let begin_pos = gaps[start].1;
    let end_pos = gaps[end].1;
    let insert_idx = gaps[..=start]
        .iter()
        .map(|&(gap_start, gap_end)| ((gap_end - gap_start) / 8) as usize)
        .sum::<usize>()
        - para
            .field_ranges
            .iter()
            .filter(|range| range.end_char_idx <= start)
            .count();

    for range in &mut para.field_ranges {
        if range.control_idx >= insert_idx {
            range.control_idx += 1;
        }
    }

    para.shift_position_metadata_for_stream_insertion(begin_pos, 8);
    para.shift_position_metadata_for_stream_insertion(end_pos + 8, 8);
    for offset in &mut para.char_offsets[start..] {
        *offset += 8;
    }
    for offset in &mut para.char_offsets[end..] {
        *offset += 8;
    }

    para.align_ctrl_data_records();
    para.controls.insert(insert_idx, Control::Field(field));
    para.ctrl_data_records.insert(insert_idx, None);

    let range = FieldRange {
        start_char_idx: start,
        end_char_idx: end,
        control_idx: insert_idx,
        ..Default::default()
    };
    let range_idx = para
        .field_ranges
        .iter()
        .position(|other| {
            other.start_char_idx > start
                || (other.start_char_idx == start && other.control_idx > insert_idx)
        })
        .unwrap_or(para.field_ranges.len());
    para.field_ranges.insert(range_idx, range);
    para.char_count = gaps[text_len].1 + 16 + 1;
    para.invalidate_layout_inputs();
    Ok(())
}

fn shift_after_removed_slot(pos: u32, slot_at: u32) -> u32 {
    if slot_at == 0 && pos == 0 {
        0
    } else if pos >= slot_at {
        pos.saturating_sub(8)
    } else {
        pos
    }
}

fn shift_metadata_after_slot(para: &mut Paragraph, slot_at: u32) {
    for shape in &mut para.char_shapes {
        shape.start_pos = shift_after_removed_slot(shape.start_pos, slot_at);
    }
    for seg in &mut para.line_segs {
        seg.text_start = shift_after_removed_slot(seg.text_start, slot_at);
    }
    for tag in &mut para.range_tags {
        tag.start = shift_after_removed_slot(tag.start, slot_at);
        tag.end = shift_after_removed_slot(tag.end, slot_at);
    }
    for mark in &mut para.markpen_marks {
        if let Some(pos) = &mut mark.utf16_pos {
            *pos = shift_after_removed_slot(*pos, slot_at);
        }
    }
}

fn remove_begin_slot(para: &mut Paragraph, control_idx: usize) -> Result<(), MemoError> {
    let anchor = para
        .control_text_positions()
        .get(control_idx)
        .copied()
        .ok_or_else(|| MemoError::remove_failed("메모 시작 자리를 찾을 수 없음"))?;
    let slot_at = para
        .char_offsets
        .get(anchor)
        .copied()
        .unwrap_or_else(|| para.char_count.saturating_sub(1));
    if slot_at < 8 {
        return Err(MemoError::remove_failed("메모 시작 표시를 찾을 수 없음"));
    }
    for offset in &mut para.char_offsets[anchor..] {
        *offset = offset.saturating_sub(8);
    }
    shift_metadata_after_slot(para, slot_at);
    para.field_ranges
        .retain(|range| range.control_idx != control_idx);
    for range in &mut para.field_ranges {
        if range.control_idx > control_idx {
            range.control_idx -= 1;
        }
    }
    para.controls.remove(control_idx);
    if control_idx < para.ctrl_data_records.len() {
        para.ctrl_data_records.remove(control_idx);
    }
    para.char_count = para.char_count.saturating_sub(8);
    para.invalidate_layout_inputs();
    Ok(())
}

fn remove_orphan_end_slot(para: &mut Paragraph, end_index: usize) -> Result<(), MemoError> {
    let char_idx = para
        .orphan_field_ends
        .get(end_index)
        .map(|end| end.char_idx)
        .ok_or_else(|| MemoError::remove_failed("메모 끝 표시를 찾을 수 없음"))?;
    let slot_at = if char_idx < para.char_offsets.len() {
        para.char_offsets[char_idx]
    } else {
        para.char_count.saturating_sub(1)
    };
    if slot_at < 8 {
        return Err(MemoError::remove_failed("메모 끝 표시를 찾을 수 없음"));
    }
    for offset in &mut para.char_offsets[char_idx..] {
        *offset = offset.saturating_sub(8);
    }
    shift_metadata_after_slot(para, slot_at);
    para.orphan_field_ends.remove(end_index);
    para.char_count = para.char_count.saturating_sub(8);
    para.invalidate_layout_inputs();
    Ok(())
}

fn remove_memo_field_in_para(
    para: &mut Paragraph,
    control_idx: usize,
    range_idx: usize,
) -> Result<(), MemoError> {
    let gaps = raw_gaps(para)
        .map_err(|_| MemoError::remove_failed("메모 자리의 원시 위치를 확인할 수 없음"))?;
    let (start, end) = {
        let range = para
            .field_ranges
            .get(range_idx)
            .ok_or_else(|| MemoError::remove_failed("메모 필드 범위를 찾을 수 없음"))?;
        (range.start_char_idx, range.end_char_idx)
    };
    if start > end || end > para.text.chars().count() {
        return Err(MemoError::remove_failed("메모 범위가 문단과 맞지 않음"));
    }
    let begin_pos = gaps[start].1;
    let end_pos = gaps[end].1;
    let text_len = para.text.chars().count();
    let shift = |pos: u32| -> u32 {
        if begin_pos == 0 && pos == 0 {
            0
        } else if pos >= end_pos {
            pos.saturating_sub(16)
        } else if pos >= begin_pos {
            pos.saturating_sub(8)
        } else {
            pos
        }
    };

    for offset in &mut para.char_offsets[start..end] {
        *offset = offset.saturating_sub(8);
    }
    for offset in &mut para.char_offsets[end..] {
        *offset = offset.saturating_sub(16);
    }
    for shape in &mut para.char_shapes {
        shape.start_pos = shift(shape.start_pos);
    }
    for seg in &mut para.line_segs {
        seg.text_start = shift(seg.text_start);
    }
    for tag in &mut para.range_tags {
        tag.start = shift(tag.start);
        tag.end = shift(tag.end);
    }
    for mark in &mut para.markpen_marks {
        if let Some(pos) = &mut mark.utf16_pos {
            *pos = shift(*pos);
        }
    }

    para.field_ranges.remove(range_idx);
    para.controls.remove(control_idx);
    if control_idx < para.ctrl_data_records.len() {
        para.ctrl_data_records.remove(control_idx);
    }
    for range in &mut para.field_ranges {
        if range.control_idx > control_idx {
            range.control_idx -= 1;
        }
    }
    para.char_count = gaps[text_len].1.saturating_sub(16) + 1;
    para.invalidate_layout_inputs();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;

    fn para_with_text(text: &str) -> Paragraph {
        let mut para = Paragraph::new_empty();
        if !text.is_empty() {
            para.insert_text_at(0, text);
            para.has_para_text = true;
        }
        para
    }

    fn memo_field(id: u32, number: u32, body: &[&str]) -> Field {
        let mut field = new_memo_field(
            id,
            number,
            65535,
            "tester",
            &body.join("\n"),
            &para_with_text("anchor"),
        )
        .unwrap();
        field.properties |= 0x8001;
        field
    }

    #[test]
    fn sanitize_author_drops_command_separators() {
        assert_eq!(sanitize_author("  A/B\\C;D\nE  ").unwrap(), "ABCDE");
        assert_eq!(sanitize_author("").unwrap(), "");
        assert!(sanitize_author(&"가".repeat(65)).is_err());
        assert_eq!(
            sanitize_author(&"가".repeat(64))
                .unwrap()
                .encode_utf16()
                .count(),
            64
        );
    }

    #[test]
    fn command_roundtrips_number_and_author() {
        let command = memo_command(65535, 3, 77, "hancom");
        let tokens: Vec<&str> = command.split('/').collect();
        assert_eq!(&tokens[..3], ["MEMO", "65535", "3"]);
        assert_eq!(tokens[3], "77");
        assert!(!tokens[4].is_empty() && tokens[4].parse::<u32>().unwrap() != 0);
        assert_eq!(tokens[5], "hancom");
        assert_eq!(tokens[6], "\\;;");
        let field = Field {
            command,
            ..Default::default()
        };
        assert_eq!(memo_number(&field), 3);
        assert_eq!(memo_author(&field), "hancom");
    }

    #[test]
    fn memo_text_joins_paragraphs_with_newlines() {
        let field = memo_field(9, 1, &["첫 줄", "", "끝"]);
        assert_eq!(memo_text(&field), "첫 줄\n\n끝");
    }

    #[test]
    fn readonly_reason_tracks_formatting_and_objects() {
        let plain = vec![para_with_text("한 줄")];
        assert_eq!(memo_readonly_reason(&plain), None);

        let mut shaped = plain.clone();
        shaped[0].char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 1,
            },
            CharShapeRef {
                start_pos: 2,
                char_shape_id: 2,
            },
        ];
        assert_eq!(memo_readonly_reason(&shaped), Some("formatted"));

        let mut with_table = plain;
        with_table[0]
            .controls
            .push(Control::Table(Default::default()));
        assert_eq!(memo_readonly_reason(&with_table), Some("object"));
    }

    #[test]
    fn insert_and_remove_keep_anchor_text_and_slots_consistent() {
        let mut para = para_with_text("앞뒤");
        let field = memo_field(10, 1, &["메모 본문"]);
        insert_memo_field_in_para(&mut para, 1, 1, field).unwrap();
        assert_eq!(para.text, "앞뒤");
        assert_eq!(para.field_ranges.len(), 1);
        assert_eq!(para.field_ranges[0].start_char_idx, 1);
        assert_eq!(para.field_ranges[0].end_char_idx, 1);
        assert_eq!(para.controls.len(), 1);
        assert_eq!(para.char_count, 2 + 16 + 1);

        let range_idx = para.field_ranges[0].control_idx;
        let idx = memo_field_range_index(&para, range_idx).unwrap();
        remove_memo_field_in_para(&mut para, range_idx, idx).unwrap();
        assert_eq!(para.text, "앞뒤");
        assert!(para.field_ranges.is_empty());
        assert!(para.controls.is_empty());
        assert_eq!(para.char_count, 2 + 1);
    }

    #[test]
    fn insert_rejects_memo_overlap_but_allows_boundary_anchor() {
        let mut para = para_with_text("abcdef");
        insert_memo_field_in_para(&mut para, 1, 3, memo_field(1, 1, &["a"])).unwrap();
        assert!(matches!(
            insert_memo_field_in_para(&mut para, 2, 4, memo_field(2, 2, &["b"])),
            Err(MemoError {
                code: MemoErrorCode::Overlap,
                ..
            })
        ));
        // Zero-length anchor on the boundary is allowed.
        insert_memo_field_in_para(&mut para, 1, 1, memo_field(3, 3, &["c"])).unwrap();
        insert_memo_field_in_para(&mut para, 3, 3, memo_field(4, 4, &["d"])).unwrap();
        assert_eq!(para.field_ranges.len(), 3);
    }

    #[test]
    fn memo_body_splits_lines_and_keeps_trailing_empty_paragraph() {
        let template = para_with_text("t");
        let paragraphs = memo_paragraphs_from_text("one\n\ntwo\n", &template).unwrap();
        assert_eq!(paragraphs.len(), 4);
        assert_eq!(paragraphs[0].text, "one");
        assert_eq!(paragraphs[1].text, "");
        assert_eq!(paragraphs[2].text, "two");
        assert_eq!(paragraphs[3].text, "");
    }

    #[test]
    fn text_limit_counts_utf16_units() {
        assert!(check_text_size(&"가".repeat(MEMO_TEXT_MAX_UNITS)).is_ok());
        assert!(check_text_size(&"가".repeat(MEMO_TEXT_MAX_UNITS + 1)).is_err());
        // An emoji is two UTF-16 units.
        assert!(check_text_size(&"😀".repeat(MEMO_TEXT_MAX_UNITS / 2)).is_ok());
        assert!(check_text_size(&"😀".repeat(MEMO_TEXT_MAX_UNITS / 2 + 1)).is_err());
    }

    #[test]
    fn set_memo_text_marks_the_section_dirty() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "anchor").unwrap();
        core.insert_memo_at(0, 0, 0, 0, "tester", "body").unwrap();
        let id = core.document.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|ctrl| match ctrl {
                Control::Field(field) if is_memo_field(field) => Some(field.field_id),
                _ => None,
            })
            .unwrap();
        core.batch_mode = true;
        core.set_memo_text(id, "new body").unwrap();
        assert!(
            core.dirty_sections[0],
            "setMemoText must mark the section dirty"
        );
    }

    #[test]
    fn formatted_memo_is_read_only_but_removable() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "anchor").unwrap();
        core.insert_memo_at(0, 0, 0, 0, "tester", "body").unwrap();
        let (id, field) = core.document.sections[0].paragraphs[0]
            .controls
            .iter_mut()
            .find_map(|ctrl| match ctrl {
                Control::Field(field) if is_memo_field(field) => Some(field),
                _ => None,
            })
            .map(|field| {
                field.memo_paragraphs[0].char_shapes.push(CharShapeRef {
                    start_pos: 1,
                    char_shape_id: 999,
                });
                (field.field_id, ())
            })
            .unwrap();
        let _ = field;
        let error = core.set_memo_text(id, "new").unwrap_err();
        assert_eq!(error.code, MemoErrorCode::ReadOnly);
        core.remove_memo(id).unwrap();
        assert!(core.get_memo_list_json() == "[]");
    }

    #[test]
    fn deleting_the_whole_anchor_removes_the_memo_and_a_part_keeps_it() {
        let mut para = para_with_text("hello world");
        insert_memo_field_in_para(&mut para, 0, 5, memo_field(1, 1, &["body"])).unwrap();
        delete_text_at_keeping_memos(&mut para, 0, 5);
        assert_eq!(para.text, " world");
        assert!(
            para.controls
                .iter()
                .all(|ctrl| !matches!(ctrl, Control::Field(field) if is_memo_field(field))),
            "the memo went with its whole anchor"
        );

        let mut para = para_with_text("hello world");
        insert_memo_field_in_para(&mut para, 0, 5, memo_field(2, 2, &["body"])).unwrap();
        delete_text_at_keeping_memos(&mut para, 1, 2);
        assert_eq!(para.text, "hlo world");
        assert!(
            para.controls
                .iter()
                .any(|ctrl| matches!(ctrl, Control::Field(field) if is_memo_field(field))),
            "a shortened anchor keeps its memo"
        );
    }

    #[test]
    fn replacing_the_whole_anchor_with_text_keeps_the_memo() {
        let mut para = para_with_text("hello world");
        insert_memo_field_in_para(&mut para, 0, 5, memo_field(3, 3, &["body"])).unwrap();
        replace_text_at_keeping_memos(&mut para, 0, 5, "HELLO");
        assert_eq!(para.text, "HELLO world");
        assert!(
            para.controls
                .iter()
                .any(|ctrl| matches!(ctrl, Control::Field(field) if is_memo_field(field))),
            "text replacement keeps the memo and expands its range"
        );
    }

    #[test]
    fn native_replace_keeps_a_memo_until_its_whole_anchor_is_deleted() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "hello world").unwrap();
        core.insert_memo_at(0, 0, 0, 5, "tester", "body").unwrap();
        assert!(core.get_memo_list_json().contains("\"text\":\"body\""));

        core.replace_text_native(0, 0, 0, 5, "HELLO").unwrap();
        assert!(
            core.get_memo_list_json().contains("\"text\":\"body\""),
            "a replacement with text keeps the memo and expands its range"
        );

        core.replace_text_native(0, 0, 0, 5, "").unwrap();
        assert_eq!(core.get_memo_list_json(), "[]");
    }

    #[test]
    fn the_hwp5_sample_reads_its_memo_body() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue5866/memo_field_hwp5.hwp"
        );
        let bytes = std::fs::read(path).expect("read the Hancom memo sample");
        let doc = crate::parser::parse_document(&bytes).expect("parse the Hancom memo sample");
        let memos: Vec<&Field> = doc
            .sections
            .iter()
            .flat_map(|section| &section.paragraphs)
            .flat_map(|para| &para.controls)
            .filter_map(|control| match control {
                Control::Field(field) if is_memo_field(field) => Some(field),
                _ => None,
            })
            .collect();
        assert_eq!(memos.len(), 1, "one memo in the Hancom sample");
        assert!(
            !memos[0].memo_paragraphs.is_empty(),
            "the memo body paragraphs were assigned to the field"
        );
        assert!(
            memos[0]
                .memo_paragraphs
                .iter()
                .any(|para| !para.text.trim().is_empty()),
            "the memo body has readable text"
        );
    }

    #[test]
    fn a_pasted_hwpx_memo_is_written_with_its_new_number() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/samples/hwpx/aift.hwpx");
        let bytes = std::fs::read(path).expect("read the HWPX memo sample");
        let mut doc = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse the HWPX sample");
        let (section_idx, para_idx) = doc
            .sections
            .iter()
            .enumerate()
            .find_map(|(section_idx, section)| {
                section
                    .paragraphs
                    .iter()
                    .position(|para| {
                        para.controls.iter().any(
                            |ctrl| matches!(ctrl, Control::Field(field) if is_memo_field(field)),
                        )
                    })
                    .map(|para_idx| (section_idx, para_idx))
            })
            .expect("a memo paragraph");
        let mut copy = vec![doc.sections[section_idx].paragraphs[para_idx].clone()];
        let next = max_memo_number(&doc) + 1;
        renumber_memos(&mut copy, next);

        let field = copy[0]
            .controls
            .iter()
            .find_map(|ctrl| match ctrl {
                Control::Field(field) if is_memo_field(field) => Some(field),
                _ => None,
            })
            .unwrap();
        assert_eq!(memo_number(field), next);
        let number = |field: &Field| {
            field.parameters.items.iter().find_map(|item| match item {
                Parameter::Integer {
                    name: Some(key),
                    value,
                } if key == "Number" => Some(*value),
                _ => None,
            })
        };
        assert_eq!(number(field), Some(next as i64));
        assert_eq!(parameter_string(field, "ID"), Some(format!("memo{next}")));
        assert_eq!(
            parameter_string(field, "Command").as_deref(),
            Some(field.command.as_str())
        );

        // The HWPX writer must not fall back to the copied original parameters.
        doc.sections[section_idx].paragraphs.extend(copy);
        let out = crate::serializer::hwpx::serialize_hwpx(&doc).expect("write HWPX");
        let reparsed = crate::parser::hwpx::parse_hwpx(&out).expect("read the written HWPX");
        let numbers: Vec<Option<i64>> = reparsed.sections[section_idx]
            .paragraphs
            .iter()
            .flat_map(|para| &para.controls)
            .filter_map(|ctrl| match ctrl {
                Control::Field(field) if is_memo_field(field) => Some(number(field)),
                _ => None,
            })
            .collect();
        assert!(
            numbers.contains(&Some(next as i64)),
            "the pasted memo keeps its new number: {numbers:?}"
        );
    }

    #[test]
    fn memo_input_is_read_as_json() {
        let (author, text) = parse_memo_input(r#"{"author": "AA", "text": "x\/y\b\fé"}"#).unwrap();
        assert_eq!(author, "AA");
        assert_eq!(text, "x/y\u{8}\u{c}é");
        assert_eq!(
            parse_memo_input(r#"{"text":"only"}"#).unwrap(),
            (String::new(), "only".to_string())
        );
        assert!(parse_memo_input("[]").is_err());
        assert!(parse_memo_input(r#"{"text":1}"#).is_err());
        assert!(parse_memo_input(r#"{"author":null,"text":"x"}"#).is_err());
        assert!(parse_memo_input("not json").is_err());
    }
}
