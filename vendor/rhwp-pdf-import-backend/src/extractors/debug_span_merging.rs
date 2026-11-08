//! Debug instrumentation for span merging analysis.
//!
//! This module provides detailed logging of span merging decisions to debug
//! spurious space insertion issues. Only compiled with the `debug-span-merging` feature.
//!
//! Phase 7 Debugging

use crate::layout::TextSpan;
use std::fmt::Write as FmtWrite;

/// Decision record for a single span gap evaluation.
#[derive(Debug, Clone)]
pub struct GapDecision {
    /// Index of the gap (0-based)
    pub gap_index: usize,
    /// Text of the left span (truncated to 20 chars)
    pub left_text: String,
    /// Text of the right span (truncated to 20 chars)
    pub right_text: String,
    /// Gap size in PDF points
    pub gap_pt: f32,
    /// Font size of left span
    pub font_size: f32,
    /// Space threshold from font size (font_size * 0.25)
    pub space_threshold_pt: f32,
    /// Conservative/adaptive threshold
    pub adaptive_threshold_pt: f32,
    /// Result of gap > space_threshold
    pub needs_space_by_gap: bool,
    /// Result of heuristic detection
    pub needs_space_by_heuristic: bool,
    /// Result of gap > adaptive_threshold
    pub needs_space_by_adaptive: bool,
    /// Final decision
    pub space_inserted: bool,
    /// Reason for the decision
    pub reason: SpaceInsertReason,
}

/// Reason why a space was or was not inserted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceInsertReason {
    /// Gap exceeded adaptive threshold
    AdaptiveThreshold,
    /// Heuristic detected word boundary (e.g., CamelCase)
    Heuristic,
    /// Both adaptive and heuristic triggered
    AdaptiveAndHeuristic,
    /// Gap below all thresholds - no space
    BelowThreshold,
    /// Gap negative (overlap) - no space
    NegativeGap,
}

impl std::fmt::Display for SpaceInsertReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceInsertReason::AdaptiveThreshold => write!(f, "adaptive"),
            SpaceInsertReason::Heuristic => write!(f, "heuristic"),
            SpaceInsertReason::AdaptiveAndHeuristic => write!(f, "adaptive+heuristic"),
            SpaceInsertReason::BelowThreshold => write!(f, "below-threshold"),
            SpaceInsertReason::NegativeGap => write!(f, "negative-gap"),
        }
    }
}

/// Statistics for gap distribution on a page.
#[derive(Debug, Clone, Default)]
pub struct PageGapStats {
    /// Page number (0-indexed)
    pub page_num: usize,
    /// Total number of spans
    pub span_count: usize,
    /// Total number of gaps
    pub gap_count: usize,
    /// Number of positive gaps
    pub positive_gaps: usize,
    /// Number of negative gaps (overlaps)
    pub negative_gaps: usize,
    /// Minimum gap
    pub min_gap: f32,
    /// Maximum gap
    pub max_gap: f32,
    /// Mean gap
    pub mean_gap: f32,
    /// Median gap (from positive gaps only)
    pub median_gap: f32,
    /// 25th percentile
    pub p25: f32,
    /// 75th percentile
    pub p75: f32,
}

/// Threshold computation details.
#[derive(Debug, Clone)]
pub struct ThresholdComputation {
    /// Page number
    pub page_num: usize,
    /// Config name (e.g., "balanced", "adaptive")
    pub config_name: String,
    /// Multiplier used
    pub multiplier: f32,
    /// Min clamp value
    pub min_threshold: f32,
    /// Max clamp value
    pub max_threshold: f32,
    /// Median gap from statistics
    pub median_gap: f32,
    /// Computed value before clamping (median * multiplier)
    pub computed_raw: f32,
    /// Final clamped value
    pub computed_final: f32,
    /// Whether bimodal detection was used
    pub used_bimodal: bool,
    /// Reason string from analyzer
    pub reason: String,
}

/// Debugger for span merging analysis.
///
/// Collects detailed information about each span merging decision
/// and generates formatted reports.
#[derive(Debug, Default)]
pub struct SpanMergingDebugger {
    /// Current page being processed
    pub current_page: usize,
    /// Gap decisions for current page
    pub gap_decisions: Vec<GapDecision>,
    /// Threshold computations per page
    pub threshold_computations: Vec<ThresholdComputation>,
    /// Gap statistics per page
    pub page_stats: Vec<PageGapStats>,
    /// Total spaces inserted
    pub total_spaces_inserted: usize,
    /// Spaces inserted by adaptive threshold
    pub spaces_by_adaptive: usize,
    /// Spaces inserted by heuristic
    pub spaces_by_heuristic: usize,
    /// Spaces inserted by both
    pub spaces_by_both: usize,
}

impl SpanMergingDebugger {
    /// Create a new debugger instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current page being processed.
    pub fn set_page(&mut self, page_num: usize) {
        self.current_page = page_num;
    }

    /// Record a threshold computation.
    pub fn record_threshold(
        &mut self,
        config_name: &str,
        multiplier: f32,
        min_threshold: f32,
        max_threshold: f32,
        median_gap: f32,
        computed_raw: f32,
        computed_final: f32,
        used_bimodal: bool,
        reason: &str,
    ) {
        self.threshold_computations.push(ThresholdComputation {
            page_num: self.current_page,
            config_name: config_name.to_string(),
            multiplier,
            min_threshold,
            max_threshold,
            median_gap,
            computed_raw,
            computed_final,
            used_bimodal,
            reason: reason.to_string(),
        });
    }

    /// Record page gap statistics.
    pub fn record_page_stats(&mut self, stats: PageGapStats) {
        self.page_stats.push(stats);
    }

    /// Record a gap decision.
    pub fn record_gap_decision(
        &mut self,
        gap_index: usize,
        left_text: &str,
        right_text: &str,
        gap_pt: f32,
        font_size: f32,
        space_threshold_pt: f32,
        adaptive_threshold_pt: f32,
        needs_space_by_gap: bool,
        needs_space_by_heuristic: bool,
        needs_space_by_adaptive: bool,
        space_inserted: bool,
    ) {
        let reason = if gap_pt < 0.0 {
            SpaceInsertReason::NegativeGap
        } else if !space_inserted {
            SpaceInsertReason::BelowThreshold
        } else if needs_space_by_adaptive && needs_space_by_heuristic {
            SpaceInsertReason::AdaptiveAndHeuristic
        } else if needs_space_by_adaptive {
            SpaceInsertReason::AdaptiveThreshold
        } else if needs_space_by_heuristic {
            SpaceInsertReason::Heuristic
        } else {
            SpaceInsertReason::BelowThreshold
        };

        // Update counters
        if space_inserted {
            self.total_spaces_inserted += 1;
            match reason {
                SpaceInsertReason::AdaptiveThreshold => self.spaces_by_adaptive += 1,
                SpaceInsertReason::Heuristic => self.spaces_by_heuristic += 1,
                SpaceInsertReason::AdaptiveAndHeuristic => self.spaces_by_both += 1,
                _ => {},
            }
        }

        // Truncate text for display
        let left_truncated = if left_text.len() > 20 {
            format!("{}...", &left_text[..17])
        } else {
            left_text.to_string()
        };
        let right_truncated = if right_text.len() > 20 {
            format!("{}...", &right_text[..17])
        } else {
            right_text.to_string()
        };

        self.gap_decisions.push(GapDecision {
            gap_index,
            left_text: left_truncated,
            right_text: right_truncated,
            gap_pt,
            font_size,
            space_threshold_pt,
            adaptive_threshold_pt,
            needs_space_by_gap,
            needs_space_by_heuristic,
            needs_space_by_adaptive,
            space_inserted,
            reason,
        });
    }

    /// Generate formatted report for a specific page.
    pub fn generate_page_report(&self, page_num: usize) -> String {
        let mut report = String::new();

        writeln!(report, "=== PAGE {} SPAN MERGING ANALYSIS ===", page_num).unwrap();
        writeln!(report).unwrap();

        // Find page stats
        if let Some(stats) = self.page_stats.iter().find(|s| s.page_num == page_num) {
            writeln!(report, "Extracted {} spans from page {}", stats.span_count, page_num)
                .unwrap();
            writeln!(report).unwrap();
            writeln!(report, "Gap Statistics:").unwrap();
            writeln!(report, "  Total gaps: {}", stats.gap_count).unwrap();
            writeln!(report, "  Positive gaps: {}", stats.positive_gaps).unwrap();
            writeln!(report, "  Negative gaps (overlaps): {}", stats.negative_gaps).unwrap();
            writeln!(report, "  Min: {:.2}pt", stats.min_gap).unwrap();
            writeln!(report, "  Max: {:.2}pt", stats.max_gap).unwrap();
            writeln!(report, "  Mean: {:.2}pt", stats.mean_gap).unwrap();
            writeln!(report, "  Median: {:.2}pt", stats.median_gap).unwrap();
            writeln!(report, "  P25: {:.2}pt, P75: {:.2}pt", stats.p25, stats.p75).unwrap();
            writeln!(report).unwrap();
        }

        // Find threshold computation
        if let Some(thresh) = self
            .threshold_computations
            .iter()
            .find(|t| t.page_num == page_num)
        {
            writeln!(report, "Adaptive Threshold Computation:").unwrap();
            writeln!(
                report,
                "  Config: {} [multiplier={}, min={}pt, max={}pt]",
                thresh.config_name, thresh.multiplier, thresh.min_threshold, thresh.max_threshold
            )
            .unwrap();
            if thresh.used_bimodal {
                writeln!(report, "  Method: Bimodal detection").unwrap();
            } else {
                writeln!(report, "  Median gap: {:.2}pt", thresh.median_gap).unwrap();
                writeln!(
                    report,
                    "  Computed: {:.2}pt * {} = {:.2}pt",
                    thresh.median_gap, thresh.multiplier, thresh.computed_raw
                )
                .unwrap();
            }
            writeln!(
                report,
                "  Clamped to: {:.2}pt (within [{}, {}])",
                thresh.computed_final, thresh.min_threshold, thresh.max_threshold
            )
            .unwrap();
            writeln!(report, "  Reason: {}", thresh.reason).unwrap();
            writeln!(report).unwrap();
        }

        // Gap decisions for this page
        let page_decisions: Vec<_> = self.gap_decisions.iter().collect();

        if !page_decisions.is_empty() {
            writeln!(report, "Space Insertion Analysis (first 30 gaps):").unwrap();
            for (i, decision) in page_decisions.iter().take(30).enumerate() {
                writeln!(
                    report,
                    "  Gap {}: {:.2}pt (span \"{}\" -> \"{}\")",
                    i + 1,
                    decision.gap_pt,
                    decision.left_text,
                    decision.right_text
                )
                .unwrap();
                writeln!(
                    report,
                    "    - needs_space_by_gap ({:.2}pt): {} ({:.2} {} {:.2})",
                    decision.space_threshold_pt,
                    if decision.needs_space_by_gap {
                        "YES"
                    } else {
                        "NO"
                    },
                    decision.gap_pt,
                    if decision.needs_space_by_gap {
                        ">"
                    } else {
                        "<"
                    },
                    decision.space_threshold_pt
                )
                .unwrap();
                writeln!(
                    report,
                    "    - needs_space_by_heuristic: {}",
                    if decision.needs_space_by_heuristic {
                        "YES"
                    } else {
                        "NO"
                    }
                )
                .unwrap();
                writeln!(
                    report,
                    "    - needs_space_by_adaptive ({:.2}pt): {} ({:.2} {} {:.2})",
                    decision.adaptive_threshold_pt,
                    if decision.needs_space_by_adaptive {
                        "YES"
                    } else {
                        "NO"
                    },
                    decision.gap_pt,
                    if decision.needs_space_by_adaptive {
                        ">"
                    } else {
                        "<"
                    },
                    decision.adaptive_threshold_pt
                )
                .unwrap();
                let marker = if decision.space_inserted {
                    "SPACE INSERTED"
                } else {
                    "NO SPACE"
                };
                writeln!(report, "    -> {} ({})", marker, decision.reason).unwrap();
                writeln!(report).unwrap();
            }
        }

        report
    }

    /// Generate summary report.
    pub fn generate_summary(&self) -> String {
        let mut report = String::new();

        writeln!(report, "=== SPAN MERGING SUMMARY ===").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "Total Spaces Inserted: {}", self.total_spaces_inserted).unwrap();
        writeln!(report, "  - By adaptive threshold: {} spaces", self.spaces_by_adaptive).unwrap();
        writeln!(report, "  - By heuristic: {} spaces", self.spaces_by_heuristic).unwrap();
        writeln!(report, "  - By both (adaptive+heuristic): {} spaces", self.spaces_by_both)
            .unwrap();
        writeln!(report).unwrap();

        // Per-page threshold summary
        writeln!(report, "Per-Page Adaptive Thresholds:").unwrap();
        for thresh in &self.threshold_computations {
            writeln!(
                report,
                "  Page {}: {:.2}pt (median: {:.2}pt, {})",
                thresh.page_num,
                thresh.computed_final,
                thresh.median_gap,
                if thresh.used_bimodal {
                    "bimodal"
                } else {
                    "median*multiplier"
                }
            )
            .unwrap();
        }

        report
    }
}

/// Compute page gap statistics from spans.
pub fn compute_page_gap_stats(page_num: usize, spans: &[TextSpan]) -> PageGapStats {
    if spans.len() < 2 {
        return PageGapStats {
            page_num,
            span_count: spans.len(),
            ..Default::default()
        };
    }

    let gaps: Vec<f32> = spans
        .windows(2)
        .map(|w| w[1].bbox.left() - w[0].bbox.right())
        .collect();

    let positive_gaps: Vec<f32> = gaps.iter().filter(|&&g| g > 0.0).copied().collect();
    let negative_count = gaps.iter().filter(|&&g| g < 0.0).count();

    let min = gaps.iter().copied().fold(f32::INFINITY, f32::min);
    let max = gaps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;

    // Compute median and percentiles from positive gaps
    let (median, p25, p75) = if !positive_gaps.is_empty() {
        let mut sorted = positive_gaps.clone();
        sorted.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        let len = sorted.len();
        let median = sorted[len / 2];
        let p25 = sorted[len / 4];
        let p75 = sorted[3 * len / 4];
        (median, p25, p75)
    } else {
        (0.0, 0.0, 0.0)
    };

    PageGapStats {
        page_num,
        span_count: spans.len(),
        gap_count: gaps.len(),
        positive_gaps: positive_gaps.len(),
        negative_gaps: negative_count,
        min_gap: min,
        max_gap: max,
        mean_gap: mean,
        median_gap: median,
        p25,
        p75,
    }
}
