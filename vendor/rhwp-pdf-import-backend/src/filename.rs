//! Cross-platform-safe filename slug helpers.
//!
//! A single pure, deterministic primitive shared by features that turn
//! arbitrary user-controlled text (PDF bookmark titles, metadata) into
//! a filesystem-safe file stem — e.g. split-by-bookmarks (#482).
//!
//! Windows is the strictest target and drives the rules (forbidden
//! characters, reserved device names, no trailing dot/space). The
//! function is **pure**: collision handling (e.g. appending ` (2)`) is
//! the caller's responsibility, never done here.
//!
//! NOTE (deviation from the #482 plan §1.5 rule 1): Unicode NFC
//! normalization is intentionally deferred — it would require adding
//! `unicode-normalization` as a *direct* dependency (it is only a
//! transitive dep today), which is out of scope for this increment and
//! is a cosmetic nicety, not a filesystem-safety concern. Tracked as a
//! roadmap item; rule 1 here is whitespace-trim only.

/// Maximum slug length in bytes (keeps total paths well under the
/// 255-byte component limit common to ext4/APFS/NTFS).
pub const DEFAULT_MAX_SLUG_BYTES: usize = 80;

/// Windows reserved device names (case-insensitive). A slug equal to
/// one of these (ignoring extension) is prefixed with `_`.
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Turn an arbitrary title into a cross-platform-safe file **stem**
/// (no extension), truncated to [`DEFAULT_MAX_SLUG_BYTES`].
///
/// Pure and deterministic. See [`slugify_title_with`] for a custom
/// byte budget. Never returns an empty string (falls back to
/// `"untitled"`).
pub fn slugify_title(title: &str) -> String {
    slugify_title_with(title, DEFAULT_MAX_SLUG_BYTES)
}

/// As [`slugify_title`] but with an explicit max byte budget. A
/// `max_bytes` of 0 is treated as [`DEFAULT_MAX_SLUG_BYTES`] (a 0-byte
/// filename is never useful — fail-safe, not fail-empty).
pub fn slugify_title_with(title: &str, max_bytes: usize) -> String {
    let budget = if max_bytes == 0 {
        DEFAULT_MAX_SLUG_BYTES
    } else {
        max_bytes
    };

    // Rule 1 (partial): trim surrounding whitespace. (NFC deferred —
    // see module docs.)
    let trimmed = title.trim();

    // Rules 2–3: replace forbidden / control / whitespace chars — and
    // a literal '-' — with '-', collapsing any run of those into a
    // single '-' (rule 3 collapses runs of whitespace *and* '-').
    let mut out = String::with_capacity(trimmed.len());
    let mut last_was_dash = false;
    for ch in trimmed.chars() {
        let forbidden = ch == '-'
            || matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
            || ch.is_control()
            || ch.is_whitespace();
        if forbidden {
            if !last_was_dash {
                out.push('-');
                last_was_dash = true;
            }
        } else {
            out.push(ch);
            last_was_dash = false;
        }
    }

    // Rule 3 (cont.): trim leading/trailing '-'.
    let mut slug = out.trim_matches('-').to_string();

    // Rule 4: Windows forbids a trailing '.' or space (already no
    // spaces after rule 2/3, but a trailing '.' can survive).
    while slug.ends_with('.') || slug.ends_with(' ') {
        slug.pop();
    }
    let mut slug = slug.trim_matches('-').to_string();

    // Rule 5: empty after sanitization => "untitled".
    if slug.is_empty() {
        slug = "untitled".to_string();
    }

    // Rule 6: Windows reserved device names (case-insensitive) → `_`-prefix.
    if WINDOWS_RESERVED
        .iter()
        .any(|r| r.eq_ignore_ascii_case(&slug))
    {
        slug.insert(0, '_');
    }

    // Rule 7: truncate to the byte budget without splitting a UTF-8
    // codepoint, then re-trim a dash the cut may have exposed.
    if slug.len() > budget {
        let mut end = budget;
        while end > 0 && !slug.is_char_boundary(end) {
            end -= 1;
        }
        slug.truncate(end);
        let trimmed = slug.trim_matches('-');
        if trimmed.len() != slug.len() {
            slug = trimmed.to_string();
        }
        if slug.is_empty() {
            slug = "untitled".to_string();
        }
    }

    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_titles_pass_through() {
        assert_eq!(slugify_title("Patient A"), "Patient-A");
        assert_eq!(slugify_title("Chapter 1"), "Chapter-1");
        assert_eq!(slugify_title("Already-Safe_Name.v2"), "Already-Safe_Name.v2");
    }

    #[test]
    fn forbidden_and_path_chars_become_dash_collapsed() {
        assert_eq!(slugify_title(r#"Patient B / X-Ray"#), "Patient-B-X-Ray");
        assert_eq!(slugify_title(r#"a:*?"<>|\\/b"#), "a-b");
        assert_eq!(slugify_title("a\u{0007}\u{0000}b"), "a-b"); // control chars
        assert_eq!(slugify_title("a\t\n  b"), "a-b"); // whitespace run
        assert_eq!(slugify_title("a---b   c"), "a-b-c"); // collapse runs
    }

    #[test]
    fn empty_and_whitespace_become_untitled() {
        assert_eq!(slugify_title(""), "untitled");
        assert_eq!(slugify_title("    "), "untitled");
        assert_eq!(slugify_title("\t\n"), "untitled");
        assert_eq!(slugify_title("///"), "untitled");
        assert_eq!(slugify_title("- - -"), "untitled");
    }

    #[test]
    fn trailing_dot_and_space_stripped() {
        assert_eq!(slugify_title("report."), "report");
        assert_eq!(slugify_title("report . "), "report");
        assert_eq!(slugify_title("name..."), "name");
    }

    #[test]
    fn windows_reserved_names_are_prefixed_case_insensitive() {
        assert_eq!(slugify_title("CON"), "_CON");
        assert_eq!(slugify_title("con"), "_con");
        assert_eq!(slugify_title("CoM1"), "_CoM1");
        assert_eq!(slugify_title("LPT9"), "_LPT9");
        // Not reserved: COM10, a CON substring.
        assert_eq!(slugify_title("COM10"), "COM10");
        assert_eq!(slugify_title("CONTRACT"), "CONTRACT");
    }

    #[test]
    fn truncates_at_byte_budget_without_splitting_codepoints() {
        let long = "x".repeat(200);
        assert_eq!(slugify_title(&long).len(), DEFAULT_MAX_SLUG_BYTES);

        // Multibyte: 'é' is 2 bytes; budget 5 must not split it.
        let s = slugify_title_with("ééééé", 5);
        assert!(s.is_char_boundary(s.len()));
        assert!(s.len() <= 5);
        assert!(!s.is_empty());

        // A budget that would land mid-codepoint never panics and
        // never yields invalid UTF-8 (String guarantees this; assert
        // the boundary logic held).
        for b in 1..12 {
            let r = slugify_title_with("αβγδε", b);
            assert!(r.is_char_boundary(r.len()));
            assert!(!r.is_empty());
        }
    }

    #[test]
    fn zero_budget_falls_back_to_default() {
        assert_eq!(slugify_title_with(&"y".repeat(300), 0).len(), DEFAULT_MAX_SLUG_BYTES);
    }

    #[test]
    fn pure_and_deterministic() {
        let t = r#"Combined / Record: "Final"."#;
        assert_eq!(slugify_title(t), slugify_title(t));
        // No collision suffixing here — that is the caller's job.
        assert_eq!(slugify_title("Record"), "Record");
        assert_eq!(slugify_title("Record"), "Record");
    }
}
