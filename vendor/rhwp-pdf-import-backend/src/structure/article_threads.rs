//! Article threads (`/Threads`) — ISO 32000-1:2008 §12.4.3.
//!
//! Article threads are an author-supplied explicit reading order that chains
//! logically-connected content ("beads") across columns and pages. They are
//! the canonical reading-order signal for untagged legacy magazine / multi-
//! column PDFs, predating the structure tree.
//!
//! Data model (Tables 160 / 161):
//! * Catalog `/Threads` → array of indirect refs to **thread dictionaries**.
//! * Thread dict: `/F` (required) → first **bead**; `/I` (optional) thread info.
//! * Bead dict: `/N` next bead, `/V` prev bead, `/P` page object, `/R` rect
//!   `[llx lly urx ury]`. Beads form a **circular doubly-linked list** (the last
//!   bead's `/N` points back to the first).
//!
//! This module only *parses* threads into page-local bead rectangles; the
//! reading-order integration lives in
//! [`crate::pipeline::reading_order::ArticleThreadStrategy`].

use std::collections::HashMap;

use crate::document::PdfDocument;
use crate::geometry::Rect;
use crate::object::{Object, ObjectRef};

/// One bead: a rectangular region on a specific page, in PDF user space.
#[derive(Debug, Clone, PartialEq)]
pub struct Bead {
    /// 0-based index of the page this bead sits on.
    pub page_index: usize,
    /// Bead rectangle (`/R`) in the page's default user space.
    pub rect: Rect,
}

/// One article thread: an ordered chain of beads (in `/N` order).
#[derive(Debug, Clone, PartialEq)]
pub struct ArticleThread {
    /// Optional thread title (`/I /Title`).
    pub title: Option<String>,
    /// Beads in reading (`/N`) order.
    pub beads: Vec<Bead>,
}

/// Upper bound on the bead chain length — a defence against malformed,
/// non-circular `/N` chains produced by buggy generators.
const MAX_BEADS_PER_THREAD: usize = 4096;

/// Resolve `obj` to a concrete object, following a single indirect reference.
fn resolve(doc: &PdfDocument, obj: &Object) -> Option<Object> {
    match obj.as_reference() {
        Some(r) => doc.load_object(r).ok(),
        None => Some(obj.clone()),
    }
}

/// Parse a `/R` rectangle array `[llx lly urx ury]` into a [`Rect`] in user space.
fn parse_rect(arr: &[Object]) -> Option<Rect> {
    if arr.len() != 4 {
        return None;
    }
    let n = |o: &Object| -> Option<f32> {
        o.as_real()
            .map(|v| v as f32)
            .or_else(|| o.as_integer().map(|v| v as f32))
    };
    let (llx, lly, urx, ury) = (n(&arr[0])?, n(&arr[1])?, n(&arr[2])?, n(&arr[3])?);
    Some(Rect::from_points(llx, lly, urx, ury))
}

/// Parse all article threads declared in the document catalog's `/Threads`.
///
/// Best-effort and panic-free: malformed threads/beads are skipped, dangling
/// references are tolerated, and non-circular `/N` chains are bounded by
/// `MAX_BEADS_PER_THREAD`. Returns an empty vector when the document declares
/// no threads.
pub fn parse_article_threads(doc: &PdfDocument) -> Vec<ArticleThread> {
    let Ok(catalog) = doc.catalog() else {
        return Vec::new();
    };
    let Some(catalog_dict) = catalog.as_dict() else {
        return Vec::new();
    };
    let Some(threads_obj) = catalog_dict.get("Threads") else {
        return Vec::new();
    };
    let Some(threads_resolved) = resolve(doc, threads_obj) else {
        return Vec::new();
    };
    let Some(threads_arr) = threads_resolved.as_array() else {
        return Vec::new();
    };

    // Map page ObjectRef -> 0-based page index for resolving each bead's /P.
    let page_index: HashMap<ObjectRef, usize> = doc
        .all_page_refs()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, r)| (r, i))
        .collect();

    let mut threads = Vec::new();
    for thread_ref in threads_arr {
        if let Some(thread) = parse_one_thread(doc, thread_ref, &page_index) {
            if !thread.beads.is_empty() {
                threads.push(thread);
            }
        }
    }
    threads
}

fn parse_one_thread(
    doc: &PdfDocument,
    thread_obj: &Object,
    page_index: &HashMap<ObjectRef, usize>,
) -> Option<ArticleThread> {
    let thread = resolve(doc, thread_obj)?;
    let thread_dict = thread.as_dict()?;

    let title = thread_dict
        .get("I")
        .and_then(|i| resolve(doc, i))
        .and_then(|info| info.as_dict()?.get("Title").and_then(string_value));

    // First bead is required (/F). Walk /N until we loop back to it.
    let first_ref = thread_dict.get("F")?.as_reference()?;
    let mut beads = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut cur = Some(first_ref);

    while let Some(bead_ref) = cur {
        if !seen.insert(bead_ref) || beads.len() >= MAX_BEADS_PER_THREAD {
            break; // circular wrap (normal terminator) or runaway chain
        }
        let Ok(bead_obj) = doc.load_object(bead_ref) else {
            break;
        };
        let Some(bead_dict) = bead_obj.as_dict() else {
            break;
        };

        if let Some(bead) = parse_bead(bead_dict, page_index) {
            beads.push(bead);
        }

        // Advance to /N (next bead). Absent /N ends the chain.
        cur = bead_dict.get("N").and_then(|n| n.as_reference());
    }

    Some(ArticleThread { title, beads })
}

fn parse_bead(
    bead_dict: &HashMap<String, Object>,
    page_index: &HashMap<ObjectRef, usize>,
) -> Option<Bead> {
    let page_ref = bead_dict.get("P")?.as_reference()?;
    let idx = *page_index.get(&page_ref)?;
    let rect = parse_rect(bead_dict.get("R")?.as_array()?)?;
    Some(Bead {
        page_index: idx,
        rect,
    })
}

/// Decode a PDF text string object into a Rust `String` (best-effort).
fn string_value(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
        _ => None,
    }
}
