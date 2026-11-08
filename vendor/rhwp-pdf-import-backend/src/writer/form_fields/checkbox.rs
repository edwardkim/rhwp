//! Checkbox widget for PDF forms.
//!
//! Implements checkbox fields per ISO 32000-1:2008 Section 12.7.4.2.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::CheckboxWidget;
//! use pdf_oxide::geometry::Rect;
//!
//! let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0))
//!     .checked()
//!     .with_export_value("Yes");
//! ```

use super::{ButtonFieldFlags, FormFieldEntry, FormFieldWidget};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A checkbox field widget.
///
/// Checkboxes are button fields that toggle between two states: on and off.
/// The appearance shows a checkmark or similar indicator when checked.
#[derive(Debug, Clone)]
pub struct CheckboxWidget {
    /// Field name (unique identifier)
    name: String,
    /// Bounding rectangle for the widget
    rect: Rect,
    /// Whether the checkbox is checked
    checked: bool,
    /// Export value when checked (default: "Yes")
    export_value: String,
    /// Field flags
    flags: ButtonFieldFlags,
    /// Border color (RGB, 0.0-1.0)
    border_color: Option<(f32, f32, f32)>,
    /// Background color (RGB, 0.0-1.0)
    background_color: Option<(f32, f32, f32)>,
    /// Check mark color (RGB, 0.0-1.0)
    check_color: (f32, f32, f32),
    /// Border width in points
    border_width: f32,
    /// Tooltip text
    tooltip: Option<String>,
}

impl CheckboxWidget {
    /// Create a new checkbox.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique field name
    /// * `rect` - Position and size (typically square, e.g., 15x15)
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            checked: false,
            export_value: "Yes".to_string(),
            flags: ButtonFieldFlags::empty(),
            border_color: Some((0.0, 0.0, 0.0)), // Black border
            background_color: Some((1.0, 1.0, 1.0)), // White background
            check_color: (0.0, 0.0, 0.0),        // Black checkmark
            border_width: 1.0,
            tooltip: None,
        }
    }

    /// Set the checkbox as checked.
    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    /// Set the checkbox as unchecked.
    pub fn unchecked(mut self) -> Self {
        self.checked = false;
        self
    }

    /// Set initial checked state.
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the export value (value submitted when checked).
    pub fn with_export_value(mut self, value: impl Into<String>) -> Self {
        self.export_value = value.into();
        self
    }

    /// Make the field read-only.
    pub fn read_only(mut self) -> Self {
        self.flags |= ButtonFieldFlags::READ_ONLY;
        self
    }

    /// Make the field required.
    pub fn required(mut self) -> Self {
        self.flags |= ButtonFieldFlags::REQUIRED;
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

    /// Set checkmark color (RGB, 0.0-1.0).
    pub fn with_check_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.check_color = (r, g, b);
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

    /// Get the current checked state.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Get the export value.
    pub fn export_value(&self) -> &str {
        &self.export_value
    }

    /// Build to a FormFieldEntry for page integration.
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

impl FormFieldWidget for CheckboxWidget {
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

        // Field type - Button (no Radio or Pushbutton flags = checkbox)
        dict.insert("FT".to_string(), Object::Name("Btn".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Value - the export value name if checked, /Off if not
        let value = if self.checked {
            self.export_value.clone()
        } else {
            "Off".to_string()
        };
        dict.insert("V".to_string(), Object::Name(value.clone()));

        // Default value (same as value for checkboxes)
        dict.insert("DV".to_string(), Object::Name(value));

        // Field flags (no RADIO or PUSHBUTTON = checkbox)
        if self.flags.bits() != 0 {
            dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));
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

        // Appearance state - which appearance to show
        let as_name = if self.checked {
            self.export_value.clone()
        } else {
            "Off".to_string()
        };
        dict.insert("AS".to_string(), Object::Name(as_name));

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

        // Caption character for checkbox (4 = checkmark in ZapfDingbats)
        mk.insert("CA".to_string(), Object::text_string("4"));

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
    fn test_checkbox_new() {
        let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0));

        assert_eq!(checkbox.name, "agree");
        assert!(!checkbox.checked);
        assert_eq!(checkbox.export_value, "Yes");
    }

    #[test]
    fn test_checkbox_checked() {
        let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0)).checked();

        assert!(checkbox.is_checked());
    }

    #[test]
    fn test_checkbox_export_value() {
        let checkbox = CheckboxWidget::new("opt_in", Rect::new(72.0, 700.0, 15.0, 15.0))
            .with_export_value("Accepted")
            .checked();

        assert_eq!(checkbox.export_value(), "Accepted");
    }

    #[test]
    fn test_checkbox_required() {
        let checkbox = CheckboxWidget::new("terms", Rect::new(72.0, 700.0, 15.0, 15.0)).required();

        assert!(checkbox.flags.contains(ButtonFieldFlags::REQUIRED));
    }

    #[test]
    fn test_checkbox_build_field_dict_checked() {
        let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0))
            .with_export_value("Yes")
            .checked();

        let dict = checkbox.build_field_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Btn".to_string())));
        assert_eq!(dict.get("V"), Some(&Object::Name("Yes".to_string())));
    }

    #[test]
    fn test_checkbox_build_field_dict_unchecked() {
        let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0));

        let dict = checkbox.build_field_dict();

        assert_eq!(dict.get("V"), Some(&Object::Name("Off".to_string())));
    }

    #[test]
    fn test_checkbox_build_widget_dict() {
        let checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 700.0, 15.0, 15.0))
            .checked()
            .with_tooltip("Check to agree");

        let page_ref = ObjectRef::new(10, 0);
        let dict = checkbox.build_widget_dict(page_ref);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Widget".to_string())));
        assert_eq!(dict.get("AS"), Some(&Object::Name("Yes".to_string())));
        assert!(dict.contains_key("TU")); // Tooltip
        assert!(dict.contains_key("MK")); // Appearance characteristics
    }

    #[test]
    fn test_checkbox_trait_impl() {
        let checkbox = CheckboxWidget::new("test", Rect::new(72.0, 700.0, 15.0, 15.0));

        assert_eq!(checkbox.field_name(), "test");
        assert_eq!(checkbox.field_type(), "Btn");
        assert!(checkbox.needs_appearance());
    }

    #[test]
    fn test_checkbox_colors() {
        let checkbox = CheckboxWidget::new("test", Rect::new(72.0, 700.0, 15.0, 15.0))
            .with_border_color(0.0, 0.0, 1.0)
            .with_background_color(0.9, 0.9, 1.0)
            .with_check_color(0.0, 0.5, 0.0);

        assert_eq!(checkbox.border_color, Some((0.0, 0.0, 1.0)));
        assert_eq!(checkbox.background_color, Some((0.9, 0.9, 1.0)));
        assert_eq!(checkbox.check_color, (0.0, 0.5, 0.0));
    }
}
