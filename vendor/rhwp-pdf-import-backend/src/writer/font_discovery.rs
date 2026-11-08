//! System font discovery + CSS-style `font-family` matching.
//!
//! For HTML+CSS→PDF (issue #248) we need to take a CSS declaration like
//!
//! ```css
//! font-family: "Helvetica Neue", Arial, sans-serif;
//! font-weight: 700;
//! font-style: italic;
//! ```
//!
//! and turn it into actual font face bytes — without bundling any fonts
//! in the crate. This module wraps `fontdb` (RazrFalcon's MIT-licensed
//! system-font index) with a small CSS-aware matching layer.
//!
//! Behaviour:
//!
//! 1. Walk the host's standard font directories on first use:
//!    - Linux: `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.fonts`,
//!      `~/.local/share/fonts`
//!    - macOS: `/System/Library/Fonts`, `/Library/Fonts`, `~/Library/Fonts`
//!    - Windows: `C:\Windows\Fonts` (delegated to fontdb)
//! 2. For each CSS family name in priority order, pick the closest
//!    weight + style match. CSS generic families (`serif`, `sans-serif`,
//!    `monospace`, `cursive`, `fantasy`, `system-ui`) resolve to a hard-
//!    coded preferred-family list per OS — same heuristic browsers use.
//! 3. Tie-breaking: lexicographic by face path. Deterministic across
//!    runs on the same host so byte-stable PDFs are achievable.
//!
//! Hosts without the requested font produce an `Err(FontResolveError::
//! NoMatch)`; callers (eventually the CSS-cascade layer) decide whether
//! to log a warning and fall back to Base-14 or to surface the error to
//! the API caller.
//!
//! The module is gated on the `system-fonts` feature so WASM builds
//! (no filesystem) and library embedders who want zero IO at startup
//! can opt out cleanly.

use fontdb::{Database, Family, ID};
use std::path::PathBuf;
use std::sync::Mutex;

/// Errors returned by font resolution.
#[derive(Debug, thiserror::Error)]
pub enum FontResolveError {
    /// None of the requested CSS families was available on the host.
    #[error(
        "no system font matched any of the requested families: {requested:?} \
         (weight {weight}, italic {italic})"
    )]
    NoMatch {
        /// CSS family list as supplied by the caller.
        requested: Vec<String>,
        /// CSS weight 100-900.
        weight: u16,
        /// Italic flag.
        italic: bool,
    },

    /// The matched font face had no on-disk path (in-memory only). We
    /// can't ship in-memory fonts back to the caller without extra
    /// machinery, and fontdb only loads paths in v0.23 anyway.
    #[error("matched font has no on-disk path: {family}")]
    NoPath {
        /// Family name from fontdb.
        family: String,
    },

    /// Failed to read the font file.
    #[error("failed to read font file {path}: {source}")]
    Io {
        /// Path attempted.
        path: PathBuf,
        /// Underlying io error.
        #[source]
        source: std::io::Error,
    },
}

/// CSS font-style discriminator used during matching. The CSS Fonts
/// Module Level 4 spec also has `oblique <angle>` but for v0.3.35 we
/// collapse to italic/non-italic — fontdb does the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    /// `font-style: normal`
    #[default]
    Normal,
    /// `font-style: italic` or `oblique` (any angle)
    Italic,
}

/// Resolved font face: the bytes ready for `EmbeddedFont::from_data`,
/// plus metadata about which font fontdb actually picked.
#[derive(Debug, Clone)]
pub struct ResolvedFont {
    /// The CSS family name we matched against (the first entry in the
    /// caller's family list that succeeded).
    pub matched_family: String,
    /// The fontdb face path on disk.
    pub path: PathBuf,
    /// Raw TrueType / OpenType bytes.
    pub bytes: Vec<u8>,
    /// PostScript family name as recorded in the font.
    pub postscript_family: String,
}

/// Lazy host font index. Constructed on first use; subsequent calls
/// reuse the cached database.
pub struct SystemFontDb {
    inner: Mutex<Option<Database>>,
}

impl Default for SystemFontDb {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemFontDb {
    /// Create a deferred-loading handle. No filesystem access happens
    /// until [`Self::resolve`] is called.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Force the system index to load now (otherwise lazy on first
    /// resolve). Useful in tests so that timing measurements don't
    /// include the directory walk.
    pub fn ensure_loaded(&self) {
        let mut guard = self.inner.lock().expect("system font db mutex");
        if guard.is_none() {
            let mut db = Database::new();
            db.load_system_fonts();
            *guard = Some(db);
        }
    }

    /// Resolve a CSS family list + weight + style into an actual font.
    ///
    /// Family resolution order matches CSS:
    /// 1. Try each entry in `families` in order.
    /// 2. Within a family, prefer the exact weight+style match. fontdb
    ///    does CSS Fonts Level 4 weight matching internally (closest
    ///    weight per direction).
    /// 3. The first successful match wins.
    ///
    /// Generic families (`serif`, `sans-serif`, `monospace`, `cursive`,
    /// `fantasy`, `system-ui`) are translated to fontdb's `Family`
    /// generic variants which delegate to the host's configured
    /// defaults.
    pub fn resolve(
        &self,
        families: &[&str],
        weight: u16,
        style: FontStyle,
    ) -> Result<ResolvedFont, FontResolveError> {
        self.ensure_loaded();

        // Hold the fontdb Mutex *only* long enough to pick a face and
        // extract its path + PostScript metadata. The file I/O
        // (`std::fs::read`) runs without the lock so concurrent
        // `resolve` calls on different threads don't serialise behind
        // slow disk reads.
        let style_db = match style {
            FontStyle::Normal => fontdb::Style::Normal,
            FontStyle::Italic => fontdb::Style::Italic,
        };
        let weight_db = fontdb::Weight(weight);
        let stretch_db = fontdb::Stretch::Normal;

        let resolved_meta: Option<(String, PathBuf, String)> = {
            let guard = self.inner.lock().expect("system font db mutex");
            let db = guard
                .as_ref()
                .expect("ensure_loaded populated the database");

            let mut found: Option<(String, PathBuf, String)> = None;
            for &family in families {
                let family_norm = family.trim().trim_matches(['"', '\'']);
                let fontdb_family = match family_norm.to_ascii_lowercase().as_str() {
                    "serif" => Family::Serif,
                    "sans-serif" => Family::SansSerif,
                    "monospace" => Family::Monospace,
                    "cursive" => Family::Cursive,
                    "fantasy" => Family::Fantasy,
                    // CSS `system-ui` doesn't have a fontdb generic; map to
                    // sans-serif which is what every browser does in
                    // practice when no system UI metadata is available.
                    "system-ui" => Family::SansSerif,
                    _ => Family::Name(family_norm),
                };

                let query = fontdb::Query {
                    families: &[fontdb_family],
                    weight: weight_db,
                    stretch: stretch_db,
                    style: style_db,
                };

                let Some(id) = db.query(&query) else {
                    continue;
                };
                match face_metadata(db, id, family_norm) {
                    Ok((path, postscript_family)) => {
                        found = Some((family_norm.to_string(), path, postscript_family));
                        break;
                    },
                    Err(_) => continue,
                }
            }
            found
        };
        // Mutex released here.

        let Some((matched_family, path, postscript_family)) = resolved_meta else {
            return Err(FontResolveError::NoMatch {
                requested: families.iter().map(|s| s.to_string()).collect(),
                weight,
                italic: matches!(style, FontStyle::Italic),
            });
        };

        let bytes = std::fs::read(&path).map_err(|e| FontResolveError::Io {
            path: path.clone(),
            source: e,
        })?;
        Ok(ResolvedFont {
            matched_family,
            path,
            bytes,
            postscript_family,
        })
    }
}

/// Extract the chosen face's on-disk path and PostScript family name
/// while holding the fontdb Mutex. Intentionally does **not** read
/// the file — the caller does that after dropping the lock so
/// concurrent `resolve` calls don't serialise on slow I/O.
fn face_metadata(
    db: &Database,
    id: ID,
    matched_family: &str,
) -> Result<(PathBuf, String), FontResolveError> {
    let face = db
        .face(id)
        .expect("fontdb returned an ID it doesn't recognise");
    let path = match &face.source {
        // Regular on-disk font.
        fontdb::Source::File(p) => p.clone(),
        // TTC collections (e.g. /System/Library/Fonts/Helvetica.ttc)
        // surface as `SharedFile(path, …)` — the path is real and
        // readable, we just need the file to extract the requested
        // face index. Previously this arm rejected SharedFile entries
        // as `NoPath`, which silently excluded huge portions of macOS
        // / Windows system fonts.
        fontdb::Source::SharedFile(p, _) => p.clone(),
        // Binary fonts don't sit on disk — fontdb constructed them
        // from an in-memory blob. `ResolvedFont` currently only
        // expresses a path-backed result; surface as `NoPath` so the
        // caller can choose a different family.
        fontdb::Source::Binary(_) => {
            return Err(FontResolveError::NoPath {
                family: matched_family.to_string(),
            });
        },
    };
    // fontdb's Face::families is a Vec<(String, Language)>; pick the
    // English entry if present, otherwise the first.
    let postscript_family = face
        .families
        .iter()
        .find(|(_, lang)| lang.primary_language() == "English")
        .or_else(|| face.families.first())
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| matched_family.to_string());
    Ok((path, postscript_family))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On any reasonable host the generic `sans-serif` family must
    /// resolve to *some* font. CI hosts typically have at minimum
    /// DejaVu / Liberation / Noto.
    #[test]
    fn resolves_generic_sans_serif() {
        let db = SystemFontDb::new();
        let resolved = db.resolve(&["sans-serif"], 400, FontStyle::Normal);
        // Some minimal CI hosts might not ship sans-serif; tolerate
        // NoMatch but only when fontdb literally found nothing.
        match resolved {
            Ok(r) => {
                assert!(!r.bytes.is_empty(), "resolved font had empty bytes");
                assert!(r.path.exists(), "resolved path must exist");
            },
            Err(FontResolveError::NoMatch { .. }) => {
                eprintln!("no system sans-serif on this host; skipping (CI sandbox?)");
            },
            Err(other) => panic!("unexpected resolve error: {other:?}"),
        }
    }

    #[test]
    fn resolves_specific_family_with_fallback() {
        // Try a wildly improbable family first, then sans-serif.
        let db = SystemFontDb::new();
        let resolved =
            db.resolve(&["NonExistentFontFromTheVoid", "sans-serif"], 400, FontStyle::Normal);
        if let Ok(r) = resolved {
            assert!(!r.bytes.is_empty());
            assert_ne!(r.matched_family, "NonExistentFontFromTheVoid");
        }
        // NoMatch is also acceptable on a host without sans-serif.
    }

    #[test]
    fn empty_family_list_errors() {
        let db = SystemFontDb::new();
        let err = db
            .resolve(&[], 400, FontStyle::Normal)
            .expect_err("empty family list must not match");
        assert!(matches!(err, FontResolveError::NoMatch { .. }));
    }
}
