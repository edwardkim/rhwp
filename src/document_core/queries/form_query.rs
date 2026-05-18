//! 양식 개체 조회/설정 API (Task 233)
//!
//! 렌더 트리에서 양식 개체를 좌표로 찾거나, 문서 트리에서 직접 값을 조회/설정한다.

use crate::document_core::DocumentCore;
use crate::model::control::{Control, FormType};
use crate::model::table::Table;
use crate::renderer::render_tree::{FormObjectNode, RenderNode, RenderNodeType};

impl DocumentCore {
    /// 페이지 좌표에서 양식 개체를 찾는다.
    ///
    /// 렌더 트리를 순회하며 FormObject 노드의 bbox와 좌표 충돌 검사.
    /// 반환: JSON 문자열
    pub fn get_form_object_at_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, crate::error::HwpError> {
        let tree = self.build_page_tree_cached(page_num)?;

        if let Some((form, bbox)) = find_form_node_at(&tree.root, x, y) {
            let form_type_str = form_type_to_str(form.form_type);
            // 셀 내부 위치 정보 직렬화
            let cell_loc_json = if let Some((tpi, tci, ci_idx, cp_idx)) = form.cell_location {
                format!(
                    r#","inCell":true,"tablePara":{},"tableCi":{},"cellIdx":{},"cellPara":{}"#,
                    tpi, tci, ci_idx, cp_idx
                )
            } else {
                String::new()
            };
            // sec/para는 최상위 문단 인덱스로 반환
            // cell_location이 있으면 table_para_index를 para로 사용
            let (ret_para, ret_ci) = if let Some((tpi, _tci, _ci_idx, _cp_idx)) = form.cell_location
            {
                (tpi, form.control_index)
            } else {
                (form.para_index, form.control_index)
            };
            Ok(format!(
                r#"{{"found":true,"sec":{},"para":{},"ci":{},"formType":"{}","name":"{}","value":{},"caption":"{}","text":"{}","bbox":{{"x":{},"y":{},"w":{},"h":{}}}{}}}"#,
                form.section_index,
                ret_para,
                ret_ci,
                form_type_str,
                escape_json(&form.name),
                form.value,
                escape_json(&form.caption),
                escape_json(&form.text),
                bbox.0,
                bbox.1,
                bbox.2,
                bbox.3,
                cell_loc_json,
            ))
        } else {
            Ok(r#"{"found":false}"#.to_string())
        }
    }

    /// 양식 개체 값을 조회한다.
    pub fn get_form_value_native(
        &self,
        sec: usize,
        para: usize,
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let control = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ci));

        match control {
            Some(Control::Form(f)) => {
                let form_type_str = form_type_to_str(f.form_type);
                Ok(format!(
                    r#"{{"ok":true,"formType":"{}","name":"{}","value":{},"text":"{}","caption":"{}","enabled":{}}}"#,
                    form_type_str,
                    escape_json(&f.name),
                    f.value,
                    escape_json(&f.text),
                    escape_json(&f.caption),
                    f.enabled,
                ))
            }
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }

    /// 양식 개체 값을 설정한다 (최상위 문단의 Form).
    ///
    /// value_json: `{"value":1}` 또는 `{"text":"입력값"}` 또는 둘 다
    pub fn set_form_value_native(
        &mut self,
        sec: usize,
        para: usize,
        ci: usize,
        value_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let control = self
            .document
            .sections
            .get_mut(sec)
            .and_then(|s| s.paragraphs.get_mut(para))
            .and_then(|p| p.controls.get_mut(ci));

        match control {
            Some(Control::Form(f)) => {
                apply_form_value(f, value_json);
                self.recompose_section(sec);
                Ok(r#"{"ok":true}"#.to_string())
            }
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }

    /// 셀 내부 양식 개체 값을 설정한다.
    ///
    /// table_para: 표를 포함한 최상위 문단 인덱스
    /// table_ci: 표 컨트롤 인덱스
    /// cell_idx: 셀 인덱스
    /// cell_para: 셀 내 문단 인덱스
    /// form_ci: 셀 내 양식 컨트롤 인덱스
    pub fn set_form_value_in_cell_native(
        &mut self,
        sec: usize,
        table_para: usize,
        table_ci: usize,
        cell_idx: usize,
        cell_para: usize,
        form_ci: usize,
        value_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let form = self
            .document
            .sections
            .get_mut(sec)
            .and_then(|s| s.paragraphs.get_mut(table_para))
            .and_then(|p| p.controls.get_mut(table_ci))
            .and_then(|c| {
                if let Control::Table(ref mut t) = c {
                    Some(t.as_mut())
                } else {
                    None
                }
            })
            .and_then(|t: &mut Table| t.cells.get_mut(cell_idx))
            .and_then(|cell| cell.paragraphs.get_mut(cell_para))
            .and_then(|p| p.controls.get_mut(form_ci))
            .and_then(|c| {
                if let Control::Form(ref mut f) = c {
                    Some(f)
                } else {
                    None
                }
            });

        match form {
            Some(f) => {
                apply_form_value(f, value_json);
                self.recompose_section(sec);
                Ok(r#"{"ok":true}"#.to_string())
            }
            None => Ok(r#"{"ok":false,"error":"cell form not found"}"#.to_string()),
        }
    }

    /// 양식 개체 상세 정보를 반환한다 (properties HashMap 포함).
    /// ComboBox인 경우 스크립트에서 추출한 항목 목록도 포함.
    pub fn get_form_object_info_native(
        &self,
        sec: usize,
        para: usize,
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let control = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ci));

        match control {
            Some(Control::Form(f)) => {
                let form_type_str = form_type_to_str(f.form_type);
                // properties를 JSON 객체로 직렬화
                let props: Vec<String> = f
                    .properties
                    .iter()
                    .map(|(k, v)| format!(r#""{}":"{}""#, escape_json(k), escape_json(v)))
                    .collect();
                let props_json = format!("{{{}}}", props.join(","));

                // ComboBox: 스크립트에서 InsertString 항목 추출
                let items_json = if f.form_type == FormType::ComboBox {
                    let items =
                        extract_combobox_items_from_script(&self.document.extra_streams, &f.name);
                    if items.is_empty() {
                        "[]".to_string()
                    } else {
                        let arr: Vec<String> = items
                            .iter()
                            .map(|s| format!(r#""{}""#, escape_json(s)))
                            .collect();
                        format!("[{}]", arr.join(","))
                    }
                } else {
                    "[]".to_string()
                };

                Ok(format!(
                    r#"{{"ok":true,"formType":"{}","name":"{}","value":{},"text":"{}","caption":"{}","enabled":{},"width":{},"height":{},"foreColor":{},"backColor":{},"properties":{},"items":{}}}"#,
                    form_type_str,
                    escape_json(&f.name),
                    f.value,
                    escape_json(&f.text),
                    escape_json(&f.caption),
                    f.enabled,
                    f.width,
                    f.height,
                    f.fore_color,
                    f.back_color,
                    props_json,
                    items_json,
                ))
            }
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }
}

/// form value/text/caption 적용 헬퍼
fn apply_form_value(f: &mut crate::model::control::FormObject, value_json: &str) {
    if let Some(v) = extract_json_int(value_json, "value") {
        f.value = v;
    }
    if let Some(t) = extract_json_string(value_json, "text") {
        f.text = t;
    }
    if let Some(c) = extract_json_string(value_json, "caption") {
        f.caption = c;
    }
}

/// 렌더 트리를 재귀 순회하여 좌표에 해당하는 FormObject 노드를 찾는다.
fn find_form_node_at(
    node: &RenderNode,
    x: f64,
    y: f64,
) -> Option<(&FormObjectNode, (f64, f64, f64, f64))> {
    // 자식 먼저 (더 구체적인 노드 우선)
    for child in &node.children {
        if let Some(result) = find_form_node_at(child, x, y) {
            return Some(result);
        }
    }
    // 현재 노드 확인
    if let RenderNodeType::FormObject(ref form) = node.node_type {
        let b = &node.bbox;
        if x >= b.x && x <= b.x + b.width && y >= b.y && y <= b.y + b.height {
            return Some((form, (b.x, b.y, b.width, b.height)));
        }
    }
    None
}

fn form_type_to_str(ft: FormType) -> &'static str {
    match ft {
        FormType::PushButton => "PushButton",
        FormType::CheckBox => "CheckBox",
        FormType::ComboBox => "ComboBox",
        FormType::RadioButton => "RadioButton",
        FormType::Edit => "Edit",
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 간단한 JSON에서 정수값 추출: `"key":123`
fn extract_json_int(json: &str, key: &str) -> Option<i32> {
    let pattern = format!(r#""{}":"#, key);
    if let Some(pos) = json.find(&pattern) {
        let start = pos + pattern.len();
        let rest = &json[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    } else {
        None
    }
}

/// 간단한 JSON에서 문자열값 추출: `"key":"value"`
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}":""#, key);
    if let Some(pos) = json.find(&pattern) {
        let start = pos + pattern.len();
        let rest = &json[start..];
        // 이스케이프되지 않은 닫는 따옴표 찾기
        let mut end = 0;
        let chars: Vec<char> = rest.chars().collect();
        while end < chars.len() {
            if chars[end] == '"' && (end == 0 || chars[end - 1] != '\\') {
                break;
            }
            end += 1;
        }
        Some(chars[..end].iter().collect())
    } else {
        None
    }
}

/// extra_streams에서 Scripts/DefaultJScript 스트림을 찾아 디코딩한다.
/// HWP 스크립트는 zlib 압축 + UTF-16LE로 저장됨.
fn decode_hwp_script(extra_streams: &[(String, Vec<u8>)]) -> Option<String> {
    let data = extra_streams
        .iter()
        .find(|(path, _)| path == "/Scripts/DefaultJScript" || path == "Scripts/DefaultJScript")
        .map(|(_, data)| data)?;

    if data.is_empty() {
        return None;
    }

    // zlib 해제 (raw deflate, no header)
    use std::io::Read;
    let mut decoder = flate2::read::DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_err() {
        return None;
    }

    // UTF-16LE 디코딩
    if decompressed.len() < 2 {
        return None;
    }
    let u16s: Vec<u16> = decompressed
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(String::from_utf16_lossy(&u16s))
}

/// 스크립트에서 ComboBox InsertString 패턴을 추출하여 항목 목록을 반환한다.
///
/// 패턴: `컨트롤이름.InsertString("항목텍스트", 인덱스);`
fn extract_combobox_items_from_script(
    extra_streams: &[(String, Vec<u8>)],
    control_name: &str,
) -> Vec<String> {
    let script = match decode_hwp_script(extra_streams) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut items: Vec<(usize, String)> = Vec::new();

    // 패턴: ControlName.InsertString("text", index)
    // 또는: ControlName.InsertString("text",index)
    let prefix = format!("{}.InsertString(", control_name);
    for line in script.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            // rest: `"봄",0);` 또는 `"봄", 0);`
            if let Some((text, idx)) = parse_insert_string_args(rest) {
                items.push((idx, text));
            }
        }
    }

    // 인덱스 순으로 정렬
    items.sort_by_key(|(idx, _)| *idx);
    items.into_iter().map(|(_, text)| text).collect()
}

/// InsertString 인자 파싱: `"텍스트", 인덱스);` → (텍스트, 인덱스)
fn parse_insert_string_args(args: &str) -> Option<(String, usize)> {
    // "텍스트" 추출
    let rest = args.strip_prefix('"')?;
    let end_quote = rest.find('"')?;
    let text = rest[..end_quote].to_string();
    let after_quote = &rest[end_quote + 1..];

    // , 인덱스 추출
    let after_comma = after_quote.trim_start().strip_prefix(',')?;
    let idx_str: String = after_comma
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let idx = idx_str.parse().unwrap_or(0);

    Some((text, idx))
}
