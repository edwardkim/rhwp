//! Provenance of a decoded character's Unicode value.
//!
//! [`MappingProvenance`] records *which tier of the ISO 32000-1 §9.10.2
//! character-to-Unicode cascade produced a character's Unicode value* — or that
//! none did and the value was chosen by fallback. It is a **fact** about how the
//! value was derived, not a **judgment** about whether the text is "correct" or
//! "corrupt".
//!
//! The library deliberately exposes this fact rather than shipping the
//! judgments callers build from it. From provenance a caller composes their own
//! policy — route a page to OCR, flag low-confidence runs, detect a "text layer
//! that can't be read", or keep the raw glyph-index echo for an intentional
//! payload — without the library having to add a flag per use case. See §9.10.2:
//! when every method fails, "there is no way to determine what the character
//! code represents … a conforming reader may choose a character code of their
//! choosing" — that case is [`MappingProvenance::Fallback`].
//!
//! Variants are ordered most-authoritative to least. [`MappingProvenance::rank`]
//! exposes that order so a multi-character span can be summarised by its
//! *weakest* character (the honest signal for "does this run contain anything
//! fabricated?").

/// Which §9.10.2 tier produced a character's Unicode value.
///
/// Serializes as the same stable lowercase labels [`Self::as_str`] returns, so
/// every serde-based binding (WASM/JSON) and every explicit accessor agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[non_exhaustive]
pub enum MappingProvenance {
    /// An `/ActualText` replacement on a structure element or marked-content
    /// sequence (§14.9.4). An explicit, authoritative override of the shown
    /// characters.
    #[serde(rename = "actual_text")]
    ActualText,
    /// The font's `/ToUnicode` CMap (§9.10.3) — the authoritative per-font map.
    #[serde(rename = "to_unicode")]
    ToUnicode,
    /// The font `/Encoding` → glyph name → Adobe Glyph List path (§9.10.2, the
    /// simple-font branch).
    #[serde(rename = "encoding")]
    EncodingName,
    /// A predefined CID→Unicode CMap for a known character collection (§9.10.2,
    /// e.g. `Adobe-Japan1-UCS2`).
    #[serde(rename = "predefined_cmap")]
    PredefinedCMap,
    /// Inversion of the embedded font program's own `cmap` table — the
    /// recoverable byte-as-GID / Identity subset shape.
    #[serde(rename = "embedded_cmap")]
    EmbeddedCmap,
    /// No mapping tier produced a value; the character was chosen by fallback
    /// (a CID-as-Unicode echo, or `U+FFFD`). Per §9.10.2 the character code's
    /// meaning is undetermined, so any Unicode here is **fabricated by the
    /// extractor, not read from the file**.
    #[serde(rename = "fallback")]
    Fallback,
}

impl MappingProvenance {
    /// Authority rank, `0` = most authoritative (`ActualText`) … `5` = least
    /// (`Fallback`). Lower is stronger, so `max` over a span's characters yields
    /// the span's weakest provenance.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::ActualText => 0,
            Self::ToUnicode => 1,
            Self::EncodingName => 2,
            Self::PredefinedCMap => 3,
            Self::EmbeddedCmap => 4,
            Self::Fallback => 5,
        }
    }

    /// Was this value actually read from the file (any real mapping tier), as
    /// opposed to fabricated by the [`Fallback`](Self::Fallback) path?
    #[must_use]
    pub fn is_from_file(self) -> bool {
        !matches!(self, Self::Fallback)
    }

    /// A stable, lowercase label for bindings and serialized surfaces. Shared
    /// so every language binding exposes the same strings:
    /// `"actual_text"`, `"to_unicode"`, `"encoding"`, `"predefined_cmap"`,
    /// `"embedded_cmap"`, `"fallback"`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ActualText => "actual_text",
            Self::ToUnicode => "to_unicode",
            Self::EncodingName => "encoding",
            Self::PredefinedCMap => "predefined_cmap",
            Self::EmbeddedCmap => "embedded_cmap",
            Self::Fallback => "fallback",
        }
    }

    /// The weaker (less authoritative) of two provenances — the reduction used
    /// to summarise a run of characters by its least-trustworthy member.
    #[must_use]
    pub fn weaker(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_orders_most_to_least_authoritative() {
        let ordered = [
            MappingProvenance::ActualText,
            MappingProvenance::ToUnicode,
            MappingProvenance::EncodingName,
            MappingProvenance::PredefinedCMap,
            MappingProvenance::EmbeddedCmap,
            MappingProvenance::Fallback,
        ];
        for pair in ordered.windows(2) {
            assert!(pair[0].rank() < pair[1].rank(), "{:?} must outrank {:?}", pair[0], pair[1]);
        }
    }

    #[test]
    fn only_fallback_is_not_from_file() {
        assert!(!MappingProvenance::Fallback.is_from_file());
        for p in [
            MappingProvenance::ActualText,
            MappingProvenance::ToUnicode,
            MappingProvenance::EncodingName,
            MappingProvenance::PredefinedCMap,
            MappingProvenance::EmbeddedCmap,
        ] {
            assert!(p.is_from_file(), "{p:?} is a real mapping tier");
        }
    }

    #[test]
    fn weaker_summarises_a_run_by_its_least_trustworthy_char() {
        // A span with one fabricated char is, as a whole, fabricated.
        let span = MappingProvenance::ToUnicode
            .weaker(MappingProvenance::ToUnicode)
            .weaker(MappingProvenance::Fallback);
        assert_eq!(span, MappingProvenance::Fallback);
        // A clean span keeps its strongest-consistent provenance.
        let clean = MappingProvenance::ToUnicode.weaker(MappingProvenance::ActualText);
        assert_eq!(clean, MappingProvenance::ToUnicode);
    }
}
