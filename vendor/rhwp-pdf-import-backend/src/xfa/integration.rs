//! XFA integration with PDF writer and editor.
//!
//! This module provides functions to integrate XFA conversion with
//! the PDF creation and editing workflow.

use super::converter::{ConvertedField, ConvertedPage, XfaConversionOptions, XfaConverter};
use super::extractor::XfaExtractor;
use super::parser::XfaParser;
use crate::document::PdfDocument;
use crate::error::Result;
use crate::writer::{PageBuilder, PdfWriter, PdfWriterConfig};

/// Add a converted field to a page builder.
///
/// This function takes a `ConvertedField` from XFA conversion and adds it
/// to a `PageBuilder` using the appropriate form field method.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::xfa::{XfaConverter, ConvertedField, add_converted_field};
/// use pdf_oxide::writer::PdfWriter;
///
/// let mut writer = PdfWriter::new();
/// let mut page = writer.add_page(612.0, 792.0);
///
/// // Add converted field to page
/// add_converted_field(&mut page, &converted_field);
/// ```
pub fn add_converted_field(page: &mut PageBuilder, field: &ConvertedField) {
    match field {
        ConvertedField::Text(widget) => {
            page.add_text_field(widget.clone());
        },
        ConvertedField::Checkbox(widget) => {
            page.add_checkbox(widget.clone());
        },
        ConvertedField::RadioGroup(group) => {
            page.add_radio_group(group.clone());
        },
        ConvertedField::ComboBox(widget) => {
            page.add_combo_box(widget.clone());
        },
        ConvertedField::ListBox(widget) => {
            page.add_list_box(widget.clone());
        },
        ConvertedField::Button(widget) => {
            page.add_push_button(widget.clone());
        },
    }
}

/// Add all fields from a converted page to a page builder.
///
/// This function takes a `ConvertedPage` from XFA conversion and adds all
/// its fields to a `PageBuilder`.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::xfa::{XfaConverter, add_converted_page};
/// use pdf_oxide::writer::PdfWriter;
///
/// let converter = XfaConverter::new();
/// let result = converter.convert(&xfa_form)?;
///
/// let mut writer = PdfWriter::new();
/// for converted_page in &result.pages {
///     let mut page = writer.add_page(converted_page.width, converted_page.height);
///     add_converted_page(&mut page, converted_page);
/// }
/// ```
pub fn add_converted_page(page: &mut PageBuilder, converted_page: &ConvertedPage) {
    // Add all form fields
    for field in &converted_page.fields {
        add_converted_field(page, field);
    }

    // Add captions as text
    for (rect, caption) in &converted_page.captions {
        page.add_text(
            caption,
            rect.x,
            rect.y,
            "Helvetica",
            12.0, // font size
        );
    }
}

/// Convert an XFA document to a new PDF with AcroForm fields.
///
/// This is the high-level function that takes a PDF document containing XFA forms,
/// extracts the XFA data, converts it to AcroForm, and creates a new PDF document
/// with the converted form fields.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::PdfDocument;
/// use pdf_oxide::xfa::{convert_xfa_document, XfaConversionOptions};
///
/// let mut doc = PdfDocument::open("xfa_form.pdf")?;
///
/// // Convert with default options
/// let pdf_bytes = convert_xfa_document(&mut doc, None)?;
/// std::fs::write("acroform.pdf", pdf_bytes)?;
///
/// // Or with custom options
/// let options = XfaConversionOptions {
///     page_width: 595.0,  // A4
///     page_height: 842.0,
///     ..Default::default()
/// };
/// let pdf_bytes = convert_xfa_document(&mut doc, Some(options))?;
/// ```
///
/// # Returns
///
/// Returns the PDF bytes of the converted document.
///
/// # Errors
///
/// Returns an error if:
/// - The document doesn't contain XFA forms
/// - XFA parsing fails
/// - Conversion fails
pub fn convert_xfa_document(
    doc: &mut PdfDocument,
    options: Option<XfaConversionOptions>,
) -> Result<Vec<u8>> {
    // Check if document has XFA
    if !XfaExtractor::has_xfa(doc)? {
        return Err(crate::error::Error::InvalidPdf(
            "Document does not contain XFA forms".to_string(),
        ));
    }

    // Extract XFA data
    let xfa_data = XfaExtractor::extract_xfa(doc)?;

    // Parse XFA
    let mut parser = XfaParser::new();
    let xfa_form = parser.parse(&xfa_data)?;

    // Convert to AcroForm
    let converter = if let Some(opts) = options {
        XfaConverter::with_options(opts)
    } else {
        XfaConverter::new()
    };
    let result = converter.convert(&xfa_form)?;

    // Create new document with converted fields using low-level API
    let mut config = PdfWriterConfig::default();
    if let Some(name) = &xfa_form.name {
        config = config.with_title(name);
    }
    let mut writer = PdfWriter::with_config(config);

    // Add pages with converted fields
    for converted_page in &result.pages {
        let mut page = writer.add_page(converted_page.width, converted_page.height);
        add_converted_page(&mut page, converted_page);
    }

    // Finish and return PDF bytes
    writer.finish()
}

/// Result of XFA document analysis.
#[derive(Debug, Clone)]
pub struct XfaAnalysis {
    /// Whether the document contains XFA forms
    pub has_xfa: bool,
    /// Number of fields found (if XFA present)
    pub field_count: Option<usize>,
    /// Number of pages found (if XFA present)
    pub page_count: Option<usize>,
    /// Field types found
    pub field_types: Vec<String>,
}

/// Analyze an XFA document without converting.
///
/// This function provides information about the XFA form structure
/// without performing the full conversion.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::PdfDocument;
/// use pdf_oxide::xfa::analyze_xfa_document;
///
/// let mut doc = PdfDocument::open("form.pdf")?;
/// let analysis = analyze_xfa_document(&mut doc)?;
///
/// if analysis.has_xfa {
///     println!("Found {} fields across {} pages",
///         analysis.field_count.unwrap_or(0),
///         analysis.page_count.unwrap_or(0));
/// }
/// ```
pub fn analyze_xfa_document(doc: &mut PdfDocument) -> Result<XfaAnalysis> {
    let has_xfa = XfaExtractor::has_xfa(doc)?;

    if !has_xfa {
        return Ok(XfaAnalysis {
            has_xfa: false,
            field_count: None,
            page_count: None,
            field_types: Vec::new(),
        });
    }

    // Extract and parse
    let xfa_data = XfaExtractor::extract_xfa(doc)?;
    let mut parser = XfaParser::new();
    let xfa_form = parser.parse(&xfa_data)?;

    // Collect field types
    let mut field_types: Vec<String> = xfa_form
        .fields
        .iter()
        .map(|f| format!("{:?}", f.field_type))
        .collect();
    field_types.sort();
    field_types.dedup();

    Ok(XfaAnalysis {
        has_xfa: true,
        field_count: Some(xfa_form.field_count()),
        page_count: Some(if xfa_form.pages.is_empty() {
            1
        } else {
            xfa_form.pages.len()
        }),
        field_types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::writer::form_fields::{CheckboxWidget, TextFieldWidget};
    use crate::xfa::{XfaField, XfaFieldType, XfaForm};

    #[test]
    fn test_add_converted_text_field() {
        let widget = TextFieldWidget::new("test", Rect::new(0.0, 0.0, 100.0, 20.0));
        let field = ConvertedField::Text(widget);

        let mut writer = PdfWriter::new();
        let mut page = writer.add_page(612.0, 792.0);

        // This should not panic
        add_converted_field(&mut page, &field);
    }

    #[test]
    fn test_add_converted_checkbox() {
        let widget = CheckboxWidget::new("check", Rect::new(0.0, 0.0, 20.0, 20.0));
        let field = ConvertedField::Checkbox(widget);

        let mut writer = PdfWriter::new();
        let mut page = writer.add_page(612.0, 792.0);

        add_converted_field(&mut page, &field);
    }

    #[test]
    fn test_add_converted_page() {
        let text_widget = TextFieldWidget::new("name", Rect::new(72.0, 700.0, 200.0, 20.0));
        let checkbox_widget = CheckboxWidget::new("agree", Rect::new(72.0, 650.0, 20.0, 20.0));

        let converted_page = ConvertedPage {
            index: 0,
            width: 612.0,
            height: 792.0,
            fields: vec![
                ConvertedField::Text(text_widget),
                ConvertedField::Checkbox(checkbox_widget),
            ],
            captions: vec![(Rect::new(72.0, 720.0, 100.0, 14.0), "Name:".to_string())],
        };

        let mut writer = PdfWriter::new();
        let mut page = writer.add_page(612.0, 792.0);

        add_converted_page(&mut page, &converted_page);
    }

    #[test]
    fn test_convert_xfa_form_to_pdf() {
        // Create a simple XFA form
        let mut form = XfaForm {
            name: Some("Test Form".to_string()),
            ..Default::default()
        };

        let mut field1 = XfaField::new("firstName", "form.firstName[0]");
        field1.field_type = XfaFieldType::Text;

        let mut field2 = XfaField::new("agree", "form.agree[0]");
        field2.field_type = XfaFieldType::Checkbox;

        form.fields.push(field1);
        form.fields.push(field2);

        // Convert
        let converter = XfaConverter::new();
        let result = converter.convert(&form).unwrap();

        // Build document using PdfWriter
        let mut writer = PdfWriter::new();

        for converted_page in &result.pages {
            let mut page = writer.add_page(converted_page.width, converted_page.height);
            add_converted_page(&mut page, converted_page);
        }

        // Should be able to finish without error
        let pdf_bytes = writer.finish().unwrap();
        assert!(!pdf_bytes.is_empty());

        // Verify structure
        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.field_count, 2);
    }
}
