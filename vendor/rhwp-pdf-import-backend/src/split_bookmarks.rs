//! Split a PDF into multiple PDFs at outline (bookmark) boundaries
//! (issue #482).
//!
//! This module is pure orchestration: it turns a parsed outline +
//! page count into a deterministic set of half-open page-range
//! segments with collision-free, filesystem-safe file stems. It does
//! **no** low-level PDF writing — byte production delegates to the
//! existing editor page-range extraction (wired in a later increment).
//!
//! The three pure stages — `flatten_outline`, `collect_split_points`,
//! `build_segments` — are independently unit-tested without any PDF
//! fixture (SRP / testability).

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::filename::{slugify_title_with, DEFAULT_MAX_SLUG_BYTES};
use crate::outline::{Destination, OutlineItem};

/// Which outline depth(s) become split points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BookmarkLevel {
    /// Only top-level (depth 1) items. Default (qpdf "level-0").
    #[default]
    TopLevel,
    /// Items at depth `<= n` (1-based; `UpTo(1)` == `TopLevel`).
    UpTo(u32),
    /// Every item at any depth.
    All,
}

impl BookmarkLevel {
    /// Map the CLI/binding `u32` (`0` = all, `1` = top-level, `n` =
    /// up-to-depth-n) to a [`BookmarkLevel`].
    pub fn from_u32(n: u32) -> Self {
        match n {
            0 => BookmarkLevel::All,
            1 => BookmarkLevel::TopLevel,
            n => BookmarkLevel::UpTo(n),
        }
    }

    /// Whether a (1-based) outline `depth` is selected by this level.
    fn includes(self, depth: u32) -> bool {
        match self {
            BookmarkLevel::TopLevel => depth == 1,
            BookmarkLevel::UpTo(n) => depth <= n.max(1),
            BookmarkLevel::All => true,
        }
    }
}

/// Options controlling a bookmark split. Construct with [`Default`]
/// (front-matter on, 80-byte slug budget).
#[derive(Debug, Clone)]
pub struct SplitByBookmarksOptions {
    /// Only split at bookmarks whose trimmed title starts with this
    /// prefix. `None` => every selected-level bookmark.
    pub title_prefix: Option<String>,
    /// Case-insensitive prefix match.
    pub ignore_case: bool,
    /// Outline depth selection.
    pub level: BookmarkLevel,
    /// Emit the pages before the first split point as a leading
    /// `front-matter` segment (only when non-empty).
    pub include_front_matter: bool,
    /// Use page labels (§12.4.2) for filenames instead of ordinals.
    /// (Label population is a later increment; the flag is plumbed.)
    pub use_page_labels: bool,
    /// Max slug byte length.
    pub max_slug_bytes: usize,
}

impl Default for SplitByBookmarksOptions {
    fn default() -> Self {
        Self {
            title_prefix: None,
            ignore_case: false,
            level: BookmarkLevel::TopLevel,
            include_front_matter: true,
            use_page_labels: false,
            max_slug_bytes: DEFAULT_MAX_SLUG_BYTES,
        }
    }
}

/// One planned output segment (also a `--dry-run` row).
///
/// `Serialize`/`Deserialize` so every binding (WASM, C-ABI, Go, C#)
/// marshals it through the one JSON shape and the parity goldens
/// compare a single source of truth (DRY — foundation §5.3).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BookmarkSegment {
    /// 1-based segment ordinal across the whole document.
    pub index: usize,
    /// Inclusive start page (0-based).
    pub start_page: usize,
    /// Exclusive end page (0-based); range is `start_page..end_page`.
    pub end_page: usize,
    /// Source bookmark title (`None` for the front-matter segment).
    pub title: Option<String>,
    /// Final de-duplicated, sanitised file stem (no extension).
    pub file_stem: String,
    /// Page label of `start_page`, if available and requested.
    pub page_label: Option<String>,
}

/// DFS pre-order flatten of the outline, applying the depth filter.
/// Returns `(title, Some(page_index))` for resolved destinations and
/// `(title, None)` for unresolvable ones (the caller drops those).
/// Children are always traversed; only items at a selected depth are
/// emitted.
pub fn flatten_outline(
    items: &[OutlineItem],
    level: BookmarkLevel,
) -> Vec<(String, Option<usize>)> {
    fn walk(
        items: &[OutlineItem],
        depth: u32,
        level: BookmarkLevel,
        out: &mut Vec<(String, Option<usize>)>,
    ) {
        for it in items {
            if level.includes(depth) {
                let page = match &it.dest {
                    Some(Destination::PageIndex(i)) => Some(*i),
                    _ => None,
                };
                out.push((it.title.clone(), page));
            }
            if !it.children.is_empty() {
                walk(&it.children, depth + 1, level, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(items, 1, level, &mut out);
    out
}

/// Apply the prefix filter, drop unresolvable destinations, sort by
/// page ascending (stable — document order breaks ties), and dedupe
/// by page (first title in document order wins).
pub fn collect_split_points(
    flat: Vec<(String, Option<usize>)>,
    opts: &SplitByBookmarksOptions,
) -> Vec<(usize, String)> {
    let matches_prefix = |title: &str| -> bool {
        match &opts.title_prefix {
            None => true,
            Some(p) => {
                let t = title.trim();
                if opts.ignore_case {
                    t.to_lowercase().starts_with(&p.to_lowercase())
                } else {
                    t.starts_with(p.as_str())
                }
            },
        }
    };

    let mut points: Vec<(usize, String)> = flat
        .into_iter()
        .filter(|(t, _)| matches_prefix(t))
        .filter_map(|(t, p)| p.map(|page| (page, t)))
        .collect();

    // Stable sort by page keeps document order for equal pages, so
    // the first-seen title wins the dedupe below.
    points.sort_by_key(|(page, _)| *page);
    let mut seen = std::collections::HashSet::new();
    points.retain(|(page, _)| seen.insert(*page));
    points
}

/// Form half-open page-range segments from sorted-unique split points
/// over a `page_count`-page document, adding an optional leading
/// front-matter segment and collision-free sanitised file stems.
///
/// # Errors
/// [`Error::InvalidOperation`] if no usable split point remains (all
/// filtered out, unresolvable, or past EOF) — the caller turns this
/// into the user-facing "matched nothing" / "no outline" message.
pub fn build_segments(
    points: &[(usize, String)],
    page_count: usize,
    opts: &SplitByBookmarksOptions,
) -> Result<Vec<BookmarkSegment>> {
    // Drop points at/after EOF (invalid/dangling destinations).
    let pts: Vec<&(usize, String)> = points.iter().filter(|(p, _)| *p < page_count).collect();
    if pts.is_empty() {
        return Err(Error::InvalidOperation(
            "no resolvable bookmark split points (empty outline, prefix matched \
             nothing, or all destinations unresolvable/past EOF)"
                .to_string(),
        ));
    }

    let mut segs: Vec<BookmarkSegment> = Vec::new();

    // Optional leading front-matter [0, p0).
    if opts.include_front_matter && pts[0].0 > 0 {
        segs.push(BookmarkSegment {
            index: 0, // re-numbered below
            start_page: 0,
            end_page: pts[0].0,
            title: None,
            file_stem: "front-matter".to_string(),
            page_label: None,
        });
    }

    for (i, (start, title)) in pts.iter().map(|p| (p.0, &p.1)).enumerate() {
        let end = pts.get(i + 1).map(|p| p.0).unwrap_or(page_count);
        segs.push(BookmarkSegment {
            index: 0,
            start_page: start,
            end_page: end,
            title: Some(title.clone()),
            file_stem: slugify_title_with(title, opts.max_slug_bytes),
            page_label: None,
        });
    }

    // 1-based ordinal + collision-free stems (` (2)`, ` (3)` …, the
    // CoolUtils-style human-readable form; suffix applied *after*
    // slugify so the base is already filesystem-safe).
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (idx, seg) in segs.iter_mut().enumerate() {
        seg.index = idx + 1;
        let n = counts.entry(seg.file_stem.clone()).or_insert(0);
        *n += 1;
        if *n > 1 {
            seg.file_stem = format!("{} ({})", seg.file_stem, *n);
        }
    }

    Ok(segs)
}

/// Plan the split for `doc` without producing any bytes — cheap;
/// ideal for `--dry-run` and for binding callers that drive their own
/// writes.
///
/// # Errors
/// [`Error::InvalidOperation`] if the document has no outline, or no
/// selected bookmark resolves to a usable split point.
pub fn plan_split_by_bookmarks(
    doc: &crate::document::PdfDocument,
    opts: &SplitByBookmarksOptions,
) -> Result<Vec<BookmarkSegment>> {
    let outline = doc.get_outline()?.ok_or_else(|| {
        Error::InvalidOperation(
            "document has no bookmarks/outline; use a plain per-page split instead".to_string(),
        )
    })?;
    let flat = flatten_outline(&outline, opts.level);
    let points = collect_split_points(flat, opts);
    let page_count = doc.page_count()?;
    build_segments(&points, page_count, opts)
}

/// Execute the split: returns each segment paired with its PDF bytes
/// (parallel to [`plan_split_by_bookmarks`]). The source is not
/// modified. Segment ranges are `[start_page, end_page)` and are fed
/// directly to the editor's half-open page-range extraction.
///
/// # Errors
/// Propagates planning errors and any extraction failure.
pub fn split_by_bookmarks_to_bytes(
    src_bytes: &[u8],
    opts: &SplitByBookmarksOptions,
) -> Result<Vec<(BookmarkSegment, Vec<u8>)>> {
    let doc = crate::document::PdfDocument::from_bytes(src_bytes.to_vec())?;
    let segments = plan_split_by_bookmarks(&doc, opts)?;

    let mut editor = crate::editor::DocumentEditor::from_bytes(src_bytes.to_vec())?;
    let ranges: Vec<(usize, usize)> = segments
        .iter()
        .map(|s| (s.start_page, s.end_page))
        .collect();
    let blobs = editor.extract_page_ranges_to_bytes(&ranges)?;

    Ok(segments.into_iter().zip(blobs).collect())
}

/// Split `src_path` and write each segment to `dir` as `{file_stem}.pdf`,
/// returning the written paths in document order. Creates `dir` if
/// missing. The CLI uses this.
///
/// NOTE (deviation from #482 plan §4): the planned `password` parameter
/// is omitted in v0.3.50. This helper `std::fs::read`s `src_path` and
/// operates on **raw bytes** — it does NOT decrypt, so encrypted inputs
/// are unsupported here. The CLI does not silently ignore a password:
/// `pdf-oxide split --by-bookmarks` rejects `--password` fail-closed
/// with an actionable message. A caller needing to split an encrypted
/// document must decrypt it first and pass the decrypted bytes/file.
/// Keeping this a pure, testable byte→files step is intentional;
/// encrypted-output handling is out of scope per plan §9.
///
/// # Errors
/// Propagates planning errors and any filesystem error
/// ([`Error::Io`]).
pub fn split_by_bookmarks_to_dir(
    src_path: &std::path::Path,
    dir: &std::path::Path,
    opts: &SplitByBookmarksOptions,
) -> Result<Vec<std::path::PathBuf>> {
    let bytes = std::fs::read(src_path)?;
    let parts = split_by_bookmarks_to_bytes(&bytes, opts)?;
    std::fs::create_dir_all(dir)?;
    let mut written = Vec::with_capacity(parts.len());
    for (seg, blob) in parts {
        let path = dir.join(format!("{}.pdf", seg.file_stem));
        std::fs::write(&path, blob)?;
        written.push(path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(title: &str, page: Option<usize>, children: Vec<OutlineItem>) -> OutlineItem {
        OutlineItem {
            title: title.to_string(),
            dest: page.map(Destination::PageIndex),
            children,
        }
    }

    #[test]
    fn flatten_depth_filter() {
        let tree = vec![
            item("Ch1", Some(0), vec![item("S1.1", Some(1), vec![])]),
            item(
                "Ch2",
                Some(3),
                vec![item("S2.1", Some(4), vec![item("S2.1.1", Some(5), vec![])])],
            ),
        ];
        let top: Vec<_> = flatten_outline(&tree, BookmarkLevel::TopLevel)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(top, vec!["Ch1", "Ch2"]);

        let upto2: Vec<_> = flatten_outline(&tree, BookmarkLevel::UpTo(2))
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(upto2, vec!["Ch1", "S1.1", "Ch2", "S2.1"]);

        let all: Vec<_> = flatten_outline(&tree, BookmarkLevel::All)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(all, vec!["Ch1", "S1.1", "Ch2", "S2.1", "S2.1.1"]);
    }

    #[test]
    fn from_u32_mapping() {
        assert_eq!(BookmarkLevel::from_u32(0), BookmarkLevel::All);
        assert_eq!(BookmarkLevel::from_u32(1), BookmarkLevel::TopLevel);
        assert_eq!(BookmarkLevel::from_u32(3), BookmarkLevel::UpTo(3));
    }

    #[test]
    fn collect_prefix_filter_case_and_sort_dedupe() {
        let flat = vec![
            ("Chapter 2".to_string(), Some(5)),
            ("chapter 1".to_string(), Some(2)),
            ("Appendix".to_string(), Some(8)),
            ("Chapter 1 (dup page)".to_string(), Some(2)), // same page as ch1
            ("Bad".to_string(), None),                     // unresolvable -> dropped
        ];
        let opts = SplitByBookmarksOptions {
            title_prefix: Some("Chapter".to_string()),
            ..Default::default()
        };
        // Case-sensitive: lowercase "chapter 1" excluded; both
        // "Chapter …" entries kept (incl. the page-2 dup), sorted by
        // page ascending.
        let cs = collect_split_points(flat.clone(), &opts);
        assert_eq!(
            cs,
            vec![
                (2, "Chapter 1 (dup page)".to_string()),
                (5, "Chapter 2".to_string())
            ]
        );

        // Case-insensitive: both chapters; sorted by page; page 2
        // de-duped to the first document-order title.
        let ci = collect_split_points(
            flat,
            &SplitByBookmarksOptions {
                ignore_case: true,
                ..opts.clone()
            },
        );
        assert_eq!(ci, vec![(2, "chapter 1".to_string()), (5, "Chapter 2".to_string())]);
    }

    #[test]
    fn build_segments_ranges_frontmatter_and_eof() {
        let points = vec![(2, "Ch1".to_string()), (5, "Ch2".to_string())];
        let segs = build_segments(&points, 9, &SplitByBookmarksOptions::default()).unwrap();
        // front-matter [0,2), Ch1 [2,5), Ch2 [5,9)
        assert_eq!(segs.len(), 3);
        assert_eq!((segs[0].start_page, segs[0].end_page), (0, 2));
        assert_eq!(segs[0].file_stem, "front-matter");
        assert_eq!(segs[0].title, None);
        assert_eq!((segs[1].start_page, segs[1].end_page), (2, 5));
        assert_eq!(segs[1].file_stem, "Ch1");
        assert_eq!((segs[2].start_page, segs[2].end_page), (5, 9));
        assert_eq!((segs[0].index, segs[1].index, segs[2].index), (1, 2, 3));

        // No front-matter when first point is page 0.
        let s2 = build_segments(
            &[(0, "A".to_string()), (3, "B".to_string())],
            6,
            &SplitByBookmarksOptions::default(),
        )
        .unwrap();
        assert_eq!(s2.len(), 2);
        assert_eq!((s2[0].start_page, s2[0].end_page), (0, 3));

        // include_front_matter=false drops the cover.
        let s3 = build_segments(
            &points,
            9,
            &SplitByBookmarksOptions {
                include_front_matter: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(s3.len(), 2);
        assert_eq!(s3[0].file_stem, "Ch1");
    }

    #[test]
    fn build_segments_dedupes_stems_and_drops_past_eof() {
        let points = vec![
            (0, "Record".to_string()),
            (2, "Record".to_string()),
            (4, "Record".to_string()),
            (99, "Past EOF".to_string()), // page >= count -> dropped
        ];
        let segs = build_segments(&points, 6, &SplitByBookmarksOptions::default()).unwrap();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].file_stem, "Record");
        assert_eq!(segs[1].file_stem, "Record (2)");
        assert_eq!(segs[2].file_stem, "Record (3)");
        assert_eq!((segs[2].start_page, segs[2].end_page), (4, 6));
    }

    #[test]
    fn build_segments_empty_is_invalid_operation() {
        let err = build_segments(&[], 10, &SplitByBookmarksOptions::default()).unwrap_err();
        assert!(matches!(err, Error::InvalidOperation(_)));
        // All points past EOF -> same error.
        let err2 = build_segments(&[(20, "X".to_string())], 5, &SplitByBookmarksOptions::default())
            .unwrap_err();
        assert!(matches!(err2, Error::InvalidOperation(_)));
    }

    /// Cross-binding wire-shape golden. Every #482 surface (C-ABI →
    /// Go/C#/Node, WASM, Python) marshals `BookmarkSegment` through
    /// this exact JSON shape; a field rename/reorder would silently
    /// break all of them at once. This is the single source of truth
    /// the binding parity tests compare against (foundation §5.3).
    #[test]
    fn bookmark_segment_json_wire_shape_is_frozen() {
        let titled = BookmarkSegment {
            index: 2,
            start_page: 3,
            end_page: 7,
            title: Some("Chapter 1".to_string()),
            file_stem: "Chapter-1".to_string(),
            page_label: None,
        };
        assert_eq!(
            serde_json::to_string(&titled).unwrap(),
            r#"{"index":2,"start_page":3,"end_page":7,"title":"Chapter 1","file_stem":"Chapter-1","page_label":null}"#
        );
        // Front-matter segment: null title.
        let fm = BookmarkSegment {
            index: 1,
            start_page: 0,
            end_page: 3,
            title: None,
            file_stem: "front-matter".to_string(),
            page_label: Some("iii".to_string()),
        };
        assert_eq!(
            serde_json::to_string(&fm).unwrap(),
            r#"{"index":1,"start_page":0,"end_page":3,"title":null,"file_stem":"front-matter","page_label":"iii"}"#
        );
        // Round-trips (Deserialize parity for any binding that reads back).
        let back: BookmarkSegment =
            serde_json::from_str(&serde_json::to_string(&titled).unwrap()).unwrap();
        assert_eq!(back, titled);
    }
}
