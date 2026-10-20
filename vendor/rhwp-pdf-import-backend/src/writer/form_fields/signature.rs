//! Signature field widget placeholder for PDF forms.
//!
//! Implements an unsigned `/FT /Sig` field per ISO 32000-1:2008 §12.8.
//! The field acts as a reserved slot that a signing application can fill
//! in later; no cryptographic signature is embedded at creation time.

use super::{FormFieldEntry, FormFieldWidget};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// An unsigned signature field placeholder.
///
/// Places a visible signature box on the page. The `/V` value is `null`
/// (unsigned). A signing tool can fill it in via incremental update.
#[derive(Debug, Clone)]
pub struct SignatureWidget {
    /// Partial field name
    name: String,
    /// Bounding rectangle
    rect: Rect,
    /// Optional tooltip
    tooltip: Option<String>,
    /// Read-only flag
    read_only: bool,
}

impl SignatureWidget {
    /// Create a new unsigned signature placeholder.
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            tooltip: None,
            read_only: false,
        }
    }

    /// Set tooltip text.
    pub fn with_tooltip(mut self, tip: impl Into<String>) -> Self {
        self.tooltip = Some(tip.into());
        self
    }

    /// Mark the field as read-only (viewer shows it but won't let it be signed).
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Build to a `FormFieldEntry`.
    pub fn build_entry(&self, page_ref: ObjectRef) -> FormFieldEntry {
        FormFieldEntry {
            widget_dict: self.build_widget_dict(page_ref),
            field_dict: self.build_field_dict(),
            name: self.name.clone(),
            rect: self.rect,
            field_type: "Sig".to_string(),
        }
    }
}

impl FormFieldWidget for SignatureWidget {
    fn field_name(&self) -> &str {
        &self.name
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn field_type(&self) -> &'static str {
        "Sig"
    }

    fn field_flags(&self) -> u32 {
        if self.read_only {
            1
        } else {
            0
        }
    }

    fn build_field_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();
        dict.insert("FT".to_string(), Object::Name("Sig".to_string()));
        dict.insert("T".to_string(), Object::text_string(&self.name));
        // /V = null means the field is unsigned
        dict.insert("V".to_string(), Object::Null);
        if self.read_only {
            dict.insert("Ff".to_string(), Object::Integer(1));
        }
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }
        dict
    }

    fn build_widget_dict(&self, page_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Widget".to_string()));
        dict.insert(
            "Rect".to_string(),
            Object::Array(vec![
                Object::Real(self.rect.x as f64),
                Object::Real(self.rect.y as f64),
                Object::Real((self.rect.x + self.rect.width) as f64),
                Object::Real((self.rect.y + self.rect.height) as f64),
            ]),
        );
        dict.insert("P".to_string(), Object::Reference(page_ref));
        // /H /N — no highlight effect for signature widgets
        dict.insert("H".to_string(), Object::Name("N".to_string()));
        dict
    }
}
