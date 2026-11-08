//! Embedded file support for PDF documents.
//!
//! This module provides functionality to embed files in PDF documents,
//! making them available as attachments that can be extracted by PDF readers.
//!
//! ## Overview
//!
//! Embedded files in PDF are stored in the /Names dictionary of the catalog,
//! under the /EmbeddedFiles key. Each embedded file has:
//! - A file specification (Filespec) dictionary
//! - An embedded file stream with the actual file data
//! - Optional parameters like creation date, modification date, size, checksum
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::writer::embedded_files::EmbeddedFile;
//!
//! let file = EmbeddedFile::new("data.csv", csv_bytes)
//!     .with_description("Monthly sales data")
//!     .with_mime_type("text/csv");
//! ```

use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Represents an embedded file to be included in a PDF.
#[derive(Debug, Clone)]
pub struct EmbeddedFile {
    /// The file name (used as the key in the EmbeddedFiles name tree)
    pub name: String,
    /// The file data
    pub data: Vec<u8>,
    /// Optional description of the file
    pub description: Option<String>,
    /// MIME type of the file (e.g., "application/pdf", "text/plain")
    pub mime_type: Option<String>,
    /// Creation date (ISO 8601 format)
    pub creation_date: Option<String>,
    /// Modification date (ISO 8601 format)
    pub modification_date: Option<String>,
    /// Associated File Relationship (AFRelationship)
    /// Per PDF 2.0 spec: Source, Data, Alternative, Supplement, etc.
    pub af_relationship: Option<AFRelationship>,
}

/// Associated File Relationship per PDF 2.0 spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AFRelationship {
    /// The file is the original source
    Source,
    /// The file contains data referenced by the document
    Data,
    /// An alternative representation
    Alternative,
    /// Supplementary data
    Supplement,
    /// Encrypted payload (for protected content)
    EncryptedPayload,
    /// A form data file
    FormData,
    /// A schema definition
    Schema,
    /// Unspecified relationship
    Unspecified,
}

impl AFRelationship {
    /// Get the PDF name for this relationship.
    pub fn pdf_name(&self) -> &'static str {
        match self {
            AFRelationship::Source => "Source",
            AFRelationship::Data => "Data",
            AFRelationship::Alternative => "Alternative",
            AFRelationship::Supplement => "Supplement",
            AFRelationship::EncryptedPayload => "EncryptedPayload",
            AFRelationship::FormData => "FormData",
            AFRelationship::Schema => "Schema",
            AFRelationship::Unspecified => "Unspecified",
        }
    }
}

impl EmbeddedFile {
    /// Create a new embedded file.
    ///
    /// # Arguments
    ///
    /// * `name` - The file name (used as identifier and display name)
    /// * `data` - The file contents
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            description: None,
            mime_type: None,
            creation_date: None,
            modification_date: None,
            af_relationship: None,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the MIME type.
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set the creation date (ISO 8601 format).
    pub fn with_creation_date(mut self, date: impl Into<String>) -> Self {
        self.creation_date = Some(date.into());
        self
    }

    /// Set the modification date (ISO 8601 format).
    pub fn with_modification_date(mut self, date: impl Into<String>) -> Self {
        self.modification_date = Some(date.into());
        self
    }

    /// Set the AF relationship (PDF 2.0).
    pub fn with_af_relationship(mut self, relationship: AFRelationship) -> Self {
        self.af_relationship = Some(relationship);
        self
    }

    /// Get the size of the embedded file data.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Build the embedded file stream dictionary.
    ///
    /// Returns the dictionary entries for the EmbeddedFile stream.
    pub fn build_stream_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("EmbeddedFile".to_string()));

        // Subtype is the MIME type if provided
        if let Some(ref mime) = self.mime_type {
            dict.insert("Subtype".to_string(), Object::Name(mime.replace('/', "#2F")));
        }

        // Build Params dictionary
        let mut params = HashMap::new();
        params.insert("Size".to_string(), Object::Integer(self.data.len() as i64));

        if let Some(ref creation) = self.creation_date {
            params.insert(
                "CreationDate".to_string(),
                Object::String(
                    format!("D:{}", creation.replace(['-', ':', 'T', 'Z'], "")).into_bytes(),
                ),
            );
        }

        if let Some(ref modification) = self.modification_date {
            params.insert(
                "ModDate".to_string(),
                Object::String(
                    format!("D:{}", modification.replace(['-', ':', 'T', 'Z'], "")).into_bytes(),
                ),
            );
        }

        // CheckSum: MD5 per PDF spec (ISO 32000-1 §7.11.4). When legacy-crypto
        // is disabled we omit the optional field rather than fail.
        #[cfg(feature = "legacy-crypto")]
        params.insert("CheckSum".to_string(), Object::String(md5_hash(&self.data)));

        dict.insert("Params".to_string(), Object::Dictionary(params));

        dict
    }

    /// Build the file specification dictionary.
    ///
    /// Returns the Filespec dictionary that references the embedded file stream.
    pub fn build_filespec(&self, embedded_stream_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("Filespec".to_string()));

        // File name (F is the generic name, UF is Unicode name)
        dict.insert("F".to_string(), Object::text_string(&self.name));
        dict.insert("UF".to_string(), Object::String(encode_utf16_be(&self.name)));

        // Description
        if let Some(ref desc) = self.description {
            dict.insert("Desc".to_string(), Object::text_string(desc));
        }

        // EF dictionary with the embedded file stream reference
        let mut ef_dict = HashMap::new();
        ef_dict.insert("F".to_string(), Object::Reference(embedded_stream_ref));
        ef_dict.insert("UF".to_string(), Object::Reference(embedded_stream_ref));
        dict.insert("EF".to_string(), Object::Dictionary(ef_dict));

        // AF relationship (PDF 2.0)
        if let Some(relationship) = self.af_relationship {
            dict.insert(
                "AFRelationship".to_string(),
                Object::Name(relationship.pdf_name().to_string()),
            );
        }

        dict
    }
}

/// Builder for creating the EmbeddedFiles name tree.
#[derive(Debug, Default)]
pub struct EmbeddedFilesBuilder {
    files: Vec<EmbeddedFile>,
}

impl EmbeddedFilesBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Add an embedded file.
    pub fn add_file(&mut self, file: EmbeddedFile) -> &mut Self {
        self.files.push(file);
        self
    }

    /// Check if there are any files to embed.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get the number of files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Get all embedded files.
    pub fn files(&self) -> &[EmbeddedFile] {
        &self.files
    }

    /// Take ownership of the files.
    pub fn into_files(self) -> Vec<EmbeddedFile> {
        self.files
    }

    /// Build the Names array for the EmbeddedFiles name tree.
    ///
    /// The Names array alternates between file names (as strings) and
    /// file specification references.
    ///
    /// Returns: Vec of (name, filespec_ref) pairs sorted by name.
    pub fn build_names_array(&self, filespec_refs: &[(String, ObjectRef)]) -> Object {
        let mut names_array = Vec::new();

        // Sort by name for proper name tree ordering
        let mut sorted_refs: Vec<_> = filespec_refs.iter().collect();
        sorted_refs.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, ref_) in sorted_refs {
            names_array.push(Object::text_string(name));
            names_array.push(Object::Reference(*ref_));
        }

        Object::Array(names_array)
    }

    /// Build the EmbeddedFiles dictionary for the Names tree.
    pub fn build_embedded_files_dict(
        &self,
        filespec_refs: &[(String, ObjectRef)],
    ) -> HashMap<String, Object> {
        let mut dict = HashMap::new();
        dict.insert("Names".to_string(), self.build_names_array(filespec_refs));
        dict
    }
}

/// MD5 hash function for embedded-file checksum (PDF spec §7.11.4).
///
/// #230 Phase C: intentionally a *direct* MD5 — the `/CheckSum` is a
/// non-security integrity tag fixed by ISO 32000-1 §7.11.4, not a
/// key-derivation, so it is NOT routed through
/// `encryption::md5_kdf_hasher` (gating it would make a strict policy
/// refuse to embed a file even in an AES-256 document).
#[cfg(feature = "legacy-crypto")]
fn md5_hash(data: &[u8]) -> Vec<u8> {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Encode a string as UTF-16BE with BOM for PDF Unicode strings.
fn encode_utf16_be(s: &str) -> Vec<u8> {
    let mut result = vec![0xFE, 0xFF]; // UTF-16BE BOM
    for c in s.encode_utf16() {
        result.push((c >> 8) as u8);
        result.push((c & 0xFF) as u8);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_file_new() {
        let file = EmbeddedFile::new("test.txt", b"Hello, World!".to_vec());
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size(), 13);
        assert!(file.description.is_none());
        assert!(file.mime_type.is_none());
    }

    #[test]
    fn test_embedded_file_builder() {
        let file = EmbeddedFile::new("data.csv", b"a,b,c".to_vec())
            .with_description("Test CSV file")
            .with_mime_type("text/csv")
            .with_creation_date("2024-01-15T10:30:00Z")
            .with_af_relationship(AFRelationship::Data);

        assert_eq!(file.description, Some("Test CSV file".to_string()));
        assert_eq!(file.mime_type, Some("text/csv".to_string()));
        assert_eq!(file.af_relationship, Some(AFRelationship::Data));
    }

    #[test]
    fn test_af_relationship_pdf_name() {
        assert_eq!(AFRelationship::Source.pdf_name(), "Source");
        assert_eq!(AFRelationship::Data.pdf_name(), "Data");
        assert_eq!(AFRelationship::Alternative.pdf_name(), "Alternative");
    }

    #[test]
    fn test_embedded_files_builder() {
        let mut builder = EmbeddedFilesBuilder::new();
        assert!(builder.is_empty());

        builder.add_file(EmbeddedFile::new("file1.txt", b"content1".to_vec()));
        builder.add_file(EmbeddedFile::new("file2.txt", b"content2".to_vec()));

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_build_stream_dict() {
        let file = EmbeddedFile::new("test.txt", b"Hello".to_vec()).with_mime_type("text/plain");

        let dict = file.build_stream_dict();

        assert!(dict.contains_key("Type"));
        assert!(dict.contains_key("Subtype"));
        assert!(dict.contains_key("Params"));

        if let Some(Object::Dictionary(params)) = dict.get("Params") {
            assert!(params.contains_key("Size"));
            #[cfg(feature = "legacy-crypto")]
            assert!(params.contains_key("CheckSum"));
        } else {
            panic!("Params should be a dictionary");
        }
    }

    #[test]
    fn test_encode_utf16_be() {
        let result = encode_utf16_be("test");
        assert_eq!(&result[0..2], &[0xFE, 0xFF]); // BOM
        assert!(result.len() > 2);
    }

    #[cfg(feature = "legacy-crypto")]
    #[test]
    fn test_md5_hash() {
        let hash = md5_hash(b"test data");
        assert_eq!(hash.len(), 16);

        // Same input should produce same output
        let hash2 = md5_hash(b"test data");
        assert_eq!(hash, hash2);

        // Different input should produce different output
        let hash3 = md5_hash(b"different data");
        assert_ne!(hash, hash3);
    }
}
