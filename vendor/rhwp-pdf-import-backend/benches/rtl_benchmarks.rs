#![allow(clippy::useless_vec)]
//! Right-to-Left (RTL) Script Benchmarks
//!
//! This benchmark suite measures the performance of RTL script processing,
//! including Arabic and Hebrew text handling, diacritic detection, contextual
//! form normalization, and BiDi text processing.
//!
//! ## Performance Targets
//!
//! - Diacritic detection: <10µs per check
//! - Contextual form normalization: <30µs
//! - Boundary detection: <20µs
//! - LAM-ALEF handling: <15µs

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::text::rtl_detector::*;
use pdf_oxide::text::{BoundaryContext, CharacterInfo};
use std::hint::black_box;

// ============================================================================
// ARABIC DIACRITIC BENCHMARKS
// ============================================================================

fn bench_arabic_diacritics(c: &mut Criterion) {
    let mut group = c.benchmark_group("arabic_diacritics");
    group.sample_size(50);

    // Basic Arabic diacritics
    group.bench_function("basic_marks", |b| {
        let marks = vec![
            0x064B, // FATHATAN
            0x064C, // DAMMATAN
            0x064D, // KASRATAN
            0x064E, // FATHA
            0x064F, // DAMMA
            0x0650, // KASRA
            0x0651, // SHADDA
            0x0652, // SUKUN
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_arabic_diacritic(black_box(code)));
            }
        });
    });

    // Extended Arabic marks
    group.bench_function("extended_marks", |b| {
        let marks = vec![
            0x06D6, 0x06D7, 0x06D8, 0x06D9, 0x06DA, 0x06DB, 0x06DC, 0x06DF, 0x06E0, 0x06E1,
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_arabic_diacritic(black_box(code)));
            }
        });
    });

    // Arabic letter detection
    group.bench_function("arabic_letters", |b| {
        let letters = vec![
            0x0627, // ALEF
            0x0644, // LAM
            0x0633, // SEEN
            0x0645, // MEEM
            0x0639, // AIN
        ];
        b.iter(|| {
            for &code in &letters {
                black_box(is_arabic_letter(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// HEBREW DIACRITIC BENCHMARKS
// ============================================================================

fn bench_hebrew_diacritics(c: &mut Criterion) {
    let mut group = c.benchmark_group("hebrew_diacritics");
    group.sample_size(50);

    // Hebrew vowel points
    group.bench_function("vowel_points", |b| {
        let marks = vec![
            0x05B0, // SHEVA
            0x05B1, // HATAF SEGOL
            0x05B4, // HIRIQ
            0x05B5, // TSERE
            0x05B8, // QAMATS
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_hebrew_diacritic(black_box(code)));
            }
        });
    });

    // Hebrew other marks
    group.bench_function("other_marks", |b| {
        let marks = vec![
            0x05BC, // DAGESH
            0x05BD, // METEG
            0x05BF, // RAFE
            0x05C1, // SHIN DOT
            0x05C2, // SIN DOT
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_hebrew_diacritic(black_box(code)));
            }
        });
    });

    // Hebrew letter detection
    group.bench_function("hebrew_letters", |b| {
        let letters = vec![
            0x05D0, // ALEF
            0x05D1, // BET
            0x05D2, // GIMEL
            0x05D3, // DALET
            0x05D4, // HE
        ];
        b.iter(|| {
            for &code in &letters {
                black_box(is_hebrew_letter(black_box(code)));
            }
        });
    });

    // Hebrew punctuation
    group.bench_function("hebrew_punctuation", |b| {
        let punct = vec![0x05F3, 0x05F4]; // GERESH, GERSHAYIM
        b.iter(|| {
            for &code in &punct {
                black_box(is_hebrew_punctuation(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// RTL DIACRITIC DETECTION (COMBINED)
// ============================================================================

fn bench_rtl_diacritic_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtl_diacritic_combined");
    group.sample_size(50);

    // Mixed Arabic and Hebrew diacritics
    group.bench_function("mixed_diacritics", |b| {
        let marks = vec![
            0x064E, // Arabic FATHA
            0x05B4, // Hebrew HIRIQ
            0x0651, // Arabic SHADDA
            0x05BC, // Hebrew DAGESH
            0x0652, // Arabic SUKUN
            0x05B8, // Hebrew QAMATS
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_rtl_diacritic(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// ARABIC CONTEXTUAL FORM NORMALIZATION
// ============================================================================

fn bench_arabic_contextual_forms(c: &mut Criterion) {
    let mut group = c.benchmark_group("arabic_contextual_forms");
    group.sample_size(50);

    // Normalize presentation forms
    group.bench_function("normalize_forms", |b| {
        let forms = vec![
            0xFB50, // ALEF WASLA (isolated)
            0xFE82, // ALEF (final)
            0xFE8D, // ALEF (isolated)
            0xFE8F, // BEH (isolated)
            0xFE90, // BEH (final)
            0xFE91, // BEH (initial)
            0xFE92, // BEH (medial)
        ];
        b.iter(|| {
            for &code in &forms {
                black_box(normalize_arabic_contextual_form(black_box(code)));
            }
        });
    });

    // Non-presentation forms (fast path)
    group.bench_function("normalize_base_chars", |b| {
        let base_chars = vec![
            0x0627, // ALEF
            0x0628, // BEH
            0x062A, // TEH
            0x062B, // THEH
            0x062C, // JEEM
        ];
        b.iter(|| {
            for &code in &base_chars {
                black_box(normalize_arabic_contextual_form(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// LAM-ALEF LIGATURE HANDLING
// ============================================================================

fn bench_lam_alef_ligatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("lam_alef_ligatures");
    group.sample_size(50);

    // LAM-ALEF detection
    group.bench_function("is_lam_alef", |b| {
        let ligatures = vec![
            0xFEF5, 0xFEF6, // LAM with ALEF WITH MADDA ABOVE
            0xFEF7, 0xFEF8, // LAM with ALEF WITH HAMZA ABOVE
            0xFEF9, 0xFEFA, // LAM with ALEF WITH HAMZA BELOW
            0xFEFB, 0xFEFC, // LAM with ALEF
        ];
        b.iter(|| {
            for &code in &ligatures {
                black_box(is_lam_alef_ligature(black_box(code)));
            }
        });
    });

    // LAM-ALEF decomposition
    group.bench_function("decompose_lam_alef", |b| {
        let ligatures = vec![
            0xFEFB, // LAM + ALEF
            0xFEF5, // LAM + ALEF WITH MADDA ABOVE
            0xFEF7, // LAM + ALEF WITH HAMZA ABOVE
            0xFEF9, // LAM + ALEF WITH HAMZA BELOW
        ];
        b.iter(|| {
            for &code in &ligatures {
                black_box(decompose_lam_alef(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// NUMBER HANDLING
// ============================================================================

fn bench_arabic_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("arabic_numbers");
    group.sample_size(50);

    // Eastern Arabic-Indic digits
    group.bench_function("eastern_arabic_digits", |b| {
        let digits = vec![
            0x06F0, 0x06F1, 0x06F2, 0x06F3, 0x06F4, 0x06F5, 0x06F6, 0x06F7, 0x06F8, 0x06F9,
        ];
        b.iter(|| {
            for &code in &digits {
                black_box(is_eastern_arabic_digit(black_box(code)));
            }
        });
    });

    // Mixed number detection (Western + Eastern)
    group.bench_function("mixed_numbers", |b| {
        let numbers = vec![
            0x0030, 0x0031, 0x0032, // Western: 0, 1, 2
            0x06F0, 0x06F1, 0x06F2, // Eastern: ٠, ١, ٢
        ];
        b.iter(|| {
            for &code in &numbers {
                black_box(is_arabic_number(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// RTL BOUNDARY DETECTION
// ============================================================================

fn bench_rtl_boundary_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtl_boundary_detection");
    group.sample_size(50);

    let context = BoundaryContext::new(12.0);

    // Arabic text boundaries
    group.bench_function("arabic_text", |b| {
        let chars = vec![
            create_char(0x0627, 0.0, 8.0),  // ALEF
            create_char(0x0644, 8.0, 8.0),  // LAM
            create_char(0x0633, 16.0, 8.0), // SEEN
            create_char(0x0020, 24.0, 3.0), // SPACE
            create_char(0x0639, 27.0, 8.0), // AIN
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(should_split_at_rtl_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                    black_box(Some(&context)),
                ));
            }
        });
    });

    // Hebrew text boundaries
    group.bench_function("hebrew_text", |b| {
        let chars = vec![
            create_char(0x05D0, 0.0, 8.0),  // ALEF
            create_char(0x05D1, 8.0, 8.0),  // BET
            create_char(0x05D2, 16.0, 8.0), // GIMEL
            create_char(0x0020, 24.0, 3.0), // SPACE
            create_char(0x05D3, 27.0, 8.0), // DALET
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(should_split_at_rtl_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                    black_box(Some(&context)),
                ));
            }
        });
    });

    // Arabic with diacritics (no boundaries)
    group.bench_function("arabic_with_diacritics", |b| {
        let chars = vec![
            create_char(0x0627, 0.0, 8.0), // ALEF
            create_char(0x064E, 0.0, 0.0), // FATHA (diacritic, no width)
            create_char(0x0644, 8.0, 8.0), // LAM
            create_char(0x0651, 8.0, 0.0), // SHADDA (diacritic)
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(should_split_at_rtl_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                    black_box(Some(&context)),
                ));
            }
        });
    });

    // Number sequences (no boundaries)
    group.bench_function("number_sequences", |b| {
        let chars = vec![
            create_char(0x0031, 0.0, 6.0),  // 1
            create_char(0x0032, 6.0, 6.0),  // 2
            create_char(0x0033, 12.0, 6.0), // 3
            create_char(0x06F0, 18.0, 6.0), // ٠
            create_char(0x06F1, 24.0, 6.0), // ١
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(should_split_at_rtl_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                    black_box(Some(&context)),
                ));
            }
        });
    });

    group.finish();
}

// ============================================================================
// BIDI TEXT PROCESSING
// ============================================================================

fn bench_bidi_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("bidi_text");
    group.sample_size(50);

    let context = BoundaryContext::new(12.0);

    // Mixed RTL/LTR text (script transitions)
    group.bench_function("rtl_ltr_transitions", |b| {
        let chars = vec![
            create_char(0x0627, 0.0, 8.0),  // Arabic ALEF
            create_char(0x0644, 8.0, 8.0),  // Arabic LAM
            create_char(0x0020, 16.0, 3.0), // SPACE
            create_char(0x0041, 19.0, 6.0), // Latin A
            create_char(0x0042, 25.0, 6.0), // Latin B
            create_char(0x0020, 31.0, 3.0), // SPACE
            create_char(0x05D0, 34.0, 8.0), // Hebrew ALEF
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(should_split_at_rtl_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                    black_box(Some(&context)),
                ));
            }
        });
    });

    group.finish();
}

// ============================================================================
// BATCH RTL PROCESSING
// ============================================================================

fn bench_batch_rtl_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_rtl");
    group.sample_size(20);

    for size in [10, 100, 1000] {
        // Arabic text
        group.bench_with_input(BenchmarkId::new("arabic", size), &size, |b, &size| {
            let chars = generate_arabic_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_rtl_script(black_box(code)));
                    black_box(is_arabic_diacritic(black_box(code)));
                    black_box(is_arabic_letter(black_box(code)));
                }
            });
        });

        // Hebrew text
        group.bench_with_input(BenchmarkId::new("hebrew", size), &size, |b, &size| {
            let chars = generate_hebrew_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_rtl_script(black_box(code)));
                    black_box(is_hebrew_diacritic(black_box(code)));
                    black_box(is_hebrew_letter(black_box(code)));
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_char(code: u32, x_pos: f32, width: f32) -> CharacterInfo {
    CharacterInfo {
        code,
        glyph_id: Some(1),
        width,
        x_position: x_pos,
        tj_offset: None,
        font_size: 12.0,
        is_ligature: false,
        original_ligature: None,
        protected_from_split: false,
    }
}

fn generate_arabic_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x0627, 0x0644, 0x0633, 0x0645, 0x0639, // Letters
        0x064E, 0x0651, 0x0652, // Diacritics
        0x0020, // Space
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

fn generate_hebrew_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x05D0, 0x05D1, 0x05D2, 0x05D3, 0x05D4, // Letters
        0x05B4, 0x05BC, 0x05B8, // Diacritics
        0x0020, // Space
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

criterion_group!(
    benches,
    bench_arabic_diacritics,
    bench_hebrew_diacritics,
    bench_rtl_diacritic_combined,
    bench_arabic_contextual_forms,
    bench_lam_alef_ligatures,
    bench_arabic_numbers,
    bench_rtl_boundary_detection,
    bench_bidi_text,
    bench_batch_rtl_processing,
);
criterion_main!(benches);
