//! 관계형 회귀: 스타일 중간 삽입 전후 참조·이름·필드·본문·실패 시 보존. API·직렬화 계약이며 화면 배치 좌표를 고정하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! inserted-style.hwp (api-8526-inserted-style-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! inserted-style.hwpx (api-8526-inserted-style-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형·고정형: 검사 대상 — 목록 가운데에 끼운 스타일과 기존 스타일 참조.
//! 입력·기준: 이 파일의 합성 입력과 `samples/SO-SUEOP.hwpx`, `samples/143E433F503322BD33.hwp`,
//! `samples/issue5866/memo_field_hwp5.hwp`, `samples/hwpx/opengov/`의 덧말 문서.
//! 모델·API·직렬화 계약을 검사하며 화면 시각 일치율은 측정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
//! 한/글 12.30은 스타일 추가하기의 새 스타일을 목록에서 고른 스타일 바로 뒤에 넣고, 뒤 스타일은
//! 단축키(Ctrl+1~0)와 함께 한 칸씩 밀린다. 문단·덧말·다른 스타일의 다음 스타일이 스타일 표의 번호를
//! 가리키므로, 끼운 뒤에도 모두 같은 이름의 스타일을 가리켜야 한다. HWP·HWPX 저장본도 같다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};
use std::path::Path;

fn sample(name: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 모든 문단(구역 정의의 바탕쪽 포함)의 모델 경로와 가리키는 스타일 이름.
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
        .map(|(path, id)| (path, styles[id as usize].local_name.clone()))
        .collect()
}

/// 스타일마다 (이름, 다음 스타일 이름).
fn next_links(doc: &HwpDocument) -> Vec<(String, String)> {
    let styles = &doc.document().doc_info.styles;
    styles
        .iter()
        .map(|s| {
            (
                s.local_name.clone(),
                styles[s.next_style_id as usize].local_name.clone(),
            )
        })
        .collect()
}

fn reopen(doc: &HwpDocument, hwpx: bool) -> HwpDocument {
    let saved = if hwpx {
        doc.export_hwpx().unwrap()
    } else {
        doc.export_hwp().unwrap()
    };
    HwpDocument::new(&saved).unwrap()
}

/// 한컴 문서의 바탕글 뒤(1번)에 스타일을 끼운다. 미주·머리말·꼬리말(`SO-SUEOP.hwpx`)과
/// 각주(`143E433F503322BD33.hwp`) 문단과 다른 스타일의 다음 스타일이 같은 이름을 가리킨다.
#[test]
fn every_paragraph_and_next_link_keeps_its_style_after_a_style_is_inserted() {
    for name in ["SO-SUEOP.hwpx", "143E433F503322BD33.hwp"] {
        let mut doc = sample(name);
        let paragraphs = paragraph_styles(&doc);
        assert!(!paragraphs.is_empty(), "{name}");
        let mut links = next_links(&doc);
        let created = doc.create_style(
            &json!({"name":"끼운 스타일","type":0,"nextStyleId":1,"index":1}).to_string(),
        );
        assert_eq!(created, 1, "{name}");
        links.insert(1, ("끼운 스타일".into(), "끼운 스타일".into()));
        let check = |doc: &HwpDocument, label: &str| {
            assert_eq!(doc.document().doc_info.styles[1].local_name, "끼운 스타일");
            assert_eq!(paragraph_styles(doc), paragraphs, "{label}");
            assert_eq!(next_links(doc), links, "{label}");
        };
        check(&doc, name);
        let saved = reopen(&doc, name.ends_with(".hwpx"));
        check(&saved, &format!("{name} 저장본"));
    }
}

/// 덧말의 `styleIDRef`도 스타일 표의 번호다.
#[test]
fn a_ruby_keeps_its_style_after_a_style_is_inserted() {
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
    let name = ruby_style(&doc);
    assert_eq!(doc.create_style(r#"{"name":"끼운 스타일","index":1}"#), 1);
    assert_eq!(ruby_style(&doc), name);
    for hwpx in [false, true] {
        assert_eq!(ruby_style(&reopen(&doc, hwpx)), name);
    }
}

/// HWP5 메모는 필드의 문단 모델과 구역 끝 원본 목록을 함께 보존한다. 끼울 때 원본 문단 머리
/// (HWP5 표 58의 스타일 BYTE, 오프셋 10)도 함께 고쳐야 저장본에서도 같은 스타일을 가리킨다.
#[test]
fn hwp_memo_list_paragraphs_keep_their_style_after_a_style_is_inserted() {
    fn tail_styles(doc: &HwpDocument) -> Vec<String> {
        let styles = &doc.document().doc_info.styles;
        doc.document()
            .sections
            .iter()
            .filter_map(|section| section.trailing_lists.as_ref())
            .flat_map(|tail| &tail.records)
            .filter(|record| record.tag_id == rhwp::parser::tags::HWPTAG_PARA_HEADER)
            .map(|record| styles[usize::from(record.data[10])].local_name.clone())
            .collect()
    }
    let mut doc = sample("issue5866/memo_field_hwp5.hwp");
    let before = tail_styles(&doc);
    assert!(before.iter().any(|name| name != "바탕글"), "{before:?}");
    assert_eq!(doc.create_style(r#"{"name":"끼운 스타일","index":1}"#), 1);
    assert_eq!(tail_styles(&doc), before);
    assert_eq!(tail_styles(&reopen(&doc, false)), before);
}

/// 목록 끝을 넘는 자리는 받지 않는다. 문단은 스타일 번호를 1바이트로 저장하므로 256개를 채운 목록
/// 가운데에 끼우면 마지막 스타일이 가리킬 수 없는 256번으로 밀린다. 끝에 붙이기는 예전처럼 받는다.
#[test]
fn style_insert_rejects_past_the_end_and_the_middle_of_a_full_style_list() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    let count = doc.document().doc_info.styles.len();
    let list = doc.get_style_list();
    for index in [count as i64 + 1, -1] {
        assert_eq!(
            doc.create_style(&json!({"name":"밖","index":index}).to_string()),
            -1
        );
        assert_eq!(doc.get_style_list(), list);
    }
    assert_eq!(
        doc.create_style(&json!({"name":"끝","index":count}).to_string()),
        count as i32
    );
    while doc.document().doc_info.styles.len() < 256 {
        let n = doc.document().doc_info.styles.len();
        assert_eq!(
            doc.create_style(&json!({"name":format!("채움{n}")}).to_string()),
            n as i32
        );
    }
    let list = doc.get_style_list();
    assert_eq!(doc.create_style(r#"{"name":"넘침","index":1}"#), -1);
    assert_eq!(doc.get_style_list(), list);
    assert_eq!(doc.create_style(r#"{"name":"끝 넘침"}"#), 256);
}

/// 복사해 둔 문단도 스타일 표의 번호를 가리킨다. 끼운 뒤에 붙여도 복사한 스타일 그대로다.
#[test]
fn copied_paragraphs_keep_their_style_when_a_style_is_inserted_before_pasting() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "가").unwrap();
    doc.split_paragraph(0, 0, 1, None).unwrap();
    doc.insert_text(0, 1, 0, "나").unwrap();
    doc.split_paragraph(0, 1, 1, None).unwrap();
    let body = doc
        .document()
        .doc_info
        .styles
        .iter()
        .position(|s| s.local_name == "본문")
        .unwrap();
    for para in 0..2 {
        doc.apply_style_native(0, para, body).unwrap();
    }
    doc.copy_selection_native(0, 0, 0, 1, 1).unwrap();
    assert_eq!(doc.create_style(r#"{"name":"끼운 스타일","index":1}"#), 1);
    doc.paste_internal_native(0, 2, 0).unwrap();
    let styles = &doc.document().doc_info.styles;
    let names: Vec<_> = doc.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| styles[usize::from(p.style_id)].local_name.as_str())
        .collect();
    assert!(names.len() > 3, "{names:?}");
    assert!(!names.contains(&"끼운 스타일"), "{names:?}");
}
