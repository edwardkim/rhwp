//! Radio button widget for PDF forms.
//!
//! Implements radio button fields per ISO 32000-1:2008 Section 12.7.4.2.
//!
//! Radio buttons work in groups where only one button can be selected at a time.
//! The group has a parent field that holds the value, and child widgets for each option.
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::RadioButtonGroup;
//! use pdf_oxide::geometry::Rect;
//!
//! let group = RadioButtonGroup::new("payment")
//!     .add_button("credit", Rect::new(72.0, 700.0, 15.0, 15.0), "Credit Card")
//!     .add_button("paypal", Rect::new(72.0, 680.0, 15.0, 15.0), "PayPal")
//!     .selected("credit");
//! ```

use super::{ButtonFieldFlags, FormFieldEntry};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// A group of radio buttons.
///
/// Radio button groups contain multiple buttons where only one can be selected.
/// The group maintains the selected value, and each button has its own export value.
#[derive(Debug, Clone)]
pub struct RadioButtonGroup {
    /// Group field name
    name: String,
    /// Individual radio buttons in the group
    buttons: Vec<RadioButtonWidget>,
    /// Currently selected button's export value (or None/Off)
    selected: Option<String>,
    /// Field flags for the group
    flags: ButtonFieldFlags,
    /// Tooltip for the group
    tooltip: Option<String>,
}

/// A single radio button within a group.
#[derive(Debug, Clone)]
pub struct RadioButtonWidget {
    /// Export value for this button (must be unique within group)
    export_value: String,
    /// Bounding rectangle
    rect: Rect,
    /// Display label (for accessibility/tooltip)
    label: String,
    /// Border color (RGB, 0.0-1.0)
    border_color: Option<(f32, f32, f32)>,
    /// Background color (RGB, 0.0-1.0)
    background_color: Option<(f32, f32, f32)>,
    /// Selected indicator color (RGB, 0.0-1.0)
    indicator_color: (f32, f32, f32),
    /// Border width in points
    border_width: f32,
}

impl RadioButtonGroup {
    /// Create a new radio button group.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique field name for the group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            buttons: Vec::new(),
            selected: None,
            flags: ButtonFieldFlags::RADIO, // Radio flag is required
            tooltip: None,
        }
    }

    /// Add a radio button to the group.
    ///
    /// # Arguments
    ///
    /// * `export_value` - Value submitted when this button is selected
    /// * `rect` - Position and size of the button
    /// * `label` - Display label for accessibility
    pub fn add_button(
        mut self,
        export_value: impl Into<String>,
        rect: Rect,
        label: impl Into<String>,
    ) -> Self {
        self.buttons.push(RadioButtonWidget {
            export_value: export_value.into(),
            rect,
            label: label.into(),
            border_color: Some((0.0, 0.0, 0.0)),
            background_color: Some((1.0, 1.0, 1.0)),
            indicator_color: (0.0, 0.0, 0.0),
            border_width: 1.0,
        });
        self
    }

    /// Add a configured radio button widget.
    pub fn add_widget(mut self, widget: RadioButtonWidget) -> Self {
        self.buttons.push(widget);
        self
    }

    /// Set which button is initially selected by its export value.
    pub fn selected(mut self, export_value: impl Into<String>) -> Self {
        self.selected = Some(export_value.into());
        self
    }

    /// Clear selection (no button selected).
    pub fn none_selected(mut self) -> Self {
        self.selected = None;
        self
    }

    /// Enable "no toggle to off" - at least one button must always be selected.
    pub fn no_toggle_to_off(mut self) -> Self {
        self.flags |= ButtonFieldFlags::NO_TOGGLE_TO_OFF;
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

    /// Set tooltip for the group.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Get the group name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the buttons in this group.
    pub fn buttons(&self) -> &[RadioButtonWidget] {
        &self.buttons
    }

    /// Get the selected value.
    pub fn selected_value(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Build the parent field dictionary.
    pub fn build_parent_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Field type - Button with RADIO flag
        dict.insert("FT".to_string(), Object::Name("Btn".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Value - selected button's export value or /Off
        let value = self.selected.clone().unwrap_or_else(|| "Off".to_string());
        dict.insert("V".to_string(), Object::Name(value.clone()));

        // Default value
        dict.insert("DV".to_string(), Object::Name(value));

        // Field flags - must include RADIO
        dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }

        dict
    }

    /// Build entries for all radio buttons.
    ///
    /// Returns a list of FormFieldEntry for each button, plus the parent field dict.
    pub fn build_entries(
        &self,
        page_ref: ObjectRef,
    ) -> (HashMap<String, Object>, Vec<FormFieldEntry>) {
        let parent_dict = self.build_parent_dict();

        let entries: Vec<FormFieldEntry> = self
            .buttons
            .iter()
            .map(|btn| {
                let is_selected = self.selected.as_ref() == Some(&btn.export_value);
                FormFieldEntry {
                    widget_dict: btn.build_widget_dict(page_ref, is_selected),
                    field_dict: HashMap::new(), // Child widgets don't have separate field dicts
                    name: format!("{}_{}", self.name, btn.export_value),
                    rect: btn.rect,
                    field_type: "Btn".to_string(),
                }
            })
            .collect();

        (parent_dict, entries)
    }
}

impl RadioButtonWidget {
    /// Create a new radio button widget.
    pub fn new(export_value: impl Into<String>, rect: Rect, label: impl Into<String>) -> Self {
        Self {
            export_value: export_value.into(),
            rect,
            label: label.into(),
            border_color: Some((0.0, 0.0, 0.0)),
            background_color: Some((1.0, 1.0, 1.0)),
            indicator_color: (0.0, 0.0, 0.0),
            border_width: 1.0,
        }
    }

    /// Get the export value.
    pub fn export_value(&self) -> &str {
        &self.export_value
    }

    /// Get the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the rectangle.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Set border color (RGB, 0.0-1.0).
    pub fn with_border_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.border_color = Some((r, g, b));
        self
    }

    /// Set background color (RGB, 0.0-1.0).
    pub fn with_background_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background_color = Some((r, g, b));
        self
    }

    /// Set indicator color (RGB, 0.0-1.0).
    pub fn with_indicator_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.indicator_color = (r, g, b);
        self
    }

    /// Set border width in points.
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Build the widget annotation dictionary.
    pub fn build_widget_dict(
        &self,
        page_ref: ObjectRef,
        is_selected: bool,
    ) -> HashMap<String, Object> {
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

        // Appearance state
        let as_name = if is_selected {
            self.export_value.clone()
        } else {
            "Off".to_string()
        };
        dict.insert("AS".to_string(), Object::Name(as_name));

        // Border style
        if self.border_width > 0.0 {
            let mut bs = HashMap::new();
            bs.insert("W".to_string(), Object::Real(self.border_width as f64));
            bs.insert("S".to_string(), Object::Name("S".to_string()));
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

        // Caption character for radio (l = bullet in ZapfDingbats)
        mk.insert("CA".to_string(), Object::text_string("l"));

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
    fn test_radio_group_new() {
        let group = RadioButtonGroup::new("payment");

        assert_eq!(group.name(), "payment");
        assert!(group.buttons().is_empty());
        assert!(group.selected_value().is_none());
    }

    #[test]
    fn test_radio_group_add_buttons() {
        let group = RadioButtonGroup::new("payment")
            .add_button("credit", Rect::new(72.0, 700.0, 15.0, 15.0), "Credit Card")
            .add_button("paypal", Rect::new(72.0, 680.0, 15.0, 15.0), "PayPal");

        assert_eq!(group.buttons().len(), 2);
        assert_eq!(group.buttons()[0].export_value(), "credit");
        assert_eq!(group.buttons()[1].export_value(), "paypal");
    }

    #[test]
    fn test_radio_group_selected() {
        let group = RadioButtonGroup::new("payment")
            .add_button("credit", Rect::new(72.0, 700.0, 15.0, 15.0), "Credit Card")
            .add_button("paypal", Rect::new(72.0, 680.0, 15.0, 15.0), "PayPal")
            .selected("credit");

        assert_eq!(group.selected_value(), Some("credit"));
    }

    #[test]
    fn test_radio_group_build_parent_dict() {
        let group = RadioButtonGroup::new("payment")
            .add_button("credit", Rect::new(72.0, 700.0, 15.0, 15.0), "Credit Card")
            .selected("credit")
            .no_toggle_to_off();

        let dict = group.build_parent_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Btn".to_string())));
        assert_eq!(dict.get("V"), Some(&Object::Name("credit".to_string())));

        // Check flags include RADIO and NO_TOGGLE_TO_OFF
        if let Some(Object::Integer(flags)) = dict.get("Ff") {
            assert!(*flags & (ButtonFieldFlags::RADIO.bits() as i64) != 0);
            assert!(*flags & (ButtonFieldFlags::NO_TOGGLE_TO_OFF.bits() as i64) != 0);
        } else {
            panic!("Expected Ff to be an integer");
        }
    }

    #[test]
    fn test_radio_group_build_entries() {
        let group = RadioButtonGroup::new("payment")
            .add_button("credit", Rect::new(72.0, 700.0, 15.0, 15.0), "Credit Card")
            .add_button("paypal", Rect::new(72.0, 680.0, 15.0, 15.0), "PayPal")
            .selected("credit");

        let page_ref = ObjectRef::new(10, 0);
        let (parent, entries) = group.build_entries(page_ref);

        assert!(!parent.is_empty());
        assert_eq!(entries.len(), 2);

        // First button should be selected
        assert_eq!(entries[0].widget_dict.get("AS"), Some(&Object::Name("credit".to_string())));

        // Second button should be off
        assert_eq!(entries[1].widget_dict.get("AS"), Some(&Object::Name("Off".to_string())));
    }

    #[test]
    fn test_radio_button_widget() {
        let button =
            RadioButtonWidget::new("option1", Rect::new(72.0, 700.0, 15.0, 15.0), "Option 1")
                .with_border_color(0.0, 0.0, 1.0)
                .with_background_color(0.9, 0.9, 1.0);

        assert_eq!(button.export_value(), "option1");
        assert_eq!(button.label(), "Option 1");
        assert_eq!(button.border_color, Some((0.0, 0.0, 1.0)));
    }

    #[test]
    fn test_radio_button_widget_dict() {
        let button = RadioButtonWidget::new("yes", Rect::new(72.0, 700.0, 15.0, 15.0), "Yes");
        let page_ref = ObjectRef::new(10, 0);

        let dict = button.build_widget_dict(page_ref, true);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Widget".to_string())));
        assert_eq!(dict.get("AS"), Some(&Object::Name("yes".to_string())));
    }

    #[test]
    fn test_radio_required() {
        let group = RadioButtonGroup::new("required_choice")
            .add_button("a", Rect::new(72.0, 700.0, 15.0, 15.0), "A")
            .required();

        assert!(group.flags.contains(ButtonFieldFlags::REQUIRED));
    }
}
