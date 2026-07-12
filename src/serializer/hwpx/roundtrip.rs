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
//!
//! Task #1388 확장:
//! - 섹션별 `PageDef`(용지 크기·방향·제본 + 여백 7필드) 비교 (`diff_page_def`) 를
//!   `SectionPageDef` 로 게이트 동승. serializer 의 secPr 템플릿 고정값 방출
//!   (여백·gutterType 변형)을 검출한다.
//!
//! Task #1387 확장:
//! - 표 캡션 비교 (`diff_table_caption`) 를 `TableCaption` 으로 게이트 동승 —
//!   존재 비대칭/속성 5종/문단 수. 캡션 내부 문단은 char_shapes·controls·linesegs
//!   재귀에 `tbl.caption.p[k]` 경로로 동승한다.
//!
//! #1403 확장:
//! - 그림/도형/묶음 캡션을 `ObjectCaption` 으로 게이트 동승 — Picture 컨트롤은
//!   `pic.caption`, ShapeObject 는 `shape_caption` 접근자(그리기 도형 `drawing.caption`
//!   + Group/Chart/Ole/Picture 전용 필드) 경유. 비교·재귀는 #1387 경로 공유.

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
    /// 섹션 `PageDef`(용지·여백) 불일치 — secPr 페이지 여백 보존 게이트 (#1388).
    ///
    /// `detail` 은 불일치 필드별 "field: expected=.. actual=.." 을 세미콜론으로
    /// 연결한 문자열 (`diff_page_def`).
    SectionPageDef {
        section: usize,
        detail: String,
    },
    /// 섹션 `<hp:visibility>`(첫쪽 머리말/꼬리말/바탕쪽 숨김·테두리·배경·첫 빈줄 숨김)
    /// 불일치 — secPr visibility 보존 게이트 (#1637).
    ///
    /// 직렬화기가 visibility 를 IR 대신 템플릿 고정값으로 방출하면 hideFirstEmptyLine 등이
    /// 드롭되어 페이지네이션이 달라진다(IR-invisible 결함). `detail` 형식은 `diff_page_def` 동형.
    SectionVisibility {
        section: usize,
        detail: String,
    },
    /// 표 캡션 불일치 — 캡션 보존 게이트 (#1387).
    ///
    /// `path` 는 `…tbl.caption` 까지의 중첩 경로. `detail` 은 존재 비대칭 또는
    /// 불일치 필드별 "field: expected=.. actual=.." 세미콜론 연결 (`diff_table_caption`).
    TableCaption {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 그림/도형/묶음 캡션 불일치 — 캡션 보존 게이트 (#1403).
    ///
    /// `path` 는 `…pic.caption` / `…shape.caption` 등 중첩 경로. `detail` 형식은
    /// `TableCaption` 과 동일 (`diff_table_caption` 공유).
    ObjectCaption {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 그림/도형/수식/묶음 설명(`hp:shapeComment`) 불일치 — 설명 보존 게이트 (#1392).
    ///
    /// `path` 는 `…pic` / `…shape` / `…eq` 등 중첩 경로. `detail` 은
    /// `"expected={:?} actual={:?}"`.
    ObjectComment {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 필드 parameters / MEMO 본문 불일치 — 필드 보존 게이트 (#1391).
    ///
    /// `path` 는 `…field` (parameters) 또는 `…field.memo.p[k]` (본문 재귀).
    FieldContent {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 그림 크기 요소(curSz/imgRect/imgDim) 불일치 — 그림 크기 보존 게이트 (#1389).
    ///
    /// `path` 는 `…pic`. `detail` 은 불일치 필드별 "field: expected=.. actual=.."
    /// 세미콜론 연결.
    PictureSize {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 표 `page_break` 불일치 — 표 분할 속성 보존 게이트 (#1393).
    ///
    /// 방출(serializer)은 PR #1405 에서 정정됨 — 본 게이트는 회귀 봉인용.
    /// `path` 는 `…tbl`.
    TablePageBreak {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 개체 `holdAnchorAndSO`(IR `prevent_page_break`) 불일치 — 페이지 하단 앵커
    /// 개체에서 1→0 드롭 시 한글 페이지 붕괴를 유발(#1594). IR-invisible 였던 갭을 봉인.
    ObjectHoldAnchor {
        section: usize,
        paragraph: usize,
        path: String,
        detail: String,
    },
    /// 개체 `flowWithText`(IR `flow_with_text`) 불일치 — 표(treatAsChar)에서 0→1 드롭 시
    /// partial-split 임계가 흔들려 페이지네이션이 달라진다(#1637). IR-invisible 였던 갭을 봉인.
    ObjectFlowWithText {
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
            SectionPageDef { section, detail } => {
                write!(f, "section[{}] page_def: {}", section, detail)
            }
            SectionVisibility { section, detail } => {
                write!(f, "section[{}] visibility: {}", section, detail)
            }
            TableCaption {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} caption: {}",
                section, paragraph, path, detail
            ),
            ObjectCaption {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} caption: {}",
                section, paragraph, path, detail
            ),
            ObjectComment {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} comment: {}",
                section, paragraph, path, detail
            ),
            FieldContent {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} field: {}",
                section, paragraph, path, detail
            ),
            PictureSize {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} pic_size: {}",
                section, paragraph, path, detail
            ),
            TablePageBreak {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} tbl page_break: {}",
                section, paragraph, path, detail
            ),
            ObjectHoldAnchor {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} holdAnchorAndSO: {}",
                section, paragraph, path, detail
            ),
            ObjectFlowWithText {
                section,
                paragraph,
                path,
                detail,
            } => write!(
                f,
                "section[{}] paragraph[{}]{} flowWithText: {}",
                section, paragraph, path, detail
            ),
        }
    }
}

/// 그림 크기 요소 비교 (#1389) — curSz(shape_attr current)·imgRect(border_x/y)·
/// imgDim. 불일치 필드를 세미콜론으로 연결. 일치하면 None.
fn diff_picture_size(
    a: &crate::model::image::Picture,
    b: &crate::model::image::Picture,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if a.shape_attr.current_width != b.shape_attr.current_width
        || a.shape_attr.current_height != b.shape_attr.current_height
    {
        parts.push(format!(
            "curSz: expected={}x{} actual={}x{}",
            a.shape_attr.current_width,
            a.shape_attr.current_height,
            b.shape_attr.current_width,
            b.shape_attr.current_height
        ));
    }
    if a.border_x != b.border_x || a.border_y != b.border_y {
        parts.push(format!(
            "imgRect: expected={:?}/{:?} actual={:?}/{:?}",
            a.border_x, a.border_y, b.border_x, b.border_y
        ));
    }
    if a.img_dim != b.img_dim {
        parts.push(format!(
            "imgDim: expected={:?} actual={:?}",
            a.img_dim, b.img_dim
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

/// 두 `CommonObjAttr.description` 비교 (#1392). 다르면 detail 문자열, 같으면 None.
fn diff_object_comment(a: &str, b: &str) -> Option<String> {
    if a == b {
        None
    } else {
        Some(format!("expected={:?} actual={:?}", a, b))
    }
}

/// 두 개체의 `prevent_page_break`(holdAnchorAndSO) 비교 (#1594). 다르면 detail, 같으면 None.
fn diff_hold_anchor(
    a: &crate::model::shape::CommonObjAttr,
    b: &crate::model::shape::CommonObjAttr,
) -> Option<String> {
    if a.prevent_page_break == b.prevent_page_break {
        None
    } else {
        Some(format!(
            "expected={} actual={}",
            a.prevent_page_break, b.prevent_page_break
        ))
    }
}

/// 두 개체의 `flow_with_text`(flowWithText) 비교 (#1637). 다르면 detail, 같으면 None.
fn diff_flow_with_text(
    a: &crate::model::shape::CommonObjAttr,
    b: &crate::model::shape::CommonObjAttr,
) -> Option<String> {
    if a.flow_with_text == b.flow_with_text {
        None
    } else {
        Some(format!(
            "expected={} actual={}",
            a.flow_with_text, b.flow_with_text
        ))
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

        // 섹션 PageDef(용지·여백) 비교 (#1388) — secPr 페이지 여백 보존 게이트.
        if let Some(detail) = diff_page_def(
            &a.sections[i].section_def.page_def,
            &b.sections[i].section_def.page_def,
        ) {
            diff.push(IrDifference::SectionPageDef { section: i, detail });
        }

        // 섹션 visibility(hideFirstEmptyLine 등) 비교 (#1637) — secPr visibility 보존 게이트.
        if let Some(detail) =
            diff_visibility(&a.sections[i].section_def, &b.sections[i].section_def)
        {
            diff.push(IrDifference::SectionVisibility { section: i, detail });
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

/// 표 캡션 비교 (#1387). 존재 비대칭/속성/문단 수 불일치를 "field: expected=..
/// actual=.." 세미콜론 연결로 돌려준다. 일치하면 `None`.
/// 그림/도형/묶음 캡션(#1403)도 동일 비교를 공유한다 (`ObjectCaption` 으로 보고).
///
/// `vert_align` 은 비교 제외 — HWPX `hp:caption` 에 대응 속성이 없는 HWP5 유래
/// 필드라(#1387 1단계 전수 측정) serializer 가 방출하지 않으며, HWP5 출발 플로우
/// 비교에서 위양성을 만든다. 내부 문단의 상세 비교는 char_shapes/controls/linesegs
/// 재귀가 담당하므로 여기서는 문단 수만 본다.
fn diff_table_caption(
    a: &Option<crate::model::shape::Caption>,
    b: &Option<crate::model::shape::Caption>,
) -> Option<String> {
    let (a, b) = match (a, b) {
        (None, None) => return None,
        (Some(_), None) => return Some("missing: expected=Some actual=None".to_string()),
        (None, Some(_)) => return Some("synthetic: expected=None actual=Some".to_string()),
        (Some(a), Some(b)) => (a, b),
    };
    let mut parts: Vec<String> = Vec::new();
    if a.direction != b.direction {
        parts.push(format!(
            "side: expected={:?} actual={:?}",
            a.direction, b.direction
        ));
    }
    if a.include_margin != b.include_margin {
        parts.push(format!(
            "fullSz: expected={} actual={}",
            a.include_margin, b.include_margin
        ));
    }
    if a.width != b.width {
        parts.push(format!("width: expected={} actual={}", a.width, b.width));
    }
    if a.spacing != b.spacing {
        parts.push(format!("gap: expected={} actual={}", a.spacing, b.spacing));
    }
    if a.max_width != b.max_width {
        parts.push(format!(
            "lastWidth: expected={} actual={}",
            a.max_width, b.max_width
        ));
    }
    if a.paragraphs.len() != b.paragraphs.len() {
        parts.push(format!(
            "paragraphs: expected={} actual={}",
            a.paragraphs.len(),
            b.paragraphs.len()
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

/// 섹션 `PageDef` 비교 (#1388). 불일치 필드를 "field: expected=.. actual=.." 로 모아
/// 세미콜론으로 연결해 돌려준다. 일치하면 `None`.
///
/// 비교 제외 필드와 사유:
/// - `attr`: 비트 원본 — `binding`/`landscape` 와 의미 중복 (해석 필드 쪽을 비교)
/// - `pagination_bottom_tolerance`: 렌더러 내부 허용치 — 파일 포맷 필드 아님
fn diff_page_def(
    a: &crate::model::page::PageDef,
    b: &crate::model::page::PageDef,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    macro_rules! cmp_field {
        ($field:ident) => {
            if a.$field != b.$field {
                parts.push(format!(
                    "{}: expected={} actual={}",
                    stringify!($field),
                    a.$field,
                    b.$field
                ));
            }
        };
    }
    cmp_field!(width);
    cmp_field!(height);
    cmp_field!(margin_left);
    cmp_field!(margin_right);
    cmp_field!(margin_top);
    cmp_field!(margin_bottom);
    cmp_field!(margin_header);
    cmp_field!(margin_footer);
    cmp_field!(margin_gutter);
    cmp_field!(landscape);
    if a.binding != b.binding {
        parts.push(format!(
            "binding: expected={:?} actual={:?}",
            a.binding, b.binding
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

/// 섹션 `<hp:visibility>` 플래그 비교 (#1637) — secPr visibility 보존 게이트.
///
/// IR(SectionDef)에 보존되는 6필드만 비교한다(hideFirstPageNum·showLineNumber 는
/// 파서가 IR 에 적재하지 않으므로 제외). 직렬화기가 visibility 를 IR 로 방출하지 않으면
/// (특히 hide_empty_line) 페이지네이션이 달라지는 IR-invisible 결함을 게이트화한다.
fn diff_visibility(
    a: &crate::model::document::SectionDef,
    b: &crate::model::document::SectionDef,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    macro_rules! cmp_field {
        ($field:ident) => {
            if a.$field != b.$field {
                parts.push(format!(
                    "{}: expected={} actual={}",
                    stringify!($field),
                    a.$field,
                    b.$field
                ));
            }
        };
    }
    cmp_field!(hide_header);
    cmp_field!(hide_footer);
    cmp_field!(hide_master_page);
    cmp_field!(hide_border);
    cmp_field!(hide_fill);
    cmp_field!(hide_empty_line);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
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
                // 표 캡션 내부 문단 lineseg 재귀 (#1387) — 존재/속성 비교는
                // diff_paragraph_char_shapes 쪽 한 곳에서 수행.
                if let (Some(ca), Some(cb)) = (&ta.caption, &tb.caption) {
                    for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]tbl.caption.p[{k}]");
                        diff_paragraph_linesegs(out, section, paragraph, &p, qa, qb);
                    }
                }
            }
            // 그림 캡션 내부 문단 lineseg 재귀 (#1403 후속 — char_shapes 쪽과 대칭 복원).
            (Control::Picture(pia), Control::Picture(pib)) => {
                if let (Some(ca), Some(cb)) = (&pia.caption, &pib.caption) {
                    for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]pic.caption.p[{k}]");
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
            // MEMO 본문 문단 lineseg 재귀 (#1391).
            (Control::Field(fa), Control::Field(fb)) => {
                for (k, (qa, qb)) in fa
                    .memo_paragraphs
                    .iter()
                    .zip(fb.memo_paragraphs.iter())
                    .enumerate()
                {
                    let p = format!("{path}/ctrl[{ci}]field.memo.p[{k}]");
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
    // 도형/묶음 캡션 내부 문단 lineseg 재귀 (#1403 후속 — char_shapes 쪽과 대칭 복원).
    if let (Some(ca), Some(cb)) = (shape_caption(sa), shape_caption(sb)) {
        for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate() {
            let p = format!("{path}.caption.p[{k}]");
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
                // 표 page_break 비교 (#1393) — 방출은 PR #1405 정정, 게이트 회귀 봉인.
                if ta.page_break != tb.page_break {
                    diff.push(IrDifference::TablePageBreak {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]tbl"),
                        detail: format!("expected={:?} actual={:?}", ta.page_break, tb.page_break),
                    });
                }
                // [#1594] holdAnchorAndSO 보존 게이트 — 페이지 하단 앵커 개체 붕괴 봉인.
                if let Some(detail) = diff_hold_anchor(&ta.common, &tb.common) {
                    diff.push(IrDifference::ObjectHoldAnchor {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]tbl"),
                        detail,
                    });
                }
                // [#1637] flowWithText 보존 게이트 — 표 partial-split 임계 변동 봉인.
                if let Some(detail) = diff_flow_with_text(&ta.common, &tb.common) {
                    diff.push(IrDifference::ObjectFlowWithText {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]tbl"),
                        detail,
                    });
                }
                for (cell_i, (cea, ceb)) in ta.cells.iter().zip(tb.cells.iter()).enumerate() {
                    for (k, (qa, qb)) in
                        cea.paragraphs.iter().zip(ceb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]tbl.cell[{cell_i}].p[{k}]");
                        diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                    }
                }
                // 표 캡션 비교 (#1387) — 존재/속성/문단 수 + 내부 문단 재귀.
                if let Some(detail) = diff_table_caption(&ta.caption, &tb.caption) {
                    diff.push(IrDifference::TableCaption {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]tbl.caption"),
                        detail,
                    });
                }
                if let (Some(ca), Some(cb)) = (&ta.caption, &tb.caption) {
                    for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]tbl.caption.p[{k}]");
                        diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                    }
                }
            }
            // 그림 캡션 비교 (#1403) — 존재/속성/문단 수 + 내부 문단 재귀.
            (Control::Picture(pia), Control::Picture(pib)) => {
                // 그림 크기 요소 비교 (#1389) — curSz/imgRect/imgDim IR 보존 게이트.
                if let Some(detail) = diff_picture_size(pia, pib) {
                    diff.push(IrDifference::PictureSize {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]pic"),
                        detail,
                    });
                }
                if let Some(detail) = diff_table_caption(&pia.caption, &pib.caption) {
                    diff.push(IrDifference::ObjectCaption {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]pic.caption"),
                        detail,
                    });
                }
                // 그림 설명 비교 (#1392).
                if let Some(detail) =
                    diff_object_comment(&pia.common.description, &pib.common.description)
                {
                    diff.push(IrDifference::ObjectComment {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]pic"),
                        detail,
                    });
                }
                // [#1594] holdAnchorAndSO 보존 게이트.
                if let Some(detail) = diff_hold_anchor(&pia.common, &pib.common) {
                    diff.push(IrDifference::ObjectHoldAnchor {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]pic"),
                        detail,
                    });
                }
                if let (Some(ca), Some(cb)) = (&pia.caption, &pib.caption) {
                    for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate()
                    {
                        let p = format!("{path}/ctrl[{ci}]pic.caption.p[{k}]");
                        diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
                    }
                }
            }
            // 수식 설명 비교 (#1392) — equation 은 본문 텍스트 비교 대상이 아니므로
            // description 만 동승.
            (Control::Equation(ea), Control::Equation(eb)) => {
                if let Some(detail) =
                    diff_object_comment(&ea.common.description, &eb.common.description)
                {
                    diff.push(IrDifference::ObjectComment {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]eq"),
                        detail,
                    });
                }
                // [#1594] holdAnchorAndSO 보존 게이트.
                if let Some(detail) = diff_hold_anchor(&ea.common, &eb.common) {
                    diff.push(IrDifference::ObjectHoldAnchor {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]eq"),
                        detail,
                    });
                }
                // [#1655] 수식 flowWithText 보존 게이트.
                if let Some(detail) = diff_flow_with_text(&ea.common, &eb.common) {
                    diff.push(IrDifference::ObjectFlowWithText {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]eq"),
                        detail,
                    });
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
            // 필드 parameters / MEMO 본문 비교 (#1391).
            (Control::Field(fa), Control::Field(fb)) => {
                if fa.raw_parameters_xml != fb.raw_parameters_xml {
                    diff.push(IrDifference::FieldContent {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]field"),
                        detail: format!(
                            "parameters: expected={:?} actual={:?}",
                            fa.raw_parameters_xml, fb.raw_parameters_xml
                        ),
                    });
                }
                if fa.memo_paragraphs.len() != fb.memo_paragraphs.len() {
                    diff.push(IrDifference::FieldContent {
                        section,
                        paragraph,
                        path: format!("{path}/ctrl[{ci}]field"),
                        detail: format!(
                            "memo paragraphs: expected={} actual={}",
                            fa.memo_paragraphs.len(),
                            fb.memo_paragraphs.len()
                        ),
                    });
                }
                for (k, (qa, qb)) in fa
                    .memo_paragraphs
                    .iter()
                    .zip(fb.memo_paragraphs.iter())
                    .enumerate()
                {
                    let p = format!("{path}/ctrl[{ci}]field.memo.p[{k}]");
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
    // 도형/묶음 캡션 비교 (#1403) — 존재/속성/문단 수 + 내부 문단 재귀.
    let (capa, capb) = (shape_caption(sa), shape_caption(sb));
    if let Some(detail) = diff_table_caption(capa, capb) {
        diff.push(IrDifference::ObjectCaption {
            section,
            paragraph,
            path: format!("{path}.caption"),
            detail,
        });
    }
    if let (Some(ca), Some(cb)) = (capa, capb) {
        for (k, (qa, qb)) in ca.paragraphs.iter().zip(cb.paragraphs.iter()).enumerate() {
            let p = format!("{path}.caption.p[{k}]");
            diff_paragraph_char_shapes(diff, section, paragraph, &p, qa, qb);
        }
    }
    // 도형/묶음 설명 비교 (#1392).
    if let Some(detail) =
        diff_object_comment(&shape_common(sa).description, &shape_common(sb).description)
    {
        diff.push(IrDifference::ObjectComment {
            section,
            paragraph,
            path: path.to_string(),
            detail,
        });
    }
    if let (ShapeObject::Group(ga), ShapeObject::Group(gb)) = (sa, sb) {
        for (k, (c1, c2)) in ga.children.iter().zip(gb.children.iter()).enumerate() {
            let p = format!("{path}.child[{k}]");
            diff_shape_char_shapes(diff, section, paragraph, &p, c1, c2);
        }
    }
}

/// ShapeObject 에서 `CommonObjAttr` 참조를 꺼낸다 (#1392 — description 비교용).
fn shape_common(s: &crate::model::shape::ShapeObject) -> &crate::model::shape::CommonObjAttr {
    use crate::model::shape::ShapeObject::*;
    match s {
        Line(x) => &x.common,
        Rectangle(x) => &x.common,
        Ellipse(x) => &x.common,
        Arc(x) => &x.common,
        Polygon(x) => &x.common,
        Curve(x) => &x.common,
        Chart(x) => &x.common,
        Ole(x) => &x.common,
        Group(x) => &x.common,
        Picture(x) => &x.common,
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

/// ShapeObject 에서 캡션 필드 참조를 꺼낸다 (#1403).
/// 그리기 도형은 `drawing.caption`, 묶음/그림/차트/OLE 는 각자의 caption 필드.
fn shape_caption(s: &crate::model::shape::ShapeObject) -> &Option<crate::model::shape::Caption> {
    use crate::model::shape::ShapeObject::*;
    match s {
        Line(x) => &x.drawing.caption,
        Rectangle(x) => &x.drawing.caption,
        Ellipse(x) => &x.drawing.caption,
        Arc(x) => &x.drawing.caption,
        Polygon(x) => &x.drawing.caption,
        Curve(x) => &x.drawing.caption,
        Chart(x) => &x.caption,
        Ole(x) => &x.caption,
        Group(x) => &x.caption,
        Picture(x) => &x.caption,
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

    // ---------- #1387: diff_table_caption (게이트 동승) ----------

    fn caption_with_paras(n: usize) -> crate::model::shape::Caption {
        let mut c = crate::model::shape::Caption::default();
        for _ in 0..n {
            c.paragraphs.push(Paragraph::default());
        }
        c
    }

    fn table_with_caption(
        cap: Option<crate::model::shape::Caption>,
    ) -> crate::model::control::Control {
        match table_control(&[]) {
            crate::model::control::Control::Table(mut t) => {
                t.caption = cap;
                crate::model::control::Control::Table(t)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn task1387_caption_loss_in_gate() {
        // #1387 결함 본체였던 캡션 소실(원본 유 → RT 무)을 게이트가 검출하는지 고정.
        let a = doc_with_control(table_with_caption(Some(caption_with_paras(1))));
        let b = doc_with_control(table_with_caption(None));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::TableCaption { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]tbl.caption");
                assert_eq!(detail, "missing: expected=Some actual=None");
            }
            other => panic!("TableCaption 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1387_caption_attr_mismatch_in_gate() {
        let mut ca = caption_with_paras(1);
        ca.spacing = 850;
        let mut cb = caption_with_paras(1);
        cb.spacing = 0;
        cb.direction = crate::model::shape::CaptionDirection::Top;
        let a = doc_with_control(table_with_caption(Some(ca)));
        let b = doc_with_control(table_with_caption(Some(cb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::TableCaption { detail, .. } => {
                assert_eq!(
                    detail,
                    "side: expected=Bottom actual=Top; gap: expected=850 actual=0"
                );
            }
            other => panic!("TableCaption 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1387_caption_paragraph_recursed_in_gate() {
        // 캡션 내부 문단의 char_shapes 차이가 `tbl.caption.p[k]` 경로로 검출.
        let mut ca = caption_with_paras(1);
        ca.paragraphs[0].char_shapes = to_refs(&[(0, 6)]);
        let mut cb = caption_with_paras(1);
        cb.paragraphs[0].char_shapes = to_refs(&[(0, 7)]);
        let a = doc_with_control(table_with_caption(Some(ca)));
        let b = doc_with_control(table_with_caption(Some(cb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ParagraphCharShapes { path, .. } => {
                assert_eq!(path, "/ctrl[0]tbl.caption.p[0]");
            }
            other => panic!("ParagraphCharShapes 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1387_ta_pic_001_r_roundtrip_caption_gate_zero() {
        // 캡션 보유 실샘플의 roundtrip 게이트 0 — 2단계 serializer 수정 + 캡션
        // 동승 후 기대치 (autoNum #1382 변위는 텍스트 축 — 게이트 비교 항목 밖).
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        let out = serialize_hwpx(&doc1).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&doc1, &doc2);
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    // ---------- #1388: diff_page_def (게이트 동승) ----------

    fn doc_with_page_def(pd: crate::model::page::PageDef) -> Document {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        section.section_def.page_def = pd;
        doc.sections.push(section);
        doc
    }

    #[test]
    fn task1388_page_def_in_gate() {
        // 여백·제본 차이는 diff_documents(게이트)에서 검출되어야 한다.
        // 값은 온새미로 sec0 실측: 원본 left=7086 right=14173 LEFT_RIGHT →
        // 종전 RT 템플릿 고정값 left=8504 right=8504 LEFT_ONLY.
        let mut a_pd = crate::model::page::PageDef::default();
        a_pd.margin_left = 7086;
        a_pd.margin_right = 14173;
        a_pd.binding = crate::model::page::BindingMethod::DuplexSided;
        let mut b_pd = a_pd.clone();
        b_pd.margin_left = 8504;
        b_pd.margin_right = 8504;
        b_pd.binding = crate::model::page::BindingMethod::SingleSided;

        let diff = diff_documents(&doc_with_page_def(a_pd), &doc_with_page_def(b_pd));
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::SectionPageDef { section, detail } => {
                assert_eq!(*section, 0);
                assert_eq!(
                    detail,
                    "margin_left: expected=7086 actual=8504; \
                     margin_right: expected=14173 actual=8504; \
                     binding: expected=DuplexSided actual=SingleSided"
                );
            }
            other => panic!("SectionPageDef 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1388_page_def_equal_is_empty() {
        // 동일 PageDef 는 차이 0 — attr/pagination_bottom_tolerance 는 비교 제외.
        let mut a_pd = crate::model::page::PageDef::default();
        a_pd.margin_left = 7086;
        let mut b_pd = a_pd.clone();
        b_pd.attr = 0xFF;
        b_pd.pagination_bottom_tolerance = 100;
        let diff = diff_documents(&doc_with_page_def(a_pd), &doc_with_page_def(b_pd));
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    #[test]
    fn task1388_roundtrip_preserves_page_def() {
        // 비템플릿 여백(left/right=4252)을 가진 실샘플의 roundtrip 에서
        // PageDef 차이 0 — 2단계 serializer 수정의 게이트 검증.
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        let pd = &doc1.sections[0].section_def.page_def;
        assert!(
            pd.margin_left != 8504 || pd.margin_right != 8504,
            "픽스처 전제: 템플릿 고정값과 다른 여백이어야 한다 (left={} right={})",
            pd.margin_left,
            pd.margin_right
        );

        let out = serialize_hwpx(&doc1).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&doc1, &doc2);
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    // ---------- #1637: secPr visibility (hideFirstEmptyLine 등 게이트 동승) ----------

    fn doc_with_visibility(hide_empty_line: bool, hide_header: bool) -> Document {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        section.section_def.hide_empty_line = hide_empty_line;
        section.section_def.hide_header = hide_header;
        doc.sections.push(section);
        doc
    }

    #[test]
    fn task1637_visibility_in_gate() {
        // hideFirstEmptyLine 등 visibility 차이는 diff_documents(게이트)에서 검출되어야 한다.
        let a = doc_with_visibility(true, false);
        let b = doc_with_visibility(false, false);
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::SectionVisibility { section, detail } => {
                assert_eq!(*section, 0);
                assert_eq!(detail, "hide_empty_line: expected=true actual=false");
            }
            other => panic!("SectionVisibility 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1637_visibility_equal_is_empty() {
        // 동일 visibility 는 차이 0.
        let diff = diff_documents(
            &doc_with_visibility(true, true),
            &doc_with_visibility(true, true),
        );
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    #[test]
    fn task1637_roundtrip_preserves_hide_empty_line() {
        // hideFirstEmptyLine="1" 문서의 roundtrip 에서 값 보존 + 게이트 0 (직렬화기
        // visibility IR 치환 검증). 종전엔 템플릿 고정값 "0" 으로 드롭되어 페이지네이션 변동.
        // serialize_hwpx 는 HWPX 패키지(ZIP)를 반환하므로 reparse 로 값 보존을 검증한다.
        let doc = doc_with_visibility(true, true);
        let out = serialize_hwpx(&doc).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        assert!(
            doc2.sections[0].section_def.hide_empty_line,
            "hide_empty_line=1 이 roundtrip 에서 보존되어야 한다"
        );
        assert!(
            doc2.sections[0].section_def.hide_header,
            "hide_header=1 이 roundtrip 에서 보존되어야 한다"
        );
        // visibility 축의 게이트 0 (다른 축의 near-empty-doc 직렬화 산물은 본 테스트 범위 밖).
        assert!(
            diff_visibility(&doc.sections[0].section_def, &doc2.sections[0].section_def).is_none(),
            "visibility roundtrip 차이가 없어야 한다"
        );
    }

    #[test]
    fn task1637_table_flow_with_text_in_gate() {
        // 표 flowWithText 차이는 diff_documents(게이트)에서 ObjectFlowWithText 로 검출.
        use crate::model::table::Table;
        let mut ta = Table::default();
        ta.common.flow_with_text = false;
        let mut tb = Table::default();
        tb.common.flow_with_text = true;
        let a = doc_with_control(crate::model::control::Control::Table(Box::new(ta)));
        let b = doc_with_control(crate::model::control::Control::Table(Box::new(tb)));
        let diff = diff_documents(&a, &b);
        assert!(
            diff.differences
                .iter()
                .any(|d| matches!(d, IrDifference::ObjectFlowWithText { .. })),
            "표 flowWithText 차이는 게이트에서 검출되어야 한다: {:?}",
            diff.differences
        );
    }

    #[test]
    fn task1655_equation_flow_with_text_in_gate() {
        // 수식 flowWithText 차이도 diff_documents(게이트)에서 ObjectFlowWithText 로 검출.
        let mut ea = crate::model::control::Equation::default();
        ea.common.flow_with_text = false;
        let mut eb = crate::model::control::Equation::default();
        eb.common.flow_with_text = true;
        let a = doc_with_control(crate::model::control::Control::Equation(Box::new(ea)));
        let b = doc_with_control(crate::model::control::Control::Equation(Box::new(eb)));
        let diff = diff_documents(&a, &b);
        assert!(
            diff.differences
                .iter()
                .any(|d| matches!(d, IrDifference::ObjectFlowWithText { .. })),
            "수식 flowWithText 차이는 게이트에서 검출되어야 한다: {:?}",
            diff.differences
        );
    }

    // ---------- #1403: 그림/도형/묶음 캡션 (게이트 동승) ----------

    #[test]
    fn task1403_pic_caption_lineseg_recursed_in_gate() {
        // #1403 후속 — 객체 캡션 내부 문단의 lineseg 차이가 char_shapes 와 대칭으로
        // `pic.caption.p[k]` 경로에서 검출되는지 고정 (merge 시 보완 1건).
        let mut ca = caption_with_paras(1);
        ca.paragraphs[0].line_segs = vec![seg(0, 1000)];
        let mut cb = caption_with_paras(1);
        cb.paragraphs[0].line_segs = vec![seg(0, 2000)];
        let mut pa = crate::model::image::Picture::default();
        pa.caption = Some(ca);
        let mut pb = crate::model::image::Picture::default();
        pb.caption = Some(cb);
        let a = doc_with_control(crate::model::control::Control::Picture(Box::new(pa)));
        let b = doc_with_control(crate::model::control::Control::Picture(Box::new(pb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ParagraphLinesegs { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]pic.caption.p[0]");
                assert_eq!(detail, "[0].vertsize: expected=1000 actual=2000");
            }
            other => panic!("ParagraphLinesegs 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1403_shape_caption_lineseg_recursed_in_gate() {
        // 도형(drawing.caption) 경로의 lineseg 재귀 — `shape.caption.p[k]`.
        let mk = |vs: i32| {
            let mut cap = caption_with_paras(1);
            cap.paragraphs[0].line_segs = vec![seg(0, vs)];
            let mut el = crate::model::shape::EllipseShape::default();
            el.drawing.caption = Some(cap);
            doc_with_control(crate::model::control::Control::Shape(Box::new(
                crate::model::shape::ShapeObject::Ellipse(el),
            )))
        };
        let diff = diff_documents(&mk(1000), &mk(2000));
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ParagraphLinesegs { path, .. } => {
                assert_eq!(path, "/ctrl[0]shape.caption.p[0]");
            }
            other => panic!("ParagraphLinesegs 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1403_pic_caption_loss_in_gate() {
        // #1403 결함 본체였던 그림 캡션 소실(원본 유 → RT 무)을 게이트가 검출하는지 고정.
        let mut pa = crate::model::image::Picture::default();
        pa.caption = Some(caption_with_paras(1));
        let pb = crate::model::image::Picture::default();
        let a = doc_with_control(crate::model::control::Control::Picture(Box::new(pa)));
        let b = doc_with_control(crate::model::control::Control::Picture(Box::new(pb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectCaption { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]pic.caption");
                assert_eq!(detail, "missing: expected=Some actual=None");
            }
            other => panic!("ObjectCaption 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1403_line_caption_loss_in_gate() {
        // 그리기 도형(drawing.caption) 경로 — line 캡션 소실 검출.
        let mut la = crate::model::shape::LineShape::default();
        la.drawing.caption = Some(caption_with_paras(1));
        let lb = crate::model::shape::LineShape::default();
        let a = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Line(la),
        )));
        let b = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Line(lb),
        )));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectCaption { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]shape.caption");
                assert_eq!(detail, "missing: expected=Some actual=None");
            }
            other => panic!("ObjectCaption 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1403_legacy_shape_caption_roundtrips() {
        // legacy 문자열 경로(ellipse/arc/polygon/curve/chart/ole)의 캡션 방출 검증 —
        // HWP5 파서는 모든 도형 캡션을 적재하므로(parser/control/shape.rs) 미방출 시
        // HWP5→HWPX 변환에서 소실된다. serialize→reparse 후 게이트 차이 0 이어야 한다.
        let mut cap = caption_with_paras(1);
        cap.width = 8504;
        cap.paragraphs[0].text = "타원 캡션".to_string();
        // 빈 char_shapes 는 reparse 시 [(0,0)] 으로 돌아오므로(픽스처 노이즈) 명시한다.
        cap.paragraphs[0].char_shapes = to_refs(&[(0, 0)]);
        let mut el = crate::model::shape::EllipseShape::default();
        el.drawing.caption = Some(cap);
        let mut a = roundtrip_doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Ellipse(el),
        )));
        a.sections[0].paragraphs[0].char_shapes = to_refs(&[(0, 0)]);
        let out = serialize_hwpx(&a).expect("serialize");
        let b = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&a, &b);
        assert!(diff.is_empty(), "{:?}", diff.differences);
        // 텍스트 보존 직접 확인
        match &b.sections[0].paragraphs[1].controls[0] {
            crate::model::control::Control::Shape(s) => match s.as_ref() {
                crate::model::shape::ShapeObject::Ellipse(e2) => {
                    let c2 = e2.drawing.caption.as_ref().expect("캡션 보존");
                    assert_eq!(c2.paragraphs[0].text, "타원 캡션");
                }
                other => panic!("Ellipse 여야 함: {other:?}"),
            },
            other => panic!("Shape 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1403_group_caption_loss_in_gate() {
        // 묶음 개체(GroupShape.caption) 경로 — container 캡션 소실 검출.
        let mut ga = crate::model::shape::GroupShape::default();
        ga.caption = Some(caption_with_paras(1));
        let gb = crate::model::shape::GroupShape::default();
        let a = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Group(ga),
        )));
        let b = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Group(gb),
        )));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectCaption { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]shape.caption");
                assert_eq!(detail, "missing: expected=Some actual=None");
            }
            other => panic!("ObjectCaption 여야 함: {other:?}"),
        }
    }

    // ---------- #1392: shapeComment(객체 설명) 게이트 동승 ----------

    #[test]
    fn task1392_pic_comment_loss_in_gate() {
        let mut pa = crate::model::image::Picture::default();
        pa.common.description = "그림입니다.".to_string();
        let pb = crate::model::image::Picture::default();
        let a = doc_with_control(crate::model::control::Control::Picture(Box::new(pa)));
        let b = doc_with_control(crate::model::control::Control::Picture(Box::new(pb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectComment { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]pic");
                assert_eq!(detail, "expected=\"그림입니다.\" actual=\"\"");
            }
            other => panic!("ObjectComment 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1392_equation_comment_loss_in_gate() {
        let mut ea = crate::model::control::Equation::default();
        ea.common.description = "수식 설명".to_string();
        let eb = crate::model::control::Equation::default();
        let a = doc_with_control(crate::model::control::Control::Equation(Box::new(ea)));
        let b = doc_with_control(crate::model::control::Control::Equation(Box::new(eb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectComment { path, .. } => assert_eq!(path, "/ctrl[0]eq"),
            other => panic!("ObjectComment 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1392_shape_comment_loss_in_gate() {
        let mut el = crate::model::shape::EllipseShape::default();
        el.common.description = "타원 설명".to_string();
        let a = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Ellipse(el),
        )));
        let b = doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Ellipse(crate::model::shape::EllipseShape::default()),
        )));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::ObjectComment { path, .. } => assert_eq!(path, "/ctrl[0]shape"),
            other => panic!("ObjectComment 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1392_equal_comment_no_diff() {
        let mut pa = crate::model::image::Picture::default();
        pa.common.description = "동일".to_string();
        let mut pb = crate::model::image::Picture::default();
        pb.common.description = "동일".to_string();
        let a = doc_with_control(crate::model::control::Control::Picture(Box::new(pa)));
        let b = doc_with_control(crate::model::control::Control::Picture(Box::new(pb)));
        assert!(diff_documents(&a, &b).is_empty());
    }

    #[test]
    fn task1451_legacy_shape_comment_serialize_roundtrip() {
        // #1451: render_common_shape_xml 경유 도형(polygon 등)의 shapeComment 가
        // serialize → parse 왕복에서 보존되는지 직접 가드한다.
        // 기존 task1392 게이트는 IR diff 검출만 하므로, 여기서 "보존 성공" 방향을 가드한다.
        let mut poly = crate::model::shape::PolygonShape::default();
        poly.common.description = "다각형입니다.".to_string();
        let doc = roundtrip_doc_with_control(crate::model::control::Control::Shape(Box::new(
            crate::model::shape::ShapeObject::Polygon(poly),
        )));

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = parse_hwpx(&bytes).expect("parse");
        let desc = match &doc2.sections[0].paragraphs[1].controls[0] {
            crate::model::control::Control::Shape(s) => match s.as_ref() {
                crate::model::shape::ShapeObject::Polygon(p) => &p.common.description,
                other => panic!("Polygon 이어야 함: {other:?}"),
            },
            other => panic!("Shape 컨트롤이어야 함: {other:?}"),
        };
        assert_eq!(desc, "다각형입니다.", "polygon shapeComment 왕복 보존");
    }

    // ---------- #1391: 필드 parameters / MEMO 본문 게이트 동승 ----------

    #[test]
    fn task1391_field_parameters_loss_in_gate() {
        let mut fa = crate::model::control::Field::default();
        fa.raw_parameters_xml = Some("<hp:parameters cnt=\"1\"></hp:parameters>".into());
        let fb = crate::model::control::Field::default();
        let a = doc_with_control(crate::model::control::Control::Field(fa));
        let b = doc_with_control(crate::model::control::Control::Field(fb));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::FieldContent { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]field");
                assert!(detail.starts_with("parameters: expected="), "{detail}");
            }
            other => panic!("FieldContent 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1391_memo_paragraph_loss_in_gate() {
        let mut fa = crate::model::control::Field::default();
        fa.field_type = crate::model::control::FieldType::Memo;
        fa.memo_paragraphs.push(Paragraph::default());
        let mut fb = crate::model::control::Field::default();
        fb.field_type = crate::model::control::FieldType::Memo;
        let a = doc_with_control(crate::model::control::Control::Field(fa));
        let b = doc_with_control(crate::model::control::Control::Field(fb));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::FieldContent { detail, .. } => {
                assert_eq!(detail, "memo paragraphs: expected=1 actual=0");
            }
            other => panic!("FieldContent 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1391_aift_memo_roundtrips() {
        // 실샘플 — aift MEMO 2건 parameters + 본문 보존, roundtrip 게이트 0.
        let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        let memo_count = doc1
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .flat_map(|p| &p.controls)
            .filter(|c| {
                matches!(c, crate::model::control::Control::Field(f)
                if f.field_type == crate::model::control::FieldType::Memo)
            })
            .count();
        assert_eq!(memo_count, 2, "aift MEMO 2건");
        let out = serialize_hwpx(&doc1).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&doc1, &doc2);
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    // ---------- #1389: 그림 크기 요소 게이트 동승 ----------

    #[test]
    fn task1389_picture_size_diff_in_gate() {
        let mut pa = crate::model::image::Picture::default();
        pa.shape_attr.current_width = 1366;
        pa.img_dim = (49380, 45840);
        let pb = crate::model::image::Picture::default(); // 크기 0
        let a = doc_with_control(crate::model::control::Control::Picture(Box::new(pa)));
        let b = doc_with_control(crate::model::control::Control::Picture(Box::new(pb)));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::PictureSize { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]pic");
                assert!(
                    detail.contains("curSz") && detail.contains("imgDim"),
                    "{detail}"
                );
            }
            other => panic!("PictureSize 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1389_ta_pic_size_roundtrips() {
        // 실샘플 — ta-pic 셀 내 그림 curSz/imgRect/imgDim 보존, roundtrip 게이트 0.
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        // 셀 내 pic 의 img_dim 이 적재됐는지 전제 확인 (49380 등 비0).
        let has_dim = doc1
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .flat_map(|p| &p.controls)
            .any(|c| matches!(c, crate::model::control::Control::Table(t)
                if t.cells.iter().flat_map(|ce| &ce.paragraphs).flat_map(|q| &q.controls)
                    .any(|cc| matches!(cc, crate::model::control::Control::Picture(pic) if pic.img_dim.0 > 0))));
        assert!(has_dim, "셀 내 pic img_dim 적재 전제");
        let out = serialize_hwpx(&doc1).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&doc1, &doc2);
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }

    // ---------- #1393: 표 page_break 게이트 동승 ----------

    #[test]
    fn task1393_table_page_break_diff_in_gate() {
        use crate::model::table::TablePageBreak;
        fn tbl(pb: TablePageBreak) -> crate::model::control::Control {
            match table_control(&[]) {
                crate::model::control::Control::Table(mut t) => {
                    t.page_break = pb;
                    crate::model::control::Control::Table(t)
                }
                _ => unreachable!(),
            }
        }
        let a = doc_with_control(tbl(TablePageBreak::RowBreak));
        let b = doc_with_control(tbl(TablePageBreak::CellBreak));
        let diff = diff_documents(&a, &b);
        assert_eq!(diff.differences.len(), 1, "{:?}", diff.differences);
        match &diff.differences[0] {
            IrDifference::TablePageBreak { path, detail, .. } => {
                assert_eq!(path, "/ctrl[0]tbl");
                assert_eq!(detail, "expected=RowBreak actual=CellBreak");
            }
            other => panic!("TablePageBreak 여야 함: {other:?}"),
        }
    }

    #[test]
    fn task1393_form_002_page_break_roundtrips() {
        // 실샘플 — form-002 표 page_break(CELL=RowBreak) 보존, roundtrip 게이트 0.
        let bytes = std::fs::read("samples/hwpx/form-002.hwpx").expect("샘플 읽기");
        let doc1 = parse_hwpx(&bytes).expect("parse 원본");
        let out = serialize_hwpx(&doc1).expect("serialize");
        let doc2 = parse_hwpx(&out).expect("reparse");
        let diff = diff_documents(&doc1, &doc2);
        assert!(diff.is_empty(), "{:?}", diff.differences);
    }
}
