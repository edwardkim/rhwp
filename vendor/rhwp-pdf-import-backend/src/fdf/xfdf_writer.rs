//! XFDF (XML Forms Data Format) writer implementation.
//!
//! Generates XFDF files according to Adobe XFDF Specification.

use crate::error::Result;
use crate::extractors::forms::FormField;
use crate::fdf::fdf_writer::{FdfField, FdfValue};
use std::path::Path;

/// XFDF file writer.
///
/// Generates XFDF (XML Forms Data Format) files for exporting form field data.
/// XFDF is an XML representation of FDF, useful for web integration and
/// human-readable data exchange.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::fdf::XfdfWriter;
///
/// let mut writer = XfdfWriter::new();
/// writer.add_field("name", "John Doe");
/// writer.add_field("email", "john@example.com");
/// writer.write_to_file("form_data.xfdf")?;
/// ```
#[derive(Debug, Default)]
pub struct XfdfWriter {
    /// Form fields to export
    fields: Vec<FdfField>,
    /// Original PDF file path (optional)
    file_spec: Option<String>,
}

impl XfdfWriter {
    /// Create a new XFDF writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an XFDF writer from extracted form fields.
    pub fn from_fields(fields: Vec<FormField>) -> Self {
        let fdf_fields: Vec<FdfField> = fields
            .into_iter()
            .map(|f| FdfField::new(f.full_name, FdfValue::from(&f.value)))
            .collect();

        Self {
            fields: fdf_fields,
            file_spec: None,
        }
    }

    /// Set the file specification (original PDF path).
    pub fn with_file_spec(mut self, path: impl Into<String>) -> Self {
        self.file_spec = Some(path.into());
        self
    }

    /// Add a text field to export.
    pub fn add_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.fields
            .push(FdfField::new(name, FdfValue::Text(value.into())));
    }

    /// Add an FDF field directly.
    pub fn add_fdf_field(&mut self, field: FdfField) {
        self.fields.push(field);
    }

    /// Write XFDF data to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let xml = self.to_xml();
        std::fs::write(path.as_ref(), xml)?;
        Ok(())
    }

    /// Generate XFDF XML string.
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();

        // XML declaration
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        // XFDF root element with namespace
        xml.push_str(r#"<xfdf xmlns="http://ns.adobe.com/xfdf/" xml:space="preserve">"#);
        xml.push('\n');

        // File reference (optional)
        if let Some(ref file_spec) = self.file_spec {
            xml.push_str(&format!("  <f href=\"{}\"/>\n", xml_escape(file_spec)));
        }

        // Fields container
        xml.push_str("  <fields>\n");

        for field in &self.fields {
            xml.push_str(&field_to_xml(field, 2));
        }

        xml.push_str("  </fields>\n");
        xml.push_str("</xfdf>\n");

        xml
    }

    /// Generate XFDF as bytes (UTF-8).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_xml().into_bytes()
    }
}

/// Convert an FDF field to XFDF XML.
fn field_to_xml(field: &FdfField, indent_level: usize) -> String {
    let indent = "  ".repeat(indent_level);
    let mut xml = String::new();

    xml.push_str(&format!("{}<field name=\"{}\">\n", indent, xml_escape(&field.name)));

    if field.kids.is_empty() {
        // Terminal field - output value
        let value_str = match &field.value {
            FdfValue::Text(s) => xml_escape(s),
            FdfValue::Boolean(b) => {
                if *b {
                    "Yes".to_string()
                } else {
                    "Off".to_string()
                }
            },
            FdfValue::Name(s) => xml_escape(s),
            FdfValue::Array(arr) => arr
                .iter()
                .map(|s| xml_escape(s))
                .collect::<Vec<_>>()
                .join(","),
            FdfValue::None => String::new(),
        };

        if !value_str.is_empty() {
            xml.push_str(&format!("{}  <value>{}</value>\n", indent, value_str));
        }
    } else {
        // Non-terminal field - recurse into kids
        for kid in &field.kids {
            xml.push_str(&field_to_xml(kid, indent_level + 1));
        }
    }

    xml.push_str(&format!("{}</field>\n", indent));
    xml
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("Hello"), "Hello");
        assert_eq!(xml_escape("<script>"), "&lt;script&gt;");
        assert_eq!(xml_escape("a&b"), "a&amp;b");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_xfdf_writer_basic() {
        let mut writer = XfdfWriter::new();
        writer.add_field("name", "John Doe");
        writer.add_field("email", "john@example.com");

        let xml = writer.to_xml();

        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<xfdf xmlns=\"http://ns.adobe.com/xfdf/\""));
        assert!(xml.contains("<fields>"));
        assert!(xml.contains("<field name=\"name\">"));
        assert!(xml.contains("<value>John Doe</value>"));
        assert!(xml.contains("<field name=\"email\">"));
        assert!(xml.contains("<value>john@example.com</value>"));
        assert!(xml.contains("</xfdf>"));
    }

    #[test]
    fn test_xfdf_with_file_spec() {
        let writer = XfdfWriter::new().with_file_spec("form.pdf");
        let xml = writer.to_xml();

        assert!(xml.contains("<f href=\"form.pdf\"/>"));
    }

    #[test]
    fn test_xfdf_escapes_special_chars() {
        let mut writer = XfdfWriter::new();
        writer.add_field("company", "Smith & Jones <Consulting>");

        let xml = writer.to_xml();

        assert!(xml.contains("<value>Smith &amp; Jones &lt;Consulting&gt;</value>"));
    }

    #[test]
    fn test_xfdf_boolean_field() {
        let mut writer = XfdfWriter::new();
        writer.add_fdf_field(FdfField::new("agree", FdfValue::Boolean(true)));
        writer.add_fdf_field(FdfField::new("decline", FdfValue::Boolean(false)));

        let xml = writer.to_xml();

        assert!(xml.contains("<field name=\"agree\">"));
        assert!(xml.contains("<value>Yes</value>"));
        assert!(xml.contains("<field name=\"decline\">"));
        assert!(xml.contains("<value>Off</value>"));
    }

    #[test]
    fn test_xfdf_hierarchical_field() {
        let mut writer = XfdfWriter::new();
        let parent = FdfField::new("address", FdfValue::None)
            .with_kid(FdfField::new("street", FdfValue::Text("123 Main St".into())))
            .with_kid(FdfField::new("city", FdfValue::Text("Anytown".into())));

        writer.add_fdf_field(parent);

        let xml = writer.to_xml();

        assert!(xml.contains("<field name=\"address\">"));
        assert!(xml.contains("<field name=\"street\">"));
        assert!(xml.contains("<value>123 Main St</value>"));
        assert!(xml.contains("<field name=\"city\">"));
        assert!(xml.contains("<value>Anytown</value>"));
    }
}
