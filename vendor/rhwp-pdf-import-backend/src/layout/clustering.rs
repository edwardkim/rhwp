//! DBSCAN clustering for text layout analysis.
//!
//! This module implements DBSCAN (Density-Based Spatial Clustering of Applications
//! with Noise) for grouping characters into words and words into lines.
//!
//! Note: This module is currently feature-gated but linfa-clustering API has changed.
//! For Phase 8 MVP, we use simplified distance-based clustering instead.

use crate::layout::text_block::{TextBlock, TextChar};

/// Cluster characters into words using DBSCAN.
///
/// This uses the spatial positions of characters to group them into words.
/// Characters that are close together (within `epsilon` distance) are grouped
/// into the same word.
///
/// # Arguments
///
/// * `chars` - The characters to cluster
/// * `epsilon` - The maximum distance between characters in the same word
///
/// # Returns
///
/// A vector of clusters, where each cluster is a vector of character indices.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "ml")]
/// # {
/// use pdf_oxide::geometry::Rect;
/// use pdf_oxide::layout::{TextChar, FontWeight, Color, clustering::cluster_chars_into_words};
///
/// let chars = vec![
///     TextChar {
///         char: 'H',
///         bbox: Rect::new(0.0, 0.0, 10.0, 12.0),
///         font_name: "Times".to_string(),
///         font_size: 12.0,
///         font_weight: FontWeight::Normal,
///         color: Color::black(),
///         mcid: None,
///         origin_x: 0.0,
///         origin_y: 0.0,
///         rotation_degrees: 0.0,
///         advance_width: 10.0,
///         rendered_advance: 10.0,
///         matrix: None,
///     },
///     TextChar {
///         char: 'i',
///         bbox: Rect::new(11.0, 0.0, 5.0, 12.0),
///         font_name: "Times".to_string(),
///         font_size: 12.0,
///         font_weight: FontWeight::Normal,
///         color: Color::black(),
///         mcid: None,
///         origin_x: 11.0,
///         origin_y: 0.0,
///         rotation_degrees: 0.0,
///         advance_width: 5.0,
///         rendered_advance: 5.0,
///         matrix: None,
///     },
/// ];
///
/// let clusters = cluster_chars_into_words(&chars, 3.0);
/// // Characters within 3.0 units are grouped together
/// # }
/// ```
#[cfg(feature = "ml")]
pub fn cluster_chars_into_words(chars: &[TextChar], epsilon: f32) -> Vec<Vec<usize>> {
    if chars.is_empty() {
        return vec![];
    }

    if chars.len() == 1 {
        return vec![vec![0]];
    }

    // Optimized clustering using sort-based approach: O(n log n)
    // Sort characters by Y then X, group into lines, then cluster by X-proximity.

    let mut indices: Vec<usize> = (0..chars.len()).collect();
    indices.sort_by(|&a, &b| {
        let y_cmp =
            crate::utils::safe_float_cmp(chars[b].bbox.center().y, chars[a].bbox.center().y);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        crate::utils::safe_float_cmp(chars[a].bbox.center().x, chars[b].bbox.center().x)
    });

    // Group into lines (chars within epsilon distance vertically)
    let mut lines: Vec<Vec<usize>> = vec![];
    let mut current_line: Vec<usize> = vec![indices[0]];
    let mut line_y = chars[indices[0]].bbox.center().y;

    for &idx in &indices[1..] {
        let y = chars[idx].bbox.center().y;
        if (y - line_y).abs() <= epsilon {
            current_line.push(idx);
        } else {
            lines.push(std::mem::take(&mut current_line));
            current_line.push(idx);
            line_y = y;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Within each line, cluster by X-proximity
    let mut clusters: Vec<Vec<usize>> = vec![];

    for line in &lines {
        let mut cluster = vec![line[0]];

        for &idx in &line[1..] {
            let prev_idx = *cluster.last().unwrap();
            let prev_right = chars[prev_idx].bbox.right();
            let curr_left = chars[idx].bbox.left();
            let x_gap = (curr_left - prev_right).max(0.0);

            if x_gap <= epsilon {
                cluster.push(idx);
            } else {
                cluster.sort_by(|&a, &b| {
                    crate::utils::safe_float_cmp(chars[a].bbox.x, chars[b].bbox.x)
                });
                clusters.push(std::mem::take(&mut cluster));
                cluster.push(idx);
            }
        }

        cluster.sort_by(|&a, &b| crate::utils::safe_float_cmp(chars[a].bbox.x, chars[b].bbox.x));
        clusters.push(cluster);
    }

    clusters
}

/// Cluster words into lines using DBSCAN based on Y-coordinate.
///
/// This groups words that have similar vertical positions into lines.
///
/// # Arguments
///
/// * `words` - The word blocks to cluster
/// * `epsilon_y` - The maximum vertical distance between words in the same line
///
/// # Returns
///
/// A vector of line clusters, where each cluster is a vector of word indices.
/// Words within each line are sorted left-to-right.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "ml")]
/// # {
/// use pdf_oxide::geometry::Rect;
/// use pdf_oxide::layout::{TextChar, TextBlock, FontWeight, Color, clustering::cluster_words_into_lines};
///
/// let chars1 = vec![
///     TextChar {
///         char: 'H',
///         bbox: Rect::new(0.0, 0.0, 10.0, 12.0),
///         font_name: "Times".to_string(),
///         font_size: 12.0,
///         font_weight: FontWeight::Normal,
///         is_italic: false,
///         is_monospace: false,
///         color: Color::black(),
///         mcid: None,
///         origin_x: 0.0,
///         origin_y: 0.0,
///         rotation_degrees: 0.0,
///         advance_width: 10.0,
///         rendered_advance: 10.0,
///         matrix: None,
///     },
/// ];
/// let word1 = TextBlock::from_chars(chars1);
///
/// let chars2 = vec![
///     TextChar {
///         char: 'W',
///         bbox: Rect::new(50.0, 1.0, 10.0, 12.0),
///         font_name: "Times".to_string(),
///         font_size: 12.0,
///         font_weight: FontWeight::Normal,
///         is_italic: false,
///         is_monospace: false,
///         color: Color::black(),
///         mcid: None,
///         origin_x: 50.0,
///         origin_y: 1.0,
///         rotation_degrees: 0.0,
///         advance_width: 10.0,
///         rendered_advance: 10.0,
///         matrix: None,
///     },
/// ];
/// let word2 = TextBlock::from_chars(chars2);
///
/// let words = vec![word1, word2];
/// let lines = cluster_words_into_lines(&words, 5.0);
/// // Words within 5.0 units vertically are grouped into the same line
/// # }
/// ```
#[cfg(feature = "ml")]
pub fn cluster_words_into_lines(words: &[TextBlock], epsilon_y: f32) -> Vec<Vec<usize>> {
    if words.is_empty() {
        return vec![];
    }

    if words.len() == 1 {
        return vec![vec![0]];
    }

    // Optimized clustering using sort-based approach: O(n log n)
    // Sort words by Y, then group consecutive words within epsilon_y.

    let mut indices: Vec<usize> = (0..words.len()).collect();
    indices.sort_by(|&a, &b| {
        let y_cmp = crate::utils::safe_float_cmp(words[b].bbox.y, words[a].bbox.y);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        crate::utils::safe_float_cmp(words[a].bbox.x, words[b].bbox.x)
    });

    let mut clusters: Vec<Vec<usize>> = vec![];
    let mut current_cluster: Vec<usize> = vec![indices[0]];
    let mut cluster_y = words[indices[0]].bbox.y;

    for &idx in &indices[1..] {
        if (words[idx].bbox.y - cluster_y).abs() <= epsilon_y {
            current_cluster.push(idx);
        } else {
            current_cluster
                .sort_by(|&a, &b| crate::utils::safe_float_cmp(words[a].bbox.x, words[b].bbox.x));
            clusters.push(std::mem::take(&mut current_cluster));
            current_cluster.push(idx);
            cluster_y = words[idx].bbox.y;
        }
    }

    if !current_cluster.is_empty() {
        current_cluster
            .sort_by(|&a, &b| crate::utils::safe_float_cmp(words[a].bbox.x, words[b].bbox.x));
        clusters.push(current_cluster);
    }

    clusters
}

// Fallback implementations when ML feature is not enabled

/// Cluster characters into words using spatial DBSCAN (fallback).
///
/// This is the fallback implementation used when the `ml` feature is not enabled.
/// It uses true spatial DBSCAN that checks ALL characters within epsilon distance,
/// not just consecutive ones. This fixes word segmentation issues where characters
/// may be out of order in the input array.
#[cfg(not(feature = "ml"))]
pub fn cluster_chars_into_words(chars: &[TextChar], epsilon: f32) -> Vec<Vec<usize>> {
    if chars.is_empty() {
        return vec![];
    }

    if chars.len() == 1 {
        return vec![vec![0]];
    }

    // Optimized spatial clustering using sort-based approach: O(n log n)
    // Instead of O(n²) brute-force DBSCAN, sort characters by Y then X,
    // and scan linearly to find connected components.

    // Create indices sorted by Y (line grouping), then X (reading order)
    let mut indices: Vec<usize> = (0..chars.len()).collect();
    indices.sort_by(|&a, &b| {
        let y_cmp =
            crate::utils::safe_float_cmp(chars[b].bbox.center().y, chars[a].bbox.center().y);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        crate::utils::safe_float_cmp(chars[a].bbox.center().x, chars[b].bbox.center().x)
    });

    // Group into lines first (chars within font_size * 0.5 vertically)
    let mut lines: Vec<Vec<usize>> = vec![];
    let mut current_line: Vec<usize> = vec![indices[0]];
    let mut line_y = chars[indices[0]].bbox.center().y;

    for &idx in &indices[1..] {
        let y = chars[idx].bbox.center().y;
        let font_half = chars[idx].font_size * 0.5;
        if (y - line_y).abs() < font_half.max(chars[current_line[0]].font_size * 0.5) {
            current_line.push(idx);
        } else {
            lines.push(std::mem::take(&mut current_line));
            current_line.push(idx);
            line_y = y;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Within each line, cluster by X-proximity (gap <= epsilon)
    let mut clusters: Vec<Vec<usize>> = vec![];

    for line in &lines {
        // Line is already sorted by X from the initial sort
        let mut cluster = vec![line[0]];

        for &idx in &line[1..] {
            let prev_idx = *cluster.last().unwrap();
            let prev_right = chars[prev_idx].bbox.right();
            let curr_left = chars[idx].bbox.left();
            let x_gap = (curr_left - prev_right).max(0.0);

            if x_gap <= epsilon {
                cluster.push(idx);
            } else {
                // Sort cluster by X position (left-to-right)
                cluster.sort_by(|&a, &b| {
                    crate::utils::safe_float_cmp(chars[a].bbox.x, chars[b].bbox.x)
                });
                clusters.push(std::mem::take(&mut cluster));
                cluster.push(idx);
            }
        }

        // Don't forget the last cluster
        cluster.sort_by(|&a, &b| crate::utils::safe_float_cmp(chars[a].bbox.x, chars[b].bbox.x));
        clusters.push(cluster);
    }

    clusters
}

/// Cluster words into lines using column-aware Y-coordinate grouping (fallback).
///
/// This is a simplified implementation used when the `ml` feature is not enabled.
/// It groups words that have similar Y coordinates AND are horizontally connected,
/// avoiding mixing words from different columns.
#[cfg(not(feature = "ml"))]
pub fn cluster_words_into_lines(words: &[TextBlock], epsilon_y: f32) -> Vec<Vec<usize>> {
    if words.is_empty() {
        return vec![];
    }

    // Optimized clustering using sort-based approach: O(n log n)
    // Sort words by Y coordinate, then group consecutive words within epsilon_y.
    // Within each Y-group, split by column gaps, font-size-relative.

    // Sort indices by Y coordinate
    let mut indices: Vec<usize> = (0..words.len()).collect();
    indices.sort_by(|&a, &b| {
        let y_cmp = crate::utils::safe_float_cmp(words[b].bbox.y, words[a].bbox.y);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        crate::utils::safe_float_cmp(words[a].bbox.x, words[b].bbox.x)
    });

    // Group into Y-bands (words within epsilon_y vertically)
    let mut y_bands: Vec<Vec<usize>> = vec![];
    let mut current_band: Vec<usize> = vec![indices[0]];
    let mut band_y = words[indices[0]].bbox.y;

    for &idx in &indices[1..] {
        if (words[idx].bbox.y - band_y).abs() <= epsilon_y {
            current_band.push(idx);
        } else {
            y_bands.push(std::mem::take(&mut current_band));
            current_band.push(idx);
            band_y = words[idx].bbox.y;
        }
    }
    if !current_band.is_empty() {
        y_bands.push(current_band);
    }

    // Within each Y-band, sort by X and split by column gaps
    let mut clusters: Vec<Vec<usize>> = vec![];

    for band in &mut y_bands {
        // Sort by X within band
        band.sort_by(|&a, &b| crate::utils::safe_float_cmp(words[a].bbox.x, words[b].bbox.x));

        let mut cluster = vec![band[0]];

        for &idx in &band[1..] {
            let prev_idx = *cluster.last().unwrap();
            let x_dist = (words[idx].bbox.left() - words[prev_idx].bbox.right())
                .abs()
                .min((words[prev_idx].bbox.left() - words[idx].bbox.right()).abs());

            // Font-size-relative column-gap threshold (mirrors
            // `is_column_gap` in the markdown converter): a flat 50pt
            // split made the gap-between-cells decision depend only on
            // how wide a row's words happened to be — a header row of
            // short words ("CEP"/"Cidade"/"UF") cleared 50pt and split
            // into one line per cell, while the value row directly below
            // it ("73751-452"/"PLANALTINA"/"GO") ate into the same column
            // width and stayed under 50pt, merging into one line — even
            // though both rows share the same column gutters.
            let font_size_ref = words[idx].avg_font_size.max(words[prev_idx].avg_font_size);
            let column_gap_threshold = (font_size_ref * 3.0).max(30.0);

            if x_dist < column_gap_threshold {
                cluster.push(idx);
            } else {
                clusters.push(std::mem::take(&mut cluster));
                cluster.push(idx);
            }
        }

        clusters.push(cluster);
    }

    clusters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::{Color, FontWeight};

    fn mock_char(c: char, x: f32, y: f32) -> TextChar {
        let bbox = Rect::new(x, y, 10.0, 12.0);
        TextChar {
            char: c,
            bbox,
            font_name: "Times".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            is_italic: false,
            is_monospace: false,
            origin_x: bbox.x,
            origin_y: bbox.y,
            rotation_degrees: 0.0,
            advance_width: bbox.width,
            rendered_advance: bbox.width,
            ascent: 0.95 * 12.0,
            descent: -0.35 * 12.0,
            matrix: None,
        }
    }

    #[test]
    fn test_cluster_chars_empty() {
        let chars = vec![];
        let clusters = cluster_chars_into_words(&chars, 8.0);
        assert_eq!(clusters.len(), 0);
    }

    #[test]
    fn test_cluster_chars_single() {
        let chars = vec![mock_char('A', 0.0, 0.0)];
        let clusters = cluster_chars_into_words(&chars, 8.0);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0], vec![0]);
    }

    #[test]
    fn test_cluster_chars_into_words() {
        // "Hello World" - two words
        let chars = vec![
            mock_char('H', 0.0, 0.0),
            mock_char('e', 11.0, 0.0),
            mock_char('l', 22.0, 0.0),
            mock_char('l', 33.0, 0.0),
            mock_char('o', 44.0, 0.0),
            // Big gap
            mock_char('W', 100.0, 0.0),
            mock_char('o', 111.0, 0.0),
            mock_char('r', 122.0, 0.0),
            mock_char('l', 133.0, 0.0),
            mock_char('d', 144.0, 0.0),
        ];

        let clusters = cluster_chars_into_words(&chars, 20.0);

        // Should have 2 clusters
        assert_eq!(clusters.len(), 2);

        // First cluster: "Hello" (indices 0-4)
        assert!(clusters[0].contains(&0));
        assert!(clusters[0].contains(&1));
        assert!(clusters[0].contains(&2));
        assert!(clusters[0].contains(&3));
        assert!(clusters[0].contains(&4));

        // Second cluster: "World" (indices 5-9)
        assert!(clusters[1].contains(&5));
        assert!(clusters[1].contains(&6));
        assert!(clusters[1].contains(&7));
        assert!(clusters[1].contains(&8));
        assert!(clusters[1].contains(&9));
    }

    // PDX-2 (liteparse report): char clustering was O(n²) (BFS scanning all
    // unvisited chars per frontier member), making dense pages quadratic. The
    // sort-then-group rewrite is O(n log n). This guard clusters a large input
    // (50 words × 5 chars across 50 lines = 2500 chars); pre-fix this was
    // millions of inner-loop iterations, post-fix it returns promptly and with
    // the correct cluster count. A plain wall-clock assert would be flaky on a
    // shared CI box, so we pin correctness on a large input — a quadratic
    // regression would manifest as a hang well inside the test timeout.
    #[test]
    fn test_pdx2_clustering_scales_on_large_input() {
        let mut chars = Vec::new();
        let words = 50usize;
        let lines = 50usize;
        for line in 0..lines {
            let y = line as f32 * 20.0;
            for w in 0..words {
                // Each word: 5 glyphs at ~11pt pitch, words separated by a big gap.
                let word_x0 = w as f32 * 80.0;
                for g in 0..5 {
                    chars.push(mock_char('a', word_x0 + g as f32 * 11.0, y));
                }
            }
        }
        assert_eq!(chars.len(), words * lines * 5);

        let clusters = cluster_chars_into_words(&chars, 20.0);

        // Exactly one cluster per word per line — no cross-word or cross-line merges.
        assert_eq!(
            clusters.len(),
            words * lines,
            "PDX-2 regression: expected {} word clusters, got {}",
            words * lines,
            clusters.len()
        );
        assert!(
            clusters.iter().all(|c| c.len() == 5),
            "PDX-2 regression: every word cluster should contain its 5 glyphs"
        );
    }

    #[test]
    fn test_cluster_words_empty() {
        let words: Vec<TextBlock> = vec![];
        let clusters = cluster_words_into_lines(&words, 5.0);
        assert_eq!(clusters.len(), 0);
    }

    #[test]
    fn test_cluster_words_single() {
        let chars = vec![mock_char('A', 0.0, 0.0)];
        let word = TextBlock::from_chars(chars);
        let words = vec![word];

        let clusters = cluster_words_into_lines(&words, 5.0);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0], vec![0]);
    }

    #[test]
    fn test_cluster_words_into_lines() {
        // Two lines: "Hello World" on line 1, "Foo Bar" on line 2.
        // Gap kept under the font-relative column threshold
        // ((12pt * 3.0).max(30.0) = 36pt) so these read as ordinary
        // same-line word spacing rather than a column boundary.
        let word1 = TextBlock::from_chars(vec![mock_char('H', 0.0, 0.0)]);
        let word2 = TextBlock::from_chars(vec![mock_char('W', 20.0, 1.0)]); // Same line
        let word3 = TextBlock::from_chars(vec![mock_char('F', 0.0, 30.0)]); // Different line
        let word4 = TextBlock::from_chars(vec![mock_char('B', 20.0, 31.0)]); // Same as word3

        let words = vec![word1, word2, word3, word4];
        let lines = cluster_words_into_lines(&words, 5.0);

        // Should have 2 lines
        assert_eq!(lines.len(), 2);

        // Verify clustering - Y-descending order
        // Line 1: words 2 and 3 (y=30)
        assert!(lines[0].contains(&2));
        assert!(lines[0].contains(&3));

        // Line 2: words 0 and 1 (y=0)
        assert!(lines[1].contains(&0));
        assert!(lines[1].contains(&1));
    }

    #[test]
    fn test_words_sorted_by_x_in_line() {
        // Create words in reverse order (right to left) on same line
        // Using realistic word spacing (< 36pt font-relative column gap threshold)
        let word1 = TextBlock::from_chars(vec![mock_char('W', 40.0, 0.0)]); // "World" at x=40
        let word2 = TextBlock::from_chars(vec![mock_char('H', 0.0, 1.0)]); // "Hello" at x=0

        let words = vec![word1, word2];
        let lines = cluster_words_into_lines(&words, 5.0);

        assert_eq!(lines.len(), 1);
        // Should be sorted: index 1 (x=0) before index 0 (x=40)
        assert_eq!(lines[0], vec![1, 0]);
    }

    /// A 3-column row whose words are wide enough that the inter-column
    /// gap shrinks to 40pt (under the old flat 50pt split threshold, so it
    /// wrongly merged into one line-cluster) must still split into 3
    /// separate clusters — same as a header row above it with shorter
    /// words and a 70pt gap, which always split correctly. Mirrors the
    /// `form_row_line_split.pdf` repro: a label row splits per-cell while
    /// the value row directly below it (same columns, narrower gap) was
    /// wrongly merged into a single line.
    #[test]
    fn test_column_gap_threshold_is_font_relative_not_flat() {
        fn word_spanning(start_x: f32, width: f32, y: f32) -> TextBlock {
            let count = (width / 10.0).round() as i32;
            let chars = (0..count)
                .map(|i| mock_char('X', start_x + i as f32 * 10.0, y))
                .collect();
            TextBlock::from_chars(chars)
        }

        // Header row: narrow words, 90pt column step -> 70pt gap.
        let header = vec![
            word_spanning(0.0, 20.0, 700.0),
            word_spanning(90.0, 20.0, 700.0),
            word_spanning(180.0, 20.0, 700.0),
        ];
        let header_lines = cluster_words_into_lines(&header, 5.0);
        assert_eq!(header_lines.len(), 3, "narrow-word header row should split into 3 cells");

        // Value row: wide words, same 90pt column step -> 40pt gap (under
        // the old flat 50pt threshold but over the new font-relative one).
        let value = vec![
            word_spanning(0.0, 50.0, 600.0),
            word_spanning(90.0, 50.0, 600.0),
            word_spanning(180.0, 50.0, 600.0),
        ];
        let value_lines = cluster_words_into_lines(&value, 5.0);
        assert_eq!(
            value_lines.len(),
            3,
            "wide-word value row with the same column gutters must also split into 3 cells, \
             not merge into 1 just because its words are wider"
        );
    }
}
