//! Script Detection Benchmarks
//!
//! This benchmark suite measures the performance of script detection functions
//! for CJK, RTL, and complex scripts.
//!
//! ## Performance Targets
//!
//! - Script detection: <20µs per check
//! - O(1) performance with fast paths
//! - Range checks should be extremely fast

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::text::script_detector::detect_cjk_script;
use pdf_oxide::text::{detect_complex_script, detect_rtl_script, is_complex_script, is_rtl_text};
use std::hint::black_box;

// ============================================================================
// CJK SCRIPT DETECTION BENCHMARKS
// ============================================================================

fn bench_cjk_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("cjk_detection");
    group.sample_size(50);

    // Fast path: Common Han characters (U+4E00-U+9FFF)
    group.bench_function("han_fast_path", |b| {
        let chars = vec![0x4E00, 0x4E2D, 0x6587, 0x5927, 0x5B66];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    // Hiragana detection
    group.bench_function("hiragana", |b| {
        let chars = vec![0x3042, 0x3044, 0x3046, 0x3048, 0x304A];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    // Katakana detection
    group.bench_function("katakana", |b| {
        let chars = vec![0x30A2, 0x30A4, 0x30A6, 0x30A8, 0x30AA];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    // Hangul detection
    group.bench_function("hangul", |b| {
        let chars = vec![0xAC00, 0xAC01, 0xAC04, 0xAC08, 0xAC10];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    // Non-CJK characters (Latin)
    group.bench_function("non_cjk_latin", |b| {
        let chars = vec![0x0041, 0x0042, 0x0043, 0x0044, 0x0045];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    // Mixed CJK detection
    group.bench_function("mixed_cjk", |b| {
        let chars = vec![
            0x4E00, // Han
            0x3042, // Hiragana
            0x30A2, // Katakana
            0xAC00, // Hangul
            0x4E2D, // Han
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// RTL SCRIPT DETECTION BENCHMARKS
// ============================================================================

fn bench_rtl_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtl_detection");
    group.sample_size(50);

    // Arabic main range (fast path)
    group.bench_function("arabic_fast_path", |b| {
        let chars = vec![0x0627, 0x0644, 0x0633, 0x0645, 0x0639];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_rtl_script(black_box(code)));
            }
        });
    });

    // Hebrew detection
    group.bench_function("hebrew", |b| {
        let chars = vec![0x05D0, 0x05D1, 0x05D2, 0x05D3, 0x05D4];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_rtl_script(black_box(code)));
            }
        });
    });

    // Arabic Supplement
    group.bench_function("arabic_supplement", |b| {
        let chars = vec![0x0750, 0x0751, 0x0752, 0x0753, 0x0754];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_rtl_script(black_box(code)));
            }
        });
    });

    // Arabic Extended-A
    group.bench_function("arabic_extended_a", |b| {
        let chars = vec![0x08A0, 0x08A1, 0x08A2, 0x08A3, 0x08A4];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_rtl_script(black_box(code)));
            }
        });
    });

    // Non-RTL characters
    group.bench_function("non_rtl", |b| {
        let chars = vec![0x0041, 0x0042, 0x0043, 0x0044, 0x0045];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_rtl_script(black_box(code)));
            }
        });
    });

    // is_rtl_text convenience function
    group.bench_function("is_rtl_text", |b| {
        let chars = vec![0x0627, 0x05D0, 0x0750, 0x0041, 0x08A0];
        b.iter(|| {
            for &code in &chars {
                black_box(is_rtl_text(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// COMPLEX SCRIPT DETECTION BENCHMARKS
// ============================================================================

fn bench_complex_script_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_script_detection");
    group.sample_size(50);

    // Devanagari (fast path)
    group.bench_function("devanagari_fast_path", |b| {
        let chars = vec![0x0928, 0x092E, 0x0938, 0x094D, 0x0924];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // Bengali
    group.bench_function("bengali", |b| {
        let chars = vec![0x0995, 0x0996, 0x0997, 0x0998, 0x0999];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // Tamil
    group.bench_function("tamil", |b| {
        let chars = vec![0x0B95, 0x0B99, 0x0B9A, 0x0B9C, 0x0B9E];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // Thai
    group.bench_function("thai", |b| {
        let chars = vec![0x0E01, 0x0E02, 0x0E03, 0x0E04, 0x0E05];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // Khmer
    group.bench_function("khmer", |b| {
        let chars = vec![0x1780, 0x1781, 0x1782, 0x1783, 0x1784];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // Non-complex scripts
    group.bench_function("non_complex", |b| {
        let chars = vec![0x0041, 0x0042, 0x0043, 0x0044, 0x0045];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    // is_complex_script convenience function
    group.bench_function("is_complex_script", |b| {
        let chars = vec![0x0928, 0x0995, 0x0B95, 0x0E01, 0x1780];
        b.iter(|| {
            for &code in &chars {
                black_box(is_complex_script(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// MIXED SCRIPT DETECTION BENCHMARKS
// ============================================================================

fn bench_mixed_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_detection");
    group.sample_size(50);

    // Realistic mixed script text
    group.bench_function("mixed_scripts", |b| {
        let chars = vec![
            0x0041, // Latin A
            0x4E00, // CJK Han
            0x0020, // Space
            0x0627, // Arabic
            0x0020, // Space
            0x0928, // Devanagari
            0x0020, // Space
            0x3042, // Hiragana
            0x0020, // Space
            0x05D0, // Hebrew
        ];
        b.iter(|| {
            for &code in &chars {
                black_box(detect_cjk_script(black_box(code)));
                black_box(detect_rtl_script(black_box(code)));
                black_box(detect_complex_script(black_box(code)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// BATCH DETECTION BENCHMARKS
// ============================================================================

fn bench_batch_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_detection");
    group.sample_size(20);

    for size in [10, 100, 1000] {
        // CJK batch
        group.bench_with_input(BenchmarkId::new("cjk_batch", size), &size, |b, &size| {
            let chars: Vec<u32> = (0..size).map(|i| 0x4E00 + (i % 100) as u32).collect();
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_cjk_script(black_box(code)));
                }
            });
        });

        // RTL batch
        group.bench_with_input(BenchmarkId::new("rtl_batch", size), &size, |b, &size| {
            let chars: Vec<u32> = (0..size).map(|i| 0x0627 + (i % 50) as u32).collect();
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_rtl_script(black_box(code)));
                }
            });
        });

        // Complex script batch
        group.bench_with_input(BenchmarkId::new("complex_batch", size), &size, |b, &size| {
            let chars: Vec<u32> = (0..size).map(|i| 0x0928 + (i % 50) as u32).collect();
            b.iter(|| {
                for &code in &chars {
                    black_box(detect_complex_script(black_box(code)));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cjk_detection,
    bench_rtl_detection,
    bench_complex_script_detection,
    bench_mixed_detection,
    bench_batch_detection,
);
criterion_main!(benches);
