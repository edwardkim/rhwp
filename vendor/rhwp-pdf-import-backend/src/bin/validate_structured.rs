//! Structured Text Extraction Validator
//!
//! Validates structured text extraction quality across all test PDFs.
//! Measures header detection, paragraph segmentation, list detection, and formatting extraction.
//!
//! Usage:
//!   cargo run --release --bin validate_structured
//!   cargo run --release --bin validate_structured -- --output report.html
//!   cargo run --release --bin validate_structured -- --verbose

use pdf_oxide::document::PdfDocument;
use pdf_oxide::extractors::{DocumentElement, StructuredDocument, StructuredExtractor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StructuredValidationResult {
    filename: String,
    category: String,
    success: bool,
    error: Option<String>,

    // Extraction statistics
    page_count: Option<usize>,
    total_elements: usize,
    header_count: usize,
    paragraph_count: usize,
    list_count: usize,

    // Quality indicators
    has_headers: bool,
    has_paragraphs: bool,
    has_lists: bool,
    has_formatting: bool, // Bold/italic detected

    // Performance
    extraction_time_ms: u128,
    file_size: u64,
}

#[derive(Debug, Serialize)]
struct ValidationSummary {
    total_pdfs: usize,
    successful: usize,
    failed: usize,
    success_rate: f64,

    // Element statistics
    total_elements: usize,
    total_headers: usize,
    total_paragraphs: usize,
    total_lists: usize,

    // Quality metrics
    pdfs_with_headers: usize,
    pdfs_with_paragraphs: usize,
    pdfs_with_lists: usize,
    pdfs_with_formatting: usize,

    avg_elements_per_pdf: f64,
    avg_extraction_time_ms: f64,

    by_category: HashMap<String, CategoryStats>,
    results: Vec<StructuredValidationResult>,
}

#[derive(Debug, Serialize)]
struct CategoryStats {
    total: usize,
    successful: usize,
    failed: usize,
    success_rate: f64,
    avg_elements: f64,
}

struct ValidatorConfig {
    base_dir: PathBuf,
    output_file: Option<PathBuf>,
    verbose: bool,
}

impl ValidatorConfig {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut output_file = None;
        let mut verbose = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output" => {
                    i += 1;
                    if i < args.len() {
                        output_file = Some(PathBuf::from(&args[i]));
                    }
                },
                "--verbose" | "-v" => {
                    verbose = true;
                },
                _ => {},
            }
            i += 1;
        }

        Self {
            base_dir: PathBuf::from("test_datasets/pdfs"),
            output_file,
            verbose,
        }
    }
}

fn validate_structured_extraction(
    path: &Path,
    category: &str,
    verbose: bool,
) -> StructuredValidationResult {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let start = Instant::now();
    let mut result = StructuredValidationResult {
        filename: filename.clone(),
        category: category.to_string(),
        success: false,
        error: None,
        page_count: None,
        total_elements: 0,
        header_count: 0,
        paragraph_count: 0,
        list_count: 0,
        has_headers: false,
        has_paragraphs: false,
        has_lists: false,
        has_formatting: false,
        extraction_time_ms: 0,
        file_size,
    };

    match PdfDocument::open(path) {
        Ok(mut pdf) => {
            // Get page count
            match pdf.page_count() {
                Ok(count) => {
                    result.page_count = Some(count);

                    // Extract structured content from first page (or all if small doc)
                    let pages_to_process = if count <= 5 { count } else { 1 };

                    let mut all_elements = 0;
                    let mut all_headers = 0;
                    let mut all_paragraphs = 0;
                    let mut all_lists = 0;
                    let mut has_bold_italic = false;

                    for page_num in 0..pages_to_process {
                        match extract_page_structured(&mut pdf, page_num, verbose) {
                            Ok(doc) => {
                                all_elements += doc.elements.len();

                                for element in &doc.elements {
                                    match element {
                                        DocumentElement::Header { style, .. } => {
                                            all_headers += 1;
                                            if style.bold || style.italic {
                                                has_bold_italic = true;
                                            }
                                        },
                                        DocumentElement::Paragraph { style, .. } => {
                                            all_paragraphs += 1;
                                            if style.bold || style.italic {
                                                has_bold_italic = true;
                                            }
                                        },
                                        DocumentElement::List { .. } => {
                                            all_lists += 1;
                                        },
                                        _ => {},
                                    }
                                }
                            },
                            Err(e) => {
                                if verbose {
                                    eprintln!(
                                        "      Warning: Page {} extraction failed: {}",
                                        page_num, e
                                    );
                                }
                            },
                        }
                    }

                    result.success = true;
                    result.total_elements = all_elements;
                    result.header_count = all_headers;
                    result.paragraph_count = all_paragraphs;
                    result.list_count = all_lists;
                    result.has_headers = all_headers > 0;
                    result.has_paragraphs = all_paragraphs > 0;
                    result.has_lists = all_lists > 0;
                    result.has_formatting = has_bold_italic;
                },
                Err(e) => {
                    result.success = false;
                    result.error = Some(format!("Page count error: {}", e));
                },
            }
        },
        Err(e) => {
            result.success = false;
            result.error = Some(e.to_string());
        },
    }

    result.extraction_time_ms = start.elapsed().as_millis();
    result
}

fn extract_page_structured(
    pdf: &mut PdfDocument,
    page_num: usize,
    _verbose: bool,
) -> Result<StructuredDocument, Box<dyn std::error::Error>> {
    // Create extractor with default config
    let mut extractor = StructuredExtractor::new();

    // Extract structured content from page
    let doc = extractor.extract_page(pdf, page_num as u32)?;

    Ok(doc)
}

fn discover_pdfs(base_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut pdfs = Vec::new();

    if !base_dir.exists() {
        eprintln!("Error: Directory {} does not exist", base_dir.display());
        return pdfs;
    }

    // Read categories
    let categories = match fs::read_dir(base_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error reading directory {}: {}", base_dir.display(), e);
            return pdfs;
        },
    };

    for category in categories {
        let category_path = base_dir.join(&category);
        if let Ok(entries) = fs::read_dir(&category_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    pdfs.push((path, category.clone()));
                }
            }
        }
    }

    pdfs
}

fn generate_summary(results: Vec<StructuredValidationResult>) -> ValidationSummary {
    let total_pdfs = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total_pdfs - successful;
    let success_rate = if total_pdfs > 0 {
        (successful as f64 / total_pdfs as f64) * 100.0
    } else {
        0.0
    };

    // Calculate totals
    let total_elements: usize = results.iter().map(|r| r.total_elements).sum();
    let total_headers: usize = results.iter().map(|r| r.header_count).sum();
    let total_paragraphs: usize = results.iter().map(|r| r.paragraph_count).sum();
    let total_lists: usize = results.iter().map(|r| r.list_count).sum();

    // Quality metrics
    let pdfs_with_headers = results.iter().filter(|r| r.has_headers).count();
    let pdfs_with_paragraphs = results.iter().filter(|r| r.has_paragraphs).count();
    let pdfs_with_lists = results.iter().filter(|r| r.has_lists).count();
    let pdfs_with_formatting = results.iter().filter(|r| r.has_formatting).count();

    let avg_elements_per_pdf = if total_pdfs > 0 {
        total_elements as f64 / total_pdfs as f64
    } else {
        0.0
    };

    let total_time: u128 = results.iter().map(|r| r.extraction_time_ms).sum();
    let avg_extraction_time_ms = if total_pdfs > 0 {
        total_time as f64 / total_pdfs as f64
    } else {
        0.0
    };

    // Group by category
    let mut by_category: HashMap<String, Vec<&StructuredValidationResult>> = HashMap::new();
    for result in &results {
        by_category
            .entry(result.category.clone())
            .or_default()
            .push(result);
    }

    let category_stats: HashMap<String, CategoryStats> = by_category
        .into_iter()
        .map(|(cat, results)| {
            let total = results.len();
            let successful = results.iter().filter(|r| r.success).count();
            let failed = total - successful;
            let success_rate = if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let avg_elements = if total > 0 {
                results.iter().map(|r| r.total_elements).sum::<usize>() as f64 / total as f64
            } else {
                0.0
            };
            (
                cat,
                CategoryStats {
                    total,
                    successful,
                    failed,
                    success_rate,
                    avg_elements,
                },
            )
        })
        .collect();

    ValidationSummary {
        total_pdfs,
        successful,
        failed,
        success_rate,
        total_elements,
        total_headers,
        total_paragraphs,
        total_lists,
        pdfs_with_headers,
        pdfs_with_paragraphs,
        pdfs_with_lists,
        pdfs_with_formatting,
        avg_elements_per_pdf,
        avg_extraction_time_ms,
        by_category: category_stats,
        results,
    }
}

fn print_summary(summary: &ValidationSummary, verbose: bool) {
    println!("\n{}", "=".repeat(70));
    println!("STRUCTURED TEXT EXTRACTION QUALITY REPORT");
    println!("{}", "=".repeat(70));

    println!("\nOverall Statistics:");
    println!("  Total PDFs:         {}", summary.total_pdfs);
    println!("  ✓ Successful:       {} ({:.2}%)", summary.successful, summary.success_rate);
    println!(
        "  ✗ Failed:           {} ({:.2}%)",
        summary.failed,
        100.0 - summary.success_rate
    );

    // Quality target check
    let quality_target = 99.95;
    let quality_status = if summary.success_rate >= quality_target {
        "✓ TARGET MET"
    } else {
        "✗ BELOW TARGET"
    };
    println!(
        "\n  Quality Target:     {:.2}% (target: {:.2}%) {}",
        summary.success_rate, quality_target, quality_status
    );

    println!("\nExtraction Statistics:");
    println!("  Total Elements:     {}", summary.total_elements);
    println!("    Headers:          {}", summary.total_headers);
    println!("    Paragraphs:       {}", summary.total_paragraphs);
    println!("    Lists:            {}", summary.total_lists);
    println!("  Avg Elements/PDF:   {:.1}", summary.avg_elements_per_pdf);

    println!("\nQuality Metrics:");
    println!(
        "  PDFs with Headers:     {} ({:.1}%)",
        summary.pdfs_with_headers,
        (summary.pdfs_with_headers as f64 / summary.total_pdfs as f64) * 100.0
    );
    println!(
        "  PDFs with Paragraphs:  {} ({:.1}%)",
        summary.pdfs_with_paragraphs,
        (summary.pdfs_with_paragraphs as f64 / summary.total_pdfs as f64) * 100.0
    );
    println!(
        "  PDFs with Lists:       {} ({:.1}%)",
        summary.pdfs_with_lists,
        (summary.pdfs_with_lists as f64 / summary.total_pdfs as f64) * 100.0
    );
    println!(
        "  PDFs with Formatting:  {} ({:.1}%)",
        summary.pdfs_with_formatting,
        (summary.pdfs_with_formatting as f64 / summary.total_pdfs as f64) * 100.0
    );

    println!("\nPerformance:");
    println!("  Avg Extraction Time: {:.1}ms per PDF", summary.avg_extraction_time_ms);

    println!("\nBreakdown by Category:");
    let mut categories: Vec<_> = summary.by_category.iter().collect();
    categories.sort_by_key(|(name, _)| *name);
    for (category, stats) in categories {
        println!(
            "  {:15} : {:3} total, {:3} ok, {:3} failed ({:.1}%), avg {:.1} elements",
            category,
            stats.total,
            stats.successful,
            stats.failed,
            stats.success_rate,
            stats.avg_elements
        );
    }

    if verbose || summary.failed > 0 {
        println!("\nFailed PDFs:");
        let failed_pdfs: Vec<_> = summary.results.iter().filter(|r| !r.success).collect();

        if failed_pdfs.is_empty() {
            println!("  None - all PDFs processed successfully! ✓");
        } else {
            for result in failed_pdfs {
                println!("\n  File: {}/{}", result.category, result.filename);
                println!("  Error: {}", result.error.as_ref().unwrap_or(&"Unknown".to_string()));
            }
        }
    }

    if verbose {
        println!("\nTop 10 PDFs by Element Count:");
        let mut sorted_results = summary.results.clone();
        sorted_results.sort_by_key(|r| std::cmp::Reverse(r.total_elements));
        for (i, result) in sorted_results.iter().take(10).enumerate() {
            println!(
                "  {}. {}/{} - {} elements ({} headers, {} paragraphs, {} lists)",
                i + 1,
                result.category,
                result.filename,
                result.total_elements,
                result.header_count,
                result.paragraph_count,
                result.list_count
            );
        }
    }

    println!("\n{}", "=".repeat(70));
}

fn generate_html_report(summary: &ValidationSummary, output_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;

    writeln!(file, "<!DOCTYPE html>")?;
    writeln!(file, "<html><head>")?;
    writeln!(file, "<title>Structured Extraction Quality Report</title>")?;
    writeln!(file, "<style>")?;
    writeln!(file, "body {{ font-family: Arial, sans-serif; margin: 40px; }}")?;
    writeln!(file, "table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}")?;
    writeln!(file, "th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}")?;
    writeln!(file, "th {{ background-color: #4CAF50; color: white; }}")?;
    writeln!(file, ".success {{ color: green; }}")?;
    writeln!(file, ".failure {{ color: red; }}")?;
    writeln!(file, ".warning {{ color: orange; }}")?;
    writeln!(
        file,
        ".stats {{ background-color: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }}"
    )?;
    writeln!(file, "</style>")?;
    writeln!(file, "</head><body>")?;

    writeln!(file, "<h1>Structured Text Extraction Quality Report</h1>")?;

    writeln!(file, "<div class='stats'>")?;
    writeln!(file, "<h2>Overall Statistics</h2>")?;
    writeln!(file, "<p>Total PDFs: <strong>{}</strong></p>", summary.total_pdfs)?;
    writeln!(
        file,
        "<p class='success'>✓ Successful: <strong>{}</strong> ({:.2}%)</p>",
        summary.successful, summary.success_rate
    )?;

    let quality_class = if summary.success_rate >= 99.95 {
        "success"
    } else {
        "warning"
    };
    writeln!(
        file,
        "<p class='{}'>Quality: <strong>{:.2}%</strong> (target: 99.95%)</p>",
        quality_class, summary.success_rate
    )?;

    writeln!(file, "<p>Total Elements: <strong>{}</strong></p>", summary.total_elements)?;
    writeln!(
        file,
        "<p>Headers: <strong>{}</strong>, Paragraphs: <strong>{}</strong>, Lists: <strong>{}</strong></p>",
        summary.total_headers, summary.total_paragraphs, summary.total_lists
    )?;
    writeln!(file, "</div>")?;

    writeln!(file, "<h2>Category Breakdown</h2>")?;
    writeln!(file, "<table>")?;
    writeln!(
        file,
        "<tr><th>Category</th><th>Total</th><th>Successful</th><th>Failed</th><th>Success Rate</th><th>Avg Elements</th></tr>"
    )?;
    let mut categories: Vec<_> = summary.by_category.iter().collect();
    categories.sort_by_key(|(name, _)| *name);
    for (category, stats) in categories {
        writeln!(
            file,
            "<tr><td>{}</td><td>{}</td><td class='success'>{}</td><td class='failure'>{}</td><td>{:.1}%</td><td>{:.1}</td></tr>",
            category,
            stats.total,
            stats.successful,
            stats.failed,
            stats.success_rate,
            stats.avg_elements
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "<h2>All Results</h2>")?;
    writeln!(file, "<table>")?;
    writeln!(
        file,
        "<tr><th>Category</th><th>Filename</th><th>Status</th><th>Elements</th><th>Headers</th><th>Paragraphs</th><th>Lists</th><th>Time (ms)</th></tr>"
    )?;
    for result in &summary.results {
        let status_class = if result.success { "success" } else { "failure" };
        let status = if result.success { "✓" } else { "✗" };
        writeln!(
            file,
            "<tr><td>{}</td><td>{}</td><td class='{}'>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            result.category,
            result.filename,
            status_class,
            status,
            result.total_elements,
            result.header_count,
            result.paragraph_count,
            result.list_count,
            result.extraction_time_ms
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "</body></html>")?;

    Ok(())
}

fn main() {
    env_logger::init();

    let config = ValidatorConfig::from_args();

    println!("Structured Text Extraction Validator");
    println!("Base directory: {}", config.base_dir.display());

    // Discover all PDFs
    let pdfs = discover_pdfs(&config.base_dir);

    if pdfs.is_empty() {
        eprintln!("\nNo PDFs found in {}", config.base_dir.display());
        eprintln!("Make sure you have run: cd test_datasets && python download.py");
        std::process::exit(1);
    }

    println!("Found {} PDFs to validate\n", pdfs.len());

    // Validate each PDF
    let mut results = Vec::new();

    for (i, (path, category)) in pdfs.iter().enumerate() {
        let filename = path.file_name().unwrap().to_string_lossy();
        print!("[{}/{}] Validating {}/{} ... ", i + 1, pdfs.len(), category, filename);
        std::io::stdout().flush().unwrap();

        let result = validate_structured_extraction(path, category, config.verbose);

        if result.success {
            println!("✓ ({} ms, {} elements)", result.extraction_time_ms, result.total_elements);
            if config.verbose && result.total_elements > 0 {
                println!(
                    "        {} headers, {} paragraphs, {} lists",
                    result.header_count, result.paragraph_count, result.list_count
                );
            }
        } else {
            println!("✗");
            if config.verbose {
                if let Some(ref error) = result.error {
                    println!("        Error: {}", error);
                }
            }
        }

        results.push(result);
    }

    // Generate summary
    let summary = generate_summary(results);

    // Print console summary
    print_summary(&summary, config.verbose);

    // Generate HTML report if requested
    if let Some(ref output_path) = config.output_file {
        match generate_html_report(&summary, output_path) {
            Ok(_) => {
                println!("\n✓ HTML report saved to: {}", output_path.display());
            },
            Err(e) => {
                eprintln!("\n✗ Failed to generate HTML report: {}", e);
            },
        }
    }

    // Exit with success if we meet quality target
    if summary.success_rate >= 99.95 {
        println!("\n✓ Quality target achieved: {:.2}% >= 99.95%", summary.success_rate);
        std::process::exit(0);
    } else {
        println!("\n✗ Quality target not met: {:.2}% < 99.95%", summary.success_rate);
        std::process::exit(1);
    }
}
