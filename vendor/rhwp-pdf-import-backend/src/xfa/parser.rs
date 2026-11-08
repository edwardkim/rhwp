//! XFA (XML Forms Architecture) parser.
//!
//! This module parses XFA form data from PDF documents. XFA is an XML-based
//! form specification used in some PDFs, particularly government and financial forms.
//!
//! # XFA Packet Structure
//!
//! XFA forms contain several XML packets:
//! - **template**: Form structure and field definitions
//! - **datasets**: Form data values
//! - **config**: Configuration settings
//! - **form**: Runtime form state
//!
//! # Limitations
//!
//! This parser supports static XFA conversion only:
//! - Extracts field definitions and values
//! - Does NOT support dynamic XFA features (scripts, calculations)
//! - Does NOT support complex layouts (tables, grids)

use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// US Letter page width in points (8.5 inches × 72 points/inch).
const LETTER_WIDTH: f32 = 612.0;
/// US Letter page height in points (11 inches × 72 points/inch).
const LETTER_HEIGHT: f32 = 792.0;

/// XFA field type extracted from template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XfaFieldType {
    /// Text field (textEdit)
    Text,
    /// Numeric field (numericEdit)
    Numeric,
    /// Date field (dateTimeEdit)
    DateTime,
    /// Checkbox (checkButton)
    Checkbox,
    /// Radio button group (choiceList with appearance=minimal)
    RadioGroup,
    /// Drop-down list (choiceList)
    DropDown,
    /// List box (choiceList with appearance=full)
    ListBox,
    /// Button (button)
    Button,
    /// Signature field (signature)
    Signature,
    /// Image field (imageEdit)
    Image,
    /// Barcode field (barcode)
    Barcode,
    /// Unknown field type
    Unknown(String),
}

impl XfaFieldType {
    /// Parse from XFA UI element name.
    pub fn from_xfa_name(name: &str) -> Self {
        match name {
            "textEdit" => Self::Text,
            "numericEdit" => Self::Numeric,
            "dateTimeEdit" => Self::DateTime,
            "checkButton" => Self::Checkbox,
            "choiceList" => Self::DropDown, // Default, may be ListBox based on appearance
            "button" => Self::Button,
            "signature" => Self::Signature,
            "imageEdit" => Self::Image,
            "barcode" => Self::Barcode,
            _ => Self::Unknown(name.to_string()),
        }
    }
}

/// An option in a choice field (dropdown, list box, radio group).
#[derive(Debug, Clone)]
pub struct XfaOption {
    /// Display text
    pub text: String,
    /// Value when selected
    pub value: String,
}

/// A field extracted from XFA template.
#[derive(Debug, Clone)]
pub struct XfaField {
    /// Field name
    pub name: String,
    /// Full path/binding (e.g., `topmostSubform[0].Page1[0].field1[0]`)
    pub binding: String,
    /// Field type
    pub field_type: XfaFieldType,
    /// Tooltip/caption
    pub tooltip: Option<String>,
    /// Caption text
    pub caption: Option<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Current value from datasets
    pub value: Option<String>,
    /// Options for choice fields
    pub options: Vec<XfaOption>,
    /// Is field required
    pub required: bool,
    /// Is field read-only
    pub readonly: bool,
    /// Maximum length (for text fields)
    pub max_length: Option<u32>,
    /// Width hint
    pub width: Option<f32>,
    /// Height hint
    pub height: Option<f32>,
    /// X position hint
    pub x: Option<f32>,
    /// Y position hint
    pub y: Option<f32>,
}

impl XfaField {
    /// Create a new XFA field.
    pub fn new(name: impl Into<String>, binding: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            binding: binding.into(),
            field_type: XfaFieldType::Text,
            tooltip: None,
            caption: None,
            default_value: None,
            value: None,
            options: Vec::new(),
            required: false,
            readonly: false,
            max_length: None,
            width: None,
            height: None,
            x: None,
            y: None,
        }
    }
}

/// A page extracted from XFA template.
#[derive(Debug, Clone)]
pub struct XfaPage {
    /// Page name
    pub name: String,
    /// Fields on this page
    pub fields: Vec<XfaField>,
    /// Width in points
    pub width: f32,
    /// Height in points
    pub height: f32,
}

impl Default for XfaPage {
    fn default() -> Self {
        Self {
            name: String::new(),
            fields: Vec::new(),
            width: LETTER_WIDTH,
            height: LETTER_HEIGHT,
        }
    }
}

/// Parsed XFA form.
#[derive(Debug, Clone, Default)]
pub struct XfaForm {
    /// Form name/title
    pub name: Option<String>,
    /// Pages in the form
    pub pages: Vec<XfaPage>,
    /// All fields (flattened)
    pub fields: Vec<XfaField>,
    /// Field values from datasets (binding -> value)
    pub dataset_values: HashMap<String, String>,
    /// Raw template XML (for debugging)
    pub template_xml: Option<String>,
    /// Raw datasets XML (for debugging)
    pub datasets_xml: Option<String>,
}

impl XfaForm {
    /// Get a field by name.
    pub fn get_field(&self, name: &str) -> Option<&XfaField> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get a field by binding path.
    pub fn get_field_by_binding(&self, binding: &str) -> Option<&XfaField> {
        self.fields.iter().find(|f| f.binding == binding)
    }

    /// Get total number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

/// XFA parser.
pub struct XfaParser {
    /// Parsed form
    form: XfaForm,
    /// Current parsing context stack (element names)
    context_stack: Vec<String>,
    /// Current binding path parts
    binding_parts: Vec<String>,
}

impl Default for XfaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl XfaParser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self {
            form: XfaForm::default(),
            context_stack: Vec::new(),
            binding_parts: Vec::new(),
        }
    }

    /// Parse XFA data from raw bytes.
    ///
    /// The XFA data may be a single XML stream or an array of packets.
    pub fn parse(&mut self, xfa_data: &[u8]) -> Result<XfaForm> {
        // Try to parse as XML
        let xml_str = String::from_utf8_lossy(xfa_data);
        self.form.template_xml = Some(xml_str.to_string());

        // Check if this is a complete XDP (XFA Data Package)
        if xml_str.contains("<xdp:xdp") || xml_str.contains("<xfa:datasets") {
            self.parse_xdp(&xml_str)?;
        } else if xml_str.contains("<template") {
            // Just a template packet
            self.parse_template(&xml_str)?;
        } else if xml_str.contains("<xfa:data") || xml_str.contains("<data>") {
            // Just a datasets packet
            self.parse_datasets(&xml_str)?;
        }

        // Apply dataset values to fields
        self.apply_dataset_values();

        Ok(std::mem::take(&mut self.form))
    }

    /// Parse XFA packets from a PDF XFA array.
    ///
    /// XFA in PDFs can be an array: [name1, stream1, name2, stream2, ...]
    pub fn parse_packets(&mut self, packets: &[(String, Vec<u8>)]) -> Result<XfaForm> {
        for (name, data) in packets {
            let xml_str = String::from_utf8_lossy(data);

            match name.as_str() {
                "template" => {
                    self.form.template_xml = Some(xml_str.to_string());
                    self.parse_template(&xml_str)?;
                },
                "datasets" => {
                    self.form.datasets_xml = Some(xml_str.to_string());
                    self.parse_datasets(&xml_str)?;
                },
                // Other packets (config, form, etc.) are not needed for static conversion
                _ => {},
            }
        }

        // Apply dataset values to fields
        self.apply_dataset_values();

        Ok(std::mem::take(&mut self.form))
    }

    /// Parse a complete XDP document.
    fn parse_xdp(&mut self, xml: &str) -> Result<()> {
        // Extract template section
        if let Some(template_start) = xml.find("<template") {
            if let Some(template_end) = xml.find("</template>") {
                let template_xml = &xml[template_start..template_end + 11];
                self.parse_template(template_xml)?;
            }
        }

        // Extract datasets section
        if let Some(data_start) = xml.find("<xfa:datasets") {
            if let Some(data_end) = xml.find("</xfa:datasets>") {
                let datasets_xml = &xml[data_start..data_end + 15];
                self.form.datasets_xml = Some(datasets_xml.to_string());
                self.parse_datasets(datasets_xml)?;
            }
        }

        Ok(())
    }

    /// Parse the template packet.
    fn parse_template(&mut self, xml: &str) -> Result<()> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut current_field: Option<XfaField> = None;
        let mut current_page: Option<XfaPage> = None;
        let mut in_items = false;
        let mut current_option_text = String::new();
        let mut current_option_value = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = e.local_name().as_ref().to_string();

                    match local_name.as_str() {
                        "subform" => {
                            // Check if this is a page-level subform
                            let name = self.get_attribute(e, "name");
                            if let Some(name) = name {
                                self.binding_parts.push(format!("{}[0]", name));

                                // Check for page-like subforms
                                if name.contains("Page") || current_page.is_none() {
                                    current_page = Some(XfaPage {
                                        name,
                                        ..Default::default()
                                    });
                                }
                            }
                        },
                        "field" => {
                            let name = self.get_attribute(e, "name").unwrap_or_default();
                            let mut binding = self.binding_parts.join(".");
                            if !binding.is_empty() {
                                binding.push('.');
                            }
                            binding.push_str(&format!("{}[0]", name));

                            let mut field = XfaField::new(&name, binding);

                            // Get dimensions from attributes
                            if let Some(w) = self.get_attribute(e, "w") {
                                field.width = self.parse_dimension(&w);
                            }
                            if let Some(h) = self.get_attribute(e, "h") {
                                field.height = self.parse_dimension(&h);
                            }
                            if let Some(x) = self.get_attribute(e, "x") {
                                field.x = self.parse_dimension(&x);
                            }
                            if let Some(y) = self.get_attribute(e, "y") {
                                field.y = self.parse_dimension(&y);
                            }

                            current_field = Some(field);
                        },
                        "textEdit" | "numericEdit" | "dateTimeEdit" | "checkButton"
                        | "choiceList" | "button" | "signature" | "imageEdit" | "barcode" => {
                            if let Some(ref mut field) = current_field {
                                field.field_type = XfaFieldType::from_xfa_name(&local_name);
                            }
                        },
                        "items" => {
                            in_items = true;
                        },
                        "text" => {
                            // Will capture text content
                        },
                        "caption" => {
                            // Caption element
                        },
                        "toolTip" => {
                            // Tooltip element
                        },
                        _ => {},
                    }

                    self.context_stack.push(local_name);
                },
                Ok(Event::Text(e)) => {
                    let text = e.xml11_content().to_string();

                    if let Some(parent) = self.context_stack.last() {
                        if in_items && parent == "text" {
                            current_option_text = text.clone();
                            current_option_value = text;
                        } else if let Some(ref mut field) = current_field {
                            match parent.as_str() {
                                "caption" | "text" => {
                                    if field.caption.is_none()
                                        && !in_items
                                        && self.context_stack.iter().any(|s| s == "caption")
                                    {
                                        field.caption = Some(text);
                                    }
                                },
                                "toolTip" => {
                                    field.tooltip = Some(text);
                                },
                                "value" => {
                                    if field.default_value.is_none() {
                                        field.default_value = Some(text);
                                    }
                                },
                                _ => {},
                            }
                        }
                    }
                },
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name().as_ref().to_string();

                    match local_name.as_str() {
                        "subform" => {
                            self.binding_parts.pop();
                        },
                        "field" => {
                            if let Some(field) = current_field.take() {
                                self.form.fields.push(field.clone());
                                if let Some(ref mut page) = current_page {
                                    page.fields.push(field);
                                }
                            }
                        },
                        "items" => {
                            in_items = false;
                        },
                        "text" => {
                            if in_items {
                                if let Some(ref mut field) = current_field {
                                    field.options.push(XfaOption {
                                        text: current_option_text.clone(),
                                        value: current_option_value.clone(),
                                    });
                                }
                                current_option_text.clear();
                                current_option_value.clear();
                            }
                        },
                        _ => {},
                    }

                    self.context_stack.pop();
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(Error::InvalidPdf(format!("XFA template parse error: {}", e)));
                },
                _ => {},
            }
        }

        // Add the last page if any
        if let Some(page) = current_page {
            if !page.fields.is_empty() {
                self.form.pages.push(page);
            }
        }

        Ok(())
    }

    /// Parse the datasets packet.
    fn parse_datasets(&mut self, xml: &str) -> Result<()> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut path_stack: Vec<String> = Vec::new();
        let mut current_text = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name().as_ref().to_string();

                    // Skip XFA namespace elements
                    if local_name != "xfa" && local_name != "datasets" && local_name != "data" {
                        path_stack.push(format!("{}[0]", local_name));
                    }
                },
                Ok(Event::Text(e)) => {
                    current_text = e.xml11_content().to_string();
                },
                Ok(Event::End(ref e)) => {
                    let local_name = e.local_name().as_ref().to_string();

                    if local_name != "xfa" && local_name != "datasets" && local_name != "data" {
                        // Store value if we have text
                        if !current_text.is_empty() && !path_stack.is_empty() {
                            let binding = path_stack.join(".");
                            self.form
                                .dataset_values
                                .insert(binding, current_text.clone());
                        }
                        current_text.clear();
                        path_stack.pop();
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(Error::InvalidPdf(format!("XFA datasets parse error: {}", e)));
                },
                _ => {},
            }
        }

        Ok(())
    }

    /// Apply dataset values to fields.
    fn apply_dataset_values(&mut self) {
        for field in &mut self.form.fields {
            // Try exact binding match
            if let Some(value) = self.form.dataset_values.get(&field.binding) {
                field.value = Some(value.clone());
            } else {
                // Try partial match (XFA bindings can be complex)
                for (binding, value) in &self.form.dataset_values {
                    if binding.ends_with(&format!(".{}[0]", field.name)) {
                        field.value = Some(value.clone());
                        break;
                    }
                }
            }
        }

        // Also update page field values
        for page in &mut self.form.pages {
            for field in &mut page.fields {
                if let Some(value) = self.form.dataset_values.get(&field.binding) {
                    field.value = Some(value.clone());
                } else {
                    for (binding, value) in &self.form.dataset_values {
                        if binding.ends_with(&format!(".{}[0]", field.name)) {
                            field.value = Some(value.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Get an attribute value from an element.
    fn get_attribute<'a>(
        &self,
        e: &'a quick_xml::events::BytesStart<'a>,
        name: &str,
    ) -> Option<String> {
        for attr in e.attributes().flatten() {
            if attr.key.as_ref() == name {
                return Some(attr.value.to_string());
            }
        }
        None
    }

    /// Parse a dimension value (e.g., "72pt", "1in", "2.5cm").
    fn parse_dimension(&self, value: &str) -> Option<f32> {
        let value = value.trim();

        if let Some(stripped) = value.strip_suffix("pt") {
            stripped.parse().ok()
        } else if let Some(stripped) = value.strip_suffix("in") {
            stripped.parse::<f32>().ok().map(|v| v * 72.0)
        } else if let Some(stripped) = value.strip_suffix("cm") {
            stripped.parse::<f32>().ok().map(|v| v * 72.0 / 2.54)
        } else if let Some(stripped) = value.strip_suffix("mm") {
            stripped.parse::<f32>().ok().map(|v| v * 72.0 / 25.4)
        } else {
            // Assume points
            value.parse().ok()
        }
    }
}

/// Check if data appears to be XFA content.
pub fn is_xfa_data(data: &[u8]) -> bool {
    let s = String::from_utf8_lossy(data);
    s.contains("<template") || s.contains("<xfa:") || s.contains("<xdp:xdp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xfa_field_type_from_name() {
        assert_eq!(XfaFieldType::from_xfa_name("textEdit"), XfaFieldType::Text);
        assert_eq!(XfaFieldType::from_xfa_name("numericEdit"), XfaFieldType::Numeric);
        assert_eq!(XfaFieldType::from_xfa_name("checkButton"), XfaFieldType::Checkbox);
        assert_eq!(XfaFieldType::from_xfa_name("choiceList"), XfaFieldType::DropDown);
        assert!(matches!(XfaFieldType::from_xfa_name("unknown"), XfaFieldType::Unknown(_)));
    }

    #[test]
    fn test_xfa_field_new() {
        let field = XfaField::new("firstName", "form.page1.firstName[0]");

        assert_eq!(field.name, "firstName");
        assert_eq!(field.binding, "form.page1.firstName[0]");
        assert_eq!(field.field_type, XfaFieldType::Text);
        assert!(field.value.is_none());
    }

    #[test]
    fn test_xfa_page_default() {
        let page = XfaPage::default();

        assert!(page.name.is_empty());
        assert!(page.fields.is_empty());
        assert_eq!(page.width, 612.0);
        assert_eq!(page.height, 792.0);
    }

    #[test]
    fn test_xfa_form_get_field() {
        let mut form = XfaForm::default();
        form.fields.push(XfaField::new("test", "binding"));

        assert!(form.get_field("test").is_some());
        assert!(form.get_field("nonexistent").is_none());
    }

    #[test]
    fn test_xfa_form_get_field_by_binding() {
        let mut form = XfaForm::default();
        form.fields.push(XfaField::new("test", "root.test[0]"));

        assert!(form.get_field_by_binding("root.test[0]").is_some());
        assert!(form.get_field_by_binding("other").is_none());
    }

    #[test]
    fn test_parser_dimension_parsing() {
        let parser = XfaParser::new();

        assert_eq!(parser.parse_dimension("72pt"), Some(72.0));
        assert_eq!(parser.parse_dimension("1in"), Some(72.0));
        assert_eq!(parser.parse_dimension("2.54cm"), Some(72.0));
        assert_eq!(parser.parse_dimension("25.4mm"), Some(72.0));
        assert_eq!(parser.parse_dimension("100"), Some(100.0));
    }

    #[test]
    fn test_is_xfa_data() {
        assert!(is_xfa_data(b"<template xmlns=\"http://www.xfa.org/schema/xfa-template/2.8/\">"));
        assert!(is_xfa_data(b"<xfa:datasets>"));
        assert!(is_xfa_data(b"<xdp:xdp xmlns:xdp=\"http://ns.adobe.com/xdp/\">"));
        assert!(!is_xfa_data(b"<html><body>Not XFA</body></html>"));
        assert!(!is_xfa_data(b"Random data"));
    }

    #[test]
    fn test_parse_simple_template() {
        let template = r#"<?xml version="1.0"?>
<template xmlns="http://www.xfa.org/schema/xfa-template/3.0/">
    <subform name="form1">
        <field name="firstName" w="200pt" h="20pt">
            <ui>
                <textEdit/>
            </ui>
            <caption>
                <value><text>First Name</text></value>
            </caption>
        </field>
        <field name="lastName">
            <ui>
                <textEdit/>
            </ui>
        </field>
    </subform>
</template>"#;

        let mut parser = XfaParser::new();
        let form = parser.parse(template.as_bytes()).unwrap();

        assert_eq!(form.fields.len(), 2);
        assert_eq!(form.fields[0].name, "firstName");
        assert_eq!(form.fields[0].field_type, XfaFieldType::Text);
        assert_eq!(form.fields[0].width, Some(200.0));
        assert_eq!(form.fields[0].height, Some(20.0));
        assert_eq!(form.fields[1].name, "lastName");
    }

    #[test]
    fn test_parse_datasets() {
        let datasets = r#"<?xml version="1.0"?>
<xfa:datasets xmlns:xfa="http://www.xfa.org/schema/xfa-data/1.0/">
    <xfa:data>
        <form1>
            <firstName>John</firstName>
            <lastName>Doe</lastName>
        </form1>
    </xfa:data>
</xfa:datasets>"#;

        let mut parser = XfaParser::new();

        // First add some fields
        parser
            .form
            .fields
            .push(XfaField::new("firstName", "form1[0].firstName[0]"));
        parser
            .form
            .fields
            .push(XfaField::new("lastName", "form1[0].lastName[0]"));

        // Parse datasets and apply values
        parser.parse_datasets(datasets).unwrap();
        parser.apply_dataset_values();

        assert_eq!(parser.form.fields[0].value, Some("John".to_string()));
        assert_eq!(parser.form.fields[1].value, Some("Doe".to_string()));
    }

    #[test]
    fn test_parse_field_with_options() {
        let template = r#"<?xml version="1.0"?>
<template>
    <subform name="form1">
        <field name="country">
            <ui>
                <choiceList/>
            </ui>
            <items>
                <text>United States</text>
                <text>Canada</text>
                <text>Mexico</text>
            </items>
        </field>
    </subform>
</template>"#;

        let mut parser = XfaParser::new();
        let form = parser.parse(template.as_bytes()).unwrap();

        assert_eq!(form.fields.len(), 1);
        assert_eq!(form.fields[0].name, "country");
        assert_eq!(form.fields[0].field_type, XfaFieldType::DropDown);
        assert_eq!(form.fields[0].options.len(), 3);
        assert_eq!(form.fields[0].options[0].text, "United States");
    }

    #[test]
    fn test_parse_checkbox_field() {
        let template = r#"<?xml version="1.0"?>
<template>
    <subform name="form1">
        <field name="agree">
            <ui>
                <checkButton/>
            </ui>
            <caption>
                <value><text>I agree to the terms</text></value>
            </caption>
        </field>
    </subform>
</template>"#;

        let mut parser = XfaParser::new();
        let form = parser.parse(template.as_bytes()).unwrap();

        assert_eq!(form.fields.len(), 1);
        assert_eq!(form.fields[0].name, "agree");
        assert_eq!(form.fields[0].field_type, XfaFieldType::Checkbox);
    }

    #[test]
    fn test_xfa_option() {
        let option = XfaOption {
            text: "Display Text".to_string(),
            value: "actual_value".to_string(),
        };

        assert_eq!(option.text, "Display Text");
        assert_eq!(option.value, "actual_value");
    }
}
