//! Markdown table formatting for detected tables.
//!
//! Note: Table detection is not PDF spec-compliant and has been removed.
//! This module is deprecated and will be removed in a future version.

use crate::converters::TableFormatConfig;
use crate::layout::TextBlock;

/// Markdown table formatter.
///
/// Converts detected table structures to valid markdown table syntax.
pub struct MarkdownTableFormatter;

impl MarkdownTableFormatter {
    /// DEPRECATED: Format a table as markdown.
    ///
    /// This module is no longer used as table detection is not PDF spec-compliant.
    /// This method is kept for backwards compatibility but always returns empty string.
    #[deprecated = "Table detection has been removed - use PDF structure tree instead"]
    pub fn format_table(
        _table_text: &str,
        _blocks: &[TextBlock],
        _config: &TableFormatConfig,
    ) -> String {
        // Stub implementation - table detection removed for PDF spec compliance
        String::new()
    }
}
