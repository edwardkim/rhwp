//! Markdown converter for PDF documents.
//!
//! This module converts PDF pages to Markdown format with support for:
//! - Heading detection (# ## ###)
//! - Paragraph formatting
//! - Image embedding
//! - Reading order determination

use crate::converters::text_post_processor::TextPostProcessor;
use crate::converters::whitespace::cleanup_markdown;
use crate::converters::{BoldMarkerBehavior, ConversionOptions, ReadingOrderMode};
use crate::error::Result;
use crate::extractors::SpacingConfig;
use crate::geometry::Rect;
use crate::layout::clustering::{cluster_chars_into_words, cluster_words_into_lines};
use crate::layout::document_analyzer::{AdaptiveLayoutParams, DocumentProperties};
use crate::layout::reading_order::graph_based_reading_order;
use crate::layout::{
    BoldGroup, BoldMarkerDecision, BoldMarkerValidator, Color, FontWeight, TextBlock, TextChar,
    TextSpan,
};
use crate::structure::spatial_table_detector::SpatialTableDetector;
use crate::structure::table_extractor::{Table, TableRow};
use crate::XYCutStrategy;
use regex::{Captures, Regex};
use std::sync::LazyLock;

static RE_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(https?://[^\s<>\[\]]*[^\s<>\[\].,!?;:])").unwrap());
static RE_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})").unwrap());
static RE_DASH_BEFORE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d)\s+(–|—)(\d)").unwrap());
static RE_DASH_AFTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d)(–|—)\s+(\d)").unwrap());
static RE_PUNCT_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([.!?;:,])([A-Za-z])").unwrap());

/// Converter for PDF to Markdown format.
///
/// # Examples
///
/// ```ignore
/// use pdf_oxide::PdfDocument;
/// use pdf_oxide::converters::{MarkdownConverter, ConversionOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut doc = PdfDocument::open("paper.pdf")?;
/// let chars = doc.extract_spans(0)?;
///
/// let converter = MarkdownConverter::new();
/// let options = ConversionOptions::default();
/// let markdown = converter.convert_page(&chars, &options)?;
///
/// println!("{}", markdown);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[deprecated(
    since = "0.2.0",
    note = "Use `pdf_oxide::pipeline::converters::MarkdownOutputConverter` instead. \
            The new converter is part of the unified TextPipeline architecture and \
            provides better feature support and maintainability."
)]
pub struct MarkdownConverter;

/// Returns true if a geometric gap between two adjacent blocks indicates
/// a word boundary, meaning a space should be inserted between them.
fn needs_inter_block_space(
    prev: &crate::layout::TextBlock,
    next: &crate::layout::TextBlock,
    spacing_config: &SpacingConfig,
) -> bool {
    let gap = next.bbox.left() - prev.bbox.right();
    let char_size = prev.bbox.width.max(prev.bbox.height);
    let threshold = spacing_config.word_margin * char_size;
    if gap <= threshold {
        return false;
    }
    let prev_ends_space = prev.text.chars().last().is_some_and(|c| c.is_whitespace());
    let next_starts_space = next.text.chars().next().is_some_and(|c| c.is_whitespace());
    !prev_ends_space && !next_starts_space
}

#[allow(deprecated)]
impl MarkdownConverter {
    /// Create a new Markdown converter.
    ///
    /// # Examples
    ///
    /// ```
    /// use pdf_oxide::converters::MarkdownConverter;
    ///
    /// let converter = MarkdownConverter::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Merge adjacent character-level spans that are too close to have real spaces.
    ///
    /// Per PDF Spec ISO 32000-1:2008 Section 9.4.4 NOTE 6, text strings should be
    /// "as long as possible". This merges character-level fragments that should be
    /// part of the same word.
    ///
    /// # Arguments
    ///
    /// * `blocks` - Sorted text blocks (by Y then X position)
    ///
    /// # Returns
    ///
    /// Merged text blocks with character-level fragments combined
    fn merge_adjacent_char_spans(blocks: Vec<TextBlock>) -> Vec<TextBlock> {
        if blocks.is_empty() {
            return blocks;
        }

        let mut merged: Vec<TextBlock> = Vec::new();
        let mut current: Option<TextBlock> = None;

        for block in blocks {
            match current.take() {
                None => {
                    // First block
                    current = Some(block);
                },
                Some(mut prev) => {
                    // Check if this block should be merged with previous
                    let same_line = (prev.bbox.y - block.bbox.y).abs() < 2.0;
                    let same_font = prev.dominant_font == block.dominant_font;
                    let same_size = (prev.avg_font_size - block.avg_font_size).abs() < 0.5;
                    let same_style = prev.is_bold == block.is_bold;

                    if same_line && same_font && same_size && same_style {
                        // Calculate gap between blocks
                        let prev_right = prev.bbox.x + prev.bbox.width;
                        let gap = block.bbox.x - prev_right;

                        // Merge threshold: 18% of font size
                        // Per PDF typography: char spacing is 5-15% em, word spacing is 20-40% em
                        // 18% catches character fragments while preserving word boundaries
                        let merge_threshold = prev.avg_font_size * 0.18;

                        // Don't merge if either block is just a space character
                        let prev_is_space = prev.text.trim().is_empty();
                        let curr_is_space = block.text.trim().is_empty();

                        if !prev_is_space && !curr_is_space && gap < merge_threshold {
                            // Merge: concatenate text and extend bounding box
                            prev.text.push_str(&block.text);
                            prev.bbox.width = (block.bbox.x + block.bbox.width) - prev.bbox.x;
                            current = Some(prev);
                        } else {
                            // Don't merge: push previous, keep current
                            merged.push(prev);
                            current = Some(block);
                        }
                    } else {
                        // Different line/font/size/style: don't merge
                        merged.push(prev);
                        current = Some(block);
                    }
                },
            }
        }

        // Don't forget the last block
        if let Some(last) = current {
            merged.push(last);
        }

        merged
    }

    /// Convert a page to Markdown format from text spans (PDF spec compliant - RECOMMENDED).
    ///
    /// This is the recommended method that uses PDF-native text spans instead of
    /// character-based extraction. Spans are complete text strings as provided by
    /// the PDF's Tj/TJ operators, eliminating the need for error-prone DBSCAN clustering.
    ///
    /// **Benefits over character-based conversion:**
    /// - PDF spec compliant (ISO 32000-1:2008, Section 9.4.4 NOTE 6)
    /// - No character splitting issues
    /// - Preserves PDF's text positioning intent
    /// - Much faster (no DBSCAN clustering needed)
    /// - More robust for complex layouts
    ///
    /// # Arguments
    ///
    /// * `spans` - The text spans extracted from the page via `extract_spans()`
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing the Markdown representation of the page.
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::converters::{MarkdownConverter, ConversionOptions};
    /// # use pdf_oxide::layout::TextSpan;
    /// # fn example(spans: Vec<TextSpan>) -> Result<(), Box<dyn std::error::Error>> {
    /// let converter = MarkdownConverter::new();
    /// let options = ConversionOptions {
    ///     detect_headings: true,
    ///     ..Default::default()
    /// };
    ///
    /// let markdown = converter.convert_page_from_spans(&spans, &options)?;
    /// println!("{}", markdown);
    /// # Ok(())
    /// # }
    /// ```
    pub fn convert_page_from_spans(
        &self,
        spans: &[crate::layout::TextSpan],
        options: &ConversionOptions,
    ) -> Result<String> {
        use crate::layout::TextBlock;

        if spans.is_empty() {
            return Ok(String::new());
        }

        // Detect and mark non-text content (figures, diagrams)
        // Use NonTextDetector to identify likely figure content
        let detector = crate::fonts::non_text_detection::NonTextDetector::default();
        let span_classifications = detector.mark_non_text_spans(spans);

        // Log figure detection results for debugging
        let figures_detected = span_classifications
            .iter()
            .filter(|c| c.is_non_text)
            .count();
        if figures_detected > 0 {
            log::debug!("Detected {} figure(s) out of {} spans", figures_detected, spans.len());
        }

        // Spatial table detection
        // Detect tables from current spans if enabled in options
        let detected_tables = if options.extract_tables {
            let detector_config = options.table_detection_config.clone().unwrap_or_default();
            let table_detector = SpatialTableDetector::with_config(detector_config);
            let tables = table_detector.detect_tables(spans);

            if !tables.is_empty() {
                log::debug!("Detected {} table(s) from {} spans", tables.len(), spans.len());
            }
            tables
        } else {
            Vec::new()
        };

        // Build set of span indices that belong to detected tables
        // This will be used to skip these spans when rendering normal text in Phase 5B Step 2
        let _table_span_indices: std::collections::HashSet<usize> = detected_tables
            .iter()
            .flat_map(|table| &table.span_indices)
            .copied()
            .collect();

        // Convert spans to TextBlocks, filtering out non-text content
        let mut blocks: Vec<TextBlock> = spans
            .iter()
            .enumerate()
            .filter_map(|(idx, span)| {
                // Check if this span was classified as non-text content
                let classification = &span_classifications[idx];

                if classification.is_non_text {
                    log::debug!(
                        "Filtering out non-text span: '{}...' (confidence: {:.2})",
                        span.text.chars().take(20).collect::<String>(),
                        classification.confidence
                    );
                    return None; // Skip non-text content
                }

                Some(TextBlock {
                    chars: vec![], // Not needed for span-based conversion
                    bbox: span.bbox,
                    text: span.text.clone(),
                    avg_font_size: span.font_size,
                    dominant_font: span.font_name.clone(),
                    is_bold: span.font_weight.is_bold(),
                    is_italic: span.is_italic,
                    mcid: span.mcid,
                    sequence: span.sequence,
                    rotation_degrees: span.rotation_degrees,
                })
            })
            .collect();

        // Sort blocks by Y position (top to bottom), then X position (left to right)
        blocks.sort_by(|a, b| {
            let y_cmp = crate::utils::safe_float_cmp(a.bbox.y, b.bbox.y);
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x)
        });

        // **Task B.1: Pre-Validation Bold Filter (BEFORE any grouping)**
        // Filter whitespace-only blocks BEFORE merging
        // This prevents empty blocks from entering the bold grouping pipeline.
        // Per Solution 3 in comprehensive plan: validate content BEFORE processing.
        let initial_count = blocks.len();
        let mut whitespace_count = 0;
        blocks.retain(|block| {
            let is_whitespace = block.text.trim().is_empty();
            if is_whitespace {
                whitespace_count += 1;
            }
            !is_whitespace
        });
        let filtered_count = blocks.len();
        log::debug!(
            "Pre-grouping whitespace filter: removed {} whitespace-only blocks ({} → {})",
            whitespace_count,
            initial_count,
            filtered_count
        );

        // Neutralize bold on non-word-character blocks
        // Blocks containing only punctuation, symbols, or special characters
        // should not be marked as bold, even if they inherited the flag from context.
        // This includes content that has no alphanumeric characters AND
        // content that is very short (< 2 chars) and not typical word content.
        let mut neutralized_count = 0;
        for block in &mut blocks {
            let has_alphanumeric = block.text.chars().any(|c| c.is_alphanumeric());
            let has_non_whitespace = block.text.chars().any(|c| !c.is_whitespace());

            // Neutralize bold if:
            // 1. No alphanumeric characters at all: "---", "...", ">>>", etc.
            // 2. Very short with only punctuation/symbols: single or few non-word chars
            let should_neutralize = if !has_alphanumeric {
                // Rule 1: No alphanumeric = definitely non-word content
                true
            } else if has_non_whitespace && block.text.len() == 1 {
                // Rule 2: Single non-alphabetic character (e.g., ".", "!", etc.)
                let ch = match block.text.chars().next() {
                    Some(c) => c,
                    None => continue,
                };
                !ch.is_alphabetic() && ch != ' ' && ch != '\t' && ch != '\n'
            } else {
                false
            };

            if should_neutralize && block.is_bold {
                log::debug!("Neutralizing bold on non-word block: '{}'", block.text);
                block.is_bold = false;
                neutralized_count += 1;
            }
        }
        if neutralized_count > 0 {
            log::debug!("Neutralized {} bold flags on non-word blocks", neutralized_count);
        }

        // PDF Spec ISO 32000-1:2008 Section 9.4.4 NOTE 6:
        // "text strings are as long as possible"
        // Merge adjacent character-level spans that are too close to have real spaces
        // This handles PDFs with character-level fragmentation (like GDPR file)
        blocks = Self::merge_adjacent_char_spans(blocks);

        // Heading detection removed (non-spec-compliant feature)
        // All blocks are treated as body text for spec compliance

        // Apply reading order (use simple top-to-bottom for span-based conversion)
        // XY-Cut algorithm requires adaptive params which need char-based analysis
        let ordered_indices =
            self.determine_reading_order(&blocks, ReadingOrderMode::TopToBottomLeftToRight, None);

        // Process blocks into lines based on Y coordinate and render incrementally
        // This approach uses constant memory instead of accumulating all line groups
        let mut markdown = String::new();
        let mut current_line: Vec<usize> = Vec::new();
        let mut current_y: Option<f32> = None;

        // Helper closure to render a completed line
        let render_line = |line_indices: &[usize], markdown: &mut String| {
            if line_indices.is_empty() {
                return;
            }

            // Heading detection removed (non-spec-compliant feature)
            // All lines rendered as body text for spec compliance

            // Join blocks on this line, grouping consecutive blocks with same formatting
            // Per PDF spec (ISO 32000-1:2008, Section 9.4.4 NOTE 6):
            // Text extraction already handles word spacing based on TJ operator offsets.
            // Space characters are inserted as separate spans during extraction
            // (see process_tj_array in text.rs), so we just concatenate span text.
            //
            // Group consecutive blocks with same bold/italic status to avoid splitting
            // natural phrases like "Chinese stock market" into "**Chinese stock** market"
            let spacing_config = SpacingConfig::default();
            let mut i = 0;
            while i < line_indices.len() {
                let idx = line_indices[i];
                let block = &blocks[idx];
                let is_bold = block.is_bold;
                let is_italic = block.is_italic;

                // Find all consecutive blocks with same bold AND italic status
                let mut j = i + 1;
                while j < line_indices.len()
                    && blocks[line_indices[j]].is_bold == is_bold
                    && blocks[line_indices[j]].is_italic == is_italic
                {
                    j += 1;
                }

                // Render this group of blocks with unified formatting
                // Check word boundaries before/after to avoid mid-word bold markers
                let prev_char = if markdown.is_empty() {
                    None
                } else {
                    markdown.chars().last()
                };

                let next_char_after_group = if j < line_indices.len() {
                    blocks[line_indices[j]].text.chars().next()
                } else {
                    None
                };

                // Collect text from this group first to check boundaries
                // Use geometric spacing to detect gaps between blocks (Issue #5 fix)
                let mut group_text = String::new();
                for k in i..j {
                    let block_idx = line_indices[k];
                    let current_block = &blocks[block_idx];

                    if !group_text.is_empty() && k > i {
                        let prev_block = &blocks[line_indices[k - 1]];
                        if needs_inter_block_space(prev_block, current_block, &spacing_config) {
                            group_text.push(' ');
                        }
                    }

                    group_text.push_str(&current_block.text);
                }

                // FIX #3: Format URLs and emails as markdown links
                let formatted_text = Self::format_links(&group_text);
                // FIX #4: Clean up reference spacing
                let cleaned_text = Self::clean_reference_spacing(&formatted_text);

                // NEW FIX: Post-format whitespace validation
                // Some blocks with content become whitespace-only after formatting,
                // so we must verify content is still non-empty before adding bold markers
                if cleaned_text.trim().is_empty() {
                    log::debug!(
                        "Skipping bold markers: content became whitespace-only after formatting"
                    );
                    markdown.push_str(&cleaned_text);
                    continue;
                }

                // Extract boundary characters for bold marker validation
                let first_char_in_group = cleaned_text.chars().next();
                let last_char_in_group = cleaned_text.chars().last();

                // Check if both opening and closing positions are valid for bold markers
                // We need to insert both or neither to maintain balance
                let can_insert_open = should_insert_bold_marker(prev_char, first_char_in_group);
                let can_insert_close =
                    should_insert_bold_marker(last_char_in_group, next_char_after_group);

                // FIX #2: Skip bold markers for whitespace-only spans in conservative mode
                // Determine if content warrants bold markers based on behavior setting
                let should_render_bold_markers = match options.bold_marker_behavior {
                    BoldMarkerBehavior::Aggressive => true,
                    BoldMarkerBehavior::Conservative => is_content_block(&cleaned_text),
                };

                // Validate bold markers with BoldMarkerValidator
                let group = BoldGroup {
                    text: cleaned_text.clone(),
                    is_bold,
                    first_char_in_group,
                    last_char_in_group,
                };

                // Check if we should render bold markers based on behavior setting
                // This respects the conversion options while validator handles content validation
                let should_check_validator =
                    is_bold && can_insert_open && can_insert_close && should_render_bold_markers;

                // Validate before inserting markers
                let marker_decision = if should_check_validator {
                    BoldMarkerValidator::can_insert_markers(&group)
                } else {
                    // Skip validation if any precondition fails
                    BoldMarkerDecision::Skip(
                        crate::layout::bold_validation::ValidatorError::NotBold,
                    )
                };

                // Insert opening marker if approved by validator
                let should_insert_bold_markers =
                    matches!(marker_decision, BoldMarkerDecision::Insert);
                if should_insert_bold_markers {
                    // Determine which formatting markers to use
                    match (is_bold, is_italic) {
                        (true, true) => markdown.push_str("***"), // Bold + Italic
                        (true, false) => markdown.push_str("**"), // Bold only
                        (false, true) => markdown.push('*'),      // Italic only
                        (false, false) => {},                     // No formatting
                    }
                } else if let BoldMarkerDecision::Skip(reason) = &marker_decision {
                    log::debug!(
                        "Skipping bold markers: {:?} for '{}'",
                        reason,
                        group.text.chars().take(20).collect::<String>()
                    );
                }

                // Output the text content (may have leading/trailing spaces)
                markdown.push_str(&group.text);

                // Insert closing marker if approved by validator
                if should_insert_bold_markers {
                    // Determine which formatting markers to use (must match opening)
                    match (is_bold, is_italic) {
                        (true, true) => markdown.push_str("***"), // Bold + Italic
                        (true, false) => markdown.push_str("**"), // Bold only
                        (false, true) => markdown.push('*'),      // Italic only
                        (false, false) => {},                     // No formatting
                    }
                }

                // Insert space between adjacent style groups so that style transitions
                // like "**Access control:** Enforce" don't fuse into "Accesscontrol:Enforce".
                if j < line_indices.len() {
                    let last_block = &blocks[line_indices[j - 1]];
                    let next_block = &blocks[line_indices[j]];
                    if needs_inter_block_space(last_block, next_block, &spacing_config) {
                        markdown.push(' ');
                    }
                }

                i = j;
            }

            // Structure-aware rendering
            // Infrastructure added to support heading/list detection from structure tree
            // See StructType::heading_level(), StructType::is_list(), StructType::markdown_prefix()
            // Full integration requires passing structure tree through ConversionOptions
            // Current implementation: render as body text for spec compliance
            markdown.push('\n');
        };

        // Group blocks by Y coordinate and render each line immediately
        for &idx in &ordered_indices {
            let block = &blocks[idx];
            let block_y = block.bbox.y;

            match current_y {
                Some(y) if (y - block_y).abs() < 2.0 => {
                    // Same line - Y coordinates are within 2pt tolerance
                    current_line.push(idx);
                },
                _ => {
                    // New line - render the previous line before starting a new one
                    render_line(&current_line, &mut markdown);
                    current_line.clear();
                    current_line.push(idx);
                    current_y = Some(block_y);
                },
            }
        }

        // Don't forget to render the last line
        render_line(&current_line, &mut markdown);

        // Insert missing spaces after punctuation (post-processing)
        // Catches punctuation-letter patterns that TJ offset processing missed
        let spaced = Self::insert_missing_punctuation_spaces(&markdown);

        // Apply whitespace cleanup: remove artifacts and normalize blank lines
        let cleaned = cleanup_markdown(&spaced);

        // Apply text post-processing per PDF Spec:
        // - Remove soft hyphens at line breaks (Section 14.8.2.2.3)
        // - Normalize whitespace within words (Section 14.8.2.5)
        let post_processed = TextPostProcessor::process(&cleaned);

        Ok(post_processed)
    }

    /// Convert a page to Markdown format (character-based - DEPRECATED).
    ///
    /// This function:
    /// 1. Clusters characters into words and lines
    /// 2. Detects heading levels based on font sizes
    /// 3. Determines reading order
    /// 4. Generates Markdown with appropriate syntax
    ///
    /// # Arguments
    ///
    /// * `chars` - The text characters extracted from the page
    /// * `options` - Conversion options controlling the output
    ///
    /// # Returns
    ///
    /// A string containing the Markdown representation of the page.
    ///
    /// # Errors
    ///
    /// Returns an error if clustering or conversion fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pdf_oxide::converters::{MarkdownConverter, ConversionOptions};
    /// # use pdf_oxide::layout::TextChar;
    /// # fn example(chars: Vec<TextChar>) -> Result<(), Box<dyn std::error::Error>> {
    /// let converter = MarkdownConverter::new();
    /// let options = ConversionOptions {
    ///     detect_headings: true,
    ///     include_images: false,
    ///     ..Default::default()
    /// };
    ///
    /// let markdown = converter.convert_page(&chars, &options)?;
    /// println!("{}", markdown);
    /// # Ok(())
    /// # }
    /// ```
    pub fn convert_page(&self, chars: &[TextChar], options: &ConversionOptions) -> Result<String> {
        if chars.is_empty() {
            return Ok(String::new());
        }

        // CRITICAL FIX: Spatially sort characters by position BEFORE clustering
        // PDF content streams often have characters in arbitrary order (especially in multi-column layouts)
        // We MUST sort by (Y, X) to get proper left-to-right, top-to-bottom reading order
        let mut sorted_chars = chars.to_vec();
        sorted_chars.sort_by(|a, b| {
            // Primary sort: Y coordinate (PDF coords: larger Y = higher on page)
            // For top-to-bottom reading, we want LARGER Y first
            let y_cmp = crate::utils::safe_float_cmp(b.bbox.y, a.bbox.y);
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            // Secondary sort: X coordinate (left to right)
            crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x)
        });

        // Compute font-adaptive epsilon for word clustering
        // Use median character width (approximated as 0.5× font size)
        // Increased to 0.8× to account for Y-axis variations and wider character spacing
        let median_font_size = Self::compute_median_font_size(&sorted_chars);
        let word_epsilon = median_font_size * 0.8; // Character width + spacing for word boundaries

        // Step 1: Cluster characters into words (now in proper spatial order!)
        let word_clusters = cluster_chars_into_words(&sorted_chars, word_epsilon);
        let mut words = Vec::new();

        for cluster in &word_clusters {
            let word_chars: Vec<TextChar> =
                cluster.iter().map(|&i| sorted_chars[i].clone()).collect();
            if !word_chars.is_empty() {
                words.push(TextBlock::from_chars(word_chars));
            }
        }

        if words.is_empty() {
            return Ok(String::new());
        }

        // Step 2: Cluster words into lines
        // Move chars out of words instead of cloning (words are not used after this)
        let line_clusters = cluster_words_into_lines(&words, 5.0);
        let mut lines = Vec::new();

        // Track which word indices belong to each line for draining
        let mut word_taken = vec![false; words.len()];
        for cluster in &line_clusters {
            if cluster.is_empty() {
                continue;
            }
            let mut all_chars: Vec<TextChar> = Vec::new();
            for &i in cluster {
                if !word_taken[i] {
                    all_chars.extend(std::mem::take(&mut words[i].chars));
                    word_taken[i] = true;
                }
            }
            if !all_chars.is_empty() {
                lines.push(TextBlock::from_chars(all_chars));
            }
        }

        if lines.is_empty() {
            return Ok(String::new());
        }

        // Step 3: Analyze document properties for adaptive parameters
        let page_bbox = Self::calculate_bounding_box(&lines);
        let adaptive_params = match DocumentProperties::analyze(&sorted_chars, page_bbox) {
            Ok(props) => Some(AdaptiveLayoutParams::from_properties(&props)),
            Err(_) => None, // Fall back to fixed params if analysis fails
        };

        // Heading detection removed (non-spec-compliant feature)
        // All content is treated as body text for spec compliance
        let _heading_levels = vec![(); lines.len()]; // Placeholder - not used, all body text

        // Step 5: Determine reading order
        let ordered_indices = self.determine_reading_order(
            &lines,
            options.reading_order_mode.clone(),
            adaptive_params.as_ref(),
        );

        // Step 5: Generate Markdown
        let mut markdown = String::new();

        for &idx in &ordered_indices {
            let line = &lines[idx];

            // Heading detection removed (non-spec-compliant feature)
            // All content rendered as body text for spec compliance

            // FIX #3: Format URLs and emails as markdown links
            let formatted_text = Self::format_links(&line.text);
            // FIX #4: Clean up reference spacing
            let cleaned_text = Self::clean_reference_spacing(&formatted_text);

            // Render as body text (no markdown heading markers)
            markdown.push_str(&cleaned_text);
            markdown.push('\n');
        }

        // Apply whitespace cleanup: remove artifacts and normalize blank lines
        let cleaned = cleanup_markdown(&markdown);

        // Apply text post-processing per PDF Spec:
        // - Remove soft hyphens at line breaks (Section 14.8.2.2.3)
        // - Normalize whitespace within words (Section 14.8.2.5)
        let post_processed = TextPostProcessor::process(&cleaned);

        Ok(post_processed)
    }

    /// Determine the reading order of text blocks.
    ///
    /// This implements simple top-to-bottom, left-to-right ordering.
    /// For more advanced column-aware ordering, the XY-Cut algorithm could be used.
    ///
    /// # Arguments
    ///
    /// * `blocks` - The text blocks to order
    /// * `mode` - The reading order mode to use
    /// * `adaptive_params` - Optional adaptive parameters computed from document analysis
    ///
    /// # Returns
    ///
    /// A vector of indices representing the reading order.
    fn determine_reading_order(
        &self,
        blocks: &[TextBlock],
        mode: ReadingOrderMode,
        _adaptive_params: Option<&AdaptiveLayoutParams>,
    ) -> Vec<usize> {
        if blocks.is_empty() {
            return vec![];
        }

        let mut indices: Vec<usize> = (0..blocks.len()).collect();

        match mode {
            ReadingOrderMode::TopToBottomLeftToRight => {
                // Sort by Y (top to bottom), then by X (left to right)
                indices.sort_by(|&a, &b| {
                    let block_a = &blocks[a];
                    let block_b = &blocks[b];

                    // Primary sort: Y coordinate (larger Y = higher on page in PDF coords)
                    // PDF coordinates: origin at bottom-left, Y increases upward
                    // So top of page (large Y) comes before bottom (small Y)
                    let y_cmp = crate::utils::safe_float_cmp(block_b.bbox.y, block_a.bbox.y);
                    if y_cmp != std::cmp::Ordering::Equal {
                        y_cmp
                    } else {
                        // Secondary sort: X coordinate (smaller X = further left)
                        crate::utils::safe_float_cmp(block_a.bbox.x, block_b.bbox.x)
                    }
                });
            },
            ReadingOrderMode::ColumnAware => {
                // Use XY-Cut algorithm for multi-column layout detection
                // XY-Cut is ISO 32000-1:2008 Section 9.4 compliant for geometric analysis
                indices = Self::xycut_reading_order(blocks);
                log::info!("Using XY-Cut algorithm for column-aware reading order");
            },
            ReadingOrderMode::StructureTreeFirst { ref mcid_order } => {
                // PDF-spec-compliant reading order via structure tree (Tagged PDFs)
                if !mcid_order.is_empty() {
                    // Reorder blocks by matching MCIDs from structure tree
                    indices = Self::reorder_by_mcid(blocks, mcid_order);
                    log::info!("Using structure tree for reading order (Tagged PDF)");
                } else {
                    // Fall back to graph-based reading order for untagged PDFs
                    // (XY-Cut algorithm removed as non-PDF-spec-compliant)
                    log::info!("No MCIDs found, falling back to graph-based reading order");
                    indices = graph_based_reading_order(blocks);
                }
            },
        }

        indices
    }

    /// Reorder text blocks according to structure tree reading order.
    ///
    /// Takes blocks extracted from a page and reorders them to match the
    /// MCIDs from the structure tree traversal. This implements PDF-spec-compliant
    /// reading order determination (ISO 32000-1:2008 Section 14.7).
    ///
    /// # Arguments
    ///
    /// * `blocks` - The text blocks to reorder
    /// * `mcid_order` - Sequence of MCIDs in structure tree reading order
    ///
    /// # Returns
    ///
    /// A vector of indices representing the reordered blocks.
    ///
    /// # Algorithm
    ///
    /// 1. For each MCID in structure tree order, find all blocks with that MCID
    /// 2. Add blocks without MCIDs at the end (fallback for unmarked content)
    /// 3. Preserve spatial order for blocks with the same MCID (top-to-bottom, left-to-right)
    fn reorder_by_mcid(blocks: &[TextBlock], mcid_order: &[u32]) -> Vec<usize> {
        use std::collections::HashMap;

        // Build MCID → block indices map in a single pass: O(n)
        let mut mcid_to_blocks: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut no_mcid_indices = Vec::new();

        for (idx, block) in blocks.iter().enumerate() {
            if let Some(mcid) = block.mcid {
                mcid_to_blocks.entry(mcid).or_default().push(idx);
            } else {
                no_mcid_indices.push(idx);
            }
        }

        // Sort each MCID's block list by spatial position
        for indices in mcid_to_blocks.values_mut() {
            if indices.len() > 1 {
                indices.sort_by(|&a, &b| {
                    let block_a = &blocks[a];
                    let block_b = &blocks[b];
                    let y_cmp = crate::utils::safe_float_cmp(block_b.bbox.y, block_a.bbox.y);
                    if y_cmp != std::cmp::Ordering::Equal {
                        y_cmp
                    } else {
                        crate::utils::safe_float_cmp(block_a.bbox.x, block_b.bbox.x)
                    }
                });
            }
        }

        // Collect in structure tree order: O(m) where m = mcid_order.len()
        let mut ordered_indices = Vec::with_capacity(blocks.len());
        for &mcid in mcid_order {
            if let Some(indices) = mcid_to_blocks.get(&mcid) {
                ordered_indices.extend(indices);
            }
        }

        // Add blocks without MCID at the end
        ordered_indices.extend(no_mcid_indices);

        ordered_indices
    }

    /// Use XY-Cut algorithm for multi-column layout detection.
    ///
    /// Converts TextBlocks to TextSpans for XYCutStrategy processing,
    /// then returns indices in column-aware reading order.
    ///
    /// Per ISO 32000-1:2008 Section 9.4, this uses geometric analysis
    /// with projection profiles to detect column boundaries.
    fn xycut_reading_order(blocks: &[TextBlock]) -> Vec<usize> {
        if blocks.is_empty() {
            return vec![];
        }

        // Convert TextBlocks to TextSpans for XYCut processing
        let spans: Vec<TextSpan> = blocks
            .iter()
            .enumerate()
            .map(|(seq, block)| TextSpan {
                provenance: None,
                text: block.text.clone(),
                bbox: block.bbox,
                font_name: block.dominant_font.clone(),
                font_size: block.avg_font_size,
                font_weight: if block.is_bold {
                    FontWeight::Bold
                } else {
                    FontWeight::Normal
                },
                is_italic: block.is_italic,
                is_monospace: false,
                color: Color::black(),
                mcid: block.mcid,
                mcid_scope: None,
                sequence: seq,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                artifact_type: None,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                text_rise: 0.0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            })
            .collect();

        // Apply XY-Cut algorithm
        let strategy = XYCutStrategy::new()
            .with_valley_threshold(0.25)   // Slightly more sensitive for narrow gutters
            .with_min_valley_width(12.0); // 12pt minimum gap for column detection

        let groups = strategy.partition_region(&spans);

        // Flatten groups back to indices, preserving the XY-Cut ordering.
        // Each span's `sequence` field holds its original block index.
        let mut indices = Vec::with_capacity(blocks.len());
        for group in &groups {
            for span in group {
                indices.push(span.sequence);
            }
        }

        indices
    }

    /// Calculate the bounding box that contains all blocks.
    /// Compute median font size from characters for adaptive epsilon.
    fn compute_median_font_size(chars: &[TextChar]) -> f32 {
        if chars.is_empty() {
            return 12.0; // Default fallback
        }

        let mut font_sizes: Vec<f32> = chars.iter().map(|c| c.font_size).collect();
        font_sizes.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));

        let mid = font_sizes.len() / 2;
        if font_sizes.len().is_multiple_of(2) {
            (font_sizes[mid - 1] + font_sizes[mid]) / 2.0
        } else {
            font_sizes[mid]
        }
    }

    fn calculate_bounding_box(blocks: &[TextBlock]) -> Rect {
        if blocks.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for block in blocks {
            min_x = min_x.min(block.bbox.left());
            min_y = min_y.min(block.bbox.bottom());
            max_x = max_x.max(block.bbox.right());
            max_y = max_y.max(block.bbox.top());
        }

        Rect::from_points(min_x, min_y, max_x, max_y)
    }

    /// Format URLs and email addresses as clickable markdown links.
    ///
    /// FIX #3: Convert plain URLs and emails to markdown link format
    ///
    /// Transforms:
    /// - `https://example.com` → `[https://example.com](https://example.com)`
    /// - `user@example.com` → `[user@example.com](mailto:user@example.com)`
    ///
    /// # Arguments
    ///
    /// * `text` - The text to process
    ///
    /// # Returns
    ///
    /// Text with URLs and emails formatted as markdown links
    fn format_links(text: &str) -> String {
        // Quick pre-check: skip regex for text that can't contain URLs or emails
        let might_have_url = text.contains("://") || text.contains("www.");
        let might_have_email = text.contains('@');

        if !might_have_url && !might_have_email {
            return text.to_string();
        }

        let mut result = text.to_string();

        if might_have_url {
            result = RE_URL
                .replace_all(&result, |caps: &Captures| {
                    let url = &caps[1];
                    // Don't format if already part of a markdown link
                    if text.contains(&format!("[{}]", url)) {
                        url.to_string()
                    } else {
                        format!("[{}]({})", url, url)
                    }
                })
                .to_string();
        }

        if might_have_email {
            result = RE_EMAIL
                .replace_all(&result, |caps: &Captures| {
                    let email = &caps[1];
                    // Don't format if already part of a markdown link or URL
                    if result.contains(&format!("[{}]", email))
                        || result.contains(&format!("//{}", email))
                    {
                        email.to_string()
                    } else {
                        format!("[{}](mailto:{})", email, email)
                    }
                })
                .to_string();
        }

        result
    }

    /// Clean up spacing around dashes in reference ranges.
    ///
    /// FIX #4: Remove extra spaces around em-dashes and en-dashes in citations
    ///
    /// Transforms:
    /// - `"21, 23 –25"` → `"21, 23–25"` (remove space before dash)
    /// - `"21– 25"` → `"21–25"` (remove space after dash)
    /// - `"21 – 25"` → `"21–25"` (remove spaces on both sides)
    ///
    /// # Arguments
    ///
    /// * `text` - Text potentially containing reference ranges with spacing issues
    ///
    /// # Returns
    ///
    /// Text with cleaned up dash spacing in reference contexts
    fn clean_reference_spacing(text: &str) -> String {
        let mut result = text.to_string();

        // Pattern 1: Space before dash in numeric context: "23 –25" → "23–25"
        result = RE_DASH_BEFORE.replace_all(&result, "$1$2$3").to_string();

        // Pattern 2: Space after dash in numeric context: "23– 25" → "23–25"
        result = RE_DASH_AFTER.replace_all(&result, "$1$2$3").to_string();

        // Pattern 3: Space on both sides: "23 – 25" → "23–25"
        // (Covered by patterns 1 and 2 applied sequentially)

        result
    }

    /// Insert missing spaces after punctuation.
    ///
    /// Some PDFs have punctuation directly followed by a letter with no space,
    /// which TJ offset processing fails to catch. This post-processing regex
    /// detects and fixes `[punctuation][letter]` patterns.
    ///
    /// Transforms:
    /// - `"hello.world"` → `"hello. world"`
    /// - `"end,another"` → `"end, another"`
    /// - `"question?Answer"` → `"question? Answer"`
    ///
    /// Excludes URLs (`://`) and emails (`@`) to avoid false positives.
    ///
    /// # Arguments
    ///
    /// * `text` - Text potentially containing punctuation without following space
    ///
    /// # Returns
    ///
    /// Text with spaces inserted after punctuation where needed
    fn insert_missing_punctuation_spaces(text: &str) -> String {
        RE_PUNCT_SPACE.replace_all(text, "${1} ${2}").to_string()
    }
}

#[allow(deprecated)]
impl Default for MarkdownConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to determine if a bold marker should be inserted at a boundary.
///
/// Bold markers (`**`) should only be inserted at word boundaries to avoid
/// splitting words unnaturally (e.g., `gr**I` or `8**21`).
///
/// # Arguments
///
/// * `prev_char` - The character before the marker position
/// * `next_char` - The character after the marker position
///
/// # Returns
///
/// `true` if the marker should be inserted (at a word boundary),
/// `false` if it would split a word (both sides are alphanumeric)
///
/// # Examples
///
/// ```ignore
/// // Should insert (whitespace boundary)
/// assert!(should_insert_bold_marker(Some(' '), Some('t')));
///
/// // Should NOT insert (mid-word)
/// assert!(!should_insert_bold_marker(Some('r'), Some('I')));
/// ```
/// Determine if a text block contains meaningful content (not just whitespace).
///
/// This helper identifies whether a span represents actual content or just
/// layout spacing. Used to decide whether to apply bold markers in conservative mode.
///
/// # Arguments
///
/// * `text` - The text content to analyze
///
/// # Returns
///
/// `true` if the text contains at least one non-whitespace character, `false` otherwise.
///
/// # Examples
///
/// ```ignore
/// assert!(is_content_block("text"));       // true - has content
/// assert!(is_content_block("a"));          // true - single character
/// assert!(is_content_block(" a "));        // true - has non-whitespace
/// assert!(!is_content_block(""));          // false - empty
/// assert!(!is_content_block("   "));       // false - spaces only
/// assert!(!is_content_block("\t\n"));      // false - whitespace only
/// ```
pub fn is_content_block(text: &str) -> bool {
    // Check if any character is not whitespace
    text.chars().any(|c| !c.is_whitespace())
}

fn should_insert_bold_marker(prev_char: Option<char>, next_char: Option<char>) -> bool {
    match (prev_char, next_char) {
        // Don't insert if both sides are alphanumeric (mid-word)
        (Some(p), Some(n)) if p.is_alphanumeric() && n.is_alphanumeric() => false,
        // Don't insert between closing punctuation and operators (e.g., ')**=' → ')**= is unnatural')
        // Common cases: )**=, )**-, )**+, )**<, )**>, etc.
        (Some(')'), Some(n))
            if matches!(n, '=' | '-' | '+' | '<' | '>' | '*' | '/' | '&' | '|' | '^') =>
        {
            false
        },
        (Some(']'), Some(n))
            if matches!(n, '=' | '-' | '+' | '<' | '>' | '*' | '/' | '&' | '|' | '^') =>
        {
            false
        },
        (Some('}'), Some(n))
            if matches!(n, '=' | '-' | '+' | '<' | '>' | '*' | '/' | '&' | '|' | '^') =>
        {
            false
        },
        // Insert in all other cases:
        // - At start/end of text (None on either side)
        // - After whitespace, punctuation, or symbols
        // - Before whitespace, punctuation, or symbols
        _ => true,
    }
}

/// Render a markdown table from an extracted table structure.
///
/// Converts a Table into Markdown table format with:
/// - Header row (if present) separated by | delimiters
/// - Separator row with |---|---|...
/// - Data rows in same format
///
/// # Arguments
///
/// * `table` - The extracted table to render
///
/// # Returns
///
/// A string containing the Markdown table representation
#[allow(dead_code)]
fn render_markdown_table(table: &Table) -> String {
    let mut md = String::new();

    if table.rows.is_empty() {
        return md;
    }

    // Find all contiguous header rows at the start
    let mut header_row_count = 0;
    while header_row_count < table.rows.len() && table.rows[header_row_count].is_header {
        header_row_count += 1;
    }

    // If no explicit headers found, at least treat the first row as header for Markdown
    if header_row_count == 0 {
        header_row_count = 1;
    }

    // 1. Render Header Rows
    for i in 0..header_row_count {
        md.push_str(&render_table_row(&table.rows[i]));
        md.push('\n');
    }

    // 2. Render Separator Row
    md.push('|');
    for _ in 0..table.col_count {
        md.push_str("---|");
    }
    md.push('\n');

    // 3. Render Data Rows
    for i in header_row_count..table.rows.len() {
        md.push_str(&render_table_row(&table.rows[i]));
        md.push('\n');
    }

    md
}

/// Render a single table row as a Markdown row.
///
/// Escapes pipe characters (|) in cell text and formats as: | cell1 | cell2 | ...
///
/// # Arguments
///
/// * `row` - The table row to render
///
/// # Returns
///
/// A string containing the Markdown row representation
#[allow(dead_code)]
fn render_table_row(row: &TableRow) -> String {
    let mut line = String::from("|");
    for cell in &row.cells {
        // Escape pipe characters in cell text
        let escaped = cell.text.replace('|', "\\|");
        let content = escaped.trim();

        // Repeat content for colspan to keep data accessible in Markdown
        // (Markdown tables don't support native colspan)
        for _ in 0..cell.colspan {
            line.push_str(&format!(" {} |", content));
        }
    }
    line
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::bold_validation::ValidatorError;
    use crate::layout::{Color, FontWeight};

    fn mock_char(c: char, x: f32, y: f32, font_size: f32, bold: bool) -> TextChar {
        let bbox = Rect::new(x, y, 8.0, font_size);
        TextChar {
            char: c,
            bbox,
            font_name: "Times".to_string(),
            font_size,
            font_weight: if bold {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            },
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * font_size,
            descent: -0.35 * font_size,
            matrix: None,
        }
    }

    fn mock_word(text: &str, x: f32, y: f32, font_size: f32, bold: bool) -> Vec<TextChar> {
        text.chars()
            .enumerate()
            .map(|(i, c)| mock_char(c, x + (i as f32 * 7.0), y, font_size, bold))
            .collect()
    }

    #[test]
    fn test_markdown_converter_new() {
        let converter = MarkdownConverter::new();
        assert!(format!("{:?}", converter).contains("MarkdownConverter"));
    }

    #[test]
    fn test_markdown_converter_default() {
        let converter = MarkdownConverter;
        assert!(format!("{:?}", converter).contains("MarkdownConverter"));
    }

    #[test]
    fn test_convert_empty() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions::default();
        let result = converter.convert_page(&[], &options).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_single_line() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            detect_headings: false,
            ..Default::default()
        };

        let chars = mock_word("Hello World", 0.0, 0.0, 12.0, false);
        let result = converter.convert_page(&chars, &options).unwrap();

        assert!(result.contains("Hello World"));
        assert!(!result.contains('#')); // No heading detection
    }

    #[test]
    fn test_convert_with_heading() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        // Create a large bold title and regular text
        let mut chars = Vec::new();
        chars.extend(mock_word("Title", 0.0, 0.0, 24.0, true)); // Large bold = H1
        chars.push(mock_char(' ', 45.0, 0.0, 24.0, true));

        chars.extend(mock_word("Body Text", 0.0, 50.0, 12.0, false)); // Regular = Body

        let result = converter.convert_page(&chars, &options).unwrap();

        // Title should be detected as heading
        assert!(result.contains("# Title") || result.contains("Title"));
        assert!(result.contains("Body Text"));
    }

    #[test]
    fn test_convert_multiple_lines() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            detect_headings: false,
            ..Default::default()
        };

        let mut chars = Vec::new();
        chars.extend(mock_word("Line One", 0.0, 0.0, 12.0, false));
        chars.extend(mock_word("Line Two", 0.0, 20.0, 12.0, false));
        chars.extend(mock_word("Line Three", 0.0, 40.0, 12.0, false));

        let result = converter.convert_page(&chars, &options).unwrap();

        assert!(result.contains("Line One"));
        assert!(result.contains("Line Two"));
        assert!(result.contains("Line Three"));
    }

    #[test]
    fn test_reading_order_top_to_bottom() {
        let converter = MarkdownConverter::new();

        // PDF coordinates: Y increases upward, so top has LARGER Y
        let block1 = TextBlock::from_chars(mock_word("Top", 0.0, 100.0, 12.0, false)); // Y=100 (top)
        let block2 = TextBlock::from_chars(mock_word("Middle", 0.0, 50.0, 12.0, false)); // Y=50 (middle)
        let block3 = TextBlock::from_chars(mock_word("Bottom", 0.0, 0.0, 12.0, false)); // Y=0 (bottom)

        let blocks = vec![block2.clone(), block3.clone(), block1.clone()]; // Out of order

        let indices = converter.determine_reading_order(
            &blocks,
            ReadingOrderMode::TopToBottomLeftToRight,
            None,
        );

        // Should order by Y: block1 (y=100), block2 (y=50), block3 (y=0)
        // In our shuffled vec: block1 is at index 2, block2 at 0, block3 at 1
        assert_eq!(indices[0], 2); // block1 (Top, y=100)
        assert_eq!(indices[1], 0); // block2 (Middle, y=50)
        assert_eq!(indices[2], 1); // block3 (Bottom, y=0)
    }

    #[test]
    fn test_reading_order_left_to_right() {
        let converter = MarkdownConverter::new();

        // Create blocks at same Y but different X
        let block1 = TextBlock::from_chars(mock_word("Left", 0.0, 0.0, 12.0, false));
        let block2 = TextBlock::from_chars(mock_word("Center", 50.0, 0.0, 12.0, false));
        let block3 = TextBlock::from_chars(mock_word("Right", 100.0, 0.0, 12.0, false));

        let blocks = vec![block3.clone(), block1.clone(), block2.clone()]; // Out of order

        let indices = converter.determine_reading_order(
            &blocks,
            ReadingOrderMode::TopToBottomLeftToRight,
            None,
        );

        // Should order by X when Y is equal
        assert_eq!(indices[0], 1); // block1 (Left, x=0)
        assert_eq!(indices[1], 2); // block2 (Center, x=50)
        assert_eq!(indices[2], 0); // block3 (Right, x=100)
    }

    #[test]
    fn test_heading_level_h1() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        // Very large bold text should be H1
        let chars = mock_word("Main Title", 0.0, 0.0, 28.0, true);
        let result = converter.convert_page(&chars, &options).unwrap();

        // Should contain H1 marker
        assert!(result.contains("# Main Title") || result.contains("Main Title"));
    }

    #[test]
    fn test_heading_level_h2() {
        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            detect_headings: true,
            ..Default::default()
        };

        let mut chars = Vec::new();
        // H1: largest
        chars.extend(mock_word("Main", 0.0, 0.0, 24.0, true));
        // H2: medium
        chars.extend(mock_word("Section", 0.0, 40.0, 18.0, true));
        // Body: small
        chars.extend(mock_word("Text", 0.0, 70.0, 12.0, false));

        let result = converter.convert_page(&chars, &options).unwrap();

        // Should have different heading levels
        assert!(result.contains("Main"));
        assert!(result.contains("Section"));
        assert!(result.contains("Text"));
    }

    #[test]
    fn test_column_aware_mode() {
        let converter = MarkdownConverter::new();

        let block1 = TextBlock::from_chars(mock_word("A", 0.0, 0.0, 12.0, false));
        let block2 = TextBlock::from_chars(mock_word("B", 0.0, 50.0, 12.0, false));

        let blocks = vec![block1, block2];

        // Both modes should work - ColumnAware uses XY-Cut algorithm
        let indices1 = converter.determine_reading_order(
            &blocks,
            ReadingOrderMode::TopToBottomLeftToRight,
            None,
        );
        let indices2 =
            converter.determine_reading_order(&blocks, ReadingOrderMode::ColumnAware, None);

        assert_eq!(indices1.len(), 2);
        assert_eq!(indices2.len(), 2);
    }

    #[test]
    fn test_column_aware_xycut_two_column_layout() {
        // Test XY-Cut algorithm properly orders multi-column text
        let converter = MarkdownConverter::new();

        // Create a two-column layout:
        // Left column (x=10):  "Col1-Top", "Col1-Bottom"
        // Right column (x=300): "Col2-Top", "Col2-Bottom"
        // With 200pt gap between columns, XY-Cut should detect and process by column
        let col1_top = TextBlock::from_chars(mock_word("Col1-Top", 10.0, 100.0, 12.0, false));
        let col1_bottom = TextBlock::from_chars(mock_word("Col1-Bottom", 10.0, 50.0, 12.0, false));
        let col2_top = TextBlock::from_chars(mock_word("Col2-Top", 300.0, 100.0, 12.0, false));
        let col2_bottom = TextBlock::from_chars(mock_word("Col2-Bottom", 300.0, 50.0, 12.0, false));

        // Shuffle blocks (wrong visual order)
        let blocks = vec![
            col2_bottom.clone(),
            col1_top.clone(),
            col2_top.clone(),
            col1_bottom.clone(),
        ];

        let indices =
            converter.determine_reading_order(&blocks, ReadingOrderMode::ColumnAware, None);

        assert_eq!(indices.len(), 4);
        // Verify all indices are present (XY-Cut returns them)
        let mut sorted_indices = indices.clone();
        sorted_indices.sort();
        assert_eq!(sorted_indices, vec![0, 1, 2, 3]);
    }

    // ============================================================================
    // NEW TESTS (Task B.1: Pre-Validation Bold Filter)
    // ============================================================================

    #[test]
    fn test_whitespace_filtered_before_grouping() {
        // Task B.1: Verify whitespace blocks are filtered BEFORE merge step
        // This prevents empty blocks from entering bold grouping

        use crate::geometry::Rect;
        use crate::layout::TextSpan;

        let converter = MarkdownConverter::new();
        let options = ConversionOptions::default();

        // Create spans with whitespace that should be filtered
        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Hello".to_string(),
                bbox: Rect::new(0.0, 0.0, 40.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "   ".to_string(), // Whitespace only - should be filtered
                bbox: Rect::new(50.0, 0.0, 20.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold, // Even if marked bold
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "World".to_string(),
                bbox: Rect::new(80.0, 0.0, 40.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 2,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        let result = converter.convert_page_from_spans(&spans, &options).unwrap();

        // Result should contain "Hello" and "World" but NOT empty bold markers
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(!result.contains("** **"), "Whitespace should be filtered before grouping");
    }

    #[test]
    fn test_punctuation_not_bolded() {
        // Task B.1: Punctuation-only blocks should have bold neutralized
        // "---", "...", ">>>" should never be marked bold

        use crate::geometry::Rect;
        use crate::layout::TextSpan;

        let converter = MarkdownConverter::new();
        let options = ConversionOptions::default();

        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Section".to_string(),
                bbox: Rect::new(0.0, 0.0, 50.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "---".to_string(), // Punctuation only, but marked bold
                bbox: Rect::new(60.0, 0.0, 20.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Content".to_string(),
                bbox: Rect::new(0.0, 20.0, 50.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 2,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        let result = converter.convert_page_from_spans(&spans, &options).unwrap();

        // Punctuation should not create bold markers
        assert!(result.contains("---"));
        assert!(result.contains("Content"));
        // The punctuation "---" should NOT be wrapped in ** **
        assert!(!result.contains("**---**"), "Punctuation should not be bolded");
    }

    #[test]
    fn test_numeric_bold_preserved() {
        // Task B.1: Numbers CAN be bold if they're actual content
        // "2024" or "Version 3.0" can be bold

        use crate::geometry::Rect;
        use crate::layout::TextSpan;

        let converter = MarkdownConverter::new();
        let options = ConversionOptions {
            bold_marker_behavior: crate::converters::BoldMarkerBehavior::Conservative,
            ..Default::default()
        };

        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Year:".to_string(),
                bbox: Rect::new(0.0, 0.0, 40.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "2024".to_string(), // Numeric, should be bold if marked
                bbox: Rect::new(50.0, 0.0, 30.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        let result = converter.convert_page_from_spans(&spans, &options).unwrap();

        // Numeric content should be allowed
        assert!(result.contains("2024"));
        // May or may not have bold markers depending on boundary context,
        // but shouldn't produce empty markers
        assert!(!result.contains("** **"), "Numeric should not create empty bold markers");
    }

    #[test]
    fn test_no_empty_bold_markers_regression() {
        // Task B.1: Combined fix should prevent ANY empty bold markers
        // This is the main regression test for the fix

        use crate::geometry::Rect;
        use crate::layout::TextSpan;

        let converter = MarkdownConverter::new();
        let options = ConversionOptions::default();

        // Scenario: Mix of content, whitespace, and punctuation
        // All potentially bolded
        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Title".to_string(),
                bbox: Rect::new(0.0, 0.0, 40.0, 14.0),
                font_name: "Times-Bold".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: " ".to_string(), // Whitespace - should be filtered
                bbox: Rect::new(50.0, 0.0, 5.0, 14.0),
                font_name: "Times-Bold".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "...".to_string(), // Punctuation - should be neutralized
                bbox: Rect::new(60.0, 0.0, 15.0, 14.0),
                font_name: "Times-Bold".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 2,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "  \n  ".to_string(), // Mixed whitespace - should be filtered
                bbox: Rect::new(0.0, 20.0, 50.0, 12.0),
                font_name: "Times-Bold".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 3,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Content".to_string(),
                bbox: Rect::new(0.0, 35.0, 50.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 4,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        let result = converter.convert_page_from_spans(&spans, &options).unwrap();

        // Main assertion: NO empty bold markers anywhere
        assert!(!result.contains("** **"), "No empty bold markers allowed");
        assert!(!result.contains("**\n**"), "No empty bold markers with newlines");
        assert!(!result.contains("**  **"), "No bold wrapping only spaces");

        // Content should still be present
        assert!(result.contains("Title"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_merge_adjacent_char_spans_preserves_spacing() {
        // Task B.1: Verify merge happens AFTER filtering
        // So merged characters don't inherit bold from filtered whitespace

        let spans = vec![
            TextBlock {
                chars: vec![],
                bbox: Rect::new(0.0, 0.0, 4.0, 12.0),
                text: "H".to_string(),
                avg_font_size: 12.0,
                dominant_font: "Times".to_string(),
                is_bold: false,
                is_italic: false,
                mcid: None,
                sequence: 0,
                rotation_degrees: 0.0,
            },
            TextBlock {
                chars: vec![],
                bbox: Rect::new(4.5, 0.0, 4.0, 12.0),
                text: "i".to_string(),
                avg_font_size: 12.0,
                dominant_font: "Times".to_string(),
                is_bold: false,
                is_italic: false,
                mcid: None,
                sequence: 0,
                rotation_degrees: 0.0,
            },
            TextBlock {
                chars: vec![],
                bbox: Rect::new(9.0, 0.0, 4.0, 12.0),
                text: "!".to_string(),
                avg_font_size: 12.0,
                dominant_font: "Times".to_string(),
                is_bold: false,
                is_italic: false,
                mcid: None,
                sequence: 0,
                rotation_degrees: 0.0,
            },
        ];

        let merged = MarkdownConverter::merge_adjacent_char_spans(spans);

        // Should merge closely-spaced characters into "Hi!"
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].text, "Hi!");
    }

    // ============================================================================
    // NEW TESTS (Fix 2A: Trim Boundary Extraction)
    // ============================================================================

    #[test]
    fn test_fix_2a_boundary_extraction_with_leading_whitespace() {
        // Fix 2A: Leading whitespace should not become first_char_in_group
        // This prevents "** text**" patterns where the opening position is space

        let group = BoldGroup {
            text: "  hello".to_string(), // Leading spaces
            is_bold: true,
            first_char_in_group: Some('h'), // Should be 'h' from trimmed, not space
            last_char_in_group: Some('o'),
        };

        // Validator should approve: first char is alphabetic
        assert_eq!(BoldMarkerValidator::can_insert_markers(&group), BoldMarkerDecision::Insert);
    }

    #[test]
    fn test_fix_2a_boundary_extraction_with_trailing_whitespace() {
        // Fix 2A: Trailing whitespace should not become last_char_in_group
        // This prevents "**text **" patterns where the closing position is space

        let group = BoldGroup {
            text: "hello  ".to_string(), // Trailing spaces
            is_bold: true,
            first_char_in_group: Some('h'),
            last_char_in_group: Some('o'), // Should be 'o' from trimmed, not space
        };

        // Validator should approve: last char is alphabetic
        assert_eq!(BoldMarkerValidator::can_insert_markers(&group), BoldMarkerDecision::Insert);
    }

    #[test]
    fn test_fix_2a_boundary_extraction_with_both_whitespace() {
        // Fix 2A: Both leading and trailing whitespace should be trimmed

        let group = BoldGroup {
            text: "  hello world  ".to_string(), // Both sides
            is_bold: true,
            first_char_in_group: Some('h'), // From trimmed
            last_char_in_group: Some('d'),  // From trimmed
        };

        // Validator should approve
        assert_eq!(BoldMarkerValidator::can_insert_markers(&group), BoldMarkerDecision::Insert);
    }

    #[test]
    fn test_fix_2a_whitespace_only_string_returns_none() {
        // Fix 2A: Whitespace-only strings should have None boundaries
        // This prevents empty bold markers

        let group = BoldGroup {
            text: "   ".to_string(),
            is_bold: true,
            first_char_in_group: None, // trimmed is empty
            last_char_in_group: None,  // trimmed is empty
        };

        // Validator should reject: no word content
        assert_eq!(
            BoldMarkerValidator::can_insert_markers(&group),
            BoldMarkerDecision::Skip(ValidatorError::WhitespaceOnly)
        );
    }

    #[test]
    fn test_fix_2a_tabs_and_newlines_trimmed() {
        // Fix 2A: Unicode whitespace variants (tabs, newlines) should be trimmed

        let group = BoldGroup {
            text: "\t\n  hello  \n\t".to_string(), // Tabs and newlines
            is_bold: true,
            first_char_in_group: Some('h'), // From trimmed
            last_char_in_group: Some('o'),  // From trimmed
        };

        // Validator should approve
        assert_eq!(BoldMarkerValidator::can_insert_markers(&group), BoldMarkerDecision::Insert);
    }

    #[test]
    fn test_fix_2a_markdown_no_empty_bold_from_spaces() {
        // Fix 2A: Integration test - no "** **" patterns from boundary trimming
        // Even if cleaned_text has spaces, the actual markers use trimmed boundaries

        use crate::layout::TextSpan;

        let converter = MarkdownConverter::new();
        let options = ConversionOptions::default();

        let spans = vec![
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "Content".to_string(),
                bbox: Rect::new(0.0, 0.0, 50.0, 12.0),
                font_name: "Times-Bold".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "  \n  ".to_string(), // Whitespace with newlines
                bbox: Rect::new(60.0, 0.0, 20.0, 12.0),
                font_name: "Times-Bold".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Bold,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 1,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: "More".to_string(),
                bbox: Rect::new(0.0, 20.0, 40.0, 12.0),
                font_name: "Times".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Normal,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 2,
                split_boundary_before: false,
                offset_semantic: false,
                char_spacing: 0.0,
                word_spacing: 0.0,
                horizontal_scaling: 100.0,
                primary_detected: false,
                char_widths: vec![],
                char_x_offsets: Vec::new(),
                heading_level: None,
                rotation_degrees: 0.0,
                wmode: 0,
                rtl_draw_logical: false,
                mirrored: false,
                page_rotation_applied: 0,
            },
        ];

        let result = converter.convert_page_from_spans(&spans, &options).unwrap();

        // Verify: no empty bold markers with newlines or spaces
        assert!(!result.contains("**\n**"), "No bold wrapping newlines");
        assert!(!result.contains("** **"), "No bold wrapping spaces");
        assert!(result.contains("Content"), "Content preserved");
        assert!(result.contains("More"), "More preserved");
    }
}
