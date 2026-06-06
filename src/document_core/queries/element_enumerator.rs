//! 문서 요소 열거자 (Element Enumerator)
//!
//! 자연스러운 HWP IR 트리(`Section → Paragraph → Control` 전체 taxonomy)를 DFS 순회하여
//! 모든 주소 지정 가능 요소를 평탄한 JSON 배열로 반환한다.
//!
//! ecrits 의 `doc.find` 추상화를 위한 단일 진입점이며, WASM(`enumerateElements`)과
//! ehwp NIF(`{"q":"elements"}`) 양쪽에서 동일 JSON 을 호출한다.
//!
//! 각 노드 스키마:
//! ```json
//! { "type": "<kind>", "ref": "<json-ref-string>", "text": "<text or ''>",
//!   "row": <n?>, "col": <n?> }
//! ```
//! - `type`: 소문자 요소 종류 (paragraph / cell / 각 Control variant)
//! - `ref`: 이 코드베이스의 기존 위치 ref 와 정합하는 JSON 객체 문자열
//! - `text`: 요소의 텍스트 (없으면 "")
//! - `row`/`col`: 표 셀에 한해 그리드 주소 (그 외 노드는 생략)
//!
//! ref 형태:
//! - 본문 문단:    `{"section":S,"paragraph":P,"offset":0}`
//! - 표 셀:        `{"section":S,"paragraph":P,"offset":0,"cell":{"parentParaIndex":P,"controlIndex":C,"cellIndex":I,"cellParaIndex":CP}}`
//! - 비-셀 컨트롤: `{"section":S,"paragraph":P,"control":C}`
//! - 컨테이너(머리말/꼬리말/각주/미주) 내부 문단:
//!   `{"section":S,"paragraph":P,"control":C,"subParagraph":SP}`
//!
//! 모든 컨트롤 추출은 패닉하지 않도록 방어적으로 작성되어 한 컨트롤이 잘못되어도
//! 전체 순회가 중단되지 않는다.

use crate::document_core::helpers::{get_textbox_from_shape, json_escape};
use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;

/// 단일 열거 노드 (직렬화 직전 중간 표현).
struct ElementNode {
    kind: &'static str,
    /// 이미 직렬화된 JSON 객체 문자열 (예: `{"section":0,...}`)
    ref_json: String,
    text: String,
    row: Option<u16>,
    col: Option<u16>,
}

impl ElementNode {
    fn to_json(&self) -> String {
        let mut s = String::with_capacity(64 + self.text.len());
        s.push('{');
        s.push_str("\"type\":\"");
        s.push_str(self.kind);
        s.push_str("\",\"ref\":");
        // ref_json 은 이미 유효한 JSON 객체 문자열이므로 escape 하지 않고 그대로 임베드.
        s.push_str(&self.ref_json);
        s.push_str(",\"text\":\"");
        s.push_str(&json_escape(&self.text));
        s.push('"');
        if let Some(r) = self.row {
            s.push_str(",\"row\":");
            s.push_str(&r.to_string());
        }
        if let Some(c) = self.col {
            s.push_str(",\"col\":");
            s.push_str(&c.to_string());
        }
        s.push('}');
        s
    }
}

/// `Control` variant → `type` 문자열 (lowercase kind).
fn control_kind(ctrl: &Control) -> &'static str {
    match ctrl {
        Control::SectionDef(_) => "section_def",
        Control::ColumnDef(_) => "column_def",
        Control::Table(_) => "table",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "picture",
        Control::Header(_) => "header",
        Control::Footer(_) => "footer",
        Control::Footnote(_) => "footnote",
        Control::Endnote(_) => "endnote",
        Control::AutoNumber(_) => "auto_number",
        Control::NewNumber(_) => "new_number",
        Control::PageNumberPos(_) => "page_number_pos",
        Control::Bookmark(_) => "bookmark",
        Control::Hyperlink(_) => "hyperlink",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "char_overlap",
        Control::PageHide(_) => "page_hide",
        Control::HiddenComment(_) => "hidden_comment",
        Control::Equation(_) => "equation",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

/// 컨트롤 노드의 대표 텍스트(있으면)를 추출한다. 본문 흐름에 텍스트가 없는
/// 컨트롤은 빈 문자열을 반환한다.
fn control_text(ctrl: &Control) -> String {
    match ctrl {
        // 텍스트가 있는 인라인/캡션 컨트롤
        Control::Field(f) => {
            // 누름틀 안내문 → 필드 이름 → command 순으로 폴백.
            if let Some(g) = f.guide_text() {
                g.to_string()
            } else if let Some(n) = f.field_name() {
                n.to_string()
            } else if !f.command.is_empty() {
                f.command.clone()
            } else {
                String::new()
            }
        }
        Control::Form(form) => {
            // 캡션(버튼/체크박스) 또는 텍스트(콤보/입력상자).
            if !form.caption.is_empty() {
                form.caption.clone()
            } else if !form.text.is_empty() {
                form.text.clone()
            } else {
                form.name.clone()
            }
        }
        Control::Hyperlink(h) => {
            if !h.text.is_empty() {
                h.text.clone()
            } else {
                h.url.clone()
            }
        }
        Control::Bookmark(b) => b.name.clone(),
        Control::Ruby(r) => r.ruby_text.clone(),
        Control::Equation(e) => e.script.clone(),
        Control::CharOverlap(c) => c.chars.iter().collect(),
        Control::HiddenComment(c) => concat_paragraph_text(&c.paragraphs),
        // 컨테이너 컨트롤의 텍스트는 자식 문단들을 연결한 값.
        Control::Table(t) => {
            let mut parts: Vec<String> = Vec::new();
            for cell in &t.cells {
                let ct = concat_paragraph_text(&cell.paragraphs);
                if !ct.is_empty() {
                    parts.push(ct);
                }
            }
            parts.join(" ")
        }
        Control::Header(h) => concat_paragraph_text(&h.paragraphs),
        Control::Footer(f) => concat_paragraph_text(&f.paragraphs),
        Control::Footnote(f) => concat_paragraph_text(&f.paragraphs),
        Control::Endnote(e) => concat_paragraph_text(&e.paragraphs),
        Control::Shape(shape) => {
            // 글상자가 있으면 그 텍스트.
            match get_textbox_from_shape(shape) {
                Some(tb) => concat_paragraph_text(&tb.paragraphs),
                None => String::new(),
            }
        }
        // 텍스트가 없는 컨트롤.
        _ => String::new(),
    }
}

/// 문단 목록의 텍스트를 개행으로 연결한다 (빈 문단 제외).
fn concat_paragraph_text(paras: &[Paragraph]) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(paras.len());
    for p in paras {
        if !p.text.is_empty() {
            parts.push(p.text.as_str());
        }
    }
    parts.join("\n")
}

/// 본문 문단 ref.
fn body_para_ref(section: usize, para: usize) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"offset\":0}}",
        section, para
    )
}

/// 비-셀 컨트롤 ref.
fn control_ref(section: usize, para: usize, control: usize) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"control\":{}}}",
        section, para, control
    )
}

/// 표 셀 ref (getCellInfo indexing 과 동일: parentParaIndex/controlIndex/cellIndex/cellParaIndex).
fn cell_ref(
    section: usize,
    para: usize,
    control: usize,
    cell_idx: usize,
    cell_para_idx: usize,
) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"offset\":0,\"cell\":{{\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}}}",
        section, para, para, control, cell_idx, cell_para_idx
    )
}

/// 컨테이너(머리말/꼬리말/각주/미주) 내부 sub-문단 ref.
/// 컨테이너 컨트롤을 `control` 로 식별하고 내부 문단 인덱스를 `subParagraph` 로 부여한다.
fn sub_para_ref(section: usize, para: usize, control: usize, sub_para: usize) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"control\":{},\"subParagraph\":{}}}",
        section, para, control, sub_para
    )
}

/// `(controlIndex, cellIndex, cellParaIndex)` 경로를 JSON 배열 문자열로 직렬화한다
/// (parse_cell_path 와 정합).
fn cell_path_json(cell_path: &[(usize, usize, usize)]) -> String {
    let mut s = String::from("[");
    for (i, (ci, cell, cp)) in cell_path.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
            ci, cell, cp
        ));
    }
    s.push(']');
    s
}

/// 셀/글상자 내부에 중첩된 비-셀 컨트롤 ref.
/// `cellPath` 로 컨테이너 경로를 부여하고 `control` 로 중첩 문단 내 컨트롤 인덱스를 부여한다.
fn nested_control_ref(
    section: usize,
    para: usize,
    cell_path: &[(usize, usize, usize)],
    control: usize,
) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"control\":{},\"cellPath\":{}}}",
        section,
        para,
        control,
        cell_path_json(cell_path)
    )
}

/// 셀/글상자 내부에 중첩된 표의 셀 ref (cellPath 마지막 세그먼트가 해당 셀).
fn nested_cell_ref(section: usize, para: usize, cell_path: &[(usize, usize, usize)]) -> String {
    format!(
        "{{\"section\":{},\"paragraph\":{},\"offset\":0,\"cellPath\":{}}}",
        section,
        para,
        cell_path_json(cell_path)
    )
}

impl DocumentCore {
    /// 자연스러운 IR 트리를 DFS 순회하여 모든 주소 지정 가능 요소를 평탄한
    /// JSON 배열로 반환한다.
    ///
    /// 반환 형식:
    /// `[{ "type": "<kind>", "ref": "<json-ref-string>", "text": "<text or ''>",
    ///    "row": <n?>, "col": <n?> }, …]`
    ///
    /// WASM(`enumerateElements`)과 ehwp NIF(`{"q":"elements"}`)가 공유하는 진입점.
    pub fn enumerate_elements(&self) -> String {
        let mut nodes: Vec<ElementNode> = Vec::new();

        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_idx, para) in section.paragraphs.iter().enumerate() {
                // 1. 본문 문단 노드.
                nodes.push(ElementNode {
                    kind: "paragraph",
                    ref_json: body_para_ref(sec_idx, para_idx),
                    text: para.text.clone(),
                    row: None,
                    col: None,
                });

                // 2. 문단 내 각 컨트롤.
                for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                    // per-control 추출은 방어적 — 패닉이 발생할 수 없는 순수 데이터
                    // 접근만 사용하므로 별도 catch_unwind 불필요. 컨테이너 재귀는
                    // 빈 자식 목록을 안전하게 처리한다.
                    self.emit_control(&mut nodes, sec_idx, para_idx, ctrl_idx, ctrl);
                }
            }
        }

        let mut out = String::from("[");
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&node.to_json());
        }
        out.push(']');
        out
    }

    /// 단일 컨트롤 노드를 emit 하고, 컨테이너성 컨트롤은 자식을 재귀 emit 한다.
    fn emit_control(
        &self,
        nodes: &mut Vec<ElementNode>,
        sec_idx: usize,
        para_idx: usize,
        ctrl_idx: usize,
        ctrl: &Control,
    ) {
        // 컨트롤 자체 노드.
        nodes.push(ElementNode {
            kind: control_kind(ctrl),
            ref_json: control_ref(sec_idx, para_idx, ctrl_idx),
            text: control_text(ctrl),
            row: None,
            col: None,
        });

        // 컨테이너 컨트롤 자식 재귀.
        match ctrl {
            Control::Table(table) => {
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    let cell_text = concat_paragraph_text(&cell.paragraphs);
                    // 셀 노드 — getCellInfo 와 동일한 cellIndex(0..cells.len()) 사용.
                    // cellParaIndex 는 셀의 첫 문단(0)을 가리킨다.
                    nodes.push(ElementNode {
                        kind: "cell",
                        ref_json: cell_ref(sec_idx, para_idx, ctrl_idx, cell_idx, 0),
                        text: cell_text,
                        row: Some(cell.row),
                        col: Some(cell.col),
                    });
                    // 셀 내부 문단들에 중첩된 컨트롤(그림/필드/중첩 표 등)을 재귀 emit.
                    // cellPath = [(controlIndex, cellIndex, cellParaIndex)] 누적.
                    for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                        let path = vec![(ctrl_idx, cell_idx, cp_idx)];
                        self.emit_nested_controls(nodes, sec_idx, para_idx, &path, cp);
                    }
                }
            }
            Control::Header(h) => {
                self.emit_sub_paragraphs(nodes, sec_idx, para_idx, ctrl_idx, &h.paragraphs);
            }
            Control::Footer(f) => {
                self.emit_sub_paragraphs(nodes, sec_idx, para_idx, ctrl_idx, &f.paragraphs);
            }
            Control::Footnote(f) => {
                self.emit_sub_paragraphs(nodes, sec_idx, para_idx, ctrl_idx, &f.paragraphs);
            }
            Control::Endnote(e) => {
                self.emit_sub_paragraphs(nodes, sec_idx, para_idx, ctrl_idx, &e.paragraphs);
            }
            Control::Shape(shape) => {
                // 글상자(텍스트박스)가 있으면 내부 문단을 sub-문단으로, 그 안의 컨트롤을 재귀 emit.
                if let Some(tb) = get_textbox_from_shape(shape) {
                    self.emit_sub_paragraphs(nodes, sec_idx, para_idx, ctrl_idx, &tb.paragraphs);
                    for (cp_idx, cp) in tb.paragraphs.iter().enumerate() {
                        // 글상자는 cellIndex 0 의 단일 컨테이너로 취급 (search_query 와 동일).
                        let path = vec![(ctrl_idx, 0, cp_idx)];
                        self.emit_nested_controls(nodes, sec_idx, para_idx, &path, cp);
                    }
                }
            }
            _ => {}
        }
    }

    /// 컨테이너 컨트롤의 내부 문단들을 `paragraph` 노드로 emit 한다.
    fn emit_sub_paragraphs(
        &self,
        nodes: &mut Vec<ElementNode>,
        sec_idx: usize,
        para_idx: usize,
        ctrl_idx: usize,
        sub_paras: &[Paragraph],
    ) {
        for (sub_idx, sub_para) in sub_paras.iter().enumerate() {
            nodes.push(ElementNode {
                kind: "paragraph",
                ref_json: sub_para_ref(sec_idx, para_idx, ctrl_idx, sub_idx),
                text: sub_para.text.clone(),
                row: None,
                col: None,
            });
        }
    }

    /// 셀/글상자 등 중첩 문단 내부의 컨트롤을 emit 한다.
    ///
    /// `cell_path` 는 `(controlIndex, cellIndex, cellParaIndex)` 의 누적 경로로,
    /// 이 코드베이스의 중첩 표 addressing(parse_cell_path) 과 정합한다. ref 는
    /// `{"section":S,"paragraph":P,"control":NC,"cellPath":[{...}]}` 형태로 부여하며,
    /// 중첩 표는 한 단계 더 재귀한다.
    fn emit_nested_controls(
        &self,
        nodes: &mut Vec<ElementNode>,
        sec_idx: usize,
        para_idx: usize,
        cell_path: &[(usize, usize, usize)],
        cp: &Paragraph,
    ) {
        // 깊이 가드: 정상 HWP 는 표 중첩이 수 단계를 넘지 않는다. 손상/악성 파일의
        // 비정상 재귀로 인한 스택 오버플로를 방지하기 위해 깊이를 제한한다 (total 보장).
        const MAX_NEST_DEPTH: usize = 16;
        if cell_path.len() > MAX_NEST_DEPTH {
            return;
        }
        for (nctrl_idx, nctrl) in cp.controls.iter().enumerate() {
            nodes.push(ElementNode {
                kind: control_kind(nctrl),
                ref_json: nested_control_ref(sec_idx, para_idx, cell_path, nctrl_idx),
                text: control_text(nctrl),
                row: None,
                col: None,
            });
            // 중첩 표는 셀까지 한 단계 더 내려가고, 셀 문단 내부 컨트롤도 재귀 emit.
            if let Control::Table(table) = nctrl {
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    let mut deeper = cell_path.to_vec();
                    deeper.push((nctrl_idx, cell_idx, 0));
                    nodes.push(ElementNode {
                        kind: "cell",
                        ref_json: nested_cell_ref(sec_idx, para_idx, &deeper),
                        text: concat_paragraph_text(&cell.paragraphs),
                        row: Some(cell.row),
                        col: Some(cell.col),
                    });
                    for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                        let mut child_path = cell_path.to_vec();
                        child_path.push((nctrl_idx, cell_idx, cp_idx));
                        self.emit_nested_controls(nodes, sec_idx, para_idx, &child_path, cp);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::wasm_api::HwpDocument;
    use std::fs;
    use std::path::Path;

    fn load(rel: &str) -> HwpDocument {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
        HwpDocument::from_bytes(&bytes).expect("parse")
    }

    /// pic-in-table-01.hwp 는 본문 문단 + 표 + 셀 + (셀 안) 그림을 포함한다.
    #[test]
    fn enumerate_contains_table_cell_paragraph_picture() {
        let doc = load("samples/pic-in-table-01.hwp");
        let json = doc.enumerate_elements();

        assert!(json.starts_with('['), "JSON must be an array");
        assert!(
            json.contains("\"type\":\"paragraph\""),
            "must contain a paragraph node"
        );
        assert!(
            json.contains("\"type\":\"table\""),
            "must contain a table node"
        );
        assert!(
            json.contains("\"type\":\"cell\""),
            "must contain a cell node"
        );
        assert!(
            json.contains("\"type\":\"picture\""),
            "must contain a picture node (pic-in-table fixture)"
        );
        // 셀 노드에는 row/col 그리드 주소가 있어야 한다.
        assert!(
            json.contains("\"row\":") && json.contains("\"col\":"),
            "cell nodes must carry row/col grid address"
        );
        // 셀 ref 는 getCellInfo indexing(cellIndex/cellParaIndex)을 따라야 한다.
        assert!(
            json.contains("\"cellIndex\":") && json.contains("\"cellParaIndex\":"),
            "cell ref must match getCellInfo indexing"
        );
    }

    /// field-01.hwp 는 누름틀 필드를 포함한다.
    #[test]
    fn enumerate_contains_field() {
        let doc = load("samples/field-01.hwp");
        let json = doc.enumerate_elements();
        assert!(
            json.contains("\"type\":\"field\""),
            "must contain a field node (field-01 fixture)"
        );
    }

    /// 기본 표 fixture 에서 table/cell/paragraph 가 모두 나와야 한다.
    #[test]
    fn enumerate_basic_table_fixture() {
        let doc = load("samples/table-001.hwp");
        let json = doc.enumerate_elements();
        assert!(json.contains("\"type\":\"paragraph\""));
        assert!(json.contains("\"type\":\"table\""));
        assert!(json.contains("\"type\":\"cell\""));
    }
}
