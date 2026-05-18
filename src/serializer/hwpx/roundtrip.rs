//! HWPX 라운드트립 IR diff — `parse → serialize → parse` 한 IR을 원본과 비교.
//!
//! ## 원칙
//!
//! - **바이트 비교 금지**: XML 속성 순서·ZIP 압축율 유동성 때문에 브리틀함
//! - **IR 의미 비교**: Document 공개 필드 단위로 비교
//! - **누적 확장**: Stage 0에선 뼈대 필드(섹션 수·문단 수·리소스 카운트)만 비교하고,
//!   Stage 1~5 진행 시 비교 대상 필드를 누적 확장한다
//!
//! Stage 0 최소 세트:
//! - sections.len()
//! - 각 section의 paragraphs.len()
//! - doc_info의 리소스 카운트 (char_shapes, para_shapes, border_fills 등)
//! - bin_data_content.len()

#![allow(dead_code)]

use crate::model::document::Document;
use crate::parser::hwpx::parse_hwpx;
use crate::serializer::hwpx::serialize_hwpx;
use crate::serializer::SerializeError;

/// IR diff 결과 — 발견된 차이 목록을 보관.
#[derive(Debug, Default)]
pub struct IrDiff {
    pub differences: Vec<IrDifference>,
}

impl IrDiff {
    pub fn is_empty(&self) -> bool {
        self.differences.is_empty()
    }

    pub fn push(&mut self, d: IrDifference) {
        self.differences.push(d);
    }

    /// 관용 규칙 하에서 통과로 볼 수 있는가 (Stage 5에서 확장 예정).
    pub fn allowed(&self, _allow: IrDiffAllow) -> bool {
        self.is_empty()
    }
}

/// Stage 5에서 도형 raw 바이트 불일치 등을 허용하기 위한 옵션 (현재 미사용).
#[derive(Debug, Default, Clone, Copy)]
pub struct IrDiffAllow {
    pub shape_raw: bool,
}

/// 발견된 단일 차이.
#[derive(Debug, Clone)]
pub enum IrDifference {
    SectionCount {
        expected: usize,
        actual: usize,
    },
    ParagraphCount {
        section: usize,
        expected: usize,
        actual: usize,
    },
    CharShapeCount {
        expected: usize,
        actual: usize,
    },
    ParaShapeCount {
        expected: usize,
        actual: usize,
    },
    BorderFillCount {
        expected: usize,
        actual: usize,
    },
    TabDefCount {
        expected: usize,
        actual: usize,
    },
    NumberingCount {
        expected: usize,
        actual: usize,
    },
    StyleCount {
        expected: usize,
        actual: usize,
    },
    BinDataContentCount {
        expected: usize,
        actual: usize,
    },
}

impl std::fmt::Display for IrDifference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IrDifference::*;
        match self {
            SectionCount { expected, actual } => {
                write!(f, "section count: expected={} actual={}", expected, actual)
            }
            ParagraphCount {
                section,
                expected,
                actual,
            } => write!(
                f,
                "section[{}] paragraph count: expected={} actual={}",
                section, expected, actual
            ),
            CharShapeCount { expected, actual } => write!(
                f,
                "char_shapes count: expected={} actual={}",
                expected, actual
            ),
            ParaShapeCount { expected, actual } => write!(
                f,
                "para_shapes count: expected={} actual={}",
                expected, actual
            ),
            BorderFillCount { expected, actual } => write!(
                f,
                "border_fills count: expected={} actual={}",
                expected, actual
            ),
            TabDefCount { expected, actual } => {
                write!(f, "tab_defs count: expected={} actual={}", expected, actual)
            }
            NumberingCount { expected, actual } => write!(
                f,
                "numberings count: expected={} actual={}",
                expected, actual
            ),
            StyleCount { expected, actual } => {
                write!(f, "styles count: expected={} actual={}", expected, actual)
            }
            BinDataContentCount { expected, actual } => write!(
                f,
                "bin_data_content count: expected={} actual={}",
                expected, actual
            ),
        }
    }
}

/// HWPX 바이트 → parse → serialize → parse → 원본 IR과 비교.
pub fn roundtrip_ir_diff(hwpx_bytes: &[u8]) -> Result<IrDiff, SerializeError> {
    let doc1 = parse_hwpx(hwpx_bytes)
        .map_err(|e| SerializeError::XmlError(format!("원본 HWPX 파싱 실패: {}", e)))?;
    let out = serialize_hwpx(&doc1)?;
    let doc2 = parse_hwpx(&out)
        .map_err(|e| SerializeError::XmlError(format!("재직렬화 HWPX 파싱 실패: {}", e)))?;
    Ok(diff_documents(&doc1, &doc2))
}

/// Stage 0 최소 필드 비교.
///
/// Stage 1~5에서 비교 대상 필드를 누적 확장한다 (문단 텍스트, 표·그림 속성 등).
fn diff_documents(a: &Document, b: &Document) -> IrDiff {
    let mut diff = IrDiff::default();

    // 섹션 수
    if a.sections.len() != b.sections.len() {
        diff.push(IrDifference::SectionCount {
            expected: a.sections.len(),
            actual: b.sections.len(),
        });
    }

    // 각 섹션의 문단 수 (섹션 수가 같을 때만 대응 비교)
    let pairs = a.sections.len().min(b.sections.len());
    for i in 0..pairs {
        let ap = a.sections[i].paragraphs.len();
        let bp = b.sections[i].paragraphs.len();
        if ap != bp {
            diff.push(IrDifference::ParagraphCount {
                section: i,
                expected: ap,
                actual: bp,
            });
        }
    }

    // DocInfo 리소스 카운트
    if a.doc_info.char_shapes.len() != b.doc_info.char_shapes.len() {
        diff.push(IrDifference::CharShapeCount {
            expected: a.doc_info.char_shapes.len(),
            actual: b.doc_info.char_shapes.len(),
        });
    }
    if a.doc_info.para_shapes.len() != b.doc_info.para_shapes.len() {
        diff.push(IrDifference::ParaShapeCount {
            expected: a.doc_info.para_shapes.len(),
            actual: b.doc_info.para_shapes.len(),
        });
    }
    if a.doc_info.border_fills.len() != b.doc_info.border_fills.len() {
        diff.push(IrDifference::BorderFillCount {
            expected: a.doc_info.border_fills.len(),
            actual: b.doc_info.border_fills.len(),
        });
    }
    if a.doc_info.tab_defs.len() != b.doc_info.tab_defs.len() {
        diff.push(IrDifference::TabDefCount {
            expected: a.doc_info.tab_defs.len(),
            actual: b.doc_info.tab_defs.len(),
        });
    }
    if a.doc_info.numberings.len() != b.doc_info.numberings.len() {
        diff.push(IrDifference::NumberingCount {
            expected: a.doc_info.numberings.len(),
            actual: b.doc_info.numberings.len(),
        });
    }
    if a.doc_info.styles.len() != b.doc_info.styles.len() {
        diff.push(IrDifference::StyleCount {
            expected: a.doc_info.styles.len(),
            actual: b.doc_info.styles.len(),
        });
    }

    // BinData
    if a.bin_data_content.len() != b.bin_data_content.len() {
        diff.push(IrDifference::BinDataContentCount {
            expected: a.bin_data_content.len(),
            actual: b.bin_data_content.len(),
        });
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ir_diff_empty_default() {
        let diff = IrDiff::default();
        assert!(diff.is_empty());
    }

    #[test]
    fn diff_documents_empty_is_empty() {
        let a = Document::default();
        let b = Document::default();
        let diff = diff_documents(&a, &b);
        assert!(diff.is_empty(), "empty vs empty must have no diff");
    }

    #[test]
    fn diff_documents_detects_section_count() {
        let a = Document::default();
        let mut b = Document::default();
        b.sections.push(Default::default());
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1);
        assert!(matches!(
            diff.differences[0],
            IrDifference::SectionCount {
                expected: 0,
                actual: 1
            }
        ));
    }
}
