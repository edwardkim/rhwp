#![allow(clippy::useless_vec)]
//! Complex Script Processing Benchmarks
//!
//! This benchmark suite measures the performance of complex script processing,
//! including Devanagari, Thai, Khmer, and other South/Southeast Asian scripts.
//!
//! ## Performance Targets
//!
//! - Diacritic detection: <10µs per check
//! - Virama/COENG handling: <15µs
//! - Boundary detection: <20µs
//! - All 15 complex scripts: O(1) detection

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::text::complex_script_detector::{
    detect_complex_script, handle_devanagari_boundary, handle_indic_boundary,
    handle_khmer_boundary, handle_thai_boundary, is_complex_script, is_devanagari_anusvar_visarga,
    is_devanagari_consonant, is_devanagari_diacritic, is_devanagari_matra, is_devanagari_nukta,
    is_devanagari_virama, is_thai_digit, is_thai_major_punctuation, is_thai_tone_mark,
    is_thai_vowel_modifier,
};
use pdf_oxide::text::CharacterInfo;
use std::hint::black_box;

// ============================================================================
// DEVANAGARI BENCHMARKS
// ============================================================================

fn bench_devanagari(c: &mut Criterion) {
    let mut group = c.benchmark_group("devanagari");
    group.sample_size(50);

    // Diacritic detection
    group.bench_function("diacritics", |b| {
        let marks = vec![
            0x0902, 0x0903, // ANUSVARA, VISARGA
            0x093C, // NUKTA
            0x093E, 0x093F, 0x0940, // Matras
            0x094D, // VIRAMA
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_devanagari_diacritic(black_box(code)));
            }
        });
    });

    // Virama detection
    group.bench_function("virama", |b| {
        let codes = vec![0x094D, 0x0915, 0x0924, 0x093E]; // virama and non-virama
        b.iter(|| {
            for &code in &codes {
                black_box(is_devanagari_virama(black_box(code)));
            }
        });
    });

    // Consonant detection
    group.bench_function("consonants", |b| {
        let consonants = vec![
            0x0915, // KA
            0x0916, // KHA
            0x0917, // GA
            0x091A, // CA
            0x0924, // TA
        ];
        b.iter(|| {
            for &code in &consonants {
                black_box(is_devanagari_consonant(black_box(code)));
            }
        });
    });

    // Matra detection
    group.bench_function("matras", |b| {
        let matras = vec![
            0x093E, // AA
            0x093F, // I
            0x0940, // II
            0x0941, // U
            0x0942, // UU
        ];
        b.iter(|| {
            for &code in &matras {
                black_box(is_devanagari_matra(black_box(code)));
            }
        });
    });

    // Anusvara/Visarga detection
    group.bench_function("anusvara_visarga", |b| {
        let codes = vec![0x0902, 0x0903, 0x0915, 0x0924];
        b.iter(|| {
            for &code in &codes {
                black_box(is_devanagari_anusvar_visarga(black_box(code)));
            }
        });
    });

    // Nukta detection
    group.bench_function("nukta", |b| {
        let codes = vec![0x093C, 0x0915, 0x0924];
        b.iter(|| {
            for &code in &codes {
                black_box(is_devanagari_nukta(black_box(code)));
            }
        });
    });

    // Boundary handling
    group.bench_function("boundary_handling", |b| {
        let chars = vec![
            create_char(0x0928, 0.0, 7.0),  // NA
            create_char(0x092E, 7.0, 7.0),  // MA
            create_char(0x0938, 14.0, 7.0), // SA
            create_char(0x094D, 21.0, 0.0), // VIRAMA
            create_char(0x0924, 21.0, 7.0), // TA
            create_char(0x0947, 28.0, 0.0), // VOWEL SIGN E
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_devanagari_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                ));
            }
        });
    });

    group.finish();
}

// ============================================================================
// THAI BENCHMARKS
// ============================================================================

fn bench_thai(c: &mut Criterion) {
    let mut group = c.benchmark_group("thai");
    group.sample_size(50);

    // Tone mark detection
    group.bench_function("tone_marks", |b| {
        let marks = vec![
            0x0E48, // MAI EK
            0x0E49, // MAI THO
            0x0E4A, // MAI TRI
            0x0E4B, // MAI CHATTAWA
        ];
        b.iter(|| {
            for &code in &marks {
                black_box(is_thai_tone_mark(black_box(code)));
            }
        });
    });

    // Vowel modifier detection
    group.bench_function("vowel_modifiers", |b| {
        let modifiers = vec![
            0x0E31, // MAI HAN-AKAT
            0x0E34, 0x0E35, 0x0E36, 0x0E37, // Above vowels
            0x0E39, 0x0E3A, // Below vowels
        ];
        b.iter(|| {
            for &code in &modifiers {
                black_box(is_thai_vowel_modifier(black_box(code)));
            }
        });
    });

    // Digit detection
    group.bench_function("digits", |b| {
        let digits = vec![
            0x0E50, 0x0E51, 0x0E52, // Thai digits
            0x0030, 0x0031, 0x0032, // Western digits
        ];
        b.iter(|| {
            for &code in &digits {
                black_box(is_thai_digit(black_box(code)));
            }
        });
    });

    // Major punctuation detection
    group.bench_function("major_punctuation", |b| {
        let punct = vec![
            0x0E2F, // PAIYANNOI
            0x0E46, // MAIYAMOK
            0x0E4F, // FONGMAN
        ];
        b.iter(|| {
            for &code in &punct {
                black_box(is_thai_major_punctuation(black_box(code)));
            }
        });
    });

    // Boundary handling
    group.bench_function("boundary_handling", |b| {
        let chars = vec![
            create_char(0x0E01, 0.0, 7.0),  // KO KAI
            create_char(0x0E48, 0.0, 0.0),  // MAI EK (tone mark)
            create_char(0x0E32, 7.0, 7.0),  // SARA AA
            create_char(0x0E23, 14.0, 7.0), // RO RUA
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_thai_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

    group.finish();
}

// ============================================================================
// KHMER BENCHMARKS
// ============================================================================

fn bench_khmer(c: &mut Criterion) {
    let mut group = c.benchmark_group("khmer");
    group.sample_size(50);

    // COENG detection (Khmer subscript marker)
    group.bench_function("coeng", |b| {
        let codes = vec![0x17D2, 0x1780, 0x1781, 0x1782];
        b.iter(|| {
            for &code in &codes {
                // COENG is U+17D2
                black_box(code == 0x17D2);
            }
        });
    });

    // Khmer vowel sign detection
    group.bench_function("vowel_signs", |b| {
        let vowels = vec![
            0x17B6, 0x17B7, 0x17B8, 0x17B9, 0x17BA, 0x17BB, 0x17BC, 0x17BD,
        ];
        b.iter(|| {
            for &code in &vowels {
                // Khmer vowel signs range
                black_box(matches!(code, 0x17B6..=0x17BD));
            }
        });
    });

    // Boundary handling
    group.bench_function("boundary_handling", |b| {
        let chars = vec![
            create_char(0x1780, 0.0, 7.0), // KA
            create_char(0x17D2, 0.0, 0.0), // COENG (subscript marker)
            create_char(0x1781, 0.0, 7.0), // KHA (subscript)
            create_char(0x17B6, 7.0, 0.0), // AA vowel sign
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_khmer_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

    group.finish();
}

// ============================================================================
// OTHER INDIC SCRIPTS BENCHMARKS
// ============================================================================

fn bench_indic_scripts(c: &mut Criterion) {
    let mut group = c.benchmark_group("indic_scripts");
    group.sample_size(50);

    // Tamil boundary handling
    group.bench_function("tamil", |b| {
        let chars = vec![
            create_char(0x0B95, 0.0, 7.0), // KA
            create_char(0x0BBE, 7.0, 0.0), // AA vowel sign
            create_char(0x0B99, 7.0, 7.0), // NGA
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_indic_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

    // Telugu boundary handling
    group.bench_function("telugu", |b| {
        let chars = vec![
            create_char(0x0C15, 0.0, 7.0), // KA
            create_char(0x0C3E, 7.0, 0.0), // AA vowel sign
            create_char(0x0C17, 7.0, 7.0), // GA
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_indic_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

    // Bengali boundary handling
    group.bench_function("bengali", |b| {
        let chars = vec![
            create_char(0x0995, 0.0, 7.0), // KA
            create_char(0x09BE, 7.0, 0.0), // AA vowel sign
            create_char(0x0997, 7.0, 7.0), // GA
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_indic_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

    group.finish();
}

// ============================================================================
// COMPLEX SCRIPT DETECTION BENCHMARKS
// ============================================================================

fn bench_all_complex_scripts(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_complex_scripts");
    group.sample_size(50);

    // Test detection for all 15 complex scripts
    group.bench_function("detect_all_scripts", |b| {
        let test_codes = vec![
            0x0928, // Devanagari
            0x0995, // Bengali
            0x0A15, // Gurmukhi
            0x0A95, // Gujarati
            0x0B15, // Oriya
            0x0B95, // Tamil
            0x0C15, // Telugu
            0x0C95, // Kannada
            0x0D15, // Malayalam
            0x0D95, // Sinhala
            0x0E01, // Thai
            0x0E81, // Lao
            0x1780, // Khmer
            0x1000, // Burmese
            0x1820, // Mongolian
        ];

        b.iter(|| {
            for &code in &test_codes {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // is_complex_script convenience function
    group.bench_function("is_complex_script", |b| {
        let codes = vec![
            0x0928, 0x0995, 0x0E01, 0x1780, // Complex scripts
            0x0041, 0x4E00, 0x0627, // Non-complex
        ];

        b.iter(|| {
            for &code in &codes {
                black_box(is_complex_script(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// BATCH COMPLEX SCRIPT PROCESSING
// ============================================================================

fn bench_batch_complex_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_complex");
    group.sample_size(20);

    for size in [10, 100, 1000] {
        // Devanagari text
        group.bench_with_input(BenchmarkId::new("devanagari", size), &size, |b, &size| {
            let chars = generate_devanagari_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_complex_script(black_box(code)));
                    black_box(is_devanagari_diacritic(black_box(code)));
                    black_box(is_devanagari_virama(black_box(code)));
                }
            });
        });

        // Thai text
        group.bench_with_input(BenchmarkId::new("thai", size), &size, |b, &size| {
            let chars = generate_thai_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_complex_script(black_box(code)));
                    black_box(is_thai_tone_mark(black_box(code)));
                    black_box(is_thai_vowel_modifier(black_box(code)));
                }
            });
        });

        // Mixed complex scripts
        group.bench_with_input(BenchmarkId::new("mixed", size), &size, |b, &size| {
            let chars = generate_mixed_complex_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_complex_script(black_box(code)));
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// VIRAMA AND COENG HANDLING BENCHMARKS
// ============================================================================

fn bench_virama_coeng_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("virama_coeng");
    group.sample_size(50);

    // Devanagari virama (conjunct consonants)
    group.bench_function("devanagari_virama_sequence", |b| {
        let chars = vec![
            create_char(0x0915, 0.0, 7.0), // KA
            create_char(0x094D, 0.0, 0.0), // VIRAMA
            create_char(0x0937, 0.0, 7.0), // SHA (conjunct)
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_devanagari_boundary(
                    black_box(&chars[i]),
                    black_box(&chars[i + 1]),
                ));
            }
        });
    });

    // Khmer COENG (subscript consonants)
    group.bench_function("khmer_coeng_sequence", |b| {
        let chars = vec![
            create_char(0x1780, 0.0, 7.0), // KA
            create_char(0x17D2, 0.0, 0.0), // COENG
            create_char(0x1781, 0.0, 7.0), // KHA (subscript)
        ];

        b.iter(|| {
            for i in 0..chars.len() - 1 {
                black_box(handle_khmer_boundary(black_box(&chars[i]), black_box(&chars[i + 1])));
            }
        });
    });

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

fn generate_devanagari_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x0928, 0x092E, 0x0938, // Consonants
        0x094D, // VIRAMA
        0x093E, 0x0947, // Matras
        0x0902, 0x0903, // Anusvara, Visarga
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

fn generate_thai_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x0E01, 0x0E02, 0x0E03, // Consonants
        0x0E48, 0x0E49, // Tone marks
        0x0E31, 0x0E34, // Vowel modifiers
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

fn generate_mixed_complex_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x0928, // Devanagari
        0x0995, // Bengali
        0x0B95, // Tamil
        0x0E01, // Thai
        0x1780, // Khmer
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

criterion_group!(
    benches,
    bench_devanagari,
    bench_thai,
    bench_khmer,
    bench_indic_scripts,
    bench_all_complex_scripts,
    bench_batch_complex_processing,
    bench_virama_coeng_handling,
);
criterion_main!(benches);
