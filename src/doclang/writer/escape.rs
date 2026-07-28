//! XML escaping helpers for the DocLang writer.
//!
//! The writer builds output via manual string concatenation (rather than a
//! streaming XML library) because DocLang's `<formula>` and `<custom>` elements
//! carry raw, already-formed payloads whose internal characters must be escaped
//! as ordinary text while the surrounding structure stays literal.  Manual
//! escaping keeps that distinction explicit and makes the OTSL table writer
//! (task T11) easier to layer on top.
//!
//! ## Whitespace handling
//!
//! DocLang v0.6 supports `xml:space="preserve"` on `<content>`, but v1 of this
//! writer does **not** emit it: we trim-normalise paragraph whitespace during
//! adapter lowering, so significant leading/trailing whitespace never reaches
//! the writer.  Tabs are emitted as literal U+0009 characters and hard line
//! breaks as U+000A; XML treats both as insignificant whitespace in element
//! content, which matches the lean-mode "structure, not pixel layout" goal.

/// Escape a string for use in XML **element text** content.
///
/// Escapes the three characters that are significant in character data: `&`,
/// `<`, and `>`.  Quote characters are left untouched (they are only
/// significant inside attribute values; see [`escape_attr`]).
pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            // XML 1.0 legal characters: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] |
            // [#x10000-#x10FFFF].  Anything else (control characters, U+FFFE/U+FFFF) is dropped
            // so the emitted document stays well-formed XML (#3382 class).
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(c)
            }
            _ => {} // XML-invalid character
        }
    }
    out
}

/// Escape a string for use in a double-quoted XML **attribute value**.
///
/// In addition to the character-data escapes, this escapes `"` and `'` so the
/// value is safe inside either quoting style; callers always emit attributes
/// with double quotes.
pub fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // See `escape_text`: XML 1.0 legal characters only (#3382 class).
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(c)
            }
            _ => {} // XML-invalid character
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_markup_chars_only() {
        assert_eq!(escape_text("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        // Quotes are NOT escaped in text content.
        assert_eq!(escape_text("\"x\" 'y'"), "\"x\" 'y'");
    }

    #[test]
    fn attr_escapes_quotes_too() {
        assert_eq!(
            escape_attr("a&b<c>\"d\"'e'"),
            "a&amp;b&lt;c&gt;&quot;d&quot;&apos;e&apos;"
        );
    }

    #[test]
    fn drops_xml_invalid_control_chars() {
        // #3382 class: XML 1.0 forbids most C0 controls outright — emitting them verbatim
        // produces a document that no conforming parser can read ("PCDATA invalid Char value 3").
        assert_eq!(escape_text("a\u{03}b"), "ab");
        assert_eq!(escape_attr("a\u{03}b"), "ab");
        for c in ['\u{00}', '\u{08}', '\u{0B}', '\u{0C}', '\u{0E}', '\u{1F}'] {
            assert_eq!(
                escape_text(&format!("x{c}y")),
                "xy",
                "control {:#04x}",
                c as u32
            );
        }
        // U+FFFE / U+FFFF are not XML characters either.
        assert_eq!(escape_text("a\u{FFFE}\u{FFFF}b"), "ab");
        // Tab / LF / CR stay — they are legal XML characters.
        assert_eq!(escape_text("a\tb\nc\rd"), "a\tb\nc\rd");
        // Ordinary text (incl. non-BMP) is untouched.
        assert_eq!(escape_text("한글 A\u{1F600}"), "한글 A\u{1F600}");
    }

    #[test]
    fn empty_string_round_trips() {
        assert_eq!(escape_text(""), "");
        assert_eq!(escape_attr(""), "");
    }

    #[test]
    fn unicode_is_preserved() {
        assert_eq!(escape_text("한글 & 漢字"), "한글 &amp; 漢字");
    }
}
