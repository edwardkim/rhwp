//! Page label writing for PDF documents.
//!
//! Provides functionality to create and write page labels to PDF documents.
//! See ISO 32000-1:2008, Section 12.4.2 - Page Labels.

use crate::extractors::page_labels::{PageLabelRange, PageLabelStyle};
use crate::object::Object;
use std::collections::HashMap;

/// Builder for creating page label number trees.
pub struct PageLabelsBuilder {
    ranges: Vec<PageLabelRange>,
}

impl PageLabelsBuilder {
    /// Create a new page labels builder.
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Add a page label range.
    ///
    /// # Arguments
    ///
    /// * `range` - The page label range to add
    ///
    /// # Example
    ///
    /// ```ignore
    /// use pdf_oxide::writer::PageLabelsBuilder;
    /// use pdf_oxide::extractors::page_labels::{PageLabelRange, PageLabelStyle};
    ///
    /// let labels = PageLabelsBuilder::new()
    ///     .add_range(PageLabelRange::new(0).with_style(PageLabelStyle::RomanLower))
    ///     .add_range(PageLabelRange::new(4).with_style(PageLabelStyle::Decimal))
    ///     .build();
    /// ```
    pub fn add_range(mut self, range: PageLabelRange) -> Self {
        self.ranges.push(range);
        self
    }

    /// Add a decimal numbering range starting at the given page.
    pub fn decimal(self, start_page: usize) -> Self {
        self.add_range(PageLabelRange::new(start_page).with_style(PageLabelStyle::Decimal))
    }

    /// Add a lowercase Roman numeral range starting at the given page.
    pub fn roman_lower(self, start_page: usize) -> Self {
        self.add_range(PageLabelRange::new(start_page).with_style(PageLabelStyle::RomanLower))
    }

    /// Add an uppercase Roman numeral range starting at the given page.
    pub fn roman_upper(self, start_page: usize) -> Self {
        self.add_range(PageLabelRange::new(start_page).with_style(PageLabelStyle::RomanUpper))
    }

    /// Add a lowercase alphabetic range starting at the given page.
    pub fn alpha_lower(self, start_page: usize) -> Self {
        self.add_range(PageLabelRange::new(start_page).with_style(PageLabelStyle::AlphaLower))
    }

    /// Add an uppercase alphabetic range starting at the given page.
    pub fn alpha_upper(self, start_page: usize) -> Self {
        self.add_range(PageLabelRange::new(start_page).with_style(PageLabelStyle::AlphaUpper))
    }

    /// Add a prefixed decimal numbering range.
    pub fn prefixed(self, start_page: usize, prefix: &str, start_value: u32) -> Self {
        self.add_range(
            PageLabelRange::new(start_page)
                .with_style(PageLabelStyle::Decimal)
                .with_prefix(prefix)
                .with_start_value(start_value),
        )
    }

    /// Build the page labels as a PDF number tree dictionary.
    ///
    /// Returns a dictionary suitable for the /PageLabels entry in the catalog.
    pub fn build(mut self) -> Object {
        // Sort ranges by start_page
        self.ranges.sort_by_key(|r| r.start_page);

        // Build the /Nums array
        let mut nums = Vec::new();

        for range in &self.ranges {
            // Add the page index
            nums.push(Object::Integer(range.start_page as i64));

            // Add the label dictionary
            nums.push(range_to_object(range));
        }

        // Build the number tree dictionary
        let mut tree = HashMap::new();
        tree.insert("Nums".to_string(), Object::Array(nums));

        Object::Dictionary(tree)
    }

    /// Build from existing ranges.
    pub fn from_ranges(ranges: Vec<PageLabelRange>) -> Self {
        Self { ranges }
    }

    /// Get the ranges.
    pub fn ranges(&self) -> &[PageLabelRange] {
        &self.ranges
    }
}

impl Default for PageLabelsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a PageLabelRange to a PDF dictionary object.
fn range_to_object(range: &PageLabelRange) -> Object {
    let mut dict = HashMap::new();

    // /S - numbering style (optional if None)
    if let Some(style_name) = range.style.to_name() {
        dict.insert("S".to_string(), Object::Name(style_name.to_string()));
    }

    // /P - prefix (optional)
    if let Some(ref prefix) = range.prefix {
        dict.insert("P".to_string(), Object::text_string(prefix));
    }

    // /St - starting value (optional, only if != 1)
    if range.start_value != 1 {
        dict.insert("St".to_string(), Object::Integer(range.start_value as i64));
    }

    Object::Dictionary(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_labels_builder() {
        let labels = PageLabelsBuilder::new()
            .roman_lower(0)
            .decimal(4)
            .prefixed(7, "A-", 8)
            .build();

        let dict = labels.as_dict().unwrap();
        let nums = dict.get("Nums").unwrap().as_array().unwrap();

        // Should have 6 entries: 3 pairs of (index, dict)
        assert_eq!(nums.len(), 6);

        // First range: page 0, roman lower
        assert_eq!(nums[0].as_integer(), Some(0));
        let dict1 = nums[1].as_dict().unwrap();
        assert_eq!(dict1.get("S").unwrap().as_name(), Some("r"));

        // Second range: page 4, decimal
        assert_eq!(nums[2].as_integer(), Some(4));
        let dict2 = nums[3].as_dict().unwrap();
        assert_eq!(dict2.get("S").unwrap().as_name(), Some("D"));

        // Third range: page 7, prefixed decimal
        assert_eq!(nums[4].as_integer(), Some(7));
        let dict3 = nums[5].as_dict().unwrap();
        assert_eq!(dict3.get("S").unwrap().as_name(), Some("D"));
        assert_eq!(dict3.get("P").unwrap().as_string(), Some("A-".as_bytes()));
        assert_eq!(dict3.get("St").unwrap().as_integer(), Some(8));
    }

    #[test]
    fn test_range_to_object_minimal() {
        let range = PageLabelRange::new(0).with_style(PageLabelStyle::Decimal);
        let obj = range_to_object(&range);
        let dict = obj.as_dict().unwrap();

        // Should have /S but no /St or /P since defaults are used
        assert!(dict.contains_key("S"));
        assert!(!dict.contains_key("St")); // Default 1
        assert!(!dict.contains_key("P")); // No prefix
    }

    #[test]
    fn test_range_to_object_full() {
        let range = PageLabelRange::new(0)
            .with_style(PageLabelStyle::RomanUpper)
            .with_prefix("Chapter-")
            .with_start_value(5);
        let obj = range_to_object(&range);
        let dict = obj.as_dict().unwrap();

        assert_eq!(dict.get("S").unwrap().as_name(), Some("R"));
        assert_eq!(dict.get("P").unwrap().as_string(), Some("Chapter-".as_bytes()));
        assert_eq!(dict.get("St").unwrap().as_integer(), Some(5));
    }
}
