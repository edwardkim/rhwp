//! Word Boundary Detection Benchmarks
//!
//! This benchmark suite measures the performance of the word boundary detection system
//! across different character array sizes and script types.
//!
//! ## Performance Targets
//!
//! - Boundary detection: <5µs per boundary
//! - 1000 chars should take <50ms
//! - Throughput: >100,000 chars/second
//!
//! ## Baseline Performance (Week 1)
//!
//! - Per-character overhead: ~0.01µs
//! - 100 chars: ~1µs
//! - 1000 chars: ~10µs

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::text::{BoundaryContext, CharacterInfo, WordBoundaryDetector};
use std::hint::black_box;

// ============================================================================
// HELPER FUNCTIONS - Test Data Generation
// ============================================================================

/// Generate Latin text characters
fn generate_latin_characters(count: usize) -> Vec<CharacterInfo> {
    let text = "The quick brown fox jumps over the lazy dog. ";
    let mut chars = Vec::with_capacity(count);
    let mut x_pos = 0.0;
    let font_size = 12.0;

    for i in 0..count {
        let ch = text.chars().nth(i % text.len()).unwrap();
        let width = if ch == ' ' { 3.0 } else { 6.0 };

        chars.push(CharacterInfo {
            code: ch as u32,
            glyph_id: Some(i as u16),
            width,
            x_position: x_pos,
            tj_offset: if ch == ' ' { Some(-100) } else { None },
            font_size,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        });

        x_pos += width;
    }

    chars
}

/// Generate CJK text characters (Chinese)
fn generate_cjk_characters(count: usize) -> Vec<CharacterInfo> {
    // Common Chinese characters
    let cjk_chars = [
        0x4E00, // 一
        0x4E8C, // 二
        0x4E09, // 三
        0x4E2D, // 中
        0x6587, // 文
        0x5927, // 大
        0x5B66, // 学
        0x751F, // 生
        0x793E, // 社
        0x4F1A, // 会
    ];

    let mut chars = Vec::with_capacity(count);
    let mut x_pos = 0.0;
    let font_size = 12.0;
    let width = 12.0; // CJK characters are typically wider

    for i in 0..count {
        let code = cjk_chars[i % cjk_chars.len()];

        chars.push(CharacterInfo {
            code,
            glyph_id: Some(i as u16),
            width,
            x_position: x_pos,
            tj_offset: None,
            font_size,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        });

        x_pos += width;
    }

    chars
}

/// Generate mixed script characters (Latin, CJK, symbols)
fn generate_mixed_characters(count: usize) -> Vec<CharacterInfo> {
    let segments = [
        ("Hello world ", false),
        ("你好世界", true), // Chinese: "Hello world"
        (" Test ", false),
        ("テスト", true), // Japanese Katakana: "Test"
        (" Mixed ", false),
    ];

    let mut chars = Vec::with_capacity(count);
    let mut x_pos = 0.0;
    let font_size = 12.0;
    let mut segment_idx = 0;
    let mut char_in_segment = 0;

    for _i in 0..count {
        let (segment, is_cjk) = &segments[segment_idx % segments.len()];
        let segment_chars: Vec<char> = segment.chars().collect();

        if char_in_segment >= segment_chars.len() {
            segment_idx += 1;
            char_in_segment = 0;
            continue;
        }

        let ch = segment_chars[char_in_segment];
        let width = if *is_cjk {
            12.0
        } else if ch == ' ' {
            3.0
        } else {
            6.0
        };

        chars.push(CharacterInfo {
            code: ch as u32,
            glyph_id: None,
            width,
            x_position: x_pos,
            tj_offset: if ch == ' ' { Some(-100) } else { None },
            font_size,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        });

        x_pos += width;
        char_in_segment += 1;
    }

    chars
}

/// Generate Arabic RTL text characters
fn generate_arabic_characters(count: usize) -> Vec<CharacterInfo> {
    // Arabic letters (RTL)
    let arabic_chars = [
        0x0627, // ALEF
        0x0644, // LAM
        0x0633, // SEEN
        0x0644, // LAM
        0x0627, // ALEF
        0x0645, // MEEM
        0x0020, // SPACE
        0x0639, // AIN
        0x0644, // LAM
        0x064A, // YEH
    ];

    let mut chars = Vec::with_capacity(count);
    let mut x_pos = 0.0;
    let font_size = 12.0;

    for i in 0..count {
        let code = arabic_chars[i % arabic_chars.len()];
        let width = if code == 0x0020 { 3.0 } else { 8.0 };

        chars.push(CharacterInfo {
            code,
            glyph_id: Some(i as u16),
            width,
            x_position: x_pos,
            tj_offset: if code == 0x0020 { Some(-100) } else { None },
            font_size,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        });

        x_pos += width;
    }

    chars
}

/// Generate Devanagari text characters
fn generate_devanagari_characters(count: usize) -> Vec<CharacterInfo> {
    // Devanagari characters (Hindi)
    let devanagari_chars = [
        0x0928, // NA
        0x092E, // MA
        0x0938, // SA
        0x094D, // VIRAMA
        0x0924, // TA
        0x0947, // VOWEL SIGN E
        0x0020, // SPACE
    ];

    let mut chars = Vec::with_capacity(count);
    let mut x_pos = 0.0;
    let font_size = 12.0;

    for i in 0..count {
        let code = devanagari_chars[i % devanagari_chars.len()];
        let width = if code == 0x0020 { 3.0 } else { 7.0 };

        chars.push(CharacterInfo {
            code,
            glyph_id: Some(i as u16),
            width,
            x_position: x_pos,
            tj_offset: if code == 0x0020 { Some(-100) } else { None },
            font_size,
            is_ligature: false,
            original_ligature: None,
            protected_from_split: false,
        });

        x_pos += width;
    }

    chars
}

// ============================================================================
// BENCHMARKS - Word Boundary Detection
// ============================================================================

fn bench_word_boundary_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_boundary_small");
    group.sample_size(20);

    // 10 characters - Latin
    group.bench_function("detect_10_latin", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_latin_characters(10);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 10 characters - CJK
    group.bench_function("detect_10_cjk", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_cjk_characters(10);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 10 characters - Mixed
    group.bench_function("detect_10_mixed", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_mixed_characters(10);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    group.finish();
}

fn bench_word_boundary_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_boundary_medium");
    group.sample_size(20);

    // 100 characters - Latin
    group.bench_function("detect_100_latin", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_latin_characters(100);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 100 characters - CJK
    group.bench_function("detect_100_cjk", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_cjk_characters(100);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 100 characters - Mixed
    group.bench_function("detect_100_mixed", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_mixed_characters(100);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 100 characters - Arabic (RTL)
    group.bench_function("detect_100_arabic", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_arabic_characters(100);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 100 characters - Devanagari
    group.bench_function("detect_100_devanagari", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_devanagari_characters(100);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    group.finish();
}

fn bench_word_boundary_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_boundary_large");
    group.sample_size(20);

    // 1000 characters - Latin
    group.bench_function("detect_1000_latin", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_latin_characters(1000);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 1000 characters - CJK
    group.bench_function("detect_1000_cjk", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_cjk_characters(1000);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    // 1000 characters - Mixed
    group.bench_function("detect_1000_mixed", |b| {
        let detector = WordBoundaryDetector::new();
        let chars = generate_mixed_characters(1000);
        let context = BoundaryContext::new(12.0);

        b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
    });

    group.finish();
}

// ============================================================================
// THROUGHPUT BENCHMARKS
// ============================================================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_boundary_throughput");
    group.sample_size(20);

    for size in [100, 500, 1000, 5000] {
        group.bench_with_input(BenchmarkId::new("latin", size), &size, |b, &size| {
            let detector = WordBoundaryDetector::new();
            let chars = generate_latin_characters(size);
            let context = BoundaryContext::new(12.0);

            b.iter(|| detector.detect_word_boundaries(black_box(&chars), black_box(&context)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_word_boundary_small,
    bench_word_boundary_medium,
    bench_word_boundary_large,
    bench_throughput,
);
criterion_main!(benches);
