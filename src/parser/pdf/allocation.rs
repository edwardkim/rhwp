//! Request-owned frontend storage. Counts requested payload and replacement
//! buffers; allocator bookkeeping and process-wide live heap remain host limits.
use super::PdfImportError;
use pdf_oxide::import_limits::ImportBudget;
use serde::Serialize;
use std::io::{self, Write};

#[derive(Clone, Copy)]
pub(crate) struct Allocation<'a>(pub &'a ImportBudget);

pub(super) fn message(error: PdfImportError) -> String {
    error.to_string()
}

#[derive(Debug)]
pub(super) struct Failure(PdfImportError);
impl From<PdfImportError> for Failure {
    fn from(value: PdfImportError) -> Self {
        Self(value)
    }
}
impl From<Failure> for PdfImportError {
    fn from(value: Failure) -> Self {
        value.0
    }
}
impl From<Failure> for String {
    fn from(value: Failure) -> Self {
        value.0.to_string()
    }
}

#[derive(Debug)]
struct RequestReservation(std::sync::Arc<ImportBudget>);

#[derive(Debug)]
struct PayloadReservation(std::sync::Arc<ImportBudget>);
impl crate::storage::Reservation for PayloadReservation {
    fn hash_payload(&self, element_bytes: usize, capacity: usize) -> Result<(), String> {
        let bytes = element_bytes.checked_mul(capacity).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("transport backing size overflow")).to_string()
        })?;
        self.0
            .charge_decoded(bytes)
            .map_err(|error| PdfImportError::backend(error).to_string())
    }
    fn reject(&self, reason: &'static str) -> String {
        PdfImportError::backend(self.0.reject(reason)).to_string()
    }
}

pub(crate) fn payload_reservation(
    request: &std::sync::Arc<ImportBudget>,
) -> Result<std::sync::Arc<dyn crate::storage::Reservation>, PdfImportError> {
    Allocation(request)
        .bytes(std::mem::size_of::<PayloadReservation>() + 2 * std::mem::size_of::<usize>())?;
    Ok(std::sync::Arc::new(PayloadReservation(
        std::sync::Arc::clone(request),
    )))
}
impl crate::storage::Reservation for RequestReservation {
    fn hash_payload(&self, element_bytes: usize, capacity: usize) -> Result<(), String> {
        Allocation(&self.0)
            .reserve_hash_payload(element_bytes, capacity)
            .map_err(|error| error.to_string())
    }
    fn reject(&self, reason: &'static str) -> String {
        PdfImportError::backend(self.0.reject(reason)).to_string()
    }
}

pub(crate) fn shared_reservation(
    request: &std::sync::Arc<ImportBudget>,
) -> Result<std::sync::Arc<dyn crate::storage::Reservation>, PdfImportError> {
    Allocation(request)
        .bytes(std::mem::size_of::<RequestReservation>() + 2 * std::mem::size_of::<usize>())?;
    Ok(std::sync::Arc::new(RequestReservation(
        std::sync::Arc::clone(request),
    )))
}

impl Allocation<'_> {
    fn reserve_hash_payload(
        self,
        element_bytes: usize,
        capacity: usize,
    ) -> Result<(), PdfImportError> {
        let payload = element_bytes.checked_mul(capacity).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("frontend hash payload size overflow"))
        })?;
        self.bytes(
            payload
                .saturating_add(capacity.saturating_mul(2 * std::mem::size_of::<usize>()))
                .saturating_mul(2),
        )
    }
    fn binary<P: crate::serializer::byte_writer::BinaryProducer>(
        self,
        producer: &P,
    ) -> Result<Vec<u8>, PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        struct Count {
            length: usize,
            remaining: usize,
        }
        impl Write for Count {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.length = self
                    .length
                    .checked_add(bytes.len())
                    .filter(|length| *length <= self.remaining)
                    .ok_or_else(|| io::Error::other("binary payload bytes"))?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut count = Count {
            length: 0,
            remaining: self
                .0
                .limits
                .decoded_bytes
                .saturating_sub(self.0.snapshot().decoded_bytes)
                .min(usize::MAX as u64) as usize,
        };
        producer
            .write(&mut crate::serializer::byte_writer::ByteWriter::with_writer(&mut count))
            .map_err(|_| PdfImportError::backend(self.0.reject("frontend binary payload bytes")))?;
        let mut bytes = self.vec(count.length)?;
        bytes.resize(count.length, 0);
        let mut writer = crate::serializer::byte_writer::ByteWriter::with_writer(io::Cursor::new(
            bytes.as_mut_slice(),
        ));
        producer.write(&mut writer).map_err(|_| {
            PdfImportError::backend(self.0.reject("frontend binary payload encoding"))
        })?;
        if writer.position() != count.length {
            return Err(PdfImportError::backend(
                self.0.reject("frontend binary payload length changed"),
            ));
        }
        self.0.check().map_err(PdfImportError::backend)?;
        Ok(bytes)
    }

    pub fn bytes(self, count: usize) -> Result<(), PdfImportError> {
        self.0
            .charge_decoded(count)
            .map_err(PdfImportError::backend)
    }

    fn size<T>(self, count: usize) -> Result<usize, PdfImportError> {
        count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| PdfImportError::backend(self.0.reject("frontend storage size overflow")))
    }

    pub fn payload<T>(self, count: usize) -> Result<(), PdfImportError> {
        self.bytes(self.size::<T>(count)?)
    }

    pub fn vec<T>(self, count: usize) -> Result<Vec<T>, PdfImportError> {
        self.bytes(self.size::<T>(count)?)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| PdfImportError::backend(self.0.reject("frontend vector allocation")))?;
        Ok(result)
    }

    pub fn copy_slice<T: Copy>(self, input: &[T]) -> Result<Vec<T>, PdfImportError> {
        let mut result = self.vec(input.len())?;
        result.extend_from_slice(input);
        Ok(result)
    }

    pub fn filled<T: Copy>(self, count: usize, value: T) -> Result<Vec<T>, PdfImportError> {
        let mut result = self.vec(count)?;
        result.resize(count, value);
        Ok(result)
    }

    pub fn set_from<T: Ord>(
        self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<std::collections::BTreeSet<T>, PdfImportError> {
        let mut result = std::collections::BTreeSet::new();
        self.extend_set(&mut result, values)?;
        Ok(result)
    }

    pub fn extend_set<T: Ord>(
        self,
        target: &mut std::collections::BTreeSet<T>,
        values: impl IntoIterator<Item = T>,
    ) -> Result<(), PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        for value in values {
            self.insert_set(target, value)?;
        }
        Ok(())
    }

    pub fn collect<T>(self, input: impl IntoIterator<Item = T>) -> Result<Vec<T>, PdfImportError> {
        let input = input.into_iter();
        let mut result = self.vec(input.size_hint().0)?;
        for value in input {
            self.push(&mut result, value)?;
        }
        Ok(result)
    }

    pub(super) fn try_collect<T, E: From<Failure>>(
        self,
        input: impl IntoIterator<Item = Result<T, E>>,
    ) -> Result<Vec<T>, E> {
        let input = input.into_iter();
        let mut result = self.vec(input.size_hint().0).map_err(Failure::from)?;
        for value in input {
            self.push(&mut result, value?).map_err(Failure::from)?;
        }
        Ok(result)
    }

    pub fn push<T>(self, target: &mut Vec<T>, value: T) -> Result<(), PdfImportError> {
        self.reserve(target, 1)?;
        target.push(value);
        Ok(())
    }

    pub fn sort_by<T>(
        self,
        values: &mut [T],
        compare: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> Result<(), PdfImportError> {
        // Preserve stable ties. Reserve a full-length scratch payload before
        // the standard library can allocate its sorting workspace.
        self.bytes(self.size::<T>(values.len())?)?;
        values.sort_by(compare);
        Ok(())
    }

    pub fn string_capacity(self, count: usize) -> Result<String, PdfImportError> {
        self.bytes(count)?;
        let mut result = String::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| PdfImportError::backend(self.0.reject("frontend string allocation")))?;
        Ok(result)
    }

    pub fn string(self, value: &str) -> Result<String, PdfImportError> {
        let mut result = self.string_capacity(value.len())?;
        result.push_str(value);
        Ok(result)
    }

    pub fn reserve_string(
        self,
        value: &mut String,
        additional: usize,
    ) -> Result<(), PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        let required = value.len().checked_add(additional).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("frontend string size overflow"))
        })?;
        if required > value.capacity() {
            let capacity = required.max(value.capacity().saturating_mul(2)).max(8);
            self.bytes(capacity)?;
            value
                .try_reserve_exact(capacity - value.len())
                .map_err(|_| PdfImportError::backend(self.0.reject("frontend string growth")))?;
        }
        Ok(())
    }

    pub fn push_str(self, target: &mut String, value: &str) -> Result<(), PdfImportError> {
        self.reserve_string(target, value.len())?;
        target.push_str(value);
        Ok(())
    }

    pub fn push_char(self, target: &mut String, value: char) -> Result<(), PdfImportError> {
        self.reserve_string(target, value.len_utf8())?;
        target.push(value);
        Ok(())
    }

    pub fn strings(self, values: &[String]) -> Result<Vec<String>, PdfImportError> {
        let mut result = self.vec(values.len())?;
        for value in values {
            result.push(self.string(value)?);
        }
        Ok(result)
    }

    /// Copy audited model records only: their custom serializers must not
    /// allocate before visiting fields, and skipped owned payload is separate.
    fn reserve_doc_info(
        self,
        value: &crate::model::document::DocInfo,
    ) -> Result<(), PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        // Exhaustive field pattern: newly owned fields must be audited here.
        let crate::model::document::DocInfo {
            bin_data_list,
            font_faces,
            border_fills,
            char_shapes,
            tab_defs,
            numberings,
            bullets,
            para_shapes,
            styles,
            extra_records,
            print_method: _,
            raw_stream,
            bullet_count: _,
            memo_shape_count: _,
            memo_properties_xml,
            distribute_doc_data_removed: _,
            raw_stream_dirty: _,
            hwpx_head_tail,
            hwpml_version,
            raw_provenance,
        } = value;
        let payload = (
            bin_data_list,
            font_faces,
            border_fills,
            char_shapes,
            tab_defs,
            numberings,
            bullets,
            para_shapes,
            styles,
            extra_records,
            raw_stream,
            memo_properties_xml,
            hwpx_head_tail,
            hwpml_version,
        );
        let work = crate::model::owned_work::measure(
            &payload,
            self.0.limits.decoded_bytes.min(usize::MAX as u64) as usize,
            0,
        )
        .map_err(|_| PdfImportError::backend(self.0.reject("DocInfo copy work")))?;
        self.bytes(work)?;
        // Provenance is a separate owned digest graph, not a serialized field.
        if let Some(seal) = raw_provenance {
            let crate::model::raw_provenance::DocInfoRecordSeals {
                props: _,
                bin_data,
                fonts,
                border_fills,
                char_shapes,
                tab_defs,
                numberings,
                bullets,
                para_shapes,
                styles,
            } = &seal.record_seals;
            self.payload::<Vec<[u8; 32]>>(fonts.len())?;
            for group in fonts {
                self.payload::<[u8; 32]>(group.len())?;
            }
            for group in [
                bin_data,
                border_fills,
                char_shapes,
                tab_defs,
                numberings,
                bullets,
                para_shapes,
                styles,
            ] {
                self.payload::<[u8; 32]>(group.len())?;
            }
        }
        Ok(())
    }

    pub fn clone_doc_info(
        self,
        value: &crate::model::document::DocInfo,
    ) -> Result<crate::model::document::DocInfo, PdfImportError> {
        self.reserve_doc_info(value)?;
        Ok(value.clone())
    }

    fn record_work<T: ?Sized + Serialize>(self, value: &T) -> Result<(), PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        let work = crate::model::owned_work::measure(
            value,
            self.0.limits.decoded_bytes.min(usize::MAX as u64) as usize,
            0,
        )
        .map_err(|_| PdfImportError::backend(self.0.reject("document copy work")))?;
        self.bytes(work)
    }

    fn reserve_section(
        self,
        section: &crate::model::document::Section,
    ) -> Result<(), PdfImportError> {
        let crate::model::document::Section {
            section_def,
            paragraphs,
            trailing_lists,
            raw_stream,
            raw_provenance: _,
        } = section;
        self.reserve_paragraph_runtime_fields(paragraphs)?;
        for master in &section_def.master_pages {
            self.reserve_paragraph_runtime_fields(&master.paragraphs)?;
        }
        self.record_work(&(section_def, paragraphs, trailing_lists, raw_stream))?;
        Ok(())
    }

    pub fn clone_section(
        self,
        section: &crate::model::document::Section,
    ) -> Result<crate::model::document::Section, PdfImportError> {
        self.reserve_section(section)?;
        Ok(section.clone())
    }

    pub fn clone_document(
        self,
        value: &crate::model::document::Document,
    ) -> Result<crate::model::document::Document, PdfImportError> {
        use crate::model::{bin_data::BinDataBytes, document::Document};
        let Document {
            header,
            doc_properties,
            doc_info,
            sections,
            preview,
            bin_data_content,
            extra_streams,
            hwpx_aux_entries,
            is_hwp3_variant: _,
            is_hwpx_variant: _,
            provenance: _,
        } = value;
        self.reserve_doc_info(doc_info)?;
        self.record_work(doc_properties)?;
        self.record_work(&header.raw_data)?;
        self.payload::<crate::model::document::Section>(sections.len())?;
        for section in sections {
            self.reserve_section(section)?;
        }
        if let Some(preview) = preview {
            self.record_work(&preview.text)?;
            if let Some(image) = &preview.image {
                self.payload::<u8>(image.data.len())?;
            }
        }
        self.payload::<crate::model::bin_data::BinDataContent>(bin_data_content.len())?;
        for entry in bin_data_content {
            self.payload::<u8>(entry.extension.len())?;
            match &entry.data {
                BinDataBytes::Loaded(bytes) => self.payload::<u8>(bytes.len())?,
                // Cloning the resolver retains its Arc, without decoding data.
                BinDataBytes::Lazy { key, .. } => self.payload::<u8>(key.len())?,
            }
        }
        self.record_work(&(extra_streams, hwpx_aux_entries))?;
        self.0.check().map_err(PdfImportError::backend)?;
        Ok(value.clone())
    }

    fn reserve_paragraph_runtime_fields(
        self,
        paragraphs: &[crate::model::paragraph::Paragraph],
    ) -> Result<(), PdfImportError> {
        use crate::model::identity::walk::{walk_ref, RefNode};
        walk_ref(paragraphs, &self, |node| {
            if let RefNode::Paragraph(para) = node {
                self.payload::<usize>(para.literal_figure_spaces.len())?;
                self.payload::<(u32, crate::model::paragraph::SpaceMetric)>(
                    para.layout_space_metrics.len(),
                )?;
                self.payload::<(u32, i16)>(para.hwp3_saved_space_corrections.len())?;
                self.record_work(&(
                    &para.layout_field_values,
                    &para.source_line_seg_vertical_pos,
                ))?;
                if let crate::model::paragraph::NumberingMarker::Text(text, _) =
                    &para.numbering_marker
                {
                    self.payload::<u8>(text.len())?;
                }
            }
            if let RefNode::Control(crate::model::control::Control::Form(form)) = node {
                // The custom serializer sorts borrowed entries before invoking
                // its visitor; reserve that scratch before the payload walk.
                self.payload::<(&String, &String)>(form.properties.len().saturating_mul(2))?;
                if !form.properties.is_empty() {
                    self.payload::<(String, String)>(form.properties.capacity().saturating_mul(2))?;
                }
            }
            Ok(())
        })
    }

    pub fn clone_record<T: Clone + Serialize>(self, value: &T) -> Result<T, PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        let work = crate::model::owned_work::measure(
            value,
            self.0.limits.decoded_bytes.min(usize::MAX as u64) as usize,
            0,
        )
        .map_err(|_| PdfImportError::backend(self.0.reject("frontend owned record work")))?;
        self.bytes(work)?;
        Ok(value.clone())
    }

    pub fn id(self, prefix: &str, indices: &[usize]) -> Result<String, PdfImportError> {
        use std::fmt::Write;
        let count = indices
            .iter()
            .try_fold(prefix.len(), |size, n| {
                size.checked_add(1 + n.checked_ilog10().map_or(1, |p| p as usize + 1))
            })
            .ok_or_else(|| PdfImportError::backend(self.0.reject("frontend ID size overflow")))?;
        let mut result = self.string_capacity(count)?;
        result.push_str(prefix);
        for index in indices {
            write!(&mut result, ":{index}")
                .map_err(|_| PdfImportError::budget("frontend ID format failed"))?;
        }
        Ok(result)
    }

    pub fn reserve<T>(self, value: &mut Vec<T>, additional: usize) -> Result<(), PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        let required = value.len().checked_add(additional).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("frontend vector size overflow"))
        })?;
        if required > value.capacity() {
            let capacity = required.max(value.capacity().saturating_mul(2)).max(4);
            // Reallocation can retain the old buffer until the copy completes.
            self.bytes(self.size::<T>(capacity)?)?;
            value
                .try_reserve_exact(capacity - value.len())
                .map_err(|_| PdfImportError::backend(self.0.reject("frontend vector growth")))?;
        }
        Ok(())
    }

    /// Logical BTree entry payload plus links. This is a request work
    /// reservation, not a claim about the standard library's node allocator.
    pub fn tree_entry<K, V>(self) -> Result<(), PdfImportError> {
        crate::storage::Storage::tree_entry::<K, V>(&self)
    }

    pub fn insert<K: Ord, V>(
        self,
        map: &mut std::collections::BTreeMap<K, V>,
        key: K,
        value: V,
    ) -> Result<Option<V>, PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        if !map.contains_key(&key) {
            self.tree_entry::<K, V>()?;
        }
        Ok(map.insert(key, value))
    }

    pub fn insert_set<T: Ord>(
        self,
        set: &mut std::collections::BTreeSet<T>,
        value: T,
    ) -> Result<bool, PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        if !set.contains(&value) {
            self.tree_entry::<T, ()>()?;
        }
        Ok(set.insert(value))
    }

    pub fn boxed<T>(self, value: T) -> Result<Box<T>, PdfImportError> {
        self.bytes(std::mem::size_of::<T>())?;
        Ok(Box::new(value))
    }

    pub fn object(self) -> Result<serde_json::Map<String, serde_json::Value>, PdfImportError> {
        self.bytes(1024)?;
        Ok(serde_json::Map::new())
    }

    pub fn object_entry(
        self,
        object: &mut serde_json::Map<String, serde_json::Value>,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), PdfImportError> {
        if !object.contains_key(key) {
            self.bytes(super::value_allocation::map_entry_work())?;
        }
        let key = self.string(key)?;
        object.insert(key, value);
        Ok(())
    }

    pub fn set(
        self,
        object: &mut serde_json::Value,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), PdfImportError> {
        let object = object
            .as_object_mut()
            .ok_or_else(|| PdfImportError::budget("frontend JSON assignment requires an object"))?;
        self.object_entry(object, key, value)
    }

    /// Count without an output buffer, then allocate the exact encoded length.
    /// Widen f32 as the former intermediate serde_json::Value did, preserving
    /// diagnostic numeric values without allocating that intermediate tree.
    /// Only audited serializers may be used; hidden serializer scratch is separate.
    pub fn json_exact<T: Serialize>(
        self,
        value: &T,
        maximum: usize,
    ) -> Result<Vec<u8>, PdfImportError> {
        self.0.check().map_err(PdfImportError::backend)?;
        struct ValueCompatibleFormatter;
        impl serde_json::ser::Formatter for ValueCompatibleFormatter {
            fn write_f32<W: ?Sized + Write>(
                &mut self,
                writer: &mut W,
                value: f32,
            ) -> io::Result<()> {
                serde_json::ser::Formatter::write_f64(
                    &mut serde_json::ser::CompactFormatter,
                    writer,
                    f64::from(value),
                )
            }
        }
        struct Count {
            length: usize,
            maximum: usize,
            exceeded: bool,
        }
        impl Write for Count {
            fn write(&mut self, input: &[u8]) -> io::Result<usize> {
                let length = self
                    .length
                    .checked_add(input.len())
                    .filter(|length| *length <= self.maximum);
                let Some(length) = length else {
                    self.exceeded = true;
                    return Err(io::Error::other("JSON output bytes"));
                };
                self.length = length;
                Ok(input.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut count = Count {
            length: 0,
            maximum,
            exceeded: false,
        };
        let measured = value.serialize(&mut serde_json::Serializer::with_formatter(
            &mut count,
            ValueCompatibleFormatter,
        ));
        if count.exceeded {
            return Err(PdfImportError::backend(
                self.0.reject("frontend JSON output bytes"),
            ));
        }
        measured.map_err(|_| PdfImportError::budget("frontend JSON serialization failed"))?;
        let bytes = self.vec(count.length)?;
        let mut writer = JsonWriter {
            allocation: self,
            bytes,
            maximum,
        };
        let result = value.serialize(&mut serde_json::Serializer::with_formatter(
            &mut writer,
            ValueCompatibleFormatter,
        ));
        self.0.check().map_err(PdfImportError::backend)?;
        result.map_err(|_| PdfImportError::budget("frontend JSON serialization failed"))?;
        Ok(writer.bytes)
    }

    pub fn json<T: Serialize>(self, value: &T, maximum: usize) -> Result<Vec<u8>, PdfImportError> {
        let mut writer = JsonWriter {
            allocation: self,
            bytes: Vec::new(),
            maximum,
        };
        let result = serde_json::to_writer(&mut writer, value);
        self.0.check().map_err(PdfImportError::backend)?;
        result.map_err(|_| PdfImportError::budget("frontend JSON serialization failed"))?;
        Ok(writer.bytes)
    }

    pub fn value<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> Result<serde_json::Value, PdfImportError> {
        super::value_allocation::to_value(self, value)
    }

    pub fn display<T: ?Sized + std::fmt::Display>(
        self,
        value: &T,
    ) -> Result<String, PdfImportError> {
        let mut writer = JsonWriter {
            allocation: self,
            bytes: Vec::new(),
            maximum: self.0.limits.decoded_bytes.min(usize::MAX as u64) as usize,
        };
        let result = writer.write_fmt(format_args!("{value}"));
        self.0.check().map_err(PdfImportError::backend)?;
        result.map_err(|_| PdfImportError::budget("frontend display formatting failed"))?;
        String::from_utf8(writer.bytes)
            .map_err(|_| PdfImportError::budget("frontend display UTF-8 failed"))
    }
}

struct JsonWriter<'a> {
    allocation: Allocation<'a>,
    bytes: Vec<u8>,
    maximum: usize,
}

impl crate::renderer::style_resolver::storage::Storage for Allocation<'_> {
    type Error = PdfImportError;
    fn binary<P: crate::serializer::byte_writer::BinaryProducer>(
        &self,
        producer: &P,
    ) -> Result<Vec<u8>, Self::Error> {
        Allocation::binary(*self, producer)
    }
    fn bin_data(
        &self,
        bytes: &crate::model::bin_data::BinDataBytes,
        maximum: usize,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        self.0.check().map_err(PdfImportError::backend)?;
        use crate::model::bin_data::BinDataBytes;
        match bytes {
            BinDataBytes::Loaded(data) if data.len() <= maximum => Ok(Some(self.copy_slice(data)?)),
            BinDataBytes::Loaded(_) => Ok(None),
            BinDataBytes::Lazy { resolver, key } => {
                let admitted = resolver
                    .stable_size_bound(key)
                    .unwrap_or(maximum)
                    .min(maximum);
                self.payload::<u8>(admitted)?;
                Ok(resolver
                    .resolve_limited(key, admitted)
                    .filter(|data| data.len() <= admitted))
            }
        }
    }

    fn display<T: ?Sized + std::fmt::Display>(&self, value: &T) -> Result<String, Self::Error> {
        Allocation::display(*self, value)
    }

    fn clone_document(
        &self,
        value: &crate::model::document::Document,
    ) -> Result<crate::model::document::Document, Self::Error> {
        Allocation::clone_document(*self, value)
    }

    fn clone_section_def(
        &self,
        value: &crate::model::document::SectionDef,
    ) -> Result<crate::model::document::SectionDef, Self::Error> {
        for master in &value.master_pages {
            self.reserve_paragraph_runtime_fields(&master.paragraphs)?;
        }
        self.clone_record(value)
    }
    fn reserve_set<T: Eq + std::hash::Hash>(
        &self,
        values: &mut std::collections::HashSet<T>,
        additional: usize,
    ) -> Result<(), Self::Error> {
        self.0.check().map_err(PdfImportError::backend)?;
        let required = values.len().checked_add(additional).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("frontend hash set size overflow"))
        })?;
        if required > values.capacity() {
            let capacity = required.max(values.capacity().saturating_mul(2)).max(8);
            self.reserve_hash_payload(std::mem::size_of::<T>(), capacity)?;
            values.try_reserve(capacity - values.len()).map_err(|_| {
                PdfImportError::backend(self.0.reject("frontend hash set allocation"))
            })?;
        }
        Ok(())
    }
    fn clone_paragraph(
        &self,
        value: &crate::model::paragraph::Paragraph,
    ) -> Result<crate::model::paragraph::Paragraph, Self::Error> {
        self.reserve_paragraph_runtime_fields(std::slice::from_ref(value))?;
        self.clone_record(value)
    }
    fn reserve_string(&self, value: &mut String, additional: usize) -> Result<(), Self::Error> {
        Allocation::reserve_string(*self, value, additional)
    }
    fn clone_doc_info(
        &self,
        value: &crate::model::document::DocInfo,
    ) -> Result<crate::model::document::DocInfo, Self::Error> {
        Allocation::clone_doc_info(*self, value)
    }

    fn clone_cell(
        &self,
        value: &crate::model::table::Cell,
    ) -> Result<crate::model::table::Cell, Self::Error> {
        self.reserve_paragraph_runtime_fields(&value.paragraphs)?;
        self.clone_record(value)
    }
    fn clone_table(
        &self,
        value: &crate::model::table::Table,
    ) -> Result<crate::model::table::Table, Self::Error> {
        for cell in &value.cells {
            self.reserve_paragraph_runtime_fields(&cell.paragraphs)?;
        }
        if let Some(caption) = &value.caption {
            self.reserve_paragraph_runtime_fields(&caption.paragraphs)?;
        }
        self.record_work(value)?;
        Ok(value.clone())
    }
    fn clone_record<T: Clone + Serialize>(&self, value: &T) -> Result<T, Self::Error> {
        Allocation::clone_record(*self, value)
    }

    fn bytes(&self, count: usize) -> Result<(), Self::Error> {
        Allocation::bytes(*self, count)
    }
    fn chars<I: Iterator<Item = char> + Clone>(&self, chars: I) -> Result<String, Self::Error> {
        let count = chars.clone().try_fold(0usize, |n, c| {
            n.checked_add(c.len_utf8())
                .ok_or_else(|| PdfImportError::backend(self.0.reject("renderer UTF-8 size")))
        })?;
        let mut text = self.string_capacity(count)?;
        for scalar in chars {
            text.push(scalar);
        }
        Ok(text)
    }

    fn vec<T>(&self, count: usize) -> Result<Vec<T>, Self::Error> {
        Allocation::vec(*self, count)
    }
    fn string(&self, text: &str) -> Result<String, Self::Error> {
        Allocation::string(*self, text)
    }
    fn reserve_map<K: Eq + std::hash::Hash, V>(
        &self,
        values: &mut std::collections::HashMap<K, V>,
        additional: usize,
    ) -> Result<(), Self::Error> {
        self.0.check().map_err(PdfImportError::backend)?;
        let required = values.len().checked_add(additional).ok_or_else(|| {
            PdfImportError::backend(self.0.reject("frontend hash map size overflow"))
        })?;
        if required > values.capacity() {
            let capacity = required.max(values.capacity().saturating_mul(2)).max(8);
            // Reserve replacement payload and conservative bucket/control work
            // before HashMap may allocate. Allocator/RSS limits remain separate.
            self.reserve_hash_payload(std::mem::size_of::<(K, V)>(), capacity)?;
            values.try_reserve(capacity - values.len()).map_err(|_| {
                PdfImportError::backend(self.0.reject("frontend hash map allocation"))
            })?;
        }
        Ok(())
    }
    fn reserve<T>(&self, values: &mut Vec<T>, additional: usize) -> Result<(), Self::Error> {
        Allocation::reserve(*self, values, additional)
    }
    fn join(&self, values: &[String], separator: &str) -> Result<String, Self::Error> {
        let size = values
            .iter()
            .try_fold(0usize, |size, text| size.checked_add(text.len()))
            .and_then(|size| {
                separator
                    .len()
                    .checked_mul(values.len().saturating_sub(1))
                    .and_then(|extra| size.checked_add(extra))
            })
            .ok_or_else(|| {
                PdfImportError::backend(self.0.reject("style family join size overflow"))
            })?;
        let mut result = self.string_capacity(size)?;
        for (index, text) in values.iter().enumerate() {
            if index > 0 {
                result.push_str(separator);
            }
            result.push_str(text);
        }
        Ok(result)
    }
}
impl Write for JsonWriter<'_> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(input.len())
            .is_none_or(|n| n > self.maximum)
        {
            self.allocation.0.reject("frontend JSON output bytes");
            return Err(io::Error::other("frontend JSON output bytes"));
        }
        self.allocation
            .reserve(&mut self.bytes, input.len())
            .map_err(io::Error::other)?;
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
