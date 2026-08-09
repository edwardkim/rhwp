//! Issue #3211: 비캐시 LineSeg 재계산의 저장값 정합성 회귀 가드.
//!
//! 한컴이 저장한 HWP 파일에는 `PARA_LINE_SEG` 레코드가 **기준값(ground truth)** 으로 들어 있다.
//! 이 테스트는 그 저장값을 정답지로 삼고, 저장값·캐시를 레이아웃 소스로 쓰지 않은 채
//! `reflow_line_segs()` 로 **새로 계산한** LineSeg 가 정답지와 정합하는지 검사한다.
//!
//! # 사용법 — 문서 목록만 주면 된다
//!
//! ```ignore
//! assert_uncached_lineseg_compliance("코호트 이름", &["samples/foo.hwp", ...], Budget::STRICT);
//! ```
//!
//! `audit_file()` 이 표/중첩 표 셀 문단까지 재귀로 훑으므로 새 문서는 파일명만 추가하면 된다.
//! 셀 inner 폭은 `document_core` 의 실제 계약(`cell.width` − 실효 pad 좌/우,
//! `apply_inner_margin` 분기)을 그대로 재현한다.
//!
//! # 임계 경계(critical edges) — 실측 분류
//!
//! `Budget::dim_rounding_tolerance_hwp` 가 HWPUNIT 반올림 잡음(±1)을 흡수하고,
//! 아래 축은 잡음이 아니라 결함으로 센다. 각 축은 실패 로그에서 이름으로 집계된다.
//!
//! - **A. 인라인 개체(수식/그림) 줄 높이 오배치** — 저장은 키 큰 개체를 L1 에 두는데
//!   재계산은 L0 에 둔다. 인접 줄의 오차 부호가 반전되는 것이 지문이다.
//!   `3-09'23 p131`: `L0 +3375 / L1 -3375`. 미주 세로 드리프트의 직접 원인.
//! - **B. 인라인 개체 문단의 줄 수 발산** — `3-09'22 p76`: 저장 5줄 vs 재계산 3줄.
//! - **C. 줄바꿈 위치(text_start) 발산** — `stored_start=43 → computed 83`.
//!   인라인 개체가 소비하는 폭을 재계산이 반영하지 못한다.
//! - **D. wrap zone 무시(segment_width)** — 저장 `10718` vs 재계산 `26788`(+16070).
//!   저장값은 개체를 피해 좁아진 줄 폭인데 재계산은 항상 단 전체 폭을 쓴다.
//! - **E. 중첩 깊이 발산** — `issue1949` depth1 text_start 36.8% 불일치,
//!   `issue2007` 은 depth3(표 안 표 안 표)에서 `+1001` HWPUNIT 계통 오차.
//! - **F. 좁은 셀의 줄 수 붕괴** — 폭 35~38px 셀에서 저장 2줄 vs 재계산 1줄.
//!
//! 기존 `tests/issue_1082_endnote_multicolumn_drift.rs` 는 렌더 결과(px 초과분)로 같은
//! 결함을 **간접** 측정한다. 이 파일은 LineSeg 필드를 **직접** 대조한다.

use rhwp::model::control::Control;
use rhwp::model::page::ColumnDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::composer::reflow_line_segs;
use rhwp::renderer::page_layout::PageLayoutInfo;
use rhwp::renderer::style_resolver::ResolvedStyleSet;
use std::collections::BTreeMap;
use std::path::Path;

const DPI: f64 = 96.0;

// ─────────────────────────── 예산(허용치) ───────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// 치수(HWPUNIT) 차이가 이 값 이하이면 반올림 잡음으로 보고 세지 않는다.
    pub dim_rounding_tolerance_hwp: i32,
    pub max_line_count_mismatch_pct: f64,
    pub max_text_start_mismatch_pct: f64,
    pub max_line_height_mismatch_pct: f64,
    /// 단일 줄 높이 절대 오차 상한 (HWPUNIT). 인라인 개체 오배치(축 A)를 잡는다.
    pub max_line_height_abs_hwp: i32,
    pub max_segment_width_mismatch_pct: f64,
}

impl Budget {
    /// 저장값과 재계산이 "정합한다"고 말할 수 있는 기준.
    /// HWPUNIT 반올림(±1)만 허용하고 구조적 발산은 허용하지 않는다.
    pub const STRICT: Budget = Budget {
        dim_rounding_tolerance_hwp: 1,
        max_line_count_mismatch_pct: 0.0,
        max_text_start_mismatch_pct: 0.0,
        max_line_height_mismatch_pct: 0.0,
        max_line_height_abs_hwp: 1,
        max_segment_width_mismatch_pct: 0.0,
    };
}

// ─────────────────────────── 측정 결과 ───────────────────────────

#[derive(Default, Clone)]
pub struct DepthStat {
    pub paragraphs: usize,
    pub lines: usize,
    pub line_count_mismatch: usize,
    pub text_start_mismatch: usize,
    pub line_height_mismatch: usize,
    pub segment_width_mismatch: usize,
    /// 이 깊이에서 관측한 셀 폭 범위 (px)
    pub min_width_px: f64,
    pub max_width_px: f64,
}

/// 심각도 점수와 함께 보관하는 대표 위반 사례
struct Offender {
    severity: i64,
    text: String,
}

#[derive(Default)]
pub struct Audit {
    pub file: String,
    pub sections: usize,
    pub paragraphs: usize,
    pub lines: usize,
    pub line_count_mismatch: usize,
    pub text_start_mismatch: usize,
    pub line_height_mismatch: usize,
    pub segment_width_mismatch: usize,
    pub line_height_max_abs: i32,
    pub by_depth: BTreeMap<usize, DepthStat>,
    /// |Δline_height| 크기 버킷: (2..9, 10..99, 100..999, 1000+)
    pub lh_buckets: [usize; 4],
    /// 축 A — 인접 줄 오차 부호 반전(키 큰 인라인 개체 오배치) 문단 수
    pub edge_a_inline_swap: usize,
    /// 축 D — 저장 segment_width 가 가용 폭보다 1000HWPUNIT 이상 좁음(wrap zone) 줄 수
    pub edge_d_wrap_zone: usize,
    /// 축 F — 좁은 셀(<60px)에서의 줄 수 발산 문단 수
    pub edge_f_narrow_cell: usize,
    offenders: Vec<Offender>,
}

impl Audit {
    fn pct(&self, n: usize) -> f64 {
        if self.paragraphs == 0 {
            0.0
        } else {
            n as f64 / self.paragraphs as f64 * 100.0
        }
    }

    pub fn violations(&self, b: Budget) -> Vec<String> {
        let mut v = Vec::new();
        let mut chk = |name: &str, raw: usize, cap: f64, got: f64| {
            if got > cap {
                v.push(format!(
                    "{name:<28} {raw:>5}건 {got:>5.1}% > 상한 {cap:.1}%"
                ));
            }
        };
        chk(
            "줄수(line count) 불일치",
            self.line_count_mismatch,
            b.max_line_count_mismatch_pct,
            self.pct(self.line_count_mismatch),
        );
        chk(
            "줄바꿈위치(text_start) 불일치",
            self.text_start_mismatch,
            b.max_text_start_mismatch_pct,
            self.pct(self.text_start_mismatch),
        );
        chk(
            "줄높이(line_height) 불일치",
            self.line_height_mismatch,
            b.max_line_height_mismatch_pct,
            self.pct(self.line_height_mismatch),
        );
        chk(
            "줄폭(segment_width) 불일치",
            self.segment_width_mismatch,
            b.max_segment_width_mismatch_pct,
            self.pct(self.segment_width_mismatch),
        );
        if self.line_height_max_abs > b.max_line_height_abs_hwp {
            v.push(format!(
                "{:<28} {:>5}   최대오차 HWPUNIT > 상한 {}",
                "줄높이 단일 최대오차", self.line_height_max_abs, b.max_line_height_abs_hwp
            ));
        }
        v
    }

    /// 사람이 읽는 상세 리포트.
    pub fn detail_report(&self, b: Budget) -> String {
        let mut o = String::new();
        let max_depth = self.by_depth.keys().copied().max().unwrap_or(0);
        o.push_str(&format!(
            "  요약: 섹션 {} · 저장 LineSeg 보유 문단 {} · 대조 줄 {} · 최대 중첩깊이 {}\n",
            self.sections, self.paragraphs, self.lines, max_depth
        ));

        o.push_str("  ┌ 깊이별 분포 (깊이 0 = 본문, 1+ = 표/중첩 표 셀)\n");
        for (d, s) in &self.by_depth {
            let p = |n: usize| {
                if s.paragraphs == 0 {
                    0.0
                } else {
                    n as f64 / s.paragraphs as f64 * 100.0
                }
            };
            o.push_str(&format!(
                "  │ d{d}: 문단{:>5} 줄{:>6} | 줄수{:>4}({:>5.1}%) 줄바꿈{:>4}({:>5.1}%) \
                 줄높이{:>4}({:>5.1}%) 줄폭{:>5}({:>5.1}%) | 폭 {:.0}~{:.0}px\n",
                s.paragraphs,
                s.lines,
                s.line_count_mismatch,
                p(s.line_count_mismatch),
                s.text_start_mismatch,
                p(s.text_start_mismatch),
                s.line_height_mismatch,
                p(s.line_height_mismatch),
                s.segment_width_mismatch,
                p(s.segment_width_mismatch),
                s.min_width_px,
                s.max_width_px,
            ));
        }
        o.push_str("  └\n");

        let v = self.violations(b);
        if !v.is_empty() {
            o.push_str("  ┌ 예산 위반 축\n");
            for line in &v {
                o.push_str(&format!("  │ · {line}\n"));
            }
            o.push_str("  └\n");
        }

        if self.lh_buckets.iter().any(|&n| n > 0) {
            o.push_str(&format!(
                "  ┌ 줄높이 오차 크기 분포 (HWPUNIT, 잡음 ±{} 제외)\n  │ |Δ| 2..9: {:>4}  \
                 10..99: {:>4}  100..999: {:>4}  1000+: {:>4}   최대 {}\n  └\n",
                b.dim_rounding_tolerance_hwp,
                self.lh_buckets[0],
                self.lh_buckets[1],
                self.lh_buckets[2],
                self.lh_buckets[3],
                self.line_height_max_abs,
            ));
        }

        if self.edge_a_inline_swap + self.edge_d_wrap_zone + self.edge_f_narrow_cell > 0 {
            o.push_str("  ┌ 임계 경계 분류\n");
            if self.edge_a_inline_swap > 0 {
                o.push_str(&format!(
                    "  │ A. 인라인 개체 줄높이 오배치(인접 줄 부호 반전): {}문단\n",
                    self.edge_a_inline_swap
                ));
            }
            if self.edge_d_wrap_zone > 0 {
                o.push_str(&format!(
                    "  │ D. wrap zone 무시(저장 줄폭이 가용폭보다 1000+ 좁음): {}줄\n",
                    self.edge_d_wrap_zone
                ));
            }
            if self.edge_f_narrow_cell > 0 {
                o.push_str(&format!(
                    "  │ F. 좁은 셀(<60px) 줄수 발산: {}문단\n",
                    self.edge_f_narrow_cell
                ));
            }
            o.push_str("  └\n");
        }

        if !self.offenders.is_empty() {
            let mut sorted: Vec<&Offender> = self.offenders.iter().collect();
            sorted.sort_by(|a, b| b.severity.cmp(&a.severity));
            o.push_str("  ┌ 대표 위반 (심각도순 상위 10)\n");
            for off in sorted.iter().take(10) {
                o.push_str(&format!("  │ {}\n", off.text));
            }
            o.push_str("  └\n");
        }
        o
    }
}

// ─────────────────────────── 순회 ───────────────────────────

fn find_column_def(paragraphs: &[Paragraph], para_idx: usize) -> ColumnDef {
    let mut last = ColumnDef::default();
    for (i, para) in paragraphs.iter().enumerate() {
        if i > para_idx {
            break;
        }
        for ctrl in &para.controls {
            if let Control::ColumnDef(cd) = ctrl {
                last = cd.clone();
            }
        }
    }
    last
}

fn preview(s: &str) -> String {
    s.chars().take(30).collect::<String>().replace('\n', "\\n")
}

struct Ctx<'a> {
    styles: &'a ResolvedStyleSet,
    budget: Budget,
    audit: Audit,
}

fn walk(paras: &[Paragraph], avail_width: f64, depth: usize, tag: &str, ctx: &mut Ctx) {
    for (pi, para) in paras.iter().enumerate() {
        compare_one(para, avail_width, depth, &format!("{tag}/p{pi}"), ctx);

        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                for (ci, cell) in table.cells.iter().enumerate() {
                    let cell_w = rhwp::renderer::hwpunit_to_px(cell.width as i32, DPI);
                    let pad = if cell.apply_inner_margin {
                        cell.padding
                    } else {
                        cell.effective_padding(&table.padding)
                    };
                    let l = rhwp::renderer::hwpunit_to_px(pad.left as i32, DPI);
                    let r = rhwp::renderer::hwpunit_to_px(pad.right as i32, DPI);
                    let inner = (cell_w - l - r).max(1.0);
                    walk(
                        &cell.paragraphs,
                        inner,
                        depth + 1,
                        &format!("{tag}/p{pi}·c{ci}"),
                        ctx,
                    );
                }
            }
        }
    }
}

fn compare_one(para: &Paragraph, avail_width: f64, depth: usize, tag: &str, ctx: &mut Ctx) {
    if para.line_segs.is_empty() || para.text.is_empty() {
        return;
    }
    // 한컴이 실제로 기록한 LineSeg 가 있는 문단만 정답지로 삼는다.
    // (HWPX 등 line_height=0 은 저장 기준값이 없다는 뜻)
    if para.line_segs[0].line_height == 0 {
        return;
    }

    let stored: Vec<LineSeg> = para.line_segs.clone();
    let mut clone = para.clone();
    // ── 캐시 차단 (이 테스트의 핵심) ───────────────────────────────────────
    // `reflow_line_segs()` 는 `para.line_segs.first()` 를 `orig` 템플릿으로 읽어
    // tag·vertical_pos 를 물려받고, 빈 문단 분기에서는 line_spacing·segment_width 까지
    // 저장값에서 복사한다. 문단을 그대로 clone 해서 넘기면 "재계산"이 저장값에 기대게 되어
    // #3211 이 말하는 **비캐시 경로**를 재현하지 못한다.
    // 저장값을 지우고 넘겨야 저장 LineSeg 를 레이아웃 소스로 쓰지 않는 계산이 된다.
    clone.line_segs.clear();
    reflow_line_segs(&mut clone, avail_width, ctx.styles, DPI);
    let computed = clone.line_segs;

    let tol = ctx.budget.dim_rounding_tolerance_hwp;
    ctx.audit.paragraphs += 1;
    {
        let d = ctx.audit.by_depth.entry(depth).or_default();
        if d.paragraphs == 0 {
            d.min_width_px = avail_width;
            d.max_width_px = avail_width;
        } else {
            d.min_width_px = d.min_width_px.min(avail_width);
            d.max_width_px = d.max_width_px.max(avail_width);
        }
        d.paragraphs += 1;
    }

    // ── 줄 수 ──
    if stored.len() != computed.len() {
        ctx.audit.line_count_mismatch += 1;
        ctx.audit
            .by_depth
            .entry(depth)
            .or_default()
            .line_count_mismatch += 1;
        if depth > 0 && avail_width < 60.0 {
            ctx.audit.edge_f_narrow_cell += 1;
        }
        let sev = (stored.len() as i64 - computed.len() as i64).abs() * 10_000;
        ctx.audit.offenders.push(Offender {
            severity: sev,
            text: format!(
                "[줄수] d{depth} {tag}: 저장 {}줄 vs 재계산 {}줄 (가용폭 {avail_width:.1}px) | {:?}",
                stored.len(),
                computed.len(),
                preview(&para.text)
            ),
        });
    }

    let n = stored.len().min(computed.len());
    let (mut ts_bad, mut lh_bad, mut sw_bad) = (false, false, false);
    let mut lh_deltas: Vec<i32> = Vec::with_capacity(n);

    for i in 0..n {
        let s = &stored[i];
        let c = &computed[i];
        ctx.audit.lines += 1;
        ctx.audit.by_depth.entry(depth).or_default().lines += 1;

        // ── 줄바꿈 위치 ──
        if s.text_start != c.text_start {
            ts_bad = true;
            let d = (c.text_start as i64 - s.text_start as i64).abs();
            ctx.audit.offenders.push(Offender {
                severity: d * 100,
                text: format!(
                    "[줄바꿈] d{depth} {tag} L{i}: 저장 start={} vs 재계산 {} (Δ{:+}) | {:?}",
                    s.text_start,
                    c.text_start,
                    c.text_start as i64 - s.text_start as i64,
                    preview(&para.text)
                ),
            });
        }

        // ── 줄 높이 ──
        let dh = c.line_height - s.line_height;
        lh_deltas.push(dh);
        if dh.abs() > tol {
            lh_bad = true;
            if dh.abs() > ctx.audit.line_height_max_abs {
                ctx.audit.line_height_max_abs = dh.abs();
            }
            let b = match dh.abs() {
                0..=9 => 0,
                10..=99 => 1,
                100..=999 => 2,
                _ => 3,
            };
            ctx.audit.lh_buckets[b] += 1;
            ctx.audit.offenders.push(Offender {
                severity: dh.abs() as i64,
                text: format!(
                    "[줄높이] d{depth} {tag} L{i}: 저장 lh={} vs 재계산 {} ({dh:+}) | {:?}",
                    s.line_height,
                    c.line_height,
                    preview(&para.text)
                ),
            });
        }

        // ── 줄 폭 ──
        let dw = c.segment_width - s.segment_width;
        if dw.abs() > tol {
            sw_bad = true;
            // 축 D: 저장 줄폭이 가용 폭보다 확연히 좁다 = 개체를 피한 wrap zone
            let avail_hwp = (avail_width * 7200.0 / DPI) as i32;
            if avail_hwp - s.segment_width > 1000 {
                ctx.audit.edge_d_wrap_zone += 1;
            }
            ctx.audit.offenders.push(Offender {
                severity: dw.abs() as i64 / 4,
                text: format!(
                    "[줄폭] d{depth} {tag} L{i}: 저장 sw={} vs 재계산 {} ({dw:+}) (가용폭 {avail_width:.1}px)",
                    s.segment_width, c.segment_width
                ),
            });
        }
    }

    // ── 축 A: 인접 줄의 줄높이 오차 부호가 반전 = 키 큰 인라인 개체가 잘못된 줄에 얹힘 ──
    let swapped = lh_deltas
        .windows(2)
        .any(|w| w[0].abs() > 100 && w[1].abs() > 100 && (w[0] > 0) != (w[1] > 0));
    if swapped {
        ctx.audit.edge_a_inline_swap += 1;
    }

    let d = ctx.audit.by_depth.entry(depth).or_default();
    if ts_bad {
        ctx.audit.text_start_mismatch += 1;
        d.text_start_mismatch += 1;
    }
    if lh_bad {
        ctx.audit.line_height_mismatch += 1;
        d.line_height_mismatch += 1;
    }
    if sw_bad {
        ctx.audit.segment_width_mismatch += 1;
        d.segment_width_mismatch += 1;
    }
}

// ─────────────────────────── 공개 진입점 ───────────────────────────

/// 문서 하나를 감사한다. 파일이 없으면 `None`(스킵).
pub fn audit_file(rel: &str, budget: Budget) -> Option<Audit> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let data = std::fs::read(&path).ok()?;
    let doc = rhwp::parse_document(&data).unwrap_or_else(|e| panic!("{rel} 파싱 실패: {e:?}"));
    let styles = rhwp::renderer::style_resolver::resolve_styles(&doc.doc_info, DPI);

    let mut ctx = Ctx {
        styles: &styles,
        budget,
        audit: Audit {
            file: rel.to_string(),
            sections: doc.sections.len(),
            ..Default::default()
        },
    };

    for (si, section) in doc.sections.iter().enumerate() {
        let page_def = &section.section_def.page_def;
        for (pi, para) in section.paragraphs.iter().enumerate() {
            let cd = find_column_def(&section.paragraphs, pi);
            let layout = PageLayoutInfo::from_page_def(page_def, &cd, DPI);
            let Some(col) = layout.column_areas.first() else {
                continue;
            };
            let ps = styles.para_styles.get(para.para_shape_id as usize);
            let ml = ps.map(|s| s.margin_left).unwrap_or(0.0);
            let mr = ps.map(|s| s.margin_right).unwrap_or(0.0);
            walk(
                std::slice::from_ref(para),
                col.width - ml - mr,
                0,
                &format!("s{si}/p{pi}"),
                &mut ctx,
            );
        }
    }
    Some(ctx.audit)
}

/// **범용 진입점** — 문서 목록을 받아 비캐시 LineSeg 정합성을 한 번에 검사한다.
/// 통과 문서도 한 줄 요약을 남기고, 위반 문서는 상세 리포트를 붙여 하나의 실패로 보고한다.
pub fn assert_uncached_lineseg_compliance(cohort: &str, files: &[&str], budget: Budget) {
    let mut report = String::new();
    let (mut failed, mut passed, mut skipped) = (0usize, 0usize, 0usize);
    let mut tot_paras = 0usize;
    let mut tot_lines = 0usize;

    for rel in files {
        let Some(a) = audit_file(rel, budget) else {
            skipped += 1;
            report.push_str(&format!("SKIP  {rel}  (파일 없음)\n"));
            continue;
        };
        if a.paragraphs == 0 {
            skipped += 1;
            report.push_str(&format!(
                "SKIP  {rel}  (저장 LineSeg 없음 — HWPX/합성 문단)\n"
            ));
            continue;
        }
        tot_paras += a.paragraphs;
        tot_lines += a.lines;

        let v = a.violations(budget);
        if v.is_empty() {
            passed += 1;
            report.push_str(&format!(
                "PASS  {rel}  문단 {} 줄 {}\n",
                a.paragraphs, a.lines
            ));
            continue;
        }
        failed += 1;
        report.push_str(&format!("\nFAIL  {rel}\n"));
        report.push_str(&a.detail_report(budget));
    }

    let header = format!(
        "\n╔══════════════════════════════════════════════════════════════════════\n\
         ║ #3211 [{cohort}] 저장 HWP LineSeg 대비 비캐시 재계산 정합성\n\
         ║ 위반 {failed} · 통과 {passed} · 스킵 {skipped}  (대조 문단 {tot_paras} / 줄 {tot_lines})\n\
         ╟──────────────────────────────────────────────────────────────────────\n\
         ║ 저장 LineSeg 는 한컴이 파일에 기록한 정답지다. 재계산이 이를 재현하지 못하면\n\
         ║ 저장값을 쓰지 않는 경로(편집 후 재배치 등)에서 레이아웃이 어긋난다.\n\
         ╚══════════════════════════════════════════════════════════════════════\n{report}"
    );
    // 통과해도 커버리지(무엇을 몇 줄 대조했는지)를 남긴다 — `-- --nocapture` 로 확인.
    eprintln!("{header}");

    // 전부 스킵되면 "위반 0" 이 되어 조용히 통과한다. 빈 검사를 통과로 위장하지 않는다.
    assert!(
        passed + failed > 0,
        "\n#3211 [{cohort}] 대조 대상이 하나도 없다 — 파일 {}건이 모두 스킵됐다.\n\
         저장 LineSeg 를 가진 문서가 목록에 있어야 이 테스트가 의미를 갖는다.\n{report}",
        files.len()
    );

    assert!(failed == 0, "{header}");
}

// ─────────────────────────── 문서 코호트 ───────────────────────────

/// #3211 이 지목한 미주 드리프트 문서 — 수식 인라인 개체가 조밀한 시험지.
/// `issue_1082_endnote_multicolumn_drift` 의 두 실패와 같은 문서 집합 + 미주 간격 변형.
const ENDNOTE_DOCS: &[&str] = &[
    "samples/3-09월_교육_통합_2022.hwp",
    "samples/3-09월_교육_통합_2023.hwp",
    "samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp",
    "samples/3-09월_교육_통합_2024-미주사이20.hwp",
    "samples/3-11월_실전_통합_2022.hwp",
];

/// 중첩 표 — 셀 inner 폭이 깊이마다 좁아지는 가혹 조건.
/// `issue1949` 는 거대 셀 + 중첩, `issue2007` 은 깊이 3(표 안의 표 안의 표)까지 들어간다.
const NESTED_TABLE_DOCS: &[&str] = &[
    "samples/issue1949_giant_cell_nested_tables_perf.hwp",
    "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
    "samples/basic/issue1994_behindtext_table_20200830.hwp",
    "samples/hwp_table_test.hwp",
    "samples/셀보호2.hwp",
];

/// LINESEG 전용 fixture — 줄간격/들여쓰기/탭/혼합 크기 등 축을 하나씩 분리한다.
const LINESEG_FIXTURES: &[&str] = &[
    "samples/lseg-01-basic.hwp",
    "samples/lseg-02-mixed.hwp",
    "samples/lseg-03-spacing.hwp",
    "samples/lseg-04-indent.hwp",
    "samples/lseg-05-tab.hwp",
    "samples/lseg-06-multisize.hwp",
];

/// 표가 많은 장문 실문서 — 제안요청서/보고서 계열.
const TABLE_HEAVY_DOCS: &[&str] = &[
    "samples/k-water-rfp-2024.hwp",
    "samples/k-water-rfp.hwp",
    "samples/issue_1133.hwp",
    "samples/kps-ai.hwp",
    "samples/hcar-001.hwp",
];

/// 자주 인용되는 일반 실문서 — 서식/필드/시험지/홍보물 등 성격이 섞인 회귀 표본.
const REAL_WORLD_DOCS: &[&str] = &[
    "samples/복학원서.hwp",
    "samples/field-01.hwp",
    "samples/aift.hwp",
    "samples/20250130-hongbo.hwp",
    "samples/SO-SUEOP.hwp",
    "samples/exam_science.hwp",
    "samples/exam_social.hwp",
    "samples/21_언어_기출_편집가능본.hwp",
    "samples/투명도0-50.hwp",
    "samples/basic/BookReview.hwp",
];

// ─────────────────────────── 테스트 ───────────────────────────

#[test]
fn endnote_docs_uncached_lineseg_matches_stored() {
    assert_uncached_lineseg_compliance("미주/수식 시험지", ENDNOTE_DOCS, Budget::STRICT);
}

#[test]
fn nested_table_docs_uncached_lineseg_matches_stored() {
    assert_uncached_lineseg_compliance("중첩 표", NESTED_TABLE_DOCS, Budget::STRICT);
}

#[test]
fn lineseg_fixtures_uncached_lineseg_matches_stored() {
    assert_uncached_lineseg_compliance("LINESEG 축별 fixture", LINESEG_FIXTURES, Budget::STRICT);
}

#[test]
fn table_heavy_docs_uncached_lineseg_matches_stored() {
    assert_uncached_lineseg_compliance("표 다수 장문 실문서", TABLE_HEAVY_DOCS, Budget::STRICT);
}

#[test]
fn real_world_docs_uncached_lineseg_matches_stored() {
    assert_uncached_lineseg_compliance("일반 실문서", REAL_WORLD_DOCS, Budget::STRICT);
}
