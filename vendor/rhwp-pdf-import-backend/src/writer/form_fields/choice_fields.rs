//! Choice field widgets for PDF forms.
//!
//! Implements choice fields per ISO 32000-1:2008 Section 12.7.4.4:
//! - Combo boxes (dropdown lists)
//! - List boxes (scrollable lists)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::{ComboBoxWidget, ListBoxWidget};
//! use pdf_oxide::geometry::Rect;
//!
//! // Dropdown
//! let country = ComboBoxWidget::new("country", Rect::new(72.0, 700.0, 150.0, 20.0))
//!     .with_options(vec!["USA", "Canada", "UK"])
//!     .with_value("USA");
//!
//! // List box
//! let interests = ListBoxWidget::new("interests", Rect::new(72.0, 600.0, 150.0, 80.0))
//!     .with_options(vec!["Sports", "Music", "Art", "Technology"])
//!     .multi_select();
//! ```

use super::{ChoiceFieldFlags, FormFieldEntry, FormFieldWidget};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

fn js_action_dict(script: &str) -> Object {
    let mut d = HashMap::new();
    d.insert("S".to_string(), Object::Name("JavaScript".to_string()));
    d.insert("JS".to_string(), Object::text_string(script));
    Object::Dictionary(d)
}

/// A combo box (dropdown) field widget.
///
/// Combo boxes present a dropdown list of options. They can optionally
/// allow users to enter custom text (editable combo boxes).
#[derive(Debug, Clone)]
pub struct ComboBoxWidget {
    /// Field name
    name: String,
    /// Bounding rectangle
    rect: Rect,
    /// Available options
    options: Vec<ChoiceOption>,
    /// Current value
    value: Option<String>,
    /// Default value
    default_value: Option<String>,
    /// Field flags
    flags: ChoiceFieldFlags,
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
    /// /AA /K — keystroke JS (editable combo boxes)
    keystroke: Option<String>,
    /// /AA /V — validate JS
    validate: Option<String>,
}

/// A list box field widget.
///
/// List boxes display a scrollable list of options. They can allow
/// single or multiple selections.
#[derive(Debug, Clone)]
pub struct ListBoxWidget {
    /// Field name
    name: String,
    /// Bounding rectangle
    rect: Rect,
    /// Available options
    options: Vec<ChoiceOption>,
    /// Current value(s)
    values: Vec<String>,
    /// Default value(s)
    default_values: Vec<String>,
    /// Field flags
    flags: ChoiceFieldFlags,
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
    /// Top visible index
    top_index: Option<u32>,
    /// /AA /V — validate JS
    validate: Option<String>,
}

/// A choice option with display text and export value.
#[derive(Debug, Clone)]
pub struct ChoiceOption {
    /// Display text shown to user
    pub display: String,
    /// Export value (may differ from display)
    pub export: String,
}

impl ChoiceOption {
    /// Create a new option where display and export are the same.
    pub fn new(value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            display: v.clone(),
            export: v,
        }
    }

    /// Create a new option with different display and export values.
    pub fn new_with_export(display: impl Into<String>, export: impl Into<String>) -> Self {
        Self {
            display: display.into(),
            export: export.into(),
        }
    }
}

impl ComboBoxWidget {
    /// Create a new combo box.
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            options: Vec::new(),
            value: None,
            default_value: None,
            flags: ChoiceFieldFlags::COMBO, // COMBO flag required for dropdowns
            font_name: "Helv".to_string(),
            font_size: 12.0,
            text_color: (0.0, 0.0, 0.0),
            border_color: Some((0.0, 0.0, 0.0)),
            background_color: Some((1.0, 1.0, 1.0)),
            border_width: 1.0,
            tooltip: None,
            keystroke: None,
            validate: None,
        }
    }

    /// Add options from strings (display = export).
    pub fn with_options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|s| ChoiceOption::new(s)).collect();
        self
    }

    /// Add options with display/export pairs.
    pub fn with_choice_options(mut self, options: Vec<ChoiceOption>) -> Self {
        self.options = options;
        self
    }

    /// Set the current value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the default value.
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Make the combo box editable (user can type custom value).
    pub fn editable(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::EDIT;
        self
    }

    /// Sort options alphabetically.
    pub fn sorted(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::SORT;
        self
    }

    /// Make the field read-only.
    pub fn read_only(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::READ_ONLY;
        self
    }

    /// Make the field required.
    pub fn required(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::REQUIRED;
        self
    }

    /// Commit value when selection changes.
    pub fn commit_on_change(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::COMMIT_ON_SEL_CHANGE;
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

    /// Set tooltip.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set a JavaScript keystroke action (`/AA /K`).
    pub fn with_keystroke(mut self, script: impl Into<String>) -> Self {
        self.keystroke = Some(script.into());
        self
    }

    /// Set a JavaScript validate action (`/AA /V`).
    pub fn with_validate(mut self, script: impl Into<String>) -> Self {
        self.validate = Some(script.into());
        self
    }

    /// Build default appearance string.
    fn build_default_appearance(&self) -> String {
        let (r, g, b) = self.text_color;
        format!("/{} {} Tf {} {} {} rg", self.font_name, self.font_size, r, g, b)
    }

    /// Build to a FormFieldEntry.
    pub fn build_entry(&self, page_ref: ObjectRef) -> FormFieldEntry {
        FormFieldEntry {
            widget_dict: self.build_widget_dict(page_ref),
            field_dict: self.build_field_dict(),
            name: self.name.clone(),
            rect: self.rect,
            field_type: "Ch".to_string(),
        }
    }
}

impl FormFieldWidget for ComboBoxWidget {
    fn field_name(&self) -> &str {
        &self.name
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn field_type(&self) -> &'static str {
        "Ch"
    }

    fn field_flags(&self) -> u32 {
        self.flags.bits()
    }

    fn build_field_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Field type
        dict.insert("FT".to_string(), Object::Name("Ch".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Options array
        let opt_array: Vec<Object> = self
            .options
            .iter()
            .map(|opt| {
                if opt.display == opt.export {
                    Object::text_string(&opt.display)
                } else {
                    Object::Array(vec![
                        Object::text_string(&opt.export),
                        Object::text_string(&opt.display),
                    ])
                }
            })
            .collect();
        dict.insert("Opt".to_string(), Object::Array(opt_array));

        // Value
        if let Some(ref value) = self.value {
            dict.insert("V".to_string(), Object::text_string(value));
        }

        // Default value
        if let Some(ref dv) = self.default_value {
            dict.insert("DV".to_string(), Object::text_string(dv));
        }

        // Field flags
        dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));

        // Default appearance
        dict.insert("DA".to_string(), Object::String(self.build_default_appearance().into_bytes()));

        // /AA — additional actions (K/V)
        let mut aa: HashMap<String, Object> = HashMap::new();
        if let Some(ref s) = self.keystroke {
            aa.insert("K".to_string(), js_action_dict(s));
        }
        if let Some(ref s) = self.validate {
            aa.insert("V".to_string(), js_action_dict(s));
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
        dict.insert("F".to_string(), Object::Integer(4));

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }

        // Border style
        if self.border_width > 0.0 {
            let mut bs = HashMap::new();
            bs.insert("W".to_string(), Object::Real(self.border_width as f64));
            bs.insert("S".to_string(), Object::Name("S".to_string()));
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Appearance characteristics
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

impl ListBoxWidget {
    /// Create a new list box.
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            name: name.into(),
            rect,
            options: Vec::new(),
            values: Vec::new(),
            default_values: Vec::new(),
            flags: ChoiceFieldFlags::empty(), // No COMBO flag = list box
            font_name: "Helv".to_string(),
            font_size: 12.0,
            text_color: (0.0, 0.0, 0.0),
            border_color: Some((0.0, 0.0, 0.0)),
            background_color: Some((1.0, 1.0, 1.0)),
            border_width: 1.0,
            tooltip: None,
            top_index: None,
            validate: None,
        }
    }

    /// Add options from strings.
    pub fn with_options(mut self, options: Vec<impl Into<String>>) -> Self {
        self.options = options.into_iter().map(|s| ChoiceOption::new(s)).collect();
        self
    }

    /// Add options with display/export pairs.
    pub fn with_choice_options(mut self, options: Vec<ChoiceOption>) -> Self {
        self.options = options;
        self
    }

    /// Set the selected value (single selection).
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.values = vec![value.into()];
        self
    }

    /// Set selected values (multiple selection).
    pub fn with_values(mut self, values: Vec<impl Into<String>>) -> Self {
        self.values = values.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Set the default value.
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_values = vec![value.into()];
        self
    }

    /// Enable multiple selection.
    pub fn multi_select(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::MULTI_SELECT;
        self
    }

    /// Sort options alphabetically.
    pub fn sorted(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::SORT;
        self
    }

    /// Make the field read-only.
    pub fn read_only(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::READ_ONLY;
        self
    }

    /// Make the field required.
    pub fn required(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::REQUIRED;
        self
    }

    /// Commit value when selection changes.
    pub fn commit_on_change(mut self) -> Self {
        self.flags |= ChoiceFieldFlags::COMMIT_ON_SEL_CHANGE;
        self
    }

    /// Set the top visible item index.
    pub fn with_top_index(mut self, index: u32) -> Self {
        self.top_index = Some(index);
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

    /// Set tooltip.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set a JavaScript validate action (`/AA /V`).
    pub fn with_validate(mut self, script: impl Into<String>) -> Self {
        self.validate = Some(script.into());
        self
    }

    /// Build default appearance string.
    fn build_default_appearance(&self) -> String {
        let (r, g, b) = self.text_color;
        format!("/{} {} Tf {} {} {} rg", self.font_name, self.font_size, r, g, b)
    }

    /// Build to a FormFieldEntry.
    pub fn build_entry(&self, page_ref: ObjectRef) -> FormFieldEntry {
        FormFieldEntry {
            widget_dict: self.build_widget_dict(page_ref),
            field_dict: self.build_field_dict(),
            name: self.name.clone(),
            rect: self.rect,
            field_type: "Ch".to_string(),
        }
    }
}

impl FormFieldWidget for ListBoxWidget {
    fn field_name(&self) -> &str {
        &self.name
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn field_type(&self) -> &'static str {
        "Ch"
    }

    fn field_flags(&self) -> u32 {
        self.flags.bits()
    }

    fn build_field_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Field type
        dict.insert("FT".to_string(), Object::Name("Ch".to_string()));

        // Field name
        dict.insert("T".to_string(), Object::text_string(&self.name));

        // Options array
        let opt_array: Vec<Object> = self
            .options
            .iter()
            .map(|opt| {
                if opt.display == opt.export {
                    Object::text_string(&opt.display)
                } else {
                    Object::Array(vec![
                        Object::text_string(&opt.export),
                        Object::text_string(&opt.display),
                    ])
                }
            })
            .collect();
        dict.insert("Opt".to_string(), Object::Array(opt_array));

        // Value(s)
        if !self.values.is_empty() {
            if self.values.len() == 1 {
                dict.insert("V".to_string(), Object::text_string(&self.values[0]));
            } else {
                let v_array: Vec<Object> = self.values.iter().map(Object::text_string).collect();
                dict.insert("V".to_string(), Object::Array(v_array));
            }
        }

        // Default value(s)
        if !self.default_values.is_empty() {
            if self.default_values.len() == 1 {
                dict.insert("DV".to_string(), Object::text_string(&self.default_values[0]));
            } else {
                let dv_array: Vec<Object> = self
                    .default_values
                    .iter()
                    .map(Object::text_string)
                    .collect();
                dict.insert("DV".to_string(), Object::Array(dv_array));
            }
        }

        // Field flags (no COMBO = list box)
        if self.flags.bits() != 0 {
            dict.insert("Ff".to_string(), Object::Integer(self.flags.bits() as i64));
        }

        // Default appearance
        dict.insert("DA".to_string(), Object::String(self.build_default_appearance().into_bytes()));

        // Top index
        if let Some(ti) = self.top_index {
            dict.insert("TI".to_string(), Object::Integer(ti as i64));
        }

        // /AA — additional actions (V)
        if let Some(ref s) = self.validate {
            let mut aa: HashMap<String, Object> = HashMap::new();
            aa.insert("V".to_string(), js_action_dict(s));
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
        dict.insert("F".to_string(), Object::Integer(4));

        // Tooltip
        if let Some(ref tip) = self.tooltip {
            dict.insert("TU".to_string(), Object::text_string(tip));
        }

        // Border style
        if self.border_width > 0.0 {
            let mut bs = HashMap::new();
            bs.insert("W".to_string(), Object::Real(self.border_width as f64));
            bs.insert("S".to_string(), Object::Name("S".to_string()));
            dict.insert("BS".to_string(), Object::Dictionary(bs));
        }

        // Appearance characteristics
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
    fn test_choice_option() {
        let opt = ChoiceOption::new("USA");
        assert_eq!(opt.display, "USA");
        assert_eq!(opt.export, "USA");

        let opt2 = ChoiceOption::new_with_export("United States", "USA");
        assert_eq!(opt2.display, "United States");
        assert_eq!(opt2.export, "USA");
    }

    #[test]
    fn test_combo_box_new() {
        let combo = ComboBoxWidget::new("country", Rect::new(72.0, 700.0, 150.0, 20.0));

        assert_eq!(combo.name, "country");
        assert!(combo.flags.contains(ChoiceFieldFlags::COMBO));
    }

    #[test]
    fn test_combo_box_with_options() {
        let combo = ComboBoxWidget::new("country", Rect::new(72.0, 700.0, 150.0, 20.0))
            .with_options(vec!["USA", "Canada", "UK"])
            .with_value("USA");

        assert_eq!(combo.options.len(), 3);
        assert_eq!(combo.value, Some("USA".to_string()));
    }

    #[test]
    fn test_combo_box_editable() {
        let combo = ComboBoxWidget::new("country", Rect::new(72.0, 700.0, 150.0, 20.0)).editable();

        assert!(combo.flags.contains(ChoiceFieldFlags::EDIT));
    }

    #[test]
    fn test_combo_box_build_field_dict() {
        let combo = ComboBoxWidget::new("country", Rect::new(72.0, 700.0, 150.0, 20.0))
            .with_options(vec!["USA", "Canada"])
            .with_value("USA")
            .required();

        let dict = combo.build_field_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Ch".to_string())));
        assert!(dict.contains_key("Opt"));
        assert!(dict.contains_key("V"));
        assert!(dict.contains_key("Ff"));
        assert!(dict.contains_key("DA"));
    }

    #[test]
    fn test_list_box_new() {
        let list = ListBoxWidget::new("interests", Rect::new(72.0, 600.0, 150.0, 80.0));

        assert_eq!(list.name, "interests");
        assert!(!list.flags.contains(ChoiceFieldFlags::COMBO));
    }

    #[test]
    fn test_list_box_multi_select() {
        let list = ListBoxWidget::new("interests", Rect::new(72.0, 600.0, 150.0, 80.0))
            .with_options(vec!["Sports", "Music", "Art"])
            .multi_select()
            .with_values(vec!["Sports", "Music"]);

        assert!(list.flags.contains(ChoiceFieldFlags::MULTI_SELECT));
        assert_eq!(list.values.len(), 2);
    }

    #[test]
    fn test_list_box_build_field_dict() {
        let list = ListBoxWidget::new("items", Rect::new(72.0, 600.0, 150.0, 80.0))
            .with_options(vec!["A", "B", "C"])
            .with_value("A")
            .with_top_index(0);

        let dict = list.build_field_dict();

        assert_eq!(dict.get("FT"), Some(&Object::Name("Ch".to_string())));
        assert!(dict.contains_key("Opt"));
        assert!(dict.contains_key("V"));
        assert!(dict.contains_key("TI"));
    }

    #[test]
    fn test_list_box_multi_value_dict() {
        let list = ListBoxWidget::new("items", Rect::new(72.0, 600.0, 150.0, 80.0))
            .with_options(vec!["A", "B", "C"])
            .multi_select()
            .with_values(vec!["A", "B"]);

        let dict = list.build_field_dict();

        // Multiple values should be an array
        if let Some(Object::Array(arr)) = dict.get("V") {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected V to be an array for multi-select");
        }
    }

    #[test]
    fn test_combo_widget_dict() {
        let combo = ComboBoxWidget::new("test", Rect::new(72.0, 700.0, 150.0, 20.0))
            .with_tooltip("Select a value")
            .with_border_color(0.0, 0.0, 1.0);

        let page_ref = ObjectRef::new(10, 0);
        let dict = combo.build_widget_dict(page_ref);

        assert_eq!(dict.get("Type"), Some(&Object::Name("Annot".to_string())));
        assert_eq!(dict.get("Subtype"), Some(&Object::Name("Widget".to_string())));
        assert!(dict.contains_key("TU"));
        assert!(dict.contains_key("MK"));
    }

    #[test]
    fn test_choice_option_pairs() {
        let combo = ComboBoxWidget::new("status", Rect::new(72.0, 700.0, 150.0, 20.0))
            .with_choice_options(vec![
                ChoiceOption::new_with_export("Active", "1"),
                ChoiceOption::new_with_export("Inactive", "0"),
            ]);

        let dict = combo.build_field_dict();

        if let Some(Object::Array(opts)) = dict.get("Opt") {
            // Each option should be an array [export, display]
            if let Object::Array(first) = &opts[0] {
                assert_eq!(first.len(), 2);
            } else {
                panic!("Expected option to be array for export/display pair");
            }
        }
    }
}
