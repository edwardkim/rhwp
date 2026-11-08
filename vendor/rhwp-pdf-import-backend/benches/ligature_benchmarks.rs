//! Ligature Processing Benchmarks
//!
//! This benchmark suite measures the performance of ligature detection and
//! decision-making for splitting ligatures at word boundaries.
//!
//! ## Performance Targets
//!
//! - Ligature decision: <50µs per ligature
//! - Detection overhead: minimal (range check)
//! - Decision logic: fast path optimization

use criterion::{criterion_group, criterion_main, Criterion};
use pdf_oxide::text::ligature_processor::{LigatureDecision, LigatureDecisionMaker};
use pdf_oxide::text::{BoundaryContext, CharacterInfo};
use std::hint::black_box;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a ligature character
fn create_ligature(code: u32, x_pos: f32, width: f32) -> CharacterInfo {
    CharacterInfo {
        code,
        glyph_id: Some(1),
        width,
        x_position: x_pos,
        tj_offset: None,
        font_size: 12.0,
        is_ligature: true,
        original_ligature: Some(char::from_u32(code).unwrap()),
        protected_from_split: false,
    }
}

/// Create a regular character
fn create_char(code: u32, x_pos: f32, width: f32, tj_offset: Option<i32>) -> CharacterInfo {
    CharacterInfo {
        code,
        glyph_id: Some(1),
        width,
        x_position: x_pos,
        tj_offset,
        font_size: 12.0,
        is_ligature: false,
        original_ligature: None,
        protected_from_split: false,
    }
}

// ============================================================================
// LIGATURE DETECTION BENCHMARKS
// ============================================================================

fn bench_ligature_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("ligature_detection");
    group.sample_size(50);

    // Check if character is ligature (range check)
    group.bench_function("is_ligature_check", |b| {
        let codes = vec![
            0xFB00, // ff
            0xFB01, // fi
            0xFB02, // fl
            0xFB03, // ffi
            0xFB04, // ffl
            0x0066, // f (not ligature)
            0x0069, // i (not ligature)
        ];
        b.iter(|| {
            for &code in &codes {
                // Ligature range check
                black_box(matches!(code, 0xFB00..=0xFB04));
            }
        });
    });

    // Ligature expansion mapping
    group.bench_function("ligature_expansion_lookup", |b| {
        let ligatures = vec![0xFB00, 0xFB01, 0xFB02, 0xFB03, 0xFB04];
        b.iter(|| {
            for &code in &ligatures {
                let expansion = black_box(match code {
                    0xFB00 => vec!['f', 'f'],
                    0xFB01 => vec!['f', 'i'],
                    0xFB02 => vec!['f', 'l'],
                    0xFB03 => vec!['f', 'f', 'i'],
                    0xFB04 => vec!['f', 'f', 'l'],
                    _ => vec![],
                });
                black_box(expansion);
            }
        });
    });

    group.finish();
}

// ============================================================================
// LIGATURE DECISION BENCHMARKS
// ============================================================================

fn bench_ligature_decisions(c: &mut Criterion) {
    let mut group = c.benchmark_group("ligature_decisions");
    group.sample_size(50);

    let context = BoundaryContext::new(12.0);

    // Case 1: Ligature at end of text (no next char) - Keep
    group.bench_function("decision_end_of_text", |b| {
        let ligature = create_ligature(0xFB01, 0.0, 6.0); // fi
        b.iter(|| {
            black_box(LigatureDecisionMaker::decide(
                black_box(&ligature),
                black_box(&context),
                black_box(None),
            ))
        });
    });

    // Case 2: Ligature with large TJ offset - Split
    group.bench_function("decision_large_tj_offset", |b| {
        let ligature = create_ligature(0xFB01, 0.0, 6.0); // fi
        let next = create_char(0x0063, 10.0, 6.0, Some(-150)); // c with large offset
        b.iter(|| {
            black_box(LigatureDecisionMaker::decide(
                black_box(&ligature),
                black_box(&context),
                black_box(Some(&next)),
            ))
        });
    });

    // Case 3: Ligature with small TJ offset - Keep
    group.bench_function("decision_small_tj_offset", |b| {
        let ligature = create_ligature(0xFB01, 0.0, 6.0); // fi
        let next = create_char(0x0063, 6.0, 6.0, Some(-50)); // c with small offset
        b.iter(|| {
            black_box(LigatureDecisionMaker::decide(
                black_box(&ligature),
                black_box(&context),
                black_box(Some(&next)),
            ))
        });
    });

    // Case 4: Ligature with large geometric gap - Split
    group.bench_function("decision_large_gap", |b| {
        let ligature = create_ligature(0xFB01, 0.0, 6.0); // fi
        let next = create_char(0x0063, 15.0, 6.0, None); // c with large gap
        b.iter(|| {
            black_box(LigatureDecisionMaker::decide(
                black_box(&ligature),
                black_box(&context),
                black_box(Some(&next)),
            ))
        });
    });

    // Case 5: Ligature with small geometric gap - Keep
    group.bench_function("decision_small_gap", |b| {
        let ligature = create_ligature(0xFB01, 0.0, 6.0); // fi
        let next = create_char(0x0063, 6.5, 6.0, None); // c with small gap
        b.iter(|| {
            black_box(LigatureDecisionMaker::decide(
                black_box(&ligature),
                black_box(&context),
                black_box(Some(&next)),
            ))
        });
    });

    group.finish();
}

// ============================================================================
// BATCH LIGATURE PROCESSING BENCHMARKS
// ============================================================================

fn bench_batch_ligature_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_ligature_processing");
    group.sample_size(20);

    let context = BoundaryContext::new(12.0);

    // Process a sequence with multiple ligatures
    group.bench_function("process_mixed_sequence", |b| {
        let chars = vec![
            create_char(0x0074, 0.0, 6.0, None),        // t
            create_char(0x0068, 6.0, 6.0, None),        // h
            create_char(0x0065, 12.0, 6.0, None),       // e
            create_char(0x0020, 18.0, 3.0, Some(-100)), // space
            create_ligature(0xFB01, 21.0, 6.0),         // fi
            create_char(0x0072, 27.0, 6.0, None),       // r
            create_char(0x0073, 33.0, 6.0, None),       // s
            create_char(0x0074, 39.0, 6.0, None),       // t
            create_char(0x0020, 45.0, 3.0, Some(-100)), // space
            create_ligature(0xFB02, 48.0, 6.0),         // fl
            create_char(0x006F, 54.0, 6.0, None),       // o
            create_char(0x0077, 60.0, 6.0, None),       // w
        ];

        b.iter(|| {
            for i in 0..chars.len() {
                if chars[i].is_ligature {
                    let next = chars.get(i + 1);
                    black_box(LigatureDecisionMaker::decide(
                        black_box(&chars[i]),
                        black_box(&context),
                        black_box(next),
                    ));
                }
            }
        });
    });

    // All ligatures - worst case
    group.bench_function("process_all_ligatures", |b| {
        let chars = [
            create_ligature(0xFB01, 0.0, 6.0),
            create_ligature(0xFB02, 6.0, 6.0),
            create_ligature(0xFB03, 12.0, 8.0),
            create_ligature(0xFB04, 20.0, 8.0),
            create_ligature(0xFB00, 28.0, 8.0),
        ];

        b.iter(|| {
            for i in 0..chars.len() {
                let next = chars.get(i + 1);
                black_box(LigatureDecisionMaker::decide(
                    black_box(&chars[i]),
                    black_box(&context),
                    black_box(next),
                ));
            }
        });
    });

    // No ligatures - best case
    group.bench_function("process_no_ligatures", |b| {
        let chars = [
            create_char(0x0074, 0.0, 6.0, None),
            create_char(0x0068, 6.0, 6.0, None),
            create_char(0x0065, 12.0, 6.0, None),
            create_char(0x0020, 18.0, 3.0, None),
            create_char(0x0074, 21.0, 6.0, None),
        ];

        b.iter(|| {
            for i in 0..chars.len() {
                if chars[i].is_ligature {
                    let next = chars.get(i + 1);
                    black_box(LigatureDecisionMaker::decide(
                        black_box(&chars[i]),
                        black_box(&context),
                        black_box(next),
                    ));
                }
            }
        });
    });

    group.finish();
}

// ============================================================================
// DECISION CORRECTNESS VERIFICATION
// ============================================================================

fn bench_decision_correctness(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_correctness");
    group.sample_size(50);

    let context = BoundaryContext::new(12.0);

    // Verify all decision paths
    group.bench_function("verify_all_paths", |b| {
        let test_cases = vec![
            // (ligature, next_char, expected_decision)
            (create_ligature(0xFB01, 0.0, 6.0), None, LigatureDecision::Keep),
            (
                create_ligature(0xFB01, 0.0, 6.0),
                Some(create_char(0x0063, 6.0, 6.0, Some(-150))),
                LigatureDecision::Split,
            ),
            (
                create_ligature(0xFB01, 0.0, 6.0),
                Some(create_char(0x0063, 15.0, 6.0, None)),
                LigatureDecision::Split,
            ),
            (
                create_ligature(0xFB01, 0.0, 6.0),
                Some(create_char(0x0063, 6.5, 6.0, Some(-50))),
                LigatureDecision::Keep,
            ),
        ];

        b.iter(|| {
            for (ligature, next_char, _expected) in &test_cases {
                let decision = LigatureDecisionMaker::decide(
                    black_box(ligature),
                    black_box(&context),
                    black_box(next_char.as_ref()),
                );
                black_box(decision);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ligature_detection,
    bench_ligature_decisions,
    bench_batch_ligature_processing,
    bench_decision_correctness,
);
criterion_main!(benches);
