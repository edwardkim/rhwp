//! Standalone document sanitization (#231 T10) — the catalog-scrub
//! decision layer (feature-231 §4.6 / §5.1 `sanitize_document`).
//!
//! Cosmetic/legacy redaction left document-level secrets untouched even
//! when the page content was removed: the XMP `/Metadata` stream, the
//! `/Info` dictionary, document JavaScript (`/OpenAction`, `/AA`,
//! `/Names/JavaScript`) and `/Names/EmbeddedFiles` all survived. This
//! module decides *what* to strip from a catalog dictionary; the engine
//! ([`crate::editor::DocumentEditor::sanitize_document`]) resolves the
//! referenced object graph and enforces G6 (no residual objects).
//!
//! It is a pure function with an injected resolver (SOLID-D — the
//! document graph is a dependency, not a hard-coded one), so it is unit
//! testable without a real PDF and reused unchanged by the engine.

#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::options::RedactionOptions;
use crate::object::Object;

/// Per-category counts of what the catalog scrub removed (feature-231
/// §5.1 — surfaced so callers can assert the scrub did real work).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SanitizeCounts {
    /// Catalog `/Metadata` (XMP) streams removed (0 or 1).
    pub metadata_streams: usize,
    /// Document JavaScript / action carriers removed
    /// (`/OpenAction`, `/AA`, `/Names/JavaScript`).
    pub javascript: usize,
    /// `/Names/EmbeddedFiles` name-tree roots removed (0 or 1).
    pub embedded_files: usize,
}

impl SanitizeCounts {
    /// Total number of top-level constructs removed.
    pub fn total(&self) -> usize {
        self.metadata_streams + self.javascript + self.embedded_files
    }
}

/// Outcome of [`sanitize_catalog`].
#[derive(Debug, Clone)]
pub struct CatalogScrub {
    /// The catalog with sensitive keys removed. `/Names` is re-emitted
    /// inline (the original `/Names` container, if it was an indirect
    /// object, becomes an orphan root so its bytes cannot survive).
    pub catalog: HashMap<String, Object>,
    /// Object ids the removed keys pointed at (subtree roots). The
    /// engine expands these into the full unreachable subtree and
    /// hard-excludes it from the output (G6 — a secret must not survive
    /// even as a GC-missed orphan).
    pub removed_roots: Vec<u32>,
    /// What was removed, by category.
    pub counts: SanitizeCounts,
}

/// Recursively collect every indirect-reference id reachable *within*
/// `obj` itself (one level of structure — arrays/dicts/streams), without
/// dereferencing. The engine follows these roots through the document.
fn collect_ref_ids(obj: &Object, out: &mut Vec<u32>) {
    match obj {
        Object::Reference(r) => out.push(r.id),
        Object::Array(a) => a.iter().for_each(|o| collect_ref_ids(o, out)),
        Object::Dictionary(d) => d.values().for_each(|o| collect_ref_ids(o, out)),
        Object::Stream { dict, .. } => dict.values().for_each(|o| collect_ref_ids(o, out)),
        _ => {},
    }
}

/// Resolve `obj` to an owned dictionary: inline dicts/streams are taken
/// as-is, a `Reference` is resolved via `resolve` (recording its id as a
/// removed root so the now-replaced container cannot survive).
fn as_owned_dict(
    obj: &Object,
    resolve: &impl Fn(u32) -> Option<Object>,
    roots: &mut Vec<u32>,
) -> Option<HashMap<String, Object>> {
    match obj {
        Object::Dictionary(d) => Some(d.clone()),
        Object::Stream { dict, .. } => Some(dict.clone()),
        Object::Reference(r) => {
            roots.push(r.id);
            match resolve(r.id) {
                Some(Object::Dictionary(d)) => Some(d),
                Some(Object::Stream { dict, .. }) => Some(dict),
                _ => None,
            }
        },
        _ => None,
    }
}

/// Remove document-level sensitive constructs from a catalog dictionary
/// per the enabled [`RedactionOptions`] toggles (feature-231 §4.6):
///
/// * `scrub_metadata`      → `/Metadata` (catalog XMP)
/// * `remove_javascript`   → `/OpenAction`, `/AA`, `/Names/JavaScript`
/// * `remove_embedded_files` → `/Names/EmbeddedFiles`
///
/// `/Names` is re-emitted inline with the scrubbed sub-entries removed;
/// if it ends up empty it is dropped entirely. `resolve` dereferences an
/// indirect `/Names` (or returns `None` if unavailable — then `/Names`
/// is conservatively left untouched and only the directly-removable
/// catalog keys are scrubbed).
pub fn sanitize_catalog(
    catalog: &HashMap<String, Object>,
    opts: &RedactionOptions,
    resolve: impl Fn(u32) -> Option<Object>,
) -> CatalogScrub {
    let mut out = catalog.clone();
    let mut roots: Vec<u32> = Vec::new();
    let mut counts = SanitizeCounts::default();

    if opts.scrub_metadata {
        if let Some(o) = out.remove("Metadata") {
            collect_ref_ids(&o, &mut roots);
            counts.metadata_streams += 1;
        }
    }

    if opts.remove_javascript {
        for key in ["OpenAction", "AA"] {
            if let Some(o) = out.remove(key) {
                collect_ref_ids(&o, &mut roots);
                counts.javascript += 1;
            }
        }
    }

    // `/Names` sub-entries (JavaScript, EmbeddedFiles) — re-emit the
    // container inline with the scrubbed entries gone.
    let want_js = opts.remove_javascript;
    let want_ef = opts.remove_embedded_files;
    if (want_js || want_ef) && out.contains_key("Names") {
        let names_obj = out.get("Names").cloned().expect("contains_key");
        if let Some(mut names) = as_owned_dict(&names_obj, &resolve, &mut roots) {
            if want_js {
                if let Some(o) = names.remove("JavaScript") {
                    collect_ref_ids(&o, &mut roots);
                    counts.javascript += 1;
                }
            }
            if want_ef {
                if let Some(o) = names.remove("EmbeddedFiles") {
                    collect_ref_ids(&o, &mut roots);
                    counts.embedded_files += 1;
                }
            }
            if names.is_empty() {
                out.remove("Names");
            } else {
                out.insert("Names".to_string(), Object::Dictionary(names));
            }
        }
    }

    CatalogScrub {
        catalog: out,
        removed_roots: roots,
        counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectRef;

    fn r(id: u32) -> Object {
        Object::Reference(ObjectRef::new(id, 0))
    }

    fn base_catalog() -> HashMap<String, Object> {
        let mut c = HashMap::new();
        c.insert("Type".to_string(), Object::Name("Catalog".to_string()));
        c.insert("Pages".to_string(), r(2));
        c.insert("Metadata".to_string(), r(10));
        c.insert("OpenAction".to_string(), r(11));
        let mut aa = HashMap::new();
        aa.insert("WC".to_string(), r(12));
        c.insert("AA".to_string(), Object::Dictionary(aa));
        c
    }

    #[test]
    fn scrubs_metadata_and_actions_and_records_roots() {
        let cat = base_catalog();
        let scrub = sanitize_catalog(&cat, &RedactionOptions::default(), |_| None);

        assert!(!scrub.catalog.contains_key("Metadata"));
        assert!(!scrub.catalog.contains_key("OpenAction"));
        assert!(!scrub.catalog.contains_key("AA"));
        // structural keys survive
        assert!(scrub.catalog.contains_key("Pages"));
        assert!(scrub.catalog.contains_key("Type"));
        // every removed reference is recorded as an orphan root
        assert!(scrub.removed_roots.contains(&10)); // /Metadata
        assert!(scrub.removed_roots.contains(&11)); // /OpenAction
        assert!(scrub.removed_roots.contains(&12)); // /AA/WC action
        assert_eq!(scrub.counts.metadata_streams, 1);
        // OpenAction + AA == 2 javascript carriers
        assert_eq!(scrub.counts.javascript, 2);
    }

    #[test]
    fn scrubs_names_javascript_and_embedded_files_via_resolver() {
        let mut cat = base_catalog();
        cat.insert("Names".to_string(), r(20));

        let mut names = HashMap::new();
        names.insert("JavaScript".to_string(), r(21));
        names.insert("EmbeddedFiles".to_string(), r(22));
        names.insert("Dests".to_string(), r(23)); // unrelated — must survive
        let names_clone = names.clone();

        let scrub = sanitize_catalog(&cat, &RedactionOptions::default(), move |id| {
            if id == 20 {
                Some(Object::Dictionary(names_clone.clone()))
            } else {
                None
            }
        });

        // /Names re-emitted inline, JS+EF gone, Dests preserved
        let n = match scrub.catalog.get("Names") {
            Some(Object::Dictionary(d)) => d,
            other => panic!("Names should be an inline dict, got {other:?}"),
        };
        assert!(!n.contains_key("JavaScript"));
        assert!(!n.contains_key("EmbeddedFiles"));
        assert!(n.contains_key("Dests"));
        // the old indirect /Names container + the removed subtrees are roots
        assert!(scrub.removed_roots.contains(&20));
        assert!(scrub.removed_roots.contains(&21));
        assert!(scrub.removed_roots.contains(&22));
        assert!(!scrub.removed_roots.contains(&23));
        assert_eq!(scrub.counts.embedded_files, 1);
        // OpenAction + AA + Names/JavaScript == 3
        assert_eq!(scrub.counts.javascript, 3);
    }

    #[test]
    fn empty_names_after_scrub_is_dropped() {
        let mut cat = base_catalog();
        let mut names = HashMap::new();
        names.insert("JavaScript".to_string(), r(21));
        names.insert("EmbeddedFiles".to_string(), r(22));
        cat.insert("Names".to_string(), Object::Dictionary(names));

        let scrub = sanitize_catalog(&cat, &RedactionOptions::default(), |_| None);
        assert!(!scrub.catalog.contains_key("Names"));
    }

    #[test]
    fn toggles_are_respected() {
        let mut cat = base_catalog();
        let mut names = HashMap::new();
        names.insert("JavaScript".to_string(), r(21));
        names.insert("EmbeddedFiles".to_string(), r(22));
        cat.insert("Names".to_string(), Object::Dictionary(names));

        let opts = RedactionOptions {
            scrub_metadata: false,
            remove_javascript: false,
            remove_embedded_files: true,
            ..RedactionOptions::default()
        };
        let scrub = sanitize_catalog(&cat, &opts, |_| None);

        // metadata + JS kept; only embedded files removed
        assert!(scrub.catalog.contains_key("Metadata"));
        assert!(scrub.catalog.contains_key("OpenAction"));
        let n = match scrub.catalog.get("Names") {
            Some(Object::Dictionary(d)) => d,
            other => panic!("Names dict expected, got {other:?}"),
        };
        assert!(n.contains_key("JavaScript"));
        assert!(!n.contains_key("EmbeddedFiles"));
        assert_eq!(
            scrub.counts,
            SanitizeCounts {
                metadata_streams: 0,
                javascript: 0,
                embedded_files: 1,
            }
        );
        assert_eq!(scrub.counts.total(), 1);
    }

    #[test]
    fn nothing_removed_when_all_toggles_off() {
        let cat = base_catalog();
        let opts = RedactionOptions {
            scrub_metadata: false,
            remove_javascript: false,
            remove_embedded_files: false,
            ..RedactionOptions::default()
        };
        let scrub = sanitize_catalog(&cat, &opts, |_| None);
        assert_eq!(scrub.catalog.len(), cat.len());
        assert!(scrub.removed_roots.is_empty());
        assert_eq!(scrub.counts.total(), 0);
    }
}
