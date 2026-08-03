//! 문서 구조(개요/조문) 구조화 추출 — 조문 DB화용.
//!
//! 문단을 순회해 **개요 계층(ParaShape.para_level/head_type)** 또는 **법률 조문 패턴
//! (편/장/절/관/조/항/호/목)** 으로 분류하여 중첩 트리(JSON)로 내보낸다. 본문(비제목) 문단은
//! 직전 제목 노드의 `body` 에 귀속된다.
//!
//! 파서/렌더 무변경의 읽기 전용 질의(추가 기능). 자기 라운드트립·시각 충실도와 무관.

use serde::Serialize;

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::document::Document;
use crate::model::style::HeadType;

/// 분류 방식.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMode {
    /// 명시적 Outline → confidence를 통과한 조 제목 → Number 순으로 증거를 선택.
    Auto,
    /// IR 개요 수준(para_level)만 사용.
    Outline,
    /// 법률 조문 텍스트 패턴(제N조/①/1./가.)만 사용.
    Clause,
}

impl StructureMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "outline" => Some(Self::Outline),
            "clause" => Some(Self::Clause),
            _ => None,
        }
    }
}

/// 구조 트리 노드.
#[derive(Debug, Clone, Serialize)]
pub struct StructureNode {
    /// 계층 깊이(1=최상위).
    pub level: u8,
    /// 종류: "outline" | "편"|"장"|"절"|"관"|"조"|"항"|"호"|"목".
    pub kind: &'static str,
    /// 검출된 번호 마커(예: "제1조", "①", "1.", "가."). 개요(자동번호)는 빈 문자열.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub marker: String,
    /// 제목 문단 텍스트.
    pub heading: String,
    pub section: usize,
    pub paragraph: usize,
    /// 이 제목에 귀속된 본문(비제목) 문단들(비어있지 않은 것만).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<StructureNode>,
}

/// 문서 구조 추출 결과.
#[derive(Debug, Clone, Serialize)]
pub struct StructureDoc {
    pub mode: &'static str,
    /// 봉투 키는 `nodeCount` 다. 최상위 레코드는 `main.rs` 가 이미 그 이름으로 올리는데
    /// 중첩된 `structure` 객체만 Rust 필드명 그대로 나가 **같은 값이 두 이름으로**
    /// 보였다. 별칭 계층이 없는 정적 매핑 언어(C#·Swift)에서는 이 필드가 사라진다.
    #[serde(rename = "nodeCount")]
    pub node_count: usize,
    /// 첫 제목 이전의 본문(서문).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub preamble: Vec<String>,
    pub roots: Vec<StructureNode>,
}

/// 제목 분류 결과.
struct Heading {
    level: u8,
    kind: &'static str,
    marker: String,
}

/// 개요(IR) 기반 제목 판정.
fn classify_outline(doc: &Document, para_shape_id: u16) -> Option<Heading> {
    let ps = doc.doc_info.para_shapes.get(para_shape_id as usize)?;
    match ps.head_type {
        HeadType::Outline | HeadType::Number => Some(Heading {
            level: ps.para_level + 1, // 0~6 → 1~7
            kind: "outline",
            marker: String::new(),
        }),
        _ => None,
    }
}

/// 법률 조문 텍스트 패턴 기반 제목 판정. 문단 텍스트 선두를 검사한다.
fn classify_clause(text: &str) -> Option<Heading> {
    let t = text.trim_start();
    if t.is_empty() {
        return None;
    }
    let chars: Vec<char> = t.chars().collect();

    // 항: 원문자 ①(U+2460)~⑳(U+2473) 로 시작.
    if let Some(&c0) = chars.first() {
        if ('\u{2460}'..='\u{2473}').contains(&c0) {
            return Some(Heading {
                level: 6,
                kind: "항",
                marker: c0.to_string(),
            });
        }
    }

    // 편/장/절/관/조: "제" + 숫자 + 단위.
    if chars[0] == '제' {
        let mut i = 1;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i > 1 && i < chars.len() {
            // 숫자와 단위 사이 공백 허용
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if let Some(&unit) = chars.get(j) {
                let (kind, level) = match unit {
                    '편' => ("편", 1u8),
                    '장' => ("장", 2),
                    '절' => ("절", 3),
                    '관' => ("관", 4),
                    '조' => ("조", 5),
                    _ => ("", 0),
                };
                if level > 0 {
                    // 가지번호는 `제1조`에서 끊지 않고 `의2`까지 marker로 보존한다. 조뿐 아니라
                    // 편/장/절/관에도 쓰이므로(`제5장의2`) 단위를 가리지 않는다. 뒤에 숫자가 없는
                    // `제1조의무`·`제3조의 규정`은 아래 `k > j + 2` 조건이 걸러낸다.
                    let mut marker_end = j;
                    if chars.get(j + 1) == Some(&'의') {
                        let mut k = j + 2;
                        while k < chars.len() && chars[k].is_ascii_digit() {
                            k += 1;
                        }
                        if k > j + 2 {
                            marker_end = k - 1;
                        }
                    }
                    let marker: String = chars[..=marker_end].iter().collect();
                    return Some(Heading {
                        level,
                        kind,
                        marker,
                    });
                }
            }
        }
    }

    // 호 후보: 숫자 + "." 또는 ")" 로 시작 (예: "1.", "1)").
    if chars[0].is_ascii_digit() {
        let mut i = 0;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if chars
            .get(i)
            .is_some_and(|delimiter| matches!(delimiter, '.' | ')'))
        {
            let marker: String = chars[..=i].iter().collect();
            return Some(Heading {
                level: 7,
                kind: "호",
                marker,
            });
        }
    }

    // 목 후보: 가~하 + "." 또는 ")" 로 시작 (예: "가.", "가)").
    const MOK: &str = "가나다라마바사아자차카타파하";
    if MOK.contains(chars[0])
        && chars
            .get(1)
            .is_some_and(|delimiter| matches!(delimiter, '.' | ')'))
    {
        return Some(Heading {
            level: 8,
            kind: "목",
            marker: chars[..=1].iter().collect(),
        });
    }

    None
}

/// 탭 뒤의 쪽번호로 끝나는 목차 행인지 판정한다.
fn has_toc_page_number_tail(text: &str) -> bool {
    text.trim_end().rsplit_once('\t').is_some_and(|(_, tail)| {
        let page = tail.trim();
        !page.is_empty() && page.chars().all(|c| c.is_ascii_digit())
    })
}

/// 조 marker 뒤의 조사형 본문 상호참조인지 판정한다.
fn starts_with_reference_particle(after_marker: &str) -> bool {
    const PARTICLES: &[&str] = &[
        "으로부터",
        "에서는",
        "에게서",
        "으로",
        "에서",
        "에는",
        "부터",
        "까지",
        "에게",
        "한테",
        "께서",
        "의",
        "에",
        "을",
        "를",
        "은",
        "는",
        "이",
        "가",
        "과",
        "와",
        "로",
        "도",
        "만",
    ];

    let tail = after_marker.trim_start();
    PARTICLES.iter().any(|particle| {
        tail.strip_prefix(particle).is_some_and(|rest| {
            rest.is_empty()
                || rest
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_whitespace() || matches!(c, ',' | '.' | ';' | ':' | '·'))
        })
    })
}

/// Number와 충돌할 때 문서 전체를 clause로 전환할 만큼 강한 제목 증거인지 판정한다.
///
/// 편·장·절·관은 보고서 목차에도 흔하므로 Number를 뒤집는 독립 증거로 쓰지 않는다. `조` 제목만
/// 후보로 삼되 탭+쪽번호 목차와 조사형 상호참조를 제외한다. explicit clause 분류에는 적용하지 않는다.
fn auto_clause_heading_allowed(text: &str, heading: &Heading) -> bool {
    if heading.kind != "조" || has_toc_page_number_tail(text) {
        return false;
    }

    let trimmed = text.trim_start();
    trimmed
        .strip_prefix(&heading.marker)
        .is_some_and(|tail| !starts_with_reference_particle(tail))
}

/// `auto` 모드의 문서 단위 분류.
///
/// `HeadType::Outline`은 작성자가 지정한 개요이므로 최우선한다. `HeadType::Number`와 충돌하면
/// 목차·상호참조가 아닌 `조` 제목만 clause 증거로 인정한다. Number가 없으면 기존처럼 clause로
/// 폴백하므로 편·장·절·관만 있는 explicit clause 문서의 기본 동작은 바뀌지 않는다.
fn select_auto_mode(doc: &Document) -> StructureMode {
    let mut has_number = false;
    let mut has_clause_article = false;

    for section in &doc.sections {
        for paragraph in &section.paragraphs {
            if let Some(para_shape) = doc
                .doc_info
                .para_shapes
                .get(paragraph.para_shape_id as usize)
            {
                match para_shape.head_type {
                    HeadType::Outline => return StructureMode::Outline,
                    HeadType::Number => has_number = true,
                    HeadType::None | HeadType::Bullet => {}
                }
            }

            // Outline은 문서 뒤쪽에도 있을 수 있어 끝까지 shape를 확인한다. 조 제목을 이미 찾았다면
            // 나머지 문단 텍스트는 조립하지 않아 auto 2-pass의 불필요한 할당을 줄인다.
            if !has_clause_article {
                let text = super::rendering::paragraph_text_with_equations(paragraph);
                if classify_clause(&text)
                    .is_some_and(|heading| auto_clause_heading_allowed(&text, &heading))
                {
                    has_clause_article = true;
                }
            }
        }
    }

    if has_clause_article {
        StructureMode::Clause
    } else if has_number {
        StructureMode::Outline
    } else {
        StructureMode::Clause
    }
}

/// 텍스트만으로 모호한 호/목 후보를 현재 법령 계층 문맥에서 수용할지 판정한다.
///
/// standalone `2022. 1.`이나 목차 `1. 개요`는 법령 marker와 같은 모양이므로, 부모 증거 없이
/// `호`/`목`으로 만들면 일반 문서를 과검출한다. strong marker는 그대로 수용하고, `호`는 열린
/// `조|항`, `목`은 열린 `호`가 있을 때만 구조 노드로 채택한다.
fn clause_heading_allowed(heading: &Heading, stack: &[StructureNode]) -> bool {
    match heading.kind {
        "호" => stack
            .iter()
            .any(|ancestor| matches!(ancestor.kind, "조" | "항")),
        "목" => stack.iter().any(|ancestor| ancestor.kind == "호"),
        _ => true,
    }
}

/// 문서 구조 트리를 구성한다.
pub fn build_structure(doc: &Document, mode: StructureMode) -> StructureDoc {
    let effective = match mode {
        StructureMode::Auto => select_auto_mode(doc),
        m => m,
    };

    let mut roots: Vec<StructureNode> = Vec::new();
    let mut stack: Vec<StructureNode> = Vec::new();
    let mut preamble: Vec<String> = Vec::new();
    let mut node_count = 0usize;

    // 스택 top(=부모)에 노드를 귀속.
    fn attach(node: StructureNode, stack: &mut Vec<StructureNode>, roots: &mut Vec<StructureNode>) {
        match stack.last_mut() {
            Some(parent) => parent.children.push(node),
            None => roots.push(node),
        }
    }

    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            // [#3413] `para.text` 는 수식 자리에 컨트롤 문자만 남긴다. 그대로 쓰면 수학·과학
            // 문서에서 발문과 선택지가 통째로 비어 나가고(수능 수학 20쪽: 선택지 33세트 전부
            // 빈 값) 종료 코드는 0 이라 파이프라인이 소실을 알 수 없다. export-text·search 와
            // 같은 조립기로 수식 script 를 합쳐 텍스트 표면 계약을 맞춘다.
            let para_text = super::rendering::paragraph_text_with_equations(para);
            let heading = match effective {
                StructureMode::Outline => classify_outline(doc, para.para_shape_id),
                StructureMode::Clause => classify_clause(&para_text)
                    .filter(|candidate| clause_heading_allowed(candidate, &stack)),
                StructureMode::Auto => unreachable!(),
            };

            match heading {
                Some(h) => {
                    // 같은/더 깊은 레벨은 닫고 부모에 귀속.
                    while stack.last().is_some_and(|t| t.level >= h.level) {
                        let n = stack.pop().unwrap();
                        attach(n, &mut stack, &mut roots);
                    }
                    stack.push(StructureNode {
                        level: h.level,
                        kind: h.kind,
                        marker: h.marker,
                        heading: para_text.clone(),
                        section: sec_idx,
                        paragraph: para_idx,
                        body: Vec::new(),
                        children: Vec::new(),
                    });
                    node_count += 1;
                }
                None => {
                    let text = para_text.trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    match stack.last_mut() {
                        Some(top) => top.body.push(text),
                        None => preamble.push(text),
                    }
                }
            }
        }
    }

    while let Some(n) = stack.pop() {
        attach(n, &mut stack, &mut roots);
    }

    StructureDoc {
        mode: match effective {
            StructureMode::Outline => "outline",
            StructureMode::Clause => "clause",
            StructureMode::Auto => "auto",
        },
        node_count,
        preamble,
        roots,
    }
}

impl DocumentCore {
    /// 문서 구조(개요/조문) 트리를 JSON으로 반환한다.
    ///
    /// `mode`: `"auto"` | `"outline"` | `"clause"` (인식 불가 시 `auto` 폴백).
    /// 파서/렌더 무변경의 읽기 전용 질의. 사이드바 목차 네비게이션용.
    pub fn get_structure_native(&self, mode: &str) -> Result<String, HwpError> {
        let mode = StructureMode::parse(mode).unwrap_or(StructureMode::Auto);
        let st = build_structure(&self.document, mode);
        serde_json::to_string(&st).map_err(|error| {
            HwpError::RenderError(format!("문서 구조 JSON 직렬화에 실패했습니다: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Section;
    use crate::model::paragraph::Paragraph;
    use crate::model::style::ParaShape;

    fn document_with_paragraphs(texts: &[&str]) -> Document {
        let mut doc = Document::default();
        doc.doc_info.para_shapes.push(ParaShape::default());
        doc.sections.push(Section {
            paragraphs: texts
                .iter()
                .map(|text| Paragraph {
                    text: (*text).to_string(),
                    ..Paragraph::new_empty()
                })
                .collect(),
            ..Section::default()
        });
        doc
    }

    #[test]
    fn clause_detects_jo_hang_ho_mok() {
        let cases = [
            ("제1조(목적)", "조", 5u8),
            ("제12장 보칙", "장", 2),
            ("①사업자는", "항", 6),
            ("1. 첫째 호", "호", 7),
            ("1) 첫째 호", "호", 7),
            ("가. 첫째 목", "목", 8),
            ("가) 첫째 목", "목", 8),
        ];
        for (text, kind, level) in cases {
            let h = classify_clause(text).unwrap_or_else(|| panic!("미검출: {text}"));
            assert_eq!(h.kind, kind, "{text}");
            assert_eq!(h.level, level, "{text}");
        }
    }

    #[test]
    fn clause_ignores_plain_text() {
        assert!(classify_clause("일반 문장입니다").is_none());
        assert!(classify_clause("").is_none());
        assert!(classify_clause("제목 없음").is_none()); // "제"+비숫자
    }

    #[test]
    fn clause_marker_extracted() {
        assert_eq!(classify_clause("제3조 적용범위").unwrap().marker, "제3조");
        assert_eq!(
            classify_clause("제1조의2(가지번호)").unwrap().marker,
            "제1조의2"
        );
        assert_eq!(classify_clause("②다음").unwrap().marker, "②");
        assert_eq!(classify_clause("12) 다음").unwrap().marker, "12)");
        assert_eq!(classify_clause("가) 다음").unwrap().marker, "가)");
    }

    #[test]
    fn clause_marker_keeps_variant_number_for_every_unit() {
        // 가지번호는 조 전용이 아니다.
        for (text, kind, marker) in [
            ("제5장의2 국세환급금", "장", "제5장의2"),
            ("제2절의3 특례", "절", "제2절의3"),
            ("제1편의2 총칙", "편", "제1편의2"),
            ("제4관의2 보칙", "관", "제4관의2"),
            ("제7조의4(적용)", "조", "제7조의4"),
        ] {
            let h = classify_clause(text).unwrap_or_else(|| panic!("미검출: {text}"));
            assert_eq!((h.kind, h.marker.as_str()), (kind, marker), "{text}");
        }

        // `의` 뒤에 숫자가 없으면 가지번호가 아니다.
        assert_eq!(classify_clause("제1조의무 규정").unwrap().marker, "제1조");
        assert_eq!(
            classify_clause("제3조의 규정에 따라").unwrap().marker,
            "제3조"
        );
        assert_eq!(classify_clause("제2장의 적용").unwrap().marker, "제2장");
        // 가지번호 뒤에 이어지는 조사도 marker에 포함하지 않는다.
        assert_eq!(
            classify_clause("제3조의2의 규정에 따라").unwrap().marker,
            "제3조의2"
        );
    }

    #[test]
    fn clause_builds_variant_hierarchy_with_parent_context() {
        let doc =
            document_with_paragraphs(&["제1조의2(정의)", "① 첫째 항", "1) 첫째 호", "가) 첫째 목"]);
        let structure = build_structure(&doc, StructureMode::Clause);

        assert_eq!(structure.node_count, 4);
        assert_eq!(structure.roots.len(), 1);
        let article = &structure.roots[0];
        assert_eq!((article.kind, article.marker.as_str()), ("조", "제1조의2"));
        let paragraph = &article.children[0];
        assert_eq!((paragraph.kind, paragraph.marker.as_str()), ("항", "①"));
        let item = &paragraph.children[0];
        assert_eq!((item.kind, item.marker.as_str()), ("호", "1)"));
        let subitem = &item.children[0];
        assert_eq!((subitem.kind, subitem.marker.as_str()), ("목", "가)"));
    }

    #[test]
    fn clause_rejects_standalone_number_candidates() {
        let doc =
            document_with_paragraphs(&["2022. 1.", "1. 일반 번호 목록", "가) 일반 하위 목록"]);
        let structure = build_structure(&doc, StructureMode::Clause);

        assert_eq!(structure.node_count, 0);
        assert_eq!(
            structure.preamble,
            vec!["2022. 1.", "1. 일반 번호 목록", "가) 일반 하위 목록"]
        );
    }
}
