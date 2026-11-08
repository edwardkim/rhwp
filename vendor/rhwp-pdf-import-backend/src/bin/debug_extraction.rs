//! Debug extraction - analyze text spans and gaps for quality issues
//!
//! Usage:
//!   cargo run --bin debug_extraction -- pdf_file.pdf
//!   RUST_LOG=debug cargo run --bin debug_extraction -- pdf_file.pdf

use pdf_oxide::document::PdfDocument;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pdf_file>", args[0]);
        std::process::exit(1);
    }

    let pdf_path = &args[1];
    println!("\n=== PDF EXTRACTION DEBUG ===");
    println!("File: {}\n", pdf_path);

    let doc = PdfDocument::open(pdf_path)?;
    let page_count = doc.page_count()?;

    // Analyze spans for first page (or specified page)
    let page_num = args
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    if page_num >= page_count {
        eprintln!("Page {} out of range (0-{})", page_num, page_count - 1);
        std::process::exit(1);
    }

    println!("Analyzing page {} of {}...\n", page_num + 1, page_count);

    // Extract spans for detailed analysis
    match doc.extract_spans(page_num) {
        Ok(spans) => {
            println!("=== EXTRACTED SPANS ===\n");
            println!("Total spans: {}\n", spans.len());

            // Analyze each span
            for (i, span) in spans.iter().enumerate() {
                let bold_status = if span.font_weight.is_bold() {
                    "BOLD"
                } else {
                    "normal"
                };

                println!("[{}] Text: '{}'", i, span.text.chars().take(50).collect::<String>());
                println!("    Font: {} {}pt {}", span.font_name, span.font_size, bold_status);
                println!(
                    "    Position: ({:.1}, {:.1}), Size: {:.1}×{:.1}",
                    span.bbox.x, span.bbox.y, span.bbox.width, span.bbox.height
                );
                println!(
                    "    Color: RGB({:.2}, {:.2}, {:.2})",
                    span.color.r, span.color.g, span.color.b
                );
                println!();
            }

            // Analyze spacing between consecutive spans
            println!("=== SPACING ANALYSIS ===\n");
            println!("Analyzing gaps between consecutive spans on same line:\n");

            let mut prev_idx: Option<usize> = None;

            for (i, span) in spans.iter().enumerate() {
                if let Some(prev_i) = prev_idx {
                    let prev = &spans[prev_i];
                    let y_diff = (prev.bbox.y - span.bbox.y).abs();

                    // Same-line tolerance scales with the larger of the two
                    // font sizes so superscripts and subscripts stay on the
                    // line they belong to (matches PdfDocument::same_line_threshold).
                    let line_tolerance = prev.font_size.max(span.font_size).max(1.0) * 0.5;

                    // Same-line check. Uses `<=` so the boundary case
                    // (y_diff exactly equal to the tolerance) is
                    // classified as same-line. Production treats a span
                    // as a new line only when `y_diff > threshold`, so
                    // the equality case belongs on the same-line side;
                    // this debug tool should report boundary deltas the
                    // same way.
                    if y_diff <= line_tolerance {
                        let gap = span.bbox.x - (prev.bbox.x + prev.bbox.width);
                        let space_threshold = prev.font_size * 0.25;

                        println!("Gap {} → {}: {:.2}pt", prev_i, i, gap);
                        println!(
                            "  Spans: '{}' | '{}'",
                            prev.text.chars().take(30).collect::<String>(),
                            span.text.chars().take(30).collect::<String>()
                        );
                        println!(
                            "  Fonts: {} {}pt | {} {}pt",
                            prev.font_name, prev.font_size, span.font_name, span.font_size
                        );
                        println!(
                            "  Bold: {} | {}",
                            if prev.font_weight.is_bold() { "Y" } else { "N" },
                            if span.font_weight.is_bold() { "Y" } else { "N" }
                        );
                        println!("  Space threshold: {:.2}pt (font_size * 0.25)", space_threshold);

                        if gap < 0.0 {
                            println!("  ⚠️  NEGATIVE GAP: Spans overlap by {:.2}pt", -gap);
                        } else if gap < 0.1 {
                            println!("  ⚠️  Very small gap (< 0.1pt)");
                        } else if gap < space_threshold {
                            println!(
                                "  ⚠️  Small gap ({:.2}pt < threshold {:.2}pt) - may merge without space",
                                gap, space_threshold
                            );
                        } else if gap < 3.0 {
                            println!("  ✓  Moderate gap ({:.2}pt) - should merge with space", gap);
                        } else {
                            println!("  ✓  Large gap ({:.2}pt) - separate spans", gap);
                        }

                        // Font transition detection
                        if prev.font_name != span.font_name {
                            println!(
                                "  🔴 Font transition: {} → {}",
                                prev.font_name, span.font_name
                            );
                        }
                        if (prev.font_size - span.font_size).abs() > 0.5 {
                            println!(
                                "  🔴 Size transition: {:.1}pt → {:.1}pt",
                                prev.font_size, span.font_size
                            );
                        }
                        if prev.font_weight.is_bold() != span.font_weight.is_bold() {
                            println!(
                                "  🔴 Bold transition: {} → {}",
                                if prev.font_weight.is_bold() {
                                    "bold"
                                } else {
                                    "normal"
                                },
                                if span.font_weight.is_bold() {
                                    "bold"
                                } else {
                                    "normal"
                                }
                            );
                        }

                        println!();
                    }
                }
                prev_idx = Some(i);
            }

            // Font analysis
            println!("=== FONT ANALYSIS ===\n");
            let mut fonts_used: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            let mut font_weights: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();

            for span in spans.iter() {
                *fonts_used.entry(span.font_name.clone()).or_insert(0) += 1;
                font_weights
                    .entry(span.font_name.clone())
                    .or_default()
                    .push(if span.font_weight.is_bold() {
                        "bold".to_string()
                    } else {
                        "normal".to_string()
                    });
            }

            for (font, count) in fonts_used {
                let weights = &font_weights[&font];
                let bold_count = weights.iter().filter(|w| w.as_str() == "bold").count();
                println!("Font: {} (used {} times, {} bold spans)", font, count, bold_count);
            }

            println!();
        },
        Err(e) => {
            eprintln!("Error extracting spans: {}", e);
            std::process::exit(1);
        },
    }

    Ok(())
}
