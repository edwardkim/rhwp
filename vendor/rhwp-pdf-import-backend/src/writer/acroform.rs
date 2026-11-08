//! AcroForm builder for interactive PDF forms.
//!
//! Implements the document-level AcroForm dictionary per ISO 32000-1:2008 Section 12.7.2.
//!
//! The AcroForm dictionary is stored in the document catalog and contains:
//! - References to all form fields
//! - Default resources (fonts, etc.)
//! - Default appearance string
//! - Signature flags and other options
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::AcroFormBuilder;
//!
//! let acroform = AcroFormBuilder::new()
//!     .need_appearances()
//!     .with_default_appearance("/Helv 12 Tf 0 g");
//! ```

use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Builder for the document-level AcroForm dictionary.
///
/// This dictionary defines the document's interactive form properties
/// and contains references to all form fields.
#[derive(Debug, Clone)]
pub struct AcroFormBuilder {
    /// Field object references
    fields: Vec<ObjectRef>,
    /// Whether to regenerate appearances
    need_appearances: bool,
    /// Signature flags
    sig_flags: Option<u32>,
    /// Default appearance string
    default_appearance: Option<String>,
    /// Calculation order (field refs in order to calculate)
    calc_order: Vec<ObjectRef>,
    /// XFA form data (not typically used with AcroForms)
    xfa: Option<Object>,
}

impl Default for AcroFormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AcroFormBuilder {
    /// Create a new AcroForm builder.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            need_appearances: true, // Default to true for compatibility
            sig_flags: None,
            default_appearance: Some("/Helv 12 Tf 0 g".to_string()),
            calc_order: Vec::new(),
            xfa: None,
        }
    }

    /// Add a field reference.
    pub fn add_field(&mut self, field_ref: ObjectRef) {
        self.fields.push(field_ref);
    }

    /// Add multiple field references.
    pub fn add_fields(&mut self, fields: impl IntoIterator<Item = ObjectRef>) {
        self.fields.extend(fields);
    }

    /// Set whether the PDF viewer should regenerate field appearances.
    ///
    /// When true (default), the viewer generates appearances automatically.
    /// When false, the PDF must provide appearance streams for all fields.
    pub fn need_appearances(mut self) -> Self {
        self.need_appearances = true;
        self
    }

    /// Disable automatic appearance generation.
    ///
    /// Use this when providing custom appearance streams for all fields.
    pub fn no_need_appearances(mut self) -> Self {
        self.need_appearances = false;
        self
    }

    /// Set the NeedAppearances flag explicitly.
    pub fn with_need_appearances(mut self, need: bool) -> Self {
        self.need_appearances = need;
        self
    }

    /// Set the default appearance string.
    ///
    /// This string defines the default font and text properties for text fields.
    /// Format: "/FontName size Tf r g b rg" (e.g., "/Helv 12 Tf 0 g")
    pub fn with_default_appearance(mut self, da: impl Into<String>) -> Self {
        self.default_appearance = Some(da.into());
        self
    }

    /// Set signature flags.
    ///
    /// Per PDF spec Table 219:
    /// - Bit 1: SignaturesExist - document contains signatures
    /// - Bit 2: AppendOnly - document shall be saved with incremental updates
    pub fn with_sig_flags(mut self, flags: u32) -> Self {
        self.sig_flags = Some(flags);
        self
    }

    /// Mark document as containing signatures.
    pub fn signatures_exist(mut self) -> Self {
        let flags = self.sig_flags.unwrap_or(0);
        self.sig_flags = Some(flags | 1);
        self
    }

    /// Mark document as append-only (for signed documents).
    pub fn append_only(mut self) -> Self {
        let flags = self.sig_flags.unwrap_or(0);
        self.sig_flags = Some(flags | 2);
        self
    }

    /// Set the calculation order for calculated fields.
    ///
    /// Fields are calculated in the order specified.
    pub fn with_calc_order(mut self, order: Vec<ObjectRef>) -> Self {
        self.calc_order = order;
        self
    }

    /// Check if this form has any fields.
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    /// Get the number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Build the AcroForm dictionary.
    ///
    /// # Arguments
    ///
    /// * `font_dict_ref` - Optional reference to a font dictionary for default resources
    pub fn build(&self, font_dict_ref: Option<ObjectRef>) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Fields array (required)
        let fields: Vec<Object> = self.fields.iter().map(|r| Object::Reference(*r)).collect();
        dict.insert("Fields".to_string(), Object::Array(fields));

        // NeedAppearances
        if self.need_appearances {
            dict.insert("NeedAppearances".to_string(), Object::Boolean(true));
        }

        // Signature flags
        if let Some(flags) = self.sig_flags {
            dict.insert("SigFlags".to_string(), Object::Integer(flags as i64));
        }

        // Default appearance
        if let Some(ref da) = self.default_appearance {
            dict.insert("DA".to_string(), Object::text_string(da));
        }

        // Default resources (DR)
        if let Some(font_ref) = font_dict_ref {
            let mut dr = HashMap::new();
            let mut font = HashMap::new();

            // Add standard form fonts
            // Helv is typically Helvetica
            font.insert("Helv".to_string(), Object::Reference(font_ref));

            dr.insert("Font".to_string(), Object::Dictionary(font));
            dict.insert("DR".to_string(), Object::Dictionary(dr));
        }

        // Calculation order
        if !self.calc_order.is_empty() {
            let co: Vec<Object> = self
                .calc_order
                .iter()
                .map(|r| Object::Reference(*r))
                .collect();
            dict.insert("CO".to_string(), Object::Array(co));
        }

        // XFA (if present)
        if let Some(ref xfa) = self.xfa {
            dict.insert("XFA".to_string(), xfa.clone());
        }

        dict
    }

    /// Build a minimal DR (Default Resources) dictionary with standard form fonts.
    ///
    /// Returns the DR dictionary that should be embedded in the AcroForm.
    pub fn build_default_resources() -> HashMap<String, Object> {
        let mut dr = HashMap::new();

        // Font dictionary with standard PDF fonts
        let mut fonts = HashMap::new();

        // Helvetica (Helv)
        let mut helv = HashMap::new();
        helv.insert("Type".to_string(), Object::Name("Font".to_string()));
        helv.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        helv.insert("BaseFont".to_string(), Object::Name("Helvetica".to_string()));
        helv.insert("Encoding".to_string(), Object::Name("WinAnsiEncoding".to_string()));
        fonts.insert("Helv".to_string(), Object::Dictionary(helv));

        // Courier (Cour)
        let mut cour = HashMap::new();
        cour.insert("Type".to_string(), Object::Name("Font".to_string()));
        cour.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        cour.insert("BaseFont".to_string(), Object::Name("Courier".to_string()));
        cour.insert("Encoding".to_string(), Object::Name("WinAnsiEncoding".to_string()));
        fonts.insert("Cour".to_string(), Object::Dictionary(cour));

        // Times Roman (TiRo)
        let mut tiro = HashMap::new();
        tiro.insert("Type".to_string(), Object::Name("Font".to_string()));
        tiro.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        tiro.insert("BaseFont".to_string(), Object::Name("Times-Roman".to_string()));
        tiro.insert("Encoding".to_string(), Object::Name("WinAnsiEncoding".to_string()));
        fonts.insert("TiRo".to_string(), Object::Dictionary(tiro));

        // ZapfDingbats (ZaDb) - for checkboxes and radio buttons
        let mut zadb = HashMap::new();
        zadb.insert("Type".to_string(), Object::Name("Font".to_string()));
        zadb.insert("Subtype".to_string(), Object::Name("Type1".to_string()));
        zadb.insert("BaseFont".to_string(), Object::Name("ZapfDingbats".to_string()));
        fonts.insert("ZaDb".to_string(), Object::Dictionary(zadb));

        dr.insert("Font".to_string(), Object::Dictionary(fonts));

        dr
    }

    /// Build complete AcroForm dictionary with embedded resources.
    ///
    /// This is a convenience method that includes standard fonts in the DR.
    pub fn build_with_resources(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Fields array
        let fields: Vec<Object> = self.fields.iter().map(|r| Object::Reference(*r)).collect();
        dict.insert("Fields".to_string(), Object::Array(fields));

        // NeedAppearances
        if self.need_appearances {
            dict.insert("NeedAppearances".to_string(), Object::Boolean(true));
        }

        // Signature flags
        if let Some(flags) = self.sig_flags {
            dict.insert("SigFlags".to_string(), Object::Integer(flags as i64));
        }

        // Default appearance
        if let Some(ref da) = self.default_appearance {
            dict.insert("DA".to_string(), Object::text_string(da));
        }

        // Default resources
        let dr = Self::build_default_resources();
        dict.insert("DR".to_string(), Object::Dictionary(dr));

        // Calculation order
        if !self.calc_order.is_empty() {
            let co: Vec<Object> = self
                .calc_order
                .iter()
                .map(|r| Object::Reference(*r))
                .collect();
            dict.insert("CO".to_string(), Object::Array(co));
        }

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acroform_new() {
        let acroform = AcroFormBuilder::new();

        assert!(!acroform.has_fields());
        assert!(acroform.need_appearances);
        assert!(acroform.default_appearance.is_some());
    }

    #[test]
    fn test_acroform_add_fields() {
        let mut acroform = AcroFormBuilder::new();
        acroform.add_field(ObjectRef::new(5, 0));
        acroform.add_field(ObjectRef::new(6, 0));

        assert_eq!(acroform.field_count(), 2);
        assert!(acroform.has_fields());
    }

    #[test]
    fn test_acroform_need_appearances() {
        let acroform = AcroFormBuilder::new().no_need_appearances();
        assert!(!acroform.need_appearances);

        let acroform = acroform.need_appearances();
        assert!(acroform.need_appearances);
    }

    #[test]
    fn test_acroform_default_appearance() {
        let acroform = AcroFormBuilder::new().with_default_appearance("/Cour 10 Tf 0 0 1 rg");

        assert_eq!(acroform.default_appearance, Some("/Cour 10 Tf 0 0 1 rg".to_string()));
    }

    #[test]
    fn test_acroform_sig_flags() {
        let acroform = AcroFormBuilder::new().signatures_exist().append_only();

        assert_eq!(acroform.sig_flags, Some(3)); // bits 1 and 2
    }

    #[test]
    fn test_acroform_build() {
        let mut acroform = AcroFormBuilder::new();
        acroform.add_field(ObjectRef::new(10, 0));

        let dict = acroform.build(None);

        assert!(dict.contains_key("Fields"));
        assert!(dict.contains_key("NeedAppearances"));
        assert!(dict.contains_key("DA"));

        if let Some(Object::Array(fields)) = dict.get("Fields") {
            assert_eq!(fields.len(), 1);
        }
    }

    #[test]
    fn test_acroform_build_with_resources() {
        let mut acroform = AcroFormBuilder::new();
        acroform.add_field(ObjectRef::new(10, 0));

        let dict = acroform.build_with_resources();

        assert!(dict.contains_key("Fields"));
        assert!(dict.contains_key("DR"));

        if let Some(Object::Dictionary(dr)) = dict.get("DR") {
            assert!(dr.contains_key("Font"));

            if let Some(Object::Dictionary(fonts)) = dr.get("Font") {
                assert!(fonts.contains_key("Helv"));
                assert!(fonts.contains_key("ZaDb"));
            }
        }
    }

    #[test]
    fn test_default_resources() {
        let dr = AcroFormBuilder::build_default_resources();

        assert!(dr.contains_key("Font"));

        if let Some(Object::Dictionary(fonts)) = dr.get("Font") {
            // Check all standard form fonts are present
            assert!(fonts.contains_key("Helv"));
            assert!(fonts.contains_key("Cour"));
            assert!(fonts.contains_key("TiRo"));
            assert!(fonts.contains_key("ZaDb"));
        }
    }

    #[test]
    fn test_acroform_calc_order() {
        let acroform = AcroFormBuilder::new()
            .with_calc_order(vec![ObjectRef::new(5, 0), ObjectRef::new(6, 0)]);

        let dict = acroform.build(None);

        assert!(dict.contains_key("CO"));

        if let Some(Object::Array(co)) = dict.get("CO") {
            assert_eq!(co.len(), 2);
        }
    }
}
