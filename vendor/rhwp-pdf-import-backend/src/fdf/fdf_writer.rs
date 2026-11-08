//! FDF (Forms Data Format) writer implementation.
//!
//! Generates FDF files according to ISO 32000-1:2008 Section 12.7.7.

use crate::error::Result;
use crate::extractors::forms::{FieldValue, FormField};
use std::io::Write;
use std::path::Path;

/// A form field for FDF export.
#[derive(Debug, Clone)]
pub struct FdfField {
    /// Full qualified field name
    pub name: String,
    /// Field value
    pub value: FdfValue,
    /// Child fields (for hierarchical structure)
    pub kids: Vec<FdfField>,
}

/// Value types for FDF fields.
#[derive(Debug, Clone)]
pub enum FdfValue {
    /// Text string value
    Text(String),
    /// Boolean value (for checkboxes)
    Boolean(bool),
    /// Name value (for radio buttons, choice fields)
    Name(String),
    /// Array of values (for multi-select)
    Array(Vec<String>),
    /// No value
    None,
}

impl From<&FieldValue> for FdfValue {
    fn from(value: &FieldValue) -> Self {
        match value {
            FieldValue::Text(s) => FdfValue::Text(s.clone()),
            FieldValue::Boolean(b) => FdfValue::Boolean(*b),
            FieldValue::Name(s) => FdfValue::Name(s.clone()),
            FieldValue::Array(arr) => FdfValue::Array(arr.clone()),
            FieldValue::None => FdfValue::None,
        }
    }
}

impl FdfField {
    /// Create a new FDF field.
    pub fn new(name: impl Into<String>, value: FdfValue) -> Self {
        Self {
            name: name.into(),
            value,
            kids: Vec::new(),
        }
    }

    /// Add a child field.
    pub fn with_kid(mut self, kid: FdfField) -> Self {
        self.kids.push(kid);
        self
    }

    /// Convert to FDF dictionary string.
    fn to_fdf_dict(&self) -> String {
        let mut dict = String::new();
        dict.push_str("<< ");

        // Field name /T
        dict.push_str("/T ");
        dict.push_str(&encode_pdf_string(&self.name));

        // Field value /V
        if !matches!(self.value, FdfValue::None) {
            dict.push_str(" /V ");
            dict.push_str(&self.value.to_fdf_value());
        }

        // Kids array /Kids
        if !self.kids.is_empty() {
            dict.push_str(" /Kids [ ");
            for kid in &self.kids {
                dict.push_str(&kid.to_fdf_dict());
                dict.push(' ');
            }
            dict.push(']');
        }

        dict.push_str(" >>");
        dict
    }
}

impl FdfValue {
    fn to_fdf_value(&self) -> String {
        match self {
            FdfValue::Text(s) => encode_pdf_string(s),
            FdfValue::Boolean(b) => {
                if *b {
                    "/Yes".to_string()
                } else {
                    "/Off".to_string()
                }
            },
            FdfValue::Name(s) => format!("/{}", s),
            FdfValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|s| encode_pdf_string(s)).collect();
                format!("[ {} ]", items.join(" "))
            },
            FdfValue::None => "null".to_string(),
        }
    }
}

/// Encode a string as a PDF literal string.
fn encode_pdf_string(s: &str) -> String {
    let mut encoded = String::from("(");
    for c in s.chars() {
        match c {
            '(' => encoded.push_str("\\("),
            ')' => encoded.push_str("\\)"),
            '\\' => encoded.push_str("\\\\"),
            '\r' => encoded.push_str("\\r"),
            '\n' => encoded.push_str("\\n"),
            '\t' => encoded.push_str("\\t"),
            _ => encoded.push(c),
        }
    }
    encoded.push(')');
    encoded
}

/// FDF file writer.
///
/// Generates FDF (Forms Data Format) files for exporting form field data.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::fdf::{FdfWriter, FdfField, FdfValue};
///
/// let mut writer = FdfWriter::new();
/// writer.add_field(FdfField::new("name", FdfValue::Text("John Doe".into())));
/// writer.add_field(FdfField::new("email", FdfValue::Text("john@example.com".into())));
/// writer.write_to_file("form_data.fdf")?;
/// ```
#[derive(Debug, Default)]
pub struct FdfWriter {
    /// Form fields to export
    fields: Vec<FdfField>,
    /// Original PDF file path (optional, for /F entry)
    file_spec: Option<String>,
}

impl FdfWriter {
    /// Create a new FDF writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an FDF writer from extracted form fields.
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

    /// Add a field to export.
    pub fn add_field(&mut self, field: FdfField) {
        self.fields.push(field);
    }

    /// Write FDF data to a file.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path.as_ref(), bytes)?;
        Ok(())
    }

    /// Generate FDF data as bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        // FDF header
        writeln!(output, "%FDF-1.2")?;
        // Binary marker (high-bit bytes to indicate binary file)
        output.write_all(b"%")?;
        output.write_all(&[0xe2, 0xe3, 0xcf, 0xd3])?;
        writeln!(output)?;

        // FDF catalog object
        writeln!(output, "1 0 obj")?;
        writeln!(output, "<<")?;
        writeln!(output, "/FDF <<")?;

        // File specification (optional)
        if let Some(ref file_spec) = self.file_spec {
            writeln!(output, "/F {}", encode_pdf_string(file_spec))?;
        }

        // Fields array
        writeln!(output, "/Fields [")?;
        for field in &self.fields {
            writeln!(output, "{}", field.to_fdf_dict())?;
        }
        writeln!(output, "]")?;

        writeln!(output, ">>")?;
        writeln!(output, ">>")?;
        writeln!(output, "endobj")?;

        // Trailer
        writeln!(output, "trailer")?;
        writeln!(output, "<< /Root 1 0 R >>")?;
        writeln!(output, "%%EOF")?;

        Ok(output)
    }

    /// Generate FDF data as a string.
    pub fn to_string(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_pdf_string() {
        assert_eq!(encode_pdf_string("Hello"), "(Hello)");
        assert_eq!(encode_pdf_string("Hello (World)"), "(Hello \\(World\\))");
        assert_eq!(encode_pdf_string("Line1\nLine2"), "(Line1\\nLine2)");
    }

    #[test]
    fn test_fdf_field_to_dict() {
        let field = FdfField::new("name", FdfValue::Text("John".into()));
        let dict = field.to_fdf_dict();
        assert!(dict.contains("/T (name)"));
        assert!(dict.contains("/V (John)"));
    }

    #[test]
    fn test_fdf_writer_to_bytes() {
        let mut writer = FdfWriter::new();
        writer.add_field(FdfField::new("test", FdfValue::Text("value".into())));

        let bytes = writer.to_bytes().unwrap();
        let content = String::from_utf8_lossy(&bytes);

        assert!(content.contains("%FDF-1.2"));
        assert!(content.contains("/Fields"));
        assert!(content.contains("/T (test)"));
        assert!(content.contains("/V (value)"));
        assert!(content.contains("%%EOF"));
    }

    #[test]
    fn test_fdf_boolean_value() {
        let field_yes = FdfField::new("check", FdfValue::Boolean(true));
        let dict_yes = field_yes.to_fdf_dict();
        assert!(dict_yes.contains("/V /Yes"));

        let field_no = FdfField::new("check", FdfValue::Boolean(false));
        let dict_no = field_no.to_fdf_dict();
        assert!(dict_no.contains("/V /Off"));
    }
}
