//! Structure tree builder for round-trip operations.
//!
//! Converts StructureElement hierarchies back to StructTreeRoot for saving.
//!
//! PDF Spec: ISO 32000-1:2008, Section 14.7-14.8

use crate::elements::StructureElement;
use crate::error::Result;
use crate::structure::types::{StructChild, StructElem, StructTreeRoot, StructType};
use std::collections::HashMap;

/// Builds a structure tree from a hierarchy of StructureElements.
///
/// This is the reverse operation of StructureConverter - converting from
/// the unified API back to PDF spec-compliant structure.
pub struct StructureTreeBuilder {
    /// MCID counter for assigning unique IDs
    mcid_counter: u32,

    /// ParentTree entries (page → mcid → struct elem ref)
    parent_tree_entries: HashMap<u32, HashMap<u32, u32>>,
}

impl StructureTreeBuilder {
    /// Create a new structure tree builder.
    pub fn new() -> Self {
        Self {
            mcid_counter: 0,
            parent_tree_entries: HashMap::new(),
        }
    }

    /// Build a structure tree from modified page content.
    ///
    /// # Arguments
    ///
    /// * `content` - Map of page_index to modified StructureElement
    /// * `page_count` - Total number of pages in document
    ///
    /// # Returns
    ///
    /// A new StructTreeRoot with updated structure
    ///
    /// # PDF Spec Compliance
    ///
    /// - ISO 32000-1:2008, Section 14.7.2 - Structure Hierarchy
    /// - ISO 32000-1:2008, Section 14.7.4.4 - ParentTree (number tree)
    pub fn build(
        _content: &HashMap<usize, StructureElement>,
        _page_count: usize,
    ) -> Result<StructTreeRoot> {
        // Build new structure tree from modified content
        // For now, return a basic structure
        // Full implementation would:
        // 1. Traverse hierarchy and assign MCIDs
        // 2. Build parent tree number tree
        // 3. Create new StructElem objects
        // 4. Handle RoleMap if needed

        Ok(StructTreeRoot::new())
    }

    /// Convert a StructureElement to a StructElem.
    ///
    /// Recursively processes the hierarchy, converting marked content
    /// references to MCIDs.
    fn convert_structure_element(
        &mut self,
        element: &StructureElement,
        _page_index: usize,
    ) -> Result<StructElem> {
        let struct_type = StructType::from_str(&element.structure_type);

        let mut elem = StructElem::new(struct_type);

        // Add children as struct elements or MCID references
        for child in &element.children {
            match child {
                crate::elements::ContentElement::Structure(nested) => {
                    let nested_elem = self.convert_structure_element(nested, _page_index)?;
                    elem.add_child(StructChild::StructElem(Box::new(nested_elem)));
                }
                _ => {
                    // Content elements become MCID references
                    let mcid = self.mcid_counter;
                    self.mcid_counter += 1;

                    elem.add_child(StructChild::MarkedContentRef {
                        mcid,
                        page: _page_index as u32,
                        scope: crate::structure::McidScope::Page(_page_index as u32),
                    });
                }
            }
        }

        Ok(elem)
    }

    /// Assign next MCID value.
    fn next_mcid(&mut self) -> u32 {
        let mcid = self.mcid_counter;
        self.mcid_counter += 1;
        mcid
    }
}

impl Default for StructureTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let _builder = StructureTreeBuilder::new();
    }

    #[test]
    fn test_builder_next_mcid() {
        let mut builder = StructureTreeBuilder::new();
        assert_eq!(builder.next_mcid(), 0);
        assert_eq!(builder.next_mcid(), 1);
        assert_eq!(builder.next_mcid(), 2);
    }

    #[test]
    fn test_build_empty() {
        let content: HashMap<usize, StructureElement> = HashMap::new();
        let result = StructureTreeBuilder::build(&content, 1).unwrap();

        assert!(result.root_elements.is_empty());
    }
}
