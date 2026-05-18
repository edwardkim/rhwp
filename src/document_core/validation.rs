//! 문서 검증 리포트 — HWPX 비표준 감지 경고 기록.
//!
//! IR과 분리된 별도 구조로 관리하여 IR 순수성 유지.
//! Document 로드 시 자동 생성되며, 사용자에게 고지하고 명시적 선택 시 reflow 적용한다.
//!
//! ## 설계 원칙 (#177 / Discussion #188)
//!
//! 한컴이 비표준 lineseg 를 자체 방어 로직으로 조용히 보정하는 현실이 드러남.
//! rhwp는 이런 숨김을 받아들이지 않음:
//! - 비표준 입력은 **감지하고 사용자에게 고지**
//! - 자동 보정은 **사용자 명시적 선택 후에만**
//! - rhwp 자신도 비표준을 **새로 생산하지 않음**

#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

/// 표 셀 내부 문단을 가리키는 경로.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPath {
    /// 부모 문단의 `controls[]` 내 Table 컨트롤 인덱스
    pub table_ctrl_idx: usize,
    /// 셀 행
    pub row: u16,
    /// 셀 열
    pub col: u16,
    /// 셀 내부 문단 인덱스
    pub inner_para_idx: usize,
}

/// 경고 종류 — 비표준 감지 유형.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarningKind {
    /// lineseg 배열이 비어있음 (텍스트가 있는데도).
    /// 한컴은 reflow 로 보정하지만 rhwp 는 렌더링 시 겹침이 발생할 수 있음.
    LinesegArrayEmpty,
    /// lineseg 가 1개만 있고 `line_height=0` — 명백한 "미계산 상태".
    /// 기존 `needs_line_seg_reflow` 조건과 동일.
    LinesegUncomputed,
    /// 긴 텍스트 문단인데 lineseg 가 1개뿐 — 한컴이 textRun 단위 reflow 로 보정하는
    /// 패턴. 명세상 각 줄마다 lineseg 가 있어야 하나 한컴 일부 버전이 문단 전체를
    /// 1개 lineseg 로 선언한다 (Discussion #188). rhwp 는 1개 lineseg 를 신뢰해
    /// 모든 텍스트를 한 줄에 그려 겹침이 발생.
    ///
    /// 휴리스틱: `text.chars().count() > threshold && text 내 '\n' 없음 && line_segs.len() == 1`
    LinesegTextRunReflow,
}

impl fmt::Display for WarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WarningKind::*;
        match self {
            LinesegArrayEmpty => write!(f, "lineseg 배열이 비어있음"),
            LinesegUncomputed => write!(f, "lineseg 가 미계산 상태 (line_height=0)"),
            LinesegTextRunReflow => write!(f, "lineseg 가 문단당 1개 (한컴 textRun reflow 의존)"),
        }
    }
}

/// 검증 리포트의 한 항목 — 문단 경로 + 경고 종류.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    /// 섹션 인덱스
    pub section_idx: usize,
    /// 문단 인덱스 (섹션 내)
    pub paragraph_idx: usize,
    /// 표 셀 내부 문단일 경우 셀 경로, 본문 문단은 `None`
    pub cell_path: Option<CellPath>,
    /// 경고 종류
    pub kind: WarningKind,
}

/// 문서 검증 리포트.
///
/// `DocumentCore::from_bytes` 시점에 자동 생성되며,
/// `DocumentCore::validation_report()` 로 접근한다.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.warnings.len()
    }

    pub fn push(&mut self, w: ValidationWarning) {
        self.warnings.push(w);
    }

    /// 경고 종류별 개수를 집계한다.
    pub fn summary(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for w in &self.warnings {
            let key = format!("{}", w.kind);
            *map.entry(key).or_insert(0) += 1;
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_default_is_empty() {
        let r = ValidationReport::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn summary_groups_by_kind() {
        let mut r = ValidationReport::new();
        r.push(ValidationWarning {
            section_idx: 0,
            paragraph_idx: 1,
            cell_path: None,
            kind: WarningKind::LinesegArrayEmpty,
        });
        r.push(ValidationWarning {
            section_idx: 0,
            paragraph_idx: 2,
            cell_path: None,
            kind: WarningKind::LinesegUncomputed,
        });
        r.push(ValidationWarning {
            section_idx: 0,
            paragraph_idx: 3,
            cell_path: None,
            kind: WarningKind::LinesegUncomputed,
        });
        let summary = r.summary();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary.get("lineseg 배열이 비어있음").copied(), Some(1));
        assert_eq!(
            summary
                .get("lineseg 가 미계산 상태 (line_height=0)")
                .copied(),
            Some(2)
        );
    }

    #[test]
    fn warning_display_messages() {
        assert_eq!(
            format!("{}", WarningKind::LinesegArrayEmpty),
            "lineseg 배열이 비어있음"
        );
        assert_eq!(
            format!("{}", WarningKind::LinesegUncomputed),
            "lineseg 가 미계산 상태 (line_height=0)"
        );
    }

    #[test]
    fn cell_path_equality() {
        let a = CellPath {
            table_ctrl_idx: 0,
            row: 1,
            col: 2,
            inner_para_idx: 3,
        };
        let b = CellPath {
            table_ctrl_idx: 0,
            row: 1,
            col: 2,
            inner_para_idx: 3,
        };
        assert_eq!(a, b);
    }
}
