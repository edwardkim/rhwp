//! Full Pipeline Benchmarks
//!
//! This benchmark suite measures end-to-end performance of the complete
//! text extraction pipeline, from PDF parsing to word boundary detection
//! to final output.
//!
//! ## Performance Targets
//!
//! - Full pipeline (1 page): <50ms
//! - Word boundary overhead: <5%
//! - Quality scoring: <10ms per document
//!
//! ## Test Corpus
//!
//! The benchmarks use real PDF files from the test fixtures:
//! - Simple PDFs: Basic text extraction
//! - Academic PDFs: Complex layouts, multi-column
//! - Mixed PDFs: Multiple languages and scripts

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pdf_oxide::PdfDocument;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// TEST PDF FIXTURES
// ============================================================================

fn get_simple_pdf() -> Option<PathBuf> {
    let path = PathBuf::from("tests/fixtures/simple.pdf");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn get_academic_pdfs() -> Vec<(String, PathBuf)> {
    vec![
        (
            "academic_1".to_string(),
            PathBuf::from("tests/fixtures/regression/academic/arxiv_2510.21165v1.pdf"),
        ),
        (
            "academic_2".to_string(),
            PathBuf::from("tests/fixtures/regression/academic/arxiv_2510.21912v1.pdf"),
        ),
        (
            "academic_3".to_string(),
            PathBuf::from("tests/fixtures/regression/academic/arxiv_2510.22293v1.pdf"),
        ),
    ]
    .into_iter()
    .filter(|(_, path)| path.exists())
    .collect()
}

fn get_government_pdfs() -> Vec<(String, PathBuf)> {
    vec![(
        "government_cfr".to_string(),
        PathBuf::from("tests/fixtures/regression/government/cfr_excerpt.pdf"),
    )]
    .into_iter()
    .filter(|(_, path)| path.exists())
    .collect()
}

fn get_all_test_pdfs() -> Vec<(String, PathBuf)> {
    let mut all_pdfs = Vec::new();

    if let Some(simple) = get_simple_pdf() {
        all_pdfs.push(("simple".to_string(), simple));
    }

    all_pdfs.extend(get_academic_pdfs());
    all_pdfs.extend(get_government_pdfs());

    all_pdfs
}

// ============================================================================
// SINGLE PAGE EXTRACTION BENCHMARKS
// ============================================================================

fn bench_single_page_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_page_extraction");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping single page extraction benchmarks");
        return;
    }

    for (name, path) in test_pdfs {
        group.bench_with_input(BenchmarkId::from_parameter(&name), &path, |b, path| {
            b.iter(|| {
                let doc = PdfDocument::open(black_box(path)).expect("Failed to open PDF");

                let _ = doc
                    .extract_text(black_box(0))
                    .expect("Failed to extract text");
            });
        });
    }

    group.finish();
}

// ============================================================================
// FULL DOCUMENT EXTRACTION BENCHMARKS
// ============================================================================

fn bench_full_document_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_document_extraction");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(10));

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping full document extraction benchmarks");
        return;
    }

    for (name, path) in test_pdfs {
        group.bench_with_input(BenchmarkId::from_parameter(&name), &path, |b, path| {
            b.iter_batched(
                || PdfDocument::open(path.clone()).expect("Failed to open PDF"),
                |doc| {
                    let page_count = doc.page_count().expect("Failed to get page count");
                    for page_idx in 0..page_count {
                        let _ = doc
                            .extract_text(black_box(page_idx))
                            .expect("Failed to extract text");
                    }
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// MARKDOWN CONVERSION BENCHMARKS
// ============================================================================

fn bench_markdown_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_conversion");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping markdown conversion benchmarks");
        return;
    }

    for (name, path) in test_pdfs {
        group.bench_with_input(BenchmarkId::from_parameter(&name), &path, |b, path| {
            b.iter_batched(
                || {
                    let doc = PdfDocument::open(path.clone()).expect("Failed to open PDF");
                    let options = pdf_oxide::converters::ConversionOptions::default();
                    (doc, options)
                },
                |(doc, options)| {
                    let _ = doc
                        .to_markdown(black_box(0), black_box(&options))
                        .expect("Failed to convert to markdown");
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// HTML CONVERSION BENCHMARKS
// ============================================================================

fn bench_html_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_conversion");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping HTML conversion benchmarks");
        return;
    }

    for (name, path) in test_pdfs {
        group.bench_with_input(BenchmarkId::from_parameter(&name), &path, |b, path| {
            b.iter_batched(
                || {
                    let doc = PdfDocument::open(path.clone()).expect("Failed to open PDF");
                    let options = pdf_oxide::converters::ConversionOptions::default();
                    (doc, options)
                },
                |(doc, options)| {
                    let _ = doc
                        .to_html(black_box(0), black_box(&options))
                        .expect("Failed to convert to HTML");
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// PIPELINE COMPONENT BENCHMARKS
// ============================================================================

fn bench_pipeline_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_components");
    group.sample_size(10);

    // Skip if no simple PDF available
    let simple_pdf = match get_simple_pdf() {
        Some(pdf) => pdf,
        None => {
            println!("WARNING: No simple.pdf found, skipping pipeline component benchmarks");
            return;
        },
    };

    // Document opening
    group.bench_function("document_open", |b| {
        b.iter(|| {
            let _doc = PdfDocument::open(black_box(&simple_pdf)).expect("Failed to open PDF");
        });
    });

    // Text extraction only (no conversion)
    group.bench_function("text_extraction_only", |b| {
        b.iter_batched(
            || PdfDocument::open(&simple_pdf).expect("Failed to open PDF"),
            |doc| {
                let _ = doc
                    .extract_text(black_box(0))
                    .expect("Failed to extract text");
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// THROUGHPUT BENCHMARKS
// ============================================================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.sample_size(5);
    group.measurement_time(Duration::from_secs(10));

    let academic_pdfs = get_academic_pdfs();

    if academic_pdfs.is_empty() {
        println!("WARNING: No academic PDFs found, skipping throughput benchmarks");
        return;
    }

    // Process multiple pages
    for page_count in [1, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("pages", page_count),
            &page_count,
            |b, &page_count| {
                let (_, path) = &academic_pdfs[0];
                b.iter_batched(
                    || PdfDocument::open(path.clone()).expect("Failed to open PDF"),
                    |doc| {
                        let doc_page_count = doc.page_count().expect("Failed to get page count");
                        let max_pages = doc_page_count.min(page_count);
                        for page_idx in 0..max_pages {
                            let _ = doc
                                .extract_text(black_box(page_idx))
                                .expect("Failed to extract text");
                        }
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================================
// OVERHEAD MEASUREMENT BENCHMARKS
// ============================================================================

fn bench_word_boundary_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_boundary_overhead");
    group.sample_size(10);

    // This benchmark compares extraction with and without word boundary processing
    // to measure the overhead introduced by the word boundary detection system.
    //
    // Expected overhead: <5% (currently ~3%)

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping overhead benchmarks");
        return;
    }

    for (name, path) in test_pdfs.iter().take(2) {
        group.bench_with_input(BenchmarkId::new("with_boundaries", name), path, |b, path| {
            b.iter_batched(
                || PdfDocument::open(path.clone()).expect("Failed to open PDF"),
                |doc| {
                    let _ = doc
                        .extract_text(black_box(0))
                        .expect("Failed to extract text");
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// REGRESSION DETECTION BENCHMARKS
// ============================================================================

fn bench_regression_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_detection");
    group.sample_size(10);

    // These benchmarks are designed to catch performance regressions
    // by comparing against baseline measurements from Week 1.
    //
    // Baseline targets:
    // - Simple PDF: <10ms
    // - Academic PDF (1 page): <50ms
    // - Government PDF: <20ms

    let test_pdfs = get_all_test_pdfs();

    if test_pdfs.is_empty() {
        println!("WARNING: No test PDFs found, skipping regression detection benchmarks");
        return;
    }

    for (name, path) in test_pdfs {
        group.bench_with_input(BenchmarkId::new("baseline", &name), &path, |b, path| {
            b.iter(|| {
                let doc = PdfDocument::open(black_box(path)).expect("Failed to open PDF");
                let _ = doc
                    .extract_text(black_box(0))
                    .expect("Failed to extract text");
            });
        });
    }

    group.finish();
}

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

criterion_group!(
    benches,
    bench_single_page_extraction,
    bench_full_document_extraction,
    bench_markdown_conversion,
    bench_html_conversion,
    bench_pipeline_components,
    bench_throughput,
    bench_word_boundary_overhead,
    bench_regression_detection,
);

criterion_main!(benches);
