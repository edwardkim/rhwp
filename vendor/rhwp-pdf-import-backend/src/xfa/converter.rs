//! XFA to AcroForm converter.
//!
//! Converts parsed XFA form data to AcroForm structure that can be used
//! with standard PDF form processing.

use super::parser::{XfaField, XfaFieldType, XfaForm, XfaPage};
use crate::error::Result;
use crate::geometry::Rect;
use crate::writer::form_fields::{
    CheckboxWidget, ChoiceOption, ComboBoxWidget, FormFieldWidget, ListBoxWidget, PushButtonWidget,
    RadioButtonGroup, TextFieldWidget,
};

/// Options for XFA to AcroForm conversion.
#[derive(Debug, Clone)]
pub struct XfaConversionOptions {
    /// Page width for layout (default: 612 = Letter)
    pub page_width: f32,
    /// Page height for layout (default: 792 = Letter)
    pub page_height: f32,
    /// Left margin
    pub margin_left: f32,
    /// Right margin
    pub margin_right: f32,
    /// Top margin
    pub margin_top: f32,
    /// Bottom margin
    pub margin_bottom: f32,
    /// Vertical spacing between fields
    pub field_spacing: f32,
    /// Default field width
    pub default_field_width: f32,
    /// Default field height
    pub default_field_height: f32,
    /// Font size for text fields
    pub font_size: f32,
    /// Include field captions as separate text
    pub include_captions: bool,
}

impl Default for XfaConversionOptions {
    fn default() -> Self {
        Self {
            page_width: 612.0,
            page_height: 792.0,
            margin_left: 72.0,
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
            field_spacing: 24.0,
            default_field_width: 200.0,
            default_field_height: 20.0,
            font_size: 12.0,
            include_captions: true,
        }
    }
}

/// Converted AcroForm field.
#[derive(Debug, Clone)]
pub enum ConvertedField {
    /// Text field
    Text(TextFieldWidget),
    /// Checkbox
    Checkbox(CheckboxWidget),
    /// Radio button group
    RadioGroup(RadioButtonGroup),
    /// Combo box (dropdown)
    ComboBox(ComboBoxWidget),
    /// List box
    ListBox(ListBoxWidget),
    /// Push button
    Button(PushButtonWidget),
}

impl ConvertedField {
    /// Get the field name.
    pub fn name(&self) -> &str {
        match self {
            ConvertedField::Text(f) => f.field_name(),
            ConvertedField::Checkbox(f) => f.field_name(),
            ConvertedField::RadioGroup(f) => f.name(),
            ConvertedField::ComboBox(f) => f.field_name(),
            ConvertedField::ListBox(f) => f.field_name(),
            ConvertedField::Button(f) => f.field_name(),
        }
    }

    /// Get the field rectangle.
    pub fn rect(&self) -> Rect {
        match self {
            ConvertedField::Text(f) => f.rect(),
            ConvertedField::Checkbox(f) => f.rect(),
            ConvertedField::RadioGroup(f) => {
                // Return first button's rect
                if let Some(btn) = f.buttons().first() {
                    btn.rect()
                } else {
                    Rect::new(0.0, 0.0, 0.0, 0.0)
                }
            },
            ConvertedField::ComboBox(f) => f.rect(),
            ConvertedField::ListBox(f) => f.rect(),
            ConvertedField::Button(f) => f.rect(),
        }
    }
}

/// A page with converted fields and optional captions.
#[derive(Debug, Clone)]
pub struct ConvertedPage {
    /// Page index
    pub index: usize,
    /// Page width
    pub width: f32,
    /// Page height
    pub height: f32,
    /// Converted fields
    pub fields: Vec<ConvertedField>,
    /// Caption text (position, text)
    pub captions: Vec<(Rect, String)>,
}

/// Result of XFA conversion.
#[derive(Debug, Clone)]
pub struct XfaConversionResult {
    /// Converted pages
    pub pages: Vec<ConvertedPage>,
    /// Total number of fields converted
    pub field_count: usize,
    /// Fields that couldn't be converted (name, reason)
    pub skipped_fields: Vec<(String, String)>,
}

/// XFA to AcroForm converter.
pub struct XfaConverter {
    options: XfaConversionOptions,
}

impl Default for XfaConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl XfaConverter {
    /// Create a new converter with default options.
    pub fn new() -> Self {
        Self {
            options: XfaConversionOptions::default(),
        }
    }

    /// Create a converter with custom options.
    pub fn with_options(options: XfaConversionOptions) -> Self {
        Self { options }
    }

    /// Convert an XFA form to AcroForm structure.
    pub fn convert(&self, form: &XfaForm) -> Result<XfaConversionResult> {
        let mut result = XfaConversionResult {
            pages: Vec::new(),
            field_count: 0,
            skipped_fields: Vec::new(),
        };

        // If form has pages, convert page by page
        if !form.pages.is_empty() {
            for (idx, xfa_page) in form.pages.iter().enumerate() {
                let page = self.convert_page(xfa_page, idx, &mut result.skipped_fields);
                result.field_count += page.fields.len();
                result.pages.push(page);
            }
        } else {
            // No pages defined, put all fields on a single page
            let fields = &form.fields;
            let page = self.convert_fields_to_page(fields, 0, &mut result.skipped_fields);
            result.field_count = page.fields.len();
            result.pages.push(page);
        }

        Ok(result)
    }

    /// Convert a single XFA page.
    fn convert_page(
        &self,
        xfa_page: &XfaPage,
        index: usize,
        skipped: &mut Vec<(String, String)>,
    ) -> ConvertedPage {
        let width = if xfa_page.width > 0.0 {
            xfa_page.width
        } else {
            self.options.page_width
        };
        let height = if xfa_page.height > 0.0 {
            xfa_page.height
        } else {
            self.options.page_height
        };

        let mut page = ConvertedPage {
            index,
            width,
            height,
            fields: Vec::new(),
            captions: Vec::new(),
        };

        let mut y_position = height - self.options.margin_top - self.options.default_field_height;

        for field in &xfa_page.fields {
            // Calculate field position
            let x = field.x.unwrap_or(self.options.margin_left);
            let y = field.y.unwrap_or(y_position);
            let w = field.width.unwrap_or(self.options.default_field_width);
            let h = field.height.unwrap_or(self.options.default_field_height);

            let rect = Rect::new(x, y, w, h);

            // Add caption if present
            if self.options.include_captions {
                if let Some(ref caption) = field.caption {
                    let caption_rect = Rect::new(
                        x,
                        y + h + 2.0, // Above the field
                        w,
                        self.options.font_size + 2.0,
                    );
                    page.captions.push((caption_rect, caption.clone()));
                }
            }

            // Convert the field
            match self.convert_field(field, rect) {
                Ok(converted) => {
                    page.fields.push(converted);
                },
                Err(reason) => {
                    skipped.push((field.name.clone(), reason));
                },
            }

            // Update y position for next field (if not using XFA positions)
            if field.y.is_none() {
                y_position -= h + self.options.field_spacing;
                if self.options.include_captions && field.caption.is_some() {
                    y_position -= self.options.font_size + 4.0;
                }
            }
        }

        page
    }

    /// Convert a list of fields to a page (for forms without explicit pages).
    fn convert_fields_to_page(
        &self,
        fields: &[XfaField],
        index: usize,
        skipped: &mut Vec<(String, String)>,
    ) -> ConvertedPage {
        let mut page = ConvertedPage {
            index,
            width: self.options.page_width,
            height: self.options.page_height,
            fields: Vec::new(),
            captions: Vec::new(),
        };

        let mut y_position =
            self.options.page_height - self.options.margin_top - self.options.default_field_height;

        for field in fields {
            // Calculate field position
            let x = field.x.unwrap_or(self.options.margin_left);
            let y = field.y.unwrap_or(y_position);
            let w = field.width.unwrap_or(self.options.default_field_width);
            let h = field.height.unwrap_or(self.options.default_field_height);

            let rect = Rect::new(x, y, w, h);

            // Add caption if present
            if self.options.include_captions {
                if let Some(ref caption) = field.caption {
                    let caption_rect = Rect::new(x, y + h + 2.0, w, self.options.font_size + 2.0);
                    page.captions.push((caption_rect, caption.clone()));
                }
            }

            // Convert the field
            match self.convert_field(field, rect) {
                Ok(converted) => {
                    page.fields.push(converted);
                },
                Err(reason) => {
                    skipped.push((field.name.clone(), reason));
                },
            }

            // Update y position for next field
            if field.y.is_none() {
                y_position -= h + self.options.field_spacing;
                if self.options.include_captions && field.caption.is_some() {
                    y_position -= self.options.font_size + 4.0;
                }

                // Check if we need a new page
                if y_position < self.options.margin_bottom {
                    // In a real implementation, we'd create a new page
                    // For simplicity, we just wrap around
                    y_position = self.options.page_height
                        - self.options.margin_top
                        - self.options.default_field_height;
                }
            }
        }

        page
    }

    /// Convert a single XFA field to an AcroForm field.
    fn convert_field(
        &self,
        field: &XfaField,
        rect: Rect,
    ) -> std::result::Result<ConvertedField, String> {
        let name = &field.name;
        let value = field.value.as_deref().or(field.default_value.as_deref());

        match field.field_type {
            XfaFieldType::Text | XfaFieldType::Numeric | XfaFieldType::DateTime => {
                let mut widget = TextFieldWidget::new(name.clone(), rect);

                if let Some(v) = value {
                    widget = widget.with_value(v);
                }

                if let Some(ref tooltip) = field.tooltip {
                    widget = widget.with_tooltip(tooltip);
                }

                if let Some(max_len) = field.max_length {
                    widget = widget.with_max_length(max_len);
                }

                if field.readonly {
                    widget = widget.read_only();
                }

                if field.required {
                    widget = widget.required();
                }

                Ok(ConvertedField::Text(widget))
            },

            XfaFieldType::Checkbox => {
                let mut widget = CheckboxWidget::new(name.clone(), rect);

                // Check if it's checked
                let is_checked = value
                    .map(|v| {
                        v == "1" || v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true")
                    })
                    .unwrap_or(false);

                if is_checked {
                    widget = widget.checked();
                }

                if let Some(ref tooltip) = field.tooltip {
                    widget = widget.with_tooltip(tooltip);
                }

                Ok(ConvertedField::Checkbox(widget))
            },

            XfaFieldType::RadioGroup => {
                let mut group = RadioButtonGroup::new(name.clone());

                // Create radio buttons from options
                for (idx, option) in field.options.iter().enumerate() {
                    let btn_rect = Rect::new(
                        rect.x,
                        rect.y - (idx as f32 * (rect.height + 4.0)),
                        rect.width,
                        rect.height,
                    );

                    group = group.add_button(&option.value, btn_rect, &option.text);
                }

                // Set selection if a value matches
                if let Some(v) = value {
                    group = group.selected(v);
                }

                Ok(ConvertedField::RadioGroup(group))
            },

            XfaFieldType::DropDown => {
                let options: Vec<ChoiceOption> = field
                    .options
                    .iter()
                    .map(|o| ChoiceOption::new_with_export(&o.text, &o.value))
                    .collect();

                let mut widget =
                    ComboBoxWidget::new(name.clone(), rect).with_choice_options(options);

                if let Some(v) = value {
                    widget = widget.with_value(v);
                }

                if let Some(ref tooltip) = field.tooltip {
                    widget = widget.with_tooltip(tooltip);
                }

                Ok(ConvertedField::ComboBox(widget))
            },

            XfaFieldType::ListBox => {
                let options: Vec<ChoiceOption> = field
                    .options
                    .iter()
                    .map(|o| ChoiceOption::new_with_export(&o.text, &o.value))
                    .collect();

                let mut widget =
                    ListBoxWidget::new(name.clone(), rect).with_choice_options(options);

                if let Some(v) = value {
                    widget = widget.with_value(v);
                }

                if let Some(ref tooltip) = field.tooltip {
                    widget = widget.with_tooltip(tooltip);
                }

                Ok(ConvertedField::ListBox(widget))
            },

            XfaFieldType::Button => {
                let label = field.caption.as_deref().unwrap_or(name);
                let widget = PushButtonWidget::new(name.clone(), rect).with_caption(label);

                Ok(ConvertedField::Button(widget))
            },

            XfaFieldType::Signature => {
                // Convert signature to a text field with special handling
                // (Full signature support would require digital signature infrastructure)
                let widget = TextFieldWidget::new(name.clone(), rect).read_only();

                Ok(ConvertedField::Text(widget))
            },

            XfaFieldType::Image | XfaFieldType::Barcode => {
                // These can't be directly converted to AcroForm
                Err(format!("Field type {:?} cannot be converted to AcroForm", field.field_type))
            },

            XfaFieldType::Unknown(ref typ) => {
                // Try to create a text field as fallback
                let widget = TextFieldWidget::new(name.clone(), rect);
                if let Some(val) = value {
                    Ok(ConvertedField::Text(widget.with_value(val)))
                } else {
                    Err(format!("Unknown field type: {}", typ))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_options_default() {
        let options = XfaConversionOptions::default();

        assert_eq!(options.page_width, 612.0);
        assert_eq!(options.page_height, 792.0);
        assert_eq!(options.margin_left, 72.0);
        assert!(options.include_captions);
    }

    #[test]
    fn test_converted_field_name() {
        let widget = TextFieldWidget::new("test".to_string(), Rect::new(0.0, 0.0, 100.0, 20.0));
        let field = ConvertedField::Text(widget);

        assert_eq!(field.name(), "test");
    }

    #[test]
    fn test_converted_field_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 30.0);
        let widget = TextFieldWidget::new("test".to_string(), rect);
        let field = ConvertedField::Text(widget);

        assert_eq!(field.rect().x, 10.0);
        assert_eq!(field.rect().y, 20.0);
    }

    #[test]
    fn test_converter_new() {
        let converter = XfaConverter::new();
        assert_eq!(converter.options.page_width, 612.0);
    }

    #[test]
    fn test_converter_with_options() {
        let options = XfaConversionOptions {
            page_width: 595.0, // A4 width
            ..Default::default()
        };

        let converter = XfaConverter::with_options(options);
        assert_eq!(converter.options.page_width, 595.0);
    }

    #[test]
    fn test_convert_text_field() {
        let mut xfa_field = XfaField::new("firstName", "binding");
        xfa_field.field_type = XfaFieldType::Text;
        xfa_field.value = Some("John".to_string());
        xfa_field.tooltip = Some("Enter your first name".to_string());

        let converter = XfaConverter::new();
        let rect = Rect::new(72.0, 700.0, 200.0, 20.0);

        let result = converter.convert_field(&xfa_field, rect);
        assert!(result.is_ok());

        if let Ok(ConvertedField::Text(widget)) = result {
            assert_eq!(widget.field_name(), "firstName");
            // Value is set internally, conversion verified by no error
        } else {
            panic!("Expected text field");
        }
    }

    #[test]
    fn test_convert_checkbox_field() {
        let mut xfa_field = XfaField::new("agree", "binding");
        xfa_field.field_type = XfaFieldType::Checkbox;
        xfa_field.value = Some("1".to_string());

        let converter = XfaConverter::new();
        let rect = Rect::new(72.0, 700.0, 20.0, 20.0);

        let result = converter.convert_field(&xfa_field, rect);
        assert!(result.is_ok());

        if let Ok(ConvertedField::Checkbox(widget)) = result {
            assert_eq!(widget.field_name(), "agree");
            assert!(widget.is_checked());
        } else {
            panic!("Expected checkbox field");
        }
    }

    #[test]
    fn test_convert_dropdown_field() {
        use super::super::parser::XfaOption;

        let mut xfa_field = XfaField::new("country", "binding");
        xfa_field.field_type = XfaFieldType::DropDown;
        xfa_field.options = vec![
            XfaOption {
                text: "United States".to_string(),
                value: "US".to_string(),
            },
            XfaOption {
                text: "Canada".to_string(),
                value: "CA".to_string(),
            },
        ];

        let converter = XfaConverter::new();
        let rect = Rect::new(72.0, 700.0, 200.0, 20.0);

        let result = converter.convert_field(&xfa_field, rect);
        assert!(result.is_ok());

        if let Ok(ConvertedField::ComboBox(widget)) = result {
            assert_eq!(widget.field_name(), "country");
            // Options are set internally, verified by successful conversion
        } else {
            panic!("Expected combo box field");
        }
    }

    #[test]
    fn test_convert_button_field() {
        let mut xfa_field = XfaField::new("submit", "binding");
        xfa_field.field_type = XfaFieldType::Button;
        xfa_field.caption = Some("Submit Form".to_string());

        let converter = XfaConverter::new();
        let rect = Rect::new(72.0, 700.0, 100.0, 30.0);

        let result = converter.convert_field(&xfa_field, rect);
        assert!(result.is_ok());

        if let Ok(ConvertedField::Button(widget)) = result {
            assert_eq!(widget.field_name(), "submit");
            // Caption is set internally, verified by successful conversion
        } else {
            panic!("Expected button field");
        }
    }

    #[test]
    fn test_convert_unsupported_field() {
        let mut xfa_field = XfaField::new("barcode", "binding");
        xfa_field.field_type = XfaFieldType::Barcode;

        let converter = XfaConverter::new();
        let rect = Rect::new(72.0, 700.0, 200.0, 100.0);

        let result = converter.convert_field(&xfa_field, rect);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_form() {
        let mut form = XfaForm::default();

        let mut field1 = XfaField::new("firstName", "form.firstName[0]");
        field1.field_type = XfaFieldType::Text;
        field1.value = Some("John".to_string());

        let mut field2 = XfaField::new("lastName", "form.lastName[0]");
        field2.field_type = XfaFieldType::Text;
        field2.value = Some("Doe".to_string());

        form.fields.push(field1);
        form.fields.push(field2);

        let converter = XfaConverter::new();
        let result = converter.convert(&form).unwrap();

        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.field_count, 2);
        assert!(result.skipped_fields.is_empty());
    }

    #[test]
    fn test_convert_form_with_pages() {
        let mut form = XfaForm::default();

        let mut page1 = XfaPage {
            name: "Page1".to_string(),
            ..Default::default()
        };
        page1.fields.push(XfaField::new("field1", "binding1"));

        let mut page2 = XfaPage {
            name: "Page2".to_string(),
            ..Default::default()
        };
        page2.fields.push(XfaField::new("field2", "binding2"));

        form.pages.push(page1);
        form.pages.push(page2);

        let converter = XfaConverter::new();
        let result = converter.convert(&form).unwrap();

        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.pages[0].index, 0);
        assert_eq!(result.pages[1].index, 1);
    }

    #[test]
    fn test_converted_page() {
        let page = ConvertedPage {
            index: 0,
            width: 612.0,
            height: 792.0,
            fields: Vec::new(),
            captions: vec![(Rect::new(72.0, 700.0, 100.0, 14.0), "Caption".to_string())],
        };

        assert_eq!(page.index, 0);
        assert_eq!(page.width, 612.0);
        assert_eq!(page.captions.len(), 1);
    }
}
