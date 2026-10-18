//! [#7993] 소프트 하이픈 제어 표기 — 한/글의 HWPX 요소 이름은 `<hp:hypen/>` 이다.
//!
//! 한/글은 소프트 하이픈 제어 문자를 `<hp:hypen/>` 으로 적고 `<hp:hyphen/>` 은 모르는 요소로
//! 버린다. 수정 전 rhwp 는 `hyphen` 으로 쓰고 `hyphen` 만 읽어, 한/글 HWPX 의 하이픈을 버려
//! 저장 줄 위치와 글자 수가 어긋났다. 그리기도 제어 표기 하이픈을 줄 중간에서 보이는 글자로
//! 그렸다 — 한/글은 줄 끝에서만 `-` 로 그리고 줄 중간에서는 폭 없이 감춘다. 리터럴 U+00AD 는
//! 한/글도 보통 글자로 그린다.
//!
//! 입력 `samples/issue7993/soft-hyphen.hwpx` 는 생성기 `gen_soft_hyphen.py` 의 출력이고,
//! `soft-hyphen-2024.hwp` 는 한/글 2024(13.0.0.564)가 이를 HWP 로 저장한 것(저장 줄 = 한/글 줄
//! 나눔), 정본 `pdf/issue7993/soft-hyphen-2024.pdf` 는 같은 한/글의 Print 출력이다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const HWPX: &str = "samples/issue7993/soft-hyphen.hwpx";
const HWP: &str = "samples/issue7993/soft-hyphen-2024.hwp";
const SOFT_HYPHEN_CONTROL_BIT: u32 = 1 << 0x18;

fn read(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read")
}

fn section_xml(bytes: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .expect("section0")
        .read_to_string(&mut xml)
        .expect("utf8");
    xml
}

#[test]
fn hwpx_hypen_element_is_read_and_written_back_with_hancom_spelling() {
    let source = read(HWPX);
    let element_count = section_xml(&source).matches("<hp:hypen/>").count();
    assert!(
        element_count >= 30,
        "표본의 <hp:hypen/> 수: {element_count}"
    );

    let core = DocumentCore::from_bytes(&source).expect("parse");
    let paragraphs = &core.document().sections[0].paragraphs;
    let read_count: usize = paragraphs
        .iter()
        .filter(|para| para.control_mask & SOFT_HYPHEN_CONTROL_BIT != 0)
        .map(|para| para.text.matches('\u{00AD}').count())
        .sum();
    assert_eq!(
        read_count, element_count,
        "<hp:hypen/> 마다 제어 표기 소프트 하이픈 한 글자"
    );
    let literal = paragraphs.last().expect("대조군 문단");
    assert!(
        literal.text.contains('\u{00AD}') && literal.control_mask & SOFT_HYPHEN_CONTROL_BIT == 0,
        "리터럴 U+00AD 문단은 제어 표기로 바뀌지 않는다"
    );

    let saved = section_xml(&core.export_hwpx_native().expect("export"));
    assert_eq!(
        saved.matches("<hp:hypen/>").count(),
        element_count,
        "한/글 표기 <hp:hypen/> 으로 같은 수를 되쓴다"
    );
    assert!(
        !saved.contains("<hp:hyphen/>"),
        "한/글이 버리는 <hp:hyphen/> 을 쓰지 않는다"
    );
    assert!(
        saved.contains("K\u{00AD}MOOC"),
        "리터럴 U+00AD 는 리터럴로 남는다"
    );
}

#[test]
fn soft_hyphen_control_is_drawn_only_at_line_end() {
    let doc = HwpDocument::from_bytes(&read(HWP)).expect("parse");
    assert_eq!(doc.page_count(), 1, "정본 PDF 와 같은 1쪽");
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();

    // 본문 글줄마다 (모델 문자열, 그리는 문자열).
    let mut lines: Vec<(String, String)> = Vec::new();
    fn walk(node: &Value, out: &mut Vec<(String, String)>) {
        if node["type"] == "TextLine" {
            let runs: Vec<&Value> = node["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|child| child["type"] == "TextRun")
                .collect();
            let model: String = runs.iter().filter_map(|r| r["text"].as_str()).collect();
            let drawn: String = runs
                .iter()
                .filter_map(|r| r["displayText"].as_str().or(r["text"].as_str()))
                .collect();
            out.push((model, drawn));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    walk(&tree, &mut lines);

    let mut line_end_breaks = 0;
    for (model, drawn) in &lines {
        if model.contains("대조군") {
            assert!(
                drawn.contains('\u{00AD}'),
                "리터럴 U+00AD 는 보통 글자로 그린다: {drawn:?}"
            );
            continue;
        }
        assert!(
            !drawn.contains('\u{00AD}'),
            "제어 표기 소프트 하이픈을 글자로 그리지 않는다: {drawn:?}"
        );
        let model_trimmed = model.trim_end();
        let breaks_at_hyphen = model_trimmed.ends_with('\u{00AD}');
        let mid_line = model_trimmed.matches('\u{00AD}').count() - usize::from(breaks_at_hyphen);
        if breaks_at_hyphen {
            line_end_breaks += 1;
            assert!(
                drawn.trim_end().ends_with('-'),
                "소프트 하이픈에서 끊긴 줄은 끝에 `-` 를 그린다: {drawn:?}"
            );
        }
        assert_eq!(
            drawn.trim_end().matches('-').count(),
            model_trimmed.matches('-').count() + usize::from(breaks_at_hyphen),
            "줄 중간 소프트 하이픈 {mid_line}개는 보이지 않는다: {drawn:?}"
        );
    }
    assert!(
        (5..=24).contains(&line_end_breaks),
        "한/글이 소프트 하이픈에서 끊은 줄 수가 범위를 벗어났다: {line_end_breaks}"
    );
}
