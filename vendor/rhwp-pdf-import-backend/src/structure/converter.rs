//! Converter from StructElem to StructureElement.
//!
//! This module bridges the PDF spec-level structure representation (StructElem)
//! with the unified content API (StructureElement) for round-trip operations.
//!
//! PDF Spec: ISO 32000-1:2008, Section 14.7-14.8 (Structure Trees and Tagged PDF)

use crate::elements::{ContentElement, StructureElement};
use crate::error::Result;
use crate::geometry::Rect;
use crate::object::Object;
use crate::structure::types::{StructChild, StructElem, StructType};
use std::collections::HashMap;

/// Converter from PDF structure elements to unified content elements.
///
/// This converter handles:
/// - Recursive structure element hierarchy conversion
/// - MCID (Marked Content ID) to content mapping
/// - Accessibility attribute extraction (alt text, language)
/// - Standard structure type mapping
pub struct StructureConverter {
    /// Map from MCID to extracted content elements
    mcid_map: HashMap<u32, Vec<ContentElement>>,
}

impl StructureConverter {
    /// Create a new converter with an MCID to content mapping.
    ///
    /// # Arguments
    ///
    /// * `mcid_map` - HashMap of MCID values to their extracted content elements
    pub fn new(mcid_map: HashMap<u32, Vec<ContentElement>>) -> Self {
        Self { mcid_map }
    }

    /// Convert a StructElem to a StructureElement.
    ///
    /// This recursively converts the entire hierarchy, populating children
    /// with actual content elements where MCIDs are referenced.
    ///
    /// # Arguments
    ///
    /// * `elem` - The structure element to convert
    ///
    /// # Returns
    ///
    /// A StructureElement with populated children
    ///
    /// # PDF Spec Compliance
    ///
    /// - ISO 32000-1:2008, Section 14.7.2 - Structure Hierarchy
    /// - ISO 32000-1:2008, Section 14.7.4 - Marked Content Identification
    /// - ISO 32000-1:2008, Section 14.9.3 - Accessibility Attributes
    pub fn convert_struct_elem(&self, elem: &StructElem) -> Result<StructureElement> {
        let mut children = Vec::new();

        // Process all children
        for child in &elem.children {
            match child {
                StructChild::StructElem(nested) => {
                    // Recursive structure element
                    let nested_structure = self.convert_struct_elem(nested)?;
                    children.push(ContentElement::Structure(nested_structure));
                },
                StructChild::MarkedContentRef { mcid, .. } => {
                    // Lookup content by MCID
                    if let Some(content_elements) = self.mcid_map.get(mcid) {
                        children.extend(content_elements.clone());
                    }
                    // If MCID not found, silently skip (per spec, missing MCIDs may occur)
                },
                StructChild::ObjectRef(_, _) => {
                    // Object references to other struct elements are deferred
                    // In a full implementation, would resolve indirect references
                },
            }
        }

        // Calculate bounding box from children
        let bbox = Self::calculate_bbox(&children);

        // Extract accessibility attributes
        let alt_text = Self::extract_alt_text(&elem.attributes);
        let language = Self::extract_language(&elem.attributes);

        Ok(StructureElement {
            structure_type: Self::format_struct_type(&elem.struct_type),
            bbox,
            children,
            reading_order: None, // Will be set from parent tree if available
            alt_text,
            language,
        })
    }

    /// Calculate bounding box from children.
    ///
    /// Computes the minimal rectangle that encompasses all child elements.
    fn calculate_bbox(children: &[ContentElement]) -> Rect {
        if children.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for child in children {
            let bbox = child.bbox();
            min_x = min_x.min(bbox.x);
            min_y = min_y.min(bbox.y);
            max_x = max_x.max(bbox.x + bbox.width);
            max_y = max_y.max(bbox.y + bbox.height);
        }

        if min_x == f32::MAX {
            Rect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
        }
    }

    /// Extract alternative text (alt text) from attributes.
    ///
    /// Per PDF Spec Section 14.9.3, alt text is stored in the `/Alt` attribute
    /// and provides a text description for accessibility.
    fn extract_alt_text(attributes: &HashMap<String, Object>) -> Option<String> {
        attributes.get("Alt").and_then(|obj| {
            if let Object::String(bytes) = obj {
                String::from_utf8(bytes.clone()).ok()
            } else {
                None
            }
        })
    }

    /// Extract language tag from attributes.
    ///
    /// Per PDF Spec Section 14.9.3, language tags are stored in the `/Lang` attribute
    /// as a string (e.g., "en-US", "fr", "de").
    fn extract_language(attributes: &HashMap<String, Object>) -> Option<String> {
        attributes.get("Lang").and_then(|obj| {
            if let Object::String(bytes) = obj {
                String::from_utf8(bytes.clone()).ok()
            } else {
                None
            }
        })
    }

    /// Format structure type for display.
    ///
    /// Converts StructType enum to human-readable string form.
    fn format_struct_type(struct_type: &StructType) -> String {
        match struct_type {
            StructType::Document => "Document".to_string(),
            StructType::Part => "Part".to_string(),
            StructType::Art => "Article".to_string(),
            StructType::Sect => "Section".to_string(),
            StructType::Div => "Division".to_string(),
            StructType::P => "P".to_string(),
            StructType::H => "H".to_string(),
            StructType::H1 => "H1".to_string(),
            StructType::H2 => "H2".to_string(),
            StructType::H3 => "H3".to_string(),
            StructType::H4 => "H4".to_string(),
            StructType::H5 => "H5".to_string(),
            StructType::H6 => "H6".to_string(),
            StructType::L => "List".to_string(),
            StructType::LI => "ListItem".to_string(),
            StructType::Lbl => "Label".to_string(),
            StructType::LBody => "ListBody".to_string(),
            StructType::Table => "Table".to_string(),
            StructType::THead => "TableHead".to_string(),
            StructType::TBody => "TableBody".to_string(),
            StructType::TFoot => "TableFoot".to_string(),
            StructType::TR => "TableRow".to_string(),
            StructType::TH => "TableHeader".to_string(),
            StructType::TD => "TableData".to_string(),
            StructType::Span => "Span".to_string(),
            StructType::Quote => "Quote".to_string(),
            StructType::Note => "Note".to_string(),
            StructType::Reference => "Reference".to_string(),
            StructType::BibEntry => "BibEntry".to_string(),
            StructType::Code => "Code".to_string(),
            StructType::Link => "Link".to_string(),
            StructType::Annot => "Annotation".to_string(),
            StructType::Figure => "Figure".to_string(),
            StructType::Formula => "Formula".to_string(),
            StructType::Form => "Form".to_string(),
            StructType::WB => "WordBreak".to_string(),
            StructType::Custom(name) => name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let mcid_map = HashMap::new();
        let _converter = StructureConverter::new(mcid_map);
    }

    #[test]
    fn test_calculate_bbox_empty() {
        let bbox = StructureConverter::calculate_bbox(&[]);
        assert_eq!(bbox.x, 0.0);
        assert_eq!(bbox.y, 0.0);
        assert_eq!(bbox.width, 0.0);
        assert_eq!(bbox.height, 0.0);
    }

    #[test]
    fn test_extract_alt_text() {
        let mut attrs = HashMap::new();
        attrs.insert("Alt".to_string(), Object::String(b"Alt text".to_vec()));

        let alt_text = StructureConverter::extract_alt_text(&attrs);
        assert_eq!(alt_text, Some("Alt text".to_string()));
    }

    #[test]
    fn test_extract_language() {
        let mut attrs = HashMap::new();
        attrs.insert("Lang".to_string(), Object::String(b"en-US".to_vec()));

        let lang = StructureConverter::extract_language(&attrs);
        assert_eq!(lang, Some("en-US".to_string()));
    }

    #[test]
    fn test_format_struct_type() {
        assert_eq!(StructureConverter::format_struct_type(&StructType::Document), "Document");
        assert_eq!(StructureConverter::format_struct_type(&StructType::H1), "H1");
        assert_eq!(StructureConverter::format_struct_type(&StructType::P), "P");
        assert_eq!(StructureConverter::format_struct_type(&StructType::Table), "Table");
    }

    #[test]
    fn test_format_struct_type_all_variants() {
        // Test all standard structure types
        let cases = vec![
            (StructType::Document, "Document"),
            (StructType::Part, "Part"),
            (StructType::Art, "Article"),
            (StructType::Sect, "Section"),
            (StructType::Div, "Division"),
            (StructType::P, "P"),
            (StructType::H, "H"),
            (StructType::H1, "H1"),
            (StructType::H2, "H2"),
            (StructType::H3, "H3"),
            (StructType::H4, "H4"),
            (StructType::H5, "H5"),
            (StructType::H6, "H6"),
            (StructType::L, "List"),
            (StructType::LI, "ListItem"),
            (StructType::Lbl, "Label"),
            (StructType::LBody, "ListBody"),
            (StructType::Table, "Table"),
            (StructType::THead, "TableHead"),
            (StructType::TBody, "TableBody"),
            (StructType::TFoot, "TableFoot"),
            (StructType::TR, "TableRow"),
            (StructType::TH, "TableHeader"),
            (StructType::TD, "TableData"),
            (StructType::Span, "Span"),
            (StructType::Quote, "Quote"),
            (StructType::Note, "Note"),
            (StructType::Reference, "Reference"),
            (StructType::BibEntry, "BibEntry"),
            (StructType::Code, "Code"),
            (StructType::Link, "Link"),
            (StructType::Annot, "Annotation"),
            (StructType::Figure, "Figure"),
            (StructType::Formula, "Formula"),
            (StructType::Form, "Form"),
            (StructType::WB, "WordBreak"),
            (StructType::Custom("MyType".to_string()), "MyType"),
        ];
        for (st, expected) in cases {
            assert_eq!(StructureConverter::format_struct_type(&st), expected);
        }
    }

    #[test]
    fn test_extract_alt_text_missing() {
        let attrs = HashMap::new();
        assert_eq!(StructureConverter::extract_alt_text(&attrs), None);
    }

    #[test]
    fn test_extract_alt_text_wrong_type() {
        let mut attrs = HashMap::new();
        attrs.insert("Alt".to_string(), Object::Integer(42));
        assert_eq!(StructureConverter::extract_alt_text(&attrs), None);
    }

    #[test]
    fn test_extract_language_missing() {
        let attrs = HashMap::new();
        assert_eq!(StructureConverter::extract_language(&attrs), None);
    }

    #[test]
    fn test_extract_language_wrong_type() {
        let mut attrs = HashMap::new();
        attrs.insert("Lang".to_string(), Object::Integer(1));
        assert_eq!(StructureConverter::extract_language(&attrs), None);
    }

    #[test]
    fn test_convert_simple_struct_elem() {
        use crate::elements::{FontSpec, TextContent, TextStyle};
        use crate::geometry::Rect;

        let mut mcid_map: HashMap<u32, Vec<ContentElement>> = HashMap::new();
        mcid_map.insert(
            0,
            vec![ContentElement::Text(TextContent::new(
                "Hello",
                Rect::new(10.0, 20.0, 100.0, 12.0),
                FontSpec::default(),
                TextStyle::default(),
            ))],
        );

        let converter = StructureConverter::new(mcid_map);

        let mut elem = StructElem::new(StructType::P);
        elem.add_child(StructChild::MarkedContentRef {
            mcid: 0,
            page: 0,
            scope: crate::structure::McidScope::Page(0),
        });

        let result = converter.convert_struct_elem(&elem).unwrap();
        assert_eq!(result.structure_type, "P");
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn test_convert_nested_struct_elem() {
        let mcid_map: HashMap<u32, Vec<ContentElement>> = HashMap::new();
        let converter = StructureConverter::new(mcid_map);

        let mut root = StructElem::new(StructType::Document);
        let child_p = StructElem::new(StructType::P);
        root.add_child(StructChild::StructElem(Box::new(child_p)));

        let result = converter.convert_struct_elem(&root).unwrap();
        assert_eq!(result.structure_type, "Document");
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn test_convert_with_object_ref() {
        let mcid_map: HashMap<u32, Vec<ContentElement>> = HashMap::new();
        let converter = StructureConverter::new(mcid_map);

        let mut elem = StructElem::new(StructType::Div);
        elem.add_child(StructChild::ObjectRef(42, 0));

        let result = converter.convert_struct_elem(&elem).unwrap();
        assert_eq!(result.structure_type, "Division");
        // ObjectRef is silently skipped
        assert_eq!(result.children.len(), 0);
    }

    #[test]
    fn test_convert_missing_mcid() {
        let mcid_map: HashMap<u32, Vec<ContentElement>> = HashMap::new();
        let converter = StructureConverter::new(mcid_map);

        let mut elem = StructElem::new(StructType::Span);
        elem.add_child(StructChild::MarkedContentRef {
            mcid: 999,
            page: 0,
            scope: crate::structure::McidScope::Page(0),
        });

        let result = converter.convert_struct_elem(&elem).unwrap();
        // Missing MCIDs are silently skipped
        assert_eq!(result.children.len(), 0);
    }

    #[test]
    fn test_convert_with_alt_text_and_language() {
        let mcid_map: HashMap<u32, Vec<ContentElement>> = HashMap::new();
        let converter = StructureConverter::new(mcid_map);

        let mut elem = StructElem::new(StructType::Figure);
        elem.attributes
            .insert("Alt".to_string(), Object::String(b"A photo".to_vec()));
        elem.attributes
            .insert("Lang".to_string(), Object::String(b"en".to_vec()));

        let result = converter.convert_struct_elem(&elem).unwrap();
        assert_eq!(result.alt_text, Some("A photo".to_string()));
        assert_eq!(result.language, Some("en".to_string()));
    }

    #[test]
    fn test_calculate_bbox_with_children() {
        use crate::elements::{FontSpec, TextContent, TextStyle};
        use crate::geometry::Rect;

        let children = vec![
            ContentElement::Text(TextContent::new(
                "A",
                Rect::new(10.0, 20.0, 50.0, 12.0),
                FontSpec::default(),
                TextStyle::default(),
            )),
            ContentElement::Text(TextContent::new(
                "B",
                Rect::new(100.0, 50.0, 80.0, 14.0),
                FontSpec::default(),
                TextStyle::default(),
            )),
        ];

        let bbox = StructureConverter::calculate_bbox(&children);
        assert!(bbox.x <= 10.0);
        assert!(bbox.y <= 20.0);
        assert!(bbox.x + bbox.width >= 180.0); // 100 + 80
        assert!(bbox.y + bbox.height >= 64.0); // 50 + 14
    }
}
