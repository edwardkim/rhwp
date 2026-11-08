//! Push button widget for PDF forms.
//!
//! Implements push button fields per ISO 32000-1:2008 Section 12.7.4.2.
//!
//! Push buttons trigger actions when clicked but don't retain a value.
//! Common uses include submit and reset buttons.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::{PushButtonWidget, FormAction};
//! use pdf_oxide::geometry::Rect;
//!
//! let submit = PushButtonWidget::new("submit", Rect::new(72.0, 100.0, 80.0, 25.0))
//!     .with_caption("Submit")
//!     .with_action(FormAction::SubmitForm {
//!         url: "https://example.com/submit".to_string(),
//!         flags: Default::default(),
//!     });
//!
//! let reset = PushButtonWidget::new("reset", Rect::new(160.0, 100.0, 80.0, 25.0))
//!     .with_caption("Reset")
//!     .with_action(FormAction::ResetForm);
//! ```

use super::{ButtonFieldFlags, FormFieldEntry, FormFieldWidget};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A push button field widget.
///
/// Push buttons perform actions when clicked. They don't have a value
/// like other form fields. Common uses are submit and reset buttons.
#[derive(Debug, Clone)]
pub struct PushButtonWidget {
    /// Field name
    name: String,
    /// Bounding rectangle
    rect: Rect,
    /// Button caption
    caption: String,
    /// Action to perform when clicked
    action: Option<FormAction>,
    /// Field flags
    flags: ButtonFieldFlags,
    /// Font name
    font_name: String,
    /// Font size
    font_size: f32,
    /// Text color (RGB)
    text_color: (f32, f32, f32),
    /// Border color (RGB)
    border_color: Option<(f32, f32, f32)>,
    /// Background color (RGB)
    background_color: Option<(f32, f32, f32)>,
    /// Border width
    border_width: f32,
    /// Tooltip
    tooltip: Option<String>,
}

/// Actions that can be triggered by a push button.
#[derive(Debug, Clone)]
pub enum FormAction {
    /// Submit form data to a URL.
    SubmitForm {
        /// URL to submit to
        url: String,
        /// Submit flags
        flags: SubmitFormFlags,
    },
    /// Reset form fields to their default values.
    ResetForm,
    /// Execute JavaScript.
    JavaScript {
        /// JavaScript code to execute
        script: String,
    },
    /// Navigate to a URI.
    Uri {
        /// URI to navigate to
        uri: String,
    },
    /// Go to a named destination.
    GoToNamed {
        /// Destination name
        name: String,
    },
}

/// Flags for form submission.
///
/// Per PDF spec Table 237.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubmitFormFlags {
    /// Include all fields, not just those with values
    pub include_no_value_fields: bool,
    /// Submit as FDF (Forms Data Format) - default is HTML
    pub export_format_fdf: bool,
    /// Get method instead of POST
    pub get_method: bool,
    /// Submit coordinates of mouse click
    pub submit_coordinates: bool,
    /// Submit as XFDF (XML FDF)
    pub xfdf: bool,
    /// Include annotations
    pub include_annotations: bool,
    /// Submit as PDF
    pub submit_pdf: bool,
    /// Canonical format for dates/numbers
    pub canonical_format: bool,
    /// Exclude non-user annotations
    pub excl_non_user_annots: bool,
    /// Exclude F entry
    pub excl_f_key: bool,
    /// Embed form in response
    pub embed_form: bool,
}

impl SubmitFormFlags {
    /// Create flags for HTML form submission (default).
    pub fn html() -> Self {
        Self::default()
    }

    /// Create flags for FDF submission.
    pub fn fdf() -> Self {
        Self {
            export_format_fdf: true,
            ..Default::default()
        }
    }

    /// Create flags for XFDF submission.
    pub fn xfdf() -> Self {
        Self {
            xfdf: true,
            ..Default::default()
        }
    }

    /// Create flags for PDF submission.
    pub fn pdf() -> Self {
        Self {
            submit_pdf: true,
            ..Default::default()
        }
    }

    /// Convert to PDF integer flags value.
    pub fn to_bits(&self) -> i64 {
        let mut bits = 0i64;
        if self.include_no_value_fields {
            bits |= 1 << 1;
        }
        if self.export_format_fdf {
            bits |= 1 << 2;
        }
        if self.get_method {
            bits |= 1 << 3;
        }
        if self.submit_coordinates {
            bits |= 1 << 4;
        }
        if self.xfdf {
            bits |= 1 << 5;
        }
        if self.include_annotations {
            bits |= 1 << 6;
        }
        if self.submit_pdf {
            bits |= 1 << 7;
        }
        if self.canonical_format {
            bits |= 1 << 8;
        }
        if self.excl_non_user_annots {
            bits |= 1 << 9;
        }
        if self.excl_f_key {
            bits |= 1 << 10;
        }
        if self.embed_form {
            bits |= 1 << 13;
        }
        bits
    }
}

impl PushButtonWidget {
    /// Create a new push button.
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            caption: String::new(),
            action: None,
            flags: ButtonFieldFlags::PUSHBUTTON, // PUSHBUTTON flag required
            font_name: "Helv".to_string(),
            font_size: 12.0,
            text_color: (0.0, 0.0, 0.0),
            border_color: Some((0.0, 0.0, 0.0)),
            background_color: Some((0.85, 0.85, 0.85)), // Light gray
            border_width: 1.0,
            tooltip: None,
        }
    }

    /// Create a submit button.
    pub fn submit(name: impl Into<String>, rect: Rect, url: impl Into<String>) -> Self {
        Self::new(name, rect)
            .with_caption("Submit")
            .with_action(FormAction::SubmitForm {
                url: url.into(),
                flags: SubmitFormFlags::html(),
            })
    }

    /// Create a reset button.
    pub fn reset(name: impl Into<String>, rect: Rect) -> Self {
        Self::new(name, rect)
            .with_caption("Reset")
            .with_action(FormAction::ResetForm)
    }

    /// Set the button caption.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    /// Set the action to perform when clicked.
    pub fn with_action(mut self, action: FormAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Make the field read-only (button disabled).
    pub fn read_only(mut self) -> Self {
        self.flags |= ButtonFieldFlags::READ_ONLY;
        self
    }

    /// Set font.
    pub fn with_font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = name.into();
        self.font_size = size;
        self
    }

    /// Set text color.
    pub fn with_text_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.text_color = (r, g, b);
        self
    }

    /// Set border color.
    pub fn with_border_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.border_color = Some((r, g, b));
        self
    }

    /// Set background color.
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background_color = Some((r, g, b));
        self
    }

    /// Set border width.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set tooltip.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Build the action dictionary.
    fn build_action_dict(&self) -> Option<HashMap<String, Object>> {
        self.action.as_ref().map(|action| {
            let mut dict = HashMap::new();

            match action {
                FormAction::SubmitForm { url, flags } => {
                    dict.insert("S".to_string(), Object::Name("SubmitForm".to_string()));
                    dict.insert("F".to_string(), Object::text_string(url));
                    let flag_bits = flags.to_bits();
                    if flag_bits != 0 {
                        dict.insert("Flags".to_string(), Object::Integer(flag_bits));
                    }
                },
                FormAction::ResetForm => {
                    dict.insert("S".to_string(), Object::Name("ResetForm".to_string()));
                },
                FormAction::JavaScript { script } => {
                    dict.insert("S".to_string(), Object::Name("JavaScript".to_string()));
                    dict.insert("JS".to_string(), Object::text_string(script));
                },
                FormAction::Uri { uri } => {
                    dict.insert("S".to_string(), Object::Name("URI".to_string()));
                    dict.insert("URI".to_string(), Object::text_string(uri));
                },
                FormAction::GoToNamed { name } => {
                    dict.insert("S".to_string(), Object::Name("GoToR".to_string()));
                    dict.insert("D".to_string(), Object::Name(name.clone()));
                },
            }

            dict
        })
    }

    /// Build to a FormFieldEntry.
    pub fn build_entry(&self, page_ref: ObjectRef) -> FormFieldEntry {
        FormFieldEntry {
            widget_dict: self.build_widget_dict(page_ref),
            field_dict: self.build_field_dict(),
            name: self.name.clone(),
            rect: self.rect,
            field_type: "Btn".to_string(),
        }
    }
}

impl FormFieldWidget for PushButtonWidget {
    fn field_name(&self) -> &str {
        &self.name
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn field_type(&self) -> &'static str {
        "Btn"
    }

    fn field_flags(&self) -> u32 {
        self.flags.bits()
    }

    fn build_field_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Field type - Button with PUSHBUTTON flag
        dict.insert("FT".to_string(), Object::Name("Btn".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Field flags - must include PUSHBUTTON
        dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));

        dict
    }

    fn build_widget_dict(&self, page_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Type and Subtype
        dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Widget".to_string()));

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

        // Page reference
        dict.insert("P".to_string(), Object::Reference(page_ref));

        // Annotation flags - Print
        dict.insert("F".to_string(), Object::Integer(4));

        // Action
        if let Some(action_dict) = self.build_action_dict() {
            dict.insert("A".to_string(), Object::Dictionary(action_dict));
        }

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }

        // Border style
        if self.border_width > 0.0 {
            let mut bs = HashMap::new();
            bs.insert("W".to_string(), Object::Real(self.border_width as f64));
            bs.insert("S".to_string(), Object::Name("B".to_string())); // Beveled for buttons
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Appearance characteristics (MK)
        let mut mk = HashMap::new();

        if let Some((r, g, b)) = self.border_color {
            mk.insert(
                "BC".to_string(),
                Object::Array(vec![
                    Object::Real(r as f64),
                    Object::Real(g as f64),
                    Object::Real(b as f64),
                ]),
            );
        }

        if let Some((r, g, b)) = self.background_color {
            mk.insert(
                "BG".to_string(),
                Object::Array(vec![
                    Object::Real(r as f64),
                    Object::Real(g as f64),
                    Object::Real(b as f64),
                ]),
            );
        }

        // Caption
        if !self.caption.is_empty() {
            mk.insert("CA".to_string(), Object::text_string(&self.caption));
        }

        if !mk.is_empty() {
            dict.insert("MK".to_string(), Object::Dictionary(mk));
        }

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_button_new() {
        let button = PushButtonWidget::new("submit", Rect::new(72.0, 100.0, 80.0, 25.0));

        assert_eq!(button.name, "submit");
        assert!(button.flags.contains(ButtonFieldFlags::PUSHBUTTON));
    }

    #[test]
    fn test_push_button_submit() {
        let button = PushButtonWidget::submit(
            "submit",
            Rect::new(72.0, 100.0, 80.0, 25.0),
            "https://example.com/submit",
        );

        assert_eq!(button.caption, "Submit");
        assert!(button.action.is_some());
    }

    #[test]
    fn test_push_button_reset() {
        let button = PushButtonWidget::reset("reset", Rect::new(160.0, 100.0, 80.0, 25.0));

        assert_eq!(button.caption, "Reset");
        if let Some(FormAction::ResetForm) = button.action {
            // OK
        } else {
            panic!("Expected ResetForm action");
        }
    }

    #[test]
    fn test_submit_form_flags() {
        let flags = SubmitFormFlags::fdf();
        assert!(flags.export_format_fdf);
        assert!(!flags.get_method);

        let bits = flags.to_bits();
        assert!(bits & (1 << 2) != 0); // FDF flag
    }

    #[test]
    fn test_push_button_with_javascript() {
        let button = PushButtonWidget::new("calc", Rect::new(72.0, 100.0, 80.0, 25.0))
            .with_caption("Calculate")
            .with_action(FormAction::JavaScript {
                script: "app.alert('Hello');".to_string(),
            });

        let action_dict = button.build_action_dict().unwrap();
        assert_eq!(action_dict.get("S"), Some(&Object::Name("JavaScript".to_string())));
    }

    #[test]
    fn test_push_button_build_field_dict() {
        let button = PushButtonWidget::new("btn", Rect::new(72.0, 100.0, 80.0, 25.0));

        let dict = button.build_field_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Btn".to_string())));

        // Check PUSHBUTTON flag is set
        if let Some(Object::Integer(flags)) = dict.get("Ff") {
            assert!(*flags & (ButtonFieldFlags::PUSHBUTTON.bits() as i64) != 0);
        }
    }

    #[test]
    fn test_push_button_build_widget_dict() {
        let button = PushButtonWidget::new("submit", Rect::new(72.0, 100.0, 80.0, 25.0))
            .with_caption("Submit")
            .with_action(FormAction::SubmitForm {
                url: "https://example.com".to_string(),
                flags: SubmitFormFlags::html(),
            })
            .with_tooltip("Click to submit");

        let page_ref = ObjectRef::new(10, 0);
        let dict = button.build_widget_dict(page_ref);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Widget".to_string())));
        assert!(dict.contains_key("A")); // Action
        assert!(dict.contains_key("TU")); // Tooltip
        assert!(dict.contains_key("MK")); // Appearance characteristics with caption
    }

    #[test]
    fn test_push_button_trait_impl() {
        let button = PushButtonWidget::new("test", Rect::new(72.0, 100.0, 80.0, 25.0));

        assert_eq!(button.field_name(), "test");
        assert_eq!(button.field_type(), "Btn");
        assert!(button.field_flags() & ButtonFieldFlags::PUSHBUTTON.bits() != 0);
    }

    #[test]
    fn test_submit_flags_defaults() {
        let html = SubmitFormFlags::html();
        assert_eq!(html.to_bits(), 0);

        let fdf = SubmitFormFlags::fdf();
        assert!(fdf.to_bits() & (1 << 2) != 0);

        let xfdf = SubmitFormFlags::xfdf();
        assert!(xfdf.to_bits() & (1 << 5) != 0);

        let pdf = SubmitFormFlags::pdf();
        assert!(pdf.to_bits() & (1 << 7) != 0);
    }
}
