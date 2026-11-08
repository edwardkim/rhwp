//! Export PDFs to Plain Text
//!
//! Exports all PDFs to plain text format.
//!
//! Usage:
//!   cargo run --release --bin export_to_text
//!   cargo run --release --bin export_to_text -- --output-dir custom/path

use pdf_oxide::converters::ConversionOptions;
use pdf_oxide::document::PdfDocument;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

struct ExportConfig {
    pdf_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
}

impl ExportConfig {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut output_dir = PathBuf::from("text_exports/our_library");
        let mut verbose = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output-dir" => {
                    i += 1;
                    if i < args.len() {
                        output_dir = PathBuf::from(&args[i]);
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
            pdf_dir: PathBuf::from("test_datasets/pdfs"),
            output_dir,
            verbose,
        }
    }
}

fn discover_pdfs(base_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut pdfs = Vec::new();

    if !base_dir.exists() {
        eprintln!("Error: Directory {} does not exist", base_dir.display());
        return pdfs;
    }

    let categories = match fs::read_dir(base_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            return pdfs;
        },
    };

    for category in categories {
        let category_path = base_dir.join(&category);

        match fs::read_dir(&category_path) {
            Ok(entries) => {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "pdf") {
                        pdfs.push((path, category.clone()));
                    }
                }
            },
            Err(e) => eprintln!("Error reading category {}: {}", category, e),
        }
    }

    pdfs
}

fn export_pdf_to_text(
    pdf_path: &Path,
    category: &str,
    output_dir: &Path,
    verbose: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let file_stem = pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    if verbose {
        println!("Processing: {}/{}.pdf", category, file_stem);
    }

    let doc = PdfDocument::open(pdf_path)?;

    // Create conversion options
    let options = ConversionOptions::default();

    // Convert to plain text
    let text = doc.to_plain_text_all(&options)?;

    // Save to output directory
    let category_dir = output_dir.join(category);
    fs::create_dir_all(&category_dir)?;

    let output_file = category_dir.join(format!("{}.txt", file_stem));
    let mut file = File::create(&output_file)?;
    file.write_all(text.as_bytes())?;

    let bytes = text.len();

    if verbose {
        let line_count = text.lines().count();
        let word_count = text.split_whitespace().count();
        println!("  ✅ Exported to: {}", output_file.display());
        println!("  📊 Size: {} bytes ({:.2} KB)", bytes, bytes as f64 / 1024.0);
        println!("  📝 Lines: {}, Words: {}", line_count, word_count);
    }

    Ok(bytes)
}

fn main() {
    let config = ExportConfig::from_args();

    println!("PDF to Plain Text Exporter");
    println!("==========================");
    println!("PDF directory: {}", config.pdf_dir.display());
    println!("Output directory: {}", config.output_dir.display());
    println!();

    let pdfs = discover_pdfs(&config.pdf_dir);
    println!("Found {} PDF files\n", pdfs.len());

    if pdfs.is_empty() {
        eprintln!("No PDFs found in {}", config.pdf_dir.display());
        return;
    }

    fs::create_dir_all(&config.output_dir).expect("Failed to create output directory");

    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, (pdf_path, category)) in pdfs.iter().enumerate() {
        let filename = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        print!("[{}/{}] Exporting {}/{}.pdf ... ", i + 1, pdfs.len(), category, filename);
        std::io::stdout().flush().unwrap();

        match export_pdf_to_text(pdf_path, category, &config.output_dir, config.verbose) {
            Ok(bytes) => {
                println!("✓ ({} bytes)", bytes);
                success_count += 1;
            },
            Err(e) => {
                println!("✗ Error: {}", e);
                error_count += 1;
            },
        }
    }

    let elapsed = start.elapsed();

    println!("\n{}", "=".repeat(70));
    println!("Export Complete");
    println!("{}", "=".repeat(70));
    println!("Success: {}/{}", success_count, pdfs.len());
    println!("Errors: {}", error_count);
    println!("Time: {:?}", elapsed);
    println!("Output: {}", config.output_dir.display());
    println!("{}", "=".repeat(70));
}
