//! Extract PDFs using the published pdf_oxide crate from crates.io (v0.1.4)
//!
//! This binary extracts PDFs using the production version of pdf_oxide
//! published on crates.io. Use this to compare quality and performance
//! against the development version with Priority 1/2/3 features.
//!
//! Usage:
//!   cargo run --release --bin extract_with_published_crate -- \
//!     --input-dir /path/to/pdfs \
//!     --output-dir /path/to/output \
//!     --verbose
//!
//! Note: This binary requires the published crate dependency to be configured
//! in Cargo.toml. See README.md for setup instructions.

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
        let mut pdf_dir = PathBuf::from("test_datasets/pdfs");
        let mut output_dir = PathBuf::from("markdown_exports/published_crate");
        let mut verbose = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--input-dir" => {
                    i += 1;
                    if i < args.len() {
                        pdf_dir = PathBuf::from(&args[i]);
                    }
                },
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
            pdf_dir,
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

    fn walk_dir(dir: &Path, pdfs: &mut Vec<(PathBuf, String)>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, pdfs);
                } else if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    let relative_path = path
                        .strip_prefix("test_datasets/pdfs")
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    pdfs.push((path, relative_path));
                }
            }
        }
    }

    walk_dir(base_dir, &mut pdfs);
    pdfs.sort();
    pdfs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExportConfig::from_args();

    // Create output directory
    fs::create_dir_all(&config.output_dir)?;

    println!("=== PDF Extraction with Published pdf_oxide Crate (v0.1.4) ===");
    println!("Input directory: {}", config.pdf_dir.display());
    println!("Output directory: {}", config.output_dir.display());
    println!();

    // Discover PDFs
    let pdfs = discover_pdfs(&config.pdf_dir);
    println!("Found {} PDFs", pdfs.len());
    println!();

    if pdfs.is_empty() {
        println!("No PDFs found in {}", config.pdf_dir.display());
        return Ok(());
    }

    // Process each PDF
    let start = Instant::now();

    for (_pdf_path, relative_path) in &pdfs {
        let output_filename = relative_path.replace(".pdf", ".md");
        let output_path = config.output_dir.join(&output_filename);

        // Create subdirectory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if config.verbose {
            println!("Processing: {}", relative_path);
        }

        // Extract using published crate
        // This would require:
        // 1. pdf_oxide = "0.1.4" in Cargo.toml
        // 2. Using the published crate's API
        //
        // Example code (requires published crate setup):
        // use pdf_oxide::document::PdfDocument;
        // use pdf_oxide::converters::MarkdownConverter;
        //
        // match PdfDocument::open(pdf_path) {
        //     Ok(mut doc) => {
        //         let converter = MarkdownConverter::new();
        //         match converter.convert(&mut doc) {
        //             Ok(markdown) => {
        //                 let mut file = File::create(&output_path)?;
        //                 file.write_all(markdown.as_bytes())?;
        //                 successful += 1;
        //             },
        //             Err(e) => {
        //                 eprintln!("✗ Conversion failed {}: {}", relative_path, e);
        //                 failed += 1;
        //             }
        //         }
        //     },
        //     Err(e) => {
        //         eprintln!("✗ Failed to open {}: {}", relative_path, e);
        //         failed += 1;
        //     }
        // }

        // Placeholder: Write error message
        let error_msg = format!(
            "ERROR: This binary requires pdf_oxide = \"0.1.4\" dependency.\n\n\
             To use the published crate:\n\n\
             1. Add to Cargo.toml:\n\
                [dev-dependencies]\n\
                pdf_oxide = \"0.1.4\"\n\n\
             2. Update this binary with published crate API calls\n\n\
             File: {}\n",
            relative_path
        );

        if let Ok(mut file) = File::create(&output_path) {
            let _ = file.write_all(error_msg.as_bytes());
        }
    }

    let elapsed = start.elapsed();

    println!();
    println!("=== Extraction Status ===");
    println!("This binary requires setup to use published crate from crates.io");
    println!();
    println!("Setup instructions:");
    println!("1. Create a separate project or cargo feature for published crate");
    println!("2. Add: pdf_oxide = \"0.1.4\" to dependencies");
    println!("3. Update extraction code to use published API");
    println!();
    println!("Total time: {:.2}s", elapsed.as_secs_f64());

    Ok(())
}
