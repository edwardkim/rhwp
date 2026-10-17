//! Interactive form field widgets for PDF generation.
//!
//! This module provides builders for creating interactive form fields
//! per ISO 32000-1:2008 Section 12.7 (Interactive Forms).
//!
//! # Supported Field Types
//!
//! - **Text Fields** (`TextFieldWidget`): Single-line and multiline text input
//! - **Checkboxes** (`CheckboxWidget`): Boolean on/off fields
//! - **Radio Buttons** (`RadioButtonGroup`): Mutually exclusive choices
//! - **Combo Boxes** (`ComboBoxWidget`): Dropdown selection lists
//! - **List Boxes** (`ListBoxWidget`): Scrollable selection lists
//! - **Push Buttons** (`PushButtonWidget`): Action triggers (submit, reset)
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::writer::form_fields::{TextFieldWidget, CheckboxWidget};
//! use pdf_oxide::geometry::Rect;
//!
//! // Create a text field
//! let name_field = TextFieldWidget::new("name", Rect::new(72.0, 700.0, 200.0, 20.0))
//!     .with_value("John Doe")
//!     .required();
//!
//! // Create a checkbox
//! let agree_checkbox = CheckboxWidget::new("agree", Rect::new(72.0, 650.0, 15.0, 15.0))
//!     .checked();
//! ```

mod checkbox;
mod choice_fields;
mod field_flags;
mod form_appearance;
mod push_button;
mod radio_button;
mod signature;
mod text_field;

pub use checkbox::CheckboxWidget;
pub use choice_fields::{ChoiceOption, ComboBoxWidget, ListBoxWidget};
pub use field_flags::{
    ButtonFieldFlags, ChoiceFieldFlags, FieldFlags, TextAlignment, TextFieldFlags,
};
pub use form_appearance::FormAppearanceGenerator;
pub use push_button::{FormAction, PushButtonWidget, SubmitFormFlags};
pub use radio_button::{RadioButtonGroup, RadioButtonWidget};
pub use signature::SignatureWidget;
pub use text_field::TextFieldWidget;

use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Common trait for all form field widgets.
///
/// This trait defines the interface that all form fields must implement
/// to be added to a PDF page.
pub trait FormFieldWidget {
    /// Get the field name (partial name for hierarchical fields).
    fn field_name(&self) -> &str;

    /// Get the bounding rectangle of the field's widget annotation.
    fn rect(&self) -> Rect;

    /// Build the field dictionary entries.
    ///
    /// Returns entries that go into the field dictionary object.
    fn build_field_dict(&self) -> HashMap<String, Object>;

    /// Build the widget annotation dictionary entries.
    ///
    /// Returns entries that go into the widget annotation object.
    /// For merged field/widget, these go in the same object.
    fn build_widget_dict(&self, page_ref: ObjectRef) -> HashMap<String, Object>;

    /// Get the field type name (Tx, Btn, Ch, Sig).
    fn field_type(&self) -> &'static str;

    /// Get the field flags value.
    fn field_flags(&self) -> u32;

    /// Whether this field requires appearance generation.
    fn needs_appearance(&self) -> bool {
        true
    }
}

/// Entry representing a form field for page integration.
#[derive(Debug, Clone)]
pub struct FormFieldEntry {
    /// The widget implementation
    pub widget_dict: HashMap<String, Object>,
    /// Field dictionary (may be same as widget for merged)
    pub field_dict: HashMap<String, Object>,
    /// Field name
    pub name: String,
    /// Bounding rectangle
    pub rect: Rect,
    /// Field type (Tx, Btn, Ch)
    pub field_type: String,
}
