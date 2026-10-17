//! Stamp annotations for PDF generation.
//!
//! This module provides support for Stamp annotations per PDF spec Section 12.5.6.12.
//! Stamp annotations display text or graphics intended to look like rubber stamps.
//!
//! # Standard Stamp Types
//!
//! PDF defines several standard stamp names:
//! - Approved, Experimental, NotApproved, AsIs, Expired
//! - NotForPublicRelease, Confidential, Final, Sold
//! - Departmental, ForComment, TopSecret, Draft, ForPublicRelease
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::StampAnnotation;
//!
//! // Create an approval stamp
//! let stamp = StampAnnotation::approved(Rect::new(100.0, 700.0, 150.0, 50.0));
//!
//! // Create a custom stamp
//! let custom = StampAnnotation::custom(Rect::new(100.0, 600.0, 150.0, 50.0), "ReviewPending");
//! ```

use crate::annotation_types::AnnotationFlags;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Standard stamp types as defined in PDF spec Section 12.5.6.12.
#[derive(Debug, Clone, PartialEq)]
pub enum StampType {
    /// Document approved
    Approved,
    /// Experimental content
    Experimental,
    /// Document not approved
    NotApproved,
    /// As-is condition
    AsIs,
    /// Content has expired
    Expired,
    /// Not for public release
    NotForPublicRelease,
    /// Confidential information
    Confidential,
    /// Final version
    Final,
    /// Document is sold
    Sold,
    /// Departmental use
    Departmental,
    /// For comment/review
    ForComment,
    /// Top secret classification
    TopSecret,
    /// Draft document
    Draft,
    /// For public release
    ForPublicRelease,
    /// Custom stamp with user-defined name
    Custom(String),
}

impl StampType {
    /// Get the PDF name for this stamp type.
    pub fn pdf_name(&self) -> String {
        match self {
            StampType::Approved => "Approved".to_string(),
            StampType::Experimental => "Experimental".to_string(),
            StampType::NotApproved => "NotApproved".to_string(),
            StampType::AsIs => "AsIs".to_string(),
            StampType::Expired => "Expired".to_string(),
            StampType::NotForPublicRelease => "NotForPublicRelease".to_string(),
            StampType::Confidential => "Confidential".to_string(),
            StampType::Final => "Final".to_string(),
            StampType::Sold => "Sold".to_string(),
            StampType::Departmental => "Departmental".to_string(),
            StampType::ForComment => "ForComment".to_string(),
            StampType::TopSecret => "TopSecret".to_string(),
            StampType::Draft => "Draft".to_string(),
            StampType::ForPublicRelease => "ForPublicRelease".to_string(),
            StampType::Custom(name) => name.clone(),
        }
    }
}

/// A Stamp annotation per PDF spec Section 12.5.6.12.
///
/// Stamp annotations display text or graphics intended to look
/// like a rubber stamp. The appearance can be:
/// - A standard stamp with predefined appearance
/// - A custom stamp with user-defined name
#[derive(Debug, Clone)]
pub struct StampAnnotation {
    /// Bounding rectangle for the stamp
    pub rect: Rect,
    /// Type of stamp
    pub stamp_type: StampType,
    /// Contents/comment
    pub contents: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Subject
    pub subject: Option<String>,
    /// Annotation flags
    pub flags: AnnotationFlags,
    /// Creation date
    pub creation_date: Option<String>,
    /// Modification date
    pub modification_date: Option<String>,
    /// Opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,
}

impl StampAnnotation {
    /// Create a new stamp annotation with the given rect and stamp type.
    pub fn new(rect: Rect, stamp_type: StampType) -> Self {
        Self {
            rect,
            stamp_type,
            contents: None,
            author: None,
            subject: None,
            flags: AnnotationFlags::printable(),
            creation_date: None,
            modification_date: None,
            opacity: None,
        }
    }

    /// Create an "Approved" stamp.
    pub fn approved(rect: Rect) -> Self {
        Self::new(rect, StampType::Approved)
    }

    /// Create a "Draft" stamp.
    pub fn draft(rect: Rect) -> Self {
        Self::new(rect, StampType::Draft)
    }

    /// Create a "Confidential" stamp.
    pub fn confidential(rect: Rect) -> Self {
        Self::new(rect, StampType::Confidential)
    }

    /// Create a "Final" stamp.
    pub fn final_stamp(rect: Rect) -> Self {
        Self::new(rect, StampType::Final)
    }

    /// Create an "Experimental" stamp.
    pub fn experimental(rect: Rect) -> Self {
        Self::new(rect, StampType::Experimental)
    }

    /// Create a "Not Approved" stamp.
    pub fn not_approved(rect: Rect) -> Self {
        Self::new(rect, StampType::NotApproved)
    }

    /// Create an "As Is" stamp.
    pub fn as_is(rect: Rect) -> Self {
        Self::new(rect, StampType::AsIs)
    }

    /// Create an "Expired" stamp.
    pub fn expired(rect: Rect) -> Self {
        Self::new(rect, StampType::Expired)
    }

    /// Create a "Not For Public Release" stamp.
    pub fn not_for_public_release(rect: Rect) -> Self {
        Self::new(rect, StampType::NotForPublicRelease)
    }

    /// Create a "For Public Release" stamp.
    pub fn for_public_release(rect: Rect) -> Self {
        Self::new(rect, StampType::ForPublicRelease)
    }

    /// Create a "Departmental" stamp.
    pub fn departmental(rect: Rect) -> Self {
        Self::new(rect, StampType::Departmental)
    }

    /// Create a "For Comment" stamp.
    pub fn for_comment(rect: Rect) -> Self {
        Self::new(rect, StampType::ForComment)
    }

    /// Create a "Top Secret" stamp.
    pub fn top_secret(rect: Rect) -> Self {
        Self::new(rect, StampType::TopSecret)
    }

    /// Create a "Sold" stamp.
    pub fn sold(rect: Rect) -> Self {
        Self::new(rect, StampType::Sold)
    }

    /// Create a custom stamp with a user-defined name.
    pub fn custom(rect: Rect, name: impl Into<String>) -> Self {
        Self::new(rect, StampType::Custom(name.into()))
    }

    /// Set the stamp type.
    pub fn with_stamp_type(mut self, stamp_type: StampType) -> Self {
        self.stamp_type = stamp_type;
        self
    }

    /// Set contents/comment for the stamp.
    pub fn with_contents(mut self, contents: impl Into<String>) -> Self {
        self.contents = Some(contents.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the subject.
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the annotation flags.
    pub fn with_flags(mut self, flags: AnnotationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set the opacity (0.0 = transparent, 1.0 = opaque).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Build the annotation dictionary for PDF output.
    pub fn build(&self, _page_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Required entries
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Stamp".to_string()));

        // Rectangle
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );

        // Stamp name (required)
        dict.insert("Name".to_string(), Object::Name(self.stamp_type.pdf_name()));

        // Contents
        if let Some(ref contents) = self.contents {
            dict.insert("Contents".to_string(), Object::text_string(contents));
        }

        // Flags
        if self.flags.bits() != 0 {
            dict.insert("F".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Opacity
        if let Some(opacity) = self.opacity {
            dict.insert("CA".to_string(), Object::Real(opacity as f64));
        }

        // Author
        if let Some(ref author) = self.author {
            dict.insert("T".to_string(), Object::text_string(author));
        }

        // Subject
        if let Some(ref subject) = self.subject {
            dict.insert("Subj".to_string(), Object::text_string(subject));
        }

        // Creation date
        if let Some(ref date) = self.creation_date {
            dict.insert("CreationDate".to_string(), Object::text_string(date));
        }

        // Modification date
        if let Some(ref date) = self.modification_date {
            dict.insert("M".to_string(), Object::text_string(date));
        }

        dict
    }

    /// Get the bounding rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

impl Default for StampAnnotation {
    fn default() -> Self {
        Self::new(Rect::new(0.0, 0.0, 100.0, 50.0), StampType::Draft)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stamp_new() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::new(rect, StampType::Approved);

        assert_eq!(stamp.rect, rect);
        assert_eq!(stamp.stamp_type, StampType::Approved);
    }

    #[test]
    fn test_stamp_approved() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::approved(rect);

        assert_eq!(stamp.stamp_type, StampType::Approved);
    }

    #[test]
    fn test_stamp_draft() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::draft(rect);

        assert_eq!(stamp.stamp_type, StampType::Draft);
    }

    #[test]
    fn test_stamp_confidential() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::confidential(rect);

        assert_eq!(stamp.stamp_type, StampType::Confidential);
    }

    #[test]
    fn test_stamp_custom() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::custom(rect, "ReviewPending");

        assert_eq!(stamp.stamp_type, StampType::Custom("ReviewPending".to_string()));
    }

    #[test]
    fn test_stamp_type_pdf_names() {
        assert_eq!(StampType::Approved.pdf_name(), "Approved");
        assert_eq!(StampType::Draft.pdf_name(), "Draft");
        assert_eq!(StampType::Confidential.pdf_name(), "Confidential");
        assert_eq!(StampType::NotApproved.pdf_name(), "NotApproved");
        assert_eq!(StampType::TopSecret.pdf_name(), "TopSecret");
        assert_eq!(StampType::Custom("MyStamp".to_string()).pdf_name(), "MyStamp");
    }

    #[test]
    fn test_stamp_fluent_builder() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::approved(rect)
            .with_contents("Approved by manager")
            .with_author("John Doe")
            .with_subject("Approval Stamp")
            .with_opacity(0.9);

        assert_eq!(stamp.contents, Some("Approved by manager".to_string()));
        assert_eq!(stamp.author, Some("John Doe".to_string()));
        assert_eq!(stamp.subject, Some("Approval Stamp".to_string()));
        assert_eq!(stamp.opacity, Some(0.9));
    }

    #[test]
    fn test_stamp_build() {
        let rect = Rect::new(100.0, 700.0, 150.0, 50.0);
        let stamp = StampAnnotation::approved(rect).with_contents("Approved");

        let dict = stamp.build(&[]);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Stamp".to_string())));
        assert_eq!(dict.get("Name"), Some(&Object::Name("Approved".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("Contents"));
    }

    #[test]
    fn test_all_standard_stamps() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);

        // Test all standard stamp types can be created
        let stamps = vec![
            StampAnnotation::approved(rect),
            StampAnnotation::draft(rect),
            StampAnnotation::confidential(rect),
            StampAnnotation::final_stamp(rect),
            StampAnnotation::experimental(rect),
            StampAnnotation::not_approved(rect),
            StampAnnotation::as_is(rect),
            StampAnnotation::expired(rect),
            StampAnnotation::not_for_public_release(rect),
            StampAnnotation::for_public_release(rect),
            StampAnnotation::departmental(rect),
            StampAnnotation::for_comment(rect),
            StampAnnotation::top_secret(rect),
            StampAnnotation::sold(rect),
        ];

        assert_eq!(stamps.len(), 14);

        // Verify each builds correctly
        for stamp in stamps {
            let dict = stamp.build(&[]);
            assert!(dict.contains_key("Name"));
        }
    }
}
