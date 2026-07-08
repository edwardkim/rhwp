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
    /// 개요(Outline/Number head_type) 있으면 개요, 없으면 조문 패턴.
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
                    let marker: String = chars[..=j].iter().collect();
                    return Some(Heading {
                        level,
                        kind,
                        marker,
                    });
                }
            }
        }
    }

    // 호: 숫자 + "." 로 시작 (예: "1.").
    if chars[0].is_ascii_digit() {
        let mut i = 0;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if chars.get(i) == Some(&'.') {
            let marker: String = chars[..=i].iter().collect();
            return Some(Heading {
                level: 7,
                kind: "호",
                marker,
            });
        }
    }

    // 목: 가~하 + "." 로 시작 (예: "가.").
    const MOK: &str = "가나다라마바사아자차카타파하";
    if MOK.contains(chars[0]) && chars.get(1) == Some(&'.') {
        return Some(Heading {
            level: 8,
            kind: "목",
            marker: chars[..=1].iter().collect(),
        });
    }

    None
}

/// 문서가 개요(Outline/Number) head_type 을 하나라도 쓰는지.
fn has_outline(doc: &Document) -> bool {
    doc.sections.iter().any(|s| {
        s.paragraphs.iter().any(|p| {
            doc.doc_info
                .para_shapes
                .get(p.para_shape_id as usize)
                .is_some_and(|ps| matches!(ps.head_type, HeadType::Outline | HeadType::Number))
        })
    })
}

/// 문서 구조 트리를 구성한다.
pub fn build_structure(doc: &Document, mode: StructureMode) -> StructureDoc {
    let effective = match mode {
        StructureMode::Auto => {
            if has_outline(doc) {
                StructureMode::Outline
            } else {
                StructureMode::Clause
            }
        }
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
            let heading = match effective {
                StructureMode::Outline => classify_outline(doc, para.para_shape_id),
                StructureMode::Clause => classify_clause(&para.text),
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
                        heading: para.text.clone(),
                        section: sec_idx,
                        paragraph: para_idx,
                        body: Vec::new(),
                        children: Vec::new(),
                    });
                    node_count += 1;
                }
                None => {
                    let text = para.text.trim().to_string();
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

    #[test]
    fn clause_detects_jo_hang_ho_mok() {
        let cases = [
            ("제1조(목적)", "조", 5u8),
            ("제12장 보칙", "장", 2),
            ("①사업자는", "항", 6),
            ("1. 첫째 호", "호", 7),
            ("가. 첫째 목", "목", 8),
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
        assert_eq!(classify_clause("②다음").unwrap().marker, "②");
    }
}
