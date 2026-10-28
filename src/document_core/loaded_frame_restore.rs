//! 로드 조판 상태가 실제로 복원됐을 때 역사적인 재조판 표식만 해제한다.

use super::{DocumentCore, TableTextReflowKey};
use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::paragraph::Paragraph;
use crate::model::raw_provenance::{
    bytes_digest, doc_info_model_digest, record_digest, section_model_digest,
};
use crate::model::shape::{Caption, ShapeObject};

struct LoadedSection {
    model: [u8; 32],
    persistence: [u8; 32],
    characters: usize,
    text_fingerprint: [u8; 32],
    // 후보를 줄이는 힌트다. 이 수치만으로 원본 복원을 판정하지 않는다.
    cell_text_delta: Option<i128>,
    text_delta: [u8; 32],
    source_stream: Option<(Vec<u8>, [u8; 32])>,
    restore_pending: bool,
}

pub(crate) struct LoadedFrames {
    doc_info: [u8; 32],
    sections: Vec<LoadedSection>,
}

#[derive(Default)]
struct PersistenceInputs<'a> {
    paragraphs: Vec<(bool, bool, Option<&'a [i32]>)>,
    characters: usize,
    text_fingerprint: [u8; 32],
}

fn mix_text_fingerprint(target: &mut [u8; 32], value: &[u8; 32]) {
    for (target, value) in target.iter_mut().zip(value) {
        *target ^= value;
    }
}

impl<'a> PersistenceInputs<'a> {
    fn visit_with_storage<S: crate::storage::Storage>(
        &mut self,
        paragraphs: &'a [Paragraph],
        storage: &S,
    ) -> Result<(), S::Error> {
        for paragraph in paragraphs {
            storage.push(
                &mut self.paragraphs,
                (
                    paragraph.stored_text_partition_dirty,
                    paragraph.cell_format_vpos_dirty,
                    paragraph.source_line_seg_vertical_pos.as_deref(),
                ),
            )?;
            self.characters += paragraph.text.chars().count();
            mix_text_fingerprint(
                &mut self.text_fingerprint,
                &bytes_digest(paragraph.text.as_bytes()),
            );
            for control in &paragraph.controls {
                match control {
                    Control::Table(table) => {
                        for cell in &table.cells {
                            self.visit_with_storage(&cell.paragraphs, storage)?;
                        }
                        self.caption_with_storage(table.caption.as_ref(), storage)?;
                    }
                    Control::Picture(picture) => {
                        self.caption_with_storage(picture.caption.as_ref(), storage)?
                    }
                    Control::Shape(shape) => self.shape_with_storage(shape, storage)?,
                    Control::Header(header) => {
                        self.visit_with_storage(&header.paragraphs, storage)?
                    }
                    Control::Footer(footer) => {
                        self.visit_with_storage(&footer.paragraphs, storage)?
                    }
                    Control::Footnote(note) => {
                        self.visit_with_storage(&note.paragraphs, storage)?
                    }
                    Control::Endnote(note) => self.visit_with_storage(&note.paragraphs, storage)?,
                    Control::HiddenComment(comment) => {
                        self.visit_with_storage(&comment.paragraphs, storage)?
                    }
                    Control::Field(field) => {
                        self.visit_with_storage(&field.memo_paragraphs, storage)?
                    }
                    Control::SectionDef(definition) => {
                        for master in &definition.master_pages {
                            self.visit_with_storage(&master.paragraphs, storage)?;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn caption_with_storage<S: crate::storage::Storage>(
        &mut self,
        caption: Option<&'a Caption>,
        storage: &S,
    ) -> Result<(), S::Error> {
        if let Some(caption) = caption {
            self.visit_with_storage(&caption.paragraphs, storage)?;
        }
        Ok(())
    }

    fn shape_with_storage<S: crate::storage::Storage>(
        &mut self,
        shape: &'a ShapeObject,
        storage: &S,
    ) -> Result<(), S::Error> {
        if let Some(drawing) = shape.drawing() {
            if let Some(textbox) = &drawing.text_box {
                self.visit_with_storage(&textbox.paragraphs, storage)?;
            }
            self.caption_with_storage(drawing.caption.as_ref(), storage)?;
        }
        match shape {
            ShapeObject::Picture(picture) => {
                self.caption_with_storage(picture.caption.as_ref(), storage)?
            }
            ShapeObject::Group(group) => {
                for child in &group.children {
                    self.shape_with_storage(child, storage)?;
                }
                self.caption_with_storage(group.caption.as_ref(), storage)?;
            }
            ShapeObject::Chart(chart) => {
                self.caption_with_storage(chart.caption.as_ref(), storage)?
            }
            ShapeObject::Ole(ole) => self.caption_with_storage(ole.caption.as_ref(), storage)?,
            _ => {}
        }
        Ok(())
    }

    fn section(section: &'a Section) -> Self {
        match Self::section_with_storage(section, &crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    fn section_with_storage<S: crate::storage::Storage>(
        section: &'a Section,
        storage: &S,
    ) -> Result<Self, S::Error> {
        let mut inputs = Self::default();
        inputs.visit_with_storage(&section.paragraphs, storage)?;
        for master in &section.section_def.master_pages {
            inputs.visit_with_storage(&master.paragraphs, storage)?;
        }
        Ok(inputs)
    }
}

impl LoadedFrames {
    fn capture(document: &Document) -> Self {
        match Self::capture_with_storage(document, &crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    fn capture_with_storage<S: crate::storage::Storage>(
        document: &Document,
        storage: &S,
    ) -> Result<Self, S::Error> {
        Ok(Self {
            doc_info: doc_info_model_digest(&document.doc_info, &document.doc_properties),
            sections: storage.try_collect(document.sections.iter().map(|section| {
                let inputs = PersistenceInputs::section_with_storage(section, storage)?;
                Ok(LoadedSection {
                    model: section_model_digest(section),
                    persistence: record_digest(&inputs.paragraphs),
                    characters: inputs.characters,
                    text_fingerprint: inputs.text_fingerprint,
                    cell_text_delta: Some(0),
                    text_delta: [0; 32],
                    source_stream: None,
                    restore_pending: false,
                })
            }))?,
        })
    }

    fn note_cell_edit(
        &mut self,
        section: usize,
        old_chars: usize,
        new_chars: usize,
        old_text: &[u8; 32],
        new_text: &[u8; 32],
    ) -> bool {
        let Some(source) = self.sections.get_mut(section) else {
            return false;
        };
        source.cell_text_delta = source
            .cell_text_delta
            .and_then(|delta| delta.checked_add(new_chars as i128 - old_chars as i128));
        source.restore_pending = false;
        mix_text_fingerprint(&mut source.text_delta, old_text);
        mix_text_fingerprint(&mut source.text_delta, new_text);
        // 같은 길이의 IME 교체도 편집 문단의 지문만 갱신한다. 이 XOR는 순서·
        // 소유를 입증하지 않으며, 일치한 후보의 전체 모델 digest로 최종 판정한다.
        new_chars <= old_chars && source.cell_text_delta == Some(0) && source.text_delta == [0; 32]
    }

    fn matches(&self, document: &Document, section: usize) -> bool {
        let Some(source) = self.sections.get(section) else {
            return false;
        };
        let Some(current) = document.sections.get(section) else {
            return false;
        };
        source.model == section_model_digest(current)
            && self.doc_info == doc_info_model_digest(&document.doc_info, &document.doc_properties)
            && source.persistence == record_digest(&PersistenceInputs::section(current).paragraphs)
    }
}

impl DocumentCore {
    /// 로드 픽스업·첫 페이지 구성 또는 명시적인 문서 교체를 완료한 지점만 호출한다.
    pub(crate) fn capture_loaded_source_frames(&mut self) {
        match self.capture_loaded_source_frames_with_storage(&crate::storage::Unbounded) {
            Ok(()) => (),
            Err(never) => match never {},
        }
    }
    pub(crate) fn capture_loaded_source_frames_with_storage<S: crate::storage::Storage>(
        &mut self,
        storage: &S,
    ) -> Result<(), S::Error> {
        self.render_normalization.loaded_frames =
            Some(LoadedFrames::capture_with_storage(&self.document, storage)?);
        Ok(())
    }

    /// 스냅샷 등 전체 파생 상태 재구성은 저비용 후보 힌트도 현재 원본에 동기화한다.
    pub(crate) fn sync_loaded_frame_text_balance(&mut self) {
        let Some(frames) = &mut self.render_normalization.loaded_frames else {
            return;
        };
        for (source, current) in frames.sections.iter_mut().zip(&self.document.sections) {
            let inputs = PersistenceInputs::section(current);
            source.cell_text_delta = Some(inputs.characters as i128 - source.characters as i128);
            source.text_delta = inputs.text_fingerprint;
            mix_text_fingerprint(&mut source.text_delta, &source.text_fingerprint);
            source.restore_pending = false;
        }
    }

    pub(crate) fn restore_loaded_frames_after_cell_text_edit(
        &mut self,
        section: usize,
        old_chars: usize,
        new_chars: usize,
        old_text: [u8; 32],
        new_text: [u8; 32],
    ) -> bool {
        let Some(frames) = &mut self.render_normalization.loaded_frames else {
            return false;
        };
        let candidate = frames.note_cell_edit(section, old_chars, new_chars, &old_text, &new_text);
        let saved_stream = frames
            .sections
            .get(section)
            .is_some_and(|source| source.source_stream.is_some());
        if !candidate
            || !(saved_stream
                || self.document.sections[section].raw_stream.is_some()
                || self
                    .render_normalization
                    .text_reflowed_tables
                    .has_cell_text())
        {
            return false;
        }
        if !frames.matches(&self.document, section) {
            return false;
        }
        frames.sections[section].restore_pending = true;
        // 구역 전체가 원래 모델·스타일·저장 캐시 계약으로 돌아왔다. 다른 구역과
        // 아직 실제 변경이 남은 모델은 이 판정으로 해제하지 않는다.
        let mut changed = false;
        for paragraph in &self.document.sections[section].paragraphs {
            for control in &paragraph.controls {
                if let Control::Table(table) = control {
                    changed |= self
                        .render_normalization
                        .text_reflowed_tables
                        .restore_cell_text(&TableTextReflowKey::from_table(table));
                }
            }
        }
        // raw 캐시 복원은 렌더 경로의 변경이다. 줄 프레임·유닛 점유가 그대로인
        // 편집까지 cellFlowChanged로 승격해 owner 전체를 재구성하지 않는다.
        changed
    }

    /// 기존 무효화 지점에서 원본 raw 캐시를 복사하지 않고 이동해 보존한다.
    /// 모델·DocInfo·저장 guard 복원을 입증한 같은 명령만 원래 봉인과 대조해 되돌린다.
    pub(crate) fn finish_loaded_source_cache_after_cell_edit(
        &mut self,
        section: usize,
        render_reset: bool,
    ) -> bool {
        let current = &mut self.document.sections[section];
        let had_source_cache = current.raw_stream.is_some();
        if let Some(source) = self
            .render_normalization
            .loaded_frames
            .as_mut()
            .and_then(|frames| frames.sections.get_mut(section))
        {
            if source.restore_pending {
                source.restore_pending = false;
                let valid_current = current.raw_stream.as_ref().is_some_and(|raw| {
                    current.raw_provenance.as_ref().is_some_and(|seal| {
                        seal.model_digest == source.model && bytes_digest(raw) == seal.raw_digest
                    })
                });
                if valid_current {
                    source.source_stream = None;
                } else {
                    current.raw_stream = None;
                    let valid_saved = source.source_stream.as_ref().is_some_and(|(_, digest)| {
                        current.raw_provenance.as_ref().is_some_and(|seal| {
                            seal.model_digest == source.model && seal.raw_digest == *digest
                        })
                    });
                    if valid_saved {
                        current.raw_stream = source.source_stream.take().map(|(bytes, _)| bytes);
                    }
                }
            } else if let Some(raw) = current.raw_stream.take() {
                if source.source_stream.is_none() {
                    if let Some(seal) = &current.raw_provenance {
                        if seal.model_digest == source.model
                            && bytes_digest(&raw) == seal.raw_digest
                        {
                            source.source_stream = Some((raw, seal.raw_digest));
                        }
                    }
                }
            }
        } else {
            current.raw_stream = None;
        }
        let source_cache_restored = !had_source_cache && current.raw_stream.is_some();
        if render_reset {
            self.layout_engine.clear_layout_caches();
            self.recompose_section(section);
        }
        source_cache_restored
    }
}
