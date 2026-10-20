//! XMP metadata writing for PDF documents.
//!
//! Generates XMP (Extensible Metadata Platform) metadata packets for PDF documents.
//! XMP is XML-based metadata that provides richer information than the
//! traditional Info dictionary. See ISO 32000-1:2008, Section 14.3.2.

use crate::extractors::xmp::XmpMetadata;

/// XMP namespace URIs
const NS_X: &str = "adobe:ns:meta/";
const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
const NS_XMP: &str = "http://ns.adobe.com/xap/1.0/";
const NS_PDF: &str = "http://ns.adobe.com/pdf/1.3/";
const NS_XMP_RIGHTS: &str = "http://ns.adobe.com/xap/1.0/rights/";

/// XMP metadata writer/builder.
pub struct XmpWriter {
    metadata: XmpMetadata,
}

impl XmpWriter {
    /// Create a new XMP writer from metadata.
    pub fn new(metadata: XmpMetadata) -> Self {
        Self { metadata }
    }

    /// Create a new XMP writer with default metadata.
    pub fn default_metadata() -> Self {
        let mut metadata = XmpMetadata::new();
        metadata.xmp_creator_tool = Some("pdf_oxide".to_string());
        metadata.pdf_producer = Some(format!("pdf_oxide {}", env!("CARGO_PKG_VERSION")));
        Self { metadata }
    }

    /// Set the document title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.metadata.dc_title = Some(title.into());
        self
    }

    /// Add a creator/author.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.metadata.dc_creator.push(creator.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.metadata.dc_description = Some(desc.into());
        self
    }

    /// Add a subject/keyword.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.metadata.dc_subject.push(subject.into());
        self
    }

    /// Set the creator tool.
    pub fn creator_tool(mut self, tool: impl Into<String>) -> Self {
        self.metadata.xmp_creator_tool = Some(tool.into());
        self
    }

    /// Set the creation date (ISO 8601 format).
    pub fn create_date(mut self, date: impl Into<String>) -> Self {
        self.metadata.xmp_create_date = Some(date.into());
        self
    }

    /// Set the modification date (ISO 8601 format).
    pub fn modify_date(mut self, date: impl Into<String>) -> Self {
        self.metadata.xmp_modify_date = Some(date.into());
        self
    }

    /// Set the PDF producer.
    pub fn producer(mut self, producer: impl Into<String>) -> Self {
        self.metadata.pdf_producer = Some(producer.into());
        self
    }

    /// Set the PDF keywords.
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.metadata.pdf_keywords = Some(keywords.into());
        self
    }

    /// Set the PDF version.
    pub fn pdf_version(mut self, version: impl Into<String>) -> Self {
        self.metadata.pdf_version = Some(version.into());
        self
    }

    /// Set the rights usage terms.
    pub fn usage_terms(mut self, terms: impl Into<String>) -> Self {
        self.metadata.xmp_rights_usage_terms = Some(terms.into());
        self
    }

    /// Set whether marked with rights.
    pub fn rights_marked(mut self, marked: bool) -> Self {
        self.metadata.xmp_rights_marked = Some(marked);
        self
    }

    /// Build the XMP packet as an XML string.
    pub fn build(self) -> String {
        self.to_xml()
    }

    /// Build the XMP packet as bytes.
    pub fn build_bytes(self) -> Vec<u8> {
        self.to_xml().into_bytes()
    }

    /// Convert metadata to XMP XML.
    fn to_xml(&self) -> String {
        let mut xml = String::new();

        // XMP packet header
        xml.push_str(r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>"#);
        xml.push('\n');

        // XMP metadata root
        xml.push_str(&format!(r#"<x:xmpmeta xmlns:x="{}">"#, NS_X));
        xml.push('\n');

        // RDF root
        xml.push_str(&format!(r#"  <rdf:RDF xmlns:rdf="{}">"#, NS_RDF));
        xml.push('\n');

        // Main description with all namespaces
        xml.push_str("    <rdf:Description rdf:about=\"\"\n");
        xml.push_str(&format!("        xmlns:dc=\"{}\"\n", NS_DC));
        xml.push_str(&format!("        xmlns:xmp=\"{}\"\n", NS_XMP));
        xml.push_str(&format!("        xmlns:pdf=\"{}\"\n", NS_PDF));
        xml.push_str(&format!("        xmlns:xmpRights=\"{}\">\n", NS_XMP_RIGHTS));

        // Dublin Core properties
        if let Some(title) = &self.metadata.dc_title {
            xml.push_str("      <dc:title>\n");
            xml.push_str("        <rdf:Alt>\n");
            xml.push_str(&format!(
                "          <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n",
                escape_xml(title)
            ));
            xml.push_str("        </rdf:Alt>\n");
            xml.push_str("      </dc:title>\n");
        }

        if !self.metadata.dc_creator.is_empty() {
            xml.push_str("      <dc:creator>\n");
            xml.push_str("        <rdf:Seq>\n");
            for creator in &self.metadata.dc_creator {
                xml.push_str(&format!("          <rdf:li>{}</rdf:li>\n", escape_xml(creator)));
            }
            xml.push_str("        </rdf:Seq>\n");
            xml.push_str("      </dc:creator>\n");
        }

        if let Some(desc) = &self.metadata.dc_description {
            xml.push_str("      <dc:description>\n");
            xml.push_str("        <rdf:Alt>\n");
            xml.push_str(&format!(
                "          <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n",
                escape_xml(desc)
            ));
            xml.push_str("        </rdf:Alt>\n");
            xml.push_str("      </dc:description>\n");
        }

        if !self.metadata.dc_subject.is_empty() {
            xml.push_str("      <dc:subject>\n");
            xml.push_str("        <rdf:Bag>\n");
            for subject in &self.metadata.dc_subject {
                xml.push_str(&format!("          <rdf:li>{}</rdf:li>\n", escape_xml(subject)));
            }
            xml.push_str("        </rdf:Bag>\n");
            xml.push_str("      </dc:subject>\n");
        }

        if let Some(language) = &self.metadata.dc_language {
            xml.push_str(&format!("      <dc:language>{}</dc:language>\n", escape_xml(language)));
        }

        if let Some(rights) = &self.metadata.dc_rights {
            xml.push_str("      <dc:rights>\n");
            xml.push_str("        <rdf:Alt>\n");
            xml.push_str(&format!(
                "          <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n",
                escape_xml(rights)
            ));
            xml.push_str("        </rdf:Alt>\n");
            xml.push_str("      </dc:rights>\n");
        }

        if let Some(format) = &self.metadata.dc_format {
            xml.push_str(&format!("      <dc:format>{}</dc:format>\n", escape_xml(format)));
        }

        // XMP Core properties
        if let Some(tool) = &self.metadata.xmp_creator_tool {
            xml.push_str(&format!(
                "      <xmp:CreatorTool>{}</xmp:CreatorTool>\n",
                escape_xml(tool)
            ));
        }

        if let Some(date) = &self.metadata.xmp_create_date {
            xml.push_str(&format!("      <xmp:CreateDate>{}</xmp:CreateDate>\n", escape_xml(date)));
        }

        if let Some(date) = &self.metadata.xmp_modify_date {
            xml.push_str(&format!("      <xmp:ModifyDate>{}</xmp:ModifyDate>\n", escape_xml(date)));
        }

        if let Some(date) = &self.metadata.xmp_metadata_date {
            xml.push_str(&format!(
                "      <xmp:MetadataDate>{}</xmp:MetadataDate>\n",
                escape_xml(date)
            ));
        }

        // PDF properties
        if let Some(producer) = &self.metadata.pdf_producer {
            xml.push_str(&format!("      <pdf:Producer>{}</pdf:Producer>\n", escape_xml(producer)));
        }

        if let Some(keywords) = &self.metadata.pdf_keywords {
            xml.push_str(&format!("      <pdf:Keywords>{}</pdf:Keywords>\n", escape_xml(keywords)));
        }

        if let Some(version) = &self.metadata.pdf_version {
            xml.push_str(&format!(
                "      <pdf:PDFVersion>{}</pdf:PDFVersion>\n",
                escape_xml(version)
            ));
        }

        if let Some(trapped) = &self.metadata.pdf_trapped {
            xml.push_str(&format!("      <pdf:Trapped>{}</pdf:Trapped>\n", escape_xml(trapped)));
        }

        // XMP Rights properties
        if let Some(terms) = &self.metadata.xmp_rights_usage_terms {
            xml.push_str("      <xmpRights:UsageTerms>\n");
            xml.push_str("        <rdf:Alt>\n");
            xml.push_str(&format!(
                "          <rdf:li xml:lang=\"x-default\">{}</rdf:li>\n",
                escape_xml(terms)
            ));
            xml.push_str("        </rdf:Alt>\n");
            xml.push_str("      </xmpRights:UsageTerms>\n");
        }

        if let Some(marked) = self.metadata.xmp_rights_marked {
            xml.push_str(&format!(
                "      <xmpRights:Marked>{}</xmpRights:Marked>\n",
                if marked { "True" } else { "False" }
            ));
        }

        if let Some(url) = &self.metadata.xmp_rights_web_statement {
            xml.push_str(&format!(
                "      <xmpRights:WebStatement>{}</xmpRights:WebStatement>\n",
                escape_xml(url)
            ));
        }

        // Custom properties, sorted by key: `custom` is a `HashMap`, so
        // iterating it directly writes the properties in the per-process
        // randomized order and the packet's bytes differ between runs.
        let mut custom_keys: Vec<&String> = self.metadata.custom.keys().collect();
        custom_keys.sort();
        for key in custom_keys {
            if let Some(value) = self.metadata.custom.get(key) {
                xml.push_str(&format!("      <{}>{}</{}>\n", key, escape_xml(value), key));
            }
        }

        // Close elements
        xml.push_str("    </rdf:Description>\n");
        xml.push_str("  </rdf:RDF>\n");
        xml.push_str("</x:xmpmeta>\n");

        // Add padding for editability (per XMP spec)
        // 2KB of padding is standard
        for _ in 0..40 {
            xml.push_str("                                                  \n");
        }

        // XMP packet end
        xml.push_str(r#"<?xpacket end="w"?>"#);

        xml
    }
}

impl Default for XmpWriter {
    fn default() -> Self {
        Self::default_metadata()
    }
}

/// Escape special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate current timestamp in ISO 8601 format.
pub fn iso_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmp_writer_basic() {
        let writer = XmpWriter::default_metadata()
            .title("Test Document")
            .creator("John Doe")
            .description("A test document");

        let xml = writer.build();

        assert!(xml.contains("<?xpacket begin"));
        assert!(xml.contains("<dc:title>"));
        assert!(xml.contains("Test Document"));
        assert!(xml.contains("<dc:creator>"));
        assert!(xml.contains("John Doe"));
        assert!(xml.contains("<dc:description>"));
        assert!(xml.contains("A test document"));
        assert!(xml.contains("<?xpacket end"));
    }

    #[test]
    fn test_xmp_writer_multiple_creators() {
        let writer = XmpWriter::new(XmpMetadata::new())
            .creator("Author 1")
            .creator("Author 2")
            .creator("Author 3");

        let xml = writer.build();

        assert!(xml.contains("<rdf:Seq>"));
        assert!(xml.contains("Author 1"));
        assert!(xml.contains("Author 2"));
        assert!(xml.contains("Author 3"));
    }

    #[test]
    fn test_xmp_writer_subjects() {
        let writer = XmpWriter::new(XmpMetadata::new())
            .subject("PDF")
            .subject("Rust")
            .subject("Metadata");

        let xml = writer.build();

        assert!(xml.contains("<dc:subject>"));
        assert!(xml.contains("<rdf:Bag>"));
        assert!(xml.contains("PDF"));
        assert!(xml.contains("Rust"));
        assert!(xml.contains("Metadata"));
    }

    #[test]
    fn test_xmp_writer_xml_escape() {
        let writer = XmpWriter::new(XmpMetadata::new()).title("Test & Document <special>");

        let xml = writer.build();

        assert!(xml.contains("Test &amp; Document &lt;special&gt;"));
    }

    #[test]
    fn test_xmp_writer_dates() {
        let writer = XmpWriter::new(XmpMetadata::new())
            .create_date("2024-01-15T10:30:00Z")
            .modify_date("2024-01-16T14:00:00Z");

        let xml = writer.build();

        assert!(xml.contains("<xmp:CreateDate>2024-01-15T10:30:00Z</xmp:CreateDate>"));
        assert!(xml.contains("<xmp:ModifyDate>2024-01-16T14:00:00Z</xmp:ModifyDate>"));
    }

    #[test]
    fn test_xmp_writer_pdf_properties() {
        let writer = XmpWriter::new(XmpMetadata::new())
            .producer("pdf_oxide 0.3.0")
            .keywords("test, pdf, metadata")
            .pdf_version("1.7");

        let xml = writer.build();

        assert!(xml.contains("<pdf:Producer>pdf_oxide 0.3.0</pdf:Producer>"));
        assert!(xml.contains("<pdf:Keywords>test, pdf, metadata</pdf:Keywords>"));
        assert!(xml.contains("<pdf:PDFVersion>1.7</pdf:PDFVersion>"));
    }

    #[test]
    fn test_xmp_writer_custom_properties_sorted() {
        let mut metadata = XmpMetadata::new();
        metadata
            .custom
            .insert("zz:Last".to_string(), "z".to_string());
        metadata
            .custom
            .insert("aa:First".to_string(), "a".to_string());
        metadata
            .custom
            .insert("mm:Middle".to_string(), "m".to_string());

        let xml = XmpWriter::new(metadata).build();

        let first = xml.find("<aa:First>").expect("aa:First missing");
        let middle = xml.find("<mm:Middle>").expect("mm:Middle missing");
        let last = xml.find("<zz:Last>").expect("zz:Last missing");
        assert!(first < middle && middle < last, "custom properties must emit in key order");
    }

    #[test]
    fn test_xmp_writer_rights() {
        let writer = XmpWriter::new(XmpMetadata::new())
            .usage_terms("All rights reserved")
            .rights_marked(true);

        let xml = writer.build();

        assert!(xml.contains("<xmpRights:UsageTerms>"));
        assert!(xml.contains("All rights reserved"));
        assert!(xml.contains("<xmpRights:Marked>True</xmpRights:Marked>"));
    }
}
