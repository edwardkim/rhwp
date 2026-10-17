#!/usr/bin/env rust
//! Comprehensive PDF feature analyzer
//!
//! Analyzes all PDFs in test dataset to identify:
//! - Successfully parsed features
//! - Missing or unsupported features
//! - Coverage statistics
//! - Potential improvements

#![allow(dead_code)]
#![allow(unused_variables)]

use pdf_oxide::{Error, PdfDocument};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
struct FeatureStats {
    total_files: usize,
    successful_parses: usize,
    failed_parses: usize,

    // Font features
    fonts_found: usize,
    font_descriptors: usize,
    missing_font_descriptors: usize,
    encoding_found: usize,
    to_unicode_cmaps: usize,

    // Content features
    images_found: usize,
    forms_found: usize,
    annotations_found: usize,
    xobjects_found: usize,

    // Structure features
    pages_found: usize,
    total_pages: usize,
    has_outline: usize,
    has_metadata: usize,

    // Compression
    flate_streams: usize,
    lzw_streams: usize,
    dct_streams: usize,
    other_filters: HashSet<String>,

    // Missing features by file
    files_missing_features: HashMap<String, Vec<String>>,
}

fn analyze_pdf(path: &Path, stats: &mut FeatureStats) -> Result<(), Error> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();

    let doc = match PdfDocument::open(path) {
        Ok(doc) => {
            stats.successful_parses += 1;
            doc
        },
        Err(e) => {
            stats.failed_parses += 1;
            println!("❌ {} - Failed to parse: {}", filename, e);
            return Err(e);
        },
    };

    let mut missing_features = Vec::new();

    // Basic info
    let page_count = match doc.page_count() {
        Ok(count) => count,
        Err(e) => {
            missing_features.push(format!("Failed to get page count: {}", e));
            0
        },
    };

    if page_count > 0 {
        stats.pages_found += 1;
        stats.total_pages += page_count;
    } else {
        missing_features.push("No pages detected".to_string());
    }

    // Analyze fonts (check if we can access font information)
    // This is a heuristic - try to extract text and see if we get font info
    if page_count > 0 {
        match doc.extract_text(0) {
            Ok(text) => {
                if !text.is_empty() {
                    stats.fonts_found += 1;
                    // We successfully extracted text, so fonts are working
                }
            },
            Err(_) => {
                missing_features.push("Text extraction failed".to_string());
            },
        }
    }

    // Check for images
    if page_count > 0 {
        match doc.extract_images(0) {
            Ok(images) => {
                if !images.is_empty() {
                    stats.images_found += 1;
                }
            },
            Err(_) => {
                // Images might not exist or extraction might be unsupported
            },
        }
    }

    if !missing_features.is_empty() {
        stats
            .files_missing_features
            .insert(filename.clone(), missing_features);
    }

    Ok(())
}

fn analyze_raw_pdf_structure(path: &Path, stats: &mut FeatureStats) {
    // Read raw PDF bytes to check for features we might not be parsing
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return,
    };

    // Convert to string, replacing invalid UTF-8 (PDFs are mostly ASCII with binary streams)
    let content = String::from_utf8_lossy(&bytes);

    // Check for various PDF features in raw content
    if content.contains("/FontDescriptor") {
        stats.font_descriptors += 1;
    }

    if content.contains("/Encoding") {
        stats.encoding_found += 1;
    }

    if content.contains("/ToUnicode") {
        stats.to_unicode_cmaps += 1;
    }

    if content.contains("/Annot") {
        stats.annotations_found += 1;
    }

    if content.contains("/AcroForm") || content.contains("/XFA") {
        stats.forms_found += 1;
    }

    if content.contains("/XObject") {
        stats.xobjects_found += 1;
    }

    if content.contains("/Outlines") {
        stats.has_outline += 1;
    }

    if content.contains("/Metadata") {
        stats.has_metadata += 1;
    }

    // Check compression filters
    if content.contains("/FlateDecode") {
        stats.flate_streams += 1;
    }

    if content.contains("/LZWDecode") {
        stats.lzw_streams += 1;
    }

    if content.contains("/DCTDecode") {
        stats.dct_streams += 1;
    }

    // Check for other filters
    let filters = [
        "ASCII85Decode",
        "ASCIIHexDecode",
        "RunLengthDecode",
        "CCITTFaxDecode",
        "JBIG2Decode",
        "JPXDecode",
        "Crypt",
    ];

    for filter in &filters {
        if content.contains(filter) {
            stats.other_filters.insert(filter.to_string());
        }
    }
}

fn main() {
    println!("=== PDF Feature Coverage Analysis ===\n");

    let test_dir = Path::new("test_datasets/pdfs");

    if !test_dir.exists() {
        eprintln!("Error: test_datasets/pdfs directory not found");
        std::process::exit(1);
    }

    let mut stats = FeatureStats::default();

    // Find all PDFs
    let mut pdf_files = Vec::new();

    for category in &["forms", "mixed", "technical"] {
        let category_dir = test_dir.join(category);
        if !category_dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&category_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "pdf").unwrap_or(false) {
                    pdf_files.push(path);
                }
            }
        }
    }

    stats.total_files = pdf_files.len();

    println!("Found {} PDF files\n", stats.total_files);

    // Analyze each PDF
    for (i, path) in pdf_files.iter().enumerate() {
        if (i + 1) % 10 == 0 {
            println!("Progress: {}/{}", i + 1, stats.total_files);
        }

        // High-level API analysis
        let _ = analyze_pdf(path, &mut stats);

        // Low-level structure analysis
        analyze_raw_pdf_structure(path, &mut stats);
    }

    println!("\n=== Results ===\n");

    // Parsing success rate
    println!("## Parsing Success");
    println!(
        "  Successful: {}/{} ({:.1}%)",
        stats.successful_parses,
        stats.total_files,
        (stats.successful_parses as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  Failed: {}/{} ({:.1}%)",
        stats.failed_parses,
        stats.total_files,
        (stats.failed_parses as f64 / stats.total_files as f64) * 100.0
    );

    // Page detection
    println!("\n## Page Detection");
    println!(
        "  Files with pages: {}/{} ({:.1}%)",
        stats.pages_found,
        stats.successful_parses,
        (stats.pages_found as f64 / stats.successful_parses.max(1) as f64) * 100.0
    );
    println!("  Total pages: {}", stats.total_pages);
    println!(
        "  Avg pages/file: {:.1}",
        stats.total_pages as f64 / stats.pages_found.max(1) as f64
    );

    // Font features
    println!("\n## Font Features (in raw PDF)");
    println!(
        "  FontDescriptor: {}/{} ({:.1}%)",
        stats.font_descriptors,
        stats.total_files,
        (stats.font_descriptors as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  Encoding: {}/{} ({:.1}%)",
        stats.encoding_found,
        stats.total_files,
        (stats.encoding_found as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  ToUnicode CMap: {}/{} ({:.1}%)",
        stats.to_unicode_cmaps,
        stats.total_files,
        (stats.to_unicode_cmaps as f64 / stats.total_files as f64) * 100.0
    );

    // Content features
    println!("\n## Content Features");
    println!(
        "  Text extraction: {}/{} ({:.1}%)",
        stats.fonts_found,
        stats.pages_found,
        (stats.fonts_found as f64 / stats.pages_found.max(1) as f64) * 100.0
    );
    println!(
        "  Images: {}/{} ({:.1}%)",
        stats.images_found,
        stats.total_files,
        (stats.images_found as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  Forms/AcroForm: {}/{} ({:.1}%)",
        stats.forms_found,
        stats.total_files,
        (stats.forms_found as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  Annotations: {}/{} ({:.1}%)",
        stats.annotations_found,
        stats.total_files,
        (stats.annotations_found as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  XObjects: {}/{} ({:.1}%)",
        stats.xobjects_found,
        stats.total_files,
        (stats.xobjects_found as f64 / stats.total_files as f64) * 100.0
    );

    // Structure features
    println!("\n## Structure Features");
    println!(
        "  Outlines/Bookmarks: {}/{} ({:.1}%)",
        stats.has_outline,
        stats.total_files,
        (stats.has_outline as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  Metadata: {}/{} ({:.1}%)",
        stats.has_metadata,
        stats.total_files,
        (stats.has_metadata as f64 / stats.total_files as f64) * 100.0
    );

    // Compression
    println!("\n## Compression Filters");
    println!(
        "  FlateDecode: {}/{} ({:.1}%)",
        stats.flate_streams,
        stats.total_files,
        (stats.flate_streams as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  LZWDecode: {}/{} ({:.1}%)",
        stats.lzw_streams,
        stats.total_files,
        (stats.lzw_streams as f64 / stats.total_files as f64) * 100.0
    );
    println!(
        "  DCTDecode (JPEG): {}/{} ({:.1}%)",
        stats.dct_streams,
        stats.total_files,
        (stats.dct_streams as f64 / stats.total_files as f64) * 100.0
    );

    if !stats.other_filters.is_empty() {
        println!("  Other filters found: {:?}", stats.other_filters);
    }

    // Files with missing features
    if !stats.files_missing_features.is_empty() {
        println!("\n## Files with Issues ({} files)", stats.files_missing_features.len());
        for (file, issues) in stats.files_missing_features.iter().take(10) {
            println!("  {}", file);
            for issue in issues {
                println!("    - {}", issue);
            }
        }

        if stats.files_missing_features.len() > 10 {
            println!("  ... and {} more", stats.files_missing_features.len() - 10);
        }
    }

    // Recommendations
    println!("\n=== Feature Coverage Assessment ===\n");

    let coverage_pct = (stats.successful_parses as f64 / stats.total_files as f64) * 100.0;

    if coverage_pct >= 95.0 {
        println!("✅ Excellent coverage ({:.1}%)", coverage_pct);
    } else if coverage_pct >= 80.0 {
        println!("⚠️  Good coverage ({:.1}%), some improvements possible", coverage_pct);
    } else {
        println!("❌ Coverage needs improvement ({:.1}%)", coverage_pct);
    }

    // Identify potential missing features
    println!("\n## Features We Support:");
    println!("  ✅ Basic PDF parsing (objects, streams, xref)");
    println!("  ✅ Text extraction");
    println!("  ✅ FlateDecode, LZWDecode, DCTDecode");
    println!("  ✅ Font encoding and ToUnicode CMaps");
    println!("  ✅ Image extraction");
    println!("  ✅ XObject forms");

    println!("\n## Features That May Need Attention:");

    if stats.font_descriptors > stats.to_unicode_cmaps {
        let diff = stats.font_descriptors - stats.to_unicode_cmaps;
        println!("  ⚠️  {} files have FontDescriptor but no ToUnicode", diff);
        println!("      (may need fallback encoding handling)");
    }

    if stats.annotations_found > 0 {
        println!("  ℹ️  {} files have annotations", stats.annotations_found);
        println!("      (currently not extracted in text output)");
    }

    if stats.has_outline > 0 {
        println!("  ℹ️  {} files have outlines/bookmarks", stats.has_outline);
        println!("      (currently not exposed in API)");
    }

    if !stats.other_filters.is_empty() {
        println!("  ℹ️  Other compression filters found: {:?}", stats.other_filters);
        println!("      (may or may not be supported)");
    }

    println!("\n=== Recommendation ===");
    println!();

    if coverage_pct >= 95.0 {
        println!("The library has excellent coverage of the test dataset.");
        println!("Current feature set is production-ready for v0.1.0.");
        println!();
        println!("Future enhancements (v1.x):");
        println!("  - Annotation extraction (if users request it)");
        println!("  - Outline/bookmark API (if users request it)");
        println!("  - Additional compression filters (as needed)");
    } else {
        println!(
            "Consider investigating the {} failed parses before release.",
            stats.failed_parses
        );
        println!("These may represent edge cases or unsupported PDF features.");
    }
}
