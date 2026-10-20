//! Export PDFs to Markdown using structured extraction
//!
//! Exports all PDFs to markdown format preserving document structure.
//!
//! Usage:

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(deprecated)]
//!   cargo run --release --bin export_to_markdown
//!   cargo run --release --bin export_to_markdown -- --output-dir custom/path

use pdf_oxide::content::{parse_content_stream, Operator};
use pdf_oxide::converters::{ConversionOptions, MarkdownConverter};
use pdf_oxide::document::PdfDocument;
use pdf_oxide::extractors::forms::{FieldValue, FormExtractor};
use pdf_oxide::layout::{FontWeight, TextBlock, TextChar};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Represents a graphics path (line, rectangle, etc.)
#[derive(Debug, Clone)]
struct GraphicsPath {
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    line_width: f32,
    is_dashed: bool,
    is_dotted: bool,
}

impl GraphicsPath {
    fn is_horizontal(&self) -> bool {
        (self.start_y - self.end_y).abs() < 2.0
    }

    fn is_vertical(&self) -> bool {
        (self.start_x - self.end_x).abs() < 2.0
    }

    fn length(&self) -> f32 {
        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        (dx * dx + dy * dy).sqrt()
    }
}

struct ExportConfig {
    pdf_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
}

impl ExportConfig {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut pdf_dir = PathBuf::from("test_datasets/pdfs");
        let mut output_dir = PathBuf::from("markdown_exports/our_library");
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

/// Extract graphics paths from a page's content stream
fn extract_graphics_paths(
    pdf: &mut PdfDocument,
    page_num: usize,
) -> Result<Vec<GraphicsPath>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    // Get page content stream
    let content_data = match pdf.get_page_content_data(page_num) {
        Ok(data) => data,
        Err(_) => return Ok(paths), // No content stream
    };

    // Parse operators
    let operators = parse_content_stream(&content_data)?;

    // Track graphics state
    let mut current_x = 0.0;
    let mut current_y = 0.0;
    let mut path_start_x = 0.0;
    let mut path_start_y = 0.0;
    let mut line_width = 1.0;
    let mut dash_pattern: Vec<f32> = Vec::new();
    let mut in_path = false;

    for op in operators {
        match op {
            Operator::MoveTo { x, y } => {
                current_x = x;
                current_y = y;
                path_start_x = x;
                path_start_y = y;
                in_path = true;
            },
            Operator::LineTo { x, y } => {
                if in_path {
                    // Create a path segment
                    let is_dashed = !dash_pattern.is_empty();
                    let is_dotted = if dash_pattern.len() >= 2 {
                        let on = dash_pattern[0];
                        let off = dash_pattern[1];
                        on < 5.0 && off < 5.0 && (on - off).abs() < 2.0
                    } else {
                        false
                    };

                    paths.push(GraphicsPath {
                        start_x: current_x,
                        start_y: current_y,
                        end_x: x,
                        end_y: y,
                        line_width,
                        is_dashed,
                        is_dotted,
                    });

                    current_x = x;
                    current_y = y;
                }
            },
            Operator::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                // Rectangle edges are potential horizontal/vertical lines
                let is_dashed = !dash_pattern.is_empty();
                let is_dotted = if dash_pattern.len() >= 2 {
                    let on = dash_pattern[0];
                    let off = dash_pattern[1];
                    on < 5.0 && off < 5.0 && (on - off).abs() < 2.0
                } else {
                    false
                };

                // Top edge
                paths.push(GraphicsPath {
                    start_x: x,
                    start_y: y + height,
                    end_x: x + width,
                    end_y: y + height,
                    line_width,
                    is_dashed,
                    is_dotted,
                });

                // Bottom edge
                paths.push(GraphicsPath {
                    start_x: x,
                    start_y: y,
                    end_x: x + width,
                    end_y: y,
                    line_width,
                    is_dashed,
                    is_dotted,
                });

                // Left edge
                paths.push(GraphicsPath {
                    start_x: x,
                    start_y: y,
                    end_x: x,
                    end_y: y + height,
                    line_width,
                    is_dashed,
                    is_dotted,
                });

                // Right edge
                paths.push(GraphicsPath {
                    start_x: x + width,
                    start_y: y,
                    end_x: x + width,
                    end_y: y + height,
                    line_width,
                    is_dashed,
                    is_dotted,
                });
            },
            Operator::SetLineWidth { width } => {
                line_width = width;
            },
            Operator::SetDash { array, phase: _ } => {
                dash_pattern = array;
            },
            Operator::Stroke | Operator::CloseFillStroke => {
                in_path = false;
            },
            Operator::EndPath => {
                in_path = false;
            },
            _ => {},
        }
    }

    Ok(paths)
}

/// Convert graphics paths to markdown representation
fn paths_to_markdown(paths: &[GraphicsPath], page_width: f32) -> String {
    let mut markdown = String::new();

    // Filter for significant horizontal lines (at least 30% of page width)
    let min_length = page_width * 0.3;
    let mut horizontal_lines: Vec<&GraphicsPath> = paths
        .iter()
        .filter(|p| p.is_horizontal() && p.length() > min_length)
        .collect();

    // Sort by Y coordinate (top to bottom)
    horizontal_lines.sort_by(|a, b| b.start_y.total_cmp(&a.start_y));

    // Group lines that are close together (within 10 pixels)
    let mut last_y = f32::INFINITY;
    for path in horizontal_lines {
        let y_gap = (last_y - path.start_y).abs();

        // Only add line if it's significantly separated from previous
        if y_gap > 10.0 {
            if path.is_dotted {
                // Dotted line - render as dots
                let num_dots = (path.length() / 10.0) as usize;
                markdown.push_str(&".".repeat(num_dots.min(80)));
            } else if path.is_dashed {
                // Dashed line - render as dashes
                markdown.push_str("- - - - - - - - - -");
            } else {
                // Solid line - markdown horizontal rule
                markdown.push_str("---");
            }
            markdown.push_str("\n\n");
            last_y = path.start_y;
        }
    }

    markdown
}

fn export_to_markdown(
    pdf_path: &Path,
    output_path: &Path,
    verbose: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut pdf = PdfDocument::open(pdf_path)?;
    let page_count = pdf.page_count()?;

    let mut markdown = String::new();

    // Add document metadata
    markdown.push_str(&format!(
        "# Extracted from: {}\n\n",
        pdf_path.file_name().unwrap().to_string_lossy()
    ));
    markdown.push_str(&format!("**Pages:** {}\n\n", page_count));
    markdown.push_str("---\n\n");

    // Extract form fields if present (AcroForm)
    match FormExtractor::extract_fields(&pdf) {
        Ok(fields) if !fields.is_empty() => {
            markdown.push_str("## Form Fields\n\n");
            markdown
                .push_str(&format!("*This document contains {} form fields:*\n\n", fields.len()));

            // Group fields by type for better organization
            let mut text_fields = Vec::new();
            let mut button_fields = Vec::new();
            let mut choice_fields = Vec::new();
            let mut signature_fields = Vec::new();

            for field in &fields {
                match field.field_type {
                    pdf_oxide::extractors::forms::FieldType::Text => text_fields.push(field),
                    pdf_oxide::extractors::forms::FieldType::Button => button_fields.push(field),
                    pdf_oxide::extractors::forms::FieldType::Choice => choice_fields.push(field),
                    pdf_oxide::extractors::forms::FieldType::Signature => {
                        signature_fields.push(field)
                    },
                    _ => {},
                }
            }

            // Format fields in markdown table
            if !text_fields.is_empty() {
                markdown.push_str("### Text Fields\n\n");
                markdown.push_str("| Field Name | Value |\n");
                markdown.push_str("|------------|-------|\n");
                for field in &text_fields {
                    let name = if field.full_name.is_empty() {
                        format!("*[unnamed field #{}]*", field.name)
                    } else {
                        field.full_name.clone()
                    };
                    let value = match &field.value {
                        FieldValue::Text(s) => s.clone(),
                        FieldValue::Boolean(b) => if *b { "☑" } else { "☐" }.to_string(),
                        FieldValue::Name(n) => n.clone(),
                        FieldValue::Array(arr) => arr.join(", "),
                        FieldValue::None => "*[empty]*".to_string(),
                    };
                    markdown.push_str(&format!(
                        "| {} | {} |\n",
                        name.replace('|', "\\|"),
                        value.replace('|', "\\|")
                    ));
                }
                markdown.push('\n');
            }

            if !button_fields.is_empty() {
                markdown.push_str("### Button Fields (Checkboxes/Radio)\n\n");
                markdown.push_str("| Field Name | Checked |\n");
                markdown.push_str("|------------|:-------:|\n");
                for field in &button_fields {
                    let name = if field.full_name.is_empty() {
                        "*[unnamed field]*".to_string()
                    } else {
                        field.full_name.clone()
                    };
                    let checked = match &field.value {
                        FieldValue::Boolean(true) => "☑",
                        FieldValue::Boolean(false) => "☐",
                        FieldValue::Name(n) if n == "Yes" || n == "On" => "☑",
                        _ => "☐",
                    };
                    markdown.push_str(&format!("| {} | {} |\n", name.replace('|', "\\|"), checked));
                }
                markdown.push('\n');
            }

            if !choice_fields.is_empty() {
                markdown.push_str("### Choice Fields (Dropdowns/Lists)\n\n");
                markdown.push_str("| Field Name | Selected Value |\n");
                markdown.push_str("|------------|----------------|\n");
                for field in &choice_fields {
                    let name = if field.full_name.is_empty() {
                        "*[unnamed field]*".to_string()
                    } else {
                        field.full_name.clone()
                    };
                    let value = match &field.value {
                        FieldValue::Text(s) => s.clone(),
                        FieldValue::Name(n) => n.clone(),
                        FieldValue::Array(arr) => arr.join(", "),
                        _ => "*[none]*".to_string(),
                    };
                    markdown.push_str(&format!(
                        "| {} | {} |\n",
                        name.replace('|', "\\|"),
                        value.replace('|', "\\|")
                    ));
                }
                markdown.push('\n');
            }

            if !signature_fields.is_empty() {
                markdown.push_str("### Signature Fields\n\n");
                for field in &signature_fields {
                    let name = if field.full_name.is_empty() {
                        "*[unnamed signature field]*".to_string()
                    } else {
                        field.full_name.clone()
                    };
                    markdown.push_str(&format!(
                        "- **{}**: {}\n",
                        name,
                        match &field.value {
                            FieldValue::None => "*[not signed]*",
                            _ => "*[signed]*",
                        }
                    ));
                }
                markdown.push('\n');
            }

            markdown.push_str("---\n\n");
        },
        Ok(_) => {
            // No form fields
            if verbose {
                markdown.push_str("*No form fields in this document.*\n\n");
            }
        },
        Err(e) => {
            if verbose {
                markdown.push_str(&format!("*Error extracting form fields: {}*\n\n", e));
            }
        },
    }

    // TEXT EXTRACTION: Use pdf.extract_text() which properly handles unusual coordinate systems
    // (like EU GDPR PDF with Y=0.0 coordinates) via sequence-based span ordering
    let pages_to_process = page_count;

    for page_num in 0..pages_to_process {
        if verbose && page_count > 1 {
            markdown.push_str(&format!("## Page {}\n\n", page_num + 1));
        }

        // Extract graphics paths (horizontal rules, etc.)
        if let Ok(paths) = extract_graphics_paths(&mut pdf, page_num) {
            if !paths.is_empty() {
                if verbose {
                    let horizontal = paths.iter().filter(|p| p.is_horizontal()).count();
                    markdown.push_str(&format!(
                        "*[Detected {} graphics paths, {} horizontal]*\n\n",
                        paths.len(),
                        horizontal
                    ));
                }

                // Get page dimensions for filtering
                // Assume standard letter size if we can't get dimensions
                let page_width = 612.0; // 8.5 inches * 72 DPI
                let graphics_md = paths_to_markdown(&paths, page_width);
                markdown.push_str(&graphics_md);
            }
        }

        // Use span-based markdown conversion which preserves BOTH reading order AND formatting (bold, etc.)
        // This fixes both the GDPR ordering issue AND preserves bold text detection
        match pdf.extract_spans(page_num) {
            Ok(spans) => {
                let converter = MarkdownConverter::new();
                let options = ConversionOptions::default();

                match converter.convert_page_from_spans(&spans, &options) {
                    Ok(page_markdown) => {
                        markdown.push_str(&page_markdown);
                        markdown.push_str("\n\n");
                    },
                    Err(e) => {
                        if verbose {
                            markdown.push_str(&format!(
                                "*[Error converting page {} to markdown: {}]*\n\n",
                                page_num, e
                            ));
                        }
                    },
                }
            },
            Err(e) => {
                if verbose {
                    markdown
                        .push_str(&format!("*[Error extracting page {}: {}]*\n\n", page_num, e));
                }
            },
        }
    }

    if page_count > pages_to_process {
        markdown.push_str(&format!(
            "\n---\n\n*[{} additional pages not shown]*\n",
            page_count - pages_to_process
        ));
    }

    // Write to file
    fs::create_dir_all(output_path.parent().unwrap())?;
    let mut file = File::create(output_path)?;
    file.write_all(markdown.as_bytes())?;

    Ok(markdown.len())
}

/// Convert TextChar array to formatted markdown with bold/italic markers
fn chars_to_formatted_markdown(chars: &[TextChar]) -> String {
    if chars.is_empty() {
        return String::new();
    }

    // IMPORTANT: Sort characters by position (top-to-bottom, left-to-right)
    // PDF streams can return characters in arbitrary order
    let mut sorted_chars = chars.to_vec();
    sorted_chars.sort_by(|a, b| {
        // First sort by Y (descending - top to bottom)
        let a_y = a.bbox.y.round() as i32;
        let b_y = b.bbox.y.round() as i32;
        match b_y.cmp(&a_y) {
            std::cmp::Ordering::Equal => {
                // Then by X (ascending - left to right)
                let a_x = a.bbox.x.round() as i32;
                let b_x = b.bbox.x.round() as i32;
                a_x.cmp(&b_x)
            },
            other => other,
        }
    });

    let mut result = String::new();
    let mut current_run = String::new();
    let mut current_weight = sorted_chars[0].font_weight;
    let mut prev_char: Option<&TextChar> = None;
    let mut prev_y: Option<i32> = None;

    for ch in sorted_chars.iter() {
        let current_y = ch.bbox.y.round() as i32;

        // Insert newline when moving to a new line (Y coordinate changes)
        if let Some(last_y) = prev_y {
            if (current_y - last_y).abs() >= 1 {
                // Flush current run before newline
                if !current_run.is_empty() {
                    if current_weight == FontWeight::Bold {
                        result.push_str("**");
                        result.push_str(&current_run);
                        result.push_str("**");
                    } else {
                        result.push_str(&current_run);
                    }
                    current_run.clear();
                }
                result.push('\n');
                prev_char = None; // Reset for new line
            }
        }
        prev_y = Some(current_y);

        // Check if we need to insert a space before this character
        // based on horizontal gap from previous character (same line only)
        if let Some(prev) = prev_char {
            let gap = ch.bbox.x - (prev.bbox.x + prev.bbox.width);

            // Dynamic threshold: use 0.25× average width of previous char, minimum 2 pixels
            // This detects word boundaries even with small fonts
            let threshold = (prev.bbox.width * 0.25).max(2.0);

            if gap > threshold {
                // Word boundary detected - flush current run and insert space
                if !current_run.is_empty() {
                    if current_weight == FontWeight::Bold {
                        result.push_str("**");
                        result.push_str(&current_run);
                        result.push_str("**");
                    } else {
                        result.push_str(&current_run);
                    }
                    current_run.clear();
                }
                result.push(' ');
            }
        }

        // Check if font weight changed
        if ch.font_weight != current_weight {
            // Flush current run with appropriate formatting
            if !current_run.is_empty() {
                if current_weight == FontWeight::Bold {
                    result.push_str("**");
                    result.push_str(&current_run);
                    result.push_str("**");
                } else {
                    result.push_str(&current_run);
                }
                current_run.clear();
            }
            current_weight = ch.font_weight;
        }

        current_run.push(ch.char);
        prev_char = Some(ch);
    }

    // Flush final run
    if !current_run.is_empty() {
        if current_weight == FontWeight::Bold {
            result.push_str("**");
            result.push_str(&current_run);
            result.push_str("**");
        } else {
            result.push_str(&current_run);
        }
    }

    result
}

/// Group characters into text blocks by spatial proximity.
///
/// Groups characters that are close together horizontally on the same line.
fn group_chars_into_blocks(chars: &[TextChar]) -> Vec<TextBlock> {
    if chars.is_empty() {
        return vec![];
    }

    // First, group into lines by Y coordinate
    // IMPORTANT: Use rounding to ensure stable, transitive sorting
    // Floating-point comparison without rounding can cause non-transitive ordering
    // which scrambles character order (causing garbled text like "FY 2B0u0d7g")
    let mut lines: Vec<Vec<TextChar>> = Vec::new();
    let mut sorted_chars: Vec<TextChar> = chars.to_vec();

    // Sort by rounded Y coordinate (descending - higher Y values first, which is top of page)
    sorted_chars.sort_by(|a, b| {
        let a_y_rounded = a.bbox.y.round() as i32;
        let b_y_rounded = b.bbox.y.round() as i32;
        b_y_rounded.cmp(&a_y_rounded) // Descending order
    });

    let mut current_line = Vec::new();
    let mut line_y_base = sorted_chars[0].bbox.y.round() as i32;

    for ch in sorted_chars {
        let ch_y_rounded = ch.bbox.y.round() as i32;
        let y_gap = (ch_y_rounded - line_y_base).abs();

        // Same line only if rounded Y is exactly the same (zero tolerance)
        // IMPORTANT: Compare to line_y_base (first char in line), not last char
        if y_gap >= 1 && !current_line.is_empty() {
            lines.push(current_line.clone());
            current_line.clear();
            line_y_base = ch_y_rounded; // Set new baseline for next line
        }

        current_line.push(ch);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Now group each line into blocks by horizontal gaps
    let mut blocks = Vec::new();

    for mut line in lines {
        // Sort by X coordinate (left to right) - also using rounding for stability
        line.sort_by(|a, b| {
            let a_x_rounded = a.bbox.x.round() as i32;
            let b_x_rounded = b.bbox.x.round() as i32;
            match a_x_rounded.cmp(&b_x_rounded) {
                std::cmp::Ordering::Equal => {
                    // If X is equal, use original float comparison as tiebreaker
                    a.bbox.x.total_cmp(&b.bbox.x)
                },
                other => other,
            }
        });

        let mut current_block_chars = Vec::new();
        let mut last_x = line[0].bbox.x;

        for ch in line {
            let x_gap = ch.bbox.x
                - (last_x
                    + current_block_chars
                        .last()
                        .map(|c: &TextChar| c.bbox.width)
                        .unwrap_or(0.0));

            // Dynamic threshold: use 0.25× average char width, minimum 2 pixels
            // This detects word boundaries even with small fonts
            let avg_char_width = if !current_block_chars.is_empty() {
                let total_width: f32 = current_block_chars.iter().map(|c| c.bbox.width).sum();
                total_width / current_block_chars.len() as f32
            } else {
                ch.bbox.width
            };
            let threshold = (avg_char_width * 0.25).max(2.0);

            // New block if horizontal gap exceeds threshold
            if x_gap > threshold && !current_block_chars.is_empty() {
                blocks.push(TextBlock::from_chars(current_block_chars.clone()));
                current_block_chars.clear();
            }

            current_block_chars.push(ch.clone());
            last_x = ch.bbox.x;
        }

        if !current_block_chars.is_empty() {
            blocks.push(TextBlock::from_chars(current_block_chars));
        }
    }

    blocks
}

fn main() {
    env_logger::init();

    let config = ExportConfig::from_args();

    println!("PDF to Markdown Exporter (Our Library)");
    println!("PDF directory: {}", config.pdf_dir.display());
    println!("Output directory: {}", config.output_dir.display());

    // Create output directory
    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    // Discover PDFs
    let pdfs = discover_pdfs(&config.pdf_dir);
    if pdfs.is_empty() {
        eprintln!("\nNo PDFs found in {}", config.pdf_dir.display());
        std::process::exit(1);
    }

    println!("Found {} PDFs to export\n", pdfs.len());

    let mut successful = 0;
    let mut failed = 0;
    let start_time = Instant::now();

    for (i, (pdf_path, category)) in pdfs.iter().enumerate() {
        let filename = pdf_path.file_stem().unwrap().to_string_lossy();
        let output_path = config
            .output_dir
            .join(category)
            .join(format!("{}.md", filename));

        print!("[{}/{}] Exporting {}/{}.pdf ... ", i + 1, pdfs.len(), category, filename);
        std::io::stdout().flush().unwrap();

        match export_to_markdown(pdf_path, &output_path, config.verbose) {
            Ok(bytes) => {
                println!("✓ ({} bytes)", bytes);
                successful += 1;
            },
            Err(e) => {
                println!("✗ Error: {}", e);
                failed += 1;
            },
        }
    }

    let elapsed = start_time.elapsed();

    println!("\n{}", "=".repeat(60));
    println!("EXPORT COMPLETE");
    println!("{}", "=".repeat(60));
    println!("Total PDFs:    {}", pdfs.len());
    println!("✓ Successful:  {}", successful);
    println!("✗ Failed:      {}", failed);
    println!("Time:          {:.2}s", elapsed.as_secs_f64());
    println!("Output:        {}", config.output_dir.display());
    println!("{}", "=".repeat(60));

    if failed > 0 {
        std::process::exit(1);
    }
}
