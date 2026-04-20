//! HWPX vs HWP IR 차이 자동 추출 도구
//!
//! 두 가지 모드:
//! 1. `diff_hwpx_vs_hwp(hwpx, hwp)` — 같은 콘텐츠의 두 IR 간 영역별 차이
//! 2. `diff_hwpx_vs_serializer_assumptions(hwpx)` — HWPX 단독 IR 이
//!    HWP 직렬화기 가정에 위배되는 영역을 추출
//!
//! 본 모듈은 **읽기 전용**. IR 을 수정하지 않는다.

use crate::model::control::Control;
use crate::model::document::Document;

/// 단일 IR 필드 차이 항목.
#[derive(Debug, Clone, PartialEq)]
pub struct IrFieldDiff {
    /// 영역 이름 — 매핑 명세서의 항목명과 일치 (예: "table.raw_ctrl_data").
    pub area: &'static str,
    /// 사람이 읽을 수 있는 위치 (예: "sec=2,para=45,ctrl=0").
    pub location: String,
    /// HWPX 측 값 (또는 HWPX 출처 IR 의 값).
    pub hwpx_value: String,
    /// HWP 측 값 (또는 HWP 직렬화기 기대값).
    pub hwp_value: String,
}

/// 영역별 카운트 요약.
#[derive(Debug, Default, Clone)]
pub struct DiffSummary {
    pub items: Vec<IrFieldDiff>,
}

impl DiffSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: IrFieldDiff) {
        self.items.push(item);
    }

    /// 영역별 카운트.
    pub fn counts_by_area(&self) -> Vec<(&'static str, usize)> {
        let mut map: std::collections::BTreeMap<&'static str, usize> = std::collections::BTreeMap::new();
        for it in &self.items {
            *map.entry(it.area).or_insert(0) += 1;
        }
        map.into_iter().collect()
    }

    /// 휴먼 리더블 리포트.
    pub fn human_report(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        let _ = writeln!(s, "[IR diff summary] total={}", self.items.len());
        for (area, count) in self.counts_by_area() {
            let _ = writeln!(s, "  {}: {}", area, count);
        }
        // 처음 20개 항목 상세
        let _ = writeln!(s, "[details, first 20]");
        for item in self.items.iter().take(20) {
            let _ = writeln!(
                s,
                "  {} @ {}: hwpx={:?} hwp={:?}",
                item.area, item.location, item.hwpx_value, item.hwp_value
            );
        }
        s
    }
}

/// HWPX IR 단독 검사: HWP 직렬화기 가정에 위배되는 영역을 추출한다.
///
/// 검사 항목 (Stage 1 베이스라인):
/// - `table.raw_ctrl_data` 가 비어있는 표 (직렬화기가 빈 ctrl_data 작성 → 한컴 거부)
/// - `cell.apply_inner_margin == true` 이지만 `raw_list_extra` 에 bit 16 보강 없음
/// - `paragraph.line_segs[i].vertical_pos == 0` 인 비-첫줄 lineseg (페이지 폭주 원인)
/// - `section.raw_stream is Some` 인지 (있으면 직렬화기 빠른 경로, 없으면 동적)
///
/// 본 함수는 **HWPX 출처 가정**으로 호출한다. HWP 출처에 호출해도 동작은 하지만 결과 의미는 다름.
pub fn diff_hwpx_vs_serializer_assumptions(hwpx: &Document) -> DiffSummary {
    let mut summary = DiffSummary::new();

    for (sec_idx, section) in hwpx.sections.iter().enumerate() {
        // section.raw_stream
        if section.raw_stream.is_some() {
            summary.push(IrFieldDiff {
                area: "section.raw_stream",
                location: format!("sec={}", sec_idx),
                hwpx_value: "Some(...)".into(),
                hwp_value: "(직렬화기는 raw_stream 우선 — 빠른 경로)".into(),
            });
        } else {
            // None 이어도 직렬화기 동적 경로로 OK — 진단에는 기록만 하고 위반 아님
            // (구현계획서 §1.0.2: 영역을 늘리지 않고 줄이는 것이 성공)
        }

        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            check_paragraph(sec_idx, para_idx, para, &mut summary, "");
        }
    }

    summary
}

fn check_paragraph(
    sec_idx: usize,
    para_idx: usize,
    para: &crate::model::paragraph::Paragraph,
    summary: &mut DiffSummary,
    path_prefix: &str,
) {
    // lineseg vpos == 0 (첫 줄 외)
    for (li, ls) in para.line_segs.iter().enumerate().skip(1) {
        if ls.vertical_pos == 0 {
            summary.push(IrFieldDiff {
                area: "paragraph.line_seg.vertical_pos",
                location: format!("{}sec={},para={},ls={}", path_prefix, sec_idx, para_idx, li),
                hwpx_value: "0".into(),
                hwp_value: "(절대 좌표 — 직렬화기는 그대로 기록)".into(),
            });
        }
    }

    // 컨트롤 검사 (표 위주)
    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
        if let Control::Table(t) = ctrl {
            if t.raw_ctrl_data.is_empty() {
                summary.push(IrFieldDiff {
                    area: "table.raw_ctrl_data",
                    location: format!(
                        "{}sec={},para={},ctrl={}",
                        path_prefix, sec_idx, para_idx, ctrl_idx
                    ),
                    hwpx_value: "(empty)".into(),
                    hwp_value: "(CommonObjAttr 직렬화 필요)".into(),
                });
            }
            if t.raw_table_record_attr == 0 && (t.attr != 0
                || t.repeat_header
                || !matches!(t.page_break, crate::model::table::TablePageBreak::None)) {
                summary.push(IrFieldDiff {
                    area: "table.raw_table_record_attr",
                    location: format!(
                        "{}sec={},para={},ctrl={}",
                        path_prefix, sec_idx, para_idx, ctrl_idx
                    ),
                    hwpx_value: "0".into(),
                    hwp_value: format!(
                        "(재구성 필요: page_break={:?}, repeat={})",
                        t.page_break, t.repeat_header
                    ),
                });
            }
            // 셀별 검사
            for (cell_idx, cell) in t.cells.iter().enumerate() {
                if cell.apply_inner_margin && !raw_list_extra_has_bit16(&cell.raw_list_extra) {
                    summary.push(IrFieldDiff {
                        area: "cell.list_attr.bit16",
                        location: format!(
                            "{}sec={},para={},ctrl={},cell={}",
                            path_prefix, sec_idx, para_idx, ctrl_idx, cell_idx
                        ),
                        hwpx_value: "apply_inner_margin=true, bit16=0".into(),
                        hwp_value: "bit16=1 (셀 안 여백 지정)".into(),
                    });
                }
                // 셀 내부 문단 재귀
                let cell_prefix = format!(
                    "{}cell[{},{},{},{}]/",
                    path_prefix, sec_idx, para_idx, ctrl_idx, cell_idx
                );
                for (cp_idx, cpara) in cell.paragraphs.iter().enumerate() {
                    check_paragraph(sec_idx, cp_idx, cpara, summary, &cell_prefix);
                }
            }
        }
    }
}

/// `raw_list_extra` 가 bit 16 (apply_inner_margin) 을 표현하는지 — Stage 3 에서 본격화.
/// Stage 1 은 보수적으로 항상 false 반환 (일단 영역 누적이 목적).
fn raw_list_extra_has_bit16(_extra: &[u8]) -> bool {
    false
}

/// 두 IR (HWPX 출처와 HWP 출처) 의 영역별 차이를 비교한다.
///
/// Stage 1: 골격만 — Stage 2 이후 영역별 비교 추가.
pub fn diff_hwpx_vs_hwp(hwpx: &Document, hwp: &Document) -> DiffSummary {
    let mut summary = DiffSummary::new();

    // 섹션 수
    if hwpx.sections.len() != hwp.sections.len() {
        summary.push(IrFieldDiff {
            area: "sections.len",
            location: "doc".into(),
            hwpx_value: format!("{}", hwpx.sections.len()),
            hwp_value: format!("{}", hwp.sections.len()),
        });
    }

    // 영역별 비교는 Stage 2 부터 본격 추가.
    // 현재 단계는 단독 검사 (diff_hwpx_vs_serializer_assumptions) 가 주력.
    let single = diff_hwpx_vs_serializer_assumptions(hwpx);
    for it in single.items {
        summary.push(it);
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_doc_no_diff_items_for_critical_areas() {
        let doc = Document::default();
        let summary = diff_hwpx_vs_serializer_assumptions(&doc);
        // 빈 문서엔 표/lineseg 자체가 없으므로 critical 영역 차이 0
        let counts = summary.counts_by_area();
        assert!(
            counts
                .iter()
                .all(|(a, _)| *a != "table.raw_ctrl_data"
                    && *a != "cell.list_attr.bit16"
                    && *a != "paragraph.line_seg.vertical_pos"),
            "empty doc should have no critical-area diffs, got: {:?}",
            counts
        );
    }

    #[test]
    fn human_report_includes_total() {
        let summary = DiffSummary::new();
        let report = summary.human_report();
        assert!(report.contains("total=0"));
    }
}
