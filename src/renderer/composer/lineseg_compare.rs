//! LINE_SEG 일치율 비교 모듈
//!
//! HWP 원본 LINE_SEG와 reflow_line_segs() 결과를 필드별로 비교하여
//! 일치율과 불일치 패턴을 분석한다.

use crate::model::paragraph::LineSeg;

/// 단일 LineSeg 필드별 비교 결과
#[derive(Debug, Clone)]
pub struct LineSegFieldDiff {
    pub line_idx: usize,
    pub text_start_delta: i64,
    pub line_height_delta: i32,
    pub text_height_delta: i32,
    pub baseline_distance_delta: i32,
    pub line_spacing_delta: i32,
    pub segment_width_delta: i32,
    pub vertical_pos_delta: i32,
}

impl LineSegFieldDiff {
    /// text_start가 일치하는지 (줄바꿈 위치 동일)
    pub fn text_start_match(&self) -> bool {
        self.text_start_delta == 0
    }

    /// 모든 필드가 일치하는지
    pub fn all_match(&self) -> bool {
        self.text_start_delta == 0
            && self.line_height_delta == 0
            && self.text_height_delta == 0
            && self.baseline_distance_delta == 0
            && self.line_spacing_delta == 0
            && self.segment_width_delta == 0
    }
}

/// 단일 문단의 LINE_SEG 비교 결과
#[derive(Debug, Clone)]
pub struct ParagraphLineSegDiff {
    pub para_idx: usize,
    pub original_line_count: usize,
    pub reflow_line_count: usize,
    pub line_count_match: bool,
    pub field_diffs: Vec<LineSegFieldDiff>,
}

impl ParagraphLineSegDiff {
    /// 줄 수가 같고 모든 text_start가 일치하는지 (줄바꿈 완전 일치)
    pub fn line_breaks_match(&self) -> bool {
        self.line_count_match && self.field_diffs.iter().all(|d| d.text_start_match())
    }

    /// 모든 필드가 일치하는지
    pub fn all_match(&self) -> bool {
        self.line_count_match && self.field_diffs.iter().all(|d| d.all_match())
    }
}

/// 섹션 전체의 LINE_SEG 비교 요약
#[derive(Debug, Clone)]
pub struct SectionLineSegReport {
    pub section_idx: usize,
    pub total_paragraphs: usize,
    pub compared_paragraphs: usize,
    pub line_count_match_count: usize,
    pub line_break_match_count: usize,
    pub all_match_count: usize,
    pub paragraph_diffs: Vec<ParagraphLineSegDiff>,
}

impl SectionLineSegReport {
    pub fn line_count_match_rate(&self) -> f64 {
        if self.compared_paragraphs == 0 {
            return 0.0;
        }
        self.line_count_match_count as f64 / self.compared_paragraphs as f64 * 100.0
    }

    pub fn line_break_match_rate(&self) -> f64 {
        if self.compared_paragraphs == 0 {
            return 0.0;
        }
        self.line_break_match_count as f64 / self.compared_paragraphs as f64 * 100.0
    }

    pub fn all_match_rate(&self) -> f64 {
        if self.compared_paragraphs == 0 {
            return 0.0;
        }
        self.all_match_count as f64 / self.compared_paragraphs as f64 * 100.0
    }

    /// 필드별 평균 오차 (비교 가능한 줄만 대상)
    pub fn avg_field_deltas(&self) -> AvgFieldDeltas {
        let mut count = 0u64;
        let mut sum_text_start = 0i64;
        let mut sum_line_height = 0i64;
        let mut sum_text_height = 0i64;
        let mut sum_baseline = 0i64;
        let mut sum_line_spacing = 0i64;
        let mut sum_segment_width = 0i64;

        for pd in &self.paragraph_diffs {
            for fd in &pd.field_diffs {
                count += 1;
                sum_text_start += fd.text_start_delta.abs();
                sum_line_height += fd.line_height_delta.abs() as i64;
                sum_text_height += fd.text_height_delta.abs() as i64;
                sum_baseline += fd.baseline_distance_delta.abs() as i64;
                sum_line_spacing += fd.line_spacing_delta.abs() as i64;
                sum_segment_width += fd.segment_width_delta.abs() as i64;
            }
        }

        if count == 0 {
            return AvgFieldDeltas::default();
        }

        AvgFieldDeltas {
            lines_compared: count as usize,
            text_start: sum_text_start as f64 / count as f64,
            line_height: sum_line_height as f64 / count as f64,
            text_height: sum_text_height as f64 / count as f64,
            baseline_distance: sum_baseline as f64 / count as f64,
            line_spacing: sum_line_spacing as f64 / count as f64,
            segment_width: sum_segment_width as f64 / count as f64,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AvgFieldDeltas {
    pub lines_compared: usize,
    pub text_start: f64,
    pub line_height: f64,
    pub text_height: f64,
    pub baseline_distance: f64,
    pub line_spacing: f64,
    pub segment_width: f64,
}

/// 두 LineSeg 배열을 필드별로 비교한다.
/// 줄 수가 다르면 min(a, b)까지만 비교.
pub fn compare_line_segs(
    para_idx: usize,
    original: &[LineSeg],
    reflowed: &[LineSeg],
) -> ParagraphLineSegDiff {
    let line_count_match = original.len() == reflowed.len();
    let compare_count = original.len().min(reflowed.len());

    let field_diffs: Vec<LineSegFieldDiff> = (0..compare_count)
        .map(|i| {
            let o = &original[i];
            let r = &reflowed[i];
            LineSegFieldDiff {
                line_idx: i,
                text_start_delta: r.text_start as i64 - o.text_start as i64,
                line_height_delta: r.line_height - o.line_height,
                text_height_delta: r.text_height - o.text_height,
                baseline_distance_delta: r.baseline_distance - o.baseline_distance,
                line_spacing_delta: r.line_spacing - o.line_spacing,
                segment_width_delta: r.segment_width - o.segment_width,
                vertical_pos_delta: r.vertical_pos - o.vertical_pos,
            }
        })
        .collect();

    ParagraphLineSegDiff {
        para_idx,
        original_line_count: original.len(),
        reflow_line_count: reflowed.len(),
        line_count_match,
        field_diffs,
    }
}

/// 포매팅된 리포트 문자열 생성
pub fn format_report(reports: &[SectionLineSegReport]) -> String {
    let mut out = String::new();
    out.push_str("# LINE_SEG 일치율 리포트\n\n");

    let mut total_paras = 0usize;
    let mut total_compared = 0usize;
    let mut total_line_match = 0usize;
    let mut total_break_match = 0usize;
    let mut total_all_match = 0usize;

    for report in reports {
        total_paras += report.total_paragraphs;
        total_compared += report.compared_paragraphs;
        total_line_match += report.line_count_match_count;
        total_break_match += report.line_break_match_count;
        total_all_match += report.all_match_count;

        out.push_str(&format!(
            "## 섹션 {}: 문단 {}개 (비교 {}개)\n",
            report.section_idx, report.total_paragraphs, report.compared_paragraphs
        ));
        out.push_str(&format!(
            "- 줄 수 일치: {}/{} ({:.1}%)\n",
            report.line_count_match_count,
            report.compared_paragraphs,
            report.line_count_match_rate()
        ));
        out.push_str(&format!(
            "- 줄바꿈 위치 일치: {}/{} ({:.1}%)\n",
            report.line_break_match_count,
            report.compared_paragraphs,
            report.line_break_match_rate()
        ));
        out.push_str(&format!(
            "- 전체 필드 일치: {}/{} ({:.1}%)\n",
            report.all_match_count,
            report.compared_paragraphs,
            report.all_match_rate()
        ));

        let avg = report.avg_field_deltas();
        if avg.lines_compared > 0 {
            out.push_str(&format!(
                "- 평균 오차 ({}줄): text_start={:.1} line_height={:.1} baseline={:.1} line_spacing={:.1} seg_width={:.1}\n",
                avg.lines_compared, avg.text_start, avg.line_height, avg.baseline_distance, avg.line_spacing, avg.segment_width
            ));
        }

        // 불일치 문단 상위 5개
        let mut mismatches: Vec<&ParagraphLineSegDiff> = report
            .paragraph_diffs
            .iter()
            .filter(|d| !d.all_match())
            .collect();
        mismatches.sort_by(|a, b| {
            let a_score = a
                .field_diffs
                .iter()
                .map(|f| f.text_start_delta.abs())
                .sum::<i64>();
            let b_score = b
                .field_diffs
                .iter()
                .map(|f| f.text_start_delta.abs())
                .sum::<i64>();
            b_score.cmp(&a_score)
        });
        if !mismatches.is_empty() {
            out.push_str("\n### 주요 불일치 문단 (상위 5개)\n\n");
            out.push_str("| 문단 | 원본 줄수 | reflow 줄수 | text_start 오차합 | 비고 |\n");
            out.push_str("|------|----------|------------|-------------------|------|\n");
            for pd in mismatches.iter().take(5) {
                let ts_sum: i64 = pd
                    .field_diffs
                    .iter()
                    .map(|f| f.text_start_delta.abs())
                    .sum();
                let note = if !pd.line_count_match {
                    "줄 수 불일치"
                } else {
                    "필드 차이"
                };
                out.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    pd.para_idx, pd.original_line_count, pd.reflow_line_count, ts_sum, note
                ));
            }
        }
        out.push('\n');
    }

    // 전체 요약
    if reports.len() > 1 || total_paras > 0 {
        out.push_str("## 전체 요약\n\n");
        let rate = |n: usize, d: usize| {
            if d == 0 {
                0.0
            } else {
                n as f64 / d as f64 * 100.0
            }
        };
        out.push_str(&format!(
            "- 총 문단: {} (비교 대상: {})\n",
            total_paras, total_compared
        ));
        out.push_str(&format!(
            "- 줄 수 일치율: {:.1}%\n",
            rate(total_line_match, total_compared)
        ));
        out.push_str(&format!(
            "- 줄바꿈 위치 일치율: {:.1}%\n",
            rate(total_break_match, total_compared)
        ));
        out.push_str(&format!(
            "- 전체 필드 일치율: {:.1}%\n",
            rate(total_all_match, total_compared)
        ));
    }

    out
}
