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
//!
//! Task #1378 확장:
//! - 본문(top-level) 문단별 `char_shapes` 시퀀스 — `(start_pos, char_shape_id)` 전체 비교.
//!   serializer 의 run 평탄화(첫 run 서식으로 통일)를 검출한다.
//!   셀·글상자(Group 재귀)·각주/미주 내부 문단 재귀 비교 포함 (3단계).
//!
//! Task #1379 확장:
//! - 문단별 인라인 슬롯 컨트롤 타입 시퀀스 비교 (`is_hwpx_inline_slot` 기준, 본문 +
//!   #1378 재귀 동승). 셀·글상자 subList 의 컨트롤 소실(그림 등)을 검출한다.
//!   Bookmark 등 위치 없는 비슬롯 컨트롤은 비교 대상에서 제외.
//!
//! Task #1380 확장:
//! - 문단별 `line_segs` 9필드 비교 (`diff_linesegs`) 를 `ParagraphLinesegs` 로 변환해
//!   게이트 동승 (3단계). 파서 zero-default 주입 제거 + serializer 방출 생략(2단계)
//!   이후의 원본 무 ↔ RT 유 합성 비대칭(개수·값 불일치)을 검출한다.

#![allow(dead_code)]

use super::section::is_hwpx_inline_slot;
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
    /// 문단의 char_shapes 시퀀스 불일치 — run 분할 보존 게이트 (#1378).
    ///
    /// `path` 는 중첩 위치 표기 — 본문 문단은 빈 문자열, 셀·글상자·각주/미주 내부
    /// 문단은 `/ctrl[i]tbl.cell[j].p[k]` 식의 경로.
    ParagraphCharShapes {
        section: usize,
        paragraph: usize,
        path: String,
        expected: String,
        actual: String,
    },
    /// 문단의 인라인 슬롯 컨트롤 타입 시퀀스 불일치 — 컨트롤 보존 게이트 (#1379).
    ///
    /// `path` 표기는 `ParagraphCharShapes` 와 동일.
    ParagraphControls {
        section: usize,
        paragraph: usize,
        path: String,
        expected: String,
        actual: String,
    },
    /// 문단의 `line_segs` 불일치 — lineseg 원본 보존 게이트 (#1380).
    ///
    /// `path` 표기는 `ParagraphCharShapes` 와 동일. `detail` 은 `LinesegDiffKind` 의
    /// 표시 문자열 (개수 불일치 또는 인덱스·필드 단위 값 불일치).
    ParagraphLinesegs {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
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
            ParagraphCharShapes {
                section,
                paragraph,
                path,
                expected,
                actual,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} char_shapes: expected={} actual={}",
                section, paragraph, path, expected, actual
            ),
            ParagraphControls {
                section,
                paragraph,
                path,
                expected,
                actual,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} controls: expected={} actual={}",
                section, paragraph, path, expected, actual
            ),
            ParagraphLinesegs {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} linesegs: {}",
                section, paragraph, path, detail
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
/// `hwpx-roundtrip` 배치 진단(Task #1315)에서도 사용한다.
pub fn diff_documents(a: &Document, b: &Document) -> IrDiff {
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

        // 문단별 char_shapes 시퀀스 비교 (#1378) — run 분할 보존 게이트.
        // 본문 + 셀(Table)·글상자(Shape/TextBox)·각주/미주 내부 문단 재귀 (3단계 확장).
        let pp = ap.min(bp);
        for j in 0..pp {
            diff_paragraph_char_shapes(
                &mut diff,
                i,
                j,
                "",
                &a.sections[i].paragraphs[j],
                &b.sections[i].paragraphs[j],
            );
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

    // 문단별 line_segs 비교 (#1380) — lineseg 원본 보존 게이트 (3단계 동승).
    // 순회 경로는 diff_paragraph_char_shapes 와 동일 (본문 + 셀·글상자·각주/미주 재귀).
    for d in diff_linesegs(a, b) {
        diff.push(IrDifference::ParagraphLinesegs {
            section: d.section,
            paragraph: d.paragraph,
            path: d.path,
            detail: d.kind.to_string(),
        });
    }

    diff
}

/// 문단별 lineseg 비교 결과 1건 (Task #1380).
///
/// `diff_documents` 가 `IrDifference::ParagraphLinesegs` 로 변환해 게이트에 동승하고
/// (3단계), `hwpx-roundtrip` 배치 진단의 필드 단위 TSV 측정에도 직접 사용한다.
///
/// `path` 표기는 `IrDifference::ParagraphCharShapes` 와 동일
/// (본문 문단은 빈 문자열, 중첩 문단은 `/ctrl[i]tbl.cell[j].p[k]` 식).
#[derive(Debug, Clone)]
pub struct LinesegDiff {
    pub section: usize,
    pub paragraph: usize,
    pub path: String,
    pub kind: LinesegDiffKind,
}

/// lineseg 불일치 종류.
#[derive(Debug, Clone)]
pub enum LinesegDiffKind {
    /// 문단의 lineseg 개수 불일치.
    CountMismatch { expected: usize, actual: usize },
    /// 같은 인덱스 lineseg 의 필드 값 불일치. `field` 는 HWPX 속성명
    /// (textpos/vertpos/vertsize/textheight/baseline/spacing/horzpos/horzsize/flags).
    ValueMismatch {
        index: usize,
        field: &'static str,
        expected: i64,
        actual: i64,
    },
}

impl std::fmt::Display for LinesegDiffKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinesegDiffKind::CountMismatch { expected, actual } => {
                write!(f, "count: expected={} actual={}", expected, actual)
            }
            LinesegDiffKind::ValueMismatch {
                index,
                field,
                expected,
                actual,
            } => write!(
                f,
                "[{}].{}: expected={} actual={}",
                index, field, expected, actual
            ),
        }
    }
}

/// 문서 전체의 문단별 `line_segs` 를 비교한다 (Task #1380).
///
/// 1단계에서는 측정 전용이었고, 3단계부터 `diff_documents` 가 이 결과를
/// `ParagraphLinesegs` 로 변환해 baseline 게이트에 동승한다.
/// 순회 경로는 `diff_paragraph_char_shapes` 와 동일 (본문 + 셀·글상자(Group 재귀)·
/// 각주/미주). 개수 불일치 시에도 공통 구간(min)은 값 비교를 계속한다.
pub fn diff_linesegs(a: &Document, b: &Document) -> Vec<LinesegDiff> {
    let mut out = Vec::new();
    let pairs = a.sections.len().min(b.sections.len());
    for i in 0..pairs {
        let pp = a.sections[i]
            .paragraphs
            .len()
            .min(b.sections[i].paragraphs.len());
        for j in 0..pp {
            diff_paragraph_linesegs(
                &mut out,
                i,
                j,
                "",
                &a.sections[i].paragraphs[j],
                &b.sections[i].paragraphs[j],
            );
        }
    }
    out
}

/// 문단 1쌍의 lineseg 비교 + 컨트롤 내부 문단 재귀 (`diff_paragraph_char_shapes` 와
/// 동일 경로 순회).
fn diff_paragraph_linesegs(
    out: &mut Vec<LinesegDiff>,
    section: usize,
    paragraph: usize,
    path: &str,
    pa: &crate::model::paragraph::Paragraph,
    pb: &crate::model::paragraph::Paragraph,
) {
    use crate::model::control::Control;

    let la = &pa.line_segs;
    let lb = &pb.line_segs;
    if la.len() != lb.len() {
        out.push(LinesegDiff {
            section,
            paragraph,
            path: path.to_string(),
            kind: LinesegDiffKind::CountMismatch {
                expected: la.len(),
                actual: lb.len(),
            },
        });
    }
    for (idx, (sa, sb)) in la.iter().zip(lb.iter()).enumerate() {
        let fields: [(&'static str, i64, i64); 9] = [
            ("textpos", sa.text_start as i64, sb.text_start as i64),
            ("vertpos", sa.vertical_pos as i64, sb.vertical_pos as i64),
            ("vertsize", sa.line_height as i64, sb.line_height as i64),
            ("textheight", sa.text_height as i64, sb.text_height as i64),
            (
                "baseline",
                sa.baseline_distance as i64,
                sb.baseline_distance as i64,
            ),
            ("spacing", sa.line_spacing as i64, sb.line_spacing as i64),
            ("horzpos", sa.column_start as i64, sb.column_start as i64),
            ("horzsize", sa.segment_width as i64, sb.segment_width as i64),
            ("flags", sa.tag as i64, sb.tag as i64),
        ];
        for (field, ea, eb) in fields {
            if ea != eb {
                out.push(LinesegDiff {
                    section,
                    paragraph,
                    path: path.to_string(),
                    kind: LinesegDiffKind::ValueMismatch {
                        index: idx,
                        field,
                        expected: ea,
                        actual: eb,
                    },
                });
            }
        }
    }

    for (ci, (ctrl_a, ctrl_b)) in pa.controls.iter().zip(pb.controls.iter()).enumerate() {
        match (ctrl_a, ctrl_b) {
            (Control::Table(ta), Control::Table(tb)) => {
                for (cell_i, (cea, ceb)) in ta.cells.iter().zip(tb.cells.iter()).enumerate() {
                    for (k, (qa, qb)) in
                        cea.paragraphs.iter().zip(ceb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]tbl.cell[{cell_i}].p[{k}]");
                        diff_paragraph_linesegs(out, section, paragraph, &p, qa, qb);
                    }
                }
            }
            (Control::Shape(sa), Control::Shape(sb)) => {
                let p = format!("{path}/ctrl[{ci}]shape");
                diff_shape_linesegs(out, section, paragraph, &p, sa, sb);
            }
            (Control::Footnote(na), Control::Footnote(nb)) => {
                for (k, (qa, qb)) in na.paragraphs.iter().zip(nb.paragraphs.iter()).enumerate() {
                    let p = format!("{path}/ctrl[{ci}]fn.p[{k}]");
                    diff_paragraph_linesegs(out, section, paragraph, &p, qa, qb);
                }
            }
            (Control::Endnote(na), Control::Endnote(nb)) => {
                for (k, (qa, qb)) in na.paragraphs.iter().zip(nb.paragraphs.iter()).enumerate() {
                    let p = format!("{path}/ctrl[{ci}]en.p[{k}]");
                    diff_paragraph_linesegs(out, section, paragraph, &p, qa, qb);
                }
            }
            _ => {}
        }
    }
}

/// 도형 내부 글상자(TextBox) 문단 lineseg 재귀 비교 — Group 은 자식 도형까지 재귀.
fn diff_shape_linesegs(
    out: &mut Vec<LinesegDiff>,
    section: usize,
    paragraph: usize,
    path: &str,
    sa: &crate::model::shape::ShapeObject,
    sb: &crate::model::shape::ShapeObject,
) {
    use crate::model::shape::ShapeObject;
    if let (Some(ta), Some(tb)) = (shape_text_box(sa), shape_text_box(sb)) {
        for (k, (qa, qb)) in ta.paragraphs.iter().zip(tb.paragraphs.iter()).enumerate() {
            let p = format!("{path}.tb.p[{k}]");
            diff_paragraph_linesegs(out, section, paragraph, &p, qa, qb);
        }
    }
    if let (ShapeObject::Group(ga), ShapeObject::Group(gb)) = (sa, sb) {
        for (k, (c1, c2)) in ga.children.iter().zip(gb.children.iter()).enumerate() {
            let p = format!("{path}.child[{k}]");
            diff_shape_linesegs(out, section, paragraph, &p, c1, c2);
        }
    }
}

/// 문단 char_shapes 시퀀스(#1378)와 인라인 슬롯 컨트롤 타입 시퀀스(#1379)를 비교하고,
/// 컨트롤 내부 문단(셀·글상자·각주/미주)을 재귀 비교한다.
///
/// 컨트롤 쌍의 재귀는 인덱스 대응(zip)으로만 내려간다 — 수·타입 불일치는
/// `ParagraphControls` 가 해당 문단 수준에서 검출한다.
fn diff_paragraph_char_shapes(
    diff: &mut IrDiff,
    section: usize,
    paragraph: usize,
    path: &str,
    pa: &crate::model::paragraph::Paragraph,
    pb: &crate::model::paragraph::Paragraph,
) {
    use crate::model::control::Control;

    let ca = &pa.char_shapes;
    let cb = &pb.char_shapes;
    let same = ca.len() == cb.len()
        && ca
            .iter()
            .zip(cb.iter())
            .all(|(x, y)| x.start_pos == y.start_pos && x.char_shape_id == y.char_shape_id);
    if !same {
        diff.push(IrDifference::ParagraphCharShapes {
            section,
            paragraph,
            path: path.to_string(),
            expected: format_char_shapes(ca),
            actual: format_char_shapes(cb),
        });
    }

    // 인라인 슬롯 컨트롤 타입 시퀀스 비교 (#1379) — subList 컨트롤 소실 검출.
    // Bookmark 등 위치 정보가 없는 비슬롯 컨트롤은 제외 (serializer 가 문단 선두로
    // 재배치하므로 순서 비교가 성립하지 않음).
    let sa: Vec<&Control> = pa
        .controls
        .iter()
        .filter(|c| is_hwpx_inline_slot(c))
        .collect();
    let sb: Vec<&Control> = pb
        .controls
        .iter()
        .filter(|c| is_hwpx_inline_slot(c))
        .collect();
    let ctrl_same = sa.len() == sb.len()
        && sa
            .iter()
            .zip(sb.iter())
            .all(|(x, y)| control_type_name(x) == control_type_name(y));
    if !ctrl_same {
        diff.push(IrDifference::ParagraphControls {
            section,
            paragraph,
            path: path.to_string(),
            expected: format_control_types(&sa),
            actual: format_control_types(&sb),
        });
    }
    for (ci, (ctrl_a, ctrl_b)) in pa.controls.iter().zip(pb.controls.iter()).enumerate() {
        match (ctrl_a, ctrl_b) {
            (Control::Table(ta), Control::Table(tb)) => {
                for (cell_i, (cea, ceb)) in ta.cells.iter().zip(tb.cells.iter()).enumerate() {
                    for (k, (qa, qb)) in
                        cea.paragraphs.iter().zip(ceb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]tbl.cell[{cell_i}].p[{k}]");
                        diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                    }
                }
            }
            (Control::Shape(sa), Control::Shape(sb)) => {
                let p = format!("{path}/ctrl[{ci}]shape");
                diff_shape_char_shapes(diff, section, paragraph, &p, sa, sb);
            }
            (Control::Footnote(na), Control::Footnote(nb)) => {
                for (k, (qa, qb)) in na.paragraphs.iter().zip(nb.paragraphs.iter()).enumerate() {
                    let p = format!("{path}/ctrl[{ci}]fn.p[{k}]");
                    diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                }
            }
            (Control::Endnote(na), Control::Endnote(nb)) => {
                for (k, (qa, qb)) in na.paragraphs.iter().zip(nb.paragraphs.iter()).enumerate() {
                    let p = format!("{path}/ctrl[{ci}]en.p[{k}]");
                    diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                }
            }
            _ => {}
        }
    }
}

/// 도형 내부 글상자(TextBox) 문단 재귀 비교 — Group 은 자식 도형까지 재귀.
fn diff_shape_char_shapes(
    diff: &mut IrDiff,
    section: usize,
    paragraph: usize,
    path: &str,
    sa: &crate::model::shape::ShapeObject,
    sb: &crate::model::shape::ShapeObject,
) {
    use crate::model::shape::ShapeObject;
    if let (Some(ta), Some(tb)) = (shape_text_box(sa), shape_text_box(sb)) {
        for (k, (qa, qb)) in ta.paragraphs.iter().zip(tb.paragraphs.iter()).enumerate() {
            let p = format!("{path}.tb.p[{k}]");
            diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
        }
    }
    if let (ShapeObject::Group(ga), ShapeObject::Group(gb)) = (sa, sb) {
        for (k, (c1, c2)) in ga.children.iter().zip(gb.children.iter()).enumerate() {
            let p = format!("{path}.child[{k}]");
            diff_shape_char_shapes(diff, section, paragraph, &p, c1, c2);
        }
    }
}

/// ShapeObject 에서 글상자(TextBox) 참조를 꺼낸다 (없으면 None).
fn shape_text_box(s: &crate::model::shape::ShapeObject) -> Option<&crate::model::shape::TextBox> {
    use crate::model::shape::ShapeObject::*;
    match s {
        Line(x) => x.drawing.text_box.as_ref(),
        Rectangle(x) => x.drawing.text_box.as_ref(),
        Ellipse(x) => x.drawing.text_box.as_ref(),
        Arc(x) => x.drawing.text_box.as_ref(),
        Polygon(x) => x.drawing.text_box.as_ref(),
        Curve(x) => x.drawing.text_box.as_ref(),
        Chart(x) => x.drawing.text_box.as_ref(),
        Ole(x) => x.drawing.text_box.as_ref(),
        Group(_) | Picture(_) => None,
    }
}

/// 컨트롤 타입 표기 — diff 메시지·시퀀스 비교용 (`render_control_slot` 디스패치 대상).
fn control_type_name(c: &crate::model::control::Control) -> &'static str {
    use crate::model::control::Control::*;
    match c {
        Table(_) => "tbl",
        Picture(_) => "pic",
        Shape(_) => "shape",
        Equation(_) => "eq",
        Footnote(_) => "fn",
        Endnote(_) => "en",
        Field(_) => "field",
        Form(_) => "form",
        Header(_) => "header",
        Footer(_) => "footer",
        AutoNumber(_) => "autoNum",
        PageHide(_) => "pageHide",
        PageNumberPos(_) => "pageNumPos",
        NewNumber(_) => "newNum",
        CharOverlap(_) => "charOverlap",
        Ruby(_) => "ruby",
        _ => "other",
    }
}

/// 컨트롤 타입 시퀀스를 `[tbl,pic, ...]` 형태로 표기 (diff 메시지용).
fn format_control_types(controls: &[&crate::model::control::Control]) -> String {
    let inner = controls
        .iter()
        .map(|c| control_type_name(c))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", inner)
}

/// char_shapes 시퀀스를 `[(start_pos,id), ...]` 형태로 표기 (diff 메시지용).
fn format_char_shapes(refs: &[crate::model::paragraph::CharShapeRef]) -> String {
    let inner = refs
        .iter()
        .map(|r| format!("({},{})", r.start_pos, r.char_shape_id))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}]", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{CharShapeRef, Paragraph};

    /// char_shapes 시퀀스를 가진 단일 문단 Document 생성.
    fn doc_with_char_shapes(refs: &[(u32, u32)]) -> Document {
        let mut para = Paragraph::default();
        para.char_shapes = refs
            .iter()
            .map(|&(start_pos, char_shape_id)| CharShapeRef {
                start_pos,
                char_shape_id,
            })
            .collect();
        let mut doc = Document::default();
        let mut section: crate::model::document::Section = Default::default();
        section.paragraphs.push(para);
        doc.sections.push(section);
        doc
    }

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
    fn diff_documents_same_char_shapes_is_empty() {
        let a = doc_with_char_shapes(&[(0, 5), (3, 2), (10, 5)]);
        let b = doc_with_char_shapes(&[(0, 5), (3, 2), (10, 5)]);
        assert!(diff_documents(&a, &b).is_empty());
    }

    #[test]
    fn diff_documents_detects_flattened_char_shapes() {
        // run 평탄화: 다중 char_shapes → 첫 entry 만 출력된 경우를 검출해야 한다.
        let a = doc_with_char_shapes(&[(0, 5), (3, 2)]);
        let b = doc_with_char_shapes(&[(0, 5)]);
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1);
        match &diff.differences[0] {
            IrDifference::ParagraphCharShapes {
                section,
                paragraph,
                path,
                expected,
                actual,
            } => {
                assert_eq!(*section, 0);
                assert_eq!(*paragraph, 0);
                assert_eq!(path, "");
                assert_eq!(expected, "[(0,5),(3,2)]");
                assert_eq!(actual, "[(0,5)]");
            }
            other => panic!("ParagraphCharShapes 여야 함: {:?}", other),
        }
    }

    #[test]
    fn diff_documents_detects_char_shape_pos_change() {
        // 같은 id 라도 start_pos 가 어긋나면 차이로 검출.
        let a = doc_with_char_shapes(&[(0, 5), (3, 2)]);
        let b = doc_with_char_shapes(&[(0, 5), (4, 2)]);
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1);
        assert!(matches!(
            diff.differences[0],
            IrDifference::ParagraphCharShapes { .. }
        ));
    }

    /// `(start_pos, char_shape_id)` 목록 → CharShapeRef 목록.
    fn to_refs(refs: &[(u32, u32)]) -> Vec<CharShapeRef> {
        refs.iter()
            .map(|&(start_pos, char_shape_id)| CharShapeRef {
                start_pos,
                char_shape_id,
            })
            .collect()
    }

    /// 본문 문단 1개 + 컨트롤 1개를 가진 Document 생성.
    fn doc_with_control(ctrl: crate::model::control::Control) -> Document {
        let mut para = Paragraph::default();
        para.controls.push(ctrl);
        let mut doc = Document::default();
        let mut section: crate::model::document::Section = Default::default();
        section.paragraphs.push(para);
        doc.sections.push(section);
        doc
    }

    /// 1x1 표 컨트롤 — 셀 문단의 char_shapes 를 지정.
    fn table_control(cell_refs: &[(u32, u32)]) -> crate::model::control::Control {
        use crate::model::table::{Cell, Table};
        let mut cell_para = Paragraph::default();
        cell_para.char_shapes = to_refs(cell_refs);
        let mut cell = Cell::default();
        cell.col_span = 1;
        cell.row_span = 1;
        cell.paragraphs.push(cell_para);
        let mut t = Table::default();
        t.row_count = 1;
        t.col_count = 1;
        t.cells.push(cell);
        t.rebuild_grid();
        crate::model::control::Control::Table(Box::new(t))
    }

    /// 글상자(Rectangle drawText) 컨트롤 — 문단의 char_shapes 를 지정.
    fn textbox_control(refs: &[(u32, u32)]) -> crate::model::control::Control {
        use crate::model::shape::{RectangleShape, ShapeObject, TextBox};
        let mut p = Paragraph::default();
        p.char_shapes = to_refs(refs);
        let mut tb = TextBox::default();
        tb.paragraphs.push(p);
        let mut rect = RectangleShape::default();
        rect.drawing.text_box = Some(tb);
        crate::model::control::Control::Shape(Box::new(ShapeObject::Rectangle(rect)))
    }

    /// 각주 컨트롤 — 문단의 char_shapes 를 지정.
    fn footnote_control(refs: &[(u32, u32)]) -> crate::model::control::Control {
        let mut p = Paragraph::default();
        p.char_shapes = to_refs(refs);
        let mut note = crate::model::footnote::Footnote::default();
        note.paragraphs.push(p);
        crate::model::control::Control::Footnote(Box::new(note))
    }

    /// 단일 ParagraphCharShapes 차이의 path 를 단언.
    fn assert_single_char_shapes_diff(diff: &IrDiff, expected_path: &str) {
        assert_eq!(diff.differences.len(), 1, "차이 1건이어야 함: {:?}", diff);
        match &diff.differences[0] {
            IrDifference::ParagraphCharShapes { path, .. } => {
                assert_eq!(path, expected_path);
            }
            other => panic!("ParagraphCharShapes 여야 함: {:?}", other),
        }
    }

    #[test]
    fn diff_documents_detects_cell_char_shapes() {
        // 셀 내부 문단 평탄화 검출 (#1378 3단계 게이트 재귀 확장).
        let a = doc_with_control(table_control(&[(0, 1), (3, 2)]));
        let b = doc_with_control(table_control(&[(0, 1)]));
        assert_single_char_shapes_diff(&diff_documents(&a, &b), "/ctrl[0]tbl.cell[0].p[0]");
    }

    #[test]
    fn diff_documents_same_cell_char_shapes_is_empty() {
        let a = doc_with_control(table_control(&[(0, 1), (3, 2)]));
        let b = doc_with_control(table_control(&[(0, 1), (3, 2)]));
        assert!(diff_documents(&a, &b).is_empty());
    }

    #[test]
    fn diff_documents_detects_textbox_char_shapes() {
        // 글상자 내부 문단 평탄화 검출.
        let a = doc_with_control(textbox_control(&[(0, 1), (3, 2)]));
        let b = doc_with_control(textbox_control(&[(0, 1)]));
        assert_single_char_shapes_diff(&diff_documents(&a, &b), "/ctrl[0]shape.tb.p[0]");
    }

    #[test]
    fn diff_documents_detects_footnote_char_shapes() {
        // 각주 내부 문단 평탄화 검출.
        let a = doc_with_control(footnote_control(&[(0, 1), (3, 2)]));
        let b = doc_with_control(footnote_control(&[(0, 1)]));
        assert_single_char_shapes_diff(&diff_documents(&a, &b), "/ctrl[0]fn.p[0]");
    }

    /// serialize → parse 왕복용 본문 구성: p0(빈 첫 문단) + p1(컨트롤 1개, slot 정합).
    fn roundtrip_doc_with_control(ctrl: crate::model::control::Control) -> Document {
        use crate::model::style::CharShape;
        let p0 = Paragraph::default();
        let mut p1 = Paragraph::default();
        p1.char_count = 9; // 슬롯 1개(8) + 1 — inferred_control_slot_count 정합
        p1.char_shapes = to_refs(&[(0, 1)]);
        p1.controls.push(ctrl);
        let mut doc = Document::default();
        doc.doc_info.char_shapes = vec![
            CharShape::default(),
            CharShape::default(),
            CharShape::default(),
        ];
        // 셀 경로는 para_shape/style id 도 reference 하므로 0번을 등록해 둔다.
        doc.doc_info.para_shapes = vec![Default::default()];
        doc.doc_info.styles = vec![Default::default()];
        let mut section: crate::model::document::Section = Default::default();
        section.paragraphs.push(p0);
        section.paragraphs.push(p1);
        doc.sections.push(section);
        doc
    }

    fn shapes_of(p: &Paragraph) -> Vec<(u32, u32)> {
        p.char_shapes
            .iter()
            .map(|r| (r.start_pos, r.char_shape_id))
            .collect()
    }

    #[test]
    fn serialize_parse_roundtrip_preserves_cell_char_shapes() {
        // 셀 다중 run 의 serialize → parse 왕복 보존 (#1378 3단계).
        let mut doc = roundtrip_doc_with_control(table_control(&[(0, 1), (2, 2)]));
        if let crate::model::control::Control::Table(t) =
            &mut doc.sections[0].paragraphs[1].controls[0]
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "abcd".to_string();
            para.char_offsets = vec![0, 1, 2, 3];
            para.char_count = 5;
        } else {
            panic!("Table 컨트롤이어야 함");
        }

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = parse_hwpx(&bytes).expect("parse");
        let cell_para = match &doc2.sections[0].paragraphs[1].controls[0] {
            crate::model::control::Control::Table(t) => &t.cells[0].paragraphs[0],
            other => panic!("Table 컨트롤이어야 함: {:?}", other),
        };
        assert_eq!(shapes_of(cell_para), vec![(0, 1), (2, 2)], "셀 문단");
    }

    #[test]
    fn serialize_parse_roundtrip_preserves_textbox_char_shapes() {
        // 글상자 다중 run 의 serialize → parse 왕복 보존 (#1378 3단계).
        let mut doc = roundtrip_doc_with_control(textbox_control(&[(0, 1), (2, 2)]));
        if let crate::model::control::Control::Shape(s) =
            &mut doc.sections[0].paragraphs[1].controls[0]
        {
            if let crate::model::shape::ShapeObject::Rectangle(r) = s.as_mut() {
                let para = &mut r.drawing.text_box.as_mut().unwrap().paragraphs[0];
                para.text = "abcd".to_string();
                para.char_offsets = vec![0, 1, 2, 3];
                para.char_count = 5;
            } else {
                panic!("Rectangle 이어야 함");
            }
        } else {
            panic!("Shape 컨트롤이어야 함");
        }

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = parse_hwpx(&bytes).expect("parse");
        let tb_para = match &doc2.sections[0].paragraphs[1].controls[0] {
            crate::model::control::Control::Shape(s) => match s.as_ref() {
                crate::model::shape::ShapeObject::Rectangle(r) => {
                    &r.drawing.text_box.as_ref().expect("text_box").paragraphs[0]
                }
                other => panic!("Rectangle 이어야 함: {:?}", other),
            },
            other => panic!("Shape 컨트롤이어야 함: {:?}", other),
        };
        assert_eq!(shapes_of(tb_para), vec![(0, 1), (2, 2)], "글상자 문단");
    }

    /// 셀 문단에 지정한 컨트롤을 가진 1x1 표 컨트롤 (#1379 게이트 테스트용).
    fn table_control_with_cell_controls(
        ctrls: Vec<crate::model::control::Control>,
    ) -> crate::model::control::Control {
        use crate::model::table::{Cell, Table};
        let mut cell_para = Paragraph::default();
        cell_para.controls = ctrls;
        let mut cell = Cell::default();
        cell.col_span = 1;
        cell.row_span = 1;
        cell.paragraphs.push(cell_para);
        let mut t = Table::default();
        t.row_count = 1;
        t.col_count = 1;
        t.cells.push(cell);
        t.rebuild_grid();
        crate::model::control::Control::Table(Box::new(t))
    }

    fn picture_control() -> crate::model::control::Control {
        crate::model::control::Control::Picture(Box::default())
    }

    /// 단일 ParagraphControls 차이의 path/expected/actual 을 단언.
    fn assert_single_controls_diff(diff: &IrDiff, path0: &str, exp: &str, act: &str) {
        assert_eq!(diff.differences.len(), 1, "차이 1건이어야 함: {:?}", diff);
        match &diff.differences[0] {
            IrDifference::ParagraphControls {
                path,
                expected,
                actual,
                ..
            } => {
                assert_eq!(path, path0);
                assert_eq!(expected, exp);
                assert_eq!(actual, act);
            }
            other => panic!("ParagraphControls 여야 함: {:?}", other),
        }
    }

    #[test]
    fn diff_documents_detects_body_control_loss() {
        // 본문 문단의 인라인 슬롯 컨트롤 소실 검출 (#1379).
        let a = doc_with_control(picture_control());
        let mut b = Document::default();
        let mut section: crate::model::document::Section = Default::default();
        section.paragraphs.push(Paragraph::default());
        b.sections.push(section);
        assert_single_controls_diff(&diff_documents(&a, &b), "", "[pic]", "[]");
    }

    #[test]
    fn diff_documents_detects_cell_control_loss() {
        // 셀 내부 문단의 컨트롤 소실 검출 (#1379) — subList 컨트롤 미출력 양상.
        let a = doc_with_control(table_control_with_cell_controls(vec![picture_control()]));
        let b = doc_with_control(table_control_with_cell_controls(vec![]));
        assert_single_controls_diff(
            &diff_documents(&a, &b),
            "/ctrl[0]tbl.cell[0].p[0]",
            "[pic]",
            "[]",
        );
    }

    #[test]
    fn diff_documents_detects_control_type_change() {
        // 수는 같아도 타입이 다르면 검출.
        let a = doc_with_control(table_control_with_cell_controls(vec![picture_control()]));
        let b = doc_with_control(table_control_with_cell_controls(vec![
            crate::model::control::Control::Equation(Box::default()),
        ]));
        assert_single_controls_diff(
            &diff_documents(&a, &b),
            "/ctrl[0]tbl.cell[0].p[0]",
            "[pic]",
            "[eq]",
        );
    }

    #[test]
    fn diff_documents_bookmark_not_compared_as_control() {
        // Bookmark 는 비슬롯 — serializer 가 문단 선두로 재배치하므로 비교 제외.
        let mut a = doc_with_control(crate::model::control::Control::Bookmark(
            crate::model::control::Bookmark {
                name: "b".to_string(),
            },
        ));
        a.sections[0].paragraphs[0].controls.push(picture_control());
        let b = doc_with_control(picture_control());
        assert!(diff_documents(&a, &b).is_empty());
    }

    #[test]
    fn diff_documents_same_cell_controls_is_empty() {
        let a = doc_with_control(table_control_with_cell_controls(vec![picture_control()]));
        let b = doc_with_control(table_control_with_cell_controls(vec![picture_control()]));
        assert!(diff_documents(&a, &b).is_empty());
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

    // ---------- #1380: diff_linesegs (게이트 동승 + 배치 TSV 측정) ----------

    use crate::model::paragraph::LineSeg;

    /// 지정 lineseg 들을 가진 단일 문단 Document 생성.
    fn doc_with_linesegs(segs: Vec<LineSeg>) -> Document {
        let mut para = Paragraph::default();
        para.line_segs = segs;
        let mut doc = Document::default();
        let mut section: crate::model::document::Section = Default::default();
        section.paragraphs.push(para);
        doc.sections.push(section);
        doc
    }

    fn seg(vertical_pos: i32, line_height: i32) -> LineSeg {
        LineSeg {
            vertical_pos,
            line_height,
            text_height: 1000,
            baseline_distance: 850,
            line_spacing: 600,
            segment_width: 42520,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }
    }

    #[test]
    fn diff_linesegs_equal_is_empty() {
        let a = doc_with_linesegs(vec![seg(0, 1200), seg(1200, 1200)]);
        let b = doc_with_linesegs(vec![seg(0, 1200), seg(1200, 1200)]);
        assert!(diff_linesegs(&a, &b).is_empty());
    }

    #[test]
    fn diff_linesegs_detects_value_mismatch() {
        let a = doc_with_linesegs(vec![seg(0, 21974)]);
        let b = doc_with_linesegs(vec![seg(0, 19924)]);
        let diffs = diff_linesegs(&a, &b);
        assert_eq!(diffs.len(), 1, "{diffs:?}");
        match &diffs[0].kind {
            LinesegDiffKind::ValueMismatch {
                index,
                field,
                expected,
                actual,
            } => {
                assert_eq!(*index, 0);
                assert_eq!(*field, "vertsize");
                assert_eq!(*expected, 21974);
                assert_eq!(*actual, 19924);
            }
            other => panic!("ValueMismatch 여야 함: {other:?}"),
        }
        assert_eq!(diffs[0].path, "");
    }

    #[test]
    fn diff_linesegs_detects_count_mismatch_and_compares_common() {
        // 개수 불일치 + 공통 구간(min) 값 비교 계속.
        let a = doc_with_linesegs(vec![seg(0, 1200), seg(1200, 1200)]);
        let b = doc_with_linesegs(vec![seg(0, 1000)]);
        let diffs = diff_linesegs(&a, &b);
        assert!(
            diffs.iter().any(|d| matches!(
                d.kind,
                LinesegDiffKind::CountMismatch {
                    expected: 2,
                    actual: 1
                }
            )),
            "{diffs:?}"
        );
        assert!(
            diffs.iter().any(|d| matches!(
                d.kind,
                LinesegDiffKind::ValueMismatch {
                    index: 0,
                    field: "vertsize",
                    ..
                }
            )),
            "{diffs:?}"
        );
    }

    #[test]
    fn diff_linesegs_recurses_into_cell() {
        // 셀 내부 문단의 lineseg 차이를 path 와 함께 검출.
        let make = |lh: i32| {
            let mut doc = doc_with_control(table_control(&[(0, 1)]));
            if let crate::model::control::Control::Table(t) =
                &mut doc.sections[0].paragraphs[0].controls[0]
            {
                t.cells[0].paragraphs[0].line_segs = vec![seg(0, lh)];
            }
            doc
        };
        let a = make(1200);
        let b = make(1000);
        let diffs = diff_linesegs(&a, &b);
        assert_eq!(diffs.len(), 1, "{diffs:?}");
        assert_eq!(diffs[0].path, "/ctrl[0]tbl.cell[0].p[0]");
        assert!(matches!(
            diffs[0].kind,
            LinesegDiffKind::ValueMismatch {
                field: "vertsize",
                ..
            }
        ));
    }

    #[test]
    fn task1380_lineseg_in_gate() {
        // lineseg 값 차이는 diff_documents(게이트)에서 검출되어야 한다 (3단계 동승).
        let a = doc_with_linesegs(vec![seg(0, 21974)]);
        let b = doc_with_linesegs(vec![seg(0, 19924)]);
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ParagraphLinesegs {
                section,
                paragraph,
                path,
                detail,
            } => {
                assert_eq!(*section, 0);
                assert_eq!(*paragraph, 0);
                assert_eq!(path, "");
                assert_eq!(detail, "[0].vertsize: expected=21974 actual=19924");
            }
            other => panic!("ParagraphLinesegs 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1380_gate_detects_synthetic_lineseg_asymmetry() {
        // #1380 결함 본체였던 원본 무 → RT 유 합성 비대칭(종전 파서 주입/serializer
        // fallback 패턴)을 게이트가 개수 불일치로 검출하는지 고정.
        let a = doc_with_linesegs(vec![]);
        let b = doc_with_linesegs(vec![seg(0, 1000)]);
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ParagraphLinesegs { detail, .. } => {
                assert_eq!(detail, "count: expected=0 actual=1");
            }
            other => panic!("ParagraphLinesegs 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1380_two_round_stable_with_empty_linesegs() {
        // linesegarray 부재 문단(38건)을 실제로 가진 H1 샘플의 2-round 안정성
        // (구현계획서 3.3): round1·round2 모두 게이트(IrDiff, lineseg 동승) 0 —
        // 빈 line_segs 가 어느 라운드에서도 합성(주입)되거나 소실되지 않는다.
        let bytes = std::fs::read("samples/hwpx/business_overview.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        assert!(
            doc1.sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .any(|p| p.line_segs.is_empty()),
            "픽스처 전제: linesegarray 부재(빈 line_segs) 문단이 있어야 한다"
        );

        let out1 = serialize_hwpx(&doc1).expect("serialize r1");
        let doc2 = parse_hwpx(&out1).expect("parse r1");
        let d1 = diff_documents(&doc1, &doc2);
        assert!(d1.is_empty(), "round1: {:?}", d1.differences);

        let out2 = serialize_hwpx(&doc2).expect("serialize r2");
        let doc3 = parse_hwpx(&out2).expect("parse r2");
        let d2 = diff_documents(&doc2, &doc3);
        assert!(d2.is_empty(), "round2: {:?}", d2.differences);
    }
}
