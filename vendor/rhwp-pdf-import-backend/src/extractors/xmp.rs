//! XMP metadata extraction from PDF documents.
//!
//! Extracts XMP (Extensible Metadata Platform) metadata from PDF documents.
//! XMP is XML-based metadata that provides richer information than the
//! traditional Info dictionary. See ISO 32000-1:2008, Section 14.3.2.
//!
//! ## XMP Namespaces
//!
//! XMP uses several standard namespaces:
//! - Dublin Core (dc): title, creator, description, etc.
//! - XMP Core (xmp): creation date, modify date, creator tool
//! - PDF (pdf): producer, keywords, trapped
//! - XMP Rights (xmpRights): usage terms, copyright

use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::object::Object;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// XMP metadata extracted from a PDF document.
#[derive(Debug, Clone, Default)]
pub struct XmpMetadata {
    // Dublin Core namespace (dc:)
    /// Document title (dc:title)
    pub dc_title: Option<String>,
    /// Document creators/authors (dc:creator)
    pub dc_creator: Vec<String>,
    /// Document description (dc:description)
    pub dc_description: Option<String>,
    /// Subject keywords (dc:subject)
    pub dc_subject: Vec<String>,
    /// Document language (dc:language)
    pub dc_language: Option<String>,
    /// Copyright (dc:rights)
    pub dc_rights: Option<String>,
    /// Document format (dc:format)
    pub dc_format: Option<String>,

    // XMP Core namespace (xmp:)
    /// Tool used to create the document (xmp:CreatorTool)
    pub xmp_creator_tool: Option<String>,
    /// Creation date (xmp:CreateDate)
    pub xmp_create_date: Option<String>,
    /// Last modification date (xmp:ModifyDate)
    pub xmp_modify_date: Option<String>,
    /// Metadata modification date (xmp:MetadataDate)
    pub xmp_metadata_date: Option<String>,

    // PDF namespace (pdf:)
    /// PDF producer (pdf:Producer)
    pub pdf_producer: Option<String>,
    /// PDF keywords (pdf:Keywords)
    pub pdf_keywords: Option<String>,
    /// PDF version (pdf:PDFVersion)
    pub pdf_version: Option<String>,
    /// Whether the document has been trapped (pdf:Trapped)
    pub pdf_trapped: Option<String>,

    // XMP Rights namespace (xmpRights:)
    /// Usage terms (xmpRights:UsageTerms)
    pub xmp_rights_usage_terms: Option<String>,
    /// Whether marked with rights (xmpRights:Marked)
    pub xmp_rights_marked: Option<bool>,
    /// Web statement URL (xmpRights:WebStatement)
    pub xmp_rights_web_statement: Option<String>,

    // Custom/unrecognized properties
    /// Custom properties (namespace:property -> value)
    pub custom: HashMap<String, String>,

    /// Raw XMP packet (the original XML)
    pub raw_xml: Option<String>,
}

impl XmpMetadata {
    /// Create a new empty XMP metadata instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any metadata is present.
    pub fn is_empty(&self) -> bool {
        self.dc_title.is_none()
            && self.dc_creator.is_empty()
            && self.dc_description.is_none()
            && self.dc_subject.is_empty()
            && self.xmp_creator_tool.is_none()
            && self.xmp_create_date.is_none()
            && self.xmp_modify_date.is_none()
            && self.pdf_producer.is_none()
            && self.pdf_keywords.is_none()
            && self.custom.is_empty()
    }

    /// Set the document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.dc_title = Some(title.into());
        self
    }

    /// Add a creator/author.
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.dc_creator.push(creator.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.dc_description = Some(desc.into());
        self
    }

    /// Set the creator tool.
    pub fn with_creator_tool(mut self, tool: impl Into<String>) -> Self {
        self.xmp_creator_tool = Some(tool.into());
        self
    }

    /// Set the creation date (ISO 8601 format).
    pub fn with_create_date(mut self, date: impl Into<String>) -> Self {
        self.xmp_create_date = Some(date.into());
        self
    }

    /// Set the modification date (ISO 8601 format).
    pub fn with_modify_date(mut self, date: impl Into<String>) -> Self {
        self.xmp_modify_date = Some(date.into());
        self
    }

    /// Set the PDF producer.
    pub fn with_producer(mut self, producer: impl Into<String>) -> Self {
        self.pdf_producer = Some(producer.into());
        self
    }

    /// Add a custom property.
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// XMP metadata extractor.
pub struct XmpExtractor;

impl XmpExtractor {
    /// Helper function to resolve an Object (handles indirect references).
    fn resolve_object(doc: &PdfDocument, obj: &Object) -> Result<Object> {
        if let Some(ref_val) = obj.as_reference() {
            doc.load_object(ref_val)
        } else {
            Ok(obj.clone())
        }
    }

    /// Extract XMP metadata from a PDF document.
    ///
    /// # Arguments
    ///
    /// * `doc` - The PDF document to extract XMP metadata from
    ///
    /// # Returns
    ///
    /// XMP metadata if present, or None if no XMP metadata exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pdf_oxide::document::PdfDocument;
    /// use pdf_oxide::extractors::xmp::XmpExtractor;
    ///
    /// let mut doc = PdfDocument::open("document.pdf")?;
    /// if let Some(xmp) = XmpExtractor::extract(&mut doc)? {
    ///     if let Some(title) = &xmp.dc_title {
    ///         println!("Title: {}", title);
    ///     }
    ///     for creator in &xmp.dc_creator {
    ///         println!("Author: {}", creator);
    ///     }
    /// }
    /// # Ok::<(), pdf_oxide::error::Error>(())
    /// ```
    pub fn extract(doc: &PdfDocument) -> Result<Option<XmpMetadata>> {
        // Get document catalog
        let catalog = doc.catalog()?;
        let catalog_dict = catalog
            .as_dict()
            .ok_or_else(|| Error::InvalidPdf("Catalog is not a dictionary".to_string()))?;

        // Check if Metadata stream exists
        let metadata_obj = match catalog_dict.get("Metadata") {
            Some(obj) => obj.clone(),
            None => return Ok(None),
        };

        // Resolve the metadata stream
        let metadata_resolved = Self::resolve_object(doc, &metadata_obj)?;

        // Get stream data
        let xml_bytes = match &metadata_resolved {
            Object::Stream { .. } => {
                // XMP metadata streams are typically not filtered (or use FlateDecode)
                // Try to decode the stream
                let decoded = metadata_resolved.decode_stream_data()?;
                decoded.to_vec()
            },
            _ => return Err(Error::InvalidPdf("Metadata is not a stream".to_string())),
        };

        // Convert to string
        let xml_str = String::from_utf8_lossy(&xml_bytes).to_string();

        // Parse the XMP XML
        Self::parse_xmp(&xml_str)
    }

    /// Parse XMP XML content.
    pub fn parse_xmp(xml: &str) -> Result<Option<XmpMetadata>> {
        // Find the XMP packet boundaries
        let start = xml.find("<x:xmpmeta").or_else(|| xml.find("<rdf:RDF"));
        let end = xml
            .rfind("</x:xmpmeta>")
            .or_else(|| xml.rfind("</rdf:RDF>"));

        let xmp_content = match (start, end) {
            (Some(s), Some(e)) => {
                let end_adjusted = if xml[e..].starts_with("</x:xmpmeta") {
                    e + "</x:xmpmeta>".len()
                } else {
                    e + "</rdf:RDF>".len()
                };
                &xml[s..end_adjusted]
            },
            _ => return Ok(None),
        };

        let mut metadata = XmpMetadata::new();
        metadata.raw_xml = Some(xml.to_string());

        let mut reader = Reader::from_str(xmp_content);
        reader.config_mut().trim_text(true);

        // Stack to track element hierarchy
        let mut element_stack: Vec<String> = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let name = e.name().as_ref().to_string();
                    element_stack.push(name);
                },
                Ok(Event::Empty(_)) => {
                    // Empty elements don't have text content
                },
                Ok(Event::Text(e)) => {
                    let text = e.xml11_content().trim().to_string();
                    if text.is_empty() {
                        continue;
                    }

                    // Find the relevant property element (skip rdf:li, rdf:Seq, rdf:Bag, rdf:Alt)
                    let property = element_stack
                        .iter()
                        .rev()
                        .find(|el| !el.starts_with("rdf:") && !el.starts_with("x:"))
                        .cloned();

                    if let Some(prop) = property {
                        // Map to appropriate field based on property element
                        match prop.as_str() {
                            // Dublin Core
                            "dc:title" => {
                                if metadata.dc_title.is_none() {
                                    metadata.dc_title = Some(text);
                                }
                            },
                            "dc:creator" => {
                                metadata.dc_creator.push(text);
                            },
                            "dc:description" => {
                                if metadata.dc_description.is_none() {
                                    metadata.dc_description = Some(text);
                                }
                            },
                            "dc:subject" => {
                                metadata.dc_subject.push(text);
                            },
                            "dc:language" => metadata.dc_language = Some(text),
                            "dc:rights" => {
                                if metadata.dc_rights.is_none() {
                                    metadata.dc_rights = Some(text);
                                }
                            },
                            "dc:format" => metadata.dc_format = Some(text),

                            // XMP Core
                            "xmp:CreatorTool" => metadata.xmp_creator_tool = Some(text),
                            "xmp:CreateDate" => metadata.xmp_create_date = Some(text),
                            "xmp:ModifyDate" => metadata.xmp_modify_date = Some(text),
                            "xmp:MetadataDate" => metadata.xmp_metadata_date = Some(text),

                            // PDF namespace
                            "pdf:Producer" => metadata.pdf_producer = Some(text),
                            "pdf:Keywords" => metadata.pdf_keywords = Some(text),
                            "pdf:PDFVersion" => metadata.pdf_version = Some(text),
                            "pdf:Trapped" => metadata.pdf_trapped = Some(text),

                            // XMP Rights
                            "xmpRights:UsageTerms" => {
                                if metadata.xmp_rights_usage_terms.is_none() {
                                    metadata.xmp_rights_usage_terms = Some(text);
                                }
                            },
                            "xmpRights:Marked" => {
                                metadata.xmp_rights_marked = Some(text.to_lowercase() == "true");
                            },
                            "xmpRights:WebStatement" => {
                                metadata.xmp_rights_web_statement = Some(text)
                            },

                            // Store unknown properties
                            _ => {
                                metadata.custom.insert(prop.clone(), text);
                            },
                        }
                    }
                },
                Ok(Event::End(_)) => {
                    element_stack.pop();
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("XMP parsing error: {:?}", e);
                    break;
                },
                _ => {},
            }
        }

        Ok(Some(metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xmp_basic() {
        let xmp = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:xmp="http://ns.adobe.com/xap/1.0/"
        xmlns:pdf="http://ns.adobe.com/pdf/1.3/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">Test Document</rdf:li>
        </rdf:Alt>
      </dc:title>
      <dc:creator>
        <rdf:Seq>
          <rdf:li>John Doe</rdf:li>
          <rdf:li>Jane Smith</rdf:li>
        </rdf:Seq>
      </dc:creator>
      <xmp:CreatorTool>pdf_oxide</xmp:CreatorTool>
      <xmp:CreateDate>2024-01-15T10:30:00Z</xmp:CreateDate>
      <pdf:Producer>pdf_oxide 0.3.0</pdf:Producer>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = XmpExtractor::parse_xmp(xmp).unwrap().unwrap();

        assert_eq!(metadata.dc_title, Some("Test Document".to_string()));
        assert_eq!(metadata.dc_creator, vec!["John Doe", "Jane Smith"]);
        assert_eq!(metadata.xmp_creator_tool, Some("pdf_oxide".to_string()));
        assert_eq!(metadata.xmp_create_date, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(metadata.pdf_producer, Some("pdf_oxide 0.3.0".to_string()));
    }

    #[test]
    fn test_xmp_metadata_builder() {
        let metadata = XmpMetadata::new()
            .with_title("My Document")
            .with_creator("Author 1")
            .with_creator("Author 2")
            .with_description("A test document")
            .with_creator_tool("pdf_oxide")
            .with_producer("pdf_oxide 0.3.0");

        assert_eq!(metadata.dc_title, Some("My Document".to_string()));
        assert_eq!(metadata.dc_creator, vec!["Author 1", "Author 2"]);
        assert_eq!(metadata.dc_description, Some("A test document".to_string()));
        assert_eq!(metadata.xmp_creator_tool, Some("pdf_oxide".to_string()));
        assert_eq!(metadata.pdf_producer, Some("pdf_oxide 0.3.0".to_string()));
    }

    #[test]
    fn test_xmp_is_empty() {
        let empty = XmpMetadata::new();
        assert!(empty.is_empty());

        let non_empty = XmpMetadata::new().with_title("Title");
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_parse_xmp_with_subjects() {
        let xmp = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:subject>
        <rdf:Bag>
          <rdf:li>PDF</rdf:li>
          <rdf:li>Rust</rdf:li>
          <rdf:li>Metadata</rdf:li>
        </rdf:Bag>
      </dc:subject>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = XmpExtractor::parse_xmp(xmp).unwrap().unwrap();
        assert_eq!(metadata.dc_subject, vec!["PDF", "Rust", "Metadata"]);
    }
}
