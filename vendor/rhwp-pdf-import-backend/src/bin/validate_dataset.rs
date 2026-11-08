//! PDF Test Dataset Validator
//!
//! Validates all PDFs in the test_datasets directory to verify library functionality.
//!
//! Usage:
//!   cargo run --bin validate_dataset
//!   cargo run --bin validate_dataset -- --output report.html
//!   cargo run --bin validate_dataset -- --category forms
//!   cargo run --bin validate_dataset -- --verbose

use pdf_oxide::document::PdfDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfValidationResult {
    filename: String,
    category: String,
    success: bool,
    error: Option<String>,
    pdf_version: Option<String>,
    page_count: Option<usize>,
    file_size: u64,
    parse_time_ms: u128,
}

#[derive(Debug, Serialize)]
struct ValidationSummary {
    total_pdfs: usize,
    successful: usize,
    failed: usize,
    success_rate: f64,
    total_time_ms: u128,
    avg_time_ms: f64,
    by_category: HashMap<String, CategoryStats>,
    by_version: HashMap<String, usize>,
    results: Vec<PdfValidationResult>,
}

#[derive(Debug, Serialize)]
struct CategoryStats {
    total: usize,
    successful: usize,
    failed: usize,
    success_rate: f64,
}

struct ValidatorConfig {
    base_dir: PathBuf,
    output_file: Option<PathBuf>,
    category_filter: Option<String>,
    verbose: bool,
}

impl ValidatorConfig {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut output_file = None;
        let mut category_filter = None;
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
                "--category" => {
                    i += 1;
                    if i < args.len() {
                        category_filter = Some(args[i].clone());
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
            category_filter,
            verbose,
        }
    }
}

fn validate_pdf(path: &Path, category: &str) -> PdfValidationResult {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let start = Instant::now();
    let mut result = PdfValidationResult {
        filename: filename.clone(),
        category: category.to_string(),
        success: false,
        error: None,
        pdf_version: None,
        page_count: None,
        file_size,
        parse_time_ms: 0,
    };

    match PdfDocument::open(path) {
        Ok(pdf) => {
            result.success = true;

            // Get PDF version
            let (major, minor) = pdf.version();
            result.pdf_version = Some(format!("{}.{}", major, minor));

            // Get page count (if possible)
            match pdf.page_count() {
                Ok(count) => {
                    result.page_count = Some(count);
                },
                Err(e) => {
                    // Page count failed but document opened
                    result.error = Some(format!("Page count error: {}", e));
                },
            }
        },
        Err(e) => {
            result.success = false;
            result.error = Some(e.to_string());
        },
    }

    result.parse_time_ms = start.elapsed().as_millis();
    result
}

fn discover_pdfs(base_dir: &Path, category_filter: Option<&str>) -> Vec<(PathBuf, String)> {
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
        // Apply category filter if specified
        if let Some(filter) = category_filter {
            if category != filter {
                continue;
            }
        }

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

fn generate_summary(results: Vec<PdfValidationResult>, total_time_ms: u128) -> ValidationSummary {
    let total_pdfs = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total_pdfs - successful;
    let success_rate = if total_pdfs > 0 {
        (successful as f64 / total_pdfs as f64) * 100.0
    } else {
        0.0
    };

    let avg_time_ms = if total_pdfs > 0 {
        total_time_ms as f64 / total_pdfs as f64
    } else {
        0.0
    };

    // Group by category
    let mut by_category: HashMap<String, Vec<&PdfValidationResult>> = HashMap::new();
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
            (
                cat,
                CategoryStats {
                    total,
                    successful,
                    failed,
                    success_rate,
                },
            )
        })
        .collect();

    // Group by version
    let mut by_version: HashMap<String, usize> = HashMap::new();
    for result in &results {
        if let Some(ref version) = result.pdf_version {
            *by_version.entry(version.clone()).or_insert(0) += 1;
        }
    }

    ValidationSummary {
        total_pdfs,
        successful,
        failed,
        success_rate,
        total_time_ms,
        avg_time_ms,
        by_category: category_stats,
        by_version,
        results,
    }
}

fn print_summary(summary: &ValidationSummary, verbose: bool) {
    println!("\n{}", "=".repeat(60));
    println!("PDF DATASET VALIDATION REPORT");
    println!("{}", "=".repeat(60));

    println!("\nOverall Statistics:");
    println!("  Total PDFs:     {}", summary.total_pdfs);
    println!("  ✓ Successful:   {} ({:.1}%)", summary.successful, summary.success_rate);
    println!("  ✗ Failed:       {} ({:.1}%)", summary.failed, 100.0 - summary.success_rate);
    println!("  Total Time:     {:.2}s", summary.total_time_ms as f64 / 1000.0);
    println!("  Avg Time/PDF:   {:.1}ms", summary.avg_time_ms);

    println!("\nBreakdown by Category:");
    let mut categories: Vec<_> = summary.by_category.iter().collect();
    categories.sort_by_key(|(name, _)| *name);
    for (category, stats) in categories {
        println!(
            "  {:15} : {:3} total, {:3} ok, {:3} failed ({:.1}%)",
            category, stats.total, stats.successful, stats.failed, stats.success_rate
        );
    }

    println!("\nBreakdown by PDF Version:");
    let mut versions: Vec<_> = summary.by_version.iter().collect();
    versions.sort_by_key(|(version, _)| *version);
    for (version, count) in versions {
        println!("  PDF {}  : {} documents", version, count);
    }

    if verbose || summary.failed > 0 {
        println!("\nFailed PDFs:");
        let failed_pdfs: Vec<_> = summary.results.iter().filter(|r| !r.success).collect();

        if failed_pdfs.is_empty() {
            println!("  None - all PDFs validated successfully! ✓");
        } else {
            for result in failed_pdfs {
                println!("\n  File: {}/{}", result.category, result.filename);
                println!("  Error: {}", result.error.as_ref().unwrap_or(&"Unknown".to_string()));
            }
        }
    }

    println!("\n{}", "=".repeat(60));
}

fn generate_html_report(summary: &ValidationSummary, output_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;

    writeln!(file, "<!DOCTYPE html>")?;
    writeln!(file, "<html><head>")?;
    writeln!(file, "<title>PDF Dataset Validation Report</title>")?;
    writeln!(file, "<style>")?;
    writeln!(file, "body {{ font-family: Arial, sans-serif; margin: 40px; }}")?;
    writeln!(file, "table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}")?;
    writeln!(file, "th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}")?;
    writeln!(file, "th {{ background-color: #4CAF50; color: white; }}")?;
    writeln!(file, ".success {{ color: green; }}")?;
    writeln!(file, ".failure {{ color: red; }}")?;
    writeln!(
        file,
        ".stats {{ background-color: #f0f0f0; padding: 20px; margin: 20px 0; border-radius: 5px; }}"
    )?;
    writeln!(file, "</style>")?;
    writeln!(file, "</head><body>")?;

    writeln!(file, "<h1>PDF Dataset Validation Report</h1>")?;

    writeln!(file, "<div class='stats'>")?;
    writeln!(file, "<h2>Overall Statistics</h2>")?;
    writeln!(file, "<p>Total PDFs: <strong>{}</strong></p>", summary.total_pdfs)?;
    writeln!(
        file,
        "<p class='success'>✓ Successful: <strong>{}</strong> ({:.1}%)</p>",
        summary.successful, summary.success_rate
    )?;
    writeln!(
        file,
        "<p class='failure'>✗ Failed: <strong>{}</strong> ({:.1}%)</p>",
        summary.failed,
        100.0 - summary.success_rate
    )?;
    writeln!(
        file,
        "<p>Total Time: <strong>{:.2}s</strong></p>",
        summary.total_time_ms as f64 / 1000.0
    )?;
    writeln!(
        file,
        "<p>Average Time per PDF: <strong>{:.1}ms</strong></p>",
        summary.avg_time_ms
    )?;
    writeln!(file, "</div>")?;

    writeln!(file, "<h2>Category Breakdown</h2>")?;
    writeln!(file, "<table>")?;
    writeln!(
        file,
        "<tr><th>Category</th><th>Total</th><th>Successful</th><th>Failed</th><th>Success Rate</th></tr>"
    )?;
    let mut categories: Vec<_> = summary.by_category.iter().collect();
    categories.sort_by_key(|(name, _)| *name);
    for (category, stats) in categories {
        writeln!(
            file,
            "<tr><td>{}</td><td>{}</td><td class='success'>{}</td><td class='failure'>{}</td><td>{:.1}%</td></tr>",
            category, stats.total, stats.successful, stats.failed, stats.success_rate
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "<h2>PDF Version Distribution</h2>")?;
    writeln!(file, "<table>")?;
    writeln!(file, "<tr><th>PDF Version</th><th>Count</th></tr>")?;
    let mut versions: Vec<_> = summary.by_version.iter().collect();
    versions.sort_by_key(|(version, _)| *version);
    for (version, count) in versions {
        writeln!(file, "<tr><td>PDF {}</td><td>{}</td></tr>", version, count)?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "<h2>All Results</h2>")?;
    writeln!(file, "<table>")?;
    writeln!(
        file,
        "<tr><th>Category</th><th>Filename</th><th>Status</th><th>Version</th><th>Pages</th><th>Size (KB)</th><th>Time (ms)</th><th>Error</th></tr>"
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
            result.pdf_version.as_ref().unwrap_or(&"-".to_string()),
            result
                .page_count
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
            result.file_size / 1024,
            result.parse_time_ms,
            result.error.as_ref().unwrap_or(&"".to_string())
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "</body></html>")?;

    Ok(())
}

fn main() {
    env_logger::init();

    let config = ValidatorConfig::from_args();

    println!("PDF Dataset Validator");
    println!("Base directory: {}", config.base_dir.display());
    if let Some(ref filter) = config.category_filter {
        println!("Category filter: {}", filter);
    }

    // Discover all PDFs
    let pdfs = discover_pdfs(&config.base_dir, config.category_filter.as_deref());

    if pdfs.is_empty() {
        eprintln!("\nNo PDFs found in {}", config.base_dir.display());
        eprintln!("Make sure you have run: cd test_datasets && python download.py");
        std::process::exit(1);
    }

    println!("Found {} PDFs to validate\n", pdfs.len());

    // Validate each PDF
    let start_time = Instant::now();
    let mut results = Vec::new();

    for (i, (path, category)) in pdfs.iter().enumerate() {
        let filename = path.file_name().unwrap().to_string_lossy();
        print!("[{}/{}] Validating {}/{} ... ", i + 1, pdfs.len(), category, filename);
        std::io::stdout().flush().unwrap();

        let result = validate_pdf(path, category);

        if result.success {
            println!("✓ ({} ms)", result.parse_time_ms);
            if config.verbose {
                if let Some(ref version) = result.pdf_version {
                    println!("        Version: PDF {}", version);
                }
                if let Some(pages) = result.page_count {
                    println!("        Pages: {}", pages);
                }
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

    let total_time = start_time.elapsed().as_millis();

    // Generate summary
    let summary = generate_summary(results, total_time);

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

    // Exit with error code if any PDFs failed
    if summary.failed > 0 {
        std::process::exit(1);
    }
}
