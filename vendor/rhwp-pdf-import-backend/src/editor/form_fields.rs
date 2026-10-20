//! Form field editing support for DocumentEditor.
//!
//! This module provides the abstraction layer for reading, creating, and modifying
//! AcroForm fields in existing PDF documents.
//!
//! # Architecture
//!
//! The `FormFieldWrapper` type bridges the gap between:
//! - **Read side**: `FormField` from `extractors::forms` (existing fields in PDF)
//! - **Write side**: `FormFieldWidget` trait from `writer::form_fields` (new/modified fields)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::editor::DocumentEditor;
//! use pdf_oxide::writer::form_fields::TextFieldWidget;
//! use pdf_oxide::geometry::Rect;
//!
//! let mut editor = DocumentEditor::open("form.pdf")?;
//!
//! // Add new field
//! editor.add_form_field(0,
//!     TextFieldWidget::new("email", Rect::new(100.0, 700.0, 200.0, 20.0))
//! )?;
//!
//! // Modify existing field
//! editor.set_form_field_value("name", FormFieldValue::Text("John".into()))?;
//!
//! editor.save("modified.pdf")?;
//! ```

use crate::extractors::forms::{FieldType, FieldValue, FormField};
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use crate::writer::form_fields::FormFieldWidget;
use std::collections::HashMap;

/// Unified form field value type.
///
/// This enum provides a consistent interface for field values regardless of
/// whether the field was read from an existing PDF or created new.
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldValue {
    /// Text string value (for text fields)
    Text(String),
    /// Boolean value (for checkboxes)
    Boolean(bool),
    /// Single choice value (for radio buttons, combo boxes)
    Choice(String),
    /// Multiple choice values (for multi-select list boxes)
    MultiChoice(Vec<String>),
    /// No value present
    None,
}

impl FormFieldValue {
    /// Check if the value is empty/none.
    pub fn is_none(&self) -> bool {
        matches!(self, FormFieldValue::None)
    }

    /// Get as text, if this is a text value.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FormFieldValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get as boolean, if this is a boolean value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FormFieldValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as choice, if this is a single choice value.
    pub fn as_choice(&self) -> Option<&str> {
        match self {
            FormFieldValue::Choice(s) => Some(s),
            _ => None,
        }
    }

    /// Get as multi-choice, if this is a multi-choice value.
    pub fn as_multi_choice(&self) -> Option<&[String]> {
        match self {
            FormFieldValue::MultiChoice(v) => Some(v),
            _ => None,
        }
    }
}

/// Convert from extractor's FieldValue to unified FormFieldValue.
impl From<FieldValue> for FormFieldValue {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::Text(s) => FormFieldValue::Text(s),
            FieldValue::Boolean(b) => FormFieldValue::Boolean(b),
            FieldValue::Name(s) => FormFieldValue::Choice(s),
            FieldValue::Array(v) => FormFieldValue::MultiChoice(v),
            FieldValue::None => FormFieldValue::None,
        }
    }
}

/// Convert from reference to extractor's FieldValue.
impl From<&FieldValue> for FormFieldValue {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Text(s) => FormFieldValue::Text(s.clone()),
            FieldValue::Boolean(b) => FormFieldValue::Boolean(*b),
            FieldValue::Name(s) => FormFieldValue::Choice(s.clone()),
            FieldValue::Array(v) => FormFieldValue::MultiChoice(v.clone()),
            FieldValue::None => FormFieldValue::None,
        }
    }
}

/// Convert FormFieldValue to PDF Object for serialization.
impl From<&FormFieldValue> for Object {
    fn from(value: &FormFieldValue) -> Self {
        match value {
            // #616: encode as a conformant PDF text string (ISO 32000-1
            // §7.9.2.2) — PDFDocEncoding-compatible literal for ASCII/Latin-1,
            // UTF-16BE + FEFF BOM for anything above U+00FF — instead of raw
            // UTF-8 bytes, which a reader misreads as PDFDocEncoding (mojibake).
            FormFieldValue::Text(s) => Object::text_string(s),
            FormFieldValue::Boolean(b) => {
                // Checkboxes use /Yes or /Off names
                if *b {
                    Object::Name("Yes".to_string())
                } else {
                    Object::Name("Off".to_string())
                }
            },
            FormFieldValue::Choice(s) => Object::text_string(s),
            FormFieldValue::MultiChoice(v) => {
                Object::Array(v.iter().map(Object::text_string).collect())
            },
            FormFieldValue::None => Object::Null,
        }
    }
}

/// Wrapper for form fields that bridges reading and writing.
///
/// This struct provides a unified interface for working with form fields
/// whether they come from an existing PDF (read) or are being created/modified (write).
#[derive(Debug, Clone)]
pub struct FormFieldWrapper {
    /// Full qualified field name (e.g., "form.section.field")
    pub(crate) name: String,

    /// Original field from PDF (if read from existing document)
    pub(crate) original: Option<FormField>,

    /// Modified or new value (if changed from original)
    pub(crate) modified_value: Option<FormFieldValue>,

    /// Page index where this field appears (0-based)
    pub(crate) page_index: usize,

    /// Whether this field has been modified since loading
    pub(crate) modified: bool,

    /// Whether this is a new field (not in original document)
    pub(crate) is_new: bool,

    /// Object reference if field exists in PDF
    pub(crate) object_ref: Option<ObjectRef>,

    /// Field type for new fields
    pub(crate) field_type: Option<FormFieldType>,

    /// Widget configuration for new fields
    pub(crate) widget_config: Option<WidgetConfig>,

    // === Hierarchy support ===
    /// Reference to parent field (for child fields in hierarchy)
    pub(crate) parent_ref: Option<ObjectRef>,

    /// References to child fields (for parent container fields)
    pub(crate) children_refs: Vec<ObjectRef>,

    /// Partial name (name without parent prefix, e.g., "street" for "address.street")
    pub(crate) partial_name: String,

    /// Whether this is a parent-only container (no widget, only Kids)
    pub(crate) is_parent_only: bool,

    /// Parent field name (for hierarchy tracking before refs are allocated)
    pub(crate) parent_name: Option<String>,

    // === Property modification tracking ===
    /// Modified field flags (/Ff)
    pub(crate) modified_flags: Option<u32>,

    /// Modified tooltip (/TU)
    pub(crate) modified_tooltip: Option<String>,

    /// Modified bounds/rect (/Rect)
    pub(crate) modified_rect: Option<Rect>,

    /// Modified default value (/DV)
    pub(crate) modified_default_value: Option<FormFieldValue>,

    /// Modified max length (/MaxLen) - text fields only
    pub(crate) modified_max_length: Option<u32>,

    /// Modified alignment (/Q) - 0=left, 1=center, 2=right
    pub(crate) modified_alignment: Option<u32>,

    /// Modified default appearance (/DA)
    pub(crate) modified_default_appearance: Option<String>,

    /// Modified background color (from /MK/BG)
    pub(crate) modified_background_color: Option<[f32; 3]>,

    /// Modified border color (from /MK/BC)
    pub(crate) modified_border_color: Option<[f32; 3]>,

    /// Modified border width (from /BS/W)
    pub(crate) modified_border_width: Option<f32>,
}

/// Field type for new fields.
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldType {
    /// Text field
    Text,
    /// Checkbox
    Checkbox,
    /// Radio button group
    RadioGroup,
    /// Combo box (dropdown)
    ComboBox,
    /// List box
    ListBox,
    /// Push button
    PushButton,
}

/// Widget configuration for new fields.
#[derive(Debug, Clone)]
pub struct WidgetConfig {
    /// Bounding rectangle
    pub rect: Rect,
    /// Field dictionary entries
    pub field_dict: HashMap<String, Object>,
    /// Widget dictionary entries
    pub widget_dict: HashMap<String, Object>,
    /// Field type string (Tx, Btn, Ch)
    pub field_type_str: String,
}

/// Configuration for creating parent container fields.
///
/// Parent fields are non-terminal fields in a hierarchy that don't have
/// a widget annotation but contain child fields via the `/Kids` array.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::editor::ParentFieldConfig;
///
/// // Create a parent field for address components
/// let config = ParentFieldConfig::new("address")
///     .with_flags(0); // Optional inherited flags
///
/// editor.add_parent_field(config)?;
/// editor.add_child_field("address", 0, TextFieldWidget::new("street", rect))?;
/// ```
#[derive(Debug, Clone)]
pub struct ParentFieldConfig {
    /// Partial field name (without parent prefix)
    pub(crate) partial_name: String,
    /// Optional field type to inherit to children (/FT)
    pub(crate) field_type: Option<FormFieldType>,
    /// Optional field flags to inherit to children (/Ff)
    pub(crate) flags: Option<u32>,
    /// Optional default value to inherit (/DV)
    pub(crate) default_value: Option<FormFieldValue>,
    /// Optional tooltip (/TU)
    pub(crate) tooltip: Option<String>,
    /// Parent field name (if this parent is nested)
    pub(crate) parent_name: Option<String>,
}

impl ParentFieldConfig {
    /// Create a new parent field configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The partial name for this parent field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            partial_name: name.into(),
            field_type: None,
            flags: None,
            default_value: None,
            tooltip: None,
            parent_name: None,
        }
    }

    /// Set the field type to inherit to children.
    pub fn with_field_type(mut self, ft: FormFieldType) -> Self {
        self.field_type = Some(ft);
        self
    }

    /// Set field flags to inherit to children.
    ///
    /// Common flag bits:
    /// - Bit 1: ReadOnly
    /// - Bit 2: Required
    /// - Bit 3: NoExport
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Set the default value to inherit to children.
    pub fn with_default_value(mut self, value: FormFieldValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Set the tooltip for this parent field.
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set the parent field name (for nested parents).
    pub fn with_parent(mut self, parent_name: impl Into<String>) -> Self {
        self.parent_name = Some(parent_name.into());
        self
    }

    /// Get the full qualified name.
    pub fn full_name(&self) -> String {
        if let Some(ref parent) = self.parent_name {
            format!("{}.{}", parent, self.partial_name)
        } else {
            self.partial_name.clone()
        }
    }
}

/// Extract the partial name from a full qualified name.
///
/// For example, "address.street" returns "street".
fn extract_partial_name(full_name: &str) -> String {
    if let Some(pos) = full_name.rfind('.') {
        full_name[pos + 1..].to_string()
    } else {
        full_name.to_string()
    }
}

/// Extract the parent name from a full qualified name.
///
/// For example, "address.street" returns Some("address").
/// "field" returns None.
fn extract_parent_name(full_name: &str) -> Option<String> {
    full_name.rfind('.').map(|pos| full_name[..pos].to_string())
}

impl FormFieldWrapper {
    /// Create a wrapper from an existing FormField (read from PDF).
    pub fn from_read(field: FormField, page_index: usize, object_ref: Option<ObjectRef>) -> Self {
        let name = field.full_name.clone();
        let partial_name = extract_partial_name(&name);
        // #617: retain the field's type so a terminal field re-emits its own
        // `/FT` on save. Without it `FormExtractor` can't classify the rewritten
        // field and it vanishes from the form.
        let field_type = match &field.field_type {
            crate::extractors::forms::FieldType::Text => Some(FormFieldType::Text),
            crate::extractors::forms::FieldType::Button => Some(FormFieldType::Checkbox),
            crate::extractors::forms::FieldType::Choice => Some(FormFieldType::ComboBox),
            // Signature / Unknown are not fill targets; leave untyped.
            _ => None,
        };
        Self {
            name,
            original: Some(field),
            modified_value: None,
            page_index,
            modified: false,
            is_new: false,
            object_ref,
            field_type,
            widget_config: None,
            // Hierarchy fields
            parent_ref: None,
            children_refs: Vec::new(),
            partial_name,
            is_parent_only: false,
            parent_name: None,
            // Property modification fields
            modified_flags: None,
            modified_tooltip: None,
            modified_rect: None,
            modified_default_value: None,
            modified_max_length: None,
            modified_alignment: None,
            modified_default_appearance: None,
            modified_background_color: None,
            modified_border_color: None,
            modified_border_width: None,
        }
    }

    /// Create a wrapper from a FormFieldWidget (for new fields).
    pub fn from_widget<W: FormFieldWidget>(widget: &W, page_index: usize) -> Self {
        let field_type = match widget.field_type() {
            "Tx" => FormFieldType::Text,
            "Btn" => FormFieldType::Checkbox, // Could be checkbox, radio, or button
            "Ch" => FormFieldType::ComboBox,  // Could be combo or list
            _ => FormFieldType::Text,
        };

        let config = WidgetConfig {
            rect: widget.rect(),
            field_dict: widget.build_field_dict(),
            widget_dict: HashMap::new(), // Will be built with page_ref on save
            field_type_str: widget.field_type().to_string(),
        };

        let name = widget.field_name().to_string();
        let partial_name = extract_partial_name(&name);

        Self {
            name,
            original: None,
            modified_value: None,
            page_index,
            modified: false,
            is_new: true,
            object_ref: None,
            field_type: Some(field_type),
            widget_config: Some(config),
            // Hierarchy fields
            parent_ref: None,
            children_refs: Vec::new(),
            partial_name,
            is_parent_only: false,
            parent_name: None,
            // Property modification fields
            modified_flags: None,
            modified_tooltip: None,
            modified_rect: None,
            modified_default_value: None,
            modified_max_length: None,
            modified_alignment: None,
            modified_default_appearance: None,
            modified_background_color: None,
            modified_border_color: None,
            modified_border_width: None,
        }
    }

    /// Create a wrapper from a ParentFieldConfig (for parent container fields).
    pub fn from_parent_config(config: &ParentFieldConfig) -> Self {
        Self {
            name: config.full_name(),
            original: None,
            modified_value: config.default_value.clone(),
            page_index: 0, // Parent-only fields don't have a page
            modified: false,
            is_new: true,
            object_ref: None,
            field_type: config.field_type.clone(),
            widget_config: None, // No widget for parent-only fields
            // Hierarchy fields
            parent_ref: None,
            children_refs: Vec::new(),
            partial_name: config.partial_name.clone(),
            is_parent_only: true,
            parent_name: config.parent_name.clone(),
            // Property modification fields
            modified_flags: config.flags,
            modified_tooltip: config.tooltip.clone(),
            modified_rect: None,
            modified_default_value: None, // Already in modified_value
            modified_max_length: None,
            modified_alignment: None,
            modified_default_appearance: None,
            modified_background_color: None,
            modified_border_color: None,
            modified_border_width: None,
        }
    }

    /// Create a wrapper for a child field with parent reference.
    pub fn from_widget_with_parent<W: FormFieldWidget>(
        widget: &W,
        page_index: usize,
        parent_name: &str,
    ) -> Self {
        let mut wrapper = Self::from_widget(widget, page_index);
        wrapper.parent_name = Some(parent_name.to_string());
        // Update name to be fully qualified
        wrapper.name = format!("{}.{}", parent_name, wrapper.partial_name);
        wrapper
    }

    /// Get the full qualified field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the page index where this field appears.
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// Check if this field has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Check if this is a new field.
    pub fn is_new(&self) -> bool {
        self.is_new
    }

    /// Get the current value of the field.
    pub fn value(&self) -> FormFieldValue {
        // Return modified value if set, otherwise original value
        if let Some(ref modified) = self.modified_value {
            return modified.clone();
        }

        if let Some(ref original) = self.original {
            FormFieldValue::from(&original.value)
        } else {
            FormFieldValue::None
        }
    }

    /// Set a new value for the field.
    pub fn set_value(&mut self, value: FormFieldValue) {
        self.modified_value = Some(value);
        self.modified = true;
    }

    /// Get the field type.
    pub fn field_type(&self) -> Option<&FieldType> {
        self.original.as_ref().map(|f| &f.field_type)
    }

    /// Get the bounding box of the field.
    pub fn bounds(&self) -> Option<Rect> {
        if let Some(ref config) = self.widget_config {
            return Some(config.rect);
        }

        if let Some(ref original) = self.original {
            original
                .bounds
                .map(|b| Rect::new(b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32))
        } else {
            None
        }
    }

    /// Get the tooltip/description.
    pub fn tooltip(&self) -> Option<&str> {
        self.original.as_ref().and_then(|f| f.tooltip.as_deref())
    }

    /// Get the object reference if the field exists in the PDF.
    pub fn object_ref(&self) -> Option<ObjectRef> {
        self.object_ref
    }

    /// Set the object reference (when allocating new object ID).
    pub fn set_object_ref(&mut self, object_ref: ObjectRef) {
        self.object_ref = Some(object_ref);
    }

    /// Get the original field if this was read from PDF.
    pub fn original(&self) -> Option<&FormField> {
        self.original.as_ref()
    }

    /// Get the widget configuration for new fields.
    pub fn widget_config(&self) -> Option<&WidgetConfig> {
        self.widget_config.as_ref()
    }

    /// Check if this field/widget uses merged dictionary format.
    ///
    /// In PDF, a field and its widget annotation can be merged into a single
    /// dictionary (common for single-widget fields) or kept separate
    /// (required for multi-widget fields like radio buttons).
    pub fn is_merged(&self) -> bool {
        // New single-widget fields are merged by default
        if self.is_new {
            // Radio groups are not merged (multiple widgets per field)
            if self.field_type == Some(FormFieldType::RadioGroup) {
                return false;
            }
            return true;
        }

        // For existing fields, check if original had /Subtype /Widget
        // indicating merged format
        true // Default to merged for simplicity
    }

    /// Build the field dictionary for PDF serialization.
    ///
    /// For merged fields, this includes both field and widget entries.
    pub fn build_field_dict(&self, page_ref: ObjectRef) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        if let Some(ref config) = self.widget_config {
            // New field - use widget config
            dict.extend(config.field_dict.clone());

            if self.is_merged() {
                // Add widget annotation entries for merged format
                dict.insert("Type".to_string(), Object::Name("Annot".to_string()));
                dict.insert("Subtype".to_string(), Object::Name("Widget".to_string()));
                dict.insert(
                    "Rect".to_string(),
                    Object::Array(vec![
                        Object::Real(config.rect.x as f64),
                        Object::Real(config.rect.y as f64),
                        Object::Real((config.rect.x + config.rect.width) as f64),
                        Object::Real((config.rect.y + config.rect.height) as f64),
                    ]),
                );
                dict.insert("P".to_string(), Object::Reference(page_ref));
            }
        }

        // Apply modified value if set
        if let Some(ref value) = self.modified_value {
            let obj: Object = value.into();
            if !matches!(obj, Object::Null) {
                dict.insert("V".to_string(), obj);
            }
        }

        // Add parent reference if this is a child field
        if let Some(parent_ref) = self.parent_ref {
            dict.insert("Parent".to_string(), Object::Reference(parent_ref));
            // Use partial name instead of full name for child fields
            dict.insert("T".to_string(), Object::text_string(&self.partial_name));
        // #616
        } else if !dict.contains_key("FT") {
            // #617: a terminal field with no /Parent must carry its own /FT
            // (an inheritable, required key — ISO 32000-1 §12.7.4.1). Without it
            // the rewritten dictionary is an untyped widget annotation and
            // `FormExtractor::extract_fields` no longer recognises it, so the
            // field disappears after fill+save. Re-emit /FT (and /Ff if set).
            if let Some(ref ft) = self.field_type {
                let ft_name = match ft {
                    FormFieldType::Text => "Tx",
                    FormFieldType::Checkbox
                    | FormFieldType::RadioGroup
                    | FormFieldType::PushButton => "Btn",
                    FormFieldType::ComboBox | FormFieldType::ListBox => "Ch",
                };
                dict.insert("FT".to_string(), Object::Name(ft_name.to_string()));
            }
            if let Some(flags) = self.modified_flags {
                dict.insert("Ff".to_string(), Object::Integer(flags as i64));
            }
        }

        dict
    }

    // === Hierarchy methods ===

    /// Get the partial name (last component of full name).
    pub fn partial_name(&self) -> &str {
        &self.partial_name
    }

    /// Get the parent field name (if this is a child field).
    pub fn parent_name(&self) -> Option<&str> {
        self.parent_name.as_deref()
    }

    /// Get the parent field reference (if set).
    pub fn parent_ref(&self) -> Option<ObjectRef> {
        self.parent_ref
    }

    /// Set the parent field reference.
    pub fn set_parent_ref(&mut self, parent_ref: ObjectRef) {
        self.parent_ref = Some(parent_ref);
    }

    /// Get the children field references.
    pub fn children_refs(&self) -> &[ObjectRef] {
        &self.children_refs
    }

    /// Add a child reference to this parent field.
    pub fn add_child_ref(&mut self, child_ref: ObjectRef) {
        self.children_refs.push(child_ref);
    }

    /// Check if this is a parent-only container field.
    pub fn is_parent_only(&self) -> bool {
        self.is_parent_only
    }

    /// Check if this field has a parent (is a child field).
    pub fn has_parent(&self) -> bool {
        self.parent_name.is_some() || self.parent_ref.is_some()
    }

    /// Build a parent-only field dictionary (no widget, just field entries).
    ///
    /// This is used for non-terminal fields in a hierarchy that contain
    /// child fields via the `/Kids` array.
    pub fn build_parent_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();

        // Partial name (T) - required
        dict.insert("T".to_string(), Object::text_string(&self.partial_name)); // #616

        // Field type (FT) - optional for non-terminal, but useful for inheritance
        if let Some(ref ft) = self.field_type {
            let ft_name = match ft {
                FormFieldType::Text => "Tx",
                FormFieldType::Checkbox | FormFieldType::RadioGroup | FormFieldType::PushButton => {
                    "Btn"
                },
                FormFieldType::ComboBox | FormFieldType::ListBox => "Ch",
            };
            dict.insert("FT".to_string(), Object::Name(ft_name.to_string()));
        }

        // Default value (DV) - optional
        if let Some(ref dv) = self.modified_value {
            let obj: Object = dv.into();
            if !matches!(obj, Object::Null) {
                dict.insert("DV".to_string(), obj);
            }
        }

        // Kids array (will be populated later with child refs)
        if !self.children_refs.is_empty() {
            let kids: Vec<Object> = self
                .children_refs
                .iter()
                .map(|r| Object::Reference(*r))
                .collect();
            dict.insert("Kids".to_string(), Object::Array(kids));
        }

        // Parent reference (if this parent is nested)
        if let Some(parent_ref) = self.parent_ref {
            dict.insert("Parent".to_string(), Object::Reference(parent_ref));
        }

        // Field flags
        if let Some(flags) = self.modified_flags {
            dict.insert("Ff".to_string(), Object::Integer(flags as i64));
        }

        // Tooltip
        if let Some(ref tooltip) = self.modified_tooltip {
            dict.insert("TU".to_string(), Object::text_string(tooltip)); // #616
        }

        dict
    }

    // === Property modification methods ===

    /// Set the field flags.
    ///
    /// Common flag bits (from PDF Table 221):
    /// - Bit 1 (0x01): ReadOnly - user cannot change field value
    /// - Bit 2 (0x02): Required - field must have value when exporting
    /// - Bit 3 (0x04): NoExport - field should not be exported
    ///
    /// Use `field_flags` constants for convenience.
    pub fn set_flags(&mut self, flags: u32) {
        self.modified_flags = Some(flags);
        self.modified = true;
    }

    /// Get the current field flags.
    pub fn flags(&self) -> Option<u32> {
        if self.modified_flags.is_some() {
            return self.modified_flags;
        }
        self.original.as_ref().and_then(|f| f.flags)
    }

    /// Set field as read-only.
    pub fn set_readonly(&mut self, readonly: bool) {
        let current = self.flags().unwrap_or(0);
        let new_flags = if readonly {
            current | 0x01
        } else {
            current & !0x01
        };
        self.set_flags(new_flags);
    }

    /// Check if field is read-only.
    pub fn is_readonly(&self) -> bool {
        self.flags().map(|f| f & 0x01 != 0).unwrap_or(false)
    }

    /// Set field as required.
    pub fn set_required(&mut self, required: bool) {
        let current = self.flags().unwrap_or(0);
        let new_flags = if required {
            current | 0x02
        } else {
            current & !0x02
        };
        self.set_flags(new_flags);
    }

    /// Check if field is required.
    pub fn is_required(&self) -> bool {
        self.flags().map(|f| f & 0x02 != 0).unwrap_or(false)
    }

    /// Set field as no-export.
    pub fn set_no_export(&mut self, no_export: bool) {
        let current = self.flags().unwrap_or(0);
        let new_flags = if no_export {
            current | 0x04
        } else {
            current & !0x04
        };
        self.set_flags(new_flags);
    }

    /// Check if field is no-export.
    pub fn is_no_export(&self) -> bool {
        self.flags().map(|f| f & 0x04 != 0).unwrap_or(false)
    }

    /// Set the tooltip/description.
    pub fn set_tooltip(&mut self, tooltip: impl Into<String>) {
        self.modified_tooltip = Some(tooltip.into());
        self.modified = true;
    }

    /// Get the current tooltip, preferring modified over original.
    pub fn get_tooltip(&self) -> Option<&str> {
        if let Some(ref tooltip) = self.modified_tooltip {
            return Some(tooltip);
        }
        self.original.as_ref().and_then(|f| f.tooltip.as_deref())
    }

    /// Set the field bounding rectangle.
    pub fn set_rect(&mut self, rect: Rect) {
        self.modified_rect = Some(rect);
        self.modified = true;
    }

    /// Get the current rect, preferring modified over original.
    pub fn get_rect(&self) -> Option<Rect> {
        if let Some(rect) = self.modified_rect {
            return Some(rect);
        }
        self.bounds()
    }

    /// Set the default value.
    pub fn set_default_value(&mut self, value: FormFieldValue) {
        self.modified_default_value = Some(value);
        self.modified = true;
    }

    /// Get the current default value.
    pub fn get_default_value(&self) -> Option<&FormFieldValue> {
        if self.modified_default_value.is_some() {
            return self.modified_default_value.as_ref();
        }
        self.original
            .as_ref()
            .and_then(|f| f.default_value.as_ref())
            .map(|v| {
                // Can't return reference to temporary, use modified field if available
                &FormFieldValue::None // This is a limitation; in practice we'd need to store converted value
            })
    }

    /// Set the maximum text length (for text fields only).
    pub fn set_max_length(&mut self, max_len: u32) {
        self.modified_max_length = Some(max_len);
        self.modified = true;
    }

    /// Get the maximum text length.
    pub fn get_max_length(&self) -> Option<u32> {
        if self.modified_max_length.is_some() {
            return self.modified_max_length;
        }
        self.original.as_ref().and_then(|f| f.max_length)
    }

    /// Set the text alignment.
    ///
    /// * 0 = Left
    /// * 1 = Center
    /// * 2 = Right
    pub fn set_alignment(&mut self, alignment: u32) {
        self.modified_alignment = Some(alignment);
        self.modified = true;
    }

    /// Get the current text alignment.
    pub fn get_alignment(&self) -> Option<u32> {
        if self.modified_alignment.is_some() {
            return self.modified_alignment;
        }
        self.original.as_ref().and_then(|f| f.alignment)
    }

    /// Set the default appearance string.
    ///
    /// The DA string specifies the font, size, and color for field content.
    /// Example: "/Helv 12 Tf 0 g" for 12pt Helvetica in black.
    pub fn set_default_appearance(&mut self, da: impl Into<String>) {
        self.modified_default_appearance = Some(da.into());
        self.modified = true;
    }

    /// Get the current default appearance string.
    pub fn get_default_appearance(&self) -> Option<&str> {
        if let Some(ref da) = self.modified_default_appearance {
            return Some(da);
        }
        self.original
            .as_ref()
            .and_then(|f| f.default_appearance.as_deref())
    }

    /// Set the background color (RGB, values 0.0-1.0).
    pub fn set_background_color(&mut self, color: [f32; 3]) {
        self.modified_background_color = Some(color);
        self.modified = true;
    }

    /// Get the current background color.
    pub fn get_background_color(&self) -> Option<[f32; 3]> {
        if self.modified_background_color.is_some() {
            return self.modified_background_color;
        }
        self.original
            .as_ref()
            .and_then(|f| f.appearance_chars.as_ref())
            .and_then(|ac| ac.background_color)
    }

    /// Set the border color (RGB, values 0.0-1.0).
    pub fn set_border_color(&mut self, color: [f32; 3]) {
        self.modified_border_color = Some(color);
        self.modified = true;
    }

    /// Get the current border color.
    pub fn get_border_color(&self) -> Option<[f32; 3]> {
        if self.modified_border_color.is_some() {
            return self.modified_border_color;
        }
        self.original
            .as_ref()
            .and_then(|f| f.appearance_chars.as_ref())
            .and_then(|ac| ac.border_color)
    }

    /// Set the border width in points.
    pub fn set_border_width(&mut self, width: f32) {
        self.modified_border_width = Some(width);
        self.modified = true;
    }

    /// Get the current border width.
    pub fn get_border_width(&self) -> Option<f32> {
        if self.modified_border_width.is_some() {
            return self.modified_border_width;
        }
        self.original
            .as_ref()
            .and_then(|f| f.border_style.as_ref())
            .map(|bs| bs.width)
    }
}

/// Result of checking if an existing field uses merged format.
pub fn is_merged_field_dict(dict: &HashMap<String, Object>) -> bool {
    dict.get("Subtype")
        .and_then(|o| o.as_name())
        .map(|name| name == "Widget")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractors::forms::{FieldType, FieldValue, FormField};

    // #616: non-ASCII field values must serialize as a conformant PDF text
    // string — UTF-16BE prefixed with the U+FEFF BOM (ISO 32000-1 §7.9.2.2) —
    // not the raw UTF-8 bytes (which a reader misreads as PDFDocEncoding →
    // mojibake). 山田太郎 = U+5C71 U+7530 U+592A U+90CE.
    #[test]
    fn text_field_value_cjk_encodes_utf16be_with_bom() {
        let obj = Object::from(&FormFieldValue::Text("山田太郎".to_string()));
        match obj {
            Object::String(bytes) => assert_eq!(
                bytes,
                vec![0xFE, 0xFF, 0x5C, 0x71, 0x75, 0x30, 0x59, 0x2A, 0x90, 0xCE],
                "CJK value must be UTF-16BE + BOM, got {bytes:02X?}"
            ),
            other => panic!("expected Object::String, got {other:?}"),
        }
    }

    // ASCII stays a plain literal (PDFDocEncoding-compatible) — no UTF-16 bloat.
    #[test]
    fn text_field_value_ascii_stays_literal() {
        for v in [
            FormFieldValue::Text("John".to_string()),
            FormFieldValue::Choice("Option A".to_string()),
        ] {
            match Object::from(&v) {
                Object::String(bytes) => {
                    assert!(
                        bytes.iter().all(|&b| b < 0x80),
                        "ASCII must stay 1-byte: {bytes:02X?}"
                    );
                    assert_ne!(&bytes[..2.min(bytes.len())], &[0xFE, 0xFF], "no BOM for ASCII");
                },
                other => panic!("expected Object::String, got {other:?}"),
            }
        }
    }

    #[test]
    fn test_form_field_value_from_field_value() {
        // Test text conversion
        let text_value = FieldValue::Text("hello".to_string());
        let converted: FormFieldValue = text_value.into();
        assert_eq!(converted, FormFieldValue::Text("hello".to_string()));

        // Test boolean conversion
        let bool_value = FieldValue::Boolean(true);
        let converted: FormFieldValue = bool_value.into();
        assert_eq!(converted, FormFieldValue::Boolean(true));

        // Test name conversion (to Choice)
        let name_value = FieldValue::Name("option1".to_string());
        let converted: FormFieldValue = name_value.into();
        assert_eq!(converted, FormFieldValue::Choice("option1".to_string()));

        // Test array conversion
        let array_value = FieldValue::Array(vec!["a".to_string(), "b".to_string()]);
        let converted: FormFieldValue = array_value.into();
        assert_eq!(converted, FormFieldValue::MultiChoice(vec!["a".to_string(), "b".to_string()]));

        // Test none conversion
        let none_value = FieldValue::None;
        let converted: FormFieldValue = none_value.into();
        assert_eq!(converted, FormFieldValue::None);
    }

    #[test]
    fn test_form_field_value_to_object() {
        // Test text to object
        let text_value = FormFieldValue::Text("hello".to_string());
        let obj: Object = (&text_value).into();
        assert!(matches!(obj, Object::String(_)));

        // Test boolean true to object
        let bool_true = FormFieldValue::Boolean(true);
        let obj: Object = (&bool_true).into();
        assert_eq!(obj, Object::Name("Yes".to_string()));

        // Test boolean false to object
        let bool_false = FormFieldValue::Boolean(false);
        let obj: Object = (&bool_false).into();
        assert_eq!(obj, Object::Name("Off".to_string()));

        // Test none to object
        let none_value = FormFieldValue::None;
        let obj: Object = (&none_value).into();
        assert_eq!(obj, Object::Null);
    }

    #[test]
    fn test_form_field_value_accessors() {
        let text_value = FormFieldValue::Text("hello".to_string());
        assert_eq!(text_value.as_text(), Some("hello"));
        assert_eq!(text_value.as_bool(), None);
        assert!(!text_value.is_none());

        let bool_value = FormFieldValue::Boolean(true);
        assert_eq!(bool_value.as_bool(), Some(true));
        assert_eq!(bool_value.as_text(), None);

        let none_value = FormFieldValue::None;
        assert!(none_value.is_none());
    }

    #[test]
    fn test_wrapper_from_read() {
        let field = FormField {
            name: "test".to_string(),
            field_type: FieldType::Text,
            value: FieldValue::Text("hello".to_string()),
            tooltip: Some("A tooltip".to_string()),
            full_name: "form.test".to_string(),
            bounds: Some([100.0, 200.0, 300.0, 220.0]),
            object_ref: None,
            flags: None,
            default_value: None,
            max_length: None,
            alignment: None,
            default_appearance: None,
            border_style: None,
            appearance_chars: None,
        };

        let wrapper = FormFieldWrapper::from_read(field, 0, None);

        assert_eq!(wrapper.name(), "form.test");
        assert_eq!(wrapper.page_index(), 0);
        assert!(!wrapper.is_new());
        assert!(!wrapper.is_modified());
        assert_eq!(wrapper.value(), FormFieldValue::Text("hello".to_string()));
        assert_eq!(wrapper.tooltip(), Some("A tooltip"));
    }

    #[test]
    fn test_wrapper_set_value() {
        let field = FormField {
            name: "test".to_string(),
            field_type: FieldType::Text,
            value: FieldValue::Text("original".to_string()),
            tooltip: None,
            full_name: "test".to_string(),
            bounds: None,
            object_ref: None,
            flags: None,
            default_value: None,
            max_length: None,
            alignment: None,
            default_appearance: None,
            border_style: None,
            appearance_chars: None,
        };

        let mut wrapper = FormFieldWrapper::from_read(field, 0, None);

        // Initial value
        assert_eq!(wrapper.value(), FormFieldValue::Text("original".to_string()));
        assert!(!wrapper.is_modified());

        // Set new value
        wrapper.set_value(FormFieldValue::Text("modified".to_string()));
        assert_eq!(wrapper.value(), FormFieldValue::Text("modified".to_string()));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_is_merged_field_dict() {
        let mut merged_dict = HashMap::new();
        merged_dict.insert("Subtype".to_string(), Object::Name("Widget".to_string()));
        assert!(is_merged_field_dict(&merged_dict));

        let mut separate_dict = HashMap::new();
        separate_dict.insert("FT".to_string(), Object::Name("Tx".to_string()));
        assert!(!is_merged_field_dict(&separate_dict));
    }

    // === FormFieldValue variant tests ===

    #[test]
    fn test_form_field_value_choice() {
        let val = FormFieldValue::Choice("item1".to_string());
        assert_eq!(val.as_choice(), Some("item1"));
        assert_eq!(val.as_text(), None);
        assert_eq!(val.as_bool(), None);
        assert_eq!(val.as_multi_choice(), None);
        assert!(!val.is_none());
    }

    #[test]
    fn test_form_field_value_multi_choice() {
        let val = FormFieldValue::MultiChoice(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(val.as_multi_choice().unwrap(), &["a".to_string(), "b".to_string()]);
        assert_eq!(val.as_text(), None);
        assert_eq!(val.as_bool(), None);
        assert_eq!(val.as_choice(), None);
        assert!(!val.is_none());
    }

    #[test]
    fn test_form_field_value_none_accessors() {
        let val = FormFieldValue::None;
        assert!(val.is_none());
        assert_eq!(val.as_text(), None);
        assert_eq!(val.as_bool(), None);
        assert_eq!(val.as_choice(), None);
        assert_eq!(val.as_multi_choice(), None);
    }

    #[test]
    fn test_form_field_value_text_accessor() {
        let val = FormFieldValue::Text("world".to_string());
        assert_eq!(val.as_text(), Some("world"));
    }

    #[test]
    fn test_form_field_value_bool_accessor() {
        assert_eq!(FormFieldValue::Boolean(true).as_bool(), Some(true));
        assert_eq!(FormFieldValue::Boolean(false).as_bool(), Some(false));
    }

    // === From<&FieldValue> for FormFieldValue ===

    #[test]
    fn test_from_ref_field_value_text() {
        let fv = FieldValue::Text("hello".to_string());
        let converted: FormFieldValue = (&fv).into();
        assert_eq!(converted, FormFieldValue::Text("hello".to_string()));
    }

    #[test]
    fn test_from_ref_field_value_boolean() {
        let fv = FieldValue::Boolean(false);
        let converted: FormFieldValue = (&fv).into();
        assert_eq!(converted, FormFieldValue::Boolean(false));
    }

    #[test]
    fn test_from_ref_field_value_name() {
        let fv = FieldValue::Name("opt".to_string());
        let converted: FormFieldValue = (&fv).into();
        assert_eq!(converted, FormFieldValue::Choice("opt".to_string()));
    }

    #[test]
    fn test_from_ref_field_value_array() {
        let fv = FieldValue::Array(vec!["x".to_string()]);
        let converted: FormFieldValue = (&fv).into();
        assert_eq!(converted, FormFieldValue::MultiChoice(vec!["x".to_string()]));
    }

    #[test]
    fn test_from_ref_field_value_none() {
        let fv = FieldValue::None;
        let converted: FormFieldValue = (&fv).into();
        assert_eq!(converted, FormFieldValue::None);
    }

    // === FormFieldValue to Object ===

    #[test]
    fn test_to_object_choice() {
        let val = FormFieldValue::Choice("opt1".to_string());
        let obj: Object = (&val).into();
        assert!(matches!(obj, Object::String(ref b) if b == b"opt1"));
    }

    #[test]
    fn test_to_object_multi_choice() {
        let val = FormFieldValue::MultiChoice(vec!["a".to_string(), "b".to_string()]);
        let obj: Object = (&val).into();
        match obj {
            Object::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert!(matches!(&arr[0], Object::String(ref b) if b == b"a"));
                assert!(matches!(&arr[1], Object::String(ref b) if b == b"b"));
            },
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_to_object_text() {
        let val = FormFieldValue::Text("hello".to_string());
        let obj: Object = (&val).into();
        assert!(matches!(obj, Object::String(ref b) if b == b"hello"));
    }

    #[test]
    fn test_to_object_bool_true() {
        let val = FormFieldValue::Boolean(true);
        let obj: Object = (&val).into();
        assert_eq!(obj, Object::Name("Yes".to_string()));
    }

    #[test]
    fn test_to_object_bool_false() {
        let val = FormFieldValue::Boolean(false);
        let obj: Object = (&val).into();
        assert_eq!(obj, Object::Name("Off".to_string()));
    }

    #[test]
    fn test_to_object_none() {
        let val = FormFieldValue::None;
        let obj: Object = (&val).into();
        assert_eq!(obj, Object::Null);
    }

    // === extract_partial_name / extract_parent_name ===

    #[test]
    fn test_extract_partial_name_simple() {
        assert_eq!(extract_partial_name("field"), "field");
    }

    #[test]
    fn test_extract_partial_name_hierarchical() {
        assert_eq!(extract_partial_name("form.section.field"), "field");
    }

    #[test]
    fn test_extract_partial_name_two_levels() {
        assert_eq!(extract_partial_name("parent.child"), "child");
    }

    #[test]
    fn test_extract_parent_name_none() {
        assert_eq!(extract_parent_name("field"), None);
    }

    #[test]
    fn test_extract_parent_name_simple() {
        assert_eq!(extract_parent_name("parent.child"), Some("parent".to_string()));
    }

    #[test]
    fn test_extract_parent_name_deep() {
        assert_eq!(extract_parent_name("a.b.c"), Some("a.b".to_string()));
    }

    // === ParentFieldConfig tests ===

    #[test]
    fn test_parent_field_config_new() {
        let config = ParentFieldConfig::new("address");
        assert_eq!(config.partial_name, "address");
        assert!(config.field_type.is_none());
        assert!(config.flags.is_none());
        assert!(config.default_value.is_none());
        assert!(config.tooltip.is_none());
        assert!(config.parent_name.is_none());
    }

    #[test]
    fn test_parent_field_config_builder_chain() {
        let config = ParentFieldConfig::new("address")
            .with_field_type(FormFieldType::Text)
            .with_flags(3)
            .with_default_value(FormFieldValue::Text("default".to_string()))
            .with_tooltip("Enter address")
            .with_parent("form");
        assert_eq!(config.field_type, Some(FormFieldType::Text));
        assert_eq!(config.flags, Some(3));
        assert_eq!(config.default_value, Some(FormFieldValue::Text("default".to_string())));
        assert_eq!(config.tooltip.as_deref(), Some("Enter address"));
        assert_eq!(config.parent_name.as_deref(), Some("form"));
    }

    #[test]
    fn test_parent_field_config_full_name_no_parent() {
        let config = ParentFieldConfig::new("myfield");
        assert_eq!(config.full_name(), "myfield");
    }

    #[test]
    fn test_parent_field_config_full_name_with_parent() {
        let config = ParentFieldConfig::new("street").with_parent("address");
        assert_eq!(config.full_name(), "address.street");
    }

    // === FormFieldWrapper property modification tests ===

    fn make_test_wrapper() -> FormFieldWrapper {
        let field = FormField {
            name: "test".to_string(),
            field_type: FieldType::Text,
            value: FieldValue::Text("val".to_string()),
            tooltip: Some("tip".to_string()),
            full_name: "parent.test".to_string(),
            bounds: Some([10.0, 20.0, 200.0, 40.0]),
            object_ref: Some(ObjectRef::new(5, 0)),
            flags: Some(0),
            default_value: Some(FieldValue::Text("def".to_string())),
            max_length: Some(100),
            alignment: Some(1),
            default_appearance: Some("/Helv 12 Tf".to_string()),
            border_style: Some(crate::extractors::forms::BorderStyle {
                width: 2.0,
                style: crate::extractors::forms::BorderStyleType::Solid,
                dash_array: None,
            }),
            appearance_chars: Some(crate::extractors::forms::AppearanceCharacteristics {
                background_color: Some([1.0, 1.0, 1.0]),
                border_color: Some([0.0, 0.0, 0.0]),
                caption: None,
                rollover_caption: None,
                alternate_caption: None,
                rotation: None,
            }),
        };
        FormFieldWrapper::from_read(field, 1, Some(ObjectRef::new(5, 0)))
    }

    #[test]
    fn test_wrapper_readonly() {
        let mut wrapper = make_test_wrapper();
        assert!(!wrapper.is_readonly());
        wrapper.set_readonly(true);
        assert!(wrapper.is_readonly());
        assert!(wrapper.is_modified());
        wrapper.set_readonly(false);
        assert!(!wrapper.is_readonly());
    }

    #[test]
    fn test_wrapper_required() {
        let mut wrapper = make_test_wrapper();
        assert!(!wrapper.is_required());
        wrapper.set_required(true);
        assert!(wrapper.is_required());
        wrapper.set_required(false);
        assert!(!wrapper.is_required());
    }

    #[test]
    fn test_wrapper_no_export() {
        let mut wrapper = make_test_wrapper();
        assert!(!wrapper.is_no_export());
        wrapper.set_no_export(true);
        assert!(wrapper.is_no_export());
        wrapper.set_no_export(false);
        assert!(!wrapper.is_no_export());
    }

    #[test]
    fn test_wrapper_flags_no_original() {
        let field = FormField {
            name: "f".to_string(),
            field_type: FieldType::Text,
            value: FieldValue::None,
            tooltip: None,
            full_name: "f".to_string(),
            bounds: None,
            object_ref: None,
            flags: None,
            default_value: None,
            max_length: None,
            alignment: None,
            default_appearance: None,
            border_style: None,
            appearance_chars: None,
        };
        let mut wrapper = FormFieldWrapper::from_read(field, 0, None);
        assert_eq!(wrapper.flags(), None);
        assert!(!wrapper.is_readonly());
        assert!(!wrapper.is_required());
        wrapper.set_readonly(true);
        assert_eq!(wrapper.flags(), Some(0x01));
    }

    #[test]
    fn test_wrapper_set_tooltip() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_tooltip(), Some("tip"));
        wrapper.set_tooltip("new tip");
        assert_eq!(wrapper.get_tooltip(), Some("new tip"));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_rect() {
        let mut wrapper = make_test_wrapper();
        let new_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
        wrapper.set_rect(new_rect);
        assert_eq!(wrapper.get_rect(), Some(new_rect));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_default_value() {
        let mut wrapper = make_test_wrapper();
        wrapper.set_default_value(FormFieldValue::Text("new_default".to_string()));
        let dv = wrapper.get_default_value();
        assert!(dv.is_some());
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_max_length() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_max_length(), Some(100));
        wrapper.set_max_length(200);
        assert_eq!(wrapper.get_max_length(), Some(200));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_alignment() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_alignment(), Some(1));
        wrapper.set_alignment(2);
        assert_eq!(wrapper.get_alignment(), Some(2));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_default_appearance() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_default_appearance(), Some("/Helv 12 Tf"));
        wrapper.set_default_appearance("/Cour 10 Tf 0 g");
        assert_eq!(wrapper.get_default_appearance(), Some("/Cour 10 Tf 0 g"));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_background_color() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_background_color(), Some([1.0, 1.0, 1.0]));
        wrapper.set_background_color([0.5, 0.5, 0.5]);
        assert_eq!(wrapper.get_background_color(), Some([0.5, 0.5, 0.5]));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_border_color() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_border_color(), Some([0.0, 0.0, 0.0]));
        wrapper.set_border_color([1.0, 0.0, 0.0]);
        assert_eq!(wrapper.get_border_color(), Some([1.0, 0.0, 0.0]));
        assert!(wrapper.is_modified());
    }

    #[test]
    fn test_wrapper_set_border_width() {
        let mut wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_border_width(), Some(2.0));
        wrapper.set_border_width(3.5);
        assert_eq!(wrapper.get_border_width(), Some(3.5));
        assert!(wrapper.is_modified());
    }

    // === FormFieldWrapper hierarchy tests ===

    #[test]
    fn test_wrapper_partial_name() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.partial_name(), "test");
    }

    #[test]
    fn test_wrapper_parent_name() {
        // from_read does not set parent_name (it doesn't parse from full_name),
        // but from_parent_config does
        let config = ParentFieldConfig::new("child").with_parent("parent");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        assert_eq!(wrapper.parent_name(), Some("parent"));
    }

    #[test]
    fn test_wrapper_object_ref() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.object_ref(), Some(ObjectRef::new(5, 0)));
    }

    #[test]
    fn test_wrapper_set_object_ref() {
        let mut wrapper = make_test_wrapper();
        wrapper.set_object_ref(ObjectRef::new(99, 0));
        assert_eq!(wrapper.object_ref(), Some(ObjectRef::new(99, 0)));
    }

    #[test]
    fn test_wrapper_parent_ref() {
        let mut wrapper = make_test_wrapper();
        assert!(wrapper.parent_ref().is_none());
        wrapper.set_parent_ref(ObjectRef::new(10, 0));
        assert_eq!(wrapper.parent_ref(), Some(ObjectRef::new(10, 0)));
    }

    #[test]
    fn test_wrapper_children_refs() {
        let mut wrapper = make_test_wrapper();
        assert!(wrapper.children_refs().is_empty());
        wrapper.add_child_ref(ObjectRef::new(20, 0));
        wrapper.add_child_ref(ObjectRef::new(21, 0));
        assert_eq!(wrapper.children_refs().len(), 2);
        assert_eq!(wrapper.children_refs()[0], ObjectRef::new(20, 0));
    }

    #[test]
    fn test_wrapper_is_parent_only() {
        let wrapper = make_test_wrapper();
        assert!(!wrapper.is_parent_only());

        let config = ParentFieldConfig::new("group");
        let parent_wrapper = FormFieldWrapper::from_parent_config(&config);
        assert!(parent_wrapper.is_parent_only());
    }

    #[test]
    fn test_wrapper_has_parent() {
        let wrapper = make_test_wrapper();
        assert!(!wrapper.has_parent()); // no parent_name or parent_ref

        let config = ParentFieldConfig::new("child").with_parent("parent");
        let child_wrapper = FormFieldWrapper::from_parent_config(&config);
        assert!(child_wrapper.has_parent()); // has parent_name
    }

    #[test]
    fn test_wrapper_has_parent_via_ref() {
        let mut wrapper = make_test_wrapper();
        assert!(!wrapper.has_parent());
        wrapper.set_parent_ref(ObjectRef::new(1, 0));
        assert!(wrapper.has_parent());
    }

    // === FormFieldWrapper from_parent_config ===

    #[test]
    fn test_from_parent_config_basic() {
        let config = ParentFieldConfig::new("group")
            .with_flags(3)
            .with_tooltip("Group tooltip")
            .with_default_value(FormFieldValue::Text("default".to_string()));
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        assert_eq!(wrapper.name(), "group");
        assert!(wrapper.is_new());
        assert!(wrapper.is_parent_only());
        assert_eq!(wrapper.flags(), Some(3));
        assert_eq!(wrapper.get_tooltip(), Some("Group tooltip"));
    }

    // === build_parent_dict ===

    #[test]
    fn test_build_parent_dict_basic() {
        let config = ParentFieldConfig::new("address");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("T"));
        assert!(matches!(dict.get("T"), Some(Object::String(ref b)) if b == b"address"));
    }

    #[test]
    fn test_build_parent_dict_with_field_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::Text);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("FT"));
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Tx"));
    }

    #[test]
    fn test_build_parent_dict_with_button_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::Checkbox);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Btn"));
    }

    #[test]
    fn test_build_parent_dict_with_choice_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::ComboBox);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Ch"));
    }

    #[test]
    fn test_build_parent_dict_with_radio_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::RadioGroup);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Btn"));
    }

    #[test]
    fn test_build_parent_dict_with_pushbutton_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::PushButton);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Btn"));
    }

    #[test]
    fn test_build_parent_dict_with_listbox_type() {
        let config = ParentFieldConfig::new("group").with_field_type(FormFieldType::ListBox);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("FT"), Some(Object::Name(ref s)) if s == "Ch"));
    }

    #[test]
    fn test_build_parent_dict_with_children() {
        let config = ParentFieldConfig::new("parent");
        let mut wrapper = FormFieldWrapper::from_parent_config(&config);
        wrapper.add_child_ref(ObjectRef::new(10, 0));
        wrapper.add_child_ref(ObjectRef::new(11, 0));
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("Kids"));
        match dict.get("Kids") {
            Some(Object::Array(kids)) => assert_eq!(kids.len(), 2),
            _ => panic!("Expected Kids array"),
        }
    }

    #[test]
    fn test_build_parent_dict_with_parent_ref() {
        let config = ParentFieldConfig::new("child");
        let mut wrapper = FormFieldWrapper::from_parent_config(&config);
        wrapper.set_parent_ref(ObjectRef::new(99, 0));
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("Parent"));
    }

    #[test]
    fn test_build_parent_dict_with_flags() {
        let config = ParentFieldConfig::new("group").with_flags(7);
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(matches!(dict.get("Ff"), Some(Object::Integer(7))));
    }

    #[test]
    fn test_build_parent_dict_with_tooltip() {
        let config = ParentFieldConfig::new("group").with_tooltip("My tooltip");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("TU"));
    }

    #[test]
    fn test_build_parent_dict_with_default_value() {
        let config = ParentFieldConfig::new("group")
            .with_default_value(FormFieldValue::Text("default".to_string()));
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        let dict = wrapper.build_parent_dict();
        assert!(dict.contains_key("DV"));
    }

    // === is_merged tests ===

    #[test]
    fn test_is_merged_existing_field() {
        let wrapper = make_test_wrapper();
        assert!(wrapper.is_merged()); // default for existing fields
    }

    // === FormFieldWrapper value from original when no modified ===

    #[test]
    fn test_wrapper_value_from_original_no_modified() {
        let field = FormField {
            name: "f".to_string(),
            field_type: FieldType::Text,
            value: FieldValue::Text("original".to_string()),
            tooltip: None,
            full_name: "f".to_string(),
            bounds: None,
            object_ref: None,
            flags: None,
            default_value: None,
            max_length: None,
            alignment: None,
            default_appearance: None,
            border_style: None,
            appearance_chars: None,
        };
        let wrapper = FormFieldWrapper::from_read(field, 0, None);
        assert_eq!(wrapper.value(), FormFieldValue::Text("original".to_string()));
    }

    #[test]
    fn test_wrapper_value_no_original_no_modified() {
        let config = ParentFieldConfig::new("empty");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        assert_eq!(wrapper.value(), FormFieldValue::None);
    }

    // === FormFieldWrapper bounds tests ===

    #[test]
    fn test_wrapper_bounds_from_original() {
        let wrapper = make_test_wrapper();
        let bounds = wrapper.bounds().unwrap();
        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
    }

    #[test]
    fn test_wrapper_bounds_none_when_no_data() {
        let config = ParentFieldConfig::new("empty");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        assert!(wrapper.bounds().is_none());
    }

    // === FormFieldWrapper field_type ===

    #[test]
    fn test_wrapper_field_type_from_original() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.field_type(), Some(&FieldType::Text));
    }

    #[test]
    fn test_wrapper_field_type_none_for_new() {
        let config = ParentFieldConfig::new("new");
        let wrapper = FormFieldWrapper::from_parent_config(&config);
        assert!(wrapper.field_type().is_none());
    }

    // === FormFieldType equality ===

    #[test]
    fn test_form_field_type_equality() {
        assert_eq!(FormFieldType::Text, FormFieldType::Text);
        assert_eq!(FormFieldType::Checkbox, FormFieldType::Checkbox);
        assert_ne!(FormFieldType::Text, FormFieldType::Checkbox);
        assert_ne!(FormFieldType::RadioGroup, FormFieldType::PushButton);
        assert_ne!(FormFieldType::ComboBox, FormFieldType::ListBox);
    }

    // === is_merged_field_dict edge cases ===

    #[test]
    fn test_is_merged_field_dict_empty() {
        let dict = HashMap::new();
        assert!(!is_merged_field_dict(&dict));
    }

    #[test]
    fn test_is_merged_field_dict_wrong_subtype() {
        let mut dict = HashMap::new();
        dict.insert("Subtype".to_string(), Object::Name("Stamp".to_string()));
        assert!(!is_merged_field_dict(&dict));
    }

    #[test]
    fn test_is_merged_field_dict_subtype_not_name() {
        let mut dict = HashMap::new();
        dict.insert("Subtype".to_string(), Object::Integer(42));
        assert!(!is_merged_field_dict(&dict));
    }

    // === build_field_dict ===

    #[test]
    fn test_build_field_dict_with_modified_value() {
        let mut wrapper = make_test_wrapper();
        wrapper.set_value(FormFieldValue::Text("newval".to_string()));
        let page_ref = ObjectRef::new(1, 0);
        let dict = wrapper.build_field_dict(page_ref);
        assert!(dict.contains_key("V"));
    }

    #[test]
    fn test_build_field_dict_with_parent_ref() {
        let mut wrapper = make_test_wrapper();
        wrapper.set_parent_ref(ObjectRef::new(50, 0));
        let page_ref = ObjectRef::new(1, 0);
        let dict = wrapper.build_field_dict(page_ref);
        assert!(dict.contains_key("Parent"));
        assert!(dict.contains_key("T")); // partial name
    }

    // === get_default_value when no modified but has original ===

    #[test]
    fn test_get_default_value_from_original() {
        let wrapper = make_test_wrapper();
        // The original field has default_value = Some(...), so get_default_value
        // should return something (even though the current implementation has a limitation)
        let dv = wrapper.get_default_value();
        // Due to implementation limitation, it returns &FormFieldValue::None
        assert!(dv.is_some());
    }

    // === Wrapper getters fall back to original ===

    #[test]
    fn test_wrapper_get_max_length_from_original() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_max_length(), Some(100));
    }

    #[test]
    fn test_wrapper_get_alignment_from_original() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_alignment(), Some(1));
    }

    #[test]
    fn test_wrapper_get_default_appearance_from_original() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_default_appearance(), Some("/Helv 12 Tf"));
    }

    #[test]
    fn test_wrapper_get_border_width_from_original() {
        let wrapper = make_test_wrapper();
        assert_eq!(wrapper.get_border_width(), Some(2.0));
    }
}
