//! Existing Unicode/text rules with request-owned output and normalization scratch.
use crate::{error::Result, import_storage::ImportStorage};
use unicode_normalization::{
    char::{canonical_combining_class, decompose_canonical},
    UnicodeNormalization,
};

fn backing(peak: usize, item: usize, allocation: ImportStorage<'_>) -> Result<usize> {
    if peak <= 4 {
        return Ok(0);
    }
    let mut capacity = 8usize;
    let mut total = capacity;
    while capacity < peak {
        capacity = capacity
            .checked_mul(2)
            .ok_or_else(|| allocation.reject("NFC growth size"))?;
        total = total
            .checked_add(capacity)
            .ok_or_else(|| allocation.reject("NFC growth work"))?;
    }
    total
        .checked_mul(item)
        .ok_or_else(|| allocation.reject("NFC scratch bytes"))
}

fn nfc(text: &str, allocation: ImportStorage<'_>) -> Result<String> {
    allocation.bytes(0)?;
    let mut stats = Some((0usize, 0usize, 0usize, 0usize));
    let mut largest_atomic = 0usize;
    for scalar in text.chars() {
        let mut expanded = Some(0usize);
        decompose_canonical(scalar, |value| {
            expanded = expanded.and_then(|n| n.checked_add(1));
            stats = stats.and_then(|(count, marks, run, longest)| {
                let count = count.checked_add(1)?;
                if canonical_combining_class(value) == 0 {
                    Some((count, marks, 0, longest))
                } else {
                    let marks = marks.checked_add(1)?;
                    let run = run.checked_add(1)?;
                    Some((count, marks, run, longest.max(run)))
                }
            });
        });
        largest_atomic =
            largest_atomic.max(expanded.ok_or_else(|| allocation.reject("NFC atomic expansion"))?);
    }
    let (count, marks, _, longest) =
        stats.ok_or_else(|| allocation.reject("NFC decomposition count"))?;
    // unicode-normalization 0.1.25: decomposition TinyVec<[(u8,char);4]>
    // receives one scalar's full decomposition before yielding. Reserve
    // the pending mark run plus the largest atomic expansion; a Hangul
    // syllable supplies up to three starters in that one callback batch.
    // Recomposition's
    // TinyVec<[char;4]> retains at most that run. tinyvec 1.12.0 spills
    // to eight entries, then Vec::push doubles. Pending-run stable sorts
    // visit disjoint mark runs, so their full scratch sums to marks.
    let decomposed = backing(
        longest
            .checked_add(largest_atomic)
            .ok_or_else(|| allocation.reject("NFC run size"))?,
        std::mem::size_of::<(u8, char)>(),
        allocation,
    )?;
    let recomposed = backing(longest, std::mem::size_of::<char>(), allocation)?;
    let work = marks
        .checked_mul(std::mem::size_of::<(u8, char)>())
        .and_then(|n| n.checked_add(decomposed))
        .and_then(|n| n.checked_add(recomposed))
        .ok_or_else(|| allocation.reject("NFC scratch work"))?;
    allocation.bytes(work)?;
    // NFC cannot produce more scalars than canonical decomposition.
    let maximum = count
        .checked_mul(4)
        .ok_or_else(|| allocation.reject("NFC output size"))?;
    let mut output = allocation.string_capacity(maximum)?;
    for scalar in text.nfc() {
        output.push(scalar);
    }
    Ok(output)
}
pub(crate) fn decimal(text: &str, allocation: ImportStorage<'_>) -> Result<String> {
    allocation.bytes(0)?;

    let chars = allocation.collect(text.chars())?;
    let mut out = allocation.string_capacity(text.len())?;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\u{00AC}' && i > 0 && chars[i - 1].is_ascii_digit() {
            // Unspaced: digit ¬ digit.
            if chars.get(i + 1).is_some_and(|n| n.is_ascii_digit()) {
                out.push('.');
                i += 1;
                continue;
            }
            // Spaced: digit ¬ <single space> digit — drop the lone space.
            if chars.get(i + 1) == Some(&' ')
                && chars.get(i + 2).is_some_and(|n| n.is_ascii_digit())
            {
                out.push('.');
                i += 2; // skip the ¬ and the single separating space
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    Ok(out)
}

pub(crate) fn compose_marks(
    spans: &mut Vec<crate::layout::TextSpan>,
    allocation: ImportStorage<'_>,
) -> Result<()> {
    allocation.bytes(0)?;

    fn combining_for(spacing: char) -> Option<char> {
        Some(match spacing {
            '\u{00B4}' => '\u{0301}', // acute
            '\u{0060}' => '\u{0300}', // grave
            '\u{005E}' => '\u{0302}', // circumflex
            '\u{02C6}' => '\u{0302}', // modifier-letter circumflex
            '\u{007E}' => '\u{0303}', // tilde
            '\u{02DC}' => '\u{0303}', // small tilde
            '\u{00A8}' => '\u{0308}', // diaeresis
            '\u{00AF}' => '\u{0304}', // macron
            '\u{02C9}' => '\u{0304}', // modifier-letter macron
            '\u{00B8}' => '\u{0327}', // cedilla
            '\u{02DA}' => '\u{030A}', // ring above
            _ => return None,
        })
    }

    // First pass: spans that already got merged at the extractor
    // (when the LaTeX `(´)(Ecole)` pair both sit at the same
    // text-matrix origin the upstream merge_adjacent_spans pulls
    // them into a single "´Ecole" span). Fold the leading
    // diacritic + base letter into the precomposed form.
    for span in spans.iter_mut() {
        let mut iter = span.text.chars();
        let Some(d) = iter.next() else { continue };
        let Some(base) = iter.next() else { continue };
        let Some(combining) = combining_for(d) else {
            continue;
        };
        if !base.is_alphabetic() {
            continue;
        }
        let rest_start = d.len_utf8() + base.len_utf8();
        let mut composed = allocation.string_capacity(
            span.text
                .len()
                .checked_add(2)
                .ok_or_else(|| allocation.reject("combining text size"))?,
        )?;
        composed.push(base);
        composed.push(combining);
        composed.push_str(&span.text[rest_start..]);
        span.text = nfc(&composed, allocation)?;
    }

    // Walk spans pairwise. The diacritic is on its own one-
    // character span; the next span carries the base letter.
    let mut i = 0;
    while i + 1 < spans.len() {
        let mark_char = {
            let s = &spans[i];
            let mut iter = s.text.chars();
            let first = iter.next();
            let rest = iter.next();
            if rest.is_some() {
                None
            } else {
                first.and_then(combining_for)
            }
        };
        let Some(combining) = mark_char else {
            i += 1;
            continue;
        };
        // Geometric: same line, diacritic anchored over the base
        // letter's left edge (within ±1 pt).
        let (same_line, overlaps_x) = {
            let p = &spans[i];
            let n = &spans[i + 1];
            let same = (p.bbox.y - n.bbox.y).abs() < p.font_size.max(n.font_size) * 0.6;
            let dx = (p.bbox.x - n.bbox.x).abs();
            (same, dx <= 1.5)
        };
        if !(same_line && overlaps_x) {
            i += 1;
            continue;
        }
        // The next span must start with a base letter we can
        // attach a combining mark to (Latin letter / digit).
        let Some(base) = spans[i + 1].text.chars().next() else {
            i += 1;
            continue;
        };
        if !base.is_alphabetic() {
            i += 1;
            continue;
        }
        // Build "<base><combining><rest>" and NFC-compose.
        let mut composed = allocation.string_capacity(
            spans[i + 1]
                .text
                .len()
                .checked_add(2)
                .ok_or_else(|| allocation.reject("combining text size"))?,
        )?;
        composed.push(base);
        composed.push(combining);
        let rest_start = base.len_utf8();
        composed.push_str(&spans[i + 1].text[rest_start..]);
        spans[i + 1].text = nfc(&composed, allocation)?;
        // Empty out the diacritic span; downstream consumers
        // skip zero-text spans.
        spans[i].text.clear();
        i += 2;
    }

    // Drop any spans we emptied.
    spans.retain(|s| !s.text.is_empty());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        import_limits::{ImportBudget, PdfImportLimits},
        layout::TextSpan,
    };
    #[test]
    fn normalization_rejects_before_library_scratch_and_output_then_recovers() {
        for text in [
            "plain ASCII".to_owned(),
            format!("A{}", "\u{0315}\u{0300}".repeat(64)),
        ] {
            let tiny = ImportBudget::new(PdfImportLimits {
                decoded_bytes: 1,
                ..Default::default()
            })
            .unwrap();
            assert!(nfc(&text, ImportStorage(Some(&tiny))).is_err());
            assert_eq!(tiny.snapshot().decoded_bytes, 0);
            assert!(nfc("", ImportStorage(Some(&tiny))).is_err());
            let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
            assert_eq!(
                nfc(&text, ImportStorage(Some(&fresh))).unwrap(),
                text.nfc().collect::<String>()
            );
        }
    }
    #[test]
    fn a_hangul_atomic_decomposition_after_marks_cannot_exceed_a_small_budget() {
        // Independent GlobalAlloc observation: the pending two marks followed
        // by this three-Jamo decomposition allocate 64 scratch bytes, plus
        // the producer's 24-byte output buffer. An 80-byte request cannot fit.
        let text = "A\u{0315}\u{0300}각";
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 80,
            ..Default::default()
        })
        .unwrap();
        assert!(nfc(text, ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(nfc(text, ImportStorage(Some(&fresh))).unwrap(), text.nfc().collect::<String>());
    }
    #[test]
    fn combining_and_decimal_rules_preserve_text_when_first_reservation_fails() {
        let mut spans = vec![TextSpan {
            text: "´Ecole".into(),
            ..Default::default()
        }];
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(compose_marks(&mut spans, ImportStorage(Some(&tiny))).is_err());
        assert_eq!(spans[0].text, "´Ecole");
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        compose_marks(&mut spans, ImportStorage(Some(&fresh))).unwrap();
        assert_eq!(spans[0].text, "École");
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(decimal("1¬ 00", ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(decimal("1¬ 00 A ¬ B", ImportStorage(Some(&fresh))).unwrap(), "1.00 A ¬ B");
    }
    #[test]
    fn unicode_spaces_share_the_fallible_producer_and_omit_zero_width_separators() {
        use crate::converters::text_post_processor::TextPostProcessor;
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(TextPostProcessor::try_normalize_unicode_spaces(
            "a\u{2009}b\u{200B}c",
            ImportStorage(Some(&tiny))
        )
        .is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(
            TextPostProcessor::try_normalize_unicode_spaces(
                "a\u{2009}b\u{200B}c",
                ImportStorage(Some(&fresh))
            )
            .unwrap(),
            "a bc"
        );
    }
}
