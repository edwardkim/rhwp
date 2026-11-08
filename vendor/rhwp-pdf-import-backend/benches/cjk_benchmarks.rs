#![allow(clippy::useless_vec)]
//! CJK Processing Benchmarks
//!
//! This benchmark suite measures the performance of CJK-specific processing,
//! including punctuation detection, script transitions, and fullwidth character handling.
//!
//! ## Performance Targets
//!
//! - Punctuation detection: <10µs per check
//! - Script transition analysis: <20µs
//! - Fullwidth detection: O(1) range check

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::text::cjk_punctuation::*;
use pdf_oxide::text::script_detector::{
    detect_cjk_script, should_split_on_script_transition, DocumentLanguage,
};
use pdf_oxide::text::CharacterInfo;
use std::hint::black_box;

// ============================================================================
// CJK PUNCTUATION BENCHMARKS
// ============================================================================

fn bench_punctuation_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cjk_punctuation");
    group.sample_size(50);

    // Sentence-ending punctuation
    group.bench_function("sentence_ending", |b| {
        let chars = vec![
            0x3002, // IDEOGRAPHIC FULL STOP
            0xFF01, // FULLWIDTH EXCLAMATION MARK
            0xFF1F, // FULLWIDTH QUESTION MARK
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_sentence_ending_punctuation(black_box(code)));
            }
        });
    });

    // Enumeration punctuation
    group.bench_function("enumeration", |b| {
        let chars = vec![
            0x3001, // IDEOGRAPHIC COMMA
            0xFF0C, // FULLWIDTH COMMA
            0xFF1B, // FULLWIDTH SEMICOLON
            0xFF1A, // FULLWIDTH COLON
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_enumeration_punctuation(black_box(code)));
            }
        });
    });

    // Bracket punctuation
    group.bench_function("brackets", |b| {
        let chars = vec![
            0x3008, // LEFT ANGLE BRACKET
            0x3009, // RIGHT ANGLE BRACKET
            0xFF08, // FULLWIDTH LEFT PARENTHESIS
            0xFF09, // FULLWIDTH RIGHT PARENTHESIS
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_bracket_punctuation(black_box(code)));
            }
        });
    });

    // Other CJK punctuation
    group.bench_function("other", |b| {
        let chars = vec![
            0x3000, // IDEOGRAPHIC SPACE
            0x3003, // DITTO MARK
            0x30FB, // KATAKANA MIDDLE DOT
            0xFF0E, // FULLWIDTH FULL STOP
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_other_cjk_punctuation(black_box(code)));
            }
        });
    });

    // Fullwidth punctuation (combined)
    group.bench_function("fullwidth_combined", |b| {
        let chars = vec![
            0x3002, 0x3001, 0x3008, 0x3000, // Various fullwidth
            0xFF01, 0xFF0C, 0xFF08, 0xFF0E, // More fullwidth
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_fullwidth_punctuation(black_box(code)));
            }
        });
    });

    // Boundary score calculation
    group.bench_function("boundary_score", |b| {
        let chars = vec![
            0x3002, // Score: 1.0
            0x3001, // Score: 0.9
            0xFF08, // Score: 0.8
            0x30FB, // Score: 0.7
            0x002E, // Score: 0.0 (not CJK)
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(get_cjk_punctuation_boundary_score(black_box(code), None));
            }
        });
    });

    // Opening/closing bracket detection
    group.bench_function("bracket_direction", |b| {
        let chars = vec![
            0x3008, 0x3009, // Angle brackets
            0xFF08, 0xFF09, // Parentheses
            0xFF3B, 0xFF3D, // Square brackets
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(is_opening_bracket(black_box(code)));
                black_box(is_closing_bracket(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// SCRIPT TRANSITION BENCHMARKS
// ============================================================================

fn bench_script_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("cjk_script_transitions");
    group.sample_size(50);

    // Japanese text - Hiragana/Katakana transitions
    group.bench_function("japanese_transitions", |b| {
        let chars = vec![
            (0x3042, 0x30A2), // Hiragana -> Katakana
            (0x30A2, 0x3042), // Katakana -> Hiragana
            (0x3042, 0x4E00), // Hiragana -> Han
            (0x4E00, 0x3042), // Han -> Hiragana
        ];
        b.iter(|| {
            for &(prev, curr) in &chars {
                let prev_script = detect_cjk_script(prev);
                let curr_script = detect_cjk_script(curr);
                black_box(should_split_on_script_transition(
                    black_box(prev_script),
                    black_box(curr_script),
                    black_box(Some(DocumentLanguage::Japanese)),
                ));
            }
        });
    });

    // Korean text - Hangul/Hanja transitions
    group.bench_function("korean_transitions", |b| {
        let chars = vec![
            (0xAC00, 0x4E00), // Hangul -> Han
            (0x4E00, 0xAC00), // Han -> Hangul
            (0xAC00, 0xAC01), // Hangul -> Hangul
        ];
        b.iter(|| {
            for &(prev, curr) in &chars {
                let prev_script = detect_cjk_script(prev);
                let curr_script = detect_cjk_script(curr);
                black_box(should_split_on_script_transition(
                    black_box(prev_script),
                    black_box(curr_script),
                    black_box(Some(DocumentLanguage::Korean)),
                ));
            }
        });
    });

    // Chinese text - Han characters
    group.bench_function("chinese_transitions", |b| {
        let chars = vec![
            (0x4E00, 0x4E2D), // Han -> Han
            (0x4E2D, 0x6587), // Han -> Han
        ];
        b.iter(|| {
            for &(prev, curr) in &chars {
                let prev_script = detect_cjk_script(prev);
                let curr_script = detect_cjk_script(curr);
                black_box(should_split_on_script_transition(
                    black_box(prev_script),
                    black_box(curr_script),
                    black_box(Some(DocumentLanguage::Chinese)),
                ));
            }
        });
    });

    // Mixed CJK/Latin transitions
    group.bench_function("cjk_latin_transitions", |b| {
        let chars = vec![
            (0x4E00, 0x0041), // Han -> Latin
            (0x0041, 0x4E00), // Latin -> Han
            (0x3042, 0x0041), // Hiragana -> Latin
        ];
        b.iter(|| {
            for &(prev, curr) in &chars {
                let prev_script = detect_cjk_script(prev);
                let curr_script = detect_cjk_script(curr);
                // Script transition from CJK to non-CJK always splits
                black_box(prev_script.is_some() != curr_script.is_some());
            }
        });
    });

    group.finish();
}

// Japanese and Korean text processing benchmarks removed - those functions
// require additional parameters beyond what these benchmarks were testing.

// ============================================================================
// FULLWIDTH CHARACTER DETECTION
// ============================================================================

fn bench_fullwidth_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("fullwidth_detection");
    group.sample_size(50);

    // Fullwidth forms (U+FF00-U+FFEF)
    group.bench_function("fullwidth_forms", |b| {
        let chars = vec![
            0xFF01, // FULLWIDTH EXCLAMATION MARK
            0xFF21, // FULLWIDTH LATIN CAPITAL LETTER A
            0xFF41, // FULLWIDTH LATIN SMALL LETTER A
            0xFF10, // FULLWIDTH DIGIT ZERO
            0xFF5E, // FULLWIDTH TILDE
        ];
        b.iter(|| {
            for &code in &chars {
                // Fullwidth range check
                black_box(matches!(code, 0xFF00..=0xFFEF));
            }
        });
    });

    // Halfwidth Katakana
    group.bench_function("halfwidth_katakana", |b| {
        let chars = vec![
            0xFF61, // HALFWIDTH IDEOGRAPHIC FULL STOP
            0xFF65, // HALFWIDTH KATAKANA MIDDLE DOT
            0xFF71, // HALFWIDTH KATAKANA LETTER A
            0xFF9F, // HALFWIDTH KATAKANA SEMI-VOICED SOUND MARK
        ];
        b.iter(|| {
            for &code in &chars {
                // Halfwidth Katakana range check
                black_box(matches!(code, 0xFF61..=0xFF9F));
            }
        });
    });

    group.finish();
}

// ============================================================================
// BATCH CJK PROCESSING
// ============================================================================

fn bench_batch_cjk_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_cjk");
    group.sample_size(20);

    for size in [10, 100, 1000] {
        // Mixed Japanese text
        group.bench_with_input(BenchmarkId::new("japanese", size), &size, |b, &size| {
            let chars = generate_japanese_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_cjk_script(black_box(code)));
                    black_box(is_fullwidth_punctuation(black_box(code)));
                }
            });
        });

        // Chinese text
        group.bench_with_input(BenchmarkId::new("chinese", size), &size, |b, &size| {
            let chars = generate_chinese_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_cjk_script(black_box(code)));
                    black_box(is_fullwidth_punctuation(black_box(code)));
                }
            });
        });

        // Korean text
        group.bench_with_input(BenchmarkId::new("korean", size), &size, |b, &size| {
            let chars = generate_korean_text(size);
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_cjk_script(black_box(code)));
                    black_box(is_fullwidth_punctuation(black_box(code)));
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

#[allow(dead_code)]
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

fn generate_japanese_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x3053, 0x3093, 0x306B, 0x3061, 0x306F, // Hiragana
        0x30C6, 0x30B9, 0x30C8, // Katakana
        0x4E00, 0x4E8C, // Han
        0x3002, // Punctuation
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

fn generate_chinese_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0x4E00, 0x4E8C, 0x4E09, 0x4E2D, 0x6587, 0x5927, 0x5B66, 0x751F, 0x793E, 0x4F1A,
        0x3002, // Punctuation
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

fn generate_korean_text(size: usize) -> Vec<u32> {
    let chars = vec![
        0xD55C, 0xAD6D, 0xC5B4, // Hangul
        0x4E2D, 0x6587, // Han (Hanja)
        0x3002, // Punctuation
    ];
    (0..size).map(|i| chars[i % chars.len()]).collect()
}

criterion_group!(
    benches,
    bench_punctuation_detection,
    bench_script_transitions,
    bench_fullwidth_detection,
    bench_batch_cjk_processing,
);
criterion_main!(benches);
