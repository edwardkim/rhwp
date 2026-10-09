#![cfg(not(target_arch = "wasm32"))]
//! 스타일을 지우면 그 뒤 스타일 번호가 하나씩 당겨진다. 표 셀·글상자·머리말·꼬리말·각주·미주·
//! 캡션·바탕쪽 문단과 덧말도 본문과 같은 스타일 표를 가리키므로 같이 옮겨야 원래 스타일을
//! 계속 가리킨다. HWP·HWPX로 저장해 다시 열어도 같아야 한다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::path::Path;

fn sample(name: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn style_id(doc: &HwpDocument, name: &str) -> u32 {
    let styles = &doc.document().doc_info.styles;
    styles.iter().position(|s| s.local_name == name).unwrap() as u32
}

/// 모든 문단의 모델 경로와 스타일 이름. 모델을 JSON으로 펼쳐 `style_id`를 가진 문단을 찾으므로
/// 문단을 담는 컨테이너 종류를 따로 나열하지 않는다.
fn paragraph_styles(doc: &HwpDocument) -> Vec<(String, String)> {
    fn collect(value: &Value, path: &str, out: &mut Vec<(String, u64)>) {
        match value {
            Value::Object(map) => {
                if map.contains_key("char_shapes") {
                    if let Some(id) = map.get("style_id").and_then(Value::as_u64) {
                        out.push((path.to_string(), id));
                    }
                }
                for (key, child) in map {
                    collect(child, &format!("{path}.{key}"), out);
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    collect(child, &format!("{path}[{index}]"), out);
                }
            }
            _ => {}
        }
    }
    let mut ids = Vec::new();
    for (index, section) in doc.document().sections.iter().enumerate() {
        let paragraphs = serde_json::to_value(&section.paragraphs).unwrap();
        collect(&paragraphs, &format!("{index}"), &mut ids);
        let section_def = serde_json::to_value(&section.section_def).unwrap();
        collect(&section_def, &format!("{index}.section_def"), &mut ids);
    }
    let styles = &doc.document().doc_info.styles;
    ids.into_iter()
        .map(|(path, id)| {
            let name = styles
                .get(id as usize)
                .map_or_else(|| format!("#{id}"), |s| s.local_name.clone());
            (path, name)
        })
        .collect()
}

fn assert_styles(actual: &[(String, String)], expected: &[(String, String)], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: 문단 수");
    let changed: Vec<_> = actual
        .iter()
        .zip(expected)
        .filter(|(a, e)| a != e)
        .collect();
    assert!(
        changed.is_empty(),
        "{label}: 문단 {}개 중 {}개가 다른 스타일을 가리킨다. 첫 문단 (실제, 기대): {:?}",
        expected.len(),
        changed.len(),
        changed.first()
    );
}

#[test]
fn a_table_cell_paragraph_keeps_its_style_after_another_style_is_deleted() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "머리").unwrap();
    doc.split_paragraph(0, 0, 2, None).unwrap();
    doc.create_table(0, 1, 0, 1, 1).unwrap();
    let toc_title = style_id(&doc, "차례 제목");
    doc.apply_cell_style(0, 1, 0, 0, 0, toc_title).unwrap();
    assert!(doc.delete_style(style_id(&doc, "본문")));

    let cell_style = |doc: &HwpDocument| {
        let style: Value = serde_json::from_str(&doc.get_cell_style_at(0, 1, 0, 0, 0)).unwrap();
        style["name"].as_str().unwrap().to_string()
    };
    assert_eq!(cell_style(&doc), "차례 제목");
    for saved in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
        assert_eq!(cell_style(&HwpDocument::new(&saved).unwrap()), "차례 제목");
    }
}

/// 한컴 문서에서 1번 스타일을 지운다. 그 스타일을 쓰던 문단만 바탕글이 되고 나머지는 지우기 전
/// 스타일 이름을 그대로 가리킨다.
///
/// - `exam_social.hwp`: 표·중첩 표·캡션·글상자, 머리말 안 표, HWP 원본 바탕쪽
/// - `SO-SUEOP.hwpx`: 미주·머리말·꼬리말
/// - `143E433F503322BD33.hwp`: 각주
#[test]
fn every_paragraph_keeps_its_style_after_a_style_is_deleted() {
    for name in ["exam_social.hwp", "SO-SUEOP.hwpx", "143E433F503322BD33.hwp"] {
        let mut doc = sample(name);
        let styles = &doc.document().doc_info.styles;
        let (base, deleted) = (styles[0].local_name.clone(), styles[1].local_name.clone());
        let mut expected = paragraph_styles(&doc);
        for (_, style) in &mut expected {
            if *style == deleted {
                style.clone_from(&base);
            }
        }

        assert!(doc.delete_style(1));
        assert_styles(&paragraph_styles(&doc), &expected, name);
        let saved = if name.ends_with(".hwp") {
            doc.export_hwp().unwrap()
        } else {
            doc.export_hwpx().unwrap()
        };
        let reopened = HwpDocument::new(&saved).unwrap();
        assert_styles(
            &paragraph_styles(&reopened),
            &expected,
            &format!("{name} 저장본"),
        );
    }
}

/// 덧말의 `styleIDRef`도 스타일 표의 번호다. 덧말이 마지막 스타일을 가리키게 한 뒤 1번 스타일을
/// 지운다.
#[test]
fn a_ruby_keeps_its_style_after_a_style_is_deleted() {
    fn ruby_style(doc: &HwpDocument) -> String {
        let document = doc.document();
        let id = document
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Ruby(ruby) => Some(ruby.style_id_ref),
                _ => None,
            })
            .expect("샘플 전제: 덧말");
        document.doc_info.styles[id as usize].local_name.clone()
    }

    let mut doc = sample("hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx");
    let last = doc.document().doc_info.styles.len() - 1;
    let name = doc.document().doc_info.styles[last].local_name.clone();
    let document = doc.document_mut();
    for control in document
        .sections
        .iter_mut()
        .flat_map(|s| &mut s.paragraphs)
        .flat_map(|p| &mut p.controls)
    {
        if let Control::Ruby(ruby) = control {
            ruby.style_id_ref = last as u16;
        }
    }

    assert!(doc.delete_style(1));
    assert_eq!(ruby_style(&doc), name);
    for saved in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
        assert_eq!(ruby_style(&HwpDocument::new(&saved).unwrap()), name);
    }
}
