#!/usr/bin/env cargo

//! Benchmark all PDFs in test_datasets/pdfs/
//!
//! This binary tests text extraction on all 356 PDFs in the dataset
//! and reports:
//! - Success/failure count
//! - Average extraction time
//! - Text length statistics
//! - Any errors encountered

use pdf_oxide::PdfDocument;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfResult {
    filename: String,
    category: String,
    success: bool,
    duration_ms: f64,
    text_length: usize,
    span_count: usize,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkSummary {
    total_pdfs: usize,
    successful: usize,
    failed: usize,
    total_duration_ms: f64,
    avg_duration_ms: f64,
    total_text_length: usize,
    avg_text_length: f64,
    results: Vec<PdfResult>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("PDF Library - Rust Benchmark (All 356 PDFs)");
    println!("{}", "=".repeat(80));
    println!();

    // Find all PDFs in test_datasets/pdfs/
    let base_dir = PathBuf::from("test_datasets/pdfs");
    let mut pdf_paths: Vec<(String, PathBuf)> = Vec::new();

    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let category_path = entry.path();

        if category_path.is_dir() {
            let category = category_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Read all PDFs in this category
            for pdf_entry in fs::read_dir(&category_path)? {
                let pdf_entry = pdf_entry?;
                let pdf_path = pdf_entry.path();

                if pdf_path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    pdf_paths.push((category.clone(), pdf_path));
                }
            }
        }
    }

    pdf_paths.sort_by(|a, b| a.1.cmp(&b.1));

    println!("Found {} PDFs across {} categories", pdf_paths.len(), {
        let mut categories: Vec<String> = pdf_paths.iter().map(|(cat, _)| cat.clone()).collect();
        categories.sort();
        categories.dedup();
        categories.len()
    });
    println!();

    let mut results: Vec<PdfResult> = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    let mut total_duration_ms = 0.0;
    let mut total_text_length = 0;

    let start_time = Instant::now();

    for (idx, (category, pdf_path)) in pdf_paths.iter().enumerate() {
        let filename = pdf_path.file_name().unwrap().to_string_lossy().to_string();

        print!("[{:3}/{:3}] {}/{} ... ", idx + 1, pdf_paths.len(), category, filename);

        let pdf_start = Instant::now();

        match process_pdf(pdf_path, category) {
            Ok((text_length, span_count)) => {
                let duration_ms = pdf_start.elapsed().as_secs_f64() * 1000.0;
                total_duration_ms += duration_ms;
                total_text_length += text_length;
                successful += 1;

                println!("✅ ({:.1}ms, {} chars, {} spans)", duration_ms, text_length, span_count);

                results.push(PdfResult {
                    filename: filename.clone(),
                    category: category.clone(),
                    success: true,
                    duration_ms,
                    text_length,
                    span_count,
                    error: None,
                });
            },
            Err(e) => {
                let duration_ms = pdf_start.elapsed().as_secs_f64() * 1000.0;
                total_duration_ms += duration_ms;
                failed += 1;

                println!("❌ Error: {}", e);

                results.push(PdfResult {
                    filename: filename.clone(),
                    category: category.clone(),
                    success: false,
                    duration_ms,
                    text_length: 0,
                    span_count: 0,
                    error: Some(e.to_string()),
                });
            },
        }
    }

    let total_elapsed = start_time.elapsed().as_secs_f64();

    println!();
    println!("{}", "=".repeat(80));
    println!("BENCHMARK SUMMARY");
    println!("{}", "=".repeat(80));
    println!();
    println!("Total PDFs:       {}", pdf_paths.len());
    println!(
        "Successful:       {} ({:.1}%)",
        successful,
        (successful as f64 / pdf_paths.len() as f64) * 100.0
    );
    println!(
        "Failed:           {} ({:.1}%)",
        failed,
        (failed as f64 / pdf_paths.len() as f64) * 100.0
    );
    println!();
    println!("Total time:       {:.2}s", total_elapsed);
    println!("Total PDF time:   {:.2}s", total_duration_ms / 1000.0);
    println!("Avg time/PDF:     {:.1}ms", total_duration_ms / pdf_paths.len() as f64);
    println!();
    println!("Total text:       {} characters", total_text_length);
    println!(
        "Avg text/PDF:     {:.0} characters",
        total_text_length as f64 / successful as f64
    );
    println!();

    // Save detailed results to JSON
    let summary = BenchmarkSummary {
        total_pdfs: pdf_paths.len(),
        successful,
        failed,
        total_duration_ms,
        avg_duration_ms: total_duration_ms / pdf_paths.len() as f64,
        total_text_length,
        avg_text_length: total_text_length as f64 / successful as f64,
        results,
    };

    let output_dir = PathBuf::from("benchmark_results");
    fs::create_dir_all(&output_dir)?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let json_path = output_dir.join(format!("rust_benchmark_{}.json", timestamp));

    let json = serde_json::to_string_pretty(&summary)?;
    fs::write(&json_path, json)?;

    println!("Detailed results saved to: {}", json_path.display());
    println!();

    // Show failed PDFs if any
    if failed > 0 {
        println!("{}", "=".repeat(80));
        println!("FAILED PDFs");
        println!("{}", "=".repeat(80));
        println!();

        for result in summary.results.iter().filter(|r| !r.success) {
            println!("  {}/{}", result.category, result.filename);
            if let Some(error) = &result.error {
                println!("    Error: {}", error);
            }
        }
        println!();
    }

    // Show slowest PDFs
    let mut sorted_results = summary.results.clone();
    sorted_results.sort_by(|a, b| b.duration_ms.total_cmp(&a.duration_ms));

    println!("{}", "=".repeat(80));
    println!("TOP 10 SLOWEST PDFs");
    println!("{}", "=".repeat(80));
    println!();

    for (idx, result) in sorted_results.iter().take(10).enumerate() {
        println!(
            "  {}. {}/{} - {:.1}ms ({} chars)",
            idx + 1,
            result.category,
            result.filename,
            result.duration_ms,
            result.text_length
        );
    }
    println!();

    Ok(())
}

fn process_pdf(path: &Path, category: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let doc = PdfDocument::open(path.to_str().unwrap())?;

    // Convert to markdown with default options
    use pdf_oxide::converters::ConversionOptions;
    let options = ConversionOptions::default();
    let markdown = doc.to_markdown_all(&options)?;

    // Save markdown to file
    let filename = path.file_stem().unwrap().to_string_lossy().to_string();
    let output_dir = PathBuf::from("benchmark_results/markdown_output").join(category);
    fs::create_dir_all(&output_dir)?;

    let md_path = output_dir.join(format!("{}.md", filename));
    fs::write(&md_path, &markdown)?;

    // Count spans for statistics
    let mut total_spans = 0;
    let page_count = doc.page_count()?;
    for page_num in 0..page_count {
        let spans = doc.extract_spans(page_num)?;
        total_spans += spans.len();
    }

    Ok((markdown.len(), total_spans))
}
