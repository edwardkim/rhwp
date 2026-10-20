//! Text field widget for PDF forms.
//!
//! Implements text input fields per ISO 32000-1:2008 Section 12.7.4.3.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::TextFieldWidget;
//! use pdf_oxide::geometry::Rect;
//!
//! let field = TextFieldWidget::new("username", Rect::new(72.0, 700.0, 200.0, 20.0))
//!     .with_value("john_doe")
//!     .with_max_length(50)
//!     .required();
//! ```

use super::{FormFieldEntry, FormFieldWidget, TextAlignment, TextFieldFlags};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Build a PDF JavaScript action dictionary for /AA entries.
fn js_action_dict(script: &str) -> Object {
    let mut d = HashMap::new();
    d.insert("S".to_string(), Object::Name("JavaScript".to_string()));
    d.insert("JS".to_string(), Object::text_string(script));
    Object::Dictionary(d)
}

/// A text input field widget.
///
/// Text fields allow users to enter text. They can be single-line or multiline,
/// and support various options like password masking, maximum length, and comb formatting.
#[derive(Debug, Clone)]
pub struct TextFieldWidget {
    /// Field name (unique identifier)
    name: String,
    /// Bounding rectangle for the widget
    rect: Rect,
    /// Current value (text content)
    value: Option<String>,
    /// Default value (for reset)
    default_value: Option<String>,
    /// Maximum number of characters
    max_length: Option<u32>,
    /// Field flags
    flags: TextFieldFlags,
    /// Text alignment
    alignment: TextAlignment,
    /// Font name for default appearance
    font_name: String,
    /// Font size for default appearance
    font_size: f32,
    /// Text color (RGB, 0.0-1.0)
    text_color: (f32, f32, f32),
    /// Border color (RGB, 0.0-1.0)
    border_color: Option<(f32, f32, f32)>,
    /// Background color (RGB, 0.0-1.0)
    background_color: Option<(f32, f32, f32)>,
    /// Border width in points
    border_width: f32,
    /// Tooltip text
    tooltip: Option<String>,
    /// /AA /K — keystroke JavaScript action
    keystroke: Option<String>,
    /// /AA /F — format JavaScript action
    format: Option<String>,
    /// /AA /V — validate JavaScript action
    validate: Option<String>,
    /// /AA /C — calculate JavaScript action
    calculate: Option<String>,
}

impl TextFieldWidget {
    /// Create a new text field.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique field name (used for form submission)
    /// * `rect` - Position and size of the field
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            value: None,
            default_value: None,
            max_length: None,
            flags: TextFieldFlags::empty(),
            alignment: TextAlignment::Left,
            font_name: "Helv".to_string(),
            font_size: 12.0,
            text_color: (0.0, 0.0, 0.0),             // Black
            border_color: Some((0.0, 0.0, 0.0)),     // Black border
            background_color: Some((1.0, 1.0, 1.0)), // White background
            border_width: 1.0,
            tooltip: None,
            keystroke: None,
            format: None,
            validate: None,
            calculate: None,
        }
    }

    /// Set the current value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the default value (used for form reset).
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set maximum character length.
    pub fn with_max_length(mut self, max_len: u32) -> Self {
        self.max_length = Some(max_len);
        self
    }

    /// Make this a multiline text field.
    pub fn multiline(mut self) -> Self {
        self.flags |= TextFieldFlags::MULTILINE;
        self
    }

    /// Make this a password field (displays asterisks).
    pub fn password(mut self) -> Self {
        self.flags |= TextFieldFlags::PASSWORD;
        self
    }

    /// Make this a comb field (evenly spaced character positions).
    ///
    /// Note: `max_length` must be set for comb fields.
    pub fn comb(mut self) -> Self {
        self.flags |= TextFieldFlags::COMB;
        self
    }

    /// Make the field read-only.
    pub fn read_only(mut self) -> Self {
        self.flags |= TextFieldFlags::READ_ONLY;
        self
    }

    /// Make the field required.
    pub fn required(mut self) -> Self {
        self.flags |= TextFieldFlags::REQUIRED;
        self
    }

    /// Disable spell checking.
    pub fn no_spell_check(mut self) -> Self {
        self.flags |= TextFieldFlags::DO_NOT_SPELL_CHECK;
        self
    }

    /// Disable scrolling.
    pub fn no_scroll(mut self) -> Self {
        self.flags |= TextFieldFlags::DO_NOT_SCROLL;
        self
    }

    /// Set custom flags.
    pub fn with_flags(mut self, flags: TextFieldFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set text alignment.
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the font for text display.
    ///
    /// # Arguments
    ///
    /// * `name` - Font name (e.g., "Helv", "Cour", "TiRo")
    /// * `size` - Font size in points
    pub fn with_font(mut self, name: impl Into<String>, size: f32) -> Self {
        self.font_name = name.into();
        self.font_size = size;
        self
    }

    /// Set text color (RGB, 0.0-1.0).
    pub fn with_text_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.text_color = (r, g, b);
        self
    }

    /// Set border color (RGB, 0.0-1.0).
    pub fn with_border_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.border_color = Some((r, g, b));
        self
    }

    /// Remove border.
    pub fn no_border(mut self) -> Self {
        self.border_color = None;
        self.border_width = 0.0;
        self
    }

    /// Set background color (RGB, 0.0-1.0).
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background_color = Some((r, g, b));
        self
    }

    /// Remove background (transparent).
    pub fn no_background(mut self) -> Self {
        self.background_color = None;
        self
    }

    /// Set border width in points.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set tooltip text.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set a JavaScript keystroke action (`/AA /K`).
    pub fn with_keystroke(mut self, script: impl Into<String>) -> Self {
        self.keystroke = Some(script.into());
        self
    }

    /// Set a JavaScript format action (`/AA /F`).
    pub fn with_format(mut self, script: impl Into<String>) -> Self {
        self.format = Some(script.into());
        self
    }

    /// Set a JavaScript validate action (`/AA /V`).
    pub fn with_validate(mut self, script: impl Into<String>) -> Self {
        self.validate = Some(script.into());
        self
    }

    /// Set a JavaScript calculate action (`/AA /C`).
    pub fn with_calculate(mut self, script: impl Into<String>) -> Self {
        self.calculate = Some(script.into());
        self
    }

    /// Build the default appearance string (DA).
    fn build_default_appearance(&self) -> String {
        let (r, g, b) = self.text_color;
        format!("/{} {} Tf {} {} {} rg", self.font_name, self.font_size, r, g, b)
    }

    /// Build to a FormFieldEntry for page integration.
    pub fn build_entry(&self, page_ref: ObjectRef) -> FormFieldEntry {
        FormFieldEntry {
            widget_dict: self.build_widget_dict(page_ref),
            field_dict: self.build_field_dict(),
            name: self.name.clone(),
            rect: self.rect,
            field_type: "Tx".to_string(),
        }
    }
}

impl FormFieldWidget for TextFieldWidget {
    fn field_name(&self) -> &str {
        &self.name
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn field_type(&self) -> &'static str {
        "Tx"
    }

    fn field_flags(&self) -> u32 {
        self.flags.bits()
    }

    fn build_field_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Field type
        dict.insert("FT".to_string(), Object::Name("Tx".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Value
        if let Some(ref value) = self.value {
            dict.insert("V".to_string(), Object::text_string(value));
        }

        // Default value
        if let Some(ref dv) = self.default_value {
            dict.insert("DV".to_string(), Object::text_string(dv));
        }

        // Maximum length
        if let Some(max_len) = self.max_length {
            dict.insert("MaxLen".to_string(), Object::Integer(max_len as i64));
        }

        // Field flags
        if self.flags.bits() != 0 {
            dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Default appearance
        dict.insert("DA".to_string(), Object::String(self.build_default_appearance().into_bytes()));

        // Quadding (text alignment)
        if self.alignment != TextAlignment::Left {
            dict.insert("Q".to_string(), Object::Integer(self.alignment.q_value()));
        }

        // /AA — additional actions (K/F/V/C JS hooks)
        let mut aa: HashMap<String, Object> = HashMap::new();
        if let Some(ref s) = self.keystroke {
            aa.insert("K".to_string(), js_action_dict(s));
        }
        if let Some(ref s) = self.format {
            aa.insert("F".to_string(), js_action_dict(s));
        }
        if let Some(ref s) = self.validate {
            aa.insert("V".to_string(), js_action_dict(s));
        }
        if let Some(ref s) = self.calculate {
            aa.insert("C".to_string(), js_action_dict(s));
        }
        if !aa.is_empty() {
            dict.insert("AA".to_string(), Object::Dictionary(aa));
        }

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
        dict.insert("F".to_string(), Object::Integer(4)); // Print flag

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }

        // Border style
        if self.border_width > 0.0 {
            let mut bs = HashMap::new();
            bs.insert("W".to_string(), Object::Real(self.border_width as f64));
            bs.insert("S".to_string(), Object::Name("S".to_string())); // Solid
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
    fn test_text_field_new() {
        let field = TextFieldWidget::new("username", Rect::new(72.0, 700.0, 200.0, 20.0));

        assert_eq!(field.name, "username");
        assert_eq!(field.rect.x, 72.0);
        assert_eq!(field.flags, TextFieldFlags::empty());
    }

    #[test]
    fn test_text_field_with_value() {
        let field = TextFieldWidget::new("name", Rect::new(72.0, 700.0, 200.0, 20.0))
            .with_value("John Doe");

        assert_eq!(field.value, Some("John Doe".to_string()));
    }

    #[test]
    fn test_text_field_multiline() {
        let field =
            TextFieldWidget::new("comments", Rect::new(72.0, 600.0, 300.0, 100.0)).multiline();

        assert!(field.flags.contains(TextFieldFlags::MULTILINE));
    }

    #[test]
    fn test_text_field_password() {
        let field =
            TextFieldWidget::new("password", Rect::new(72.0, 700.0, 200.0, 20.0)).password();

        assert!(field.flags.contains(TextFieldFlags::PASSWORD));
    }

    #[test]
    fn test_text_field_required() {
        let field = TextFieldWidget::new("email", Rect::new(72.0, 700.0, 200.0, 20.0)).required();

        assert!(field.flags.contains(TextFieldFlags::REQUIRED));
    }

    #[test]
    fn test_text_field_comb() {
        let field = TextFieldWidget::new("ssn", Rect::new(72.0, 700.0, 200.0, 20.0))
            .with_max_length(9)
            .comb();

        assert!(field.flags.contains(TextFieldFlags::COMB));
        assert_eq!(field.max_length, Some(9));
    }

    #[test]
    fn test_text_field_default_appearance() {
        let field = TextFieldWidget::new("test", Rect::new(72.0, 700.0, 200.0, 20.0))
            .with_font("Cour", 10.0)
            .with_text_color(1.0, 0.0, 0.0);

        let da = field.build_default_appearance();
        assert!(da.contains("/Cour"));
        assert!(da.contains("10"));
        assert!(da.contains("1 0 0 rg"));
    }

    #[test]
    fn test_text_field_build_field_dict() {
        let field = TextFieldWidget::new("test_field", Rect::new(72.0, 700.0, 200.0, 20.0))
            .with_value("Hello")
            .with_max_length(100)
            .required()
            .with_alignment(TextAlignment::Center);

        let dict = field.build_field_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Tx".to_string())));
        assert!(dict.contains_key("T"));
        assert!(dict.contains_key("V"));
        assert!(dict.contains_key("MaxLen"));
        assert!(dict.contains_key("Ff"));
        assert!(dict.contains_key("DA"));
        assert_eq!(dict.get("Q"), Some(&Object::Integer(1))); // Center
    }

    #[test]
    fn test_text_field_build_widget_dict() {
        let field = TextFieldWidget::new("test", Rect::new(72.0, 700.0, 200.0, 20.0))
            .with_tooltip("Enter your name")
            .with_border_color(0.0, 0.0, 1.0)
            .with_background_color(0.9, 0.9, 1.0);

        let page_ref = ObjectRef::new(10, 0);
        let dict = field.build_widget_dict(page_ref);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Widget".to_string())));
        assert!(dict.contains_key("Rect"));
        assert!(dict.contains_key("P"));
        assert!(dict.contains_key("TU")); // Tooltip
        assert!(dict.contains_key("MK")); // Appearance characteristics
    }

    #[test]
    fn test_text_field_no_border() {
        let field = TextFieldWidget::new("test", Rect::new(72.0, 700.0, 200.0, 20.0)).no_border();

        assert!(field.border_color.is_none());
        assert_eq!(field.border_width, 0.0);
    }

    #[test]
    fn test_text_field_trait_impl() {
        let field = TextFieldWidget::new("name", Rect::new(72.0, 700.0, 200.0, 20.0)).required();

        assert_eq!(field.field_name(), "name");
        assert_eq!(field.field_type(), "Tx");
        assert!(field.field_flags() & TextFieldFlags::REQUIRED.bits() != 0);
        assert!(field.needs_appearance());
    }
}
