//! PDF editing module for modifying existing PDF documents.
#![allow(
    dead_code,
    unused_variables,
    unused_mut,
    missing_docs,
    clippy::write_with_newline
)]
//!
//! This module provides a high-level API for editing PDF documents:
//! - Metadata editing (title, author, subject, keywords)
//! - Page operations (add, remove, reorder, extract)
//! - Content manipulation
//! - PDF merging
//!
//! ## Architecture
//!
//! ```text
//! PdfDocument (read-only source)
//!     ↓
//! [DocumentEditor] (tracks modifications)
//!     ↓
//! Save Options:
//!   - Incremental update (append to original)
//!   - Full rewrite (new PDF structure)
//! ```
//!
//! ## Encryption Handling
//!
//! When opening encrypted PDFs:
//!
//! - **Reading**: Encrypted PDFs are decrypted transparently when opened.
//!   The user/owner password can be provided via `PdfDocument::open_with_password()`.
//!   Once opened, all content is accessible in decrypted form.
//!
//! - **Writing**: Saved PDFs are written **unencrypted** by default.
//!   The original encryption is **not preserved** during save operations.
//!   This is intentional as encryption requires separate configuration.
//!
//! ### Current Limitations
//!
//! Re-encryption on save is not yet supported. If you need to preserve encryption:
//!
//! 1. Save the modified PDF without encryption
//! 2. Use an external tool to re-encrypt:
//!    ```bash
//!    qpdf --encrypt user-pass owner-pass 256 -- unencrypted.pdf encrypted.pdf
//!    ```
//!
//! ### Planned for v0.4.0
//!
//! `SaveOptions::with_encryption()` will allow specifying encryption on save:
//!
//! ```ignore
//! // Future API (v0.4.0)
//! editor.save_with_options("output.pdf", SaveOptions::full_rewrite()
//!     .with_encryption(EncryptionConfig {
//!         user_password: "user123".to_string(),
//!         owner_password: "owner456".to_string(),
//!         algorithm: EncryptionAlgorithm::Aes256,
//!         permissions: Permissions::default(),
//!     })
//! )?;
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::editor::DocumentEditor;
//!
//! // Open an existing PDF for editing
//! let mut editor = DocumentEditor::open("input.pdf")?;
//!
//! // Edit metadata
//! editor.set_title("Updated Title");
//! editor.set_author("John Doe");
//!
//! // Modify pages
//! editor.remove_page(5)?;
//! editor.move_page(2, 0)?;  // Move page 2 to front
//!
//! // Save changes
//! editor.save("output.pdf")?;  // Full rewrite
//! // or
//! editor.save_incremental("output.pdf")?;  // Append changes
//! ```

mod document_editor;
pub mod dom;
pub mod form_fields;
pub mod resource_manager;

pub use document_editor::{
    DocumentEditor, DocumentInfo, EditableDocument, EncryptionAlgorithm, EncryptionConfig,
    ImageInfo, PageInfo, Permissions, SaveOptions,
};
pub use dom::{
    AnnotationId, AnnotationWrapper, ElementId, ImageElementCollectionEditor, PageEditor,
    PathElementCollectionEditor, PdfElement, PdfImage, PdfPage, PdfPath, PdfStructure, PdfTable,
    PdfText, TableElementCollectionEditor, TextElementCollectionEditor,
};
pub use form_fields::{
    FormFieldType, FormFieldValue, FormFieldWrapper, ParentFieldConfig, WidgetConfig,
};
pub use resource_manager::ResourceManager;
