//! Main document editing interface.
//!
//! Provides the DocumentEditor type for modifying PDF documents.

use crate::document::PdfDocument;
use crate::editor::form_fields::FormFieldWrapper;
use crate::editor::resource_manager::ResourceManager;
use crate::elements::{ContentElement, StructureElement};
use crate::error::{Error, Result};
use crate::extractors::HierarchicalExtractor;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use crate::writer::{hoist_appearance_streams, ContentStreamBuilder, ObjectSerializer};
use std::collections::{HashMap, HashSet};
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::BufWriter;
use std::io::{Read, Seek, Write};
use std::path::Path;

/// Document metadata (Info dictionary).
#[derive(Debug, Clone, Default)]
pub struct DocumentInfo {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document keywords (comma-separated)
    pub keywords: Option<String>,
    /// Creator application
    pub creator: Option<String>,
    /// PDF producer
    pub producer: Option<String>,
    /// Creation date (PDF date format)
    pub creation_date: Option<String>,
    /// Modification date (PDF date format)
    pub mod_date: Option<String>,
}

impl DocumentInfo {
    /// Create a new empty DocumentInfo.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the keywords.
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    /// Set the creator.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// Set the producer.
    pub fn producer(mut self, producer: impl Into<String>) -> Self {
        self.producer = Some(producer.into());
        self
    }

    /// Convert to a PDF Info dictionary object.
    pub fn to_object(&self) -> Object {
        let mut dict = HashMap::new();

        if let Some(ref title) = self.title {
            dict.insert("Title".to_string(), Object::String(title.as_bytes().to_vec()));
        }
        if let Some(ref author) = self.author {
            dict.insert("Author".to_string(), Object::String(author.as_bytes().to_vec()));
        }
        if let Some(ref subject) = self.subject {
            dict.insert("Subject".to_string(), Object::String(subject.as_bytes().to_vec()));
        }
        if let Some(ref keywords) = self.keywords {
            dict.insert("Keywords".to_string(), Object::String(keywords.as_bytes().to_vec()));
        }
        if let Some(ref creator) = self.creator {
            dict.insert("Creator".to_string(), Object::String(creator.as_bytes().to_vec()));
        }
        if let Some(ref producer) = self.producer {
            dict.insert("Producer".to_string(), Object::String(producer.as_bytes().to_vec()));
        }
        if let Some(ref creation_date) = self.creation_date {
            dict.insert(
                "CreationDate".to_string(),
                Object::String(creation_date.as_bytes().to_vec()),
            );
        }
        if let Some(ref mod_date) = self.mod_date {
            dict.insert("ModDate".to_string(), Object::String(mod_date.as_bytes().to_vec()));
        }

        Object::Dictionary(dict)
    }

    /// Parse from a PDF Info dictionary object.
    ///
    /// PDF text strings are UTF-16BE-with-BOM or PDFDocEncoding, never raw
    /// UTF-8 (ISO 32000-1:2008 §7.9.2.2) — decoded via
    /// [`crate::optional_content::decode_pdf_text_string`], which every
    /// other text-string call site in this codebase already uses.
    pub fn from_object(obj: &Object) -> Self {
        use crate::optional_content::decode_pdf_text_string;
        let mut info = Self::default();

        if let Some(dict) = obj.as_dict() {
            if let Some(Object::String(s)) = dict.get("Title") {
                info.title = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("Author") {
                info.author = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("Subject") {
                info.subject = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("Keywords") {
                info.keywords = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("Creator") {
                info.creator = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("Producer") {
                info.producer = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("CreationDate") {
                info.creation_date = decode_pdf_text_string(s).into();
            }
            if let Some(Object::String(s)) = dict.get("ModDate") {
                info.mod_date = decode_pdf_text_string(s).into();
            }
        }

        info
    }
}

/// Information about a page.
#[derive(Debug, Clone)]
pub struct PageInfo {
    /// Page index (0-based)
    pub index: usize,
    /// Page width in points
    pub width: f32,
    /// Page height in points
    pub height: f32,
    /// Page rotation (0, 90, 180, 270)
    pub rotation: i32,
    /// Object reference for this page
    pub object_ref: ObjectRef,
}

/// Options for saving the document.
#[derive(Debug, Clone, Default)]
pub struct SaveOptions {
    /// Use incremental update (append to original file)
    pub incremental: bool,
    /// Compress streams
    pub compress: bool,
    /// Linearize for fast web view
    pub linearize: bool,
    /// Remove unused objects
    pub garbage_collect: bool,
    /// Encryption configuration (None = no encryption)
    pub encryption: Option<EncryptionConfig>,
}

impl SaveOptions {
    /// Create options for full rewrite (default).
    pub fn full_rewrite() -> Self {
        Self {
            incremental: false,
            compress: true,
            garbage_collect: true,
            ..Default::default()
        }
    }

    /// Create options for incremental update.
    pub fn incremental() -> Self {
        Self {
            incremental: true,
            compress: false,
            garbage_collect: false,
            ..Default::default()
        }
    }

    /// Create options with encryption enabled.
    ///
    /// Uses full rewrite mode since incremental updates don't support
    /// adding encryption to an existing PDF.
    pub fn with_encryption(config: EncryptionConfig) -> Self {
        Self {
            incremental: false,
            compress: true,
            garbage_collect: true,
            encryption: Some(config),
            ..Default::default()
        }
    }
}

/// Encryption algorithm for PDF security.
///
/// Per ISO 32000-1:2008 Section 7.6, PDF supports multiple encryption algorithms.
/// This enum represents the commonly used algorithms.
///
/// **Note**: This is a placeholder for v0.4.0 encryption support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EncryptionAlgorithm {
    /// RC4 with 40-bit key (PDF 1.1+, considered weak).
    Rc4_40,
    /// RC4 with 128-bit key (PDF 1.4+).
    Rc4_128,
    /// AES with 128-bit key (PDF 1.5+).
    Aes128,
    /// AES with 256-bit key (PDF 1.7 Extension Level 3+, recommended).
    #[default]
    Aes256,
}

/// Permission flags for encrypted PDFs.
///
/// Per ISO 32000-1:2008 Section 7.6.3.2, these flags control what operations
/// are permitted when the document is opened with the user password.
///
/// **Note**: This is a placeholder for v0.4.0 encryption support.
#[derive(Debug, Clone, Default)]
pub struct Permissions {
    /// Allow printing the document.
    pub print: bool,
    /// Allow high-resolution printing.
    pub print_high_quality: bool,
    /// Allow modifying the document contents.
    pub modify: bool,
    /// Allow copying or extracting text and graphics.
    pub copy: bool,
    /// Allow adding annotations and form fields.
    pub annotate: bool,
    /// Allow filling in form fields.
    pub fill_forms: bool,
    /// Allow extracting content for accessibility.
    pub accessibility: bool,
    /// Allow document assembly (insert, rotate, delete pages).
    pub assemble: bool,
}

impl Permissions {
    /// Create with all permissions granted.
    pub fn all() -> Self {
        Self {
            print: true,
            print_high_quality: true,
            modify: true,
            copy: true,
            annotate: true,
            fill_forms: true,
            accessibility: true,
            assemble: true,
        }
    }

    /// Create with minimal permissions (view only).
    pub fn read_only() -> Self {
        Self {
            accessibility: true, // Always allow for compliance
            ..Default::default()
        }
    }

    /// Convert permissions to the 32-bit P value for the encryption dictionary.
    ///
    /// PDF Spec: Table 22 - User access permissions
    ///
    /// The returned value has reserved bits set appropriately:
    /// - Bits 7-8 must be 1
    /// - Bits 13-32 must be 1 (for compatibility)
    pub fn to_bits(&self) -> i32 {
        // Base value with required reserved bits set
        // Bits 7-8 (0-indexed: 6-7) and bits 13-32 (0-indexed: 12-31) must be 1
        let mut bits: i32 = 0xFFFFF0C0u32 as i32;

        // Bit 3 (0-indexed: 2): Print
        if self.print {
            bits |= 1 << 2;
        }

        // Bit 4 (0-indexed: 3): Modify contents
        if self.modify {
            bits |= 1 << 3;
        }

        // Bit 5 (0-indexed: 4): Copy or extract text and graphics
        if self.copy {
            bits |= 1 << 4;
        }

        // Bit 6 (0-indexed: 5): Add or modify annotations
        if self.annotate {
            bits |= 1 << 5;
        }

        // Bit 9 (0-indexed: 8): Fill in form fields (R>=3)
        if self.fill_forms {
            bits |= 1 << 8;
        }

        // Bit 10 (0-indexed: 9): Extract text for accessibility (R>=3)
        if self.accessibility {
            bits |= 1 << 9;
        }

        // Bit 11 (0-indexed: 10): Assemble document (R>=3)
        if self.assemble {
            bits |= 1 << 10;
        }

        // Bit 12 (0-indexed: 11): Print high quality (R>=3)
        if self.print_high_quality {
            bits |= 1 << 11;
        }

        bits
    }
}

/// Configuration for PDF encryption on save.
///
/// This struct configures how a PDF should be encrypted when saved.
/// Use with `SaveOptions::with_encryption()` to enable encryption.
///
/// # Example (Planned for v0.4.0)
///
/// ```ignore
/// use pdf_oxide::editor::{EncryptionConfig, EncryptionAlgorithm, Permissions};
///
/// let config = EncryptionConfig {
///     user_password: "user123".to_string(),
///     owner_password: "owner456".to_string(),
///     algorithm: EncryptionAlgorithm::Aes256,
///     permissions: Permissions::all(),
/// };
/// ```
///
/// **Note**: This is a placeholder for v0.4.0 encryption support.
/// Currently, PDFs are saved without encryption.
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Password required to open the document (can be empty for no user password).
    pub user_password: String,
    /// Password for full access and changing security settings.
    pub owner_password: String,
    /// Encryption algorithm to use.
    pub algorithm: EncryptionAlgorithm,
    /// Permission flags when opened with user password.
    pub permissions: Permissions,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            user_password: String::new(),
            owner_password: String::new(),
            algorithm: EncryptionAlgorithm::default(),
            permissions: Permissions::all(),
        }
    }
}

impl EncryptionConfig {
    /// Create a new encryption config with the given passwords.
    pub fn new(user_password: impl Into<String>, owner_password: impl Into<String>) -> Self {
        Self {
            user_password: user_password.into(),
            owner_password: owner_password.into(),
            ..Default::default()
        }
    }

    /// Set the encryption algorithm.
    pub fn with_algorithm(mut self, algorithm: EncryptionAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Set the permissions.
    pub fn with_permissions(mut self, permissions: Permissions) -> Self {
        self.permissions = permissions;
        self
    }
}

/// Trait for editable document operations.
pub trait EditableDocument {
    /// Get document metadata.
    fn get_info(&mut self) -> Result<DocumentInfo>;

    /// Set document metadata.
    fn set_info(&mut self, info: DocumentInfo) -> Result<()>;

    /// Get the number of pages.
    fn page_count(&mut self) -> Result<usize>;

    /// Get information about a specific page.
    fn get_page_info(&mut self, index: usize) -> Result<PageInfo>;

    /// Remove a page by index.
    fn remove_page(&mut self, index: usize) -> Result<()>;

    /// Move a page from one index to another.
    fn move_page(&mut self, from: usize, to: usize) -> Result<()>;

    /// Duplicate a page.
    fn duplicate_page(&mut self, index: usize) -> Result<usize>;

    /// Save the document to a file.
    fn save(&mut self, path: impl AsRef<Path>) -> Result<()>;

    /// Save with specific options.
    fn save_with_options(&mut self, path: impl AsRef<Path>, options: SaveOptions) -> Result<()>;
}

/// PDF document editor.
///
/// Provides a high-level interface for modifying PDF documents.
/// Changes are tracked and can be saved either as incremental updates
/// or as a complete rewrite.
pub struct DocumentEditor {
    /// Source document (for reading)
    source: PdfDocument,
    /// Path to the source file
    source_path: String,
    /// Modified objects (object ID -> new object)
    modified_objects: HashMap<u32, Object>,
    /// New objects to add (will be assigned new IDs)
    new_objects: Vec<Object>,
    /// Next object ID to use for new objects
    next_object_id: u32,
    /// Modified metadata
    modified_info: Option<DocumentInfo>,
    /// Page order (indices into original pages, or negative for removed)
    page_order: Vec<i32>,
    /// Number of pages in original document
    original_page_count: usize,
    /// Track if document has been modified
    is_modified: bool,
    /// Modified page content (page_index → new structure, full replacement)
    modified_content: HashMap<usize, StructureElement>,
    /// Overlay additions for source-loaded pages (source_page_index → new elements).
    /// These are appended to the original content stream rather than replacing it.
    overlay_additions: HashMap<usize, Vec<ContentElement>>,
    /// Resource manager for fonts/images
    resource_manager: ResourceManager,
    /// Track if structure tree needs rebuilding
    structure_modified: bool,
    /// Modified page annotations (page_index → annotations)
    modified_annotations: HashMap<usize, Vec<crate::editor::dom::AnnotationWrapper>>,
    /// Modified page properties (rotation, boxes)
    modified_page_props: HashMap<usize, ModifiedPageProps>,
    /// Erase regions per page (whiteout overlays)
    erase_regions: HashMap<usize, Vec<[f32; 4]>>,
    /// Pages where annotations should be flattened
    flatten_annotations_pages: std::collections::HashSet<usize>,
    /// Pages where redactions should be applied
    apply_redactions_pages: std::collections::HashSet<usize>,
    /// Programmatic redaction regions queued per source page index
    /// (in addition to any `/Redact` annotations), used by destructive
    /// redaction (#231).
    redaction_regions: HashMap<usize, Vec<crate::redaction::RedactionRegion>>,
    /// Destructively-redacted replacement content bytes per source page
    /// index, produced by `apply_redactions_destructive` and written by
    /// the save path as the page's single new `/Contents` stream so the
    /// old content object is GC'd (no residual recoverable bytes, G6).
    redacted_content: HashMap<usize, Vec<u8>>,
    /// Object ids of the *original* `/Contents` streams of redacted
    /// pages. These hold the pre-redaction (secret) bytes and must NEVER
    /// be emitted, independent of GC reachability — the hard G6 guarantee
    /// (#231; reachability alone is insufficient because the page is only
    /// repointed transiently during the write).
    redacted_orphan_ids: std::collections::HashSet<u32>,
    /// Image modifications per page: page_index -> (image_name -> modification)
    image_modifications: HashMap<usize, HashMap<String, ImageModification>>,
    /// Pages where form fields should be flattened
    flatten_forms_pages: std::collections::HashSet<usize>,
    /// Flag to remove AcroForm from catalog after form flattening
    remove_acroform: bool,
    /// Warnings collected during form flattening (e.g. widgets with no /AP stream)
    flatten_warnings: Vec<String>,
    /// Embedded files to add to the document
    embedded_files: Vec<crate::writer::EmbeddedFile>,
    /// Modified or new form fields (field name → wrapper)
    modified_form_fields: HashMap<String, FormFieldWrapper>,
    /// Deleted form field names
    deleted_form_fields: HashSet<String>,
    /// Flag indicating AcroForm dictionary needs rebuilding on save
    acroform_modified: bool,
    /// Pages imported from other PDFs via merge operations.
    /// Each entry contains a page object and all its dependent objects,
    /// with references remapped to new IDs in this document.
    merged_pages: Vec<MergedPageData>,
}

/// Data for a single page imported from another PDF during a merge operation.
#[derive(Debug, Clone)]
struct MergedPageData {
    /// The page dictionary object (with remapped references).
    page_object: Object,
    /// All dependent objects (fonts, content streams, resources, etc.)
    /// keyed by their new object ID in this document.
    objects: Vec<(u32, Object)>,
}

/// Tracks modified page properties.
#[derive(Debug, Clone, Default)]
pub struct ModifiedPageProps {
    /// New rotation value (0, 90, 180, 270)
    pub rotation: Option<i32>,
    /// New MediaBox
    pub media_box: Option<[f32; 4]>,
    /// New CropBox
    pub crop_box: Option<[f32; 4]>,
}

/// Stores annotation appearance data for flattening.
#[derive(Debug, Clone)]
struct AnnotationAppearance {
    /// Content stream bytes from the appearance
    content: Vec<u8>,
    /// BBox of the appearance XObject
    bbox: [f32; 4],
    /// Rect of the annotation on the page
    annot_rect: [f32; 4],
    /// Optional transformation matrix from the appearance
    matrix: Option<[f32; 6]>,
    /// Resources used by the appearance
    resources: Option<Object>,
    /// Extra indirect objects this appearance depends on (e.g. an embedded
    /// fallback-font object graph), written verbatim alongside the appearance
    /// XObject. Empty for self-contained appearances.
    extra_objects: Vec<(u32, Object)>,
}

/// Information about an image on a page.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageInfo {
    /// XObject name (e.g., "Im1")
    pub name: String,
    /// Position and size: x, y, width, height
    pub bounds: [f32; 4],
    /// Full transformation matrix [a, b, c, d, e, f]
    pub matrix: [f32; 6],
}

/// Modification to apply to an image.
#[derive(Debug, Clone)]
struct ImageModification {
    /// New x position (if Some, changes position)
    x: Option<f32>,
    /// New y position (if Some, changes position)
    y: Option<f32>,
    /// New width (if Some, changes width)
    width: Option<f32>,
    /// New height (if Some, changes height)
    height: Option<f32>,
}

impl DocumentEditor {
    /// Open a PDF document for editing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let editor = DocumentEditor::open("document.pdf")?;
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut source = PdfDocument::open(path.as_ref())?;

        // Get page count
        let page_count = source.page_count()?;

        // Find the highest object ID to know where to start for new objects
        let next_id = Self::find_max_object_id(&source) + 1;

        // Initialize page order as sequential
        let page_order: Vec<i32> = (0..page_count as i32).collect();

        Ok(Self {
            source,
            source_path: path_str,
            modified_objects: HashMap::new(),
            new_objects: Vec::new(),
            next_object_id: next_id,
            modified_info: None,
            page_order,
            original_page_count: page_count,
            is_modified: false,
            modified_content: HashMap::new(),
            overlay_additions: HashMap::new(),
            resource_manager: ResourceManager::new(),
            structure_modified: false,
            modified_annotations: HashMap::new(),
            modified_page_props: HashMap::new(),
            erase_regions: HashMap::new(),
            flatten_annotations_pages: std::collections::HashSet::new(),
            apply_redactions_pages: std::collections::HashSet::new(),
            redaction_regions: HashMap::new(),
            redacted_content: HashMap::new(),
            redacted_orphan_ids: std::collections::HashSet::new(),
            image_modifications: HashMap::new(),
            flatten_forms_pages: std::collections::HashSet::new(),
            remove_acroform: false,
            flatten_warnings: Vec::new(),
            embedded_files: Vec::new(),
            modified_form_fields: HashMap::new(),
            deleted_form_fields: HashSet::new(),
            acroform_modified: false,
            merged_pages: Vec::new(),
        })
    }

    /// Open a PDF document for editing from an existing PdfDocument object.
    pub fn from_document(mut source: PdfDocument) -> Result<Self> {
        let page_count = source.page_count()?;
        let next_id = Self::find_max_object_id(&source) + 1;
        let page_order: Vec<i32> = (0..page_count as i32).collect();

        Ok(Self {
            source,
            source_path: String::new(),
            modified_objects: HashMap::new(),
            new_objects: Vec::new(),
            next_object_id: next_id,
            modified_info: None,
            page_order,
            original_page_count: page_count,
            is_modified: false,
            modified_content: HashMap::new(),
            overlay_additions: HashMap::new(),
            resource_manager: ResourceManager::new(),
            structure_modified: false,
            modified_annotations: HashMap::new(),
            modified_page_props: HashMap::new(),
            erase_regions: HashMap::new(),
            flatten_annotations_pages: std::collections::HashSet::new(),
            apply_redactions_pages: std::collections::HashSet::new(),
            redaction_regions: HashMap::new(),
            redacted_content: HashMap::new(),
            redacted_orphan_ids: std::collections::HashSet::new(),
            image_modifications: HashMap::new(),
            flatten_forms_pages: std::collections::HashSet::new(),
            remove_acroform: false,
            flatten_warnings: Vec::new(),
            embedded_files: Vec::new(),
            modified_form_fields: HashMap::new(),
            deleted_form_fields: HashSet::new(),
            acroform_modified: false,
            merged_pages: Vec::new(),
        })
    }

    /// Open a PDF document for editing from in-memory bytes.
    ///
    /// This is the equivalent of `open()` but works with byte data instead of a file path,
    /// making it suitable for WASM environments where filesystem access is unavailable.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let mut source = PdfDocument::from_bytes(data)?;
        let page_count = source.page_count()?;
        let next_id = Self::find_max_object_id(&source) + 1;
        let page_order: Vec<i32> = (0..page_count as i32).collect();

        Ok(Self {
            source,
            source_path: String::new(),
            modified_objects: HashMap::new(),
            new_objects: Vec::new(),
            next_object_id: next_id,
            modified_info: None,
            page_order,
            original_page_count: page_count,
            is_modified: false,
            modified_content: HashMap::new(),
            overlay_additions: HashMap::new(),
            resource_manager: ResourceManager::new(),
            structure_modified: false,
            modified_annotations: HashMap::new(),
            modified_page_props: HashMap::new(),
            erase_regions: HashMap::new(),
            flatten_annotations_pages: std::collections::HashSet::new(),
            apply_redactions_pages: std::collections::HashSet::new(),
            redaction_regions: HashMap::new(),
            redacted_content: HashMap::new(),
            redacted_orphan_ids: std::collections::HashSet::new(),
            image_modifications: HashMap::new(),
            flatten_forms_pages: std::collections::HashSet::new(),
            remove_acroform: false,
            flatten_warnings: Vec::new(),
            embedded_files: Vec::new(),
            modified_form_fields: HashMap::new(),
            deleted_form_fields: HashSet::new(),
            acroform_modified: false,
            merged_pages: Vec::new(),
        })
    }

    /// Deprecated alias for `from_bytes`.
    #[deprecated(since = "0.3.15", note = "Use `from_bytes` instead")]
    pub fn open_from_bytes(data: Vec<u8>) -> Result<Self> {
        Self::from_bytes(data)
    }

    /// Save the document to an in-memory byte vector.
    ///
    /// This is the equivalent of `save()` but returns bytes instead of writing to a file,
    /// making it suitable for WASM environments where filesystem access is unavailable.
    pub fn save_to_bytes(&mut self) -> Result<Vec<u8>> {
        self.save_to_bytes_with_options(SaveOptions::full_rewrite())
    }

    /// Save the document to an in-memory byte vector with specific options.
    pub fn save_to_bytes_with_options(&mut self, options: SaveOptions) -> Result<Vec<u8>> {
        use std::io::Cursor;
        if options.incremental {
            return Err(Error::InvalidPdf(
                "Incremental saves are not supported for in-memory output".to_string(),
            ));
        }
        let mut cursor = Cursor::new(Vec::new());
        self.write_full_to_writer(&mut cursor, &options)?;
        Ok(cursor.into_inner())
    }

    /// Find the maximum object ID in the document.
    fn find_max_object_id(doc: &PdfDocument) -> u32 {
        // Get /Size from trailer - this is the number of xref entries (max ID + 1)
        doc.trailer()
            .as_dict()
            .and_then(|d| d.get("Size"))
            .and_then(|s| s.as_integer())
            .map(|size| size as u32)
            .unwrap_or(100) // Fallback to reasonable default
    }

    /// Allocate a new object ID.
    fn allocate_object_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Allocate a new object ID (accessible to sibling crate modules).
    pub(crate) fn alloc_id(&mut self) -> u32 {
        self.allocate_object_id()
    }

    /// Stage a new or modified object (accessible to sibling crate modules).
    pub(crate) fn insert_modified(&mut self, id: u32, obj: Object) {
        self.modified_objects.insert(id, obj);
        self.is_modified = true;
    }

    /// Look up a staged object by ID (accessible to sibling crate modules).
    pub(crate) fn get_modified(&self, id: u32) -> Option<&Object> {
        self.modified_objects.get(&id)
    }

    /// Serialise all staged changes to bytes, re-parse, and reset staging state
    /// so that subsequent reads (e.g. from the validator) see the mutations.
    pub(crate) fn commit_in_place(&mut self) -> Result<()> {
        if !self.is_modified {
            return Ok(());
        }
        let new_bytes = self.save_to_bytes()?;
        self.source = PdfDocument::from_bytes(new_bytes)?;
        self.modified_objects.clear();
        self.new_objects.clear();
        self.next_object_id = Self::find_max_object_id(&self.source) + 1;
        self.is_modified = false;
        Ok(())
    }

    /// Replace the source document wholesale from external bytes.
    ///
    /// Used when a conversion step produces a completely new PDF (e.g. transparency
    /// flattening via re-rendering) rather than mutating individual objects.
    /// Clears all staged modifications so subsequent fixes operate on the new document.
    pub(crate) fn replace_source_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.source = PdfDocument::from_bytes(bytes)?;
        self.modified_objects.clear();
        self.new_objects.clear();
        self.next_object_id = Self::find_max_object_id(&self.source) + 1;
        // is_modified stays false: the source already holds the new bytes and
        // no objects are staged, so commit_in_place() would be a no-op anyway.
        // Subsequent insert_modified/alloc calls will flip it back to true.
        Ok(())
    }

    /// Consume the editor and return the underlying document.
    pub fn into_source(self) -> PdfDocument {
        self.source
    }

    /// Apply page property modifications to a page object.
    ///
    /// Returns a new page object with the modifications applied.
    fn apply_page_props_to_object(
        &self,
        page_obj: &Object,
        props: &ModifiedPageProps,
    ) -> Result<Object> {
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;

        let mut new_dict = page_dict.clone();

        // Apply rotation if modified
        if let Some(rotation) = props.rotation {
            new_dict.insert("Rotate".to_string(), Object::Integer(rotation as i64));
        }

        // Apply MediaBox if modified
        if let Some(media_box) = props.media_box {
            let box_array = Object::Array(vec![
                Object::Real(media_box[0] as f64),
                Object::Real(media_box[1] as f64),
                Object::Real(media_box[2] as f64),
                Object::Real(media_box[3] as f64),
            ]);
            new_dict.insert("MediaBox".to_string(), box_array);
        }

        // Apply CropBox if modified
        if let Some(crop_box) = props.crop_box {
            let box_array = Object::Array(vec![
                Object::Real(crop_box[0] as f64),
                Object::Real(crop_box[1] as f64),
                Object::Real(crop_box[2] as f64),
                Object::Real(crop_box[3] as f64),
            ]);
            new_dict.insert("CropBox".to_string(), box_array);
        }

        Ok(Object::Dictionary(new_dict))
    }

    /// Check if the document has unsaved changes.
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Get the source file path.
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Get immutable reference to the source document.
    pub fn source(&self) -> &PdfDocument {
        &self.source
    }

    /// Get mutable reference to the source document.
    ///
    /// This provides access to PdfDocument methods for extraction and conversion.
    pub fn source_mut(&mut self) -> &mut PdfDocument {
        &mut self.source
    }

    /// Get the PDF version.
    pub fn version(&self) -> (u8, u8) {
        self.source.version()
    }

    // === Metadata operations ===

    /// Get the document title.
    pub fn title(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.title)
    }

    /// Set the document title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        let title = title.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.title = Some(title);
        }
        self.is_modified = true;
    }

    /// Get the document author.
    pub fn author(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.author)
    }

    /// Set the document author.
    pub fn set_author(&mut self, author: impl Into<String>) {
        let author = author.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.author = Some(author);
        }
        self.is_modified = true;
    }

    /// Get the document subject.
    pub fn subject(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.subject)
    }

    /// Set the document subject.
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        let subject = subject.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.subject = Some(subject);
        }
        self.is_modified = true;
    }

    /// Get the document keywords.
    pub fn keywords(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.keywords)
    }

    /// Set the document keywords.
    pub fn set_keywords(&mut self, keywords: impl Into<String>) {
        let keywords = keywords.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.keywords = Some(keywords);
        }
        self.is_modified = true;
    }

    /// Get the document producer (tool that produced the PDF).
    pub fn producer(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.producer)
    }

    /// Set the document producer. Persists to `/Info.Producer` on save.
    pub fn set_producer(&mut self, producer: impl Into<String>) {
        let producer = producer.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.producer = Some(producer);
        }
        self.is_modified = true;
    }

    /// Get the raw PDF creation-date string (e.g. `D:20260421120000Z`).
    pub fn creation_date(&mut self) -> Result<Option<String>> {
        let info = self.get_info()?;
        Ok(info.creation_date)
    }

    /// Set the raw PDF creation-date string. Persists to
    /// `/Info.CreationDate` on save.
    pub fn set_creation_date(&mut self, date: impl Into<String>) {
        let date = date.into();
        if self.modified_info.is_none() {
            self.modified_info = Some(self.get_info().unwrap_or_default());
        }
        if let Some(ref mut info) = self.modified_info {
            info.creation_date = Some(date);
        }
        self.is_modified = true;
    }

    // === Page operations ===

    /// Get the current page count (after modifications).
    pub fn current_page_count(&self) -> usize {
        self.page_order.iter().filter(|&&i| i >= 0).count() + self.merged_pages.len()
    }

    /// Translate a user-visible (output-order) page index to the internal source page index.
    ///
    /// Per-page HashMaps (erase_regions, overlay_additions, modified_page_props, …) are keyed
    /// by source index so that the serialization loop—which iterates `page_order` and already
    /// uses source indices—can find entries regardless of how `select_pages` / `remove_page`
    /// has reordered the output.
    ///
    /// When no reordering has been applied `page_order = [0, 1, 2, …]` so the mapping is the
    /// identity and there is no observable difference for callers.
    fn output_to_source_index(&self, output: usize) -> usize {
        self.page_order
            .iter()
            .filter(|&&i| i >= 0)
            .map(|&i| i as usize)
            .nth(output)
            .unwrap_or(output)
    }

    /// Get the list of page objects in current order.
    fn get_page_refs(&mut self) -> Result<Vec<ObjectRef>> {
        // Get catalog and pages tree
        let catalog = self.source.catalog()?;
        let catalog_dict = catalog
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Catalog is not a dictionary".to_string()))?;

        let pages_ref = catalog_dict
            .get("Pages")
            .ok_or_else(|| Error::InvalidPdf("Catalog missing /Pages".to_string()))?
            .as_reference()
            .ok_or_else(|| Error::InvalidPdf("/Pages is not a reference".to_string()))?;

        let pages_obj = self.source.load_object(pages_ref)?;
        let pages_dict = pages_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Pages is not a dictionary".to_string()))?;

        // Get Kids array
        let kids = pages_dict
            .get("Kids")
            .ok_or_else(|| Error::InvalidPdf("Pages missing /Kids".to_string()))?
            .as_array()
            .ok_or_else(|| Error::InvalidPdf("/Kids is not an array".to_string()))?;

        // Collect page references (flattening any intermediate Pages nodes)
        let mut page_refs = Vec::new();
        self.collect_page_refs(kids, &mut page_refs)?;

        Ok(page_refs)
    }

    /// Recursively collect page references from a Kids array.
    fn collect_page_refs(&mut self, kids: &[Object], refs: &mut Vec<ObjectRef>) -> Result<()> {
        for kid in kids {
            if let Some(kid_ref) = kid.as_reference() {
                let kid_obj = self.source.load_object(kid_ref)?;
                if let Some(kid_dict) = kid_obj.as_dict() {
                    let type_name = kid_dict.get("Type").and_then(|t| t.as_name()).unwrap_or("");

                    if type_name == "Page" {
                        refs.push(kid_ref);
                    } else if type_name == "Pages" {
                        // Intermediate Pages node - recurse
                        if let Some(sub_kids) = kid_dict.get("Kids").and_then(|k| k.as_array()) {
                            self.collect_page_refs(sub_kids, refs)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract a subset of pages into a new PDF and write it to `output`.
    ///
    /// `pages` is a list of 0-based indices to keep. The current document is
    /// not modified. Works by cloning the document and removing all unwanted
    /// pages in reverse-index order (same approach used by the WASM binding).
    pub fn extract_pages(&mut self, pages: &[usize], output: impl AsRef<Path>) -> Result<()> {
        let bytes = self.extract_pages_to_bytes(pages)?;
        std::fs::write(output, bytes)?;
        Ok(())
    }

    /// Extract a subset of pages and return the result as PDF bytes.
    ///
    /// `pages` is a list of 0-based indices to keep. The current document is
    /// not modified.
    pub fn extract_pages_to_bytes(&mut self, pages: &[usize]) -> Result<Vec<u8>> {
        use crate::editor::EditableDocument;

        if pages.is_empty() {
            return Err(Error::InvalidPdf("pages list must not be empty".to_string()));
        }

        let page_count = self.page_count()?;
        for &page in pages {
            if page >= page_count {
                return Err(Error::InvalidPdf(format!(
                    "Page index {} out of range (document has {} pages)",
                    page, page_count
                )));
            }
        }

        // Slow path for documents with appended pages from merge_from(): the
        // fast path swaps page_order, but merged pages live in a separate
        // vector and aren't represented there.
        if !self.merged_pages.is_empty() {
            let keep: std::collections::HashSet<usize> = pages.iter().copied().collect();
            let snapshot = self.save_to_bytes()?;
            let mut copy = DocumentEditor::from_bytes(snapshot)?;
            for i in (0..page_count).rev() {
                if !keep.contains(&i) {
                    copy.remove_page(i)?;
                }
            }
            return copy.save_to_bytes();
        }

        // Fast path: serialise once with a trimmed page_order and a staged
        // Pages dict, so collect_reachable_ids() drops orphan objects from
        // dropped pages instead of walking the original page tree.
        let visible: Vec<i32> = self
            .page_order
            .iter()
            .filter(|&&i| i >= 0)
            .copied()
            .collect();
        let new_order: Vec<i32> = pages.iter().map(|&i| visible[i]).collect();

        // Stage a trimmed /Pages dict in modified_objects so GC reachability
        // sees only kept pages. write_full_to_writer rebuilds its own Kids
        // list, so this staging only matters for the GC walk.
        let pages_ref = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Root"))
            .and_then(|r| r.as_reference())
            .and_then(|catalog_ref| self.source.load_object(catalog_ref).ok())
            .and_then(|catalog_obj| {
                catalog_obj
                    .as_dict()
                    .and_then(|d| d.get("Pages"))
                    .and_then(|p| p.as_reference())
            });

        // Resolve all leaf page refs in one tree walk (avoids O(n²) of calling
        // get_page_ref(i) per index).
        let all_refs = self.source.all_page_refs().unwrap_or_default();

        let staged_pages: Option<(u32, Option<Object>)> = if let Some(pages_ref) = pages_ref {
            let pages_obj = self.source.load_object(pages_ref).ok();
            let pages_dict = pages_obj.as_ref().and_then(|p| p.as_dict()).cloned();
            if let Some(mut new_pages_dict) = pages_dict {
                let mut kids: Vec<Object> = Vec::with_capacity(new_order.len());
                for &leaf_idx in &new_order {
                    if leaf_idx >= 0 {
                        let idx = leaf_idx as usize;
                        if idx < all_refs.len() {
                            kids.push(Object::Reference(all_refs[idx]));
                        }
                    }
                }
                new_pages_dict.insert("Count".to_string(), Object::Integer(kids.len() as i64));
                new_pages_dict.insert("Kids".to_string(), Object::Array(kids));
                let prior = self
                    .modified_objects
                    .insert(pages_ref.id, Object::Dictionary(new_pages_dict));
                Some((pages_ref.id, prior))
            } else {
                None
            }
        } else {
            None
        };

        let saved_order = std::mem::replace(&mut self.page_order, new_order);
        let saved_is_modified = std::mem::replace(&mut self.is_modified, true);

        let result = self.save_to_bytes();

        // Always restore — even on Err — so the document is observably unchanged.
        self.page_order = saved_order;
        self.is_modified = saved_is_modified;
        if let Some((pages_id, prior)) = staged_pages {
            match prior {
                Some(prev_obj) => {
                    self.modified_objects.insert(pages_id, prev_obj);
                },
                None => {
                    self.modified_objects.remove(&pages_id);
                },
            }
        }

        result
    }

    /// Extract several non-overlapping page ranges in one call, returning the
    /// PDF bytes for each.
    ///
    /// `ranges` is a list of `(start, end)` tuples interpreted as `[start, end)`
    /// half-open ranges over current visible pages. `start` may equal `end`
    /// (empty range is rejected, matching `extract_pages_to_bytes`). Each output
    /// is independent; this call does not deduplicate work between ranges
    /// beyond the per-document caches that warm up after the first one.
    ///
    /// Mirrors the chunked workflow described in issue #474: a 12k-page
    /// document split into 3000-page chunks for downstream processing.
    pub fn extract_page_ranges_to_bytes(
        &mut self,
        ranges: &[(usize, usize)],
    ) -> Result<Vec<Vec<u8>>> {
        let mut out: Vec<Vec<u8>> = Vec::with_capacity(ranges.len());
        for &(start, end) in ranges {
            if end < start {
                return Err(Error::InvalidPdf(format!(
                    "Invalid page range: end ({}) < start ({})",
                    end, start
                )));
            }
            let pages: Vec<usize> = (start..end).collect();
            out.push(self.extract_pages_to_bytes(&pages)?);
        }
        Ok(out)
    }

    /// Restrict the document to the listed pages, in the order given. The
    /// pages not listed are dropped (in `page_order`); subsequent
    /// `save_to_bytes()` / `save()` will produce a PDF containing only the
    /// selected pages, with garbage-collected resources.
    ///
    /// This is the in-place analogue of `extract_pages_to_bytes`: it mutates
    /// the editor instead of returning bytes, so chained edits (e.g. add
    /// annotations, then save) work as expected. Equivalent to PyMuPDF's
    /// `doc.select(page_list)`.
    pub fn select_pages(&mut self, pages: &[usize]) -> Result<()> {
        use crate::editor::EditableDocument;

        if pages.is_empty() {
            return Err(Error::InvalidPdf("pages list must not be empty".to_string()));
        }
        let page_count = self.page_count()?;
        for &p in pages {
            if p >= page_count {
                return Err(Error::InvalidPdf(format!(
                    "Page index {} out of range (document has {} pages)",
                    p, page_count
                )));
            }
        }
        if !self.merged_pages.is_empty() {
            return Err(Error::InvalidPdf(
                "select_pages does not yet support documents with pages added via merge_from"
                    .to_string(),
            ));
        }

        let visible: Vec<i32> = self
            .page_order
            .iter()
            .filter(|&&i| i >= 0)
            .copied()
            .collect();
        let new_order: Vec<i32> = pages.iter().map(|&i| visible[i]).collect();

        self.page_order = new_order;
        self.is_modified = true;
        Ok(())
    }

    /// Overlay a PNG image onto an existing page at the given position.
    ///
    /// `page_index` is 0-based. `x`, `y`, `width`, `height` are in PDF points
    /// (1/72 inch), measured from the bottom-left of the page.
    ///
    /// The PNG bytes are decoded, then appended to the page's content stream as
    /// an XObject.  The page is saved back after the operation.
    pub fn add_image_bytes_to_page(
        &mut self,
        page_index: usize,
        png_bytes: &[u8],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        use crate::elements::ImageContent;
        use crate::geometry::Rect;

        let image = ImageContent::from_bytes(Rect::new(x, y, width, height), png_bytes.to_vec())
            .map_err(|e| crate::error::Error::Image(e.to_string()))?;
        self.edit_page(page_index, |page| {
            page.add_image(image);
            Ok(())
        })
    }

    /// Merge pages from another PDF into this document.
    ///
    /// This appends all pages from the source PDF to the end of this document.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("main.pdf")?;
    /// editor.merge_from("appendix.pdf")?;
    /// editor.save("combined.pdf")?;
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn merge_from(&mut self, source_path: impl AsRef<Path>) -> Result<usize> {
        let data = std::fs::read(source_path.as_ref())?;
        self.merge_from_bytes(&data)
    }

    /// Merge another PDF (from raw bytes) into this document.
    ///
    /// Works in all environments including WebAssembly.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes of the PDF to merge
    ///
    /// # Returns
    ///
    /// Number of pages merged from the source PDF.
    pub fn merge_from_bytes(&mut self, data: &[u8]) -> Result<usize> {
        let mut source_doc = PdfDocument::from_bytes(data.to_vec())?;
        let source_page_count = source_doc.page_count()?;

        if source_page_count == 0 {
            return Ok(0);
        }

        // Import each page from the source document
        for page_idx in 0..source_page_count {
            let page_data = self.import_page_from_document(&mut source_doc, page_idx)?;
            self.merged_pages.push(page_data);
        }

        self.is_modified = true;
        Ok(source_page_count)
    }

    /// Import a single page and all its dependent objects from a source document.
    ///
    /// Performs a deep copy of the page object graph, remapping all indirect
    /// references to new object IDs allocated in this document.
    fn import_page_from_document(
        &mut self,
        source: &mut PdfDocument,
        page_index: usize,
    ) -> Result<MergedPageData> {
        let page_ref = source.get_page_ref(page_index)?;
        let page_obj = source.load_object(page_ref)?;

        // Strip /Parent before deep import to avoid pulling in the entire
        // source page tree (which causes cycles and imports unreachable objects)
        let stripped_page = if let Object::Dictionary(mut dict) = page_obj {
            dict.remove("Parent");
            Object::Dictionary(dict)
        } else {
            page_obj
        };

        // Map from source object ID -> new object ID in this document
        let mut id_map: HashMap<u32, u32> = HashMap::new();
        // Collected objects: new_id -> remapped object
        let mut collected: Vec<(u32, Object)> = Vec::new();

        // Deep-copy the page object, recursively importing all referenced objects
        let final_page = self.deep_import_object(
            source,
            &stripped_page,
            &mut id_map,
            &mut collected,
            &mut HashSet::new(),
        )?;

        Ok(MergedPageData {
            page_object: final_page,
            objects: collected,
        })
    }

    /// Recursively import a PDF object, remapping all indirect references.
    ///
    /// When an `Object::Reference` is encountered, the referenced object is
    /// loaded from the source document, assigned a new ID, and recursively
    /// imported. The reference is rewritten to point to the new ID.
    fn deep_import_object(
        &mut self,
        source: &mut PdfDocument,
        obj: &Object,
        id_map: &mut HashMap<u32, u32>,
        collected: &mut Vec<(u32, Object)>,
        visiting: &mut HashSet<u32>,
    ) -> Result<Object> {
        match obj {
            Object::Reference(obj_ref) => {
                // Check if we already remapped this reference
                if let Some(&new_id) = id_map.get(&obj_ref.id) {
                    return Ok(Object::Reference(ObjectRef::new(new_id, 0)));
                }

                // Cycle detection
                if !visiting.insert(obj_ref.id) {
                    // Already visiting this object (cycle) - allocate an ID
                    // and return a reference; the object will be filled later
                    let new_id = self.allocate_object_id();
                    id_map.insert(obj_ref.id, new_id);
                    return Ok(Object::Reference(ObjectRef::new(new_id, 0)));
                }

                // Allocate a new ID for this object
                let new_id = self.allocate_object_id();
                id_map.insert(obj_ref.id, new_id);

                // Load and recursively import the referenced object
                let loaded = source.load_object(*obj_ref)?;
                let remapped =
                    self.deep_import_object(source, &loaded, id_map, collected, visiting)?;

                visiting.remove(&obj_ref.id);

                // Store the imported object
                collected.push((new_id, remapped));

                Ok(Object::Reference(ObjectRef::new(new_id, 0)))
            },
            Object::Dictionary(dict) => {
                let mut new_dict = HashMap::with_capacity(dict.len());
                for (key, value) in dict {
                    let new_value =
                        self.deep_import_object(source, value, id_map, collected, visiting)?;
                    new_dict.insert(key.clone(), new_value);
                }
                Ok(Object::Dictionary(new_dict))
            },
            Object::Array(arr) => {
                let mut new_arr = Vec::with_capacity(arr.len());
                for item in arr {
                    let new_item =
                        self.deep_import_object(source, item, id_map, collected, visiting)?;
                    new_arr.push(new_item);
                }
                Ok(Object::Array(new_arr))
            },
            Object::Stream { dict, data } => {
                let mut new_dict = HashMap::with_capacity(dict.len());
                for (key, value) in dict {
                    let new_value =
                        self.deep_import_object(source, value, id_map, collected, visiting)?;
                    new_dict.insert(key.clone(), new_value);
                }
                Ok(Object::Stream {
                    dict: new_dict,
                    data: data.clone(),
                })
            },
            // Primitive types need no remapping
            _ => Ok(obj.clone()),
        }
    }

    /// Merge specific pages from another PDF into this document.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the PDF to merge from
    /// * `pages` - Indices of pages to merge (0-based)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("main.pdf")?;
    /// editor.merge_pages_from("source.pdf", &[0, 2, 4])?;  // Merge pages 1, 3, 5
    /// editor.save("combined.pdf")?;
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn merge_pages_from(
        &mut self,
        source_path: impl AsRef<Path>,
        pages: &[usize],
    ) -> Result<usize> {
        let mut source_doc = PdfDocument::open(source_path.as_ref())?;
        let source_page_count = source_doc.page_count()?;

        // Validate page indices
        for &page in pages {
            if page >= source_page_count {
                return Err(Error::InvalidPdf(format!(
                    "Page index {} out of range (source has {} pages)",
                    page, source_page_count
                )));
            }
        }

        if pages.is_empty() {
            return Ok(0);
        }

        for &page_idx in pages {
            let page_data = self.import_page_from_document(&mut source_doc, page_idx)?;
            self.merged_pages.push(page_data);
        }

        self.is_modified = true;
        Ok(pages.len())
    }

    // === Internal save helpers ===

    /// Read the original PDF file bytes.
    fn read_source_bytes(&self) -> Result<Vec<u8>> {
        let mut file = File::open(&self.source_path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Build the Info dictionary object for the trailer.
    fn build_info_object(&self) -> Option<Object> {
        self.modified_info.as_ref().map(|info| info.to_object())
    }

    /// Write an incremental update to the PDF.
    #[cfg(not(target_arch = "wasm32"))]
    fn write_incremental(&mut self, path: impl AsRef<Path>) -> Result<()> {
        // Refuse rather than silently corrupt (#1032): an incremental
        // update appends new/modified objects as plain, unencrypted bytes
        // after the untouched original file (which keeps its own streams
        // correctly encrypted and readable under the original password),
        // but the trailer's /Prev chain still points back to the source's
        // /Encrypt dictionary — a conforming reader would try to decrypt
        // the newly-appended plaintext objects with the document's
        // original key and corrupt them. Properly supporting this would
        // mean re-encrypting the appended objects under the source's own
        // key (EncryptionHandler already exposes what that needs), which
        // is a real but separate feature; for now, direct callers to the
        // full-rewrite path instead, which already decrypts-then-emits
        // every copied stream plaintext.
        if self.source.is_encrypted() {
            return Err(Error::InvalidPdf(
                "incremental save is not supported for encrypted documents; \
                 use save_to_bytes()/save_with_options(SaveOptions::full_rewrite()) instead"
                    .to_string(),
            ));
        }

        // Read original file
        let original_bytes = self.read_source_bytes()?;
        let original_len = original_bytes.len();

        // Open output file
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        // Write original content
        writer.write_all(&original_bytes)?;

        // Start incremental update section
        let update_start = original_len as u64;

        // Sync form field changes into modified_objects before writing
        if self.acroform_modified {
            self.flush_form_fields_to_modified_objects()?;
        }

        // Track new xref entries
        let mut xref_entries: Vec<(u32, u64, u16)> = Vec::new();
        let serializer = ObjectSerializer::compact();

        // Write modified objects
        for (&obj_id, obj) in &self.modified_objects {
            let offset = writer.stream_position().unwrap_or(update_start);
            let bytes = serializer.serialize_indirect(obj_id, 0, obj);
            writer.write_all(&bytes)?;
            xref_entries.push((obj_id, offset, 0));
        }

        // Write new Info object if metadata was modified
        if let Some(info_obj) = self.build_info_object() {
            let info_id = self.next_object_id;
            let offset = writer.stream_position().unwrap_or(update_start);
            let bytes = serializer.serialize_indirect(info_id, 0, &info_obj);
            writer.write_all(&bytes)?;
            xref_entries.push((info_id, offset, 0));
        }

        // Write new xref section
        let xref_offset = writer.stream_position().unwrap_or(update_start);
        write!(writer, "xref\n")?;

        // Sort entries by object ID
        xref_entries.sort_by_key(|(id, _, _)| *id);

        // Write xref subsections
        // For simplicity, write each entry as its own subsection
        for (obj_id, offset, gen) in &xref_entries {
            write!(writer, "{} 1\n", obj_id)?;
            write!(writer, "{:010} {:05} n \n", offset, gen)?;
        }

        // Write trailer
        write!(writer, "trailer\n")?;
        write!(writer, "<<\n")?;
        write!(writer, "  /Size {}\n", self.next_object_id + 1)?;
        write!(writer, "  /Prev {}\n", self.find_prev_xref_offset(&original_bytes)?)?;

        // Add /Root reference (from original trailer)
        if let Ok(catalog) = self.source.catalog() {
            if let Some(dict) = self.source.trailer().as_dict() {
                if let Some(root_ref) = dict.get("Root") {
                    write!(writer, "  /Root ")?;
                    writer.write_all(&serializer.serialize(root_ref))?;
                    write!(writer, "\n")?;
                }
            }
        }

        // Add /Info reference if we created one
        if self.modified_info.is_some() {
            write!(writer, "  /Info {} 0 R\n", self.next_object_id)?;
        }

        // ISO 32000-1:2008 §7.5.6 (`docs/spec/pdf.md:3639`): "The added trailer
        // shall contain all the entries except the Prev entry (if present) from
        // the previous trailer, whether modified or not."
        //
        // Only /Size, /Prev, /Root and /Info were being written, so every other
        // entry the original trailer carried was dropped — /ID most notably,
        // whose absence "might prevent the file from functioning in some
        // workflows that depend on files being uniquely identified" (Table 15
        // NOTE 2). Carry the rest across verbatim.
        //
        // The four handled above are skipped: /Size and /Prev are recomputed
        // for this update by definition, /Root is written from the source
        // trailer just above, and /Info is either rewritten here or inherited
        // through the /Prev chain. /Encrypt cannot appear — an encrypted source
        // is refused at the top of this function, because appending plaintext
        // objects under a document key would corrupt them.
        if let Some(trailer) = self.source.trailer().as_dict() {
            let mut carried: Vec<&String> = trailer
                .keys()
                .filter(|k| !matches!(k.as_str(), "Size" | "Prev" | "Root" | "Info" | "XRefStm"))
                .collect();
            // Deterministic output: a HashMap's order is not stable, and two
            // saves of one document must produce the same bytes.
            carried.sort();
            for key in carried {
                if let Some(value) = trailer.get(key) {
                    write!(writer, "  /{key} ")?;
                    writer.write_all(&serializer.serialize(value))?;
                    write!(writer, "\n")?;
                }
            }
        }

        write!(writer, ">>\n")?;
        write!(writer, "startxref\n")?;
        write!(writer, "{}\n", xref_offset)?;
        write!(writer, "%%EOF\n")?;

        writer.flush()?;
        Ok(())
    }

    /// Sync modified form fields into `modified_objects` for incremental save.
    ///
    /// For each modified field wrapper, loads the original PDF annotation object,
    /// updates its `/V` entry (and `/AS` for checkboxes/radios), and inserts the
    /// updated object into `modified_objects`. Also sets `/NeedAppearances true`
    /// in the AcroForm dictionary so PDF readers regenerate appearance streams.
    fn flush_form_fields_to_modified_objects(&mut self) -> Result<()> {
        use crate::extractors::forms::FieldType;

        // Collect field data we need before mutating self
        let fields_to_flush: Vec<(u32, u16, Object, bool)> = {
            let mut result = Vec::new();
            for wrapper in self.modified_form_fields.values() {
                if !wrapper.is_modified() || wrapper.is_new() {
                    continue;
                }
                let obj_ref = match wrapper.object_ref() {
                    Some(r) => r,
                    None => continue,
                };

                // Build the new /V value from the modified value
                let new_value: Object = match &wrapper.modified_value {
                    Some(val) => val.into(),
                    None => continue,
                };

                let is_button = wrapper
                    .field_type()
                    .map(|ft| *ft == FieldType::Button)
                    .unwrap_or(false);

                result.push((obj_ref.id, obj_ref.gen, new_value, is_button));
            }
            result
        };

        // Now load and update each field object
        for (obj_id, obj_gen, new_value, is_button) in &fields_to_flush {
            let obj_ref = ObjectRef::new(*obj_id, *obj_gen);
            let original = self.source.load_object(obj_ref)?;

            let dict = match original.as_dict() {
                Some(d) => d.clone(),
                None => continue,
            };

            let mut new_dict = dict;
            new_dict.insert("V".to_string(), new_value.clone());

            // For button fields (checkboxes/radios), also update /AS to match /V
            if *is_button {
                new_dict.insert("AS".to_string(), new_value.clone());
            }

            self.modified_objects
                .insert(*obj_id, Object::Dictionary(new_dict));
        }

        // Set /NeedAppearances true in the AcroForm dictionary
        if !fields_to_flush.is_empty() {
            let catalog = self.source.catalog()?;
            let catalog_dict = catalog
                .as_dict()
                .ok_or_else(|| Error::InvalidPdf("Catalog is not a dictionary".to_string()))?;

            if let Some(acroform_obj) = catalog_dict.get("AcroForm") {
                if let Some(acroform_ref) = acroform_obj.as_reference() {
                    // AcroForm is an indirect reference — load, modify, and write back
                    let acroform = self.source.load_object(acroform_ref)?;
                    if let Some(af_dict) = acroform.as_dict() {
                        let mut new_af = af_dict.clone();
                        new_af.insert("NeedAppearances".to_string(), Object::Boolean(true));
                        self.modified_objects
                            .insert(acroform_ref.id, Object::Dictionary(new_af));
                    }
                } else if let Some(af_dict) = acroform_obj.as_dict() {
                    // AcroForm is an inline dictionary in the catalog — modify catalog
                    let mut new_af = af_dict.clone();
                    new_af.insert("NeedAppearances".to_string(), Object::Boolean(true));
                    let mut new_catalog = catalog_dict.clone();
                    new_catalog.insert("AcroForm".to_string(), Object::Dictionary(new_af));

                    // Get catalog object ID from trailer /Root reference
                    if let Some(trailer_dict) = self.source.trailer().as_dict() {
                        if let Some(root_ref) =
                            trailer_dict.get("Root").and_then(|r| r.as_reference())
                        {
                            self.modified_objects
                                .insert(root_ref.id, Object::Dictionary(new_catalog));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Find the offset of the previous xref table in the original PDF.
    fn find_prev_xref_offset(&self, bytes: &[u8]) -> Result<u64> {
        // Search backwards from the end for "startxref"
        let search = b"startxref";
        let end = bytes.len();
        let mut pos = if end >= search.len() {
            end - search.len()
        } else {
            0
        };

        while pos > 0 {
            if bytes[pos..].starts_with(search) {
                // Found it - parse the offset that follows
                let after_keyword = pos + search.len();
                let remaining = &bytes[after_keyword..];

                // Skip whitespace and parse number
                let offset_str: String = remaining
                    .iter()
                    .skip_while(|&&b| b == b' ' || b == b'\n' || b == b'\r')
                    .take_while(|&&b| b.is_ascii_digit())
                    .map(|&b| b as char)
                    .collect();

                if let Ok(offset) = offset_str.parse::<u64>() {
                    return Ok(offset);
                }
            }
            pos = pos.saturating_sub(1);
        }

        Err(Error::InvalidPdf("Could not find startxref in original PDF".to_string()))
    }

    /// Write a full rewrite of the PDF.
    #[cfg(not(target_arch = "wasm32"))]
    fn write_full(&mut self, path: impl AsRef<Path>, options: &SaveOptions) -> Result<()> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        self.write_full_to_writer(&mut writer, options)
    }

    /// Collect all object IDs reachable from the catalog root via BFS.
    ///
    /// Used by garbage collection: any source-document object not in this set is
    /// an orphan and can be omitted from the output.  Modified objects are
    /// consulted first so that references introduced by edits are honoured.
    fn collect_reachable_ids(&self) -> std::collections::HashSet<u32> {
        use std::collections::{HashSet, VecDeque};

        fn traverse(obj: &Object, queue: &mut VecDeque<u32>) {
            match obj {
                Object::Reference(r) => queue.push_back(r.id),
                Object::Array(arr) => arr.iter().for_each(|o| traverse(o, queue)),
                Object::Dictionary(d) => d.values().for_each(|o| traverse(o, queue)),
                Object::Stream { dict, .. } => dict.values().for_each(|o| traverse(o, queue)),
                _ => {},
            }
        }

        let mut reachable: HashSet<u32> = HashSet::new();
        let mut queue: VecDeque<u32> = VecDeque::new();

        if let Some(r) = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Root"))
            .and_then(|v| v.as_reference())
        {
            queue.push_back(r.id);
        }
        if let Some(r) = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Info"))
            .and_then(|v| v.as_reference())
        {
            queue.push_back(r.id);
        }

        while let Some(id) = queue.pop_front() {
            if !reachable.insert(id) {
                continue;
            }
            let obj = if let Some(m) = self.modified_objects.get(&id) {
                Some(m.clone())
            } else {
                self.source.load_object(ObjectRef { id, gen: 0 }).ok()
            };
            if let Some(obj) = obj {
                traverse(&obj, &mut queue);
            }
        }

        reachable
    }

    /// Load an object from `self.source` for copying into the output file,
    /// decrypting its stream data (if any) so it survives into an output
    /// with no `/Encrypt` dictionary of its own (#1032). A no-op beyond the
    /// plain load when the source isn't encrypted. Every `self.source.load_object`
    /// call in `write_full_to_writer` whose result can reach the output
    /// writer goes through this instead of the raw accessor — the one
    /// exception is `collect_reachable_ids`'s graph walk just above, which
    /// only inspects references and never emits bytes.
    fn load_source_object(&self, r: ObjectRef) -> Result<Object> {
        let obj = self.source.load_object(r)?;
        self.source.decrypt_stream_for_copy(obj, r)
    }

    /// Write a full rewrite of the PDF to a generic writer.
    fn write_full_to_writer(
        &mut self,
        writer: &mut (impl Write + Seek),
        options: &SaveOptions,
    ) -> Result<()> {
        use crate::encryption::{
            generate_file_id, Algorithm, EncryptDictBuilder, EncryptionWriteHandler,
        };
        use flate2::{write::ZlibEncoder, Compression};

        // Fail closed rather than silently copying unreadable ciphertext
        // (#1032): every stream object read from `self.source` below goes
        // through `load_source_object` to decrypt it, but that requires an
        // authenticated encryption handler. An encrypted-but-unauthenticated
        // source has no key to decrypt with at all — refuse rather than
        // write out raw source ciphertext with no /Encrypt dict, which
        // silently produces a structurally valid but unreadable PDF.
        if self.source.is_encrypted() && !self.source.is_authenticated() {
            return Err(Error::EncryptedPdf);
        }

        /// Compress a stream object with FlateDecode if it has no filter yet.
        fn compress_stream_if_raw(obj: Object) -> Object {
            match obj {
                Object::Stream { mut dict, data } => {
                    if dict.contains_key("Filter") {
                        return Object::Stream { dict, data };
                    }
                    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                    if std::io::Write::write_all(&mut enc, &data).is_err() {
                        return Object::Stream { dict, data };
                    }
                    match enc.finish() {
                        Ok(compressed) => {
                            dict.insert(
                                "Filter".to_string(),
                                Object::Name("FlateDecode".to_string()),
                            );
                            dict.insert(
                                "Length".to_string(),
                                Object::Integer(compressed.len() as i64),
                            );
                            Object::Stream {
                                dict,
                                data: compressed.into(),
                            }
                        },
                        Err(_) => Object::Stream { dict, data },
                    }
                },
                other => other,
            }
        }

        // Write PDF header
        let (major, minor) = self.version();
        write!(writer, "%PDF-{}.{}\n", major, minor)?;
        // Binary marker per spec (bytes > 127 to indicate binary content)
        writer.write_all(b"%\x80\x81\x82\x83\n")?;

        let serializer = ObjectSerializer::compact();

        // Set up encryption if configured
        let (file_id, encrypt_dict, encryption_handler) =
            if let Some(config) = options.encryption.as_ref() {
                let (id1, id2) = generate_file_id();

                // Convert EncryptionAlgorithm to encryption::Algorithm
                let algorithm = match config.algorithm {
                    EncryptionAlgorithm::Rc4_40 => Algorithm::RC4_40,
                    EncryptionAlgorithm::Rc4_128 => Algorithm::Rc4_128,
                    EncryptionAlgorithm::Aes128 => Algorithm::Aes128,
                    EncryptionAlgorithm::Aes256 => Algorithm::Aes256,
                };

                // Build the encryption dictionary, keeping the file
                // encryption key it wrapped into /UE and /OE.
                let (encrypt_dict, file_key) = EncryptDictBuilder::new(algorithm)
                    .user_password(config.user_password.as_bytes())
                    .owner_password(config.owner_password.as_bytes())
                    .permissions(config.permissions.to_bits())
                    .encrypt_metadata(true)
                    .build_with_key(&id1)?;

                // Create encryption handler.
                //
                // For AES-256 (R6) the key is the one just wrapped into /UE —
                // ISO 32000-2 Algorithm 8 generates exactly one, and there is
                // nothing to re-derive. Deriving separately here produced a
                // second random key, so the file authenticated and then
                // decrypted every stream to noise. For R<=4 the key *is* a
                // derivation from the password, owner hash, permissions and
                // file id, and the handler recomputes it.
                let handler = match file_key {
                    Some(key) => EncryptionWriteHandler::with_file_key(key, algorithm, true),
                    None => EncryptionWriteHandler::new(
                        config.user_password.as_bytes(),
                        &encrypt_dict.owner_password,
                        encrypt_dict.permissions,
                        &id1,
                        algorithm,
                        true,
                    )?,
                };

                (Some((id1, id2)), Some(encrypt_dict), Some(handler))
            } else {
                (None, None, None)
            };

        // Helper to serialize with or without encryption
        let serialize_obj = |s: &ObjectSerializer,
                             id: u32,
                             gen: u16,
                             obj: &Object,
                             handler: &Option<EncryptionWriteHandler>|
         -> Vec<u8> {
            if let Some(ref h) = handler {
                s.serialize_indirect_encrypted(id, gen, obj, h)
            } else {
                s.serialize_indirect(id, gen, obj)
            }
        };

        let mut xref_entries: Vec<(u32, u64, u16, bool)> = Vec::new(); // (id, offset, gen, in_use)
        let mut written_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();

        // Object 0 is always free
        xref_entries.push((0, 65535, 65535, false));

        // Collect all objects we need to write
        let mut objects_to_write: Vec<(u32, Object)> = Vec::new();

        // Get catalog and traverse to collect all referenced objects
        let catalog = self.source.catalog()?;
        let catalog_ref = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Root"))
            .and_then(|r| r.as_reference())
            .ok_or_else(|| Error::InvalidPdf("Missing catalog reference".to_string()))?;

        // For now, do a simple copy of essential objects
        // Full implementation would do complete object traversal

        // Write encryption dictionary if encrypting (must not be encrypted itself)
        let encrypt_obj_id = if let Some(ref enc_dict) = encrypt_dict {
            let enc_id = self.allocate_object_id();
            let enc_obj = enc_dict.to_object();
            let offset = writer.stream_position()?;
            // Encryption dict is NOT encrypted
            let bytes = serializer.serialize_indirect(enc_id, 0, &enc_obj);
            writer.write_all(&bytes)?;
            xref_entries.push((enc_id, offset, 0, true));
            Some(enc_id)
        } else {
            None
        };

        // Write catalog (possibly modified)
        let mut catalog_obj = self
            .modified_objects
            .get(&catalog_ref.id)
            .cloned()
            .unwrap_or(catalog);

        // Remove or rebuild AcroForm after form flattening
        if self.remove_acroform {
            // All pages flattened — drop the entire AcroForm
            if let Some(catalog_dict) = catalog_obj.as_dict() {
                let mut new_catalog = catalog_dict.clone();
                new_catalog.remove("AcroForm");
                catalog_obj = Object::Dictionary(new_catalog);
            }
        } else if !self.flatten_forms_pages.is_empty() {
            // Partial flatten — rebuild AcroForm keeping only fields whose widgets
            // remain on non-flattened pages (ISO 32000-1 §12.7.2).
            if let Some(catalog_dict) = catalog_obj.as_dict() {
                if let Some(rebuilt) = self.rebuild_partial_acroform(catalog_dict)? {
                    let mut new_catalog = catalog_dict.clone();
                    new_catalog.insert("AcroForm".to_string(), rebuilt);
                    catalog_obj = Object::Dictionary(new_catalog);
                }
            }
        }

        // Pre-allocate form field IDs and build AcroForm if we have form field changes
        // Stores: (page_index, object_id, wrapper, is_root_field)
        let mut all_form_field_data: Vec<(usize, u32, FormFieldWrapper, bool)> = Vec::new();
        // Map field name -> allocated ObjectRef (for parent/child linking)
        let mut field_name_to_ref: HashMap<String, ObjectRef> = HashMap::new();

        if self.acroform_modified && !self.remove_acroform {
            // Existing (already-in-document) modified fields are updated IN
            // PLACE: load the original field/widget object, override /V (and
            // /AS for buttons), and stage it in `modified_objects` so the graph
            // copy re-emits it with its full dictionary intact. This is
            // essential for a *merged* field+widget — where the field dict IS
            // the widget annotation (ISO 32000-1 §12.7.4.1): allocating a fresh
            // object for it instead would strand a bare `<< /FT /V >>` (no
            // /T, /Subtype, /Rect) and leave the real widget with an empty
            // value. Also sets /NeedAppearances on the existing AcroForm.
            self.flush_form_fields_to_modified_objects()?;

            // #647: for an *inline* AcroForm the flush above sets
            // /NeedAppearances true by re-writing the catalog into
            // `modified_objects`, but `catalog_obj` was snapshotted before the
            // flush — so without this the patch is silently dropped and viewers
            // honour the field's pre-existing empty /AP /N over /DA, rendering
            // the field blank (ISO 32000-1 §12.7.3.3, Table 226 — AP takes
            // precedence over DA). Re-adopt the patched catalog. Guarded to the
            // pure-fill case: the flatten branches above already rebuild the
            // catalog's AcroForm in-place (and must not be clobbered), and the
            // new-fields branch below emits an AcroForm with NeedAppearances via
            // AcroFormBuilder.
            if !self.remove_acroform && self.flatten_forms_pages.is_empty() {
                if let Some(patched) = self.modified_objects.get(&catalog_ref.id) {
                    catalog_obj = patched.clone();
                }
            }

            // Only genuinely NEW fields need freshly-allocated objects plus
            // /Fields and /Annots entries. Existing fields keep their object
            // ids and their place in /Fields/Annots (updated in place above).
            let mut all_wrappers: Vec<_> = self
                .modified_form_fields
                .values()
                .filter(|w| w.is_new())
                .cloned()
                .collect();

            // Sort: parent-only fields first, then terminal fields
            // This ensures parents get IDs before children that reference them
            all_wrappers.sort_by(|a, b| {
                let a_parent = a.is_parent_only();
                let b_parent = b.is_parent_only();
                // Parents first, then by name for deterministic ordering
                match (a_parent, b_parent) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name().cmp(b.name()),
                }
            });

            // First pass: allocate IDs for all fields
            for wrapper in &all_wrappers {
                let field_id = self.allocate_object_id();
                let field_ref = ObjectRef::new(field_id, 0);
                field_name_to_ref.insert(wrapper.name().to_string(), field_ref);
            }

            // Second pass: build field data with parent/child references resolved
            for mut wrapper in all_wrappers {
                let field_id = field_name_to_ref
                    .get(wrapper.name())
                    .map(|r| r.id)
                    .unwrap_or_else(|| self.allocate_object_id());

                // Set parent reference if this is a child field
                if let Some(parent_name) = wrapper.parent_name() {
                    if let Some(&parent_ref) = field_name_to_ref.get(parent_name) {
                        wrapper.set_parent_ref(parent_ref);
                    }
                }

                // Determine if this is a root field (no parent, goes in AcroForm /Fields)
                let is_root = wrapper.parent_name().is_none();

                all_form_field_data.push((wrapper.page_index(), field_id, wrapper, is_root));
            }

            // Update parent wrappers with child references
            // Build a map of parent -> children
            let mut parent_children: HashMap<String, Vec<ObjectRef>> = HashMap::new();
            for (_, field_id, wrapper, _) in &all_form_field_data {
                if let Some(parent_name) = wrapper.parent_name() {
                    parent_children
                        .entry(parent_name.to_string())
                        .or_default()
                        .push(ObjectRef::new(*field_id, 0));
                }
            }

            // Add child refs to parent wrappers
            for (_, _, wrapper, _) in &mut all_form_field_data {
                if let Some(children) = parent_children.get(wrapper.name()) {
                    for &child_ref in children {
                        wrapper.add_child_ref(child_ref);
                    }
                }
            }

            // Build AcroForm dictionary if we have fields
            if !all_form_field_data.is_empty() {
                use crate::writer::AcroFormBuilder;

                let mut acroform_builder = AcroFormBuilder::new();

                // Preserve the existing AcroForm /Fields entries — those fields
                // keep their object ids and were updated in place above, so they
                // must stay referenced (otherwise a fill that also adds a new
                // field would drop every pre-existing field from /Fields).
                if let Ok(cat) = self.source.catalog() {
                    if let Some(af_ref) = cat
                        .as_dict()
                        .and_then(|d| d.get("AcroForm"))
                        .and_then(|o| o.as_reference())
                    {
                        if let Ok(af) = self.load_source_object(af_ref) {
                            if let Some(orig_fields) = af
                                .as_dict()
                                .and_then(|d| d.get("Fields"))
                                .and_then(|o| o.as_array())
                            {
                                for r in orig_fields {
                                    if let Some(rf) = r.as_reference() {
                                        acroform_builder.add_field(rf);
                                    }
                                }
                            }
                        }
                    }
                }

                // Add ROOT new fields (no parent) to AcroForm's /Fields array
                for (_, field_id, _, is_root) in &all_form_field_data {
                    if *is_root {
                        acroform_builder.add_field(ObjectRef::new(*field_id, 0));
                    }
                }

                // Build AcroForm dictionary with embedded resources
                let acroform_dict = acroform_builder.build_with_resources();

                // Update catalog to include AcroForm
                if let Some(catalog_dict) = catalog_obj.as_dict() {
                    let mut new_catalog = catalog_dict.clone();
                    new_catalog.insert("AcroForm".to_string(), Object::Dictionary(acroform_dict));
                    catalog_obj = Object::Dictionary(new_catalog);
                }
            }
        }

        // Write embedded files and update catalog if any files are pending
        let mut embedded_file_refs: Vec<(String, ObjectRef)> = Vec::new();
        let embedded_files = std::mem::take(&mut self.embedded_files);
        if !embedded_files.is_empty() {
            for file in &embedded_files {
                // Allocate IDs for embedded file stream and filespec
                let stream_id = self.allocate_object_id();
                let filespec_id = self.allocate_object_id();

                // Build and write embedded file stream
                let stream_dict = file.build_stream_dict();
                let stream_obj = Object::Stream {
                    dict: stream_dict,
                    data: file.data.clone().into(),
                };
                let offset = writer.stream_position()?;
                let bytes =
                    serialize_obj(&serializer, stream_id, 0, &stream_obj, &encryption_handler);
                writer.write_all(&bytes)?;
                xref_entries.push((stream_id, offset, 0, true));

                // Build and write filespec dictionary
                let stream_ref = ObjectRef {
                    id: stream_id,
                    gen: 0,
                };
                let filespec_dict = file.build_filespec(stream_ref);
                let filespec_obj = Object::Dictionary(filespec_dict);
                let offset = writer.stream_position()?;
                let bytes =
                    serialize_obj(&serializer, filespec_id, 0, &filespec_obj, &encryption_handler);
                writer.write_all(&bytes)?;
                xref_entries.push((filespec_id, offset, 0, true));

                embedded_file_refs.push((
                    file.name.clone(),
                    ObjectRef {
                        id: filespec_id,
                        gen: 0,
                    },
                ));
            }

            // Update catalog with Names/EmbeddedFiles
            if let Some(catalog_dict) = catalog_obj.as_dict() {
                let mut new_catalog = catalog_dict.clone();

                // Build EmbeddedFiles name tree
                let mut names_array = Vec::new();
                // Sort by name for proper name tree ordering
                let mut sorted_refs = embedded_file_refs.clone();
                sorted_refs.sort_by(|a, b| a.0.cmp(&b.0));
                for (name, ref_) in sorted_refs {
                    names_array.push(Object::String(name.as_bytes().to_vec()));
                    names_array.push(Object::Reference(ref_));
                }

                let mut embedded_files_dict = HashMap::new();
                embedded_files_dict.insert("Names".to_string(), Object::Array(names_array));

                // Get or create Names dictionary
                let mut names_dict = match new_catalog.get("Names") {
                    Some(Object::Dictionary(d)) => d.clone(),
                    _ => HashMap::new(),
                };
                names_dict
                    .insert("EmbeddedFiles".to_string(), Object::Dictionary(embedded_files_dict));
                new_catalog.insert("Names".to_string(), Object::Dictionary(names_dict));

                catalog_obj = Object::Dictionary(new_catalog);
            }
        }

        let offset = writer.stream_position()?;
        let bytes =
            serialize_obj(&serializer, catalog_ref.id, 0, &catalog_obj, &encryption_handler);
        writer.write_all(&bytes)?;
        xref_entries.push((catalog_ref.id, offset, 0, true));

        // Pre-allocate IDs for merged pages so we can include them in the Pages tree
        let merged_page_count = self.merged_pages.len();
        let mut merged_page_ids: Vec<u32> = Vec::with_capacity(merged_page_count);
        for _ in 0..merged_page_count {
            merged_page_ids.push(self.allocate_object_id());
        }

        // Get and write pages tree
        if let Some(catalog_dict) = catalog_obj.as_dict() {
            if let Some(pages_ref) = catalog_dict.get("Pages").and_then(|p| p.as_reference()) {
                let pages_obj = self.load_source_object(pages_ref)?;

                // Rebuild Pages tree: filter by page_order, reorder, append merged pages
                let final_pages_obj = if let Some(pages_dict) = pages_obj.as_dict() {
                    let mut new_pages_dict = pages_dict.clone();

                    // Collect flattened leaf page refs from the (possibly multi-level) page tree.
                    // page_order indices refer to leaf pages, not Kids array entries.
                    // Single tree walk — calling get_page_ref(i) in a loop is O(n²).
                    let original_page_refs: Vec<ObjectRef> =
                        self.source.all_page_refs().unwrap_or_default();

                    // Build visible kids in page_order sequence
                    // page_order contains original leaf-page indices; -1 means removed
                    let mut kids: Vec<Object> = Vec::new();
                    for &order in &self.page_order {
                        if order >= 0 {
                            let idx = order as usize;
                            if idx < original_page_refs.len() {
                                kids.push(Object::Reference(original_page_refs[idx]));
                            }
                        }
                    }

                    // Append merged page refs
                    for &page_id in &merged_page_ids {
                        kids.push(Object::Reference(ObjectRef::new(page_id, 0)));
                    }

                    new_pages_dict.insert("Kids".to_string(), Object::Array(kids.clone()));
                    new_pages_dict.insert("Count".to_string(), Object::Integer(kids.len() as i64));

                    Object::Dictionary(new_pages_dict)
                } else {
                    pages_obj.clone()
                };

                let offset = writer.stream_position()?;
                let bytes = serialize_obj(
                    &serializer,
                    pages_ref.id,
                    0,
                    &final_pages_obj,
                    &encryption_handler,
                );
                writer.write_all(&bytes)?;
                xref_entries.push((pages_ref.id, offset, 0, true));

                // Write individual pages (use final_pages_obj which includes merged pages)
                if let Some(pages_dict) = final_pages_obj.as_dict() {
                    if let Some(kids) = pages_dict.get("Kids").and_then(|k| k.as_array()) {
                        // Build a mapping: output loop position → original source page index.
                        // After select_pages() the page_order may differ from 0..n, so all
                        // per-page HashMaps (modified_content, modified_annotations, …) must
                        // be keyed by the original source index, not the output position.
                        let source_indices: Vec<usize> = self
                            .page_order
                            .iter()
                            .filter(|&&i| i >= 0)
                            .map(|&i| i as usize)
                            .collect();
                        let source_page_count = source_indices.len();

                        let mut page_index = 0;
                        for kid in kids {
                            if let Some(page_ref) = kid.as_reference() {
                                let page_obj = self.load_source_object(page_ref)?;

                                // Resolve the original source page index for all HashMap lookups.
                                // Merged pages (appended after source pages) use the loop counter.
                                let source_page_index = if page_index < source_page_count {
                                    source_indices[page_index]
                                } else {
                                    page_index
                                };

                                // Check if we have erase overlays for this page
                                let has_erase_overlay =
                                    self.erase_regions.contains_key(&source_page_index);
                                let erase_overlay_id = if has_erase_overlay {
                                    Some(self.allocate_object_id())
                                } else {
                                    None
                                };

                                // Check if we have new annotations to add for this page
                                let new_annotation_count = self
                                    .modified_annotations
                                    .get(&source_page_index)
                                    .map(|anns| anns.iter().filter(|a| a.is_new()).count())
                                    .unwrap_or(0);
                                let new_annotation_ids: Vec<u32> = (0..new_annotation_count)
                                    .map(|_| self.allocate_object_id())
                                    .collect();

                                // Get pre-allocated form field data for this page
                                // Only include terminal fields (not parent-only) that have widgets
                                let page_form_fields: Vec<(u32, FormFieldWrapper)> =
                                    all_form_field_data
                                        .iter()
                                        .filter(|(pg_idx, _, wrapper, _)| {
                                            *pg_idx == source_page_index
                                                && !wrapper.is_parent_only()
                                        })
                                        .map(|(_, id, wrapper, _)| (*id, wrapper.clone()))
                                        .collect();
                                let new_form_field_ids: Vec<u32> =
                                    page_form_fields.iter().map(|(id, _)| *id).collect();
                                let new_form_field_wrappers: Vec<FormFieldWrapper> =
                                    page_form_fields.iter().map(|(_, w)| w.clone()).collect();

                                // Check if we need to flatten annotations for this page
                                let should_flatten =
                                    self.flatten_annotations_pages.contains(&source_page_index);
                                let flatten_data: Option<(
                                    Vec<AnnotationAppearance>,
                                    u32,
                                    Vec<(u32, String)>,
                                )> = if should_flatten {
                                    // Get annotation appearances
                                    let appearances =
                                        self.get_annotation_appearances(source_page_index)?;
                                    if !appearances.is_empty() {
                                        // Allocate object IDs for each XObject and one for the overlay
                                        let overlay_id = self.allocate_object_id();
                                        let xobj_ids: Vec<(u32, String)> = appearances
                                            .iter()
                                            .enumerate()
                                            .map(|(i, _)| {
                                                let id = self.allocate_object_id();
                                                let name = format!("FlatAnnot{}", i);
                                                (id, name)
                                            })
                                            .collect();
                                        Some((appearances, overlay_id, xobj_ids))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Check if we need to apply redactions for this page
                                let should_apply_redactions =
                                    self.apply_redactions_pages.contains(&source_page_index);
                                // Destructive redaction (#231): pre-computed
                                // replacement content for this page. Takes
                                // precedence over the legacy cosmetic overlay
                                // path; the bytes already include the opaque
                                // overlay and the secret is physically gone.
                                let destructive_redaction: Option<(Vec<u8>, u32)> = self
                                    .redacted_content
                                    .get(&source_page_index)
                                    .cloned()
                                    .map(|bytes| (bytes, self.allocate_object_id()));

                                let redaction_data: Option<(Vec<RedactionData>, u32)> =
                                    if destructive_redaction.is_some() {
                                        // Destructive path owns Contents/Annots; skip
                                        // the cosmetic overlay entirely.
                                        None
                                    } else if should_apply_redactions {
                                        let redactions =
                                            self.get_redaction_data(source_page_index)?;
                                        if !redactions.is_empty() {
                                            let overlay_id = self.allocate_object_id();
                                            Some((redactions, overlay_id))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                // Check if we need to flatten form fields for this page
                                let should_flatten_forms =
                                    self.flatten_forms_pages.contains(&source_page_index);
                                let form_flatten_data: Option<(
                                    Vec<AnnotationAppearance>,
                                    u32,
                                    Vec<(u32, String)>,
                                )> = if should_flatten_forms {
                                    let appearances =
                                        self.get_widget_appearances(source_page_index)?;
                                    if !appearances.is_empty() {
                                        let overlay_id = self.allocate_object_id();
                                        let xobj_ids: Vec<(u32, String)> = appearances
                                            .iter()
                                            .enumerate()
                                            .map(|(i, _)| {
                                                let id = self.allocate_object_id();
                                                let name = format!("FlatForm{}", i);
                                                (id, name)
                                            })
                                            .collect();
                                        Some((appearances, overlay_id, xobj_ids))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Check if we have modified content for this page
                                let modified_content_id: Option<u32> = if self.structure_modified
                                    && self.modified_content.contains_key(&source_page_index)
                                {
                                    Some(self.allocate_object_id())
                                } else {
                                    None
                                };

                                // Check if we have overlay additions for this page
                                let overlay_additions_id: Option<u32> =
                                    if self.overlay_additions.contains_key(&source_page_index) {
                                        Some(self.allocate_object_id())
                                    } else {
                                        None
                                    };

                                // Every Standard-14 base-font name that the overlay content
                                // stream will actually emit via `Tf` needs a matching Font
                                // object registered in this page's `/Resources/Font`, else the
                                // resource name is dangling.
                                //
                                // The name to register is NOT the raw `font.name` the caller
                                // passed — it's the transformed name the writer emits, i.e.
                                // `map_base14_font_name(&font.name, is_bold)` (the same source
                                // of truth used by `ContentStreamBuilder::map_font_name`). Keying
                                // off the raw name diverges from the emitted `Tf` name for:
                                //   - bold/italic overlays (raw "Helvetica" + bold → emits
                                //     "/Helvetica-Bold");
                                //   - generic families ("Arial", "sans-serif" → "/Helvetica");
                                //   - Symbol/ZapfDingbats (routed to "/Helvetica").
                                // Registering the emitted name closes all three; only Standard-14
                                // names are ever emitted here (embedded fonts take the
                                // ShowEmbeddedText path and never reach this registration).
                                let mut overlay_font_ids: HashMap<String, u32> = HashMap::new();
                                if let Some(added) = self.overlay_additions.get(&source_page_index)
                                {
                                    let mut needed: Vec<String> = added
                                        .iter()
                                        .filter_map(|el| match el {
                                            ContentElement::Text(t) => {
                                                Some(crate::writer::map_base14_font_name(
                                                    &t.font.name,
                                                    t.is_bold(),
                                                ))
                                            },
                                            _ => None,
                                        })
                                        .collect();
                                    needed.sort();
                                    needed.dedup();
                                    for name in needed {
                                        overlay_font_ids.insert(name, self.allocate_object_id());
                                    }
                                }

                                // Apply page property modifications if any
                                let mut final_page_obj = if let Some(props) =
                                    self.modified_page_props.get(&source_page_index)
                                {
                                    self.apply_page_props_to_object(&page_obj, props)?
                                } else {
                                    page_obj.clone()
                                };

                                // If we have an erase overlay, update Contents to include it
                                if let (Some(overlay_obj_id), Some(page_dict)) =
                                    (erase_overlay_id, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();
                                    // Get existing Contents reference
                                    if let Some(contents) = new_dict.get("Contents").cloned() {
                                        // Create an array with original content + overlay
                                        let overlay_ref =
                                            Object::Reference(ObjectRef::new(overlay_obj_id, 0));
                                        let contents_array = match contents {
                                            Object::Reference(_) => {
                                                Object::Array(vec![contents, overlay_ref])
                                            },
                                            Object::Array(mut arr) => {
                                                arr.push(overlay_ref);
                                                Object::Array(arr)
                                            },
                                            _ => Object::Array(vec![contents, overlay_ref]),
                                        };
                                        new_dict.insert("Contents".to_string(), contents_array);
                                    }
                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // If we have overlay additions (add_text on existing page), append
                                // a new stream after the original content rather than replacing it.
                                if let (Some(additions_id), Some(page_dict)) =
                                    (overlay_additions_id, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();
                                    if let Some(contents) = new_dict.get("Contents").cloned() {
                                        let additions_ref =
                                            Object::Reference(ObjectRef::new(additions_id, 0));
                                        let contents_array = match contents {
                                            Object::Reference(_) => {
                                                Object::Array(vec![contents, additions_ref])
                                            },
                                            Object::Array(mut arr) => {
                                                arr.push(additions_ref);
                                                Object::Array(arr)
                                            },
                                            _ => Object::Array(vec![contents, additions_ref]),
                                        };
                                        new_dict.insert("Contents".to_string(), contents_array);
                                    }
                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // Register any Base-14 fonts used by overlay_additions text
                                // into this page's /Resources/Font, so the `Tf` operator in the
                                // overlay stream resolves to a real font object instead of an
                                // undeclared resource name. /Resources (and /Resources/Font)
                                // are very commonly *indirect references* on a page loaded from
                                // an existing document, not inline dictionaries — both levels
                                // must be resolved via `self.source.load_object()` or the
                                // existing Font entries (e.g. /F1, /F3 used by the page's
                                // original content) are lost when /Resources is rebuilt below.
                                if !overlay_font_ids.is_empty() {
                                    if let Some(page_dict) = final_page_obj.as_dict() {
                                        let mut new_dict = page_dict.clone();

                                        let resolve_dict = |obj: &Object,
                                                             src: &PdfDocument|
                                         -> HashMap<String, Object> {
                                            match obj {
                                                Object::Dictionary(d) => d.clone(),
                                                Object::Reference(r) => src
                                                    .load_object(*r)
                                                    .ok()
                                                    .and_then(|o| o.as_dict().cloned())
                                                    .unwrap_or_default(),
                                                _ => HashMap::new(),
                                            }
                                        };

                                        // /Resources is an inheritable attribute (ISO 32000-1
                                        // §7.7.3.4): a page may legally omit its own /Resources
                                        // and inherit it from an ancestor in the /Pages chain.
                                        // Writing a fresh font-only /Resources here would shadow
                                        // that inherited dict and drop the page's real fonts, so
                                        // when the page has no own /Resources we seed from the
                                        // inherited one.
                                        //
                                        // SECURITY (destructive redaction): we seed the page's
                                        // materialised /Resources with ONLY the inherited /Font
                                        // sub-dictionary, never the whole inherited dict. An
                                        // ancestor /Resources can carry /XObject, /Pattern, etc.
                                        // that reference content-bearing streams unrelated to
                                        // this page; copying them wholesale onto the page would
                                        // make them freshly reachable and thus survive the
                                        // orphan-drop backstop — re-introducing exactly the kind
                                        // of stray reference destructive redaction must not
                                        // resurrect. /Font is the only sub-dict the overlay `Tf`
                                        // resolution needs, and font programs are not a redaction
                                        // leak surface here (the secret lives in content streams,
                                        // which are rewritten). A page that genuinely renders an
                                        // inherited XObject still inherits it at render time via
                                        // the /Pages chain — we are not removing that path, only
                                        // declining to inline-copy it onto the page.
                                        let mut resources_dict: HashMap<String, Object> =
                                            match new_dict.get("Resources") {
                                                Some(obj) => resolve_dict(obj, &self.source),
                                                None => {
                                                    let mut node = new_dict
                                                        .get("Parent")
                                                        .and_then(|p| p.as_reference())
                                                        .and_then(|r| {
                                                            self.load_source_object(r).ok()
                                                        });
                                                    let mut inherited_font: Option<Object> = None;
                                                    let mut guard = 0;
                                                    while let Some(obj) = node {
                                                        guard += 1;
                                                        if guard > 64 {
                                                            break; // cyclic /Parent — bail
                                                        }
                                                        let Some(dict) = obj.as_dict() else {
                                                            break;
                                                        };
                                                        if let Some(res) = dict.get("Resources") {
                                                            // Found the inherited /Resources:
                                                            // take ONLY its /Font sub-dict.
                                                            let res_dict =
                                                                resolve_dict(res, &self.source);
                                                            inherited_font =
                                                                res_dict.get("Font").cloned();
                                                            break;
                                                        }
                                                        node = dict
                                                            .get("Parent")
                                                            .and_then(|p| p.as_reference())
                                                            .and_then(|r| {
                                                                self.load_source_object(r).ok()
                                                            });
                                                    }
                                                    let mut seeded: HashMap<String, Object> =
                                                        HashMap::new();
                                                    if let Some(font) = inherited_font {
                                                        seeded.insert("Font".to_string(), font);
                                                    }
                                                    seeded
                                                },
                                            };
                                        let mut font_dict: HashMap<String, Object> =
                                            match resources_dict.get("Font") {
                                                Some(obj) => resolve_dict(obj, &self.source),
                                                None => HashMap::new(),
                                            };
                                        for (name, id) in &overlay_font_ids {
                                            font_dict.insert(
                                                name.clone(),
                                                Object::Reference(ObjectRef::new(*id, 0)),
                                            );
                                        }
                                        resources_dict.insert(
                                            "Font".to_string(),
                                            Object::Dictionary(font_dict),
                                        );
                                        new_dict.insert(
                                            "Resources".to_string(),
                                            Object::Dictionary(resources_dict),
                                        );
                                        final_page_obj = Object::Dictionary(new_dict);
                                    }
                                }

                                // If we're flattening annotations, update page dictionary
                                if let (
                                    Some((ref appearances, flatten_overlay_id, ref xobj_ids)),
                                    Some(page_dict),
                                ) = (&flatten_data, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();

                                    // Add flatten overlay to Contents
                                    if let Some(contents) = new_dict.get("Contents").cloned() {
                                        let overlay_ref = Object::Reference(ObjectRef::new(
                                            *flatten_overlay_id,
                                            0,
                                        ));
                                        let contents_array = match contents {
                                            Object::Reference(_) => {
                                                Object::Array(vec![contents, overlay_ref])
                                            },
                                            Object::Array(mut arr) => {
                                                arr.push(overlay_ref);
                                                Object::Array(arr)
                                            },
                                            _ => Object::Array(vec![contents, overlay_ref]),
                                        };
                                        new_dict.insert("Contents".to_string(), contents_array);
                                    }

                                    // Add XObjects to Resources
                                    let resources = new_dict.get("Resources").cloned();
                                    let mut resources_dict = match resources {
                                        Some(Object::Dictionary(d)) => d,
                                        Some(Object::Reference(res_ref)) => {
                                            match self.load_source_object(res_ref) {
                                                Ok(Object::Dictionary(d)) => d,
                                                _ => HashMap::new(),
                                            }
                                        },
                                        _ => HashMap::new(),
                                    };

                                    // Get or create XObject subdictionary
                                    let mut xobject_dict = match resources_dict.get("XObject") {
                                        Some(Object::Dictionary(d)) => d.clone(),
                                        Some(Object::Reference(xobj_ref)) => {
                                            match self.load_source_object(*xobj_ref) {
                                                Ok(Object::Dictionary(d)) => d,
                                                _ => HashMap::new(),
                                            }
                                        },
                                        _ => HashMap::new(),
                                    };

                                    // Add our flattened annotation XObjects
                                    for (obj_id, name) in xobj_ids {
                                        xobject_dict.insert(
                                            name.clone(),
                                            Object::Reference(ObjectRef::new(*obj_id, 0)),
                                        );
                                    }

                                    resources_dict.insert(
                                        "XObject".to_string(),
                                        Object::Dictionary(xobject_dict),
                                    );
                                    new_dict.insert(
                                        "Resources".to_string(),
                                        Object::Dictionary(resources_dict),
                                    );

                                    // Remove /Annots array
                                    new_dict.remove("Annots");

                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // If we're applying redactions, update page dictionary
                                if let (
                                    Some((ref redactions, redact_overlay_id)),
                                    Some(page_dict),
                                ) = (&redaction_data, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();

                                    // Add redaction overlay to Contents
                                    if let Some(contents) = new_dict.get("Contents").cloned() {
                                        let overlay_ref = Object::Reference(ObjectRef::new(
                                            *redact_overlay_id,
                                            0,
                                        ));
                                        let contents_array = match contents {
                                            Object::Reference(_) => {
                                                Object::Array(vec![contents, overlay_ref])
                                            },
                                            Object::Array(mut arr) => {
                                                arr.push(overlay_ref);
                                                Object::Array(arr)
                                            },
                                            _ => Object::Array(vec![contents, overlay_ref]),
                                        };
                                        new_dict.insert("Contents".to_string(), contents_array);
                                    }

                                    // Remove Redact annotations from /Annots array
                                    // For now, we remove the entire /Annots array when applying redactions
                                    // A more sophisticated implementation would only remove Redact subtypes
                                    new_dict.remove("Annots");

                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // Destructive redaction (#231): REPLACE the
                                // page /Contents with the single rewritten
                                // stream (secret physically removed + opaque
                                // overlay). The old content object becomes
                                // unreferenced and is dropped by the
                                // garbage-collected full rewrite (no residual
                                // recoverable bytes, G6). /Redact annotations
                                // are removed (ISO 32000-1 §12.5.6.23).
                                if let (Some((_, redacted_id)), Some(page_dict)) =
                                    (&destructive_redaction, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();
                                    let redacted_ref =
                                        Object::Reference(ObjectRef::new(*redacted_id, 0));
                                    // If overlay-additions already appended a second stream to
                                    // Contents earlier in this save (DOM edits to a source page
                                    // via `save_page`, #940), keep that addition but replace
                                    // *all* original content references with the single redacted
                                    // stream — not just entry 0. `redacted_ref` already carries
                                    // the fully redacted, concatenated content of every original
                                    // stream (`get_page_content_bytes` concatenates a multi-entry
                                    // `/Contents` array before redacting it), so any other
                                    // original entries left in the array would be pure
                                    // duplicates, and every one of them is dropped from the
                                    // output as an orphan (`redacted_orphan_ids`) regardless —
                                    // leaving them referenced here produces a dangling
                                    // `/Contents` entry that strict readers reject (#799, #941).
                                    // We know the addition's object id directly, so there's no
                                    // need to infer it positionally from the array.
                                    let contents_value = match overlay_additions_id {
                                        Some(additions_id) => Object::Array(vec![
                                            redacted_ref,
                                            Object::Reference(ObjectRef::new(additions_id, 0)),
                                        ]),
                                        None => redacted_ref,
                                    };
                                    new_dict.insert("Contents".to_string(), contents_value);
                                    new_dict.remove("Annots");
                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // If we're flattening form fields, update page dictionary
                                if let (
                                    Some((
                                        ref form_appearances,
                                        form_overlay_id,
                                        ref form_xobj_ids,
                                    )),
                                    Some(page_dict),
                                ) = (&form_flatten_data, final_page_obj.as_dict())
                                {
                                    let mut new_dict = page_dict.clone();

                                    // Add form flatten overlay to Contents
                                    if let Some(contents) = new_dict.get("Contents").cloned() {
                                        let overlay_ref =
                                            Object::Reference(ObjectRef::new(*form_overlay_id, 0));
                                        let contents_array = match contents {
                                            Object::Reference(_) => {
                                                Object::Array(vec![contents, overlay_ref])
                                            },
                                            Object::Array(mut arr) => {
                                                arr.push(overlay_ref);
                                                Object::Array(arr)
                                            },
                                            _ => Object::Array(vec![contents, overlay_ref]),
                                        };
                                        new_dict.insert("Contents".to_string(), contents_array);
                                    }

                                    // Add XObjects to Resources
                                    let resources = new_dict.get("Resources").cloned();
                                    let mut resources_dict = match resources {
                                        Some(Object::Dictionary(d)) => d,
                                        Some(Object::Reference(res_ref)) => {
                                            match self.load_source_object(res_ref) {
                                                Ok(Object::Dictionary(d)) => d,
                                                _ => HashMap::new(),
                                            }
                                        },
                                        _ => HashMap::new(),
                                    };

                                    // Get or create XObject subdictionary
                                    let mut xobject_dict = match resources_dict.get("XObject") {
                                        Some(Object::Dictionary(d)) => d.clone(),
                                        Some(Object::Reference(xobj_ref)) => {
                                            match self.load_source_object(*xobj_ref) {
                                                Ok(Object::Dictionary(d)) => d,
                                                _ => HashMap::new(),
                                            }
                                        },
                                        _ => HashMap::new(),
                                    };

                                    // Add flattened form XObjects
                                    for (obj_id, name) in form_xobj_ids {
                                        xobject_dict.insert(
                                            name.clone(),
                                            Object::Reference(ObjectRef::new(*obj_id, 0)),
                                        );
                                    }

                                    resources_dict.insert(
                                        "XObject".to_string(),
                                        Object::Dictionary(xobject_dict),
                                    );
                                    new_dict.insert(
                                        "Resources".to_string(),
                                        Object::Dictionary(resources_dict),
                                    );

                                    // Remove Widget annotations from /Annots array, preserving others
                                    if let Some(annots) = new_dict.get("Annots").cloned() {
                                        let annots_array = match annots {
                                            Object::Array(arr) => arr,
                                            Object::Reference(annots_ref) => {
                                                match self.load_source_object(annots_ref) {
                                                    Ok(Object::Array(arr)) => arr,
                                                    _ => vec![],
                                                }
                                            },
                                            _ => vec![],
                                        };

                                        // Filter out Widget annotations
                                        let mut filtered_annots = Vec::new();
                                        for annot_ref in annots_array {
                                            if let Some(ref_obj) = annot_ref.as_reference() {
                                                if let Ok(annot_obj) =
                                                    self.load_source_object(ref_obj)
                                                {
                                                    if let Some(annot_dict) = annot_obj.as_dict() {
                                                        let subtype = annot_dict
                                                            .get("Subtype")
                                                            .and_then(|s| s.as_name());
                                                        if subtype != Some("Widget") {
                                                            // Keep non-Widget annotations
                                                            filtered_annots.push(annot_ref);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if filtered_annots.is_empty() {
                                            // All annotations were widgets, remove Annots entirely
                                            new_dict.remove("Annots");
                                        } else {
                                            // Keep remaining annotations
                                            new_dict.insert(
                                                "Annots".to_string(),
                                                Object::Array(filtered_annots),
                                            );
                                        }
                                    }

                                    final_page_obj = Object::Dictionary(new_dict);
                                }

                                // Add new annotations and form fields to the page's /Annots array
                                if !new_annotation_ids.is_empty() || !new_form_field_ids.is_empty()
                                {
                                    if let Some(page_dict) = final_page_obj.as_dict() {
                                        let mut new_dict = page_dict.clone();

                                        // Get existing Annots array or create new one
                                        let mut annots_array = match new_dict.get("Annots").cloned()
                                        {
                                            Some(Object::Array(arr)) => arr,
                                            Some(Object::Reference(annots_ref)) => {
                                                match self.load_source_object(annots_ref) {
                                                    Ok(Object::Array(arr)) => arr,
                                                    _ => vec![],
                                                }
                                            },
                                            _ => vec![],
                                        };

                                        // Add references to new annotations
                                        for annot_id in &new_annotation_ids {
                                            annots_array.push(Object::Reference(ObjectRef::new(
                                                *annot_id, 0,
                                            )));
                                        }

                                        // Add references to new form fields (widget annotations)
                                        for field_id in &new_form_field_ids {
                                            annots_array.push(Object::Reference(ObjectRef::new(
                                                *field_id, 0,
                                            )));
                                        }

                                        new_dict.insert(
                                            "Annots".to_string(),
                                            Object::Array(annots_array),
                                        );
                                        final_page_obj = Object::Dictionary(new_dict);
                                    }
                                }

                                // Update page's /Contents reference if we have modified content
                                if let Some(new_content_id) = modified_content_id {
                                    if let Some(page_dict) = final_page_obj.as_dict() {
                                        let mut new_dict = page_dict.clone();
                                        // Replace the Contents reference with the new content stream
                                        new_dict.insert(
                                            "Contents".to_string(),
                                            Object::Reference(ObjectRef::new(new_content_id, 0)),
                                        );
                                        final_page_obj = Object::Dictionary(new_dict);
                                    }
                                }

                                let offset = writer.stream_position()?;
                                let bytes = serialize_obj(
                                    &serializer,
                                    page_ref.id,
                                    0,
                                    &final_page_obj,
                                    &encryption_handler,
                                );
                                writer.write_all(&bytes)?;
                                xref_entries.push((page_ref.id, offset, 0, true));

                                // Collect new XObject refs from content generation (for Resources update)
                                let mut new_xobject_refs: Vec<(String, ObjectRef)> = Vec::new();

                                // Write page contents if present
                                if let Some(page_dict) = page_obj.as_dict() {
                                    // Check if this page has modified content (structure rebuild)
                                    if self.structure_modified
                                        && self.modified_content.contains_key(&source_page_index)
                                    {
                                        // Generate new content stream from modified StructureElement
                                        if let Some(structure) =
                                            self.modified_content.get(&source_page_index)
                                        {
                                            let (content_bytes, pending_images) =
                                                self.generate_content_stream(structure)?;

                                            // Create XObject entries for pending images
                                            let mut xobject_refs: Vec<(String, ObjectRef)> =
                                                Vec::new();
                                            for pending_image in pending_images {
                                                let xobj_id = self.allocate_object_id();

                                                // Build XObject stream for the image
                                                let xobj_stream =
                                                    Self::build_image_xobject(&pending_image.image);
                                                let offset = writer.stream_position()?;
                                                let bytes = serialize_obj(
                                                    &serializer,
                                                    xobj_id,
                                                    0,
                                                    &xobj_stream,
                                                    &encryption_handler,
                                                );
                                                writer.write_all(&bytes)?;
                                                xref_entries.push((xobj_id, offset, 0, true));

                                                xobject_refs.push((
                                                    pending_image.resource_id,
                                                    ObjectRef::new(xobj_id, 0),
                                                ));
                                            }

                                            // Create stream object for the content
                                            let content_stream_obj = Object::Stream {
                                                dict: HashMap::new(),
                                                data: content_bytes.into(),
                                            };

                                            // Use the pre-allocated content ID (page /Contents already updated)
                                            if let Some(content_id) = modified_content_id {
                                                let offset = writer.stream_position()?;
                                                let bytes = serialize_obj(
                                                    &serializer,
                                                    content_id,
                                                    0,
                                                    &content_stream_obj,
                                                    &encryption_handler,
                                                );
                                                writer.write_all(&bytes)?;
                                                xref_entries.push((content_id, offset, 0, true));
                                            }

                                            // Collect xobject refs for Resources/XObject update
                                            new_xobject_refs.extend(xobject_refs);
                                        }
                                    } else {
                                        // Check if we have image modifications for this page
                                        let has_image_mods = self
                                            .image_modifications
                                            .contains_key(&source_page_index);

                                        if has_image_mods {
                                            // Rewrite content stream with image modifications
                                            if let Some(contents) = page_dict.get("Contents") {
                                                match contents {
                                                    Object::Reference(contents_ref) => {
                                                        let contents_obj = self
                                                            .source
                                                            .load_object(*contents_ref)?;
                                                        if let Ok(content_data) =
                                                            contents_obj.decode_stream_data()
                                                        {
                                                            let mods = self
                                                                .image_modifications
                                                                .get(&source_page_index)
                                                                .unwrap();
                                                            match self.rewrite_content_stream_with_image_mods(&content_data, mods) {
                                                                Ok(modified_content) => {
                                                                    let modified_stream = Object::Stream {
                                                                        dict: HashMap::new(),
                                                                        data: modified_content.into(),
                                                                    };
                                                                    let offset = writer.stream_position()?;
                                                                    let bytes = serialize_obj(&serializer,
                                                                        contents_ref.id,
                                                                        0,
                                                                        &modified_stream,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((contents_ref.id, offset, 0, true));
                                                                }
                                                                Err(_) => {
                                                                    // Fallback to original content on error
                                                                    let offset = writer.stream_position()?;
                                                                    let bytes = serialize_obj(&serializer,
                                                                        contents_ref.id,
                                                                        0,
                                                                        &contents_obj,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((contents_ref.id, offset, 0, true));
                                                                }
                                                            }
                                                        } else {
                                                            // Can't decode, write original
                                                            let offset =
                                                                writer.stream_position()?;
                                                            let bytes = serialize_obj(
                                                                &serializer,
                                                                contents_ref.id,
                                                                0,
                                                                &contents_obj,
                                                                &encryption_handler,
                                                            );
                                                            writer.write_all(&bytes)?;
                                                            xref_entries.push((
                                                                contents_ref.id,
                                                                offset,
                                                                0,
                                                                true,
                                                            ));
                                                        }
                                                    },
                                                    Object::Array(arr) => {
                                                        // Multiple content streams - apply modifications to all
                                                        let mods = self
                                                            .image_modifications
                                                            .get(&source_page_index)
                                                            .unwrap();
                                                        for item in arr {
                                                            if let Object::Reference(ref_obj) = item
                                                            {
                                                                let stream_obj = self
                                                                    .source
                                                                    .load_object(*ref_obj)?;
                                                                if let Ok(content_data) =
                                                                    stream_obj.decode_stream_data()
                                                                {
                                                                    match self.rewrite_content_stream_with_image_mods(&content_data, mods) {
                                                                        Ok(modified_content) => {
                                                                            let modified_stream = Object::Stream {
                                                                                dict: HashMap::new(),
                                                                                data: modified_content.into(),
                                                                            };
                                                                            let offset = writer.stream_position()?;
                                                                            let bytes = serialize_obj(&serializer,
                                                                                ref_obj.id,
                                                                                0,
                                                                                &modified_stream,
                                                                                &encryption_handler,
                                                                            );
                                                                            writer.write_all(&bytes)?;
                                                                            xref_entries.push((ref_obj.id, offset, 0, true));
                                                                        }
                                                                        Err(_) => {
                                                                            let offset = writer.stream_position()?;
                                                                            let bytes = serialize_obj(&serializer,
                                                                                ref_obj.id,
                                                                                0,
                                                                                &stream_obj,
                                                                                &encryption_handler,
                                                                            );
                                                                            writer.write_all(&bytes)?;
                                                                            xref_entries.push((ref_obj.id, offset, 0, true));
                                                                        }
                                                                    }
                                                                } else {
                                                                    let offset =
                                                                        writer.stream_position()?;
                                                                    let bytes = serialize_obj(
                                                                        &serializer,
                                                                        ref_obj.id,
                                                                        0,
                                                                        &stream_obj,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((
                                                                        ref_obj.id, offset, 0, true,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    },
                                                    _ => {},
                                                }
                                            }
                                        } else if destructive_redaction.is_none() {
                                            // Use original contents — but NEVER when this
                                            // page was destructively redacted (#231 G6):
                                            // the redacted replacement is written below
                                            // and the page /Contents now points to it;
                                            // emitting the original here would re-introduce
                                            // the secret bytes.
                                            if let Some(contents_ref) = page_dict
                                                .get("Contents")
                                                .and_then(|c| c.as_reference())
                                            {
                                                let contents_obj =
                                                    self.load_source_object(contents_ref)?;
                                                let offset = writer.stream_position()?;
                                                let bytes = serialize_obj(
                                                    &serializer,
                                                    contents_ref.id,
                                                    0,
                                                    &contents_obj,
                                                    &encryption_handler,
                                                );
                                                writer.write_all(&bytes)?;
                                                xref_entries.push((
                                                    contents_ref.id,
                                                    offset,
                                                    0,
                                                    true,
                                                ));
                                            }
                                        }
                                    }

                                    // Write resources if present (as reference)
                                    if let Some(resources_ref) =
                                        page_dict.get("Resources").and_then(|r| r.as_reference())
                                    {
                                        let mut resources_obj =
                                            self.load_source_object(resources_ref)?;

                                        // Inject new XObject refs into Resources dict
                                        if !new_xobject_refs.is_empty() {
                                            if let Some(res_dict) = resources_obj.as_dict() {
                                                let mut new_res = res_dict.clone();
                                                // Resolve existing XObject dict (may be inline or indirect ref)
                                                let mut xobj_entries = match new_res.get("XObject")
                                                {
                                                    Some(Object::Dictionary(d)) => d.clone(),
                                                    Some(Object::Reference(r)) => self
                                                        .source
                                                        .load_object(*r)
                                                        .ok()
                                                        .and_then(|o| o.as_dict().cloned())
                                                        .unwrap_or_default(),
                                                    _ => HashMap::new(),
                                                };
                                                for (name, obj_ref) in &new_xobject_refs {
                                                    xobj_entries.insert(
                                                        name.clone(),
                                                        Object::Reference(*obj_ref),
                                                    );
                                                }
                                                new_res.insert(
                                                    "XObject".to_string(),
                                                    Object::Dictionary(xobj_entries),
                                                );
                                                resources_obj = Object::Dictionary(new_res);
                                            }
                                        }

                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            resources_ref.id,
                                            0,
                                            &resources_obj,
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((resources_ref.id, offset, 0, true));
                                    } else if !new_xobject_refs.is_empty() {
                                        // Resources is inline (not a reference) — new XObject refs
                                        // cannot be injected because the page dict was already written.
                                        log::warn!(
                                            "Page {} has inline Resources dict; {} new image XObject(s) \
                                             could not be added to Resources/XObject",
                                            page_index,
                                            new_xobject_refs.len(),
                                        );
                                    }

                                    // Write font objects referenced in Resources (handles inline Resources dict)
                                    if let Some(resources) = page_dict.get("Resources") {
                                        let resources_dict = match resources {
                                            Object::Dictionary(d) => Some(d.clone()),
                                            Object::Reference(r) => self
                                                .source
                                                .load_object(*r)
                                                .ok()
                                                .and_then(|o| o.as_dict().cloned()),
                                            _ => None,
                                        };
                                        if let Some(res_dict) = resources_dict {
                                            // Copy Font dictionary entries
                                            if let Some(fonts) = res_dict.get("Font") {
                                                let font_dict = match fonts {
                                                    Object::Dictionary(d) => Some(d.clone()),
                                                    Object::Reference(r) => self
                                                        .source
                                                        .load_object(*r)
                                                        .ok()
                                                        .and_then(|o| o.as_dict().cloned()),
                                                    _ => None,
                                                };
                                                if let Some(fdict) = font_dict {
                                                    // Rebuild written_ids for O(1) dedup lookups
                                                    written_ids.clear();
                                                    written_ids.extend(
                                                        xref_entries
                                                            .iter()
                                                            .map(|(id, _, _, _)| *id),
                                                    );
                                                    for font_ref in fdict.values() {
                                                        if let Some(ref_obj) =
                                                            font_ref.as_reference()
                                                        {
                                                            // Check if we've already written this object
                                                            if !written_ids.contains(&ref_obj.id) {
                                                                // Prefer a staged modification over the original source.
                                                                let font_obj = self
                                                                    .modified_objects
                                                                    .get(&ref_obj.id)
                                                                    .cloned()
                                                                    .map(Ok)
                                                                    .unwrap_or_else(|| {
                                                                        self.source
                                                                            .load_object(ref_obj)
                                                                    });
                                                                if let Ok(font_obj) = font_obj {
                                                                    let offset =
                                                                        writer.stream_position()?;
                                                                    let bytes = serialize_obj(
                                                                        &serializer,
                                                                        ref_obj.id,
                                                                        0,
                                                                        &font_obj,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((
                                                                        ref_obj.id, offset, 0, true,
                                                                    ));
                                                                    written_ids.insert(ref_obj.id);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Copy XObject dictionary entries (images, forms, etc.)
                                            if let Some(xobjects) = res_dict.get("XObject") {
                                                let xobject_dict = match xobjects {
                                                    Object::Dictionary(d) => Some(d.clone()),
                                                    Object::Reference(r) => {
                                                        let loaded =
                                                            self.load_source_object(*r).map_err(|e| {
                                                                log::warn!("Failed to load resource object {} during save: {}", r.id, e);
                                                                e
                                                            }).ok();
                                                        if !written_ids.contains(&r.id) {
                                                            if let Some(ref obj) = loaded {
                                                                let offset =
                                                                    writer.stream_position()?;
                                                                let bytes = serialize_obj(
                                                                    &serializer,
                                                                    r.id,
                                                                    0,
                                                                    obj,
                                                                    &encryption_handler,
                                                                );
                                                                writer.write_all(&bytes)?;
                                                                xref_entries
                                                                    .push((r.id, offset, 0, true));
                                                                written_ids.insert(r.id);
                                                            }
                                                        }
                                                        loaded.and_then(|o| o.as_dict().cloned())
                                                    },
                                                    _ => None,
                                                };
                                                if let Some(xobj_dict) = xobject_dict {
                                                    for xobj_ref in xobj_dict.values() {
                                                        if let Some(ref_obj) =
                                                            xobj_ref.as_reference()
                                                        {
                                                            if !written_ids.contains(&ref_obj.id) {
                                                                if let Ok(xobj_obj) =
                                                                    self.load_source_object(ref_obj)
                                                                {
                                                                    let offset =
                                                                        writer.stream_position()?;
                                                                    let bytes = serialize_obj(
                                                                        &serializer,
                                                                        ref_obj.id,
                                                                        0,
                                                                        &xobj_obj,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((
                                                                        ref_obj.id, offset, 0, true,
                                                                    ));
                                                                    written_ids.insert(ref_obj.id);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            // Copy ExtGState dictionary entries
                                            if let Some(gs_obj) = res_dict.get("ExtGState") {
                                                let gs_dict = match gs_obj {
                                                    Object::Dictionary(d) => Some(d.clone()),
                                                    Object::Reference(r) => {
                                                        let loaded =
                                                            self.load_source_object(*r).map_err(|e| {
                                                                log::warn!("Failed to load resource object {} during save: {}", r.id, e);
                                                                e
                                                            }).ok();
                                                        if !written_ids.contains(&r.id) {
                                                            if let Some(ref obj) = loaded {
                                                                let offset =
                                                                    writer.stream_position()?;
                                                                let bytes = serialize_obj(
                                                                    &serializer,
                                                                    r.id,
                                                                    0,
                                                                    obj,
                                                                    &encryption_handler,
                                                                );
                                                                writer.write_all(&bytes)?;
                                                                xref_entries
                                                                    .push((r.id, offset, 0, true));
                                                                written_ids.insert(r.id);
                                                            }
                                                        }
                                                        loaded.and_then(|o| o.as_dict().cloned())
                                                    },
                                                    _ => None,
                                                };
                                                if let Some(gsd) = gs_dict {
                                                    for gs_ref in gsd.values() {
                                                        if let Some(ref_obj) = gs_ref.as_reference()
                                                        {
                                                            if !written_ids.contains(&ref_obj.id) {
                                                                if let Ok(obj) =
                                                                    self.load_source_object(ref_obj)
                                                                {
                                                                    let offset =
                                                                        writer.stream_position()?;
                                                                    let bytes = serialize_obj(
                                                                        &serializer,
                                                                        ref_obj.id,
                                                                        0,
                                                                        &obj,
                                                                        &encryption_handler,
                                                                    );
                                                                    writer.write_all(&bytes)?;
                                                                    xref_entries.push((
                                                                        ref_obj.id, offset, 0, true,
                                                                    ));
                                                                    written_ids.insert(ref_obj.id);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Write erase overlay content stream if present
                                if let Some(overlay_obj_id) = erase_overlay_id {
                                    if let Some(overlay_content) =
                                        self.generate_erase_overlay(source_page_index)
                                    {
                                        // Create stream object for the overlay
                                        let overlay_stream = Object::Stream {
                                            dict: HashMap::new(),
                                            data: overlay_content.into(),
                                        };
                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            overlay_obj_id,
                                            0,
                                            &overlay_stream,
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((overlay_obj_id, offset, 0, true));
                                    }
                                }

                                // Write overlay additions stream (add_text on existing page)
                                if let Some(additions_id) = overlay_additions_id {
                                    if let Some(added) =
                                        self.overlay_additions.get(&source_page_index)
                                    {
                                        let wrapper = StructureElement {
                                            structure_type: "Document".to_string(),
                                            bbox: crate::geometry::Rect::new(0.0, 0.0, 0.0, 0.0),
                                            children: added.clone(),
                                            reading_order: None,
                                            alt_text: None,
                                            language: None,
                                        };
                                        if let Ok((content_bytes, _pending)) =
                                            self.generate_content_stream(&wrapper)
                                        {
                                            let additions_stream = Object::Stream {
                                                dict: HashMap::new(),
                                                data: content_bytes.into(),
                                            };
                                            let offset = writer.stream_position()?;
                                            let bytes = serialize_obj(
                                                &serializer,
                                                additions_id,
                                                0,
                                                &additions_stream,
                                                &encryption_handler,
                                            );
                                            writer.write_all(&bytes)?;
                                            xref_entries.push((additions_id, offset, 0, true));
                                        }
                                    }
                                }

                                // Write a minimal Type1 Font object for each Base-14 font
                                // referenced by overlay_additions text that wasn't already
                                // declared on this page.
                                for (name, font_id) in &overlay_font_ids {
                                    let mut font_obj_dict: HashMap<String, Object> = HashMap::new();
                                    font_obj_dict.insert(
                                        "Type".to_string(),
                                        Object::Name("Font".to_string()),
                                    );
                                    font_obj_dict.insert(
                                        "Subtype".to_string(),
                                        Object::Name("Type1".to_string()),
                                    );
                                    font_obj_dict
                                        .insert("BaseFont".to_string(), Object::Name(name.clone()));
                                    font_obj_dict.insert(
                                        "Encoding".to_string(),
                                        Object::Name("WinAnsiEncoding".to_string()),
                                    );
                                    let font_obj = Object::Dictionary(font_obj_dict);
                                    let offset = writer.stream_position()?;
                                    let bytes = serialize_obj(
                                        &serializer,
                                        *font_id,
                                        0,
                                        &font_obj,
                                        &encryption_handler,
                                    );
                                    writer.write_all(&bytes)?;
                                    xref_entries.push((*font_id, offset, 0, true));
                                }

                                // Write new annotation objects
                                if !new_annotation_ids.is_empty() {
                                    // Get page refs for building annotations (needed for link destinations)
                                    let page_refs = self.get_page_refs().unwrap_or_default();

                                    // Build the annotation dictionaries first, in
                                    // a scope that releases the `modified_annotations`
                                    // borrow before we allocate ids / mutate `self`.
                                    let built_annots: Vec<(u32, HashMap<String, Object>)> =
                                        if let Some(annotations) =
                                            self.modified_annotations.get(&source_page_index)
                                        {
                                            new_annotation_ids
                                                .iter()
                                                .zip(annotations.iter().filter(|a| a.is_new()))
                                                .filter_map(|(annot_id, annot_wrapper)| {
                                                    annot_wrapper
                                                        .writer_annotation()
                                                        .map(|wa| (*annot_id, wa.build(&page_refs)))
                                                })
                                                .collect()
                                        } else {
                                            Vec::new()
                                        };

                                    for (annot_id, mut annot_dict) in built_annots {
                                        // Hoist any inline appearance stream (e.g. a
                                        // watermark's `/AP /N`) into its own indirect
                                        // object; a stream may not be a direct dict value.
                                        let hoisted = hoist_appearance_streams(
                                            &mut annot_dict,
                                            &mut self.next_object_id,
                                        );
                                        for (stream_id, stream_obj) in hoisted {
                                            let offset = writer.stream_position()?;
                                            let bytes = serialize_obj(
                                                &serializer,
                                                stream_id,
                                                0,
                                                &stream_obj,
                                                &encryption_handler,
                                            );
                                            writer.write_all(&bytes)?;
                                            xref_entries.push((stream_id, offset, 0, true));
                                        }

                                        // Write the annotation object
                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            annot_id,
                                            0,
                                            &Object::Dictionary(annot_dict),
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((annot_id, offset, 0, true));
                                    }
                                }

                                // Write new form field objects
                                if !new_form_field_ids.is_empty() {
                                    let page_ref_for_fields = ObjectRef::new(page_ref.id, 0);

                                    for (field_id, wrapper) in new_form_field_ids
                                        .iter()
                                        .zip(new_form_field_wrappers.iter())
                                    {
                                        // Build the form field dictionary
                                        let field_dict =
                                            wrapper.build_field_dict(page_ref_for_fields);

                                        // Write the form field object
                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            *field_id,
                                            0,
                                            &Object::Dictionary(field_dict),
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((*field_id, offset, 0, true));
                                    }
                                }

                                // Write flatten annotation XObjects and overlay
                                if let Some((ref appearances, overlay_id, ref xobj_ids)) =
                                    flatten_data
                                {
                                    // Write each appearance as a Form XObject
                                    for ((obj_id, _name), appearance) in
                                        xobj_ids.iter().zip(appearances.iter())
                                    {
                                        // Build Form XObject dictionary
                                        let mut form_dict = HashMap::new();
                                        form_dict.insert(
                                            "Type".to_string(),
                                            Object::Name("XObject".to_string()),
                                        );
                                        form_dict.insert(
                                            "Subtype".to_string(),
                                            Object::Name("Form".to_string()),
                                        );
                                        form_dict
                                            .insert("FormType".to_string(), Object::Integer(1));
                                        form_dict.insert(
                                            "BBox".to_string(),
                                            Object::Array(vec![
                                                Object::Real(appearance.bbox[0] as f64),
                                                Object::Real(appearance.bbox[1] as f64),
                                                Object::Real(appearance.bbox[2] as f64),
                                                Object::Real(appearance.bbox[3] as f64),
                                            ]),
                                        );

                                        // Add matrix if present
                                        if let Some(m) = appearance.matrix {
                                            form_dict.insert(
                                                "Matrix".to_string(),
                                                Object::Array(vec![
                                                    Object::Real(m[0] as f64),
                                                    Object::Real(m[1] as f64),
                                                    Object::Real(m[2] as f64),
                                                    Object::Real(m[3] as f64),
                                                    Object::Real(m[4] as f64),
                                                    Object::Real(m[5] as f64),
                                                ]),
                                            );
                                        }

                                        // Add resources if present
                                        if let Some(ref resources) = appearance.resources {
                                            form_dict
                                                .insert("Resources".to_string(), resources.clone());
                                        }

                                        // Create stream object
                                        let form_stream = Object::Stream {
                                            dict: form_dict,
                                            data: appearance.content.clone().into(),
                                        };

                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            *obj_id,
                                            0,
                                            &form_stream,
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((*obj_id, offset, 0, true));

                                        // Write any indirect objects this appearance depends
                                        // on (e.g. an embedded fallback-font graph).
                                        for (extra_id, extra_obj) in &appearance.extra_objects {
                                            let off = writer.stream_position()?;
                                            let b = serialize_obj(
                                                &serializer,
                                                *extra_id,
                                                0,
                                                extra_obj,
                                                &encryption_handler,
                                            );
                                            writer.write_all(&b)?;
                                            xref_entries.push((*extra_id, off, 0, true));
                                        }
                                    }

                                    // Write the overlay content stream that invokes the XObjects
                                    let xobj_names: Vec<String> =
                                        xobj_ids.iter().map(|(_, name)| name.clone()).collect();
                                    let overlay_content =
                                        self.generate_flatten_overlay(appearances, &xobj_names);

                                    let overlay_stream = Object::Stream {
                                        dict: HashMap::new(),
                                        data: overlay_content.into(),
                                    };

                                    let offset = writer.stream_position()?;
                                    let bytes = serialize_obj(
                                        &serializer,
                                        overlay_id,
                                        0,
                                        &overlay_stream,
                                        &encryption_handler,
                                    );
                                    writer.write_all(&bytes)?;
                                    xref_entries.push((overlay_id, offset, 0, true));
                                }

                                // Write redaction overlay content stream if present
                                if let Some((ref redactions, redact_overlay_id)) = redaction_data {
                                    let overlay_content =
                                        self.generate_redaction_overlay(redactions);

                                    let overlay_stream = Object::Stream {
                                        dict: HashMap::new(),
                                        data: overlay_content.into(),
                                    };

                                    let offset = writer.stream_position()?;
                                    let bytes = serialize_obj(
                                        &serializer,
                                        redact_overlay_id,
                                        0,
                                        &overlay_stream,
                                        &encryption_handler,
                                    );
                                    writer.write_all(&bytes)?;
                                    xref_entries.push((redact_overlay_id, offset, 0, true));
                                }

                                // Write the destructively-redacted replacement
                                // content stream (#231). These bytes already
                                // contain the pruned content + opaque overlay;
                                // the page /Contents was repointed to this
                                // object above, orphaning the original.
                                if let Some((redacted_bytes, redacted_id)) = &destructive_redaction
                                {
                                    let redacted_stream = Object::Stream {
                                        dict: HashMap::new(),
                                        data: redacted_bytes.clone().into(),
                                    };
                                    let offset = writer.stream_position()?;
                                    let bytes = serialize_obj(
                                        &serializer,
                                        *redacted_id,
                                        0,
                                        &redacted_stream,
                                        &encryption_handler,
                                    );
                                    writer.write_all(&bytes)?;
                                    xref_entries.push((*redacted_id, offset, 0, true));
                                }

                                // Write form flatten XObjects and overlay if present
                                if let Some((
                                    ref form_appearances,
                                    form_overlay_id,
                                    ref form_xobj_ids,
                                )) = form_flatten_data
                                {
                                    // Write each form appearance as an XObject
                                    for ((obj_id, _), appearance) in
                                        form_xobj_ids.iter().zip(form_appearances.iter())
                                    {
                                        let mut form_dict: HashMap<String, Object> = HashMap::new();
                                        form_dict.insert(
                                            "Type".to_string(),
                                            Object::Name("XObject".to_string()),
                                        );
                                        form_dict.insert(
                                            "Subtype".to_string(),
                                            Object::Name("Form".to_string()),
                                        );
                                        form_dict
                                            .insert("FormType".to_string(), Object::Integer(1));
                                        form_dict.insert(
                                            "BBox".to_string(),
                                            Object::Array(vec![
                                                Object::Real(appearance.bbox[0] as f64),
                                                Object::Real(appearance.bbox[1] as f64),
                                                Object::Real(appearance.bbox[2] as f64),
                                                Object::Real(appearance.bbox[3] as f64),
                                            ]),
                                        );

                                        // Add matrix if present
                                        if let Some(m) = appearance.matrix {
                                            form_dict.insert(
                                                "Matrix".to_string(),
                                                Object::Array(vec![
                                                    Object::Real(m[0] as f64),
                                                    Object::Real(m[1] as f64),
                                                    Object::Real(m[2] as f64),
                                                    Object::Real(m[3] as f64),
                                                    Object::Real(m[4] as f64),
                                                    Object::Real(m[5] as f64),
                                                ]),
                                            );
                                        }

                                        // Add resources if present
                                        if let Some(ref resources) = appearance.resources {
                                            form_dict
                                                .insert("Resources".to_string(), resources.clone());
                                        }

                                        // Create stream object
                                        let form_stream = Object::Stream {
                                            dict: form_dict,
                                            data: appearance.content.clone().into(),
                                        };

                                        let offset = writer.stream_position()?;
                                        let bytes = serialize_obj(
                                            &serializer,
                                            *obj_id,
                                            0,
                                            &form_stream,
                                            &encryption_handler,
                                        );
                                        writer.write_all(&bytes)?;
                                        xref_entries.push((*obj_id, offset, 0, true));

                                        // Write any indirect objects this appearance depends
                                        // on (e.g. an embedded fallback-font graph).
                                        for (extra_id, extra_obj) in &appearance.extra_objects {
                                            let off = writer.stream_position()?;
                                            let b = serialize_obj(
                                                &serializer,
                                                *extra_id,
                                                0,
                                                extra_obj,
                                                &encryption_handler,
                                            );
                                            writer.write_all(&b)?;
                                            xref_entries.push((*extra_id, off, 0, true));
                                        }
                                    }

                                    // Write the overlay content stream that invokes the XObjects
                                    let xobj_names: Vec<String> = form_xobj_ids
                                        .iter()
                                        .map(|(_, name)| name.clone())
                                        .collect();
                                    let overlay_content = self
                                        .generate_flatten_overlay(form_appearances, &xobj_names);

                                    let overlay_stream = Object::Stream {
                                        dict: HashMap::new(),
                                        data: overlay_content.into(),
                                    };

                                    let offset = writer.stream_position()?;
                                    let bytes = serialize_obj(
                                        &serializer,
                                        form_overlay_id,
                                        0,
                                        &overlay_stream,
                                        &encryption_handler,
                                    );
                                    writer.write_all(&bytes)?;
                                    xref_entries.push((form_overlay_id, offset, 0, true));
                                }

                                page_index += 1;
                            }
                        }
                    }
                }
            }
        }

        // Write merged pages and their dependent objects
        // Rebuild written_ids for O(1) dedup lookups in merged page loop
        written_ids.clear();
        written_ids.extend(xref_entries.iter().map(|(id, _, _, _)| *id));
        for (page_data, &page_id) in self.merged_pages.iter().zip(merged_page_ids.iter()) {
            // Set /Parent on the merged page to point to the Pages tree root
            let final_page_obj = if let Some(catalog_dict) = catalog_obj.as_dict() {
                if let Some(pages_ref) = catalog_dict.get("Pages").and_then(|p| p.as_reference()) {
                    if let Object::Dictionary(mut dict) = page_data.page_object.clone() {
                        dict.insert("Parent".to_string(), Object::Reference(pages_ref));
                        Object::Dictionary(dict)
                    } else {
                        page_data.page_object.clone()
                    }
                } else {
                    page_data.page_object.clone()
                }
            } else {
                page_data.page_object.clone()
            };

            // Write the page object
            let offset = writer.stream_position()?;
            let bytes =
                serialize_obj(&serializer, page_id, 0, &final_page_obj, &encryption_handler);
            writer.write_all(&bytes)?;
            xref_entries.push((page_id, offset, 0, true));
            written_ids.insert(page_id);

            // Write all dependent objects for this page
            for (obj_id, obj) in &page_data.objects {
                // Skip if already written (dedup)
                if written_ids.contains(obj_id) {
                    continue;
                }
                let offset = writer.stream_position()?;
                let bytes = serialize_obj(&serializer, *obj_id, 0, obj, &encryption_handler);
                writer.write_all(&bytes)?;
                xref_entries.push((*obj_id, offset, 0, true));
                written_ids.insert(*obj_id);
            }
        }

        // Write parent-only form fields (non-terminal fields with no widget)
        // These don't belong to any specific page, so write them after page processing
        for (_, field_id, wrapper, _) in &all_form_field_data {
            if wrapper.is_parent_only() {
                // Build parent field dictionary (no widget entries)
                let field_dict = wrapper.build_parent_dict();

                // Write the parent field object
                let offset = writer.stream_position()?;
                let bytes = serialize_obj(
                    &serializer,
                    *field_id,
                    0,
                    &Object::Dictionary(field_dict),
                    &encryption_handler,
                );
                writer.write_all(&bytes)?;
                xref_entries.push((*field_id, offset, 0, true));
            }
        }

        // Write info dictionary if modified
        let info_ref = if self.modified_info.is_some() {
            let info = self.modified_info.clone().unwrap();
            let info_id = self.allocate_object_id();
            let info_obj = info.to_object();
            let offset = writer.stream_position()?;
            let bytes = serialize_obj(&serializer, info_id, 0, &info_obj, &encryption_handler);
            writer.write_all(&bytes)?;
            xref_entries.push((info_id, offset, 0, true));
            Some(ObjectRef::new(info_id, 0))
        } else {
            None
        };

        // ── Remaining-objects sweep ────────────────────────────────────────────
        //
        // The page-tree traversal above is deliberately shallow: it handles the
        // catalog, pages tree, page dicts, content streams, and the first level
        // of font/XObject resource references.  For documents built by
        // `DocumentBuilder` with embedded TrueType fonts, the full object graph
        // is deeper:
        //
        //   Type0 font dict  →  DescendantFonts  →  CIDFontType2 dict
        //                                         →  FontDescriptor dict
        //                                            → FontFile2 stream
        //                    →  ToUnicode CMap stream
        //
        // Any of these that were not reached above would produce dangling
        // cross-reference entries in the output, making the PDF unrenderable
        // even though it is structurally valid.  (Issue #401.)
        //
        // Solution: after the main traversal, enumerate every object ID in the
        // source document's xref table and write any that were not yet written.
        // This is safe — written_ids provides O(1) dedup, so already-written
        // objects are skipped; orphaned objects are harmless in a PDF reader.
        //
        // When garbage_collect=true, only reachable objects are written.
        // When compress=true, raw (unfiltered) streams are FlateDecode-compressed.
        //
        // NOTE: `written_ids` must be rebuilt from `xref_entries` here because
        // the merge-page and parent-field loops above push to `xref_entries`
        // without updating `written_ids`.
        written_ids.clear();
        written_ids.extend(xref_entries.iter().map(|(id, _, _, _)| *id));

        let reachable_ids = if options.garbage_collect {
            Some(self.collect_reachable_ids())
        } else {
            None
        };

        let all_source_ids = self.source.all_object_ids();
        for obj_id in all_source_ids {
            if obj_id == 0 || written_ids.contains(&obj_id) {
                continue;
            }
            // #231 G6: never emit the original content stream of a
            // destructively-redacted page (it holds the secret bytes),
            // regardless of GC reachability.
            if self.redacted_orphan_ids.contains(&obj_id) {
                continue;
            }
            if let Some(ref reachable) = reachable_ids {
                if !reachable.contains(&obj_id) {
                    log::debug!("write_full_to_writer: GC dropping unreachable object {}", obj_id);
                    continue;
                }
            }
            // Prefer a staged modification over the original source object.
            let loaded = if let Some(m) = self.modified_objects.get(&obj_id) {
                Ok(m.clone())
            } else {
                self.load_source_object(ObjectRef { id: obj_id, gen: 0 })
            };
            match loaded {
                Ok(obj) => {
                    let obj = if options.compress {
                        compress_stream_if_raw(obj)
                    } else {
                        obj
                    };
                    let offset = writer.stream_position()?;
                    let bytes = serialize_obj(&serializer, obj_id, 0, &obj, &encryption_handler);
                    writer.write_all(&bytes)?;
                    xref_entries.push((obj_id, offset, 0, true));
                    written_ids.insert(obj_id);
                },
                Err(e) => {
                    log::debug!(
                        "write_full_to_writer: skipping unloadable object {} during sweep: {}",
                        obj_id,
                        e
                    );
                },
            }
        }

        // Write any new objects from modified_objects whose IDs are not in the
        // original source xref (e.g. XMP streams allocated by the PDF/A converter).
        {
            let already_written: std::collections::HashSet<u32> =
                xref_entries.iter().map(|(id, _, _, _)| *id).collect();
            let new_objs: Vec<(u32, Object)> = self
                .modified_objects
                .iter()
                .filter(|(&id, _)| !already_written.contains(&id))
                .map(|(&id, obj)| (id, obj.clone()))
                .collect();
            for (obj_id, obj) in new_objs {
                let offset = writer.stream_position()?;
                let bytes = serialize_obj(&serializer, obj_id, 0, &obj, &encryption_handler);
                writer.write_all(&bytes)?;
                xref_entries.push((obj_id, offset, 0, true));
            }
        }

        // Sort xref entries by object ID
        xref_entries.sort_by_key(|(id, _, _, _)| *id);

        // Write xref table
        let xref_offset = writer.stream_position()?;
        write!(writer, "xref\n")?;

        // Find max object ID
        let max_id = xref_entries
            .iter()
            .map(|(id, _, _, _)| *id)
            .max()
            .unwrap_or(0);
        write!(writer, "0 {}\n", max_id + 1)?;

        // Write entries (fill gaps with free entries)
        let mut entry_map: HashMap<u32, (u64, u16, bool)> = xref_entries
            .into_iter()
            .map(|(id, off, gen, used)| (id, (off, gen, used)))
            .collect();

        for id in 0..=max_id {
            if let Some((offset, gen, in_use)) = entry_map.get(&id) {
                if *in_use {
                    write!(writer, "{:010} {:05} n \n", offset, gen)?;
                } else {
                    write!(writer, "{:010} {:05} f \n", offset, gen)?;
                }
            } else {
                // Free entry pointing to object 0
                write!(writer, "0000000000 65535 f \n")?;
            }
        }

        // Write trailer
        write!(writer, "trailer\n")?;
        write!(writer, "<<\n")?;
        write!(writer, "  /Size {}\n", max_id + 1)?;
        write!(writer, "  /Root {} 0 R\n", catalog_ref.id)?;

        if let Some(info_ref) = info_ref {
            write!(writer, "  /Info {} {} R\n", info_ref.id, info_ref.gen)?;
        }

        // Write encryption entries if encrypting
        if let Some(enc_id) = encrypt_obj_id {
            write!(writer, "  /Encrypt {} 0 R\n", enc_id)?;
        }

        // Write file ID if encryption is enabled
        if let Some((id1, id2)) = file_id {
            let id1_hex: String = id1.iter().map(|b| format!("{:02X}", b)).collect();
            let id2_hex: String = id2.iter().map(|b| format!("{:02X}", b)).collect();
            write!(writer, "  /ID [<{}> <{}>]\n", id1_hex, id2_hex)?;
        }

        write!(writer, ">>\n")?;
        write!(writer, "startxref\n")?;
        write!(writer, "{}\n", xref_offset)?;
        write!(writer, "%%EOF\n")?;

        writer.flush()?;
        self.is_modified = false;
        Ok(())
    }

    // === Content modification operations ===

    /// Extract hierarchical content from a page.
    ///
    /// Returns the page's hierarchical content structure with all children populated.
    /// For untagged PDFs, returns a synthetic hierarchy based on geometric analysis.
    ///
    /// # Arguments
    ///
    /// * `page_index` - The page to extract from (0-indexed)
    ///
    /// # Returns
    ///
    /// `Ok(Some(structure))` if structure is found or generated,
    /// `Ok(None)` if no structure is available,
    /// `Err` if an error occurs during extraction
    pub fn get_page_content(&mut self, page_index: usize) -> Result<Option<StructureElement>> {
        HierarchicalExtractor::extract_page(&self.source, page_index)
    }

    /// Replace the content of a page with a new structure.
    ///
    /// Marks the document as modified and sets the structure_modified flag
    /// so the structure tree will be rebuilt on save.
    ///
    /// # Arguments
    ///
    /// * `page_index` - The page to modify (0-indexed)
    /// * `content` - The new hierarchical structure for the page
    ///
    /// # Returns
    ///
    /// `Err` if the page index is out of range
    pub fn set_page_content(&mut self, page_index: usize, content: StructureElement) -> Result<()> {
        let page_count = self.current_page_count();
        if page_index >= page_count {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of range (document has {} pages)",
                page_index, page_count
            )));
        }

        let source_index = self.output_to_source_index(page_index);
        self.modified_content.insert(source_index, content);
        self.structure_modified = true;
        self.is_modified = true;
        Ok(())
    }

    /// Modify a page's structure in-place using a closure.
    ///
    /// Extracts the current content, passes it to the closure for modification,
    /// then saves it back.
    ///
    /// # Arguments
    ///
    /// * `page_index` - The page to modify
    /// * `f` - Closure that modifies the structure
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.modify_structure(0, |structure| {
    ///     // Modify structure in place
    ///     structure.alt_text = Some("Modified alt text".to_string());
    ///     Ok(())
    /// })?;
    /// ```
    pub fn modify_structure<F>(&mut self, page_index: usize, f: F) -> Result<()>
    where
        F: FnOnce(&mut StructureElement) -> Result<()>,
    {
        let mut content = self
            .get_page_content(page_index)?
            .ok_or_else(|| Error::InvalidPdf("No structure available for page".to_string()))?;

        f(&mut content)?;
        self.set_page_content(page_index, content)
    }

    /// Get the resource manager for allocating fonts, images, etc.
    ///
    /// Use this when manually constructing content elements that need resources.
    pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }

    /// Get a reference to the resource manager.
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get a page for DOM-like editing.
    ///
    /// Returns a PdfPage that allows hierarchical navigation and querying
    /// of page content with a DOM-like API.
    pub fn get_page(&mut self, page_index: usize) -> Result<crate::editor::dom::PdfPage> {
        // Get the page info first
        let page_info = self.get_page_info(page_index)?;

        // Get or extract the page content
        let content = if let Some(structure) = self.get_page_content(page_index)? {
            structure
        } else {
            // If no modified content, try to extract from original
            match HierarchicalExtractor::extract_page(&self.source, page_index)? {
                Some(structure) => structure,
                None => {
                    // Create empty structure if extraction fails
                    StructureElement {
                        structure_type: "Document".to_string(),
                        bbox: crate::geometry::Rect::new(
                            0.0,
                            0.0,
                            page_info.width,
                            page_info.height,
                        ),
                        children: Vec::new(),
                        reading_order: Some(0),
                        alt_text: None,
                        language: None,
                    }
                },
            }
        };

        // Load annotations from source document
        let read_annotations = self.source.get_annotations(page_index).unwrap_or_default();
        let annotations: Vec<crate::editor::dom::AnnotationWrapper> = read_annotations
            .into_iter()
            .map(crate::editor::dom::AnnotationWrapper::from_read)
            .collect();

        Ok(crate::editor::dom::PdfPage::from_structure_with_annotations(
            page_index,
            content,
            page_info.width,
            page_info.height,
            annotations,
        ))
    }

    /// Save a modified page back to the document.
    ///
    /// This saves both the page content and any modified annotations.
    pub fn save_page(&mut self, page: crate::editor::dom::PdfPage) -> Result<()> {
        let page_index = self.output_to_source_index(page.page_index);
        let annotations_modified = page.has_annotations_modified();

        // Extract annotations before moving root
        let annotations: Vec<crate::editor::dom::AnnotationWrapper> = if annotations_modified {
            page.annotations().to_vec()
        } else {
            Vec::new()
        };

        if page.is_loaded_from_source() {
            // Page was loaded from an existing PDF.  Use overlay approach: preserve the
            // original content stream and append only the newly added elements as a second
            // stream.  This avoids losing graphics/form-structure that the text-only
            // HierarchicalExtractor cannot round-trip.
            //
            // Always mark as modified so callers that check `is_modified()` after
            // `save_page()` continue to see the expected true value.
            self.is_modified = true;
            let added: Vec<ContentElement> = page.added_children().to_vec();
            // `save_page()` can legitimately be called more than once for the same
            // page (e.g. one add_text() + save_page_from_editor() per inserted
            // element). `.insert()` here replaced any elements staged by a prior
            // call instead of accumulating them, silently dropping all but the
            // last call's additions. Extend the existing Vec instead.
            if !added.is_empty() {
                self.overlay_additions
                    .entry(page_index)
                    .or_default()
                    .extend(added);
            }

            // Pre-existing content that was edited/removed via the DOM can't be
            // captured by the overlay above (it was never "added" — see
            // `PdfPage::pending_erasures`), so destructively redact it from the
            // real content stream instead of silently dropping the edit.
            let erasures = page.pending_erasures().to_vec();
            if !erasures.is_empty() {
                let regions = self.redaction_regions.entry(page_index).or_default();
                for bbox in &erasures {
                    regions.push(crate::redaction::RedactionRegion::from_rect(
                        bbox.x,
                        bbox.y,
                        bbox.x + bbox.width,
                        bbox.y + bbox.height,
                        None,
                    ));
                }
                self.apply_redactions_destructive_for_page(
                    page_index,
                    &crate::redaction::RedactionOptions::default(),
                )?;
            }
        } else {
            // Freshly created page — replace the entire content stream.
            self.set_page_content(page_index, page.root)?;
        }

        // Save annotations if they were modified
        if annotations_modified {
            self.modified_annotations.insert(page_index, annotations);
            self.is_modified = true;
        }

        Ok(())
    }

    /// Get the modified annotations for a page (if any).
    pub fn get_page_annotations(
        &self,
        page_index: usize,
    ) -> Option<&Vec<crate::editor::dom::AnnotationWrapper>> {
        self.modified_annotations.get(&page_index)
    }

    /// Check if a page has modified annotations.
    pub fn has_modified_annotations(&self, page_index: usize) -> bool {
        self.modified_annotations.contains_key(&page_index)
    }

    /// Edit a page with a closure, automatically saving changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.edit_page(0, |page| {
    ///     let text_elements = page.find_text_containing("Hello");
    ///     for text in text_elements {
    ///         page.set_text(text.id(), "Hi")?;
    ///     }
    ///     Ok(())
    /// })?;
    /// ```
    pub fn edit_page<F>(&mut self, page_index: usize, f: F) -> Result<()>
    where
        F: FnOnce(&mut crate::editor::dom::PdfPage) -> Result<()>,
    {
        let mut page = self.get_page(page_index)?;
        f(&mut page)?;
        self.save_page(page)
    }

    /// Get a page editor for fluent/XMLDocument-style editing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.page_editor(0)?
    ///    .find_text_containing("Hello")?
    ///    .for_each(|mut text| {
    ///        text.set_text("Hi");
    ///        Ok(())
    ///    })?
    ///    .done()?;
    /// editor.save_page_editor_modified()?;
    /// ```
    pub fn page_editor(&mut self, page_index: usize) -> Result<crate::editor::dom::PageEditor> {
        let page = self.get_page(page_index)?;
        Ok(crate::editor::dom::PageEditor { page })
    }

    /// Save a page from the fluent editor back to the document.
    pub fn save_page_from_editor(&mut self, page: crate::editor::dom::PdfPage) -> Result<()> {
        self.save_page(page)
    }

    // =========================================================================
    // Page Properties: Rotation, Cropping
    // =========================================================================

    /// Get the rotation of a page in degrees (0, 90, 180, 270).
    ///
    /// Returns the effective rotation, considering any modifications.
    pub fn get_page_rotation(&mut self, index: usize) -> Result<i32> {
        let source_index = self.output_to_source_index(index);
        // Check if we have a modified rotation
        if let Some(props) = self.modified_page_props.get(&source_index) {
            if let Some(rotation) = props.rotation {
                return Ok(rotation);
            }
        }

        // Otherwise get from original document
        let info = self.get_page_info(index)?;
        Ok(info.rotation)
    }

    /// Set the rotation of a page.
    ///
    /// Rotation must be 0, 90, 180, or 270 degrees.
    pub fn set_page_rotation(&mut self, index: usize, degrees: i32) -> Result<()> {
        // Validate rotation
        if ![0, 90, 180, 270].contains(&degrees) {
            return Err(Error::InvalidPdf(
                "Rotation must be 0, 90, 180, or 270 degrees".to_string(),
            ));
        }

        // Validate page index
        if index >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of range (document has {} pages)",
                index,
                self.current_page_count()
            )));
        }

        // Store the modified rotation keyed by source index so serialization can find it.
        let source_index = self.output_to_source_index(index);
        let props = self.modified_page_props.entry(source_index).or_default();
        props.rotation = Some(degrees);

        self.is_modified = true;
        Ok(())
    }

    /// Rotate a page by the given degrees (adds to current rotation).
    ///
    /// The result is normalized to 0, 90, 180, or 270.
    pub fn rotate_page_by(&mut self, index: usize, degrees: i32) -> Result<()> {
        let current = self.get_page_rotation(index)?;
        let new_rotation = ((current + degrees) % 360 + 360) % 360;

        // Normalize to valid PDF rotation
        let normalized = match new_rotation {
            0..=44 => 0,
            45..=134 => 90,
            135..=224 => 180,
            225..=314 => 270,
            _ => 0,
        };

        self.set_page_rotation(index, normalized)
    }

    /// Rotate all pages by the given degrees.
    pub fn rotate_all_pages(&mut self, degrees: i32) -> Result<()> {
        let count = self.current_page_count();
        for i in 0..count {
            self.rotate_page_by(i, degrees)?;
        }
        Ok(())
    }

    /// Get the MediaBox of a page (physical page size).
    ///
    /// Returns [llx, lly, urx, ury] (lower-left x, lower-left y, upper-right x, upper-right y).
    pub fn get_page_media_box(&mut self, index: usize) -> Result<[f32; 4]> {
        let source_index = self.output_to_source_index(index);
        // Check if we have a modified MediaBox
        if let Some(props) = self.modified_page_props.get(&source_index) {
            if let Some(media_box) = props.media_box {
                return Ok(media_box);
            }
        }

        // Get from original document
        let page_refs = self.get_page_refs()?;
        if source_index >= page_refs.len() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", index)));
        }

        let page_ref = page_refs[source_index];
        let page_obj = self.source.load_object(page_ref)?;
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;

        if let Some(media_box) = page_dict.get("MediaBox").and_then(|m| m.as_array()) {
            if media_box.len() >= 4 {
                let llx = media_box[0]
                    .as_real()
                    .or_else(|| media_box[0].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0) as f32;
                let lly = media_box[1]
                    .as_real()
                    .or_else(|| media_box[1].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0) as f32;
                let urx = media_box[2]
                    .as_real()
                    .or_else(|| media_box[2].as_integer().map(|i| i as f64))
                    .unwrap_or(612.0) as f32;
                let ury = media_box[3]
                    .as_real()
                    .or_else(|| media_box[3].as_integer().map(|i| i as f64))
                    .unwrap_or(792.0) as f32;
                return Ok([llx, lly, urx, ury]);
            }
        }

        // Default to Letter size
        Ok([0.0, 0.0, 612.0, 792.0])
    }

    /// Set the MediaBox of a page.
    pub fn set_page_media_box(&mut self, index: usize, box_: [f32; 4]) -> Result<()> {
        if index >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", index)));
        }

        let source_index = self.output_to_source_index(index);
        let props = self.modified_page_props.entry(source_index).or_default();
        props.media_box = Some(box_);

        self.is_modified = true;
        Ok(())
    }

    /// Get the CropBox of a page (visible/printable area).
    ///
    /// Returns None if no CropBox is set (defaults to MediaBox).
    pub fn get_page_crop_box(&mut self, index: usize) -> Result<Option<[f32; 4]>> {
        let source_index = self.output_to_source_index(index);
        // Check if we have a modified CropBox
        if let Some(props) = self.modified_page_props.get(&source_index) {
            if let Some(crop_box) = props.crop_box {
                return Ok(Some(crop_box));
            }
        }

        // Get from original document
        let page_refs = self.get_page_refs()?;
        if source_index >= page_refs.len() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", index)));
        }

        let page_ref = page_refs[source_index];
        let page_obj = self.source.load_object(page_ref)?;
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;

        if let Some(crop_box) = page_dict.get("CropBox").and_then(|c| c.as_array()) {
            if crop_box.len() >= 4 {
                let llx = crop_box[0]
                    .as_real()
                    .or_else(|| crop_box[0].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0) as f32;
                let lly = crop_box[1]
                    .as_real()
                    .or_else(|| crop_box[1].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0) as f32;
                let urx = crop_box[2]
                    .as_real()
                    .or_else(|| crop_box[2].as_integer().map(|i| i as f64))
                    .unwrap_or(612.0) as f32;
                let ury = crop_box[3]
                    .as_real()
                    .or_else(|| crop_box[3].as_integer().map(|i| i as f64))
                    .unwrap_or(792.0) as f32;
                return Ok(Some([llx, lly, urx, ury]));
            }
        }

        Ok(None)
    }

    /// Set the CropBox of a page.
    pub fn set_page_crop_box(&mut self, index: usize, box_: [f32; 4]) -> Result<()> {
        if index >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", index)));
        }

        let source_index = self.output_to_source_index(index);
        let props = self.modified_page_props.entry(source_index).or_default();
        props.crop_box = Some(box_);

        self.is_modified = true;
        Ok(())
    }

    /// Crop margins from all pages.
    ///
    /// This sets the CropBox to be smaller than the MediaBox by the specified margins.
    pub fn crop_margins(&mut self, left: f32, right: f32, top: f32, bottom: f32) -> Result<()> {
        let count = self.current_page_count();
        for i in 0..count {
            let media_box = self.get_page_media_box(i)?;
            let crop_box = [
                media_box[0] + left,
                media_box[1] + bottom,
                media_box[2] - right,
                media_box[3] - top,
            ];
            self.set_page_crop_box(i, crop_box)?;
        }
        Ok(())
    }

    // =========================================================================
    // Content Erasing (Whiteout)
    // =========================================================================

    /// Erase a rectangular region on a page by covering it with white.
    ///
    /// This adds a white rectangle overlay that covers the specified region.
    /// The original content is not removed but hidden beneath the white overlay.
    ///
    /// # Arguments
    ///
    /// * `page` - Page index (0-based)
    /// * `rect` - Rectangle to erase [llx, lly, urx, ury]
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Erase a region in the upper-left corner
    /// editor.erase_region(0, [72.0, 700.0, 200.0, 792.0])?;
    /// editor.save("output.pdf")?;
    /// ```
    pub fn erase_region(&mut self, page: usize, rect: [f32; 4]) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        let regions = self.erase_regions.entry(source_page).or_default();
        regions.push(rect);

        self.is_modified = true;
        Ok(())
    }

    /// Erase multiple rectangular regions on a page.
    pub fn erase_regions(&mut self, page: usize, rects: &[[f32; 4]]) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        let regions = self.erase_regions.entry(source_page).or_default();
        regions.extend_from_slice(rects);

        self.is_modified = true;
        Ok(())
    }

    /// Clear all pending erase operations for a page.
    pub fn clear_erase_regions(&mut self, page: usize) {
        let source_page = self.output_to_source_index(page);
        self.erase_regions.remove(&source_page);
    }

    /// Generate the content stream for erase overlays.
    ///
    /// Returns PDF operators that draw white rectangles over the specified regions.
    fn generate_erase_overlay(&self, page: usize) -> Option<Vec<u8>> {
        let regions = self.erase_regions.get(&page)?;
        if regions.is_empty() {
            return None;
        }

        let mut content = Vec::new();

        // Save graphics state
        content.extend_from_slice(b"q\n");

        // Set fill color to white (RGB 1 1 1)
        content.extend_from_slice(b"1 1 1 rg\n");

        // Draw each rectangle
        for rect in regions {
            let x = rect[0];
            let y = rect[1];
            let width = rect[2] - rect[0];
            let height = rect[3] - rect[1];

            // Rectangle path and fill
            content.extend_from_slice(
                format!("{:.2} {:.2} {:.2} {:.2} re f\n", x, y, width, height).as_bytes(),
            );
        }

        // Restore graphics state
        content.extend_from_slice(b"Q\n");

        Some(content)
    }

    // ========================================================================
    // Annotation Flattening
    // ========================================================================

    /// Mark annotations on a page for flattening.
    ///
    /// When the document is saved, annotations on this page will be rendered
    /// into the page content and removed from the annotations array.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Flatten annotations on page 0
    /// editor.flatten_page_annotations(0)?;
    /// editor.save("output.pdf")?;
    /// ```
    pub fn flatten_page_annotations(&mut self, page: usize) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        self.flatten_annotations_pages.insert(source_page);
        self.is_modified = true;
        Ok(())
    }

    /// Mark all pages for annotation flattening.
    ///
    /// When the document is saved, all annotations will be rendered
    /// into the page content and removed.
    pub fn flatten_all_annotations(&mut self) -> Result<()> {
        let page_count = self.current_page_count();
        for page in 0..page_count {
            self.flatten_annotations_pages.insert(page);
        }
        self.is_modified = true;
        Ok(())
    }

    /// Check if a page has annotations marked for flattening.
    pub fn is_page_marked_for_flatten(&self, page: usize) -> bool {
        let source_page = self.output_to_source_index(page);
        self.flatten_annotations_pages.contains(&source_page)
    }

    /// Clear the flatten annotation flag for a page.
    pub fn unmark_page_for_flatten(&mut self, page: usize) {
        let source_page = self.output_to_source_index(page);
        self.flatten_annotations_pages.remove(&source_page);
    }

    // ========================================================================
    // Form Flattening
    // ========================================================================

    /// Mark form fields on a specific page for flattening.
    ///
    /// When the document is saved, form fields (Widget annotations) on this page
    /// will be rendered into the page content. Only Widget annotations are flattened,
    /// other annotation types are preserved.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Flatten forms on page 0
    /// editor.flatten_forms_on_page(0)?;
    /// editor.save("flattened.pdf")?;
    /// ```
    pub fn flatten_forms_on_page(&mut self, page: usize) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        self.flatten_forms_pages.insert(source_page);
        self.is_modified = true;
        Ok(())
    }

    /// Mark all pages for form field flattening.
    ///
    /// When the document is saved, all form fields will be rendered into the page
    /// content and the AcroForm dictionary will be removed from the catalog.
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.flatten_forms()?;
    /// editor.save("flattened.pdf")?;
    /// ```
    pub fn flatten_forms(&mut self) -> Result<()> {
        let page_count = self.current_page_count();
        for page in 0..page_count {
            self.flatten_forms_pages.insert(page);
        }
        self.remove_acroform = true;
        self.is_modified = true;
        Ok(())
    }

    /// Check if a page has form fields marked for flattening.
    pub fn is_page_marked_for_form_flatten(&self, page: usize) -> bool {
        let source_page = self.output_to_source_index(page);
        self.flatten_forms_pages.contains(&source_page)
    }

    /// Check if AcroForm will be removed on save.
    pub fn will_remove_acroform(&self) -> bool {
        self.remove_acroform
    }

    /// Warnings collected during the last form-flattening save.
    ///
    /// Each entry names a widget field that had no `/AP` appearance stream and
    /// could not have one generated — flattening it produces a blank rectangle.
    pub fn flatten_warnings(&self) -> &[String] {
        &self.flatten_warnings
    }

    /// Rebuild an AcroForm dict containing only root fields that still have at
    /// least one widget on a non-flattened page.
    ///
    /// Returns `None` if the original catalog has no AcroForm, or if the AcroForm
    /// contains `/XFA` (in which case we leave it untouched and emit a warning).
    fn rebuild_partial_acroform(
        &mut self,
        catalog_dict: &HashMap<String, Object>,
    ) -> Result<Option<Object>> {
        let acroform_obj = match catalog_dict.get("AcroForm") {
            Some(o) => o.clone(),
            None => return Ok(None),
        };

        let acroform_dict: HashMap<String, Object> = match acroform_obj {
            Object::Dictionary(d) => d,
            Object::Reference(r) => match self.source.load_object(r)? {
                Object::Dictionary(d) => d,
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };

        // XFA forms use a completely different field-addressing model; rebuilding
        // the /Fields array would break SOM expressions inside the XFA stream.
        // Leave the AcroForm as-is and warn the caller (ISO 32000-1 §12.7.8).
        if acroform_dict.contains_key("XFA") {
            self.flatten_warnings.push(
                "XFA form detected — AcroForm not rebuilt after partial flatten; \
                 XFA SOM expressions may reference flattened widgets"
                    .to_string(),
            );
            return Ok(None);
        }

        let fields_array = match acroform_dict.get("Fields") {
            Some(Object::Array(arr)) => arr.clone(),
            _ => return Ok(None),
        };

        // Build page-ref → page-index map in one tree walk
        // (per-index get_page_ref is O(n), looping it is O(n²)).
        let page_ref_to_index: HashMap<u32, usize> = self
            .source
            .all_page_refs()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();

        // Collect root field refs that survive the partial flatten
        let flattened = self.flatten_forms_pages.clone();
        let mut surviving: Vec<Object> = Vec::new();
        for field_entry in &fields_array {
            if let Object::Reference(field_ref) = field_entry {
                if self.field_has_surviving_widgets(field_ref, &flattened, &page_ref_to_index)? {
                    surviving.push(Object::Reference(*field_ref));
                }
            }
        }

        // Preserve all original top-level AcroForm keys; replace only /Fields.
        let mut new_acroform = acroform_dict.clone();
        new_acroform.insert("Fields".to_string(), Object::Array(surviving));
        Ok(Some(Object::Dictionary(new_acroform)))
    }

    /// Returns true if `field_ref` or any of its descendants has a widget whose
    /// `/P` page reference maps to a page that was NOT flattened.
    fn field_has_surviving_widgets(
        &mut self,
        field_ref: &ObjectRef,
        flattened_pages: &HashSet<usize>,
        page_ref_to_index: &HashMap<u32, usize>,
    ) -> Result<bool> {
        let field_obj = match self.source.load_object(*field_ref) {
            Ok(o) => o,
            Err(_) => return Ok(true), // can't load → keep to be safe
        };
        let dict = match field_obj.as_dict() {
            Some(d) => d.clone(),
            None => return Ok(true),
        };

        if let Some(Object::Array(kids)) = dict.get("Kids").cloned() {
            // Non-terminal field — recurse into kids
            for kid in &kids {
                if let Object::Reference(kid_ref) = kid {
                    if self.field_has_surviving_widgets(
                        kid_ref,
                        flattened_pages,
                        page_ref_to_index,
                    )? {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            // Terminal field / merged field+widget — check /P (page ref)
            match dict.get("P") {
                Some(Object::Reference(page_ref)) => {
                    match page_ref_to_index.get(&page_ref.id) {
                        Some(idx) => Ok(!flattened_pages.contains(idx)),
                        // Page ref not in map — keep the field (unknown page)
                        None => Ok(true),
                    }
                },
                // No /P — keep the field
                _ => Ok(true),
            }
        }
    }

    // =========================================================================
    // File Attachments (Embedded Files)
    // =========================================================================

    /// Embed a file in the document.
    ///
    /// The file will be added to the document's EmbeddedFiles name tree
    /// when the document is saved.
    ///
    /// # Arguments
    ///
    /// * `name` - The file name (used as identifier and display name)
    /// * `data` - The file contents
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("input.pdf")?;
    /// editor.embed_file("data.csv", csv_bytes)?;
    /// editor.save("output.pdf")?;
    /// ```
    pub fn embed_file(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        let file = crate::writer::EmbeddedFile::new(name, data);
        self.embedded_files.push(file);
        self.is_modified = true;
        Ok(())
    }

    /// Embed a file with additional metadata.
    ///
    /// # Arguments
    ///
    /// * `file` - The embedded file configuration
    pub fn embed_file_with_options(&mut self, file: crate::writer::EmbeddedFile) -> Result<()> {
        self.embedded_files.push(file);
        self.is_modified = true;
        Ok(())
    }

    /// Get the list of files that will be embedded on save.
    pub fn pending_embedded_files(&self) -> &[crate::writer::EmbeddedFile] {
        &self.embedded_files
    }

    /// Clear all pending embedded files.
    pub fn clear_embedded_files(&mut self) {
        self.embedded_files.clear();
    }

    // =========================================================================
    // XFA Forms Support
    // =========================================================================

    /// Check if this document contains XFA forms.
    ///
    /// XFA (XML Forms Architecture) is an XML-based form specification used
    /// in some PDFs, particularly government and financial forms.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    /// if editor.has_xfa()? {
    ///     println!("Document contains XFA forms");
    /// }
    /// ```
    pub fn has_xfa(&mut self) -> Result<bool> {
        crate::xfa::XfaExtractor::has_xfa(&mut self.source)
    }

    /// Analyze XFA forms in this document without converting.
    ///
    /// Returns information about the XFA form structure including
    /// field count, page count, and field types.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    /// let analysis = editor.analyze_xfa()?;
    ///
    /// if analysis.has_xfa {
    ///     println!("Found {} fields across {} pages",
    ///         analysis.field_count.unwrap_or(0),
    ///         analysis.page_count.unwrap_or(0));
    /// }
    /// ```
    pub fn analyze_xfa(&mut self) -> Result<crate::xfa::XfaAnalysis> {
        crate::xfa::analyze_xfa_document(&mut self.source)
    }

    /// Convert XFA forms to AcroForm and return new PDF bytes.
    ///
    /// This creates a new PDF document with the XFA forms converted to
    /// standard AcroForm fields. The original document is not modified.
    ///
    /// # Limitations
    ///
    /// This implementation supports **static conversion only**:
    /// - Extracts field definitions and current values
    /// - Converts fields to equivalent AcroForm types
    /// - Uses simple vertical stacking layout
    ///
    /// **NOT supported:**
    /// - Dynamic XFA features (scripts, calculations, conditional logic)
    /// - Complex layouts (tables, grids, repeating sections)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("xfa_form.pdf")?;
    /// if editor.has_xfa()? {
    ///     let acroform_bytes = editor.convert_xfa_to_acroform(None)?;
    ///     std::fs::write("converted.pdf", acroform_bytes)?;
    /// }
    /// ```
    pub fn convert_xfa_to_acroform(
        &mut self,
        options: Option<crate::xfa::XfaConversionOptions>,
    ) -> Result<Vec<u8>> {
        crate::xfa::convert_xfa_document(&mut self.source, options)
    }

    // =========================================================================
    // Form Field Editing
    // =========================================================================

    /// Get all form fields from the document.
    ///
    /// Returns form fields from the document's AcroForm, including any modifications
    /// made during this editing session. Deleted fields are not included.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    /// let fields = editor.get_form_fields()?;
    ///
    /// for field in &fields {
    ///     println!("{}: {:?}", field.name(), field.value());
    /// }
    /// ```
    pub fn get_form_fields(&mut self) -> Result<Vec<FormFieldWrapper>> {
        use crate::extractors::forms::FormExtractor;

        // Extract fields from source document
        let source_fields = FormExtractor::extract_fields(&self.source)?;

        // Build page ref -> index map for resolving field page indices in
        // one tree walk (per-index get_page_ref is O(n), looping is O(n²)).
        let page_ref_map: HashMap<u32, usize> = self
            .source
            .all_page_refs()
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();

        let mut result = Vec::new();

        // Add original fields (wrapped), excluding deleted ones
        for field in source_fields {
            let full_name = field.full_name.clone();

            // Skip if deleted
            if self.deleted_form_fields.contains(&full_name) {
                continue;
            }

            // Check if we have a modified version
            if let Some(wrapper) = self.modified_form_fields.get(&full_name) {
                result.push(wrapper.clone());
            } else {
                // Determine page index from widget annotation's /P entry
                let page_index = field
                    .object_ref
                    .and_then(|obj_ref| self.source.load_object(obj_ref).ok())
                    .and_then(|obj| obj.as_dict().cloned())
                    .and_then(|dict| {
                        // Check /P on the field dict first (merged field+widget)
                        if let Some(page_ref) = dict.get("P").and_then(|p| p.as_reference()) {
                            return Some(page_ref);
                        }
                        // If no /P, follow /Kids to the first widget annotation
                        dict.get("Kids")
                            .and_then(|k| match k {
                                Object::Array(arr) => Some(arr.clone()),
                                Object::Reference(r) => self
                                    .source
                                    .load_object(*r)
                                    .ok()
                                    .and_then(|o| o.as_array().cloned()),
                                _ => None,
                            })
                            .and_then(|kids| kids.first().cloned())
                            .and_then(|kid_ref| {
                                kid_ref
                                    .as_reference()
                                    .and_then(|r| self.source.load_object(r).ok())
                            })
                            .and_then(|kid_obj| kid_obj.as_dict().cloned())
                            .and_then(|kid_dict| kid_dict.get("P").and_then(|p| p.as_reference()))
                    })
                    .and_then(|page_ref| page_ref_map.get(&page_ref.id).copied())
                    .unwrap_or(0);
                result.push(FormFieldWrapper::from_read(field, page_index, None));
            }
        }

        // Add new fields (not from original document)
        for (name, wrapper) in &self.modified_form_fields {
            if wrapper.is_new() && !self.deleted_form_fields.contains(name) {
                result.push(wrapper.clone());
            }
        }

        Ok(result)
    }

    /// Get the value of a specific form field by name.
    ///
    /// Returns the current value of the field, which may be the original value
    /// or a modified value if `set_form_field_value()` was called.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field (e.g., "form.section.field")
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    ///
    /// if let Some(value) = editor.get_form_field_value("email")? {
    ///     println!("Email: {:?}", value);
    /// }
    /// ```
    pub fn get_form_field_value(
        &mut self,
        name: &str,
    ) -> Result<Option<crate::editor::form_fields::FormFieldValue>> {
        use crate::editor::form_fields::FormFieldValue;
        use crate::extractors::forms::FormExtractor;

        // Check if deleted
        if self.deleted_form_fields.contains(name) {
            return Ok(None);
        }

        // Check modified fields first
        if let Some(wrapper) = self.modified_form_fields.get(name) {
            return Ok(Some(wrapper.value()));
        }

        // Look up in original document
        let source_fields = FormExtractor::extract_fields(&self.source)?;

        for field in source_fields {
            if field.full_name == name {
                return Ok(Some(FormFieldValue::from(&field.value)));
            }
        }

        Ok(None)
    }

    /// Check if a form field with the given name exists.
    ///
    /// Returns true if the field exists in the original document or was added
    /// during this editing session, and has not been deleted.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    ///
    /// if editor.has_form_field("email")? {
    ///     println!("Email field exists");
    /// }
    /// ```
    pub fn has_form_field(&mut self, name: &str) -> Result<bool> {
        use crate::extractors::forms::FormExtractor;

        // Check if deleted
        if self.deleted_form_fields.contains(name) {
            return Ok(false);
        }

        // Check modified fields (includes new fields)
        if self.modified_form_fields.contains_key(name) {
            return Ok(true);
        }

        // Look up in original document
        let source_fields = FormExtractor::extract_fields(&self.source)?;

        for field in source_fields {
            if field.full_name == name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Add a new form field to a page.
    ///
    /// Creates a new form field and widget annotation on the specified page.
    /// The field will be added to the document's AcroForm on save.
    ///
    /// # Arguments
    ///
    /// * `page` - The page index (0-based) where the field should appear
    /// * `widget` - A form field widget implementing `FormFieldWidget`
    ///
    /// # Returns
    ///
    /// The full qualified name of the added field, which may be modified if
    /// a field with the same name already exists.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    /// use pdf_oxide::writer::form_fields::TextFieldWidget;
    /// use pdf_oxide::geometry::Rect;
    ///
    /// let mut editor = DocumentEditor::open("document.pdf")?;
    ///
    /// // Add a text field to page 0
    /// let name = editor.add_form_field(0,
    ///     TextFieldWidget::new("email", Rect::new(100.0, 700.0, 200.0, 20.0))
    ///         .with_value("user@example.com")
    /// )?;
    ///
    /// println!("Added field: {}", name);
    /// editor.save("output.pdf")?;
    /// ```
    pub fn add_form_field<W: crate::writer::form_fields::FormFieldWidget>(
        &mut self,
        page: usize,
        widget: W,
    ) -> Result<String> {
        // Validate page index
        let page_count = self.page_count()?;
        if page >= page_count {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of bounds (document has {} pages)",
                page, page_count
            )));
        }

        // Make name unique if it already exists
        let mut name = widget.field_name().to_string();
        let mut counter = 1;
        while self.has_form_field(&name)? {
            name = format!("{}_{}", widget.field_name(), counter);
            counter += 1;
        }

        // Create wrapper from widget
        let mut wrapper = FormFieldWrapper::from_widget(&widget, page);

        // Override name if it was modified for uniqueness
        if name != widget.field_name() {
            wrapper.name = name.clone();
        }

        // Mark document as modified
        self.is_modified = true;
        self.acroform_modified = true;

        // Store in modified fields
        self.modified_form_fields.insert(name.clone(), wrapper);

        Ok(name)
    }

    /// Add a parent container field for hierarchical form fields.
    ///
    /// Parent fields are non-terminal fields that don't have a widget annotation
    /// but contain child fields via the `/Kids` array. They can be used to:
    /// - Group related fields (e.g., `address.street`, `address.city`)
    /// - Inherit properties to children (flags, field type, default appearance)
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the parent field
    ///
    /// # Returns
    ///
    /// The full qualified name of the parent field.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::{DocumentEditor, ParentFieldConfig};
    ///
    /// let mut editor = DocumentEditor::open("document.pdf")?;
    ///
    /// // Create a parent field
    /// editor.add_parent_field(ParentFieldConfig::new("address"))?;
    ///
    /// // Add children under the parent
    /// editor.add_child_field("address", 0, TextFieldWidget::new("street", rect))?;
    /// editor.add_child_field("address", 0, TextFieldWidget::new("city", rect2))?;
    ///
    /// editor.save("output.pdf")?;
    /// ```
    pub fn add_parent_field(
        &mut self,
        config: crate::editor::form_fields::ParentFieldConfig,
    ) -> Result<String> {
        let name = config.full_name();

        // Check if parent already exists
        if self.has_form_field(&name)? {
            return Err(Error::InvalidPdf(format!("Parent field already exists: {}", name)));
        }

        // If this parent has a parent, verify it exists
        if let Some(ref parent_name) = config.parent_name {
            if !self.has_form_field(parent_name)? {
                return Err(Error::InvalidPdf(format!("Parent field not found: {}", parent_name)));
            }
        }

        // Create wrapper from config
        let wrapper = FormFieldWrapper::from_parent_config(&config);

        // Mark document as modified
        self.is_modified = true;
        self.acroform_modified = true;

        // Store in modified fields
        self.modified_form_fields.insert(name.clone(), wrapper);

        Ok(name)
    }

    /// Add a form field as a child of an existing parent field.
    ///
    /// Creates a hierarchical relationship where the child field's partial name
    /// becomes the full name: `parent_name.widget_name`.
    ///
    /// # Arguments
    ///
    /// * `parent_name` - Name of the existing parent field
    /// * `page` - Page index where the widget appears (0-based)
    /// * `widget` - The form field widget to add
    ///
    /// # Returns
    ///
    /// The full qualified name of the child field.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::{DocumentEditor, ParentFieldConfig};
    /// use pdf_oxide::writer::form_fields::TextFieldWidget;
    /// use pdf_oxide::geometry::Rect;
    ///
    /// let mut editor = DocumentEditor::open("document.pdf")?;
    ///
    /// // Create parent first
    /// editor.add_parent_field(ParentFieldConfig::new("contact"))?;
    ///
    /// // Add children
    /// let name = editor.add_child_field("contact", 0,
    ///     TextFieldWidget::new("email", Rect::new(100.0, 700.0, 200.0, 20.0))
    /// )?;
    /// assert_eq!(name, "contact.email");
    ///
    /// editor.save("output.pdf")?;
    /// ```
    pub fn add_child_field<W: crate::writer::form_fields::FormFieldWidget>(
        &mut self,
        parent_name: &str,
        page: usize,
        widget: W,
    ) -> Result<String> {
        // Validate page index
        let page_count = self.page_count()?;
        if page >= page_count {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of bounds (document has {} pages)",
                page, page_count
            )));
        }

        // Verify parent exists
        if !self.has_form_field(parent_name)? {
            return Err(Error::InvalidPdf(format!("Parent field not found: {}", parent_name)));
        }

        // Create wrapper with parent reference
        let wrapper = FormFieldWrapper::from_widget_with_parent(&widget, page, parent_name);
        let name = wrapper.name.clone();

        // Check for duplicate name
        if self.has_form_field(&name)? {
            return Err(Error::InvalidPdf(format!("Child field already exists: {}", name)));
        }

        // Mark document as modified
        self.is_modified = true;
        self.acroform_modified = true;

        // Store in modified fields
        self.modified_form_fields.insert(name.clone(), wrapper);

        Ok(name)
    }

    /// Add a form field with automatic hierarchical parent creation.
    ///
    /// If the widget name contains dots (e.g., "address.street"), this method
    /// automatically creates any missing parent fields. This provides a convenient
    /// way to create hierarchical forms without manually managing parents.
    ///
    /// # Arguments
    ///
    /// * `page` - Page index where the widget appears (0-based)
    /// * `widget` - The form field widget to add
    ///
    /// # Returns
    ///
    /// The full qualified name of the added field.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    /// use pdf_oxide::writer::form_fields::TextFieldWidget;
    /// use pdf_oxide::geometry::Rect;
    ///
    /// let mut editor = DocumentEditor::open("document.pdf")?;
    ///
    /// // Automatically creates "address" parent if needed
    /// editor.add_form_field_hierarchical(0,
    ///     TextFieldWidget::new("address.street", Rect::new(100.0, 700.0, 200.0, 20.0))
    /// )?;
    ///
    /// // Reuses existing "address" parent
    /// editor.add_form_field_hierarchical(0,
    ///     TextFieldWidget::new("address.city", Rect::new(100.0, 670.0, 200.0, 20.0))
    /// )?;
    ///
    /// // Creates nested hierarchy: "contact" -> "address" -> "zip"
    /// editor.add_form_field_hierarchical(0,
    ///     TextFieldWidget::new("contact.address.zip", Rect::new(100.0, 640.0, 100.0, 20.0))
    /// )?;
    ///
    /// editor.save("output.pdf")?;
    /// ```
    pub fn add_form_field_hierarchical<W: crate::writer::form_fields::FormFieldWidget>(
        &mut self,
        page: usize,
        widget: W,
    ) -> Result<String> {
        use crate::editor::form_fields::ParentFieldConfig;

        let full_name = widget.field_name().to_string();

        // If no dots, delegate to regular add_form_field
        if !full_name.contains('.') {
            return self.add_form_field(page, widget);
        }

        // Parse the hierarchy path
        let parts: Vec<&str> = full_name.split('.').collect();

        // Create parent fields as needed
        let mut current_parent = String::new();
        for i in 0..(parts.len() - 1) {
            let part = parts[i];
            let parent_name = if current_parent.is_empty() {
                part.to_string()
            } else {
                format!("{}.{}", current_parent, part)
            };

            // Create parent if it doesn't exist
            if !self.has_form_field(&parent_name)? {
                let mut config = ParentFieldConfig::new(part);
                if !current_parent.is_empty() {
                    config = config.with_parent(&current_parent);
                }
                self.add_parent_field(config)?;
            }

            current_parent = parent_name;
        }

        // Add the terminal field as a child
        self.add_child_field(&current_parent, page, widget)
    }

    /// Set the value of an existing form field.
    ///
    /// Modifies the value of a form field. The field must exist in the document
    /// (either from the original PDF or added via `add_form_field`).
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `value` - The new value for the field
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::{DocumentEditor, FormFieldValue};
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    ///
    /// editor.set_form_field_value("name", FormFieldValue::Text("John Doe".into()))?;
    /// editor.set_form_field_value("subscribe", FormFieldValue::Boolean(true))?;
    ///
    /// editor.save("updated.pdf")?;
    /// ```
    pub fn set_form_field_value(
        &mut self,
        name: &str,
        value: crate::editor::form_fields::FormFieldValue,
    ) -> Result<()> {
        use crate::extractors::forms::FormExtractor;

        // Check if deleted
        if self.deleted_form_fields.contains(name) {
            return Err(Error::InvalidPdf(format!("Cannot set value on deleted field: {}", name)));
        }

        // Check if we already have a wrapper for this field
        if let Some(wrapper) = self.modified_form_fields.get_mut(name) {
            wrapper.set_value(value);
            self.is_modified = true;
            self.acroform_modified = true;
            return Ok(());
        }

        // Look up in original document and create wrapper
        let source_fields = FormExtractor::extract_fields(&self.source)?;

        for field in source_fields {
            if field.full_name == name {
                // Create wrapper and set value
                let obj_ref = field.object_ref;
                let mut wrapper = FormFieldWrapper::from_read(field, 0, obj_ref);
                wrapper.set_value(value);

                self.modified_form_fields.insert(name.to_string(), wrapper);
                self.is_modified = true;
                self.acroform_modified = true;
                return Ok(());
            }
        }

        Err(Error::InvalidPdf(format!("Form field not found: {}", name)))
    }

    /// Remove a form field from the document.
    ///
    /// Marks a form field for removal. The field will be removed from the
    /// document's AcroForm and its widget annotation will be removed from
    /// the page when the document is saved.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field to remove
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    ///
    /// editor.remove_form_field("obsolete_field")?;
    ///
    /// editor.save("cleaned.pdf")?;
    /// ```
    pub fn remove_form_field(&mut self, name: &str) -> Result<()> {
        // Check if field exists
        if !self.has_form_field(name)? {
            return Err(Error::InvalidPdf(format!("Form field not found: {}", name)));
        }

        // Remove from modified fields if present
        self.modified_form_fields.remove(name);

        // Add to deleted set
        self.deleted_form_fields.insert(name.to_string());

        self.is_modified = true;
        self.acroform_modified = true;

        Ok(())
    }

    // ========== Form Field Property Modification APIs ==========

    /// Set a form field to read-only.
    ///
    /// A read-only field cannot be edited by the user in a PDF viewer.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `readonly` - Whether the field should be read-only
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("form.pdf")?;
    /// editor.set_form_field_readonly("signature_field", true)?;
    /// editor.save("readonly.pdf")?;
    /// ```
    pub fn set_form_field_readonly(&mut self, name: &str, readonly: bool) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_readonly(readonly);
        })
    }

    /// Set a form field as required.
    ///
    /// A required field must have a value when the form is submitted/exported.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `required` - Whether the field should be required
    pub fn set_form_field_required(&mut self, name: &str, required: bool) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_required(required);
        })
    }

    /// Set a form field's tooltip/description.
    ///
    /// The tooltip is displayed when the user hovers over the field.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `tooltip` - The tooltip text
    pub fn set_form_field_tooltip(&mut self, name: &str, tooltip: impl Into<String>) -> Result<()> {
        let tooltip_str = tooltip.into();
        self.modify_form_field(name, |wrapper| {
            wrapper.set_tooltip(tooltip_str);
        })
    }

    /// Set a form field's bounding rectangle.
    ///
    /// This changes the position and size of the field on the page.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `rect` - The new bounding rectangle
    pub fn set_form_field_rect(&mut self, name: &str, rect: Rect) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_rect(rect);
        })
    }

    /// Set a form field's maximum text length.
    ///
    /// Only applicable to text fields.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `max_len` - The maximum number of characters
    pub fn set_form_field_max_length(&mut self, name: &str, max_len: u32) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_max_length(max_len);
        })
    }

    /// Set a form field's text alignment.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `alignment` - 0 = left, 1 = center, 2 = right
    pub fn set_form_field_alignment(&mut self, name: &str, alignment: u32) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_alignment(alignment);
        })
    }

    /// Set a form field's background color.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `color` - RGB color values (0.0 to 1.0)
    pub fn set_form_field_background_color(&mut self, name: &str, color: [f32; 3]) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_background_color(color);
        })
    }

    /// Set a form field's border color.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `color` - RGB color values (0.0 to 1.0)
    pub fn set_form_field_border_color(&mut self, name: &str, color: [f32; 3]) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_border_color(color);
        })
    }

    /// Set a form field's border width.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `width` - Border width in points
    pub fn set_form_field_border_width(&mut self, name: &str, width: f32) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_border_width(width);
        })
    }

    /// Set a form field's default appearance string.
    ///
    /// The DA string specifies font, size, and color for field content.
    /// Example: "/Helv 12 Tf 0 g" for 12pt Helvetica in black.
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `da` - The default appearance string
    pub fn set_form_field_default_appearance(
        &mut self,
        name: &str,
        da: impl Into<String>,
    ) -> Result<()> {
        let da_str = da.into();
        self.modify_form_field(name, |wrapper| {
            wrapper.set_default_appearance(da_str);
        })
    }

    /// Set form field flags directly.
    ///
    /// Use this for setting custom flag combinations. Common flags:
    /// - Bit 1 (0x01): ReadOnly
    /// - Bit 2 (0x02): Required
    /// - Bit 3 (0x04): NoExport
    ///
    /// # Arguments
    ///
    /// * `name` - The full qualified name of the field
    /// * `flags` - The field flag bits
    pub fn set_form_field_flags(&mut self, name: &str, flags: u32) -> Result<()> {
        self.modify_form_field(name, |wrapper| {
            wrapper.set_flags(flags);
        })
    }

    /// Internal helper to modify a form field.
    ///
    /// Gets or creates a wrapper for the field and applies the modification.
    fn modify_form_field<F>(&mut self, name: &str, modify_fn: F) -> Result<()>
    where
        F: FnOnce(&mut FormFieldWrapper),
    {
        use crate::extractors::forms::FormExtractor;

        // Check if deleted
        if self.deleted_form_fields.contains(name) {
            return Err(Error::InvalidPdf(format!("Cannot modify deleted field: {}", name)));
        }

        // Check if we already have a wrapper for this field
        if let Some(wrapper) = self.modified_form_fields.get_mut(name) {
            modify_fn(wrapper);
            self.is_modified = true;
            self.acroform_modified = true;
            return Ok(());
        }

        // Look up in original document and create wrapper
        let source_fields = FormExtractor::extract_fields(&self.source)?;

        for field in source_fields {
            if field.full_name == name {
                // Get object ref from the field
                let object_ref = field.object_ref;

                // Create wrapper
                let mut wrapper = FormFieldWrapper::from_read(field, 0, object_ref);
                modify_fn(&mut wrapper);

                self.modified_form_fields.insert(name.to_string(), wrapper);
                self.is_modified = true;
                self.acroform_modified = true;
                return Ok(());
            }
        }

        Err(Error::InvalidPdf(format!("Form field not found: {}", name)))
    }

    // ========== Form Data Export APIs ==========

    /// Export form field data to FDF format.
    ///
    /// Writes all form field data (original and modified) to an FDF file.
    /// This is useful for data extraction, backup, or batch processing.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path to write the FDF file
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("filled_form.pdf")?;
    /// editor.export_form_data_fdf("form_data.fdf")?;
    /// ```
    pub fn export_form_data_fdf(&mut self, output_path: impl AsRef<std::path::Path>) -> Result<()> {
        use crate::extractors::forms::FormExtractor;
        FormExtractor::export_fdf(&self.source, output_path)
    }

    /// Export form field data to XFDF format.
    ///
    /// Writes all form field data (original and modified) to an XFDF (XML) file.
    /// XFDF is useful for web integration and human-readable data exchange.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path to write the XFDF file
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::editor::DocumentEditor;
    ///
    /// let mut editor = DocumentEditor::open("filled_form.pdf")?;
    /// editor.export_form_data_xfdf("form_data.xfdf")?;
    /// ```
    pub fn export_form_data_xfdf(
        &mut self,
        output_path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        use crate::extractors::forms::FormExtractor;
        FormExtractor::export_xfdf(&self.source, output_path)
    }

    /// Get widget annotation appearances for form flattening.
    ///
    /// Returns appearance data for Widget annotations only.
    /// Generates appearance streams for widgets that don't have them.
    fn get_widget_appearances(&mut self, page: usize) -> Result<Vec<AnnotationAppearance>> {
        use crate::annotation_types::AnnotationSubtype;

        let annotations = self.source.get_annotations(page)?;
        let mut appearances = Vec::new();

        for annotation in annotations {
            // Only process Widget annotations (form fields)
            if annotation.subtype_enum != AnnotationSubtype::Widget {
                continue;
            }

            // Skip annotations without a raw dictionary
            let raw_dict = match &annotation.raw_dict {
                Some(dict) => dict,
                None => continue,
            };

            // If this widget's value was changed via set_form_field_value, the
            // document's stored /AP is stale — it still renders the original
            // value or the empty placeholder the form writer never refreshed.
            // Regenerate the appearance from the new value so the flattened page
            // shows what was set, instead of baking the blank placeholder.
            if let Some(text) = self.modified_widget_text(&annotation) {
                // Values with CJK/emoji glyphs the /DA font cannot render are
                // regenerated with an embedded fallback font; pure-Latin values
                // return None here and fall through to the plain /DA path.
                if let Some(app) = self.regenerate_text_appearance_embedded(&annotation, &text)? {
                    appearances.push(app);
                    continue;
                }
                let mut updated = annotation.clone();
                match updated.field_type {
                    Some(crate::annotation_types::WidgetFieldType::Choice {
                        ref options, ..
                    }) => {
                        let options = options.clone();
                        updated.field_type =
                            Some(crate::annotation_types::WidgetFieldType::Choice {
                                options,
                                selected: Some(text),
                            });
                    },
                    _ => {
                        updated.field_value = Some(text);
                    },
                }
                if let Some(generated) = self.generate_widget_appearance(&updated)? {
                    appearances.push(generated);
                    continue;
                }
            }

            // Try to get appearance from AP dictionary
            let appearance_result = self.extract_widget_appearance(&annotation, raw_dict);

            match appearance_result {
                Ok(Some(appearance)) => appearances.push(appearance),
                Ok(None) => {
                    // No appearance stream - try to generate one
                    if let Some(generated) = self.generate_widget_appearance(&annotation)? {
                        appearances.push(generated);
                    } else {
                        // Widget has neither /AP nor a generatable appearance — flattening
                        // it produces a blank rectangle. Record field name as a warning.
                        let field_name = annotation
                            .raw_dict
                            .as_ref()
                            .and_then(|d| d.get("T"))
                            .and_then(|t| match t {
                                Object::String(s) => String::from_utf8(s.clone()).ok(),
                                _ => None,
                            })
                            .unwrap_or_else(|| format!("widget@page{}", page));
                        self.flatten_warnings.push(format!(
                            "field '{}' has no /AP appearance stream — flattening produces blank rectangle",
                            field_name
                        ));
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(appearances)
    }

    /// Look up the value set via `set_form_field_value` for the field that owns
    /// `annotation`, if it was modified in this editing session. Used by the
    /// flattener to regenerate a stale `/AP` from the new value.
    fn modified_widget_text(&self, annotation: &crate::annotations::Annotation) -> Option<String> {
        use crate::editor::form_fields::FormFieldValue;
        let field_name = annotation.field_name.as_deref()?;
        // The widget's /T is the partial name; match the modified field by exact
        // name first, then by trailing segment for hierarchical fields.
        let wrapper = self.modified_form_fields.get(field_name).or_else(|| {
            self.modified_form_fields
                .iter()
                .find(|(full, _)| full.rsplit('.').next() == Some(field_name))
                .map(|(_, w)| w)
        })?;
        match wrapper.value() {
            FormFieldValue::Text(s) | FormFieldValue::Choice(s) => Some(s),
            _ => None,
        }
    }

    /// Resolve an object one level of indirection deep against the source.
    fn resolve_obj(&self, o: &Object) -> Object {
        match o {
            Object::Reference(r) => self.source.load_object(*r).unwrap_or(Object::Null),
            other => other.clone(),
        }
    }

    /// The interactive form's default resource dictionary (`/DR`), resolved.
    fn acroform_default_resources(&self) -> Option<Object> {
        let cat = self.source.catalog().ok()?;
        let af = self.resolve_obj(cat.as_dict()?.get("AcroForm")?);
        let dr = af.as_dict()?.get("DR")?.clone();
        Some(self.resolve_obj(&dr))
    }

    /// The field's effective default-appearance string: the widget's own `/DA`,
    /// else the document-wide AcroForm `/DA` (ISO 32000-1 §12.7.3.3, inheritable).
    fn effective_da(&self, annotation: &crate::annotations::Annotation) -> Option<String> {
        if let Some(Object::String(s)) = annotation.raw_dict.as_ref().and_then(|d| d.get("DA")) {
            return Some(String::from_utf8_lossy(s).into_owned());
        }
        let cat = self.source.catalog().ok()?;
        let af = self.resolve_obj(cat.as_dict()?.get("AcroForm")?);
        match af.as_dict()?.get("DA") {
            Some(Object::String(s)) => Some(String::from_utf8_lossy(s).into_owned()),
            _ => None,
        }
    }

    /// Parse a `/DA` string into `(font resource name incl. '/', size, rgb)`.
    /// A size of 0 means auto-size; colour defaults to black.
    fn parse_da(da: &str) -> (String, f32, (f32, f32, f32)) {
        let toks: Vec<&str> = da.split_whitespace().collect();
        let mut font = "/Helv".to_string();
        let mut size = 0.0f32;
        let mut color = (0.0, 0.0, 0.0);
        for i in 0..toks.len() {
            match toks[i] {
                "Tf" if i >= 2 => {
                    if toks[i - 2].starts_with('/') {
                        font = toks[i - 2].to_string();
                    }
                    if let Ok(s) = toks[i - 1].parse::<f32>() {
                        size = s;
                    }
                },
                "g" if i >= 1 => {
                    if let Ok(v) = toks[i - 1].parse::<f32>() {
                        color = (v, v, v);
                    }
                },
                "rg" if i >= 3 => {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        toks[i - 3].parse::<f32>(),
                        toks[i - 2].parse::<f32>(),
                        toks[i - 1].parse::<f32>(),
                    ) {
                        color = (r, g, b);
                    }
                },
                _ => {},
            }
        }
        (font, size, color)
    }

    /// A self-contained standard-14 Helvetica font dictionary (no external
    /// dependencies, so it survives the garbage-collected full rewrite).
    fn helvetica_font_dict() -> Object {
        let mut d = HashMap::new();
        d.insert("Type".to_string(), Object::Name("Font".to_string()));
        d.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        d.insert("BaseFont".to_string(), Object::Name("Helvetica".to_string()));
        d.insert("Encoding".to_string(), Object::Name("WinAnsiEncoding".to_string()));
        Object::Dictionary(d)
    }

    /// Build the `/Resources` dict for a regenerated text appearance so the
    /// `/DA` font name resolves inside the flattened form XObject.
    ///
    /// The form's `/DR` fonts are referenced from the AcroForm, which is dropped
    /// when flattening (so its font objects are garbage-collected). We therefore
    /// *inline* a self-contained font dict under the `/DA` name: the document's
    /// own font when it is a standard-14 Type1 (no embedded program), otherwise
    /// a Helvetica stand-in. Non-Latin fonts that need an embedded program are
    /// handled by the fallback-embedding path.
    fn build_text_appearance_resources(&self, da_font_name: &str) -> Object {
        let name = da_font_name.trim_start_matches('/').to_string();

        let font_dict = self
            .acroform_default_resources()
            .and_then(|dr| dr.as_dict()?.get("Font").map(|f| self.resolve_obj(f)))
            .and_then(|fonts| fonts.as_dict()?.get(&name).map(|f| self.resolve_obj(f)))
            .filter(Self::is_self_contained_simple_font)
            .unwrap_or_else(Self::helvetica_font_dict);

        let mut fonts = HashMap::new();
        fonts.insert(name, font_dict);
        let mut res = HashMap::new();
        res.insert("Font".to_string(), Object::Dictionary(fonts));
        Object::Dictionary(res)
    }

    /// True for a standard-14 Type1 font dict with no embedded font program,
    /// i.e. one that can be inlined verbatim and stay self-contained.
    fn is_self_contained_simple_font(font: &Object) -> bool {
        let Some(d) = font.as_dict() else {
            return false;
        };
        if d.get("Subtype").and_then(|s| s.as_name()) != Some("Type1") {
            return false;
        }
        // A FontDescriptor implies an embedded program / extra object graph.
        !d.contains_key("FontDescriptor")
    }

    /// Extract appearance stream from a widget annotation.
    fn extract_widget_appearance(
        &mut self,
        annotation: &crate::annotations::Annotation,
        raw_dict: &HashMap<String, Object>,
    ) -> Result<Option<AnnotationAppearance>> {
        // Get the /AP (appearance) dictionary
        let ap_dict = match raw_dict.get("AP") {
            Some(Object::Dictionary(d)) => d.clone(),
            Some(Object::Reference(ap_ref)) => match self.source.load_object(*ap_ref)? {
                Object::Dictionary(d) => d,
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };

        // Get the /N (normal appearance) entry
        let normal_appearance = match ap_dict.get("N") {
            Some(obj) => obj.clone(),
            None => return Ok(None),
        };

        // Handle appearance states (e.g., /Yes and /Off for checkboxes)
        let (appearance_obj, appearance_ref) = match normal_appearance {
            Object::Reference(ref_obj) => {
                let obj = self.source.load_object(ref_obj)?;
                (obj, Some(ref_obj))
            },
            Object::Dictionary(ref dict) => {
                // Check if this is a Form XObject or a state dictionary
                if dict.get("Type").and_then(|t| t.as_name()) == Some("XObject") {
                    (Object::Dictionary(dict.clone()), None)
                } else {
                    // This is a state dictionary - get the current appearance state
                    let state = annotation.appearance_state.as_deref().unwrap_or("Off");
                    match dict.get(state) {
                        Some(Object::Reference(ref_obj)) => {
                            let obj = self.source.load_object(*ref_obj)?;
                            (obj, Some(*ref_obj))
                        },
                        Some(obj) => (obj.clone(), None),
                        None => {
                            // Try "Yes" as fallback for checkboxes
                            if state == "Off" {
                                return Ok(None); // Off state - skip
                            }
                            match dict.get("Yes") {
                                Some(Object::Reference(ref_obj)) => {
                                    let obj = self.source.load_object(*ref_obj)?;
                                    (obj, Some(*ref_obj))
                                },
                                Some(obj) => (obj.clone(), None),
                                None => return Ok(None),
                            }
                        },
                    }
                }
            },
            _ => return Ok(None),
        };

        // Extract Form XObject properties
        let form_dict = match appearance_obj.as_dict() {
            Some(d) => d,
            None => return Ok(None),
        };

        // Get BBox
        let bbox = match form_dict.get("BBox") {
            Some(Object::Array(arr)) if arr.len() >= 4 => {
                let values: Vec<f64> = arr
                    .iter()
                    .filter_map(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                    .collect();
                if values.len() >= 4 {
                    [
                        values[0] as f32,
                        values[1] as f32,
                        values[2] as f32,
                        values[3] as f32,
                    ]
                } else {
                    return Ok(None);
                }
            },
            _ => return Ok(None),
        };

        // Get Matrix (optional)
        let matrix = match form_dict.get("Matrix") {
            Some(Object::Array(arr)) if arr.len() >= 6 => {
                let values: Vec<f64> = arr
                    .iter()
                    .filter_map(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                    .collect();
                if values.len() >= 6 {
                    Some([
                        values[0] as f32,
                        values[1] as f32,
                        values[2] as f32,
                        values[3] as f32,
                        values[4] as f32,
                        values[5] as f32,
                    ])
                } else {
                    None
                }
            },
            _ => None,
        };

        // Get Resources
        let resources = form_dict.get("Resources").cloned();

        // Get the annotation's Rect
        let annot_rect = annotation.rect.unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let annot_rect = [
            annot_rect[0] as f32,
            annot_rect[1] as f32,
            annot_rect[2] as f32,
            annot_rect[3] as f32,
        ];

        // Get content stream bytes
        let content_bytes = if let Some(ref_obj) = appearance_ref {
            let stream_obj = self.source.load_object(ref_obj)?;
            match stream_obj.decode_stream_data() {
                Ok(data) => data,
                Err(_) => return Ok(None),
            }
        } else {
            match appearance_obj.decode_stream_data() {
                Ok(data) => data,
                Err(_) => return Ok(None),
            }
        };

        Ok(Some(AnnotationAppearance {
            content: content_bytes.to_vec(),
            bbox,
            annot_rect,
            matrix,
            resources,
            extra_objects: Vec::new(),
        }))
    }

    /// Escape a string for a PDF literal `(...)` text operand.
    fn escape_pdf_literal(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '(' => out.push_str("\\("),
                ')' => out.push_str("\\)"),
                '\r' => out.push_str("\\r"),
                '\n' => out.push_str("\\n"),
                _ => out.push(c),
            }
        }
        out
    }

    /// Regenerate a *text* widget appearance whose value contains non-Latin or
    /// emoji characters by embedding bundled fallback fonts and emitting
    /// CID-keyed text (ISO 32000-1 §12.7.3.3 + §9.7 composite fonts).
    ///
    /// Returns `Ok(None)` when no fallback is needed (pure Latin-1) or available
    /// (the `cjk-form-fonts` feature is off, or a font fails to parse/embed), so
    /// the caller falls back to the plain `/DA` appearance path.
    fn regenerate_text_appearance_embedded(
        &mut self,
        annotation: &crate::annotations::Annotation,
        text: &str,
    ) -> Result<Option<AnnotationAppearance>> {
        use crate::annotation_types::WidgetFieldType;
        use crate::fonts::form_fallback;

        match annotation.field_type {
            Some(WidgetFieldType::Text)
            | Some(WidgetFieldType::Choice { .. })
            | Some(WidgetFieldType::Button) => {},
            _ => return Ok(None),
        }
        if text.is_empty() || !text.chars().any(|c| form_fallback::classify(c).is_some()) {
            return Ok(None);
        }

        #[cfg(not(feature = "cjk-form-fonts"))]
        {
            self.flatten_warnings.push(
                "field value contains non-Latin/emoji characters the /DA font cannot render; \
                 rebuild with the `cjk-form-fonts` feature to embed a fallback font when flattening"
                    .to_string(),
            );
            Ok(None)
        }

        #[cfg(feature = "cjk-form-fonts")]
        {
            self.embed_fallback_text_appearance(annotation, text)
        }
    }

    /// The `cjk-form-fonts`-gated body of [`Self::regenerate_text_appearance_embedded`].
    #[cfg(feature = "cjk-form-fonts")]
    fn embed_fallback_text_appearance(
        &mut self,
        annotation: &crate::annotations::Annotation,
        text: &str,
    ) -> Result<Option<AnnotationAppearance>> {
        use crate::fonts::form_fallback::{self, Fallback};
        use crate::writer::{build_embedded_font_objects, EmbeddedFont};
        use std::collections::BTreeMap;

        let rect = match annotation.rect {
            Some(r) => r,
            None => return Ok(None),
        };
        let width = (rect[2] - rect[0]) as f32;
        let height = (rect[3] - rect[1]) as f32;

        let (da_font, da_size, da_color) =
            Self::parse_da(&self.effective_da(annotation).unwrap_or_default());
        let font_size = if da_size > 0.0 {
            da_size
        } else {
            (height * 0.7).clamp(6.0, 12.0)
        };

        // Resource dict seeded with the /DA font (used for any Latin runs).
        let mut font_res: HashMap<String, Object> = HashMap::new();
        if let Object::Dictionary(res) = self.build_text_appearance_resources(&da_font) {
            if let Some(Object::Dictionary(f)) = res.get("Font") {
                font_res = f.clone();
            }
        }

        // Parse the value into runs, and build one EmbeddedFont per fallback
        // kind present, recording the glyphs each run uses.
        let runs = form_fallback::split_runs(text);
        let mut fonts: BTreeMap<Fallback, EmbeddedFont> = BTreeMap::new();
        for run in &runs {
            if let Some(kind) = run.fallback {
                if !fonts.contains_key(&kind) {
                    match EmbeddedFont::from_data(None, form_fallback::font_bytes(kind).to_vec()) {
                        Ok(f) => {
                            fonts.insert(kind, f);
                        },
                        // A font that won't parse → give up, use the /DA path.
                        Err(_) => return Ok(None),
                    }
                }
            }
        }
        // Per run, the original-face glyph IDs (for fallback runs).
        let mut run_gids: Vec<Vec<u16>> = Vec::with_capacity(runs.len());
        for run in &runs {
            match run.fallback {
                Some(kind) => {
                    let gids = fonts
                        .get_mut(&kind)
                        .expect("font present")
                        .encode_string(&run.text);
                    run_gids.push(gids);
                },
                None => run_gids.push(Vec::new()),
            }
        }

        // Build the embedded-font object graph for each kind and remember the
        // subset remapper + the resource name we registered it under.
        let mut extra_objects: Vec<(u32, Object)> = Vec::new();
        let mut placed: BTreeMap<Fallback, (String, crate::fonts::GlyphRemapper)> = BTreeMap::new();
        for (kind, font) in fonts.iter_mut() {
            let (ids, objs, remapper) =
                build_embedded_font_objects(font, || self.allocate_object_id())?;
            let res_name = match kind {
                Fallback::Cjk => "FbCJK",
                Fallback::Emoji => "FbEmoji",
            };
            font_res.insert(res_name.to_string(), Object::Reference(ObjectRef::new(ids.type0, 0)));
            extra_objects.extend(objs);
            placed.insert(*kind, (res_name.to_string(), remapper));
        }

        // Emit the appearance content per ISO 32000-1 §12.7.3.3.
        let mut content = String::new();
        content.push_str("/Tx BMC\nq\nBT\n");
        let (r, g, b) = da_color;
        content.push_str(&format!("{} {} {} rg\n", r, g, b));
        let y = ((height - font_size) / 2.0).max(2.0);
        content.push_str(&format!("2 {} Td\n", y));

        for (run, gids) in runs.iter().zip(run_gids.iter()) {
            match run.fallback {
                None => {
                    content.push_str(&format!("{} {} Tf\n", da_font, font_size));
                    content.push_str(&format!("({}) Tj\n", Self::escape_pdf_literal(&run.text)));
                },
                Some(kind) => {
                    let (res_name, remapper) = placed.get(&kind).expect("placed font");
                    content.push_str(&format!("/{} {} Tf\n", res_name, font_size));
                    content.push('<');
                    for &orig in gids {
                        let sub = remapper.get(orig).unwrap_or(orig);
                        content.push_str(&format!("{:04X}", sub));
                    }
                    content.push_str("> Tj\n");
                },
            }
        }
        content.push_str("ET\nQ\nEMC\n");

        let mut res = HashMap::new();
        res.insert("Font".to_string(), Object::Dictionary(font_res));

        Ok(Some(AnnotationAppearance {
            content: content.into_bytes(),
            bbox: [0.0, 0.0, width, height],
            annot_rect: [
                rect[0] as f32,
                rect[1] as f32,
                rect[2] as f32,
                rect[3] as f32,
            ],
            matrix: None,
            resources: Some(Object::Dictionary(res)),
            extra_objects,
        }))
    }

    /// Generate appearance stream for a widget without one.
    fn generate_widget_appearance(
        &self,
        annotation: &crate::annotations::Annotation,
    ) -> Result<Option<AnnotationAppearance>> {
        use crate::annotation_types::WidgetFieldType;
        use crate::geometry::Rect;
        use crate::writer::FormAppearanceGenerator;

        let rect = match annotation.rect {
            Some(r) => r,
            None => return Ok(None),
        };

        let annot_rect = [
            rect[0] as f32,
            rect[1] as f32,
            rect[2] as f32,
            rect[3] as f32,
        ];
        let width = annot_rect[2] - annot_rect[0];
        let height = annot_rect[3] - annot_rect[1];
        let geom_rect = Rect::new(0.0, 0.0, width, height);

        // Text-bearing fields regenerate per ISO 32000-1 §12.7.3.3: the font
        // name, size, and colour come from the field's /DA, and the appearance
        // /Resources are built from the form's /DR. A text-only generator (no
        // opaque background or border) avoids painting over page content the
        // field rect may overlap.
        let field_type = annotation.field_type.as_ref();
        let shape_generator = FormAppearanceGenerator::new()
            .with_background(1.0, 1.0, 1.0)
            .with_border(1.0, 0.0, 0.0, 0.0);
        let text_generator = FormAppearanceGenerator::new();

        let (da_font, da_size, da_color) =
            Self::parse_da(&self.effective_da(annotation).unwrap_or_default());
        // Auto-size (/DA size 0) → fit the annotation height with padding.
        let font_size = if da_size > 0.0 {
            da_size
        } else {
            (height * 0.7).clamp(6.0, 12.0)
        };

        let mut text_resources: Option<Object> = None;
        let content_str = match field_type {
            Some(WidgetFieldType::Text) => {
                let text = annotation.field_value.as_deref().unwrap_or("");
                text_resources = Some(self.build_text_appearance_resources(&da_font));
                text_generator.text_field_appearance(geom_rect, text, &da_font, font_size, da_color)
            },
            Some(WidgetFieldType::Checkbox { checked }) => {
                if *checked {
                    shape_generator.checkbox_on_appearance(geom_rect, (0.0, 0.0, 0.0))
                } else {
                    shape_generator.checkbox_off_appearance(geom_rect)
                }
            },
            Some(WidgetFieldType::Radio { selected }) => {
                if selected.is_some() {
                    shape_generator.radio_on_appearance(geom_rect, (0.0, 0.0, 0.0))
                } else {
                    shape_generator.radio_off_appearance(geom_rect)
                }
            },
            Some(WidgetFieldType::Button) => {
                let caption = annotation.field_value.as_deref().unwrap_or("");
                text_resources = Some(self.build_text_appearance_resources(&da_font));
                text_generator.button_appearance(geom_rect, caption, &da_font, font_size, da_color)
            },
            Some(WidgetFieldType::Choice { selected, .. }) => {
                let text = selected.as_deref().unwrap_or("");
                text_resources = Some(self.build_text_appearance_resources(&da_font));
                text_generator.text_field_appearance(geom_rect, text, &da_font, font_size, da_color)
            },
            Some(WidgetFieldType::Signature) | Some(WidgetFieldType::Unknown) | None => {
                return Ok(None);
            },
        };

        let content_bytes = content_str.into_bytes();
        let bbox = [0.0, 0.0, width, height];

        Ok(Some(AnnotationAppearance {
            content: content_bytes,
            bbox,
            annot_rect,
            matrix: None,
            resources: text_resources,
            extra_objects: Vec::new(),
        }))
    }

    /// Get annotation appearance stream data for flattening.
    ///
    /// Returns a list of (content_bytes, bbox, resources) for each annotation
    /// that has an appearance stream.
    fn get_annotation_appearances(&mut self, page: usize) -> Result<Vec<AnnotationAppearance>> {
        let annotations = self.source.get_annotations(page)?;
        let mut appearances = Vec::new();

        for annotation in annotations {
            // Skip annotations without a raw dictionary
            let raw_dict = match &annotation.raw_dict {
                Some(dict) => dict,
                None => continue,
            };

            // Get the /AP (appearance) dictionary
            let ap_dict = match raw_dict.get("AP") {
                Some(Object::Dictionary(d)) => d.clone(),
                Some(Object::Reference(ap_ref)) => match self.source.load_object(*ap_ref)? {
                    Object::Dictionary(d) => d,
                    _ => continue,
                },
                _ => continue,
            };

            // Get the /N (normal appearance) entry
            let normal_appearance = match ap_dict.get("N") {
                Some(obj) => obj.clone(),
                None => continue,
            };

            // The normal appearance can be:
            // 1. A reference to a Form XObject
            // 2. A dictionary of appearance states (e.g., for checkboxes: /Yes, /Off)
            let (appearance_obj, appearance_ref) = match normal_appearance {
                Object::Reference(ref_obj) => {
                    let obj = self.source.load_object(ref_obj)?;
                    (obj, Some(ref_obj))
                },
                Object::Dictionary(ref dict) => {
                    // Check if this is a Form XObject or a state dictionary
                    if dict.get("Type").and_then(|t| t.as_name()) == Some("XObject") {
                        (Object::Dictionary(dict.clone()), None)
                    } else {
                        // This is a state dictionary - get the current appearance state
                        let state = annotation.appearance_state.as_deref().unwrap_or("Off");
                        match dict.get(state) {
                            Some(Object::Reference(ref_obj)) => {
                                let obj = self.source.load_object(*ref_obj)?;
                                (obj, Some(*ref_obj))
                            },
                            Some(obj) => (obj.clone(), None),
                            None => continue,
                        }
                    }
                },
                _ => continue,
            };

            // Extract the Form XObject properties
            let form_dict = match appearance_obj.as_dict() {
                Some(d) => d,
                None => continue,
            };

            // Get BBox
            let bbox = match form_dict.get("BBox") {
                Some(Object::Array(arr)) if arr.len() >= 4 => {
                    let values: Vec<f64> = arr
                        .iter()
                        .filter_map(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                        .collect();
                    if values.len() >= 4 {
                        [
                            values[0] as f32,
                            values[1] as f32,
                            values[2] as f32,
                            values[3] as f32,
                        ]
                    } else {
                        continue;
                    }
                },
                _ => continue,
            };

            // Get Matrix (optional, defaults to identity)
            let matrix = match form_dict.get("Matrix") {
                Some(Object::Array(arr)) if arr.len() >= 6 => {
                    let values: Vec<f64> = arr
                        .iter()
                        .filter_map(|o| o.as_real().or_else(|| o.as_integer().map(|i| i as f64)))
                        .collect();
                    if values.len() >= 6 {
                        Some([
                            values[0] as f32,
                            values[1] as f32,
                            values[2] as f32,
                            values[3] as f32,
                            values[4] as f32,
                            values[5] as f32,
                        ])
                    } else {
                        None
                    }
                },
                _ => None,
            };

            // Get Resources (optional)
            let resources = form_dict.get("Resources").cloned();

            // Get the annotation's Rect (position on page)
            let annot_rect = annotation.rect.unwrap_or([0.0, 0.0, 0.0, 0.0]);
            let annot_rect = [
                annot_rect[0] as f32,
                annot_rect[1] as f32,
                annot_rect[2] as f32,
                annot_rect[3] as f32,
            ];

            // Get the content stream bytes
            let content_bytes = if let Some(ref_obj) = appearance_ref {
                // Load the object and decode its stream data
                let stream_obj = match self.source.load_object(ref_obj) {
                    Ok(obj) => obj,
                    Err(_) => continue,
                };
                match stream_obj.decode_stream_data() {
                    Ok(data) => data,
                    Err(_) => continue,
                }
            } else {
                // Inline stream - try to decode directly
                match appearance_obj.decode_stream_data() {
                    Ok(data) => data,
                    Err(_) => continue,
                }
            };

            appearances.push(AnnotationAppearance {
                content: content_bytes,
                bbox,
                annot_rect,
                matrix,
                resources,
                extra_objects: Vec::new(),
            });
        }

        Ok(appearances)
    }

    /// Generate content stream to render flattened annotations.
    ///
    /// This creates PDF operators that invoke each annotation's appearance
    /// as a Form XObject at the correct position.
    fn generate_flatten_overlay(
        &self,
        appearances: &[AnnotationAppearance],
        xobject_names: &[String],
    ) -> Vec<u8> {
        let mut content = Vec::new();

        for (appearance, xobj_name) in appearances.iter().zip(xobject_names.iter()) {
            // Save graphics state
            content.extend_from_slice(b"q\n");

            // Calculate transformation to position the XObject
            // The appearance is defined in BBox coordinates and needs to be
            // positioned at annot_rect on the page.
            let bbox = appearance.bbox;
            let rect = appearance.annot_rect;

            // Calculate scale and translation
            let bbox_width = bbox[2] - bbox[0];
            let bbox_height = bbox[3] - bbox[1];
            let rect_width = rect[2] - rect[0];
            let rect_height = rect[3] - rect[1];

            // Avoid division by zero
            let sx = if bbox_width != 0.0 {
                rect_width / bbox_width
            } else {
                1.0
            };
            let sy = if bbox_height != 0.0 {
                rect_height / bbox_height
            } else {
                1.0
            };

            // Translation to position the XObject
            let tx = rect[0] - bbox[0] * sx;
            let ty = rect[1] - bbox[1] * sy;

            // Apply transformation matrix: [sx 0 0 sy tx ty]
            content.extend_from_slice(
                format!("{:.6} 0 0 {:.6} {:.6} {:.6} cm\n", sx, sy, tx, ty).as_bytes(),
            );

            // If the appearance has its own matrix, apply it
            if let Some(m) = appearance.matrix {
                content.extend_from_slice(
                    format!(
                        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n",
                        m[0], m[1], m[2], m[3], m[4], m[5]
                    )
                    .as_bytes(),
                );
            }

            // Invoke the XObject
            content.extend_from_slice(format!("/{} Do\n", xobj_name).as_bytes());

            // Restore graphics state
            content.extend_from_slice(b"Q\n");
        }

        content
    }

    // ========================================================================
    // Redaction Application
    // ========================================================================

    /// Mark a page for redaction application.
    ///
    /// When the document is saved, redaction annotations on this page will be
    /// applied: content will be visually obscured and the redaction annotations
    /// removed.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Apply redactions on page 0
    /// editor.apply_page_redactions(0)?;
    /// editor.save("output.pdf")?;
    /// ```
    /// Queue an explicit rectangular redaction on `page` (zero-based,
    /// output order) in page user-space points `[x0, y0, x1, y1]`.
    ///
    /// The region is applied by [`apply_redactions_destructive`], which
    /// physically removes the underlying text — not a cosmetic overlay
    /// (#231, ISO 32000-1:2008 §12.5.6.23). `fill` is the overlay colour
    /// (DeviceRGB); `None` uses the configured default.
    ///
    /// [`apply_redactions_destructive`]: DocumentEditor::apply_redactions_destructive
    ///
    /// # Errors
    /// [`Error::InvalidPdf`] if `page` is out of range.
    pub fn add_redaction(
        &mut self,
        page: usize,
        rect: [f32; 4],
        fill: Option<[f32; 3]>,
    ) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }
        let source_page = self.output_to_source_index(page);
        self.redaction_regions.entry(source_page).or_default().push(
            crate::redaction::RedactionRegion::from_rect(rect[0], rect[1], rect[2], rect[3], fill),
        );
        self.apply_redactions_pages.insert(source_page);
        self.is_modified = true;
        Ok(())
    }

    /// Number of redaction regions queued for `page` (zero-based, output
    /// order): `/Redact` annotations in the source plus programmatic
    /// [`add_redaction`] rectangles.
    ///
    /// [`add_redaction`]: DocumentEditor::add_redaction
    ///
    /// # Errors
    /// [`Error::InvalidPdf`] if `page` is out of range.
    pub fn redaction_count(&mut self, page: usize) -> Result<usize> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }
        let source_page = self.output_to_source_index(page);
        let annots = self.get_redaction_data(source_page)?.len();
        let prog = self
            .redaction_regions
            .get(&source_page)
            .map_or(0, |v| v.len());
        Ok(annots + prog)
    }

    /// Decode a source page's `/Contents` to raw bytes (the `/Contents`
    /// array, if any, is the logical concatenation of its streams,
    /// ISO 32000-1:2008 §7.8.2). Empty if the page has no contents.
    fn get_page_content_bytes(&mut self, source_page: usize) -> Result<Vec<u8>> {
        let page_ref = self.source.get_page_ref(source_page)?;
        let page_obj = self.source.load_object(page_ref)?;
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;
        let contents = match page_dict.get("Contents") {
            Some(c) => c.clone(),
            None => return Ok(Vec::new()),
        };
        match contents {
            Object::Reference(r) => Ok(self.source.load_object(r)?.decode_stream_data()?),
            Object::Array(arr) => {
                let mut data = Vec::new();
                for item in arr {
                    if let Object::Reference(r) = item {
                        if let Ok(s) = self.source.load_object(r)?.decode_stream_data() {
                            data.extend_from_slice(&s);
                            data.push(b'\n');
                        }
                    }
                }
                Ok(data)
            },
            _ => Ok(Vec::new()),
        }
    }

    /// Build the [`crate::redaction::FontInfoMetrics`] for a source page
    /// from its (possibly inherited) `/Resources/Font`. A font that fails
    /// to parse is simply omitted — the engine then reports it
    /// non-simple and *refuses* rather than under-redacting.
    fn build_page_font_metrics(
        &mut self,
        source_page: usize,
    ) -> Result<crate::redaction::FontInfoMetrics> {
        use std::collections::HashMap as Map;
        let mut map: Map<String, std::sync::Arc<crate::fonts::FontInfo>> = Map::new();

        // Resolve /Resources: the page's own, else inherited from the
        // /Pages chain (ISO 32000-1:2008 §7.7.3.4 inheritable attributes).
        // `rendering`-gated `get_page_resources` is unavailable in the
        // default build, so resolve it here. No resources ⇒ empty map ⇒
        // the engine reports unknown fonts non-simple and *refuses*
        // (fail closed, never under-redact).
        let page_ref = self.source.get_page_ref(source_page)?;
        let mut node = self.source.load_object(page_ref).ok();
        let mut resources = Object::Null;
        let mut guard = 0;
        while let Some(obj) = node {
            guard += 1;
            if guard > 64 {
                break; // cyclic /Parent — bail (fail closed)
            }
            let Some(dict) = obj.as_dict() else { break };
            if let Some(res) = dict.get("Resources") {
                resources = match res.as_reference() {
                    Some(r) => self.source.load_object(r).unwrap_or(Object::Null),
                    None => res.clone(),
                };
                break;
            }
            node = dict
                .get("Parent")
                .and_then(|p| p.as_reference())
                .and_then(|r| self.source.load_object(r).ok());
        }

        if let Some(res_dict) = resources.as_dict() {
            if let Some(font_obj) = res_dict.get("Font") {
                let font_dict_obj = match font_obj.as_reference() {
                    Some(r) => self.source.load_object(r)?,
                    None => font_obj.clone(),
                };
                if let Some(font_dict) = font_dict_obj.as_dict() {
                    for (name, val) in font_dict {
                        let resolved = match val.as_reference() {
                            Some(r) => match self.source.load_object(r) {
                                Ok(o) => o,
                                Err(_) => continue,
                            },
                            None => val.clone(),
                        };
                        if let Ok(fi) = crate::fonts::FontInfo::from_dict(&resolved, &self.source) {
                            map.insert(name.clone(), std::sync::Arc::new(fi));
                        }
                    }
                }
            }
        }
        Ok(crate::redaction::FontInfoMetrics::new(map))
    }

    /// Destructively apply every queued redaction (programmatic
    /// rectangles + source `/Redact` annotations): the underlying text
    /// is physically removed from the content stream and an opaque
    /// overlay drawn over the area (ISO 32000-1:2008 §12.5.6.23 — "remove
    /// all traces"). The redacted pages' content is rewritten so that a
    /// subsequent garbage-collected full-rewrite save (the default
    /// [`save_to_bytes`]/[`save`]) leaves no residual recoverable bytes
    /// (G6); the `/Redact` annotations are removed.
    ///
    /// Returns an aggregate [`crate::redaction::RedactionReport`].
    ///
    /// [`save_to_bytes`]: DocumentEditor::save_to_bytes
    /// [`save`]: DocumentEditor::save
    ///
    /// # Errors
    /// - [`Error::Unsupported`] if a redacted page shows text in a
    ///   composite/Type0/unknown font (refused rather than risk a silent
    ///   under-redaction — fail closed).
    /// - [`Error::InvalidPdf`]/[`Error::ParseError`] on an unreadable
    ///   page or content stream.
    pub fn apply_redactions_destructive(
        &mut self,
        opts: crate::redaction::RedactionOptions,
    ) -> Result<crate::redaction::RedactionReport> {
        let mut pages: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        pages.extend(self.apply_redactions_pages.iter().copied());
        pages.extend(self.redaction_regions.keys().copied());

        let mut total = crate::redaction::RedactionReport::default();
        for src in pages {
            let rep = self.apply_redactions_destructive_for_page(src, &opts)?;
            total.regions += rep.regions;
            total.glyphs_removed += rep.glyphs_removed;
            total.bytes_removed += rep.bytes_removed;
        }
        self.is_modified = true;
        Ok(total)
    }

    /// Destructively apply queued redactions (programmatic rectangles +
    /// source `/Redact` annotations) for a single source page. Shared by
    /// [`apply_redactions_destructive`] (all queued pages) and `save_page`
    /// (DOM edits to pre-existing content on one page).
    ///
    /// [`apply_redactions_destructive`]: DocumentEditor::apply_redactions_destructive
    fn apply_redactions_destructive_for_page(
        &mut self,
        src: usize,
        opts: &crate::redaction::RedactionOptions,
    ) -> Result<crate::redaction::RedactionReport> {
        use crate::redaction::{redact_content_stream, RedactionRegion, RegionSet};
        let mut rs = RegionSet::new(src);
        for rd in self.get_redaction_data(src)? {
            rs.push(RedactionRegion::from_rect(
                rd.rect[0],
                rd.rect[1],
                rd.rect[2],
                rd.rect[3],
                Some(rd.color),
            ));
        }
        if let Some(progs) = self.redaction_regions.get(&src) {
            for r in progs {
                rs.push(*r);
            }
        }
        if rs.is_empty() {
            return Ok(crate::redaction::RedactionReport::default());
        }
        let content = self.get_page_content_bytes(src)?;
        if content.is_empty() {
            return Ok(crate::redaction::RedactionReport::default());
        }
        let fonts = self.build_page_font_metrics(src)?;
        let (bytes, rep) = redact_content_stream(&content, &rs, opts, &fonts)?;
        // Record the original /Contents object ids so the save path
        // hard-drops them (G6) — the secret must not survive even as
        // an orphaned, GC-missed object.
        if let Ok(page_ref) = self.source.get_page_ref(src) {
            if let Ok(page_obj) = self.source.load_object(page_ref) {
                if let Some(d) = page_obj.as_dict() {
                    match d.get("Contents") {
                        Some(Object::Reference(r)) => {
                            self.redacted_orphan_ids.insert(r.id);
                        },
                        Some(Object::Array(arr)) => {
                            for it in arr {
                                if let Object::Reference(r) = it {
                                    self.redacted_orphan_ids.insert(r.id);
                                }
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
        self.redacted_content.insert(src, bytes);
        self.apply_redactions_pages.insert(src);
        self.is_modified = true;
        Ok(rep)
    }

    /// Standalone document sanitization without geometric redaction
    /// (#231 T10, feature plan §4.6 / §5.1).
    ///
    /// Strips document-level secrets that cosmetic redaction left behind:
    /// the `/Info` dictionary, the catalog XMP `/Metadata` stream,
    /// document JavaScript (`/OpenAction`, `/AA`, `/Names/JavaScript`)
    /// and `/Names/EmbeddedFiles`, subject to the
    /// [`crate::redaction::RedactionOptions`] toggles. The removed object
    /// subtrees are hard-excluded from the
    /// output (G6) so a secret cannot survive even as a GC-missed
    /// orphan, and `/Info` is replaced with an empty dictionary.
    ///
    /// Returns a [`crate::redaction::RedactionReport`]; `bytes_removed`
    /// is a best-effort total of the dropped objects' sizes so callers
    /// can assert the scrub did real work.
    ///
    /// # Errors
    /// [`Error::InvalidPdf`] if the document has no resolvable catalog.
    pub fn sanitize_document(
        &mut self,
        opts: crate::redaction::RedactionOptions,
    ) -> Result<crate::redaction::RedactionReport> {
        use std::collections::{HashSet, VecDeque};

        let catalog_ref = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Root"))
            .and_then(|r| r.as_reference())
            .ok_or_else(|| Error::InvalidPdf("Missing catalog reference".to_string()))?;
        let info_root = self
            .source
            .trailer()
            .as_dict()
            .and_then(|d| d.get("Info"))
            .and_then(|r| r.as_reference())
            .map(|r| r.id);

        let catalog_dict = match self.modified_objects.get(&catalog_ref.id) {
            Some(obj) => obj.clone(),
            None => self.source.catalog()?,
        };
        let catalog_dict = catalog_dict
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Catalog is not a dictionary".to_string()))?
            .clone();

        let scrub = crate::redaction::sanitize_catalog(&catalog_dict, &opts, |id| {
            self.source.load_object(ObjectRef { id, gen: 0 }).ok()
        });
        let counts = scrub.counts;
        let removed_roots = scrub.removed_roots.clone();

        // Stage the scrubbed catalog and the cleared /Info first so the
        // post-scrub reachability set reflects them.
        self.modified_objects
            .insert(catalog_ref.id, Object::Dictionary(scrub.catalog));
        if opts.scrub_metadata {
            self.modified_info = Some(DocumentInfo::default());
        }

        let reachable = self.collect_reachable_ids();

        // Expand the catalog-derived roots → full subtree, excluding
        // anything still reachable after the scrub (a genuinely shared
        // object — e.g. a filespec also referenced by a file-attachment
        // annotation — must not be dropped).
        let mut to_drop: HashSet<u32> = HashSet::new();
        let mut queue: VecDeque<u32> = removed_roots.into_iter().collect();
        let mut bytes_removed: u64 = 0;
        let account = |obj: &Object, b: &mut u64| {
            *b += match obj {
                Object::Stream { data, .. } => data.len() as u64 + 64,
                _ => 48,
            };
        };
        while let Some(id) = queue.pop_front() {
            if id == 0 || reachable.contains(&id) || !to_drop.insert(id) {
                continue;
            }
            if let Ok(obj) = self.source.load_object(ObjectRef { id, gen: 0 }) {
                account(&obj, &mut bytes_removed);
                fn walk(o: &Object, q: &mut VecDeque<u32>) {
                    match o {
                        Object::Reference(r) => q.push_back(r.id),
                        Object::Array(a) => a.iter().for_each(|x| walk(x, q)),
                        Object::Dictionary(d) => d.values().for_each(|x| walk(x, q)),
                        Object::Stream { dict, .. } => dict.values().for_each(|x| walk(x, q)),
                        _ => {},
                    }
                }
                walk(&obj, &mut queue);
            }
        }

        // The original /Info object is *replaced* by an empty dict, so it
        // must be hard-excluded unconditionally — `collect_reachable_ids`
        // always seeds the source trailer /Info, so the reachable-subtract
        // guard above would wrongly protect the secret-bearing original
        // (same defense-in-depth as redacted content streams, G6).
        if opts.scrub_metadata {
            if let Some(id) = info_root {
                if to_drop.insert(id) {
                    if let Ok(obj) = self.source.load_object(ObjectRef { id, gen: 0 }) {
                        account(&obj, &mut bytes_removed);
                    }
                }
            }
        }

        self.redacted_orphan_ids.extend(to_drop.iter().copied());
        self.is_modified = true;

        Ok(crate::redaction::RedactionReport {
            annotations_removed: counts.total(),
            bytes_removed,
            ..crate::redaction::RedactionReport::default()
        })
    }

    /// Mark a page for (legacy) redaction application.
    ///
    /// Historically this drew a cosmetic overlay only. Prefer
    /// [`add_redaction`] + [`apply_redactions_destructive`] for true
    /// content removal (#231).
    ///
    /// [`add_redaction`]: DocumentEditor::add_redaction
    /// [`apply_redactions_destructive`]: DocumentEditor::apply_redactions_destructive
    ///
    /// # Errors
    /// [`Error::InvalidPdf`] if `page` is out of range.
    pub fn apply_page_redactions(&mut self, page: usize) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        self.apply_redactions_pages.insert(source_page);
        self.is_modified = true;
        Ok(())
    }

    /// Mark all pages for redaction application.
    pub fn apply_all_redactions(&mut self) -> Result<()> {
        let page_count = self.current_page_count();
        for page in 0..page_count {
            self.apply_redactions_pages.insert(page);
        }
        self.is_modified = true;
        Ok(())
    }

    /// Check if a page is marked for redaction application.
    pub fn is_page_marked_for_redaction(&self, page: usize) -> bool {
        let source_page = self.output_to_source_index(page);
        self.apply_redactions_pages.contains(&source_page)
    }

    /// Clear the apply redactions flag for a page.
    pub fn unmark_page_for_redaction(&mut self, page: usize) {
        let source_page = self.output_to_source_index(page);
        self.apply_redactions_pages.remove(&source_page);
    }

    /// Get redaction annotation data for a page.
    ///
    /// Returns a list of redaction areas with their fill colors.
    fn get_redaction_data(&mut self, page: usize) -> Result<Vec<RedactionData>> {
        use crate::annotation_types::AnnotationSubtype;

        let annotations = self.source.get_annotations(page)?;
        let mut redactions = Vec::new();

        for annotation in annotations {
            // Only process Redact annotations
            if annotation.subtype_enum != AnnotationSubtype::Redact {
                continue;
            }

            // Get the redaction rectangle
            let rect = match annotation.rect {
                Some(r) => [r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32],
                None => continue,
            };

            // Get interior color (IC entry) - the fill color for the redaction
            // Default to black if not specified
            let color = match &annotation.interior_color {
                Some(color) if color.len() >= 3 => {
                    [color[0] as f32, color[1] as f32, color[2] as f32]
                },
                _ => [0.0, 0.0, 0.0], // Default to black
            };

            // Also handle QuadPoints if present (multiple redaction areas)
            if let Some(ref quad_points) = annotation.quad_points {
                for quad in quad_points {
                    // QuadPoints are 8 values: x1,y1,x2,y2,x3,y3,x4,y4
                    // representing corners in a specific order
                    // Convert to bounding box
                    let xs = [quad[0], quad[2], quad[4], quad[6]];
                    let ys = [quad[1], quad[3], quad[5], quad[7]];

                    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min) as f32;
                    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max) as f32;
                    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min) as f32;
                    let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max) as f32;

                    redactions.push(RedactionData {
                        rect: [min_x, min_y, max_x, max_y],
                        color,
                    });
                }
            } else {
                // Just use the main Rect
                redactions.push(RedactionData { rect, color });
            }
        }

        Ok(redactions)
    }

    /// Generate content stream to draw redaction overlays.
    fn generate_redaction_overlay(&self, redactions: &[RedactionData]) -> Vec<u8> {
        let mut content = Vec::new();

        for redaction in redactions {
            // Save graphics state
            content.extend_from_slice(b"q\n");

            // Set fill color (RGB)
            content.extend_from_slice(
                format!(
                    "{:.3} {:.3} {:.3} rg\n",
                    redaction.color[0], redaction.color[1], redaction.color[2]
                )
                .as_bytes(),
            );

            // Draw filled rectangle
            let x = redaction.rect[0];
            let y = redaction.rect[1];
            let width = redaction.rect[2] - redaction.rect[0];
            let height = redaction.rect[3] - redaction.rect[1];

            content.extend_from_slice(
                format!("{:.2} {:.2} {:.2} {:.2} re f\n", x, y, width, height).as_bytes(),
            );

            // Restore graphics state
            content.extend_from_slice(b"Q\n");
        }

        content
    }

    // ========================================================================
    // Image Repositioning & Resizing
    // ========================================================================

    /// Get information about images on a page.
    ///
    /// Returns a list of images with their names, positions, and sizes.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    ///
    /// # Example
    ///
    /// ```ignore
    /// let images = editor.get_page_images(0)?;
    /// for img in images {
    ///     println!("Image {} at ({}, {}) size {}x{}",
    ///         img.name, img.bounds[0], img.bounds[1],
    ///         img.bounds[2], img.bounds[3]);
    /// }
    /// ```
    pub fn get_page_images(&mut self, page: usize) -> Result<Vec<ImageInfo>> {
        use crate::content::parser::parse_content_stream;

        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        // Get the original page index
        let original_page_idx = self.page_order[page];
        if original_page_idx < 0 {
            return Err(Error::InvalidPdf("Page has been deleted".to_string()));
        }

        // Get page reference
        let page_ref = self.source.get_page_ref(original_page_idx as usize)?;
        let page_obj = self.source.load_object(page_ref)?;
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;

        // Get Contents
        let contents = match page_dict.get("Contents") {
            Some(c) => c.clone(),
            None => return Ok(Vec::new()),
        };

        // Load content stream data
        let content_data = match contents {
            Object::Reference(ref_obj) => {
                let obj = self.source.load_object(ref_obj)?;
                obj.decode_stream_data()?
            },
            Object::Array(arr) => {
                // Concatenate multiple content streams
                let mut data = Vec::new();
                for item in arr {
                    if let Object::Reference(ref_obj) = item {
                        let obj = self.source.load_object(ref_obj)?;
                        if let Ok(stream_data) = obj.decode_stream_data() {
                            data.extend_from_slice(&stream_data);
                            data.push(b'\n');
                        }
                    }
                }
                data
            },
            _ => return Ok(Vec::new()),
        };

        // Parse the content stream
        let operators = parse_content_stream(&content_data)?;

        // Track CTM through the operators to find images
        let mut images = Vec::new();
        let mut ctm_stack: Vec<[f32; 6]> = vec![[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]]; // Identity
        let mut current_ctm = [1.0f32, 0.0, 0.0, 1.0, 0.0, 0.0];

        for op in operators {
            match op {
                crate::content::operators::Operator::SaveState => {
                    ctm_stack.push(current_ctm);
                },
                crate::content::operators::Operator::RestoreState => {
                    if let Some(saved) = ctm_stack.pop() {
                        current_ctm = saved;
                    }
                },
                crate::content::operators::Operator::Cm { a, b, c, d, e, f } => {
                    // Concatenate transformation matrix
                    // New CTM = [a,b,c,d,e,f] * current_ctm
                    let new_a = a * current_ctm[0] + b * current_ctm[2];
                    let new_b = a * current_ctm[1] + b * current_ctm[3];
                    let new_c = c * current_ctm[0] + d * current_ctm[2];
                    let new_d = c * current_ctm[1] + d * current_ctm[3];
                    let new_e = e * current_ctm[0] + f * current_ctm[2] + current_ctm[4];
                    let new_f = e * current_ctm[1] + f * current_ctm[3] + current_ctm[5];
                    current_ctm = [new_a, new_b, new_c, new_d, new_e, new_f];
                },
                crate::content::operators::Operator::Do { ref name } => {
                    // Check if this is an image XObject (vs Form XObject)
                    // For now, include all XObjects; a more refined implementation
                    // would check the XObject's Subtype
                    let matrix = current_ctm;

                    // Extract position and size from matrix
                    // Standard image matrix: [width, 0, 0, height, x, y]
                    let x = matrix[4];
                    let y = matrix[5];
                    // Width and height from scaling components
                    let width = (matrix[0] * matrix[0] + matrix[1] * matrix[1]).sqrt();
                    let height = (matrix[2] * matrix[2] + matrix[3] * matrix[3]).sqrt();

                    images.push(ImageInfo {
                        name: name.clone(),
                        bounds: [x, y, width, height],
                        matrix,
                    });
                },
                _ => {},
            }
        }

        Ok(images)
    }

    /// Reposition an image on a page.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    /// * `image_name` - The XObject name (e.g., "Im1")
    /// * `x` - New x position
    /// * `y` - New y position
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.reposition_image(0, "Im1", 100.0, 200.0)?;
    /// editor.save("output.pdf")?;
    /// ```
    pub fn reposition_image(
        &mut self,
        page: usize,
        image_name: &str,
        x: f32,
        y: f32,
    ) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        let page_mods = self.image_modifications.entry(source_page).or_default();
        let modification = page_mods
            .entry(image_name.to_string())
            .or_insert(ImageModification {
                x: None,
                y: None,
                width: None,
                height: None,
            });
        modification.x = Some(x);
        modification.y = Some(y);

        self.is_modified = true;
        Ok(())
    }

    /// Resize an image on a page.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    /// * `image_name` - The XObject name (e.g., "Im1")
    /// * `width` - New width
    /// * `height` - New height
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.resize_image(0, "Im1", 200.0, 150.0)?;
    /// editor.save("output.pdf")?;
    /// ```
    pub fn resize_image(
        &mut self,
        page: usize,
        image_name: &str,
        width: f32,
        height: f32,
    ) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        let page_mods = self.image_modifications.entry(source_page).or_default();
        let modification = page_mods
            .entry(image_name.to_string())
            .or_insert(ImageModification {
                x: None,
                y: None,
                width: None,
                height: None,
            });
        modification.width = Some(width);
        modification.height = Some(height);

        self.is_modified = true;
        Ok(())
    }

    /// Reposition and resize an image on a page.
    ///
    /// # Arguments
    /// * `page` - The zero-based page index
    /// * `image_name` - The XObject name (e.g., "Im1")
    /// * `x` - New x position
    /// * `y` - New y position
    /// * `width` - New width
    /// * `height` - New height
    pub fn set_image_bounds(
        &mut self,
        page: usize,
        image_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        if page >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!("Page index {} out of range", page)));
        }

        let source_page = self.output_to_source_index(page);
        let page_mods = self.image_modifications.entry(source_page).or_default();
        page_mods.insert(
            image_name.to_string(),
            ImageModification {
                x: Some(x),
                y: Some(y),
                width: Some(width),
                height: Some(height),
            },
        );

        self.is_modified = true;
        Ok(())
    }

    /// Clear image modifications for a page.
    pub fn clear_image_modifications(&mut self, page: usize) {
        self.image_modifications.remove(&page);
    }

    /// Check if a page has image modifications.
    pub fn has_image_modifications(&self, page: usize) -> bool {
        self.image_modifications
            .get(&page)
            .map(|m| !m.is_empty())
            .unwrap_or(false)
    }

    /// Rewrite content stream with image modifications applied.
    fn rewrite_content_stream_with_image_mods(
        &self,
        content_data: &[u8],
        modifications: &HashMap<String, ImageModification>,
    ) -> Result<Vec<u8>> {
        use crate::content::parser::parse_content_stream;

        let operators = parse_content_stream(content_data)?;
        let mut output = Vec::new();

        // Track the last cm operator to potentially modify it
        let mut i = 0;
        while i < operators.len() {
            let op = &operators[i];

            // Look for pattern: q ... cm ... Do ... Q
            // We need to find cm operators that precede Do operators
            match op {
                crate::content::operators::Operator::Cm { a, b, c, d, e, f } => {
                    // Look ahead to see if next relevant op is Do
                    let mut j = i + 1;
                    let mut found_do = None;
                    while j < operators.len() {
                        match &operators[j] {
                            crate::content::operators::Operator::Do { name } => {
                                found_do = Some(name.clone());
                                break;
                            },
                            crate::content::operators::Operator::RestoreState => break,
                            crate::content::operators::Operator::SaveState => break,
                            crate::content::operators::Operator::Cm { .. } => break,
                            _ => {},
                        }
                        j += 1;
                    }

                    if let Some(name) = found_do {
                        if let Some(modification) = modifications.get(&name) {
                            // Apply modification to the matrix
                            let new_a = modification.width.unwrap_or(*a);
                            let new_d = modification.height.unwrap_or(*d);
                            let new_e = modification.x.unwrap_or(*e);
                            let new_f = modification.y.unwrap_or(*f);

                            output.extend_from_slice(
                                format!(
                                    "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n",
                                    new_a, b, c, new_d, new_e, new_f
                                )
                                .as_bytes(),
                            );
                            i += 1;
                            continue;
                        }
                    }

                    // No modification, output as-is
                    output.extend_from_slice(
                        format!("{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n", a, b, c, d, e, f)
                            .as_bytes(),
                    );
                },
                _ => {
                    // Serialize the operator
                    self.serialize_operator(&mut output, op);
                },
            }
            i += 1;
        }

        Ok(output)
    }

    /// Serialize an operator to content-stream bytes.
    ///
    /// Delegates to [`crate::redaction::serialize`] — the single source
    /// of truth (DRY, #231 T1). Binary strings are emitted as
    /// hexadecimal per ISO 32000-1 §7.3.4.3 (binary-safe).
    fn serialize_operator(&self, output: &mut Vec<u8>, op: &crate::content::operators::Operator) {
        crate::redaction::serialize::serialize_operator(output, op);
    }

    /// Serialize a PDF object to bytes.
    ///
    /// Delegates to [`crate::redaction::serialize`] (single source of
    /// truth, #231 T1).
    fn serialize_object(&self, output: &mut Vec<u8>, obj: &crate::object::Object) {
        crate::redaction::serialize::serialize_object(output, obj);
    }
}

/// Data for a redaction area.
#[derive(Debug, Clone)]
struct RedactionData {
    /// Redaction rectangle [llx, lly, urx, ury]
    rect: [f32; 4],
    /// Fill color [r, g, b]
    color: [f32; 3],
}

impl EditableDocument for DocumentEditor {
    fn get_info(&mut self) -> Result<DocumentInfo> {
        // Return modified info if available
        if let Some(ref info) = self.modified_info {
            return Ok(info.clone());
        }

        // Otherwise, load from source document
        let trailer = self.source.trailer();
        if let Some(trailer_dict) = trailer.as_dict() {
            if let Some(info_ref) = trailer_dict.get("Info").and_then(|i| i.as_reference()) {
                let info_obj = self.source.load_object(info_ref)?;
                return Ok(DocumentInfo::from_object(&info_obj));
            }
        }

        // No Info dictionary
        Ok(DocumentInfo::default())
    }

    fn set_info(&mut self, info: DocumentInfo) -> Result<()> {
        self.modified_info = Some(info);
        self.is_modified = true;
        Ok(())
    }

    fn page_count(&mut self) -> Result<usize> {
        Ok(self.current_page_count())
    }

    fn get_page_info(&mut self, index: usize) -> Result<PageInfo> {
        let page_refs = self.get_page_refs()?;

        if index >= page_refs.len() {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of range (document has {} pages)",
                index,
                page_refs.len()
            )));
        }

        let page_ref = page_refs[index];
        let page_obj = self.source.load_object(page_ref)?;
        let page_dict = page_obj
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Page is not a dictionary".to_string()))?;

        // Get MediaBox for dimensions
        let (width, height) = if let Some(media_box) = page_dict.get("MediaBox") {
            self.parse_media_box(media_box)?
        } else {
            // Try to inherit from parent
            (612.0, 792.0) // Default to Letter size
        };

        let rotation = page_dict
            .get("Rotate")
            .and_then(|r| r.as_integer())
            .unwrap_or(0) as i32;

        Ok(PageInfo {
            index,
            width,
            height,
            rotation,
            object_ref: page_ref,
        })
    }

    fn remove_page(&mut self, index: usize) -> Result<()> {
        if index >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of range (document has {} pages)",
                index,
                self.current_page_count()
            )));
        }

        // Mark page as removed in page_order
        let mut visible_index = 0;
        for order in &mut self.page_order {
            if *order >= 0 {
                if visible_index == index {
                    *order = -1; // Mark as removed
                    break;
                }
                visible_index += 1;
            }
        }

        self.is_modified = true;
        Ok(())
    }

    fn move_page(&mut self, from: usize, to: usize) -> Result<()> {
        let count = self.current_page_count();
        if from >= count || to >= count {
            return Err(Error::InvalidPdf(format!(
                "Page index out of range (document has {} pages)",
                count
            )));
        }

        // Get current visible pages
        let visible: Vec<i32> = self
            .page_order
            .iter()
            .filter(|&&i| i >= 0)
            .copied()
            .collect();

        // Reorder
        let mut new_visible = visible.clone();
        let moved = new_visible.remove(from);
        new_visible.insert(to, moved);

        // Rebuild page_order
        self.page_order = new_visible;
        self.is_modified = true;
        Ok(())
    }

    fn duplicate_page(&mut self, index: usize) -> Result<usize> {
        if index >= self.current_page_count() {
            return Err(Error::InvalidPdf(format!(
                "Page index {} out of range (document has {} pages)",
                index,
                self.current_page_count()
            )));
        }

        // Get the original page index from page_order
        let visible: Vec<i32> = self
            .page_order
            .iter()
            .filter(|&&i| i >= 0)
            .copied()
            .collect();
        let original_index = visible[index];

        // Add duplicate reference
        self.page_order.push(original_index);
        self.is_modified = true;

        Ok(self.current_page_count() - 1)
    }

    fn save(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.save_with_options(path, SaveOptions::full_rewrite())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_with_options(&mut self, path: impl AsRef<Path>, options: SaveOptions) -> Result<()> {
        if options.incremental {
            self.write_incremental(path)
        } else {
            self.write_full(path, &options)
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save_with_options(&mut self, _path: impl AsRef<Path>, _options: SaveOptions) -> Result<()> {
        Err(Error::InvalidPdf(
            "Filesystem save is not available in WASM. Use save_to_bytes() instead.".to_string(),
        ))
    }
}

impl DocumentEditor {
    /// Parse a MediaBox array into (width, height).
    fn parse_media_box(&self, media_box: &Object) -> Result<(f32, f32)> {
        if let Some(arr) = media_box.as_array() {
            if arr.len() >= 4 {
                let llx = arr[0]
                    .as_real()
                    .or_else(|| arr[0].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0);
                let lly = arr[1]
                    .as_real()
                    .or_else(|| arr[1].as_integer().map(|i| i as f64))
                    .unwrap_or(0.0);
                let urx = arr[2]
                    .as_real()
                    .or_else(|| arr[2].as_integer().map(|i| i as f64))
                    .unwrap_or(612.0);
                let ury = arr[3]
                    .as_real()
                    .or_else(|| arr[3].as_integer().map(|i| i as f64))
                    .unwrap_or(792.0);

                return Ok(((urx - llx) as f32, (ury - lly) as f32));
            }
        }

        // Default to Letter size
        Ok((612.0, 792.0))
    }

    /// Generate a content stream from a StructureElement with marked content wrapping.
    ///
    /// This is used when writing modified structure elements back to a PDF.
    /// Wraps each element in BDC/EMC (Begin/End Marked Content) operators for tagged PDF support.
    ///
    /// Returns the content stream bytes and any pending images that need XObject registration.
    ///
    /// # PDF Spec Compliance
    ///
    /// - ISO 32000-1:2008, Section 14.7.4 - Marked Content Sequences
    fn generate_content_stream(
        &self,
        elem: &StructureElement,
    ) -> Result<(Vec<u8>, Vec<crate::writer::PendingImage>)> {
        let mut builder = ContentStreamBuilder::new();
        builder.add_structure_element(elem);
        let bytes = builder.build()?;
        let pending_images = builder.take_pending_images();
        Ok((bytes, pending_images))
    }

    /// Build an XObject stream from ImageContent.
    ///
    /// Creates a PDF Image XObject suitable for embedding in a PDF.
    /// Per PDF spec Section 8.9, images are represented as XObject streams.
    fn build_image_xobject(image: &crate::elements::ImageContent) -> Object {
        use crate::elements::{ColorSpace as ElemColorSpace, ImageFormat as ElemImageFormat};

        let mut dict = HashMap::new();

        dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        dict.insert("Width".to_string(), Object::Integer(image.width as i64));
        dict.insert("Height".to_string(), Object::Integer(image.height as i64));
        dict.insert(
            "BitsPerComponent".to_string(),
            Object::Integer(image.bits_per_component as i64),
        );

        // Map color space
        let color_space_name = match image.color_space {
            ElemColorSpace::Gray => "DeviceGray",
            ElemColorSpace::RGB => "DeviceRGB",
            ElemColorSpace::CMYK => "DeviceCMYK",
            ElemColorSpace::Indexed => "Indexed",
            ElemColorSpace::Lab => "Lab",
        };
        dict.insert("ColorSpace".to_string(), Object::Name(color_space_name.to_string()));

        // Set filter based on image format
        match image.format {
            ElemImageFormat::Jpeg => {
                dict.insert("Filter".to_string(), Object::Name("DCTDecode".to_string()));
            },
            ElemImageFormat::Png | ElemImageFormat::Raw => {
                dict.insert("Filter".to_string(), Object::Name("FlateDecode".to_string()));
            },
            ElemImageFormat::Jpeg2000 => {
                dict.insert("Filter".to_string(), Object::Name("JPXDecode".to_string()));
            },
            ElemImageFormat::Jbig2 => {
                dict.insert("Filter".to_string(), Object::Name("JBIG2Decode".to_string()));
            },
            ElemImageFormat::Unknown => {
                // No filter for unknown format (raw data)
            },
        }

        dict.insert("Length".to_string(), Object::Integer(image.data.len() as i64));

        Object::Stream {
            dict,
            data: image.data.clone().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_info_builder() {
        let info = DocumentInfo::new()
            .title("Test Document")
            .author("Test Author")
            .subject("Test Subject")
            .keywords("test, rust, pdf");

        assert_eq!(info.title, Some("Test Document".to_string()));
        assert_eq!(info.author, Some("Test Author".to_string()));
        assert_eq!(info.subject, Some("Test Subject".to_string()));
        assert_eq!(info.keywords, Some("test, rust, pdf".to_string()));
    }

    #[test]
    fn test_document_info_to_object() {
        let info = DocumentInfo::new().title("My PDF").author("John Doe");

        let obj = info.to_object();
        let dict = obj.as_dict().unwrap();

        assert!(dict.contains_key("Title"));
        assert!(dict.contains_key("Author"));
        assert!(!dict.contains_key("Subject"));
    }

    #[test]
    fn test_document_info_from_object() {
        let mut dict = HashMap::new();
        dict.insert("Title".to_string(), Object::String(b"Test Title".to_vec()));
        dict.insert("Author".to_string(), Object::String(b"Test Author".to_vec()));

        let obj = Object::Dictionary(dict);
        let info = DocumentInfo::from_object(&obj);

        assert_eq!(info.title, Some("Test Title".to_string()));
        assert_eq!(info.author, Some("Test Author".to_string()));
        assert_eq!(info.subject, None);
    }

    #[test]
    fn test_save_options() {
        let full = SaveOptions::full_rewrite();
        assert!(!full.incremental);
        assert!(full.compress);
        assert!(full.garbage_collect);

        let inc = SaveOptions::incremental();
        assert!(inc.incremental);
        assert!(!inc.compress);
        assert!(!inc.garbage_collect);
    }

    // =========================================================================
    // DocumentInfo: additional coverage
    // =========================================================================

    #[test]
    fn test_document_info_default_is_all_none() {
        let info = DocumentInfo::default();
        assert_eq!(info.title, None);
        assert_eq!(info.author, None);
        assert_eq!(info.subject, None);
        assert_eq!(info.keywords, None);
        assert_eq!(info.creator, None);
        assert_eq!(info.producer, None);
        assert_eq!(info.creation_date, None);
        assert_eq!(info.mod_date, None);
    }

    #[test]
    fn test_document_info_creator_producer() {
        let info = DocumentInfo::new()
            .creator("pdf_oxide")
            .producer("Rust PDF Library");
        assert_eq!(info.creator, Some("pdf_oxide".to_string()));
        assert_eq!(info.producer, Some("Rust PDF Library".to_string()));
    }

    #[test]
    fn test_document_info_to_object_all_fields() {
        let info = DocumentInfo {
            title: Some("Title".into()),
            author: Some("Author".into()),
            subject: Some("Subject".into()),
            keywords: Some("Keywords".into()),
            creator: Some("Creator".into()),
            producer: Some("Producer".into()),
            creation_date: Some("D:20260101000000".into()),
            mod_date: Some("D:20260226000000".into()),
        };

        let obj = info.to_object();
        let dict = obj.as_dict().unwrap();

        assert_eq!(dict.len(), 8);
        assert!(dict.contains_key("Title"));
        assert!(dict.contains_key("Author"));
        assert!(dict.contains_key("Subject"));
        assert!(dict.contains_key("Keywords"));
        assert!(dict.contains_key("Creator"));
        assert!(dict.contains_key("Producer"));
        assert!(dict.contains_key("CreationDate"));
        assert!(dict.contains_key("ModDate"));
    }

    #[test]
    fn test_document_info_to_object_empty() {
        let info = DocumentInfo::default();
        let obj = info.to_object();
        let dict = obj.as_dict().unwrap();
        assert!(dict.is_empty());
    }

    #[test]
    fn test_document_info_roundtrip() {
        let original = DocumentInfo {
            title: Some("My Title".into()),
            author: Some("My Author".into()),
            subject: Some("My Subject".into()),
            keywords: Some("a, b, c".into()),
            creator: Some("TestCreator".into()),
            producer: Some("TestProducer".into()),
            creation_date: Some("D:20260101".into()),
            mod_date: Some("D:20260226".into()),
        };

        let obj = original.to_object();
        let reconstructed = DocumentInfo::from_object(&obj);

        assert_eq!(original.title, reconstructed.title);
        assert_eq!(original.author, reconstructed.author);
        assert_eq!(original.subject, reconstructed.subject);
        assert_eq!(original.keywords, reconstructed.keywords);
        assert_eq!(original.creator, reconstructed.creator);
        assert_eq!(original.producer, reconstructed.producer);
        assert_eq!(original.creation_date, reconstructed.creation_date);
        assert_eq!(original.mod_date, reconstructed.mod_date);
    }

    #[test]
    fn test_document_info_from_non_dict() {
        // from_object on a non-dict should return all None
        let obj = Object::Integer(42);
        let info = DocumentInfo::from_object(&obj);
        assert_eq!(info.title, None);
        assert_eq!(info.author, None);
    }

    #[test]
    fn test_document_info_from_dict_with_wrong_types() {
        // Values that are not Object::String should be ignored
        let mut dict = HashMap::new();
        dict.insert("Title".to_string(), Object::Integer(123));
        dict.insert("Author".to_string(), Object::Boolean(true));

        let obj = Object::Dictionary(dict);
        let info = DocumentInfo::from_object(&obj);
        assert_eq!(info.title, None);
        assert_eq!(info.author, None);
    }

    // =========================================================================
    // SaveOptions: additional coverage
    // =========================================================================

    #[test]
    fn test_save_options_default() {
        let opts = SaveOptions::default();
        assert!(!opts.incremental);
        assert!(!opts.compress);
        assert!(!opts.linearize);
        assert!(!opts.garbage_collect);
        assert!(opts.encryption.is_none());
    }

    #[test]
    fn test_save_options_with_encryption() {
        let config = EncryptionConfig::new("user", "owner");
        let opts = SaveOptions::with_encryption(config);
        assert!(!opts.incremental);
        assert!(opts.compress);
        assert!(opts.garbage_collect);
        assert!(opts.encryption.is_some());
    }

    // =========================================================================
    // EncryptionAlgorithm
    // =========================================================================

    #[test]
    fn test_encryption_algorithm_default() {
        let algo = EncryptionAlgorithm::default();
        assert_eq!(algo, EncryptionAlgorithm::Aes256);
    }

    #[test]
    fn test_encryption_algorithm_variants() {
        let _ = EncryptionAlgorithm::Rc4_40;
        let _ = EncryptionAlgorithm::Rc4_128;
        let _ = EncryptionAlgorithm::Aes128;
        let _ = EncryptionAlgorithm::Aes256;
    }

    // =========================================================================
    // Permissions
    // =========================================================================

    #[test]
    fn test_permissions_all() {
        let perms = Permissions::all();
        assert!(perms.print);
        assert!(perms.print_high_quality);
        assert!(perms.modify);
        assert!(perms.copy);
        assert!(perms.annotate);
        assert!(perms.fill_forms);
        assert!(perms.accessibility);
        assert!(perms.assemble);
    }

    #[test]
    fn test_permissions_read_only() {
        let perms = Permissions::read_only();
        assert!(!perms.print);
        assert!(!perms.print_high_quality);
        assert!(!perms.modify);
        assert!(!perms.copy);
        assert!(!perms.annotate);
        assert!(!perms.fill_forms);
        assert!(perms.accessibility); // Always true for compliance
        assert!(!perms.assemble);
    }

    #[test]
    fn test_permissions_to_bits_all() {
        let perms = Permissions::all();
        let bits = perms.to_bits();

        // All permission bits should be set
        assert!(bits & (1 << 2) != 0); // print
        assert!(bits & (1 << 3) != 0); // modify
        assert!(bits & (1 << 4) != 0); // copy
        assert!(bits & (1 << 5) != 0); // annotate
        assert!(bits & (1 << 8) != 0); // fill_forms
        assert!(bits & (1 << 9) != 0); // accessibility
        assert!(bits & (1 << 10) != 0); // assemble
        assert!(bits & (1 << 11) != 0); // print_high_quality
    }

    #[test]
    fn test_permissions_to_bits_read_only() {
        let perms = Permissions::read_only();
        let bits = perms.to_bits();

        // Only accessibility should be set (plus reserved bits)
        assert!(bits & (1 << 2) == 0); // print not set
        assert!(bits & (1 << 3) == 0); // modify not set
        assert!(bits & (1 << 4) == 0); // copy not set
        assert!(bits & (1 << 5) == 0); // annotate not set
        assert!(bits & (1 << 8) == 0); // fill_forms not set
        assert!(bits & (1 << 9) != 0); // accessibility is set
        assert!(bits & (1 << 10) == 0); // assemble not set
        assert!(bits & (1 << 11) == 0); // print_high_quality not set
    }

    #[test]
    fn test_permissions_to_bits_reserved_bits() {
        // Reserved bits 7-8 (0-indexed: 6-7) and 13-32 (0-indexed: 12-31) must be 1
        let perms = Permissions::default();
        let bits = perms.to_bits();

        // Bits 6 and 7 must be set
        assert!(bits & (1 << 6) != 0);
        assert!(bits & (1 << 7) != 0);

        // Bits 12-31 must be set
        for bit in 12..32 {
            assert!(bits & (1 << bit) != 0, "bit {} should be set", bit);
        }
    }

    #[test]
    fn test_permissions_to_bits_individual_flags() {
        // Test each flag individually
        let mut perms = Permissions::default();
        let base_bits = perms.to_bits();

        perms.print = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 2));

        perms = Permissions::default();
        perms.modify = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 3));

        perms = Permissions::default();
        perms.copy = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 4));

        perms = Permissions::default();
        perms.annotate = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 5));

        perms = Permissions::default();
        perms.fill_forms = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 8));

        perms = Permissions::default();
        perms.assemble = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 10));

        perms = Permissions::default();
        perms.print_high_quality = true;
        assert_eq!(perms.to_bits(), base_bits | (1 << 11));
    }

    // =========================================================================
    // EncryptionConfig
    // =========================================================================

    #[test]
    fn test_encryption_config_new() {
        let config = EncryptionConfig::new("user_pw", "owner_pw");
        assert_eq!(config.user_password, "user_pw");
        assert_eq!(config.owner_password, "owner_pw");
        assert_eq!(config.algorithm, EncryptionAlgorithm::Aes256);
        // Permissions should be all by default
        assert!(config.permissions.print);
        assert!(config.permissions.copy);
    }

    #[test]
    fn test_encryption_config_default() {
        let config = EncryptionConfig::default();
        assert!(config.user_password.is_empty());
        assert!(config.owner_password.is_empty());
        assert_eq!(config.algorithm, EncryptionAlgorithm::Aes256);
    }

    #[test]
    fn test_encryption_config_with_algorithm() {
        let config = EncryptionConfig::new("u", "o").with_algorithm(EncryptionAlgorithm::Aes128);
        assert_eq!(config.algorithm, EncryptionAlgorithm::Aes128);
    }

    #[test]
    fn test_encryption_config_with_permissions() {
        let config = EncryptionConfig::new("u", "o").with_permissions(Permissions::read_only());
        assert!(!config.permissions.print);
        assert!(config.permissions.accessibility);
    }

    // =========================================================================
    // ModifiedPageProps
    // =========================================================================

    #[test]
    fn test_modified_page_props_default() {
        let props = ModifiedPageProps::default();
        assert!(props.rotation.is_none());
        assert!(props.media_box.is_none());
        assert!(props.crop_box.is_none());
    }

    #[test]
    fn test_modified_page_props_with_values() {
        let props = ModifiedPageProps {
            rotation: Some(90),
            media_box: Some([0.0, 0.0, 612.0, 792.0]),
            crop_box: Some([10.0, 10.0, 602.0, 782.0]),
        };
        assert_eq!(props.rotation, Some(90));
        assert_eq!(props.media_box, Some([0.0, 0.0, 612.0, 792.0]));
        assert_eq!(props.crop_box, Some([10.0, 10.0, 602.0, 782.0]));
    }

    // =========================================================================
    // apply_page_props_to_object (private, tested from within module)
    // =========================================================================

    /// Helper to create a minimal DocumentEditor for internal method testing.
    /// Returns a DocumentEditor from a minimal valid PDF.
    fn create_test_editor() -> DocumentEditor {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let pdf_bytes = minimal_pdf_bytes();
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp =
            std::env::temp_dir().join(format!("pdf_oxide_test_{}_{id}.pdf", std::process::id()));
        std::fs::write(&tmp, &pdf_bytes).unwrap();
        let editor = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        editor
    }

    /// Generates bytes for a minimal valid PDF with one page.
    fn minimal_pdf_bytes() -> Vec<u8> {
        let pdf = b"%PDF-1.4\n\
            1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
            2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
            3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
            xref\n\
            0 4\n\
            0000000000 65535 f \n\
            0000000009 00000 n \n\
            0000000058 00000 n \n\
            0000000115 00000 n \n\
            trailer\n<< /Size 4 /Root 1 0 R >>\n\
            startxref\n197\n%%EOF\n";
        pdf.to_vec()
    }

    #[test]
    fn test_apply_page_props_rotation() {
        let editor = create_test_editor();

        let mut page_dict = HashMap::new();
        page_dict.insert("Type".to_string(), Object::Name("Page".to_string()));
        let page_obj = Object::Dictionary(page_dict);

        let props = ModifiedPageProps {
            rotation: Some(90),
            media_box: None,
            crop_box: None,
        };

        let result = editor
            .apply_page_props_to_object(&page_obj, &props)
            .unwrap();
        let dict = result.as_dict().unwrap();
        assert_eq!(dict.get("Rotate").unwrap().as_integer().unwrap(), 90);
    }

    #[test]
    fn test_apply_page_props_media_box() {
        let editor = create_test_editor();

        let mut page_dict = HashMap::new();
        page_dict.insert("Type".to_string(), Object::Name("Page".to_string()));
        let page_obj = Object::Dictionary(page_dict);

        let props = ModifiedPageProps {
            rotation: None,
            media_box: Some([0.0, 0.0, 300.0, 400.0]),
            crop_box: None,
        };

        let result = editor
            .apply_page_props_to_object(&page_obj, &props)
            .unwrap();
        let dict = result.as_dict().unwrap();
        let mb = dict.get("MediaBox").unwrap().as_array().unwrap();
        assert_eq!(mb.len(), 4);
    }

    #[test]
    fn test_apply_page_props_crop_box() {
        let editor = create_test_editor();

        let mut page_dict = HashMap::new();
        page_dict.insert("Type".to_string(), Object::Name("Page".to_string()));
        let page_obj = Object::Dictionary(page_dict);

        let props = ModifiedPageProps {
            rotation: None,
            media_box: None,
            crop_box: Some([10.0, 10.0, 602.0, 782.0]),
        };

        let result = editor
            .apply_page_props_to_object(&page_obj, &props)
            .unwrap();
        let dict = result.as_dict().unwrap();
        assert!(dict.contains_key("CropBox"));
    }

    #[test]
    fn test_apply_page_props_all_at_once() {
        let editor = create_test_editor();

        let mut page_dict = HashMap::new();
        page_dict.insert("Type".to_string(), Object::Name("Page".to_string()));
        let page_obj = Object::Dictionary(page_dict);

        let props = ModifiedPageProps {
            rotation: Some(180),
            media_box: Some([0.0, 0.0, 500.0, 700.0]),
            crop_box: Some([20.0, 20.0, 480.0, 680.0]),
        };

        let result = editor
            .apply_page_props_to_object(&page_obj, &props)
            .unwrap();
        let dict = result.as_dict().unwrap();
        assert!(dict.contains_key("Rotate"));
        assert!(dict.contains_key("MediaBox"));
        assert!(dict.contains_key("CropBox"));
    }

    #[test]
    fn test_apply_page_props_non_dict_error() {
        let editor = create_test_editor();
        let page_obj = Object::Integer(42);
        let props = ModifiedPageProps::default();

        let result = editor.apply_page_props_to_object(&page_obj, &props);
        assert!(result.is_err());
    }

    // =========================================================================
    // generate_erase_overlay
    // =========================================================================

    #[test]
    fn test_generate_erase_overlay_no_regions() {
        let editor = create_test_editor();
        // No regions added for page 0
        let result = editor.generate_erase_overlay(0);
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_erase_overlay_empty_regions() {
        let mut editor = create_test_editor();
        // Insert an empty vec for page 5
        editor.erase_regions.insert(5, vec![]);
        let result = editor.generate_erase_overlay(5);
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_erase_overlay_single_region() {
        let mut editor = create_test_editor();
        editor
            .erase_regions
            .insert(0, vec![[10.0, 20.0, 100.0, 200.0]]);

        let content = editor.generate_erase_overlay(0).unwrap();
        let content_str = String::from_utf8(content).unwrap();

        assert!(content_str.contains("q\n"));
        assert!(content_str.contains("1 1 1 rg\n"));
        assert!(content_str.contains("re f\n"));
        assert!(content_str.contains("Q\n"));
    }

    #[test]
    fn test_generate_erase_overlay_multiple_regions() {
        let mut editor = create_test_editor();
        editor
            .erase_regions
            .insert(0, vec![[10.0, 20.0, 100.0, 200.0], [300.0, 400.0, 500.0, 600.0]]);

        let content = editor.generate_erase_overlay(0).unwrap();
        let content_str = String::from_utf8(content).unwrap();

        // Should contain two "re f" operations
        assert_eq!(content_str.matches("re f\n").count(), 2);
    }

    // =========================================================================
    // generate_flatten_overlay
    // =========================================================================

    #[test]
    fn test_generate_flatten_overlay_empty() {
        let editor = create_test_editor();
        let content = editor.generate_flatten_overlay(&[], &[]);
        assert!(content.is_empty());
    }

    #[test]
    fn test_generate_flatten_overlay_single_appearance() {
        let editor = create_test_editor();

        let appearance = AnnotationAppearance {
            content: b"test content".to_vec(),
            bbox: [0.0, 0.0, 100.0, 50.0],
            annot_rect: [10.0, 20.0, 210.0, 70.0],
            matrix: None,
            resources: None,
            extra_objects: Vec::new(),
        };

        let names = vec!["FlatAnnot0".to_string()];
        let content = editor.generate_flatten_overlay(&[appearance], &names);
        let content_str = String::from_utf8(content).unwrap();

        assert!(content_str.contains("q\n"));
        assert!(content_str.contains("cm\n"));
        assert!(content_str.contains("/FlatAnnot0 Do\n"));
        assert!(content_str.contains("Q\n"));
    }

    #[test]
    fn test_generate_flatten_overlay_with_matrix() {
        let editor = create_test_editor();

        let appearance = AnnotationAppearance {
            content: b"test".to_vec(),
            bbox: [0.0, 0.0, 100.0, 100.0],
            annot_rect: [0.0, 0.0, 100.0, 100.0],
            matrix: Some([1.0, 0.0, 0.0, 1.0, 5.0, 5.0]),
            resources: None,
            extra_objects: Vec::new(),
        };

        let names = vec!["Xobj0".to_string()];
        let content = editor.generate_flatten_overlay(&[appearance], &names);
        let content_str = String::from_utf8(content).unwrap();

        // Should have two cm operators: one for positioning, one for the appearance matrix
        assert_eq!(content_str.matches("cm\n").count(), 2);
    }

    #[test]
    fn test_generate_flatten_overlay_zero_size_bbox() {
        let editor = create_test_editor();

        let appearance = AnnotationAppearance {
            content: b"empty".to_vec(),
            bbox: [0.0, 0.0, 0.0, 0.0], // zero-size
            annot_rect: [10.0, 20.0, 110.0, 120.0],
            matrix: None,
            resources: None,
            extra_objects: Vec::new(),
        };

        let names = vec!["Xobj0".to_string()];
        let content = editor.generate_flatten_overlay(&[appearance], &names);
        let content_str = String::from_utf8(content).unwrap();

        // Should still generate content even with zero bbox (uses 1.0 fallback for scale)
        assert!(content_str.contains("/Xobj0 Do\n"));
    }

    // =========================================================================
    // generate_redaction_overlay
    // =========================================================================

    #[test]
    fn test_generate_redaction_overlay_single() {
        let editor = create_test_editor();

        let redactions = vec![RedactionData {
            rect: [50.0, 100.0, 200.0, 150.0],
            color: [0.0, 0.0, 0.0],
        }];

        let content = editor.generate_redaction_overlay(&redactions);
        let content_str = String::from_utf8(content).unwrap();

        assert!(content_str.contains("q\n"));
        assert!(content_str.contains("0.000 0.000 0.000 rg\n"));
        assert!(content_str.contains("re f\n"));
        assert!(content_str.contains("Q\n"));
    }

    #[test]
    fn test_generate_redaction_overlay_custom_color() {
        let editor = create_test_editor();

        let redactions = vec![RedactionData {
            rect: [0.0, 0.0, 100.0, 100.0],
            color: [1.0, 0.0, 0.0], // Red
        }];

        let content = editor.generate_redaction_overlay(&redactions);
        let content_str = String::from_utf8(content).unwrap();
        assert!(content_str.contains("1.000 0.000 0.000 rg\n"));
    }

    #[test]
    fn test_generate_redaction_overlay_multiple() {
        let editor = create_test_editor();

        let redactions = vec![
            RedactionData {
                rect: [0.0, 0.0, 100.0, 50.0],
                color: [0.0, 0.0, 0.0],
            },
            RedactionData {
                rect: [200.0, 200.0, 400.0, 300.0],
                color: [0.5, 0.5, 0.5],
            },
        ];

        let content = editor.generate_redaction_overlay(&redactions);
        let content_str = String::from_utf8(content).unwrap();

        // Should contain two q/Q pairs
        assert_eq!(content_str.matches("q\n").count(), 2);
        assert_eq!(content_str.matches("Q\n").count(), 2);
    }

    #[test]
    fn test_generate_redaction_overlay_empty() {
        let editor = create_test_editor();
        let content = editor.generate_redaction_overlay(&[]);
        assert!(content.is_empty());
    }

    // =========================================================================
    // serialize_operator
    // =========================================================================

    #[test]
    fn test_serialize_operator_save_restore_state() {
        let editor = create_test_editor();
        let mut output = Vec::new();

        editor.serialize_operator(&mut output, &crate::content::operators::Operator::SaveState);
        assert_eq!(&output, b"q\n");

        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::RestoreState);
        assert_eq!(&output, b"Q\n");
    }

    #[test]
    fn test_serialize_operator_cm() {
        let editor = create_test_editor();
        let mut output = Vec::new();

        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Cm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 10.0,
                f: 20.0,
            },
        );

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("1.0"));
        assert!(s.contains("10.0"));
        assert!(s.contains("20.0"));
        assert!(s.ends_with("cm\n"));
    }

    #[test]
    fn test_serialize_operator_text_operations() {
        let editor = create_test_editor();

        // BT
        let mut output = Vec::new();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::BeginText);
        assert_eq!(&output, b"BT\n");

        // ET
        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::EndText);
        assert_eq!(&output, b"ET\n");

        // Tf
        output.clear();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Tf {
                font: "Helv".to_string(),
                size: 12.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("/Helv"));
        assert!(s.contains("Tf\n"));
    }

    #[test]
    fn test_serialize_operator_td() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Td { tx: 5.0, ty: 10.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("Td\n"));
    }

    #[test]
    fn test_serialize_operator_tj() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Tj {
                text: b"Hello World".to_vec(),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Hello World"));
        assert!(s.ends_with("Tj\n"));
    }

    #[test]
    fn test_serialize_operator_tj_with_escapes() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Tj {
                text: b"Hello (World) \\ test".to_vec(),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("\\("));
        assert!(s.contains("\\)"));
        assert!(s.contains("\\\\"));
    }

    #[test]
    fn test_serialize_operator_tj_array() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::TJ {
                array: vec![
                    crate::content::operators::TextElement::String(b"AB".to_vec()),
                    crate::content::operators::TextElement::Offset(-120.0),
                    crate::content::operators::TextElement::String(b"CD".to_vec()),
                ],
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.starts_with("["));
        assert!(s.contains("(AB)"));
        assert!(s.contains("(CD)"));
        assert!(s.ends_with("TJ\n"));
    }

    #[test]
    fn test_serialize_operator_do() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Do {
                name: "Im1".to_string(),
            },
        );
        assert_eq!(&output, b"/Im1 Do\n");
    }

    #[test]
    fn test_serialize_operator_color_ops() {
        let editor = create_test_editor();

        // SetFillRgb
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillRgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("rg\n"));

        // SetStrokeGray
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeGray { gray: 0.5 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("G\n"));

        // SetFillCmyk
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillCmyk {
                c: 0.1,
                m: 0.2,
                y: 0.3,
                k: 0.4,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("k\n"));
    }

    #[test]
    fn test_serialize_operator_path_ops() {
        let editor = create_test_editor();

        // Stroke
        let mut output = Vec::new();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::Stroke);
        assert_eq!(&output, b"S\n");

        // Fill
        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::Fill);
        assert_eq!(&output, b"f\n");

        // FillEvenOdd
        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::FillEvenOdd);
        assert_eq!(&output, b"f*\n");

        // EndPath
        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::EndPath);
        assert_eq!(&output, b"n\n");

        // ClosePath
        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::ClosePath);
        assert_eq!(&output, b"h\n");

        // CloseFillStroke
        output.clear();
        editor
            .serialize_operator(&mut output, &crate::content::operators::Operator::CloseFillStroke);
        assert_eq!(&output, b"b\n");
    }

    #[test]
    fn test_serialize_operator_clipping() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::ClipNonZero);
        assert_eq!(&output, b"W\n");

        output.clear();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::ClipEvenOdd);
        assert_eq!(&output, b"W*\n");
    }

    #[test]
    fn test_serialize_operator_rectangle() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Rectangle {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("re\n"));
    }

    #[test]
    fn test_serialize_operator_marked_content() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::BeginMarkedContent {
                tag: "P".to_string(),
            },
        );
        assert_eq!(&output, b"/P BMC\n");

        output.clear();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::EndMarkedContent,
        );
        assert_eq!(&output, b"EMC\n");
    }

    #[test]
    fn test_serialize_operator_set_line_width() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetLineWidth { width: 2.5 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("w\n"));
    }

    #[test]
    fn test_serialize_operator_set_dash() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetDash {
                array: vec![3.0, 2.0],
                phase: 0.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.starts_with("["));
        assert!(s.ends_with("d\n"));
    }

    #[test]
    fn test_serialize_operator_tstar() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(&mut output, &crate::content::operators::Operator::TStar);
        assert_eq!(&output, b"T*\n");
    }

    #[test]
    fn test_serialize_operator_quote() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Quote {
                text: b"Hello".to_vec(),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Hello"));
        assert!(s.ends_with("'\n"));
    }

    #[test]
    fn test_serialize_operator_double_quote() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::DoubleQuote {
                word_space: 1.0,
                char_space: 0.5,
                text: b"Hi".to_vec(),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Hi"));
        assert!(s.ends_with("\"\n"));
    }

    #[test]
    fn test_serialize_operator_ext_gstate() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetExtGState {
                dict_name: "GS1".to_string(),
            },
        );
        assert_eq!(&output, b"/GS1 gs\n");
    }

    #[test]
    fn test_serialize_operator_paint_shading() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::PaintShading {
                name: "Sh1".to_string(),
            },
        );
        assert_eq!(&output, b"/Sh1 sh\n");
    }

    #[test]
    fn test_serialize_operator_set_rendering_intent() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetRenderingIntent {
                intent: "RelativeColorimetric".to_string(),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("/RelativeColorimetric"));
        assert!(s.ends_with("ri\n"));
    }

    #[test]
    fn test_serialize_operator_color_space() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillColorSpace {
                name: "DeviceRGB".to_string(),
            },
        );
        assert_eq!(&output, b"/DeviceRGB cs\n");

        output.clear();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeColorSpace {
                name: "DeviceCMYK".to_string(),
            },
        );
        assert_eq!(&output, b"/DeviceCMYK CS\n");
    }

    #[test]
    fn test_serialize_operator_set_fill_stroke_color() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillColor {
                components: vec![0.5, 0.6, 0.7],
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("sc\n"));

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeColor {
                components: vec![0.1, 0.2],
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("SC\n"));
    }

    #[test]
    fn test_serialize_operator_color_n_with_pattern() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillColorN {
                components: vec![0.5],
                name: Some(Box::new("P1".to_string())),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("/P1"));
        assert!(s.ends_with("scn\n"));

        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeColorN {
                components: vec![],
                name: None,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("SCN\n"));
    }

    #[test]
    fn test_serialize_operator_other() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Other {
                name: "DP".to_string(),
                operands: Box::new(vec![Object::Name("MC0".to_string())]),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("/MC0"));
        assert!(s.contains("DP"));
    }

    #[test]
    fn test_serialize_operator_inline_image() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("W".to_string(), Object::Integer(10));
        dict.insert("H".to_string(), Object::Integer(10));
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::InlineImage {
                dict: Box::new(dict),
                data: vec![0xFF; 10],
            },
        );
        let s = String::from_utf8_lossy(&output);
        assert!(s.contains("BI\n"));
        assert!(s.contains("ID "));
        assert!(s.contains("EI\n"));
    }

    #[test]
    fn test_serialize_operator_curves() {
        let editor = create_test_editor();

        // CurveTo
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::CurveTo {
                x1: 1.0,
                y1: 2.0,
                x2: 3.0,
                y2: 4.0,
                x3: 5.0,
                y3: 6.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("c\n"));

        // CurveToV
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::CurveToV {
                x2: 1.0,
                y2: 2.0,
                x3: 3.0,
                y3: 4.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("v\n"));

        // CurveToY
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::CurveToY {
                x1: 1.0,
                y1: 2.0,
                x3: 3.0,
                y3: 4.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("y\n"));
    }

    #[test]
    fn test_serialize_operator_text_state() {
        let editor = create_test_editor();

        let ops: Vec<(crate::content::operators::Operator, &str)> = vec![
            (crate::content::operators::Operator::Tc { char_space: 1.0 }, "Tc\n"),
            (crate::content::operators::Operator::Tw { word_space: 2.0 }, "Tw\n"),
            (crate::content::operators::Operator::Tz { scale: 100.0 }, "Tz\n"),
            (crate::content::operators::Operator::TL { leading: 14.0 }, "TL\n"),
            (crate::content::operators::Operator::Tr { render: 0 }, "Tr\n"),
            (crate::content::operators::Operator::Ts { rise: 3.0 }, "Ts\n"),
        ];

        for (op, suffix) in ops {
            let mut output = Vec::new();
            editor.serialize_operator(&mut output, &op);
            let s = String::from_utf8(output).unwrap();
            assert!(s.ends_with(suffix), "Expected suffix '{}', got '{}'", suffix, s);
        }
    }

    // =========================================================================
    // serialize_object
    // =========================================================================

    #[test]
    fn test_serialize_object_null() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Null);
        assert_eq!(&output, b"null");
    }

    #[test]
    fn test_serialize_object_boolean() {
        let editor = create_test_editor();

        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Boolean(true));
        assert_eq!(&output, b"true");

        output.clear();
        editor.serialize_object(&mut output, &Object::Boolean(false));
        assert_eq!(&output, b"false");
    }

    #[test]
    fn test_serialize_object_integer() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Integer(42));
        assert_eq!(&output, b"42");
    }

    #[test]
    fn test_serialize_object_real() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Real(std::f64::consts::PI));
        let s = String::from_utf8(output).unwrap();
        assert!(s.starts_with("3.14"));
    }

    #[test]
    fn test_serialize_object_name() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Name("Type".to_string()));
        assert_eq!(&output, b"/Type");
    }

    #[test]
    fn test_serialize_object_string() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::String(b"hello".to_vec()));
        assert_eq!(&output, b"(hello)");
    }

    #[test]
    fn test_serialize_object_string_with_escapes() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::String(b"a(b)c\\d".to_vec()));
        assert_eq!(&output, b"(a\\(b\\)c\\\\d)");
    }

    #[test]
    fn test_serialize_object_array() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(
            &mut output,
            &Object::Array(vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)]),
        );
        assert_eq!(&output, b"[1 2 3]");
    }

    #[test]
    fn test_serialize_object_empty_array() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Array(vec![]));
        assert_eq!(&output, b"[]");
    }

    #[test]
    fn test_serialize_object_dictionary() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        let mut dict = HashMap::new();
        dict.insert("Key".to_string(), Object::Integer(42));
        editor.serialize_object(&mut output, &Object::Dictionary(dict));
        let s = String::from_utf8(output).unwrap();
        assert!(s.starts_with("<<"));
        assert!(s.ends_with(">>"));
        assert!(s.contains("/Key"));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_serialize_object_reference() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(&mut output, &Object::Reference(ObjectRef::new(5, 0)));
        assert_eq!(&output, b"5 0 R");
    }

    #[test]
    fn test_serialize_object_stream_placeholder() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_object(
            &mut output,
            &Object::Stream {
                dict: HashMap::new(),
                data: crate::import_limits::StreamData::from_static(b"data"),
            },
        );
        assert_eq!(&output, b"(stream)");
    }

    // =========================================================================
    // build_image_xobject
    // =========================================================================

    #[test]
    fn test_build_image_xobject_jpeg() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent::new(
            crate::geometry::Rect::new(0.0, 0.0, 100.0, 50.0),
            ImageFormat::Jpeg,
            vec![0xFF, 0xD8, 0xFF, 0xE0], // Fake JPEG header
            100,
            50,
        );

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, data } = obj {
            assert_eq!(dict.get("Type").unwrap(), &Object::Name("XObject".to_string()));
            assert_eq!(dict.get("Subtype").unwrap(), &Object::Name("Image".to_string()));
            assert_eq!(dict.get("Width").unwrap().as_integer().unwrap(), 100);
            assert_eq!(dict.get("Height").unwrap().as_integer().unwrap(), 50);
            assert_eq!(dict.get("Filter").unwrap(), &Object::Name("DCTDecode".to_string()));
            assert_eq!(dict.get("ColorSpace").unwrap(), &Object::Name("DeviceRGB".to_string()));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_png() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 200.0, 100.0),
            format: ImageFormat::Png,
            data: vec![0x89, 0x50, 0x4E, 0x47], // Fake PNG header
            width: 200,
            height: 100,
            bits_per_component: 8,
            color_space: crate::elements::ColorSpace::Gray,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            assert_eq!(dict.get("Filter").unwrap(), &Object::Name("FlateDecode".to_string()));
            assert_eq!(dict.get("ColorSpace").unwrap(), &Object::Name("DeviceGray".to_string()));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_jpeg2000() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 10.0, 10.0),
            format: ImageFormat::Jpeg2000,
            data: vec![0; 8],
            width: 10,
            height: 10,
            bits_per_component: 8,
            color_space: crate::elements::ColorSpace::CMYK,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            assert_eq!(dict.get("Filter").unwrap(), &Object::Name("JPXDecode".to_string()));
            assert_eq!(dict.get("ColorSpace").unwrap(), &Object::Name("DeviceCMYK".to_string()));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_jbig2() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 10.0, 10.0),
            format: ImageFormat::Jbig2,
            data: vec![0; 4],
            width: 10,
            height: 10,
            bits_per_component: 1,
            color_space: crate::elements::ColorSpace::Gray,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            assert_eq!(dict.get("Filter").unwrap(), &Object::Name("JBIG2Decode".to_string()));
            assert_eq!(dict.get("BitsPerComponent").unwrap().as_integer().unwrap(), 1);
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_unknown_format() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 10.0, 10.0),
            format: ImageFormat::Unknown,
            data: vec![0; 4],
            width: 10,
            height: 10,
            bits_per_component: 8,
            color_space: crate::elements::ColorSpace::RGB,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            // Unknown format should not have a Filter
            assert!(!dict.contains_key("Filter"));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_indexed_colorspace() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 10.0, 10.0),
            format: ImageFormat::Raw,
            data: vec![0; 10],
            width: 10,
            height: 10,
            bits_per_component: 8,
            color_space: crate::elements::ColorSpace::Indexed,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            assert_eq!(dict.get("ColorSpace").unwrap(), &Object::Name("Indexed".to_string()));
            assert_eq!(dict.get("Filter").unwrap(), &Object::Name("FlateDecode".to_string()));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_lab_colorspace() {
        use crate::elements::{ImageContent, ImageFormat};

        let image = ImageContent {
            bbox: crate::geometry::Rect::new(0.0, 0.0, 10.0, 10.0),
            format: ImageFormat::Raw,
            data: vec![0; 10],
            width: 10,
            height: 10,
            bits_per_component: 8,
            color_space: crate::elements::ColorSpace::Lab,
            reading_order: None,
            alt_text: None,
            horizontal_dpi: None,
            vertical_dpi: None,
            soft_mask: None,
            matrix: None,
            is_artifact: false,
        };

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream { dict, .. } = obj {
            assert_eq!(dict.get("ColorSpace").unwrap(), &Object::Name("Lab".to_string()));
        } else {
            panic!("Expected Stream object");
        }
    }

    #[test]
    fn test_build_image_xobject_length() {
        use crate::elements::{ImageContent, ImageFormat};

        let data = vec![1, 2, 3, 4, 5];
        let image = ImageContent::new(
            crate::geometry::Rect::new(0.0, 0.0, 5.0, 1.0),
            ImageFormat::Raw,
            data.clone(),
            5,
            1,
        );

        let obj = DocumentEditor::build_image_xobject(&image);
        if let Object::Stream {
            dict,
            data: stream_data,
        } = obj
        {
            assert_eq!(dict.get("Length").unwrap().as_integer().unwrap(), 5);
            assert_eq!(stream_data.as_ref(), &data[..]);
        } else {
            panic!("Expected Stream object");
        }
    }

    // =========================================================================
    // parse_media_box
    // =========================================================================

    #[test]
    fn test_parse_media_box_real_values() {
        let editor = create_test_editor();
        let media_box = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(595.28),
            Object::Real(841.89),
        ]);

        let (w, h) = editor.parse_media_box(&media_box).unwrap();
        assert!((w - 595.28).abs() < 0.01);
        assert!((h - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_parse_media_box_integer_values() {
        let editor = create_test_editor();
        let media_box = Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(612),
            Object::Integer(792),
        ]);

        let (w, h) = editor.parse_media_box(&media_box).unwrap();
        assert_eq!(w, 612.0);
        assert_eq!(h, 792.0);
    }

    #[test]
    fn test_parse_media_box_non_zero_origin() {
        let editor = create_test_editor();
        let media_box = Object::Array(vec![
            Object::Real(50.0),
            Object::Real(50.0),
            Object::Real(562.0),
            Object::Real(742.0),
        ]);

        let (w, h) = editor.parse_media_box(&media_box).unwrap();
        assert!((w - 512.0).abs() < 0.01);
        assert!((h - 692.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_media_box_non_array() {
        let editor = create_test_editor();
        let media_box = Object::Integer(42);

        let (w, h) = editor.parse_media_box(&media_box).unwrap();
        // Should return default Letter size
        assert_eq!(w, 612.0);
        assert_eq!(h, 792.0);
    }

    #[test]
    fn test_parse_media_box_short_array() {
        let editor = create_test_editor();
        let media_box = Object::Array(vec![Object::Integer(0), Object::Integer(0)]);

        let (w, h) = editor.parse_media_box(&media_box).unwrap();
        // Should return default Letter size
        assert_eq!(w, 612.0);
        assert_eq!(h, 792.0);
    }

    // =========================================================================
    // find_prev_xref_offset
    // =========================================================================

    #[test]
    fn test_find_prev_xref_offset_valid() {
        let editor = create_test_editor();

        // The function searches backwards from pos = len - 100 down to 1.
        // At each pos it checks if bytes[pos..] starts with "startxref".
        // So "startxref" must be at a position p where 1 <= p <= len - 100.
        // We put startxref at position 10, then pad after it to make total > 110.
        let mut pdf_data = Vec::new();
        pdf_data.extend_from_slice(b"%PDF-1.4\n"); // 9 bytes
        pdf_data.extend_from_slice(b"startxref\n12345\n%%EOF\n"); // startxref at byte 9
                                                                  // Pad to ensure len - 100 >= 9 (i.e., len >= 109)
        while pdf_data.len() < 120 {
            pdf_data.push(b'\n');
        }
        let result = editor.find_prev_xref_offset(&pdf_data);
        assert_eq!(result.unwrap(), 12345);
    }

    #[test]
    fn test_find_prev_xref_offset_with_whitespace() {
        let editor = create_test_editor();

        let mut pdf_data = Vec::new();
        pdf_data.extend_from_slice(b"%PDF-1.4\n"); // 9 bytes
        pdf_data.extend_from_slice(b"startxref\n  \r\n67890\n%%EOF\n"); // startxref at byte 9
        while pdf_data.len() < 120 {
            pdf_data.push(b'\n');
        }
        let result = editor.find_prev_xref_offset(&pdf_data);
        assert_eq!(result.unwrap(), 67890);
    }

    #[test]
    fn test_find_prev_xref_offset_not_found() {
        let editor = create_test_editor();

        // No "startxref" anywhere, and data is long enough
        let mut pdf_data = vec![b'X'; 200];
        pdf_data.extend_from_slice(b"\n%%EOF\n");
        let result = editor.find_prev_xref_offset(&pdf_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_prev_xref_offset_short_data() {
        let editor = create_test_editor();

        // Data shorter than 100 bytes -- pos starts at 0, loop doesn't execute
        let pdf_data = b"startxref\n999\n%%EOF\n";
        let result = editor.find_prev_xref_offset(pdf_data);
        // Short data means pos starts at 0 and while pos > 0 is false, so it should err
        assert!(result.is_err());
    }

    // =========================================================================
    // allocate_object_id
    // =========================================================================

    #[test]
    fn test_allocate_object_id_sequential() {
        let mut editor = create_test_editor();
        let first = editor.allocate_object_id();
        let second = editor.allocate_object_id();
        let third = editor.allocate_object_id();

        assert_eq!(second, first + 1);
        assert_eq!(third, first + 2);
    }

    // =========================================================================
    // current_page_count and page manipulation
    // =========================================================================

    #[test]
    fn test_current_page_count_initial() {
        let editor = create_test_editor();
        assert_eq!(editor.current_page_count(), 1);
    }

    #[test]
    fn test_current_page_count_after_removal() {
        let mut editor = create_test_editor();
        // Mark page 0 as removed
        editor.page_order[0] = -1;
        assert_eq!(editor.current_page_count(), 0);
    }

    #[test]
    fn test_current_page_count_mixed() {
        let mut editor = create_test_editor();
        // Simulate 3 pages: [0, 1, 2], then remove page at index 1
        editor.page_order = vec![0, -1, 2];
        assert_eq!(editor.current_page_count(), 2);
    }

    // =========================================================================
    // is_modified
    // =========================================================================

    #[test]
    fn test_is_modified_initial() {
        let editor = create_test_editor();
        assert!(!editor.is_modified());
    }

    // =========================================================================
    // source_path
    // =========================================================================

    #[test]
    fn test_source_path() {
        let pdf_bytes = minimal_pdf_bytes();
        let tmp = std::env::temp_dir().join("pdf_oxide_test_path.pdf");
        std::fs::write(&tmp, &pdf_bytes).unwrap();
        let editor = DocumentEditor::open(&tmp).unwrap();
        assert!(editor.source_path().contains("pdf_oxide_test_path.pdf"));
        let _ = std::fs::remove_file(&tmp);
    }

    // =========================================================================
    // version
    // =========================================================================

    #[test]
    fn test_version() {
        let editor = create_test_editor();
        let (major, minor) = editor.version();
        assert_eq!(major, 1);
        assert_eq!(minor, 4);
    }

    // =========================================================================
    // build_info_object
    // =========================================================================

    #[test]
    fn test_build_info_object_none() {
        let editor = create_test_editor();
        assert!(editor.build_info_object().is_none());
    }

    #[test]
    fn test_build_info_object_some() {
        let mut editor = create_test_editor();
        editor.modified_info = Some(DocumentInfo::new().title("Test"));
        let obj = editor.build_info_object();
        assert!(obj.is_some());
        let dict = obj.unwrap().as_dict().unwrap().clone();
        assert!(dict.contains_key("Title"));
    }

    // =========================================================================
    // RedactionData
    // =========================================================================

    #[test]
    fn test_redaction_data_construction() {
        let rd = RedactionData {
            rect: [10.0, 20.0, 100.0, 200.0],
            color: [1.0, 0.0, 0.0],
        };
        assert_eq!(rd.rect[0], 10.0);
        assert_eq!(rd.color[0], 1.0);
    }

    // =========================================================================
    // AnnotationAppearance
    // =========================================================================

    #[test]
    fn test_annotation_appearance_construction() {
        let ap = AnnotationAppearance {
            content: b"q 1 0 0 1 0 0 cm Q".to_vec(),
            bbox: [0.0, 0.0, 100.0, 50.0],
            annot_rect: [10.0, 20.0, 110.0, 70.0],
            matrix: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            resources: None,
            extra_objects: Vec::new(),
        };
        assert_eq!(ap.bbox[2], 100.0);
        assert_eq!(ap.annot_rect[0], 10.0);
        assert!(ap.matrix.is_some());
        assert!(ap.resources.is_none());
    }

    #[test]
    fn test_annotation_appearance_no_matrix_no_resources() {
        let ap = AnnotationAppearance {
            content: vec![],
            bbox: [0.0, 0.0, 0.0, 0.0],
            annot_rect: [0.0, 0.0, 0.0, 0.0],
            matrix: None,
            resources: None,
            extra_objects: Vec::new(),
        };
        assert!(ap.matrix.is_none());
        assert!(ap.content.is_empty());
    }

    // =========================================================================
    // ImageInfo
    // =========================================================================

    #[test]
    fn test_image_info_construction() {
        let info = ImageInfo {
            name: "Im1".to_string(),
            bounds: [10.0, 20.0, 100.0, 50.0],
            matrix: [100.0, 0.0, 0.0, 50.0, 10.0, 20.0],
        };
        assert_eq!(info.name, "Im1");
        assert_eq!(info.bounds[2], 100.0);
    }

    // =========================================================================
    // ImageModification
    // =========================================================================

    #[test]
    fn test_image_modification_construction() {
        let m = ImageModification {
            x: Some(10.0),
            y: None,
            width: Some(200.0),
            height: None,
        };
        assert_eq!(m.x, Some(10.0));
        assert_eq!(m.y, None);
        assert_eq!(m.width, Some(200.0));
        assert_eq!(m.height, None);
    }

    // =========================================================================
    // PageInfo
    // =========================================================================

    #[test]
    fn test_page_info_construction() {
        let info = PageInfo {
            index: 0,
            width: 612.0,
            height: 792.0,
            rotation: 90,
            object_ref: ObjectRef::new(5, 0),
        };
        assert_eq!(info.index, 0);
        assert_eq!(info.width, 612.0);
        assert_eq!(info.rotation, 90);
        assert_eq!(info.object_ref.id, 5);
    }

    // =========================================================================
    // Metadata setters (set_title, set_author, etc.)
    // =========================================================================

    #[test]
    fn test_set_title_marks_modified() {
        let mut editor = create_test_editor();
        assert!(!editor.is_modified());
        editor.set_title("New Title");
        assert!(editor.is_modified());
        assert_eq!(editor.modified_info.as_ref().unwrap().title, Some("New Title".to_string()));
    }

    #[test]
    fn test_set_author_marks_modified() {
        let mut editor = create_test_editor();
        editor.set_author("New Author");
        assert!(editor.is_modified());
        assert_eq!(editor.modified_info.as_ref().unwrap().author, Some("New Author".to_string()));
    }

    #[test]
    fn test_set_subject_marks_modified() {
        let mut editor = create_test_editor();
        editor.set_subject("New Subject");
        assert!(editor.is_modified());
        assert_eq!(editor.modified_info.as_ref().unwrap().subject, Some("New Subject".to_string()));
    }

    #[test]
    fn test_set_keywords_marks_modified() {
        let mut editor = create_test_editor();
        editor.set_keywords("kw1, kw2");
        assert!(editor.is_modified());
        assert_eq!(editor.modified_info.as_ref().unwrap().keywords, Some("kw1, kw2".to_string()));
    }

    #[test]
    fn test_set_multiple_metadata_fields() {
        let mut editor = create_test_editor();
        editor.set_title("T1");
        editor.set_author("A1");
        editor.set_subject("S1");
        editor.set_keywords("K1");

        let info = editor.modified_info.as_ref().unwrap();
        assert_eq!(info.title, Some("T1".to_string()));
        assert_eq!(info.author, Some("A1".to_string()));
        assert_eq!(info.subject, Some("S1".to_string()));
        assert_eq!(info.keywords, Some("K1".to_string()));
    }

    // =========================================================================
    // Erase region API methods
    // =========================================================================

    #[test]
    fn test_erase_region_valid() {
        let mut editor = create_test_editor();
        let result = editor.erase_region(0, [10.0, 20.0, 100.0, 200.0]);
        assert!(result.is_ok());
        assert!(editor.is_modified());
        assert_eq!(editor.erase_regions.get(&0).unwrap().len(), 1);
    }

    #[test]
    fn test_erase_region_out_of_range() {
        let mut editor = create_test_editor();
        let result = editor.erase_region(99, [10.0, 20.0, 100.0, 200.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_erase_regions_multiple() {
        let mut editor = create_test_editor();
        let rects = &[[10.0, 20.0, 100.0, 200.0], [200.0, 300.0, 400.0, 500.0]];
        let result = editor.erase_regions(0, rects);
        assert!(result.is_ok());
        assert_eq!(editor.erase_regions.get(&0).unwrap().len(), 2);
    }

    #[test]
    fn test_erase_regions_out_of_range() {
        let mut editor = create_test_editor();
        let result = editor.erase_regions(99, &[[10.0, 20.0, 100.0, 200.0]]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_erase_regions() {
        let mut editor = create_test_editor();
        editor.erase_region(0, [10.0, 20.0, 100.0, 200.0]).unwrap();
        assert!(editor.erase_regions.contains_key(&0));
        editor.clear_erase_regions(0);
        assert!(!editor.erase_regions.contains_key(&0));
    }

    // =========================================================================
    // Annotation flatten methods
    // =========================================================================

    #[test]
    fn test_flatten_page_annotations_valid() {
        let mut editor = create_test_editor();
        assert!(!editor.is_page_marked_for_flatten(0));
        editor.flatten_page_annotations(0).unwrap();
        assert!(editor.is_page_marked_for_flatten(0));
        assert!(editor.is_modified());
    }

    #[test]
    fn test_flatten_page_annotations_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.flatten_page_annotations(99).is_err());
    }

    #[test]
    fn test_flatten_all_annotations() {
        let mut editor = create_test_editor();
        editor.flatten_all_annotations().unwrap();
        assert!(editor.is_page_marked_for_flatten(0));
    }

    #[test]
    fn test_unmark_page_for_flatten() {
        let mut editor = create_test_editor();
        editor.flatten_page_annotations(0).unwrap();
        assert!(editor.is_page_marked_for_flatten(0));
        editor.unmark_page_for_flatten(0);
        assert!(!editor.is_page_marked_for_flatten(0));
    }

    // =========================================================================
    // Form flatten methods
    // =========================================================================

    #[test]
    fn test_flatten_forms_on_page_valid() {
        let mut editor = create_test_editor();
        assert!(!editor.is_page_marked_for_form_flatten(0));
        editor.flatten_forms_on_page(0).unwrap();
        assert!(editor.is_page_marked_for_form_flatten(0));
    }

    #[test]
    fn test_flatten_forms_on_page_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.flatten_forms_on_page(99).is_err());
    }

    #[test]
    fn test_flatten_forms() {
        let mut editor = create_test_editor();
        assert!(!editor.will_remove_acroform());
        editor.flatten_forms().unwrap();
        assert!(editor.is_page_marked_for_form_flatten(0));
        assert!(editor.will_remove_acroform());
    }

    // =========================================================================
    // Redaction methods
    // =========================================================================

    #[test]
    fn test_apply_page_redactions_valid() {
        let mut editor = create_test_editor();
        assert!(!editor.is_page_marked_for_redaction(0));
        editor.apply_page_redactions(0).unwrap();
        assert!(editor.is_page_marked_for_redaction(0));
        assert!(editor.is_modified());
    }

    #[test]
    fn test_apply_page_redactions_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.apply_page_redactions(99).is_err());
    }

    #[test]
    fn test_apply_all_redactions() {
        let mut editor = create_test_editor();
        editor.apply_all_redactions().unwrap();
        assert!(editor.is_page_marked_for_redaction(0));
    }

    #[test]
    fn test_unmark_page_for_redaction() {
        let mut editor = create_test_editor();
        editor.apply_page_redactions(0).unwrap();
        editor.unmark_page_for_redaction(0);
        assert!(!editor.is_page_marked_for_redaction(0));
    }

    // =========================================================================
    // Image modification methods
    // =========================================================================

    #[test]
    fn test_reposition_image_valid() {
        let mut editor = create_test_editor();
        let result = editor.reposition_image(0, "Im1", 100.0, 200.0);
        assert!(result.is_ok());
        assert!(editor.is_modified());
        assert!(editor.has_image_modifications(0));
    }

    #[test]
    fn test_reposition_image_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.reposition_image(99, "Im1", 100.0, 200.0).is_err());
    }

    #[test]
    fn test_resize_image_valid() {
        let mut editor = create_test_editor();
        let result = editor.resize_image(0, "Im1", 300.0, 200.0);
        assert!(result.is_ok());
        assert!(editor.has_image_modifications(0));
    }

    #[test]
    fn test_resize_image_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.resize_image(99, "Im1", 300.0, 200.0).is_err());
    }

    #[test]
    fn test_set_image_bounds_valid() {
        let mut editor = create_test_editor();
        let result = editor.set_image_bounds(0, "Im1", 10.0, 20.0, 300.0, 200.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_image_bounds_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor
            .set_image_bounds(99, "Im1", 10.0, 20.0, 300.0, 200.0)
            .is_err());
    }

    #[test]
    fn test_clear_image_modifications() {
        let mut editor = create_test_editor();
        editor.reposition_image(0, "Im1", 10.0, 20.0).unwrap();
        assert!(editor.has_image_modifications(0));
        editor.clear_image_modifications(0);
        assert!(!editor.has_image_modifications(0));
    }

    #[test]
    fn test_has_image_modifications_empty() {
        let editor = create_test_editor();
        assert!(!editor.has_image_modifications(0));
        assert!(!editor.has_image_modifications(99));
    }

    // =========================================================================
    // Embedded files
    // =========================================================================

    #[test]
    fn test_embed_file() {
        let mut editor = create_test_editor();
        let result = editor.embed_file("test.txt", b"Hello World".to_vec());
        assert!(result.is_ok());
        assert!(editor.is_modified());
        assert_eq!(editor.pending_embedded_files().len(), 1);
    }

    #[test]
    fn test_embed_file_multiple() {
        let mut editor = create_test_editor();
        editor.embed_file("a.txt", b"AAA".to_vec()).unwrap();
        editor.embed_file("b.txt", b"BBB".to_vec()).unwrap();
        assert_eq!(editor.pending_embedded_files().len(), 2);
    }

    #[test]
    fn test_clear_embedded_files() {
        let mut editor = create_test_editor();
        editor.embed_file("a.txt", b"AAA".to_vec()).unwrap();
        assert_eq!(editor.pending_embedded_files().len(), 1);
        editor.clear_embedded_files();
        assert!(editor.pending_embedded_files().is_empty());
    }

    // =========================================================================
    // set_page_rotation and rotate_page_by
    // =========================================================================

    #[test]
    fn test_set_page_rotation_valid_values() {
        let mut editor = create_test_editor();
        for &deg in &[0, 90, 180, 270] {
            assert!(editor.set_page_rotation(0, deg).is_ok());
        }
    }

    #[test]
    fn test_set_page_rotation_invalid_values() {
        let mut editor = create_test_editor();
        assert!(editor.set_page_rotation(0, 45).is_err());
        assert!(editor.set_page_rotation(0, 360).is_err());
        assert!(editor.set_page_rotation(0, -90).is_err());
    }

    #[test]
    fn test_set_page_rotation_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.set_page_rotation(99, 90).is_err());
    }

    #[test]
    fn test_set_page_media_box_valid() {
        let mut editor = create_test_editor();
        let result = editor.set_page_media_box(0, [0.0, 0.0, 500.0, 700.0]);
        assert!(result.is_ok());
        assert!(editor.is_modified());
    }

    #[test]
    fn test_set_page_media_box_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor
            .set_page_media_box(99, [0.0, 0.0, 500.0, 700.0])
            .is_err());
    }

    #[test]
    fn test_set_page_crop_box_valid() {
        let mut editor = create_test_editor();
        let result = editor.set_page_crop_box(0, [10.0, 10.0, 602.0, 782.0]);
        assert!(result.is_ok());
        assert!(editor.is_modified());
    }

    #[test]
    fn test_set_page_crop_box_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor
            .set_page_crop_box(99, [10.0, 10.0, 602.0, 782.0])
            .is_err());
    }

    // =========================================================================
    // EditableDocument trait: page_count, set_info, get_info
    // =========================================================================

    #[test]
    fn test_editable_page_count() {
        let mut editor = create_test_editor();
        assert_eq!(EditableDocument::page_count(&mut editor).unwrap(), 1);
    }

    #[test]
    fn test_editable_set_info() {
        let mut editor = create_test_editor();
        let info = DocumentInfo::new().title("SetInfo Test");
        EditableDocument::set_info(&mut editor, info).unwrap();
        assert!(editor.is_modified());

        let retrieved = EditableDocument::get_info(&mut editor).unwrap();
        assert_eq!(retrieved.title, Some("SetInfo Test".to_string()));
    }

    #[test]
    fn test_editable_get_info_returns_modified() {
        let mut editor = create_test_editor();

        // Set info via the trait
        let info = DocumentInfo::new()
            .title("Modified Title")
            .author("Modified Author");
        EditableDocument::set_info(&mut editor, info).unwrap();

        // Get should return the modified version
        let result = EditableDocument::get_info(&mut editor).unwrap();
        assert_eq!(result.title, Some("Modified Title".to_string()));
        assert_eq!(result.author, Some("Modified Author".to_string()));
    }

    // =========================================================================
    // set_page_content - error path
    // =========================================================================

    #[test]
    fn test_set_page_content_out_of_range() {
        let mut editor = create_test_editor();
        let content = StructureElement::default();
        let result = editor.set_page_content(99, content);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_page_content_valid() {
        let mut editor = create_test_editor();
        let content = StructureElement {
            structure_type: "Document".to_string(),
            bbox: crate::geometry::Rect::new(0.0, 0.0, 612.0, 792.0),
            children: Vec::new(),
            reading_order: Some(0),
            alt_text: None,
            language: None,
        };
        let result = editor.set_page_content(0, content);
        assert!(result.is_ok());
        assert!(editor.is_modified());
        assert!(editor.structure_modified);
        assert!(editor.modified_content.contains_key(&0));
    }

    // =========================================================================
    // Annotations query methods
    // =========================================================================

    #[test]
    fn test_has_modified_annotations_false() {
        let editor = create_test_editor();
        assert!(!editor.has_modified_annotations(0));
    }

    #[test]
    fn test_get_page_annotations_none() {
        let editor = create_test_editor();
        assert!(editor.get_page_annotations(0).is_none());
    }

    // =========================================================================
    // rewrite_content_stream_with_image_mods
    // =========================================================================

    #[test]
    fn test_rewrite_content_stream_no_mods() {
        let editor = create_test_editor();
        let content = b"q\n100 0 0 50 10 20 cm\n/Im1 Do\nQ\n";
        let mods = HashMap::new(); // empty modifications
        let result = editor
            .rewrite_content_stream_with_image_mods(content, &mods)
            .unwrap();
        let s = String::from_utf8(result).unwrap();
        // Should preserve the cm operator as-is
        assert!(s.contains("cm\n"));
        assert!(s.contains("/Im1 Do\n"));
    }

    #[test]
    fn test_rewrite_content_stream_with_position_mod() {
        let editor = create_test_editor();
        let content = b"q\n100 0 0 50 10 20 cm\n/Im1 Do\nQ\n";
        let mut mods = HashMap::new();
        mods.insert(
            "Im1".to_string(),
            ImageModification {
                x: Some(200.0),
                y: Some(300.0),
                width: None,
                height: None,
            },
        );
        let result = editor
            .rewrite_content_stream_with_image_mods(content, &mods)
            .unwrap();
        let s = String::from_utf8(result).unwrap();
        // The e and f values (position) should be modified
        assert!(s.contains("200.0"));
        assert!(s.contains("300.0"));
    }

    #[test]
    fn test_rewrite_content_stream_with_size_mod() {
        let editor = create_test_editor();
        let content = b"q\n100 0 0 50 10 20 cm\n/Im1 Do\nQ\n";
        let mut mods = HashMap::new();
        mods.insert(
            "Im1".to_string(),
            ImageModification {
                x: None,
                y: None,
                width: Some(400.0),
                height: Some(300.0),
            },
        );
        let result = editor
            .rewrite_content_stream_with_image_mods(content, &mods)
            .unwrap();
        let s = String::from_utf8(result).unwrap();
        // The a and d values (size) should be modified
        assert!(s.contains("400.0"));
        assert!(s.contains("300.0"));
    }

    // =========================================================================
    // Open with non-existent file
    // =========================================================================

    #[test]
    fn test_open_nonexistent_file() {
        let result = DocumentEditor::open("/tmp/nonexistent_pdf_oxide_test.pdf");
        assert!(result.is_err());
    }

    // =========================================================================
    // EditableDocument trait: remove_page, move_page, duplicate_page
    // =========================================================================

    /// Helper that creates a test editor with N pages (using a multi-page minimal PDF).
    fn create_multi_page_editor(n: usize) -> DocumentEditor {
        // Build a minimal PDF with n pages
        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-1.4\n");

        // Object 1: Catalog
        let catalog_offset = pdf.len();
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Build Kids array
        let mut kids = String::from("[");
        for i in 0..n {
            if i > 0 {
                kids.push(' ');
            }
            kids.push_str(&format!("{} 0 R", i + 3));
        }
        kids.push(']');

        // Object 2: Pages
        let pages_offset = pdf.len();
        let pages_str =
            format!("2 0 obj\n<< /Type /Pages /Kids {} /Count {} >>\nendobj\n", kids, n);
        pdf.extend_from_slice(pages_str.as_bytes());

        // Page objects
        let mut page_offsets = Vec::new();
        for i in 0..n {
            page_offsets.push(pdf.len());
            let page_str = format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
                i + 3
            );
            pdf.extend_from_slice(page_str.as_bytes());
        }

        // xref
        let xref_offset = pdf.len();
        let total_objects = n + 3;
        pdf.extend_from_slice(format!("xref\n0 {}\n", total_objects).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        pdf.extend_from_slice(format!("{:010} 00000 n \n", catalog_offset).as_bytes());
        pdf.extend_from_slice(format!("{:010} 00000 n \n", pages_offset).as_bytes());
        for offset in &page_offsets {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // trailer
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                total_objects, xref_offset
            )
            .as_bytes(),
        );

        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = std::env::temp_dir()
            .join(format!("pdf_oxide_multipage_test_{}_{id}.pdf", std::process::id()));
        std::fs::write(&tmp, &pdf).unwrap();
        let editor = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        editor
    }

    #[test]
    fn test_remove_page_valid() {
        let mut editor = create_multi_page_editor(3);
        assert_eq!(editor.current_page_count(), 3);
        editor.remove_page(1).unwrap();
        assert_eq!(editor.current_page_count(), 2);
        assert!(editor.is_modified());
    }

    #[test]
    fn test_remove_page_out_of_range() {
        let mut editor = create_multi_page_editor(2);
        assert!(editor.remove_page(5).is_err());
    }

    #[test]
    fn test_move_page_valid() {
        let mut editor = create_multi_page_editor(3);
        editor.move_page(0, 2).unwrap();
        assert!(editor.is_modified());
        // Page order should be rearranged
        assert_eq!(editor.page_order, vec![1, 2, 0]);
    }

    #[test]
    fn test_move_page_out_of_range() {
        let mut editor = create_multi_page_editor(2);
        assert!(editor.move_page(0, 5).is_err());
        assert!(editor.move_page(5, 0).is_err());
    }

    #[test]
    fn test_duplicate_page_valid() {
        let mut editor = create_multi_page_editor(2);
        let new_idx = editor.duplicate_page(0).unwrap();
        assert_eq!(new_idx, 2); // Was 2 pages, new one is at index 2
        assert_eq!(editor.current_page_count(), 3);
        assert!(editor.is_modified());
    }

    #[test]
    fn test_duplicate_page_out_of_range() {
        let mut editor = create_multi_page_editor(2);
        assert!(editor.duplicate_page(5).is_err());
    }

    // =========================================================================
    // get_page_info
    // =========================================================================

    #[test]
    fn test_get_page_info_valid() {
        let mut editor = create_test_editor();
        let info = editor.get_page_info(0).unwrap();
        assert_eq!(info.index, 0);
        assert_eq!(info.width, 612.0);
        assert_eq!(info.height, 792.0);
        assert_eq!(info.rotation, 0);
    }

    #[test]
    fn test_get_page_info_out_of_range() {
        let mut editor = create_test_editor();
        assert!(editor.get_page_info(99).is_err());
    }

    // =========================================================================
    // extract_pages error
    // =========================================================================

    #[test]
    fn test_extract_pages_works() {
        let mut editor = create_test_editor();
        let out = std::env::temp_dir().join("pdf_oxide_extract_test.pdf");
        let result = editor.extract_pages(&[0], &out);
        let _ = std::fs::remove_file(&out);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_pages_out_of_range() {
        let mut editor = create_test_editor();
        let out = std::env::temp_dir().join("pdf_oxide_extract_oob.pdf");
        let result = editor.extract_pages(&[99], &out);
        assert!(result.is_err());
    }

    /// Sequential chunked extraction — mirrors issue #474's S3-Lambda
    /// workflow: open once, extract non-overlapping page ranges in turn,
    /// each call must (a) succeed, (b) leave the source observably
    /// unchanged, (c) produce a PDF whose page count matches the slice.
    #[test]
    fn test_extract_pages_chunked_sequential() {
        let mut editor = create_multi_page_editor(10);
        let original_count = editor.current_page_count();

        for (start, end) in [(0, 3), (3, 6), (6, 10)] {
            let pages: Vec<usize> = (start..end).collect();
            let bytes = editor.extract_pages_to_bytes(&pages).unwrap();
            assert!(!bytes.is_empty());

            // Source document must be unchanged after each call.
            assert_eq!(
                editor.current_page_count(),
                original_count,
                "source page count changed after chunk {start}..{end}"
            );

            // Round-trip the chunk and verify its page count.
            let mut chunk = DocumentEditor::from_bytes(bytes).unwrap();
            assert_eq!(chunk.page_count().unwrap(), end - start);
        }
    }

    /// Out-of-order page selection (non-sequential indices).
    #[test]
    fn test_extract_pages_non_sequential() {
        let mut editor = create_multi_page_editor(5);
        let bytes = editor.extract_pages_to_bytes(&[3, 0, 4]).unwrap();
        let mut chunk = DocumentEditor::from_bytes(bytes).unwrap();
        assert_eq!(chunk.page_count().unwrap(), 3);
    }

    /// Batch extraction (issue #474 Option A). Each output independently
    /// reopens with the right page count.
    #[test]
    fn test_extract_page_ranges_to_bytes_batch() {
        let mut editor = create_multi_page_editor(10);
        let chunks = editor
            .extract_page_ranges_to_bytes(&[(0, 3), (3, 6), (6, 10)])
            .unwrap();
        assert_eq!(chunks.len(), 3);
        for (i, bytes) in chunks.into_iter().enumerate() {
            let mut chunk = DocumentEditor::from_bytes(bytes).unwrap();
            let expected = match i {
                0 => 3,
                1 => 3,
                _ => 4,
            };
            assert_eq!(chunk.page_count().unwrap(), expected);
        }
    }

    /// In-place selection (issue #474 Option B). After `select_pages`, the
    /// document has only the listed pages; subsequent `save_to_bytes` round-
    /// trips with the same count.
    #[test]
    fn test_select_pages_in_place() {
        let mut editor = create_multi_page_editor(5);
        editor.select_pages(&[1, 3, 4]).unwrap();
        assert_eq!(editor.current_page_count(), 3);
        let bytes = editor.save_to_bytes().unwrap();
        let mut reopened = DocumentEditor::from_bytes(bytes).unwrap();
        assert_eq!(reopened.page_count().unwrap(), 3);
    }

    /// `select_pages` validates indices.
    #[test]
    fn test_select_pages_out_of_range() {
        let mut editor = create_multi_page_editor(3);
        let result = editor.select_pages(&[5]);
        assert!(result.is_err());
    }

    // =========================================================================
    // rotate_page_by normalization
    // =========================================================================

    #[test]
    fn test_rotate_page_by_normalization() {
        let mut editor = create_test_editor();

        // Start at 0, rotate by 90
        editor.rotate_page_by(0, 90).unwrap();
        assert_eq!(editor.get_page_rotation(0).unwrap(), 90);

        // Rotate by another 90 -> 180
        editor.rotate_page_by(0, 90).unwrap();
        assert_eq!(editor.get_page_rotation(0).unwrap(), 180);

        // Rotate by another 90 -> 270
        editor.rotate_page_by(0, 90).unwrap();
        assert_eq!(editor.get_page_rotation(0).unwrap(), 270);

        // Rotate by another 90 -> 360 -> normalized to 0
        editor.rotate_page_by(0, 90).unwrap();
        assert_eq!(editor.get_page_rotation(0).unwrap(), 0);
    }

    #[test]
    fn test_rotate_all_pages() {
        let mut editor = create_multi_page_editor(3);
        editor.rotate_all_pages(90).unwrap();

        for i in 0..3 {
            assert_eq!(editor.get_page_rotation(i).unwrap(), 90);
        }
    }

    // =========================================================================
    // crop_margins
    // =========================================================================

    #[test]
    fn test_crop_margins() {
        let mut editor = create_test_editor();
        editor.crop_margins(72.0, 72.0, 72.0, 72.0).unwrap();

        // Should have set crop box for page 0
        let props = editor.modified_page_props.get(&0).unwrap();
        let crop = props.crop_box.unwrap();
        assert!((crop[0] - 72.0).abs() < 0.01); // left margin
        assert!((crop[1] - 72.0).abs() < 0.01); // bottom margin
        assert!((crop[2] - 540.0).abs() < 0.01); // 612 - 72
        assert!((crop[3] - 720.0).abs() < 0.01); // 792 - 72
    }

    // =========================================================================
    // find_max_object_id
    // =========================================================================

    #[test]
    fn test_find_max_object_id() {
        let editor = create_test_editor();
        // Our minimal PDF has /Size 4 in the trailer, so max_id should be 4
        let max = DocumentEditor::find_max_object_id(&editor.source);
        assert_eq!(max, 4);
    }

    // =========================================================================
    // Operator: MoveTo, LineTo, Tm, TD
    // =========================================================================

    #[test]
    fn test_serialize_operator_move_to() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::MoveTo { x: 10.0, y: 20.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("m\n"));
    }

    #[test]
    fn test_serialize_operator_line_to() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::LineTo { x: 100.0, y: 200.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("l\n"));
    }

    #[test]
    fn test_serialize_operator_tm() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::Tm {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 72.0,
                f: 700.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("Tm\n"));
    }

    #[test]
    fn test_serialize_operator_big_td() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::TD { tx: 0.0, ty: -14.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("TD\n"));
    }

    #[test]
    fn test_serialize_operator_set_line_cap() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetLineCap { cap_style: 1 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("J\n"));
    }

    #[test]
    fn test_serialize_operator_set_line_join() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetLineJoin { join_style: 2 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("j\n"));
    }

    #[test]
    fn test_serialize_operator_set_miter_limit() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetMiterLimit { limit: 10.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("M\n"));
    }

    #[test]
    fn test_serialize_operator_set_flatness() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFlatness { tolerance: 50.0 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("i\n"));
    }

    #[test]
    fn test_serialize_operator_set_stroke_cmyk() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeCmyk {
                c: 0.0,
                m: 1.0,
                y: 0.5,
                k: 0.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("K\n"));
    }

    #[test]
    fn test_serialize_operator_set_stroke_rgb() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetStrokeRgb {
                r: 0.0,
                g: 0.5,
                b: 1.0,
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("RG\n"));
    }

    #[test]
    fn test_serialize_operator_set_fill_gray() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::SetFillGray { gray: 0.75 },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.ends_with("g\n"));
    }

    #[test]
    fn test_serialize_operator_begin_marked_content_dict() {
        let editor = create_test_editor();
        let mut output = Vec::new();
        editor.serialize_operator(
            &mut output,
            &crate::content::operators::Operator::BeginMarkedContentDict {
                tag: "Span".to_string(),
                properties: Box::new(Object::Name("MC0".to_string())),
            },
        );
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("/Span"));
        assert!(s.contains("/MC0"));
        assert!(s.ends_with("BDC\n"));
    }

    // =========================================================================
    // Issue #262: CLI merge creates blank documents
    // =========================================================================

    #[test]
    fn test_merge_from_bytes_preserves_content() {
        // Create two PDFs with distinct text content
        let pdf1_bytes = crate::api::Pdf::from_text("Hello from document one")
            .unwrap()
            .into_bytes();
        let pdf2_bytes = crate::api::Pdf::from_text("Goodbye from document two")
            .unwrap()
            .into_bytes();

        // Merge pdf2 into pdf1
        let mut editor = DocumentEditor::from_bytes(pdf1_bytes).unwrap();
        let merged_count = editor.merge_from_bytes(&pdf2_bytes).unwrap();
        assert_eq!(merged_count, 1, "should report 1 page merged");

        let result_bytes = editor.save_to_bytes().unwrap();

        // Open the merged result and verify both pages have content
        let mut merged_doc = crate::document::PdfDocument::from_bytes(result_bytes).unwrap();
        assert_eq!(merged_doc.page_count().unwrap(), 2, "merged PDF should have 2 pages");

        // Extract text from page 1 (original)
        let text1 = merged_doc.extract_text(0).unwrap();
        assert!(
            text1.contains("Hello from document one"),
            "Page 1 should contain original text, got: {:?}",
            text1
        );

        // Extract text from page 2 (merged)
        let text2 = merged_doc.extract_text(1).unwrap();
        assert!(
            text2.contains("Goodbye from document two"),
            "Page 2 should contain merged text, got: {:?}",
            text2
        );
    }

    #[test]
    fn test_merge_multiple_pages() {
        // Create PDFs with different content
        let pdf1_bytes = crate::api::Pdf::from_text("First").unwrap().into_bytes();
        let pdf2_bytes = crate::api::Pdf::from_text("Second").unwrap().into_bytes();
        let pdf3_bytes = crate::api::Pdf::from_text("Third").unwrap().into_bytes();

        let mut editor = DocumentEditor::from_bytes(pdf1_bytes).unwrap();
        editor.merge_from_bytes(&pdf2_bytes).unwrap();
        editor.merge_from_bytes(&pdf3_bytes).unwrap();

        let result_bytes = editor.save_to_bytes().unwrap();

        let mut merged_doc = crate::document::PdfDocument::from_bytes(result_bytes).unwrap();
        assert_eq!(merged_doc.page_count().unwrap(), 3, "merged PDF should have 3 pages");

        let text1 = merged_doc.extract_text(0).unwrap();
        let text2 = merged_doc.extract_text(1).unwrap();
        let text3 = merged_doc.extract_text(2).unwrap();

        assert!(text1.contains("First"), "Page 1 text: {:?}", text1);
        assert!(text2.contains("Second"), "Page 2 text: {:?}", text2);
        assert!(text3.contains("Third"), "Page 3 text: {:?}", text3);
    }

    /// Regression test: merged PDFs must have extractable text on every page.
    ///
    /// Creates two single-page PDFs with known text content, merges them,
    /// then verifies that extract_text() returns the correct content for
    /// each page of the merged document.
    #[test]
    fn test_merge_text_extractable() {
        let pdf_a = crate::api::Pdf::from_text("Hello from A")
            .unwrap()
            .into_bytes();
        let pdf_b = crate::api::Pdf::from_text("Hello from B")
            .unwrap()
            .into_bytes();

        let mut editor = DocumentEditor::from_bytes(pdf_a).unwrap();
        let pages_merged = editor.merge_from_bytes(&pdf_b).unwrap();
        assert_eq!(pages_merged, 1, "should merge exactly 1 page from B");

        let result = editor.save_to_bytes().unwrap();

        let mut doc = crate::document::PdfDocument::from_bytes(result).unwrap();
        assert_eq!(doc.page_count().unwrap(), 2, "merged PDF must have 2 pages");

        let text_page0 = doc.extract_text(0).unwrap();
        let text_page1 = doc.extract_text(1).unwrap();

        assert!(
            text_page0.contains("Hello from A"),
            "Page 0 should contain 'Hello from A', got: {:?}",
            text_page0
        );
        assert!(
            text_page1.contains("Hello from B"),
            "Page 1 should contain 'Hello from B', got: {:?}",
            text_page1
        );

        // Verify no cross-contamination
        assert!(!text_page0.contains("Hello from B"), "Page 0 should NOT contain text from B");
        assert!(!text_page1.contains("Hello from A"), "Page 1 should NOT contain text from A");
    }

    // =========================================================================
    // Regression: destructive redaction + add_text() interaction on the same
    // page (three related bugs found together — see PR description).
    // =========================================================================

    /// A minimal single-page PDF with two separate text-showing (`Tj`) runs
    /// on one Type1/Helvetica font — unlike `minimal_pdf_bytes()` above,
    /// this page has real extractable text, split into two runs so that
    /// redacting one leaves the other's extraction untouched.
    fn minimal_pdf_with_text_bytes() -> Vec<u8> {
        let content_stream: &[u8] = b"BT\n/F1 12 Tf\n1 0 0 1 100 700 Tm\n(SECRETVALUE) Tj\n\
             1 0 0 1 100 680 Tm\n(public text) Tj\nET\n";
        let mut objs: Vec<Vec<u8>> = Vec::new();
        objs.push(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec());
        objs.push(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_vec());
        objs.push(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
              /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"
                .to_vec(),
        );
        objs.push(
            [
                format!("4 0 obj\n<< /Length {} >>\nstream\n", content_stream.len()).into_bytes(),
                content_stream.to_vec(),
                b"\nendstream\nendobj\n".to_vec(),
            ]
            .concat(),
        );
        objs.push(
            b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_vec(),
        );

        let header = b"%PDF-1.4\n".to_vec();
        let mut body = header.clone();
        let mut offsets = vec![0u64];
        let mut pos = header.len() as u64;
        for o in &objs {
            offsets.push(pos);
            body.extend_from_slice(o);
            pos += o.len() as u64;
        }
        let xref_offset = pos;
        let mut xref = format!("xref\n0 {}\n", objs.len() + 1).into_bytes();
        xref.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets[1..] {
            xref.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        let trailer = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objs.len() + 1,
            xref_offset
        )
        .into_bytes();

        [body, xref, trailer].concat()
    }

    /// Writes `minimal_pdf_with_text_bytes()` to a uniquely-named temp file
    /// and opens it as a `DocumentEditor`, mirroring `create_test_editor()`
    /// above but with real extractable text on the page.
    fn create_test_editor_with_text() -> DocumentEditor {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let pdf_bytes = minimal_pdf_with_text_bytes();
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = std::env::temp_dir()
            .join(format!("pdf_oxide_redact_addtext_test_{}_{id}.pdf", std::process::id()));
        std::fs::write(&tmp, &pdf_bytes).unwrap();
        let editor = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        editor
    }

    /// Bug 1 + bug 2 regression: destructive redaction on a page that also
    /// has `add_text()` overlay additions must not silently drop the added
    /// text (bug 1: `/Contents` was overwritten wholesale instead of
    /// preserving the overlay-additions array entry), and the replacement
    /// text's Base-14 font must be usable without breaking extraction of
    /// unrelated text on the same page (bug 2: the font resource was never
    /// registered in `/Resources/Font`).
    #[test]
    fn destructive_redaction_preserves_add_text_overlay() {
        let mut editor = create_test_editor_with_text();

        // Locate "SECRETVALUE" and redact it destructively.
        let bbox = {
            let pe = editor.page_editor(0).unwrap();
            let matches = pe.page.find_text_containing("SECRETVALUE");
            assert_eq!(matches.len(), 1, "expected exactly one match for SECRETVALUE");
            matches[0].bbox()
        };
        let rect = [bbox.x, bbox.y, bbox.x + bbox.width, bbox.y + bbox.height];
        editor
            .add_redaction(0, rect, Some([0.0, 0.0, 0.0]))
            .unwrap();
        let report = editor
            .apply_redactions_destructive(crate::redaction::RedactionOptions::default())
            .unwrap();
        assert!(report.glyphs_removed > 0, "expected some glyphs to be removed");

        // Add replacement text at the same position (Base-14 font, no
        // dependency on the original page's font encoding).
        let mut pe = editor.page_editor(0).unwrap();
        let font = crate::elements::FontSpec {
            name: "Helvetica".to_string(),
            size: 12.0,
        };
        pe.page.add_text(crate::elements::TextContent::new(
            "{{REDACTED}}",
            bbox,
            font,
            crate::elements::TextStyle::default(),
        ));
        let page = pe.done().unwrap();
        editor.save_page_from_editor(page).unwrap();

        let bytes = editor.save_to_bytes().unwrap();

        // Re-open the saved bytes independently.
        let tmp = std::env::temp_dir()
            .join(format!("pdf_oxide_redact_addtext_verify_{}.pdf", std::process::id()));
        std::fs::write(&tmp, &bytes).unwrap();
        let mut verify = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let pe = verify.page_editor(0).unwrap();
        assert!(
            pe.page.find_text_containing("SECRETVALUE").is_empty(),
            "redacted text must not survive"
        );
        assert!(
            !pe.page.find_text_containing("{{REDACTED}}").is_empty(),
            "added replacement text must be present after save (was silently dropped \
             before the /Contents-array-preservation fix)"
        );
        assert!(
            !pe.page.find_text_containing("public text").is_empty(),
            "unrelated text on the same page must survive untouched — if the added \
             text's Base-14 font resource were left undeclared in /Resources/Font, a \
             strict content-stream parser would choke on the added text's `Tf` \
             operator and fail to extract this text too"
        );
    }

    /// Bug 3 regression: two separate `add_text()` + `save_page_from_editor()`
    /// round-trips on the same page must both survive to the saved output.
    /// Before the fix, `save_page()` used `HashMap::insert` on
    /// `overlay_additions`, so the second call silently discarded the first
    /// call's added element instead of accumulating both.
    #[test]
    fn multiple_add_text_calls_on_same_page_all_survive() {
        let mut editor = create_test_editor_with_text();
        let font = || crate::elements::FontSpec {
            name: "Helvetica".to_string(),
            size: 10.0,
        };
        let bbox_a = crate::geometry::Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 12.0,
        };
        let bbox_b = crate::geometry::Rect {
            x: 50.0,
            y: 80.0,
            width: 100.0,
            height: 12.0,
        };

        for (bbox, label) in [(bbox_a, "TOKEN_ONE"), (bbox_b, "TOKEN_TWO")] {
            let mut pe = editor.page_editor(0).unwrap();
            pe.page.add_text(crate::elements::TextContent::new(
                label,
                bbox,
                font(),
                crate::elements::TextStyle::default(),
            ));
            let page = pe.done().unwrap();
            editor.save_page_from_editor(page).unwrap();
        }

        let bytes = editor.save_to_bytes().unwrap();
        let tmp = std::env::temp_dir()
            .join(format!("pdf_oxide_multi_addtext_verify_{}.pdf", std::process::id()));
        std::fs::write(&tmp, &bytes).unwrap();
        let mut verify = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let pe = verify.page_editor(0).unwrap();
        assert!(
            !pe.page.find_text_containing("TOKEN_ONE").is_empty(),
            "first add_text() call must survive a second, separate save_page() call"
        );
        assert!(
            !pe.page.find_text_containing("TOKEN_TWO").is_empty(),
            "second add_text() call must also be present"
        );
    }

    /// A single-page PDF whose *original* `/Contents` is already a
    /// two-element array `[c1 c2]` (ISO 32000-1 §7.7.3.3), with the
    /// page's `/Resources` **and** `/Resources/Font` reached through
    /// *indirect* references (the common shape for a page loaded from a
    /// real document). `c1` holds the secret, `c2` holds unrelated text.
    fn multi_stream_indirect_resources_pdf_bytes() -> Vec<u8> {
        let c1: &[u8] = b"BT\n/F1 12 Tf\n1 0 0 1 100 700 Tm\n(SECRETVALUE) Tj\nET\n";
        let c2: &[u8] = b"BT\n/F1 12 Tf\n1 0 0 1 100 680 Tm\n(public text) Tj\nET\n";
        let mut objs: Vec<Vec<u8>> = Vec::new();
        objs.push(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec());
        objs.push(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_vec());
        // Page: /Contents is an array [4 0 R 5 0 R]; /Resources is an
        // indirect reference to obj 6; obj 6's /Font is an indirect
        // reference to obj 7.
        objs.push(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
              /Contents [4 0 R 5 0 R] /Resources 6 0 R >>\nendobj\n"
                .to_vec(),
        );
        objs.push(
            [
                format!("4 0 obj\n<< /Length {} >>\nstream\n", c1.len()).into_bytes(),
                c1.to_vec(),
                b"\nendstream\nendobj\n".to_vec(),
            ]
            .concat(),
        );
        objs.push(
            [
                format!("5 0 obj\n<< /Length {} >>\nstream\n", c2.len()).into_bytes(),
                c2.to_vec(),
                b"\nendstream\nendobj\n".to_vec(),
            ]
            .concat(),
        );
        objs.push(b"6 0 obj\n<< /Font 7 0 R >>\nendobj\n".to_vec());
        objs.push(b"7 0 obj\n<< /F1 8 0 R >>\nendobj\n".to_vec());
        objs.push(
            b"8 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_vec(),
        );

        let header = b"%PDF-1.4\n".to_vec();
        let mut body = header.clone();
        let mut offsets = vec![0u64];
        let mut pos = header.len() as u64;
        for o in &objs {
            offsets.push(pos);
            body.extend_from_slice(o);
            pos += o.len() as u64;
        }
        let xref_offset = pos;
        let mut xref = format!("xref\n0 {}\n", objs.len() + 1).into_bytes();
        xref.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets[1..] {
            xref.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        let trailer = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objs.len() + 1,
            xref_offset
        )
        .into_bytes();

        [body, xref, trailer].concat()
    }

    fn open_editor_from_bytes(bytes: &[u8], tag: &str) -> DocumentEditor {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp =
            std::env::temp_dir().join(format!("pdf_oxide_{tag}_{}_{id}.pdf", std::process::id()));
        std::fs::write(&tmp, bytes).unwrap();
        let editor = DocumentEditor::open(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
        editor
    }

    /// Assert that every reference in the page's `/Contents` array points
    /// to an object that is **physically defined** in the saved bytes —
    /// i.e. no *dangling* reference. This is the property strict validators
    /// (veraPDF / Poppler strict mode) reject and the one the multi-stream
    /// `/Contents` must-fix is about.
    ///
    /// This deliberately works on the raw saved bytes rather than through
    /// `DocumentEditor::load_object`: a missing object is silently
    /// recovered by `load_object` (file-scan fallback, or `Object::Null`
    /// per §7.3.10), which would *mask* a dangling reference. A strict
    /// validator has no such tolerance, so the test must not either.
    fn assert_no_dangling_contents_in_bytes(bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);
        // Find the page object's /Contents array. Our fixtures have a
        // single /Type /Page; locate its dict and the [...] after
        // /Contents.
        let page_pos = text
            .find("/Type /Page")
            .or_else(|| text.find("/Type/Page"))
            .expect("no /Type /Page object found in saved bytes");
        let contents_pos = text[page_pos..]
            .find("/Contents")
            .map(|p| page_pos + p)
            .expect("page has no /Contents");
        let after = &text[contents_pos + "/Contents".len()..];
        let after = after.trim_start();
        assert!(
            after.starts_with('['),
            "expected an array /Contents in this multi-stream fixture, got: {}",
            &after[..after.len().min(40)]
        );
        let close = after.find(']').expect("unterminated /Contents array");
        let inner = &after[1..close];

        // Parse "N G R" triples out of the array.
        let toks: Vec<&str> = inner.split_whitespace().collect();
        let mut ids: Vec<u32> = Vec::new();
        let mut i = 0;
        while i + 2 < toks.len() + 1 && i + 2 <= toks.len() {
            if toks.get(i + 2) == Some(&"R") {
                if let Ok(id) = toks[i].parse::<u32>() {
                    ids.push(id);
                }
                i += 3;
            } else {
                i += 1;
            }
        }
        assert!(!ids.is_empty(), "no object references parsed from /Contents array");

        for id in ids {
            // A defined object appears as "\n<id> <gen> obj". Accept any
            // generation number.
            let defined =
                text.contains(&format!("\n{id} 0 obj")) || text.starts_with(&format!("{id} 0 obj"));
            assert!(
                defined,
                "dangling /Contents reference to object {id}: it appears in the \
                 array but no `{id} 0 obj` is defined in the saved file — strict \
                 validators (veraPDF / Poppler strict mode) reject this"
            );
        }
    }

    /// Must-fix 1 regression: a page whose *original* `/Contents` is
    /// already a `[c1 c2]` array, redacted destructively and then given an
    /// `add_text()` overlay, must (a) drop the secret, (b) keep the
    /// unrelated stream's text, (c) keep the overlay, and (d) leave **no
    /// dangling `/Contents` reference** — the entry-0-only replacement in
    /// the original PR left `c2` dangling. Also exercises indirect
    /// `/Resources` and `/Resources/Font` (the headline complexity), which
    /// the earlier tests did not cover.
    #[test]
    fn destructive_redaction_multi_stream_contents_no_dangling_ref() {
        let mut editor =
            open_editor_from_bytes(&multi_stream_indirect_resources_pdf_bytes(), "multistream");

        let bbox = {
            let pe = editor.page_editor(0).unwrap();
            let matches = pe.page.find_text_containing("SECRETVALUE");
            assert_eq!(matches.len(), 1, "expected exactly one match for SECRETVALUE");
            matches[0].bbox()
        };
        let rect = [bbox.x, bbox.y, bbox.x + bbox.width, bbox.y + bbox.height];
        editor
            .add_redaction(0, rect, Some([0.0, 0.0, 0.0]))
            .unwrap();
        let report = editor
            .apply_redactions_destructive(crate::redaction::RedactionOptions::default())
            .unwrap();
        assert!(report.glyphs_removed > 0, "expected some glyphs to be removed");

        let mut pe = editor.page_editor(0).unwrap();
        let font = crate::elements::FontSpec {
            name: "Helvetica".to_string(),
            size: 12.0,
        };
        pe.page.add_text(crate::elements::TextContent::new(
            "{{REDACTED}}",
            bbox,
            font,
            crate::elements::TextStyle::default(),
        ));
        let page = pe.done().unwrap();
        editor.save_page_from_editor(page).unwrap();

        let bytes = editor.save_to_bytes().unwrap();

        // (d) No dangling /Contents reference after the multi-stream
        // rewrite — this is the core of must-fix 1. Checked on the raw
        // bytes, since load_object() would silently mask a missing object.
        assert_no_dangling_contents_in_bytes(&bytes);

        let mut verify = open_editor_from_bytes(&bytes, "multistream_verify");

        let pe = verify.page_editor(0).unwrap();
        // (a) secret gone.
        assert!(
            pe.page.find_text_containing("SECRETVALUE").is_empty(),
            "redacted text must not survive"
        );
        // (b) unrelated original stream (c2) text survives.
        assert!(
            !pe.page.find_text_containing("public text").is_empty(),
            "text from the *second* original content stream must survive the redaction"
        );
        // (c) overlay survives.
        assert!(
            !pe.page.find_text_containing("{{REDACTED}}").is_empty(),
            "add_text() overlay must survive destructive redaction of a multi-stream page"
        );
    }

    /// Must-fix 2 regression: a **bold** overlay makes the writer emit
    /// `/Helvetica-Bold` via `map_font_name`, so the registration must key
    /// off that transformed name — not the raw `"Helvetica"` — or the
    /// `Tf` operator references an undeclared font and a strict parser
    /// chokes on the overlay (and, via the shared stream, unrelated text).
    #[test]
    fn styled_overlay_font_registered_under_emitted_name() {
        let mut editor = create_test_editor_with_text();

        let bbox = crate::geometry::Rect {
            x: 100.0,
            y: 600.0,
            width: 120.0,
            height: 14.0,
        };
        let mut pe = editor.page_editor(0).unwrap();
        let font = crate::elements::FontSpec {
            name: "Helvetica".to_string(),
            size: 12.0,
        };
        let style = crate::elements::TextStyle {
            weight: crate::layout::FontWeight::Bold,
            ..crate::elements::TextStyle::default()
        };
        pe.page
            .add_text(crate::elements::TextContent::new("BOLDTOKEN", bbox, font, style));
        let page = pe.done().unwrap();
        editor.save_page_from_editor(page).unwrap();

        let bytes = editor.save_to_bytes().unwrap();
        let mut verify = open_editor_from_bytes(&bytes, "boldfont_verify");

        // The bold overlay text and the original page text must both
        // extract cleanly: if `/Helvetica-Bold` were unregistered while
        // the stream emits `/Helvetica-Bold Tf`, a strict parser would
        // fail on the overlay's `Tf`.
        let pe = verify.page_editor(0).unwrap();
        assert!(
            !pe.page.find_text_containing("BOLDTOKEN").is_empty(),
            "bold overlay text must survive — the emitted /Helvetica-Bold must be registered"
        );
        assert!(
            !pe.page.find_text_containing("public text").is_empty(),
            "unrelated original text must still extract"
        );

        // The emitted font name (/Helvetica-Bold) must be present in the
        // saved bytes as a registered **/BaseFont** — not merely as the
        // `/Helvetica-Bold Tf` operand inside the content stream (which is
        // there regardless of registration). Checking for the `/BaseFont`
        // key proves registration keyed off the transformed name rather
        // than the raw "Helvetica".
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(
            haystack.contains("/BaseFont /Helvetica-Bold")
                || haystack.contains("/BaseFont/Helvetica-Bold"),
            "a Font object with /BaseFont /Helvetica-Bold must be written — the \
             overlay stream emits `/Helvetica-Bold Tf`, so registering the raw \
             \"Helvetica\" leaves that Tf dangling"
        );
    }

    /// A single-page PDF where the page has NO own `/Resources` and inherits
    /// it from the `/Pages` node. The inherited `/Resources` carries both a
    /// `/Font` sub-dict (needed by the page's text) and an `/XObject`
    /// sub-dict referencing a Form XObject (`/SecretForm`) whose stream
    /// contains sensitive text. This models the shape behind the
    /// inherited-/Resources leak concern.
    fn inherited_resources_with_xobject_pdf_bytes() -> Vec<u8> {
        let content: &[u8] = b"BT\n/F1 12 Tf\n1 0 0 1 100 700 Tm\n(visible text) Tj\nET\n";
        let secret: &[u8] = b"BT\n/F1 12 Tf\n1 0 0 1 10 10 Tm\n(XOBJECTSECRET) Tj\nET\n";
        let mut objs: Vec<Vec<u8>> = Vec::new();
        objs.push(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec());
        // /Pages node holds the inheritable /Resources (Font + XObject).
        objs.push(
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 \
              /Resources << /Font << /F1 5 0 R >> /XObject << /SecretForm 6 0 R >> >> >>\nendobj\n"
                .to_vec(),
        );
        // Page: NO own /Resources — must inherit from /Pages (§7.7.3.4).
        objs.push(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
              /Contents 4 0 R >>\nendobj\n"
                .to_vec(),
        );
        objs.push(
            [
                format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len()).into_bytes(),
                content.to_vec(),
                b"\nendstream\nendobj\n".to_vec(),
            ]
            .concat(),
        );
        objs.push(
            b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_vec(),
        );
        objs.push(
            [
                format!(
                    "6 0 obj\n<< /Type /XObject /Subtype /Form /BBox [0 0 20 20] /Length {} >>\nstream\n",
                    secret.len()
                )
                .into_bytes(),
                secret.to_vec(),
                b"\nendstream\nendobj\n".to_vec(),
            ]
            .concat(),
        );

        let header = b"%PDF-1.4\n".to_vec();
        let mut body = header.clone();
        let mut offsets = vec![0u64];
        let mut pos = header.len() as u64;
        for o in &objs {
            offsets.push(pos);
            body.extend_from_slice(o);
            pos += o.len() as u64;
        }
        let xref_offset = pos;
        let mut xref = format!("xref\n0 {}\n", objs.len() + 1).into_bytes();
        xref.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets[1..] {
            xref.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        let trailer = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objs.len() + 1,
            xref_offset
        )
        .into_bytes();

        [body, xref, trailer].concat()
    }

    /// Adversarial-review regression: when overlay text forces the page's
    /// `/Resources` to be materialised and the page inherits `/Resources`
    /// from an ancestor, the materialised dict must seed ONLY the inherited
    /// `/Font` — never the inherited `/XObject` (or other content-bearing
    /// resources). Inlining an inherited Form XObject reference onto the
    /// page would make that stream freshly reachable and let it survive a
    /// later destructive-redaction orphan-drop, re-introducing a stray
    /// reference. The overlay font must still be registered so the added
    /// text's `Tf` does not dangle.
    #[test]
    fn overlay_does_not_inline_inherited_xobject_into_page_resources() {
        let mut editor =
            open_editor_from_bytes(&inherited_resources_with_xobject_pdf_bytes(), "inherit_xobj");

        let bbox = crate::geometry::Rect {
            x: 100.0,
            y: 500.0,
            width: 120.0,
            height: 14.0,
        };
        let mut pe = editor.page_editor(0).unwrap();
        let font = crate::elements::FontSpec {
            name: "Helvetica".to_string(),
            size: 12.0,
        };
        pe.page.add_text(crate::elements::TextContent::new(
            "OVERLAYTOKEN",
            bbox,
            font,
            crate::elements::TextStyle::default(),
        ));
        let page = pe.done().unwrap();
        editor.save_page_from_editor(page).unwrap();

        let bytes = editor.save_to_bytes().unwrap();
        let mut verify = open_editor_from_bytes(&bytes, "inherit_xobj_verify");

        // Inspect the saved page's OWN /Resources (now materialised).
        let page_ref = verify.source.get_page_ref(0).unwrap();
        let page_obj = verify.source.load_object(page_ref).unwrap();
        let page_dict = page_obj.as_dict().unwrap();
        let res_obj = page_dict
            .get("Resources")
            .cloned()
            .expect("page should have a materialised /Resources after overlay");
        let res_dict = match res_obj {
            Object::Dictionary(d) => d,
            Object::Reference(r) => verify
                .source
                .load_object(r)
                .unwrap()
                .as_dict()
                .cloned()
                .unwrap(),
            other => panic!("unexpected /Resources object: {other:?}"),
        };

        // The overlay font must be registered (Tf must resolve)...
        assert!(
            res_dict.contains_key("Font"),
            "materialised /Resources must carry a /Font sub-dict for the overlay"
        );
        // ...but the inherited /XObject must NOT have been inlined onto the page.
        assert!(
            !res_dict.contains_key("XObject"),
            "inherited /XObject must not be copied into the page's /Resources — \
             doing so re-references the Form XObject and would defeat a later \
             destructive-redaction orphan-drop"
        );

        // The overlay text still extracts.
        let pe = verify.page_editor(0).unwrap();
        assert!(
            !pe.page.find_text_containing("OVERLAYTOKEN").is_empty(),
            "overlay text must survive with its font registered"
        );
    }
}
