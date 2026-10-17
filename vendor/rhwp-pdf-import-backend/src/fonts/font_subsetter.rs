//! Font subsetting for PDF embedding.
//!
//! Subsets TrueType fonts to include only glyphs that are actually used,
//! significantly reducing PDF file size. Per PDF spec Section 9.9,
//! subset fonts use a tag prefix (e.g., "ABCDEF+FontName").
//!
//! # Two layers
//!
//! - [`FontSubsetter`] tracks which Unicode codepoints + glyph IDs the
//!   document actually uses, and computes a deterministic 6-letter
//!   subset tag from that set.
//! - [`subset_font_bytes`] feeds the tracked glyph IDs to the `subsetter`
//!   crate (Typst's pure-Rust OpenType subsetter, MIT/Apache) and
//!   returns minimal TTF bytes plus a [`subsetter::GlyphRemapper`] —
//!   the remapper renumbers the kept glyphs starting from 0, so every
//!   downstream consumer (PDF content stream, widths array, ToUnicode
//!   CMap) must translate original GIDs through the remapper before
//!   emitting. The mapping survives in `GlyphRemapper::get(old_gid)`.
//!
//! Composite glyph closure, GSUB/GPOS feature tables, and hinting data
//! are all handled by the `subsetter` crate.

use std::collections::{BTreeSet, HashMap};

pub use subsetter::GlyphRemapper;

/// Errors from the binary subsetter.
#[derive(Debug, thiserror::Error)]
pub enum SubsetError {
    /// The `subsetter` crate failed to subset the font (malformed data,
    /// unsupported font kind, CFF2, etc.).
    #[error("font subsetting failed: {0}")]
    Subsetter(String),
}

/// Subset a TrueType/OpenType font face to only the glyphs in `used_glyphs`,
/// returning the new (smaller) font bytes plus the remapper that turns
/// original glyph IDs into post-subset glyph IDs.
///
/// Glyph 0 (`.notdef`) is always retained per PDF spec.
///
/// The returned byte buffer is suitable for embedding directly as a
/// PDF FontFile2 stream (ISO 32000-1 §9.9). The remapper is required by
/// every other write-side consumer:
///
/// - The content stream emits hex glyph IDs that must be the *new* IDs.
/// - The `/W` widths array indexes the *new* IDs.
/// - The ToUnicode CMap maps *new* IDs back to source codepoints.
///
/// # Errors
///
/// Returns [`SubsetError::Subsetter`] if the font is malformed, the
/// font kind is unsupported (CFF2 in particular), or `index` points
/// past the end of a TrueType collection.
///
/// # Example
///
/// ```ignore
/// use pdf_oxide::fonts::{subset_font_bytes, GlyphRemapper};
/// use std::collections::BTreeSet;
///
/// let face_bytes = std::fs::read("DejaVuSans.ttf")?;
/// let mut used = BTreeSet::new();
/// for gid in [36, 37, 38, 39, 40] { used.insert(gid); } // a few glyphs
/// let (subset_bytes, remapper) = subset_font_bytes(&face_bytes, 0, &used)?;
/// assert!(subset_bytes.len() < face_bytes.len());
/// // After subsetting, original glyph 36 lives at remapper.get(36).unwrap()
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn subset_font_bytes(
    face_bytes: &[u8],
    index: u32,
    used_glyphs: &BTreeSet<u16>,
) -> Result<(Vec<u8>, GlyphRemapper), SubsetError> {
    // Always include .notdef (glyph 0) per PDF spec; otherwise PDF readers
    // that need to render an unmapped character will choke on a font with
    // no fallback glyph.
    let mut all = Vec::with_capacity(used_glyphs.len() + 1);
    all.push(0u16);
    for &gid in used_glyphs {
        if gid != 0 {
            all.push(gid);
        }
    }

    let remapper = GlyphRemapper::new_from_glyphs(&all);
    let subset = subsetter::subset(face_bytes, index, &remapper)
        .map_err(|e| SubsetError::Subsetter(e.to_string()))?;

    Ok((subset, remapper))
}

/// Font subsetter for tracking used glyphs and generating subset metadata.
///
/// This tracks which Unicode characters and glyphs are used in a document,
/// enabling efficient ToUnicode CMap generation and potential future subsetting.
#[derive(Debug, Default)]
pub struct FontSubsetter {
    /// Used Unicode codepoints mapped to their glyph IDs
    used_chars: HashMap<u32, u16>,
    /// Set of used glyph IDs (for width array generation)
    used_glyphs: BTreeSet<u16>,
    /// Subset tag (6 uppercase letters, e.g., "ABCDEF")
    subset_tag: Option<String>,
}

impl FontSubsetter {
    /// Create a new font subsetter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a character as used.
    ///
    /// # Arguments
    /// * `codepoint` - Unicode codepoint
    /// * `glyph_id` - Corresponding glyph ID from the font
    pub fn use_char(&mut self, codepoint: u32, glyph_id: u16) {
        self.used_chars.insert(codepoint, glyph_id);
        self.used_glyphs.insert(glyph_id);
    }

    /// Record a glyph ID as used without an associated codepoint.
    ///
    /// Used by the shaping path: harfrust returns glyph IDs (after
    /// ligature substitution and contextual reordering) that have no
    /// 1:1 source codepoint. The cluster information is preserved
    /// elsewhere for ToUnicode mapping.
    pub fn use_glyph(&mut self, glyph_id: u16) {
        self.used_glyphs.insert(glyph_id);
    }

    /// Record multiple characters as used.
    pub fn use_string(&mut self, text: &str, glyph_lookup: impl Fn(u32) -> Option<u16>) {
        for ch in text.chars() {
            let codepoint = ch as u32;
            if let Some(glyph_id) = glyph_lookup(codepoint) {
                self.use_char(codepoint, glyph_id);
            }
        }
    }

    /// Get the set of used glyph IDs.
    pub fn used_glyphs(&self) -> &BTreeSet<u16> {
        &self.used_glyphs
    }

    /// Get the used character to glyph mapping.
    pub fn used_chars(&self) -> &HashMap<u32, u16> {
        &self.used_chars
    }

    /// Get the number of used glyphs.
    pub fn glyph_count(&self) -> usize {
        self.used_glyphs.len()
    }

    /// Get the number of used characters.
    pub fn char_count(&self) -> usize {
        self.used_chars.len()
    }

    /// Check if any characters have been used.
    pub fn is_empty(&self) -> bool {
        self.used_chars.is_empty()
    }

    /// Generate a subset tag for the font name.
    ///
    /// Per PDF spec, subset fonts should be named "ABCDEF+FontName"
    /// where ABCDEF is a unique 6-letter tag.
    pub fn generate_subset_tag(&mut self) -> &str {
        if self.subset_tag.is_none() {
            // Generate a deterministic tag based on used glyphs
            // This ensures the same subset gets the same tag
            let hash = self.compute_subset_hash();
            let tag = Self::hash_to_tag(hash);
            self.subset_tag = Some(tag);
        }
        // Safety: subset_tag is set to Some on the line above
        self.subset_tag
            .as_ref()
            .expect("subset_tag set on prior line")
    }

    /// Get the subset tag if already generated.
    pub fn subset_tag(&self) -> Option<&str> {
        self.subset_tag.as_deref()
    }

    /// Compute a hash of the subset for tag generation.
    fn compute_subset_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for glyph in &self.used_glyphs {
            glyph.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Convert a hash to a 6-letter uppercase tag.
    fn hash_to_tag(hash: u64) -> String {
        let mut tag = String::with_capacity(6);
        let mut h = hash;
        for _ in 0..6 {
            let ch = (h % 26) as u8 + b'A';
            tag.push(ch as char);
            h /= 26;
        }
        tag
    }

    /// Create the subset font name.
    ///
    /// # Arguments
    /// * `base_name` - Original font name (e.g., "Arial")
    ///
    /// # Returns
    /// Subset name (e.g., "ABCDEF+Arial")
    pub fn subset_font_name(&mut self, base_name: &str) -> String {
        let tag = self.generate_subset_tag();
        format!("{}+{}", tag, base_name)
    }

    /// Clear the subsetter for reuse.
    pub fn clear(&mut self) {
        self.used_chars.clear();
        self.used_glyphs.clear();
        self.subset_tag = None;
    }

    /// Get statistics about the subset.
    pub fn stats(&self) -> SubsetStats {
        SubsetStats {
            unique_chars: self.used_chars.len(),
            unique_glyphs: self.used_glyphs.len(),
            min_glyph_id: self.used_glyphs.first().copied(),
            max_glyph_id: self.used_glyphs.last().copied(),
        }
    }
}

/// Statistics about a font subset.
#[derive(Debug, Clone)]
pub struct SubsetStats {
    /// Number of unique Unicode characters used
    pub unique_chars: usize,
    /// Number of unique glyphs used
    pub unique_glyphs: usize,
    /// Minimum glyph ID used
    pub min_glyph_id: Option<u16>,
    /// Maximum glyph ID used
    pub max_glyph_id: Option<u16>,
}

impl SubsetStats {
    /// Calculate potential file size reduction percentage.
    ///
    /// This is an estimate based on glyph count ratio.
    /// Actual reduction depends on glyph complexity.
    pub fn estimated_reduction(&self, total_glyphs: u16) -> f32 {
        if total_glyphs == 0 || self.unique_glyphs == 0 {
            return 0.0;
        }
        let used = self.unique_glyphs as f32;
        let total = total_glyphs as f32;
        (1.0 - used / total) * 100.0
    }
}

/// Builder for creating subsets with additional options.
#[derive(Debug)]
pub struct SubsetBuilder {
    subsetter: FontSubsetter,
    /// Always include certain glyphs (e.g., .notdef)
    always_include: BTreeSet<u16>,
}

impl SubsetBuilder {
    /// Create a new subset builder.
    pub fn new() -> Self {
        let mut always_include = BTreeSet::new();
        // Always include glyph 0 (.notdef) per PDF spec
        always_include.insert(0);

        Self {
            subsetter: FontSubsetter::new(),
            always_include,
        }
    }

    /// Add a glyph ID that should always be included.
    pub fn always_include_glyph(mut self, glyph_id: u16) -> Self {
        self.always_include.insert(glyph_id);
        self
    }

    /// Record a character as used.
    pub fn use_char(mut self, codepoint: u32, glyph_id: u16) -> Self {
        self.subsetter.use_char(codepoint, glyph_id);
        self
    }

    /// Record a string as used.
    pub fn use_string(mut self, text: &str, glyph_lookup: impl Fn(u32) -> Option<u16>) -> Self {
        self.subsetter.use_string(text, glyph_lookup);
        self
    }

    /// Build the final subsetter with always-included glyphs.
    pub fn build(mut self) -> FontSubsetter {
        for glyph in self.always_include {
            self.subsetter.used_glyphs.insert(glyph);
        }
        self.subsetter
    }
}

impl Default for SubsetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsetter_creation() {
        let subsetter = FontSubsetter::new();
        assert!(subsetter.is_empty());
        assert_eq!(subsetter.glyph_count(), 0);
    }

    #[test]
    fn test_use_char() {
        let mut subsetter = FontSubsetter::new();
        subsetter.use_char(0x0041, 1); // 'A' -> GID 1
        subsetter.use_char(0x0042, 2); // 'B' -> GID 2

        assert!(!subsetter.is_empty());
        assert_eq!(subsetter.char_count(), 2);
        assert_eq!(subsetter.glyph_count(), 2);
        assert!(subsetter.used_glyphs().contains(&1));
        assert!(subsetter.used_glyphs().contains(&2));
    }

    #[test]
    fn test_use_string() {
        let mut subsetter = FontSubsetter::new();
        // Simple lookup: codepoint = glyph_id for testing
        subsetter.use_string("AB", |cp| Some(cp as u16));

        assert_eq!(subsetter.char_count(), 2);
        assert!(subsetter.used_chars().contains_key(&0x41));
        assert!(subsetter.used_chars().contains_key(&0x42));
    }

    #[test]
    fn test_subset_tag_generation() {
        let mut subsetter = FontSubsetter::new();
        subsetter.use_char(0x0041, 1);

        let tag = subsetter.generate_subset_tag().to_string();
        assert_eq!(tag.len(), 6);
        assert!(tag.chars().all(|c| c.is_ascii_uppercase()));

        // Same subset should generate same tag
        let tag2 = subsetter.generate_subset_tag().to_string();
        assert_eq!(tag, tag2);
    }

    #[test]
    fn test_subset_font_name() {
        let mut subsetter = FontSubsetter::new();
        subsetter.use_char(0x0041, 1);

        let name = subsetter.subset_font_name("Arial");
        assert!(name.contains('+'));
        assert!(name.ends_with("Arial"));
        assert_eq!(name.split('+').next().unwrap().len(), 6);
    }

    #[test]
    fn test_stats() {
        let mut subsetter = FontSubsetter::new();
        subsetter.use_char(0x0041, 5);
        subsetter.use_char(0x0042, 10);
        subsetter.use_char(0x0043, 15);

        let stats = subsetter.stats();
        assert_eq!(stats.unique_chars, 3);
        assert_eq!(stats.unique_glyphs, 3);
        assert_eq!(stats.min_glyph_id, Some(5));
        assert_eq!(stats.max_glyph_id, Some(15));
    }

    #[test]
    fn test_estimated_reduction() {
        let mut subsetter = FontSubsetter::new();
        for i in 0..10 {
            subsetter.use_char(0x0041 + i, i as u16 + 1);
        }

        let stats = subsetter.stats();
        // Using 10 out of 1000 glyphs = 99% reduction
        let reduction = stats.estimated_reduction(1000);
        assert!(reduction > 98.0);
        assert!(reduction < 100.0);
    }

    #[test]
    fn test_builder_always_includes_notdef() {
        let subsetter = SubsetBuilder::new().use_char(0x0041, 1).build();

        // Glyph 0 (.notdef) should be included automatically
        assert!(subsetter.used_glyphs().contains(&0));
        assert!(subsetter.used_glyphs().contains(&1));
    }

    #[test]
    fn test_clear() {
        let mut subsetter = FontSubsetter::new();
        subsetter.use_char(0x0041, 1);
        let _ = subsetter.generate_subset_tag();

        subsetter.clear();

        assert!(subsetter.is_empty());
        assert!(subsetter.subset_tag().is_none());
    }

    /// Regression test for #449: subsetting a CFF/OTF font must not change
    /// the font's SFNT magic bytes — the output must remain a valid CFF font
    /// (starts with "OTTO") so callers can detect the font type and emit the
    /// correct PDF objects (FontFile3 / CIDFontType0, not FontFile2 / CIDFontType2).
    #[test]
    fn test_subset_cff_otf_preserves_magic() {
        let font_path =
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fonts/StandardSymbolsPS.otf");
        let font_bytes = std::fs::read(font_path).expect("StandardSymbolsPS.otf fixture missing");

        // Confirm the input is a CFF/OTF font
        assert_eq!(&font_bytes[..4], b"OTTO", "fixture is not a CFF/OTF font");

        // Subset to a handful of glyphs (plus .notdef which subset_font_bytes always adds)
        let mut used = BTreeSet::new();
        used.insert(1u16);
        used.insert(2u16);
        used.insert(3u16);

        let (subset_bytes, remapper) =
            subset_font_bytes(&font_bytes, 0, &used).expect("subsetting should succeed");

        // The subset must still be a CFF/OTF font
        assert!(
            subset_bytes.starts_with(b"OTTO"),
            "subsetting corrupted CFF magic bytes: {:?}",
            &subset_bytes[..4.min(subset_bytes.len())]
        );

        // GID 0 (.notdef) is always preserved; our requested glyphs should map
        assert!(remapper.get(0).is_some(), ".notdef (GID 0) must survive");
        assert!(remapper.get(1).is_some(), "GID 1 must survive");
    }
}
