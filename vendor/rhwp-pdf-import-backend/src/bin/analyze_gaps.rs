/// Debug tool to analyze gap distributions and adaptive thresholds on real PDFs
///
/// Usage: cargo run --release --bin analyze_gaps -- <pdf_path>
/// Example: cargo run --release --bin analyze_gaps -- tests/fixtures/regression/policy/Anti-bribery.pdf
use pdf_oxide::document::PdfDocument;
use pdf_oxide::extractors::{gap_statistics, SpanMergingConfig};
use std::env;
use std::path::Path;

fn main() {
    // Enable detailed logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .try_init()
        .ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pdf_path>", args[0]);
        std::process::exit(1);
    }

    let pdf_path = &args[1];
    if !Path::new(pdf_path).exists() {
        eprintln!("Error: File not found: {}", pdf_path);
        std::process::exit(1);
    }

    println!("=== Gap Distribution & Adaptive Threshold Analysis ===");
    println!("PDF: {}\n", pdf_path);

    match analyze_pdf(pdf_path) {
        Ok(_) => println!("\n✅ Analysis completed"),
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            std::process::exit(1);
        },
    }
}

fn analyze_pdf(pdf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open PDF
    let doc = PdfDocument::open(pdf_path)?;
    let page_count = doc.page_count()?;

    println!("📄 PDF has {} pages\n", page_count);

    // Scan pages to find one with sufficient spans for analysis
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("SEARCHING FOR PAGE WITH ADEQUATE SPANS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let default_config = SpanMergingConfig::default();
    let mut spans = Vec::new();

    // Try to find first page with >= 50 spans
    for page_num in 0..std::cmp::min(page_count, 5) {
        match doc.extract_spans_with_config(page_num, default_config.clone()) {
            Ok(page_spans) => {
                println!("  Page {}: {} spans", page_num, page_spans.len());
                if page_spans.len() >= 50 {
                    spans = page_spans;
                    println!(
                        "  ✅ Selected page {} for analysis ({} spans)\n",
                        page_num,
                        spans.len()
                    );
                    break;
                }
            },
            Err(_) => {
                println!("  Page {}: (error reading)\n", page_num);
            },
        }
    }

    if spans.len() < 2 {
        println!("\n⚠️  Could not find page with sufficient spans (need >= 2)");
        return Ok(());
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("GAP ANALYSIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Analyze gap distribution
    analyze_gaps(&spans);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ADAPTIVE THRESHOLD ANALYSIS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test different adaptive configurations
    let configs = vec![
        ("Balanced (default)", gap_statistics::AdaptiveThresholdConfig::balanced()),
        ("Aggressive", gap_statistics::AdaptiveThresholdConfig::aggressive()),
        ("Conservative", gap_statistics::AdaptiveThresholdConfig::conservative()),
        ("Policy documents", gap_statistics::AdaptiveThresholdConfig::policy_documents()),
        ("Academic", gap_statistics::AdaptiveThresholdConfig::academic()),
    ];

    for (name, config) in configs {
        println!("Config: {}", name);
        println!("  Multiplier: {}", config.median_multiplier);
        println!("  Min: {}pt, Max: {}pt", config.min_threshold_pt, config.max_threshold_pt);
        println!("  Use IQR: {}", config.use_iqr);

        let result = gap_statistics::analyze_document_gaps(&spans, Some(config));
        println!("  Computed threshold: {:.4}pt", result.threshold_pt);
        println!("  Reason: {}\n", result.reason);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("COMPARISON: Default vs Adaptive Config");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Default Config (use_adaptive_threshold = false):");
    println!("  conservative_threshold_pt: {}pt\n", default_config.conservative_threshold_pt);

    let adaptive_config = SpanMergingConfig::adaptive();
    println!("Adaptive Config (use_adaptive_threshold = true):");
    println!(
        "  conservative_threshold_pt: {}pt (ignored, overridden by adaptive)",
        adaptive_config.conservative_threshold_pt
    );

    if let Some(ref ac) = adaptive_config.adaptive_config {
        let result = gap_statistics::analyze_document_gaps(&spans, Some(ac.clone()));
        println!("  Computed threshold: {:.4}pt\n", result.threshold_pt);
    }

    // Diagnostic analysis
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("DIAGNOSTIC INSIGHTS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Collect gaps for analysis
    let mut gaps = Vec::new();
    for window in spans.windows(2) {
        let gap = window[1].bbox.left() - window[0].bbox.right();
        gaps.push(gap);
    }

    if !gaps.is_empty() {
        let default_threshold = default_config.conservative_threshold_pt;
        let mut count_below_threshold = 0;
        let mut count_above_threshold = 0;

        for gap in &gaps {
            if *gap < default_threshold {
                count_below_threshold += 1;
            } else {
                count_above_threshold += 1;
            }
        }

        println!("📊 SPURIOUS SPACE ANALYSIS:");
        println!("  Default threshold: {:.4}pt", default_threshold);
        println!("  Gaps < threshold (no space): {}", count_below_threshold);
        println!("  Gaps >= threshold (insert space): {}", count_above_threshold);
        println!();

        if count_below_threshold > gaps.len() / 2 {
            println!("  ⚠️  WARNING: More than 50% of gaps are below threshold!");
            println!("     This suggests threshold may be TOO AGGRESSIVE");
            println!("     Many normal letter-spacing gaps treated as word spaces\n");
        }

        if count_above_threshold > 0 {
            let mut above_gaps: Vec<_> = gaps
                .iter()
                .filter(|g| **g >= default_threshold)
                .copied()
                .collect();
            above_gaps.sort_by(|a, b| a.total_cmp(b));
            println!("  Gaps that WOULD create spaces:");
            println!(
                "    Min: {:.4}pt, Max: {:.4}pt, Count: {}",
                above_gaps.first().unwrap_or(&0.0),
                above_gaps.last().unwrap_or(&0.0),
                above_gaps.len()
            );
        }
    }

    Ok(())
}

fn analyze_gaps(spans: &[pdf_oxide::layout::TextSpan]) {
    println!("📊 GAP ANALYSIS");
    println!();

    // Collect gaps
    let mut gaps = Vec::new();
    let mut gap_details = Vec::new();

    for window in spans.windows(2) {
        let gap = window[1].bbox.left() - window[0].bbox.right();
        gaps.push(gap);

        gap_details.push((window[0].text.clone(), window[1].text.clone(), gap));
    }

    println!("  Total gaps measured: {}", gaps.len());

    if gaps.is_empty() {
        println!("  (No gaps to analyze)\n");
        return;
    }

    // Compute statistics manually
    let min = gaps.iter().copied().fold(f32::INFINITY, f32::min);
    let max = gaps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;
    let variance: f32 = gaps.iter().map(|&g| (g - mean).powi(2)).sum::<f32>() / gaps.len() as f32;
    let std_dev = variance.sqrt();

    println!("  Min gap:  {:.4}pt", min);
    println!("  Max gap:  {:.4}pt", max);
    println!("  Mean gap: {:.4}pt", mean);
    println!("  Std Dev:  {:.4}pt\n", std_dev);

    // Use official statistics calculator
    if let Some(stats) = gap_statistics::calculate_statistics(gaps.clone()) {
        println!("  📈 DETAILED STATISTICS:");
        println!("    Median:  {:.4}pt", stats.median);
        println!("    P10:     {:.4}pt", stats.p10);
        println!("    P25:     {:.4}pt", stats.p25);
        println!("    P75:     {:.4}pt", stats.p75);
        println!("    P90:     {:.4}pt", stats.p90);
        println!("    IQR:     {:.4}pt", stats.iqr());
        println!("    CV:      {:.4}\n", stats.coefficient_of_variation());

        // Show gap distribution
        println!("  📊 GAP DISTRIBUTION (Histogram):");
        print_histogram(&gaps, 15);

        // Show gaps with details
        println!("  📋 INDIVIDUAL GAPS (first 20):");
        println!("    (sorted from smallest to largest)\n");

        let mut sorted_gaps = gap_details.clone();
        sorted_gaps.sort_by(|a, b| a.2.total_cmp(&b.2));

        for (i, (text1, text2, gap)) in sorted_gaps.iter().take(20).enumerate() {
            let text1_display = if text1.len() > 15 {
                format!("{}...", &text1[..12])
            } else {
                text1.clone()
            };
            let text2_display = if text2.len() > 15 {
                format!("{}...", &text2[..12])
            } else {
                text2.clone()
            };

            let marker = if *gap < 0.05 { "⚠️ " } else { "   " };
            println!(
                "    {}{:2}. {:.4}pt | '{}' → '{}'",
                marker,
                i + 1,
                gap,
                text1_display,
                text2_display
            );
        }
        println!();
    }
}

fn print_histogram(gaps: &[f32], buckets: usize) {
    if gaps.is_empty() {
        return;
    }

    let min = gaps.iter().copied().fold(f32::INFINITY, f32::min);
    let max = gaps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = if max > min { max - min } else { 0.001 };
    let bucket_size = range / buckets as f32;

    let mut histogram = vec![0; buckets];
    for &gap in gaps {
        let normalized = (gap - min) / range;
        let bucket = (normalized * buckets as f32).min(buckets as f32 - 1.0) as usize;
        histogram[bucket] += 1;
    }

    let max_count = *histogram.iter().max().unwrap_or(&0);
    let scale = if max_count > 0 { 40 / max_count } else { 1 };

    for (i, &count) in histogram.iter().enumerate() {
        let label_min = min + (i as f32) * bucket_size;
        let label_max = min + ((i + 1) as f32) * bucket_size;
        let bar = "█".repeat(count * scale);
        println!("    {:.4}-{:.4}pt [{:3}] {}", label_min, label_max, count, bar);
    }
    println!();
}
