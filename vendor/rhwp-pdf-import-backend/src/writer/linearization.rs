//! PDF Linearization (Fast Web View) support.
//!
//! This module provides functionality to linearize PDF files for efficient
//! network access, allowing the first page to be displayed quickly while
//! the rest of the document downloads.
//!
//! ## Overview
//!
//! Linearized PDFs reorganize objects so that:
//! - The first page can be displayed immediately
//! - Subsequent pages can be accessed efficiently via hint tables
//! - Objects are ordered for optimal sequential/random access
//!
//! ## PDF Structure
//!
//! A linearized PDF has 11 parts in this order:
//! 1. Header
//! 2. Linearization parameter dictionary
//! 3. First-page cross-reference table and trailer
//! 4. Document catalog and document-level objects
//! 5. Primary hint stream
//! 6. First-page section (objects for first page)
//! 7. Remaining pages
//! 8. Shared objects
//! 9. Other objects
//! 10. Overflow hint stream (optional)
//! 11. Main cross-reference table and trailer
//!
//! ## Standards Reference
//!
//! - PDF Reference 1.7: Annex F "Linearized PDF"
//! - ISO 32000-1:2008: Annex F

use crate::object::{Object, ObjectRef};
use std::collections::{HashMap, HashSet};

/// Linearization configuration options.
#[derive(Debug, Clone)]
pub struct LinearizationConfig {
    /// First page to display (0-indexed).
    pub first_page: usize,
    /// Whether to include hint tables.
    pub include_hints: bool,
}

impl Default for LinearizationConfig {
    fn default() -> Self {
        Self {
            first_page: 0,
            include_hints: true,
        }
    }
}

/// Linearization parameter dictionary entries.
///
/// Per PDF spec Table F.1.
#[derive(Debug, Clone)]
pub struct LinearizationParams {
    /// Version identification for linearized format.
    pub version: f32,
    /// Total length of the file in bytes.
    pub file_length: u64,
    /// Offset and length of primary hint stream [offset, length].
    pub hint_stream: [u64; 2],
    /// Object number of first page's page object.
    pub first_page_object: u32,
    /// Offset of end of first page.
    pub end_of_first_page: u64,
    /// Number of pages in document.
    pub num_pages: u32,
    /// Offset of first entry of main cross-reference table.
    pub main_xref_offset: u64,
    /// Page number of first page (0-indexed).
    pub first_page_num: u32,
}

impl LinearizationParams {
    /// Create a new linearization parameters structure.
    pub fn new(num_pages: u32) -> Self {
        Self {
            version: 1.0,
            file_length: 0,
            hint_stream: [0, 0],
            first_page_object: 0,
            end_of_first_page: 0,
            num_pages,
            main_xref_offset: 0,
            first_page_num: 0,
        }
    }

    /// Build the linearization parameter dictionary as a PDF Object.
    pub fn to_object(&self) -> Object {
        let mut dict = HashMap::new();

        dict.insert("Linearized".to_string(), Object::Real(self.version as f64));
        dict.insert("L".to_string(), Object::Integer(self.file_length as i64));
        dict.insert(
            "H".to_string(),
            Object::Array(vec![
                Object::Integer(self.hint_stream[0] as i64),
                Object::Integer(self.hint_stream[1] as i64),
            ]),
        );
        dict.insert("O".to_string(), Object::Integer(self.first_page_object as i64));
        dict.insert("E".to_string(), Object::Integer(self.end_of_first_page as i64));
        dict.insert("N".to_string(), Object::Integer(self.num_pages as i64));
        dict.insert("T".to_string(), Object::Integer(self.main_xref_offset as i64));

        if self.first_page_num != 0 {
            dict.insert("P".to_string(), Object::Integer(self.first_page_num as i64));
        }

        Object::Dictionary(dict)
    }
}

/// Page offset hint table entry per PDF spec Table F.4.
#[derive(Debug, Clone, Default)]
pub struct PageOffsetEntry {
    /// Number of objects in the page (delta from minimum).
    pub num_objects_delta: u32,
    /// Page length in bytes (delta from minimum).
    pub page_length_delta: u32,
    /// Number of shared objects referenced from this page.
    pub num_shared_objects: u32,
    /// Shared object identifiers.
    pub shared_object_ids: Vec<u32>,
    /// Numerators of fractional positions for shared objects.
    pub shared_object_numerators: Vec<u32>,
    /// Offset to content stream start (delta from minimum).
    pub content_stream_offset_delta: u32,
    /// Content stream length (delta from minimum).
    pub content_stream_length_delta: u32,
}

/// Page offset hint table header per PDF spec Table F.3.
#[derive(Debug, Clone, Default)]
pub struct PageOffsetHeader {
    /// Minimum object number in first page.
    pub min_object_num: u32,
    /// Location of first page's page object.
    pub first_page_location: u64,
    /// Bits needed for page length delta.
    pub bits_page_length: u8,
    /// Minimum page length.
    pub min_page_length: u32,
    /// Bits needed for object count delta.
    pub bits_object_count: u8,
    /// Minimum object count per page.
    pub min_object_count: u32,
    /// Bits for content stream offset delta.
    pub bits_content_offset: u8,
    /// Minimum content stream offset.
    pub min_content_offset: u32,
    /// Bits for content stream length delta.
    pub bits_content_length: u8,
    /// Minimum content stream length.
    pub min_content_length: u32,
    /// Bits for shared object identifier.
    pub bits_shared_object_id: u8,
    /// Bits for numerator of fractional position.
    pub bits_shared_numerator: u8,
    /// Denominator of fractional position.
    pub shared_denominator: u32,
}

/// Shared object hint table entry per PDF spec Table F.6.
#[derive(Debug, Clone, Default)]
pub struct SharedObjectEntry {
    /// Object length in bytes (delta from minimum).
    pub object_length_delta: u32,
    /// Whether the object is referenced by the first page.
    pub in_first_page: bool,
    /// Object number delta (for objects in object streams).
    pub object_num_delta: u32,
    /// Number of objects in the group.
    pub num_objects: u32,
}

/// Shared object hint table header per PDF spec Table F.5.
#[derive(Debug, Clone, Default)]
pub struct SharedObjectHeader {
    /// Object number of first shared object.
    pub first_object_num: u32,
    /// Location of first shared object.
    pub first_object_location: u64,
    /// Number of shared object entries for first page.
    pub num_first_page_entries: u32,
    /// Number of shared object entries for remaining pages.
    pub num_remaining_entries: u32,
    /// Bits needed for object length delta.
    pub bits_object_length: u8,
    /// Minimum object length.
    pub min_object_length: u32,
    /// Bits for object number delta.
    pub bits_object_num: u8,
}

/// Combined hint tables for linearization.
#[derive(Debug, Clone, Default)]
pub struct HintTables {
    /// Page offset hint table header.
    pub page_offset_header: PageOffsetHeader,
    /// Page offset entries (one per page).
    pub page_offset_entries: Vec<PageOffsetEntry>,
    /// Shared object hint table header.
    pub shared_object_header: SharedObjectHeader,
    /// Shared object entries.
    pub shared_object_entries: Vec<SharedObjectEntry>,
}

impl HintTables {
    /// Create new empty hint tables.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize hint tables to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Write page offset hint table
        self.write_page_offset_table(&mut data);

        // Write shared object hint table
        self.write_shared_object_table(&mut data);

        data
    }

    fn write_page_offset_table(&self, data: &mut Vec<u8>) {
        let header = &self.page_offset_header;

        // Write header fields (fixed sizes per spec)
        Self::write_u32(data, header.min_object_num);
        Self::write_u64(data, header.first_page_location);
        Self::write_u16(data, header.bits_page_length as u16);
        Self::write_u32(data, header.min_page_length);
        Self::write_u16(data, header.bits_object_count as u16);
        Self::write_u32(data, header.min_object_count);
        Self::write_u16(data, header.bits_content_offset as u16);
        Self::write_u32(data, header.min_content_offset);
        Self::write_u16(data, header.bits_content_length as u16);
        Self::write_u32(data, header.min_content_length);
        Self::write_u16(data, header.bits_shared_object_id as u16);
        Self::write_u16(data, header.bits_shared_numerator as u16);
        Self::write_u16(data, header.shared_denominator as u16);

        // Write per-page entries (variable bit-packed)
        let mut bit_writer = BitWriter::new();

        for entry in &self.page_offset_entries {
            bit_writer.write_bits(entry.num_objects_delta as u64, header.bits_object_count);
            bit_writer.write_bits(entry.page_length_delta as u64, header.bits_page_length);
            bit_writer.write_bits(entry.num_shared_objects as u64, header.bits_shared_object_id);

            for &id in &entry.shared_object_ids {
                bit_writer.write_bits(id as u64, header.bits_shared_object_id);
            }

            for &num in &entry.shared_object_numerators {
                bit_writer.write_bits(num as u64, header.bits_shared_numerator);
            }

            bit_writer
                .write_bits(entry.content_stream_offset_delta as u64, header.bits_content_offset);
            bit_writer
                .write_bits(entry.content_stream_length_delta as u64, header.bits_content_length);
        }

        data.extend(bit_writer.finish());
    }

    fn write_shared_object_table(&self, data: &mut Vec<u8>) {
        let header = &self.shared_object_header;

        // Write header fields
        Self::write_u32(data, header.first_object_num);
        Self::write_u64(data, header.first_object_location);
        Self::write_u32(data, header.num_first_page_entries);
        Self::write_u32(data, header.num_remaining_entries);
        Self::write_u16(data, header.bits_object_length as u16);
        Self::write_u32(data, header.min_object_length);
        Self::write_u16(data, header.bits_object_num as u16);

        // Write entries (bit-packed)
        let mut bit_writer = BitWriter::new();

        for entry in &self.shared_object_entries {
            bit_writer.write_bits(entry.object_length_delta as u64, header.bits_object_length);
            bit_writer.write_bits(if entry.in_first_page { 1 } else { 0 }, 1);
            bit_writer.write_bits(entry.object_num_delta as u64, header.bits_object_num);
            bit_writer.write_bits(entry.num_objects as u64, header.bits_object_num);
        }

        data.extend(bit_writer.finish());
    }

    fn write_u16(data: &mut Vec<u8>, value: u16) {
        data.extend(&value.to_be_bytes());
    }

    fn write_u32(data: &mut Vec<u8>, value: u32) {
        data.extend(&value.to_be_bytes());
    }

    fn write_u64(data: &mut Vec<u8>, value: u64) {
        data.extend(&value.to_be_bytes());
    }
}

/// Bit writer for encoding hint table entries.
struct BitWriter {
    buffer: Vec<u8>,
    current_byte: u8,
    bit_position: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_byte: 0,
            bit_position: 0,
        }
    }

    fn write_bits(&mut self, value: u64, num_bits: u8) {
        if num_bits == 0 {
            return;
        }

        for i in (0..num_bits).rev() {
            let bit = ((value >> i) & 1) as u8;
            self.current_byte = (self.current_byte << 1) | bit;
            self.bit_position += 1;

            if self.bit_position == 8 {
                self.buffer.push(self.current_byte);
                self.current_byte = 0;
                self.bit_position = 0;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.bit_position > 0 {
            // Pad remaining bits with zeros
            self.current_byte <<= 8 - self.bit_position;
            self.buffer.push(self.current_byte);
        }
        self.buffer
    }
}

/// Information about an object for linearization analysis.
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    /// Object reference (number and generation).
    pub obj_ref: ObjectRef,
    /// Byte offset in the file.
    pub offset: u64,
    /// Length in bytes.
    pub length: u64,
    /// Pages that reference this object (empty for first page objects).
    pub referenced_by_pages: HashSet<usize>,
    /// Whether this is a content stream.
    pub is_content_stream: bool,
    /// Whether this is a page object.
    pub is_page_object: bool,
}

/// Linearization analyzer to determine object ordering.
#[derive(Debug)]
pub struct LinearizationAnalyzer {
    /// All objects in the document.
    objects: Vec<ObjectInfo>,
    /// Number of pages.
    #[allow(dead_code)]
    num_pages: usize,
    /// First page index.
    first_page: usize,
    /// Objects belonging to first page.
    first_page_objects: HashSet<u32>,
    /// Shared objects (referenced by multiple pages).
    shared_objects: HashSet<u32>,
    /// Objects for each page (indexed by page number).
    page_objects: Vec<HashSet<u32>>,
}

impl LinearizationAnalyzer {
    /// Create a new analyzer.
    pub fn new(num_pages: usize, first_page: usize) -> Self {
        Self {
            objects: Vec::new(),
            num_pages,
            first_page,
            first_page_objects: HashSet::new(),
            shared_objects: HashSet::new(),
            page_objects: vec![HashSet::new(); num_pages],
        }
    }

    /// Add an object for analysis.
    pub fn add_object(&mut self, info: ObjectInfo) {
        self.objects.push(info);
    }

    /// Analyze objects to determine categorization.
    pub fn analyze(&mut self) {
        // First pass: count references
        let mut reference_counts: HashMap<u32, usize> = HashMap::new();

        for obj in &self.objects {
            for &page in &obj.referenced_by_pages {
                *reference_counts.entry(obj.obj_ref.id).or_default() += 1;

                if page < self.page_objects.len() {
                    self.page_objects[page].insert(obj.obj_ref.id);
                }
            }
        }

        // Categorize objects
        for obj in &self.objects {
            let id = obj.obj_ref.id;

            if obj.referenced_by_pages.contains(&self.first_page) {
                self.first_page_objects.insert(id);
            }

            let ref_count = reference_counts.get(&id).copied().unwrap_or(0);
            if ref_count > 1 {
                self.shared_objects.insert(id);
            }
        }
    }

    /// Get the ordered list of objects for the first page.
    pub fn get_first_page_objects(&self) -> Vec<u32> {
        let mut objects: Vec<_> = self.first_page_objects.iter().copied().collect();
        objects.sort();
        objects
    }

    /// Get shared objects.
    pub fn get_shared_objects(&self) -> Vec<u32> {
        let mut objects: Vec<_> = self.shared_objects.iter().copied().collect();
        objects.sort();
        objects
    }

    /// Get objects for a specific page (excluding first page and shared).
    pub fn get_page_specific_objects(&self, page: usize) -> Vec<u32> {
        if page >= self.page_objects.len() || page == self.first_page {
            return Vec::new();
        }

        let mut objects: Vec<_> = self.page_objects[page]
            .iter()
            .filter(|id| !self.first_page_objects.contains(id) && !self.shared_objects.contains(id))
            .copied()
            .collect();
        objects.sort();
        objects
    }
}

/// Builder for creating linearized PDFs.
pub struct LinearizedPdfBuilder {
    #[allow(dead_code)]
    config: LinearizationConfig,
    params: LinearizationParams,
    hint_tables: HintTables,
}

impl LinearizedPdfBuilder {
    /// Create a new linearized PDF builder.
    pub fn new(num_pages: u32, config: LinearizationConfig) -> Self {
        let params = LinearizationParams::new(num_pages);

        Self {
            config,
            params,
            hint_tables: HintTables::new(),
        }
    }

    /// Set the first page object number.
    pub fn set_first_page_object(&mut self, obj_num: u32) {
        self.params.first_page_object = obj_num;
    }

    /// Set the first page number (for alternate first page display).
    pub fn set_first_page_num(&mut self, page_num: u32) {
        self.params.first_page_num = page_num;
    }

    /// Update file length after writing.
    pub fn set_file_length(&mut self, length: u64) {
        self.params.file_length = length;
    }

    /// Set hint stream offset and length.
    pub fn set_hint_stream_info(&mut self, offset: u64, length: u64) {
        self.params.hint_stream = [offset, length];
    }

    /// Set end of first page offset.
    pub fn set_end_of_first_page(&mut self, offset: u64) {
        self.params.end_of_first_page = offset;
    }

    /// Set main cross-reference table offset.
    pub fn set_main_xref_offset(&mut self, offset: u64) {
        self.params.main_xref_offset = offset;
    }

    /// Get the linearization parameters.
    pub fn params(&self) -> &LinearizationParams {
        &self.params
    }

    /// Get mutable access to hint tables.
    pub fn hint_tables_mut(&mut self) -> &mut HintTables {
        &mut self.hint_tables
    }

    /// Build the linearization parameter dictionary object.
    pub fn build_params_object(&self) -> Object {
        self.params.to_object()
    }

    /// Build the hint stream data.
    pub fn build_hint_stream(&self) -> Vec<u8> {
        self.hint_tables.to_bytes()
    }
}

/// Calculate the number of bits needed to represent a value.
pub fn bits_needed(value: u32) -> u8 {
    if value == 0 {
        return 0;
    }
    32 - value.leading_zeros() as u8
}

/// Calculate minimum and bits needed for a set of values.
pub fn calculate_delta_encoding(values: &[u32]) -> (u32, u8) {
    if values.is_empty() {
        return (0, 0);
    }

    let min = *values.iter().min().unwrap_or(&0);
    let max_delta = values
        .iter()
        .map(|&v| v.saturating_sub(min))
        .max()
        .unwrap_or(0);

    (min, bits_needed(max_delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linearization_params() {
        let params = LinearizationParams::new(5);
        assert_eq!(params.num_pages, 5);
        assert_eq!(params.version, 1.0);
    }

    #[test]
    fn test_linearization_params_to_object() {
        let mut params = LinearizationParams::new(10);
        params.file_length = 50000;
        params.hint_stream = [1024, 512];
        params.first_page_object = 4;
        params.end_of_first_page = 5000;
        params.main_xref_offset = 45000;

        let obj = params.to_object();

        if let Object::Dictionary(dict) = obj {
            assert!(dict.contains_key("Linearized"));
            assert!(dict.contains_key("L"));
            assert!(dict.contains_key("H"));
            assert!(dict.contains_key("O"));
            assert!(dict.contains_key("E"));
            assert!(dict.contains_key("N"));
            assert!(dict.contains_key("T"));
            // P should not be present when first_page_num is 0
            assert!(!dict.contains_key("P"));
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_bits_needed() {
        assert_eq!(bits_needed(0), 0);
        assert_eq!(bits_needed(1), 1);
        assert_eq!(bits_needed(2), 2);
        assert_eq!(bits_needed(3), 2);
        assert_eq!(bits_needed(4), 3);
        assert_eq!(bits_needed(255), 8);
        assert_eq!(bits_needed(256), 9);
    }

    #[test]
    fn test_delta_encoding() {
        let values = vec![10, 15, 20, 25];
        let (min, bits) = calculate_delta_encoding(&values);
        assert_eq!(min, 10);
        assert_eq!(bits, 4); // max delta is 15, needs 4 bits
    }

    #[test]
    fn test_bit_writer() {
        let mut writer = BitWriter::new();
        writer.write_bits(0b101, 3);
        writer.write_bits(0b1100, 4);
        writer.write_bits(0b1, 1);

        let data = writer.finish();
        assert_eq!(data, vec![0b10111001]);
    }

    #[test]
    fn test_linearization_analyzer() {
        let mut analyzer = LinearizationAnalyzer::new(3, 0);

        // Add some test objects
        let mut first_page_refs = HashSet::new();
        first_page_refs.insert(0);

        analyzer.add_object(ObjectInfo {
            obj_ref: ObjectRef::new(1, 0),
            offset: 100,
            length: 50,
            referenced_by_pages: first_page_refs.clone(),
            is_content_stream: false,
            is_page_object: true,
        });

        let mut shared_refs = HashSet::new();
        shared_refs.insert(0);
        shared_refs.insert(1);

        analyzer.add_object(ObjectInfo {
            obj_ref: ObjectRef::new(2, 0),
            offset: 200,
            length: 100,
            referenced_by_pages: shared_refs,
            is_content_stream: false,
            is_page_object: false,
        });

        analyzer.analyze();

        assert!(analyzer.first_page_objects.contains(&1));
        assert!(analyzer.shared_objects.contains(&2));
    }

    #[test]
    fn test_linearized_builder() {
        let config = LinearizationConfig::default();
        let mut builder = LinearizedPdfBuilder::new(5, config);

        builder.set_first_page_object(4);
        builder.set_file_length(50000);
        builder.set_hint_stream_info(1024, 512);
        builder.set_end_of_first_page(5000);
        builder.set_main_xref_offset(45000);

        let params = builder.params();
        assert_eq!(params.first_page_object, 4);
        assert_eq!(params.file_length, 50000);
    }

    #[test]
    fn test_hint_tables_serialization() {
        let mut tables = HintTables::new();

        tables.page_offset_header.min_object_num = 1;
        tables.page_offset_header.bits_page_length = 8;
        tables.page_offset_header.min_page_length = 1000;

        let bytes = tables.to_bytes();
        assert!(!bytes.is_empty());
    }
}
