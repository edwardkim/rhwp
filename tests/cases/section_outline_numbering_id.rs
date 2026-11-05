//! 검사 유형: 관계형·고정형(번호 정의와 표시 문자열); 페이지 절대 px 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T14:31:40+09:00; 직전 파일 변경: 49dfa0ea644c3eabc724be11506ae4a9d94d1c27 2026-11-05T11:53:56+09:00.
//! 측정일 2026-11-05, 제품 source 6337fe4d10; 96 DPI·Print·Native Skia/fresh WASM Canvas·2px raw 실루엣.
//! outline-id2-hwp: 전1쪽 Native 최저 96.49% / fresh WASM 최저 97.10%; 누락 없음.
//! outline-id2-hwpx: 전1쪽 Native 최저 96.49% / fresh WASM 최저 97.10%; 누락 없음.
//! 증거: mydocs/pr/assets/semanticist21-20261105-next10/validation/pre8622/candidate/{native,wasm}/<위 key>.tsv, validation.md 및 같은 영역의 독립 Print 확대 판독.
//! 순수 API/합성 계약 검사와 실물 문서의 시각 측정은 구분한다. 계산한 원본 크기·관계는 유지한다.
//!
//! 구역 정의의 개요 번호 모양(번호 정의 ID)을 바꾸고 읽으며, HWP·HWPX 저장 뒤에도 남긴다.
//!
//! 한/글의 서식 > 개요 번호 모양은 구역의 `outlineShapeIDRef`(HWP5 구역 정의의 개요 번호
//! ID)가 가리키는 번호 정의로 개요 문단 머리를 그린다. `setSectionDef`가 이 값을 받지
//! 않으면 저장본은 늘 처음 정의(1)를 가리키고 `getSectionDef`도 값을 알리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn page_text(core: &DocumentCore) -> String {
    fn collect(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut out = String::new();
    collect(&tree.root, &mut out);
    out
}

fn outline_id(core: &DocumentCore) -> u64 {
    let json: Value =
        serde_json::from_str(&core.get_section_def_native(0).expect("section def")).unwrap();
    json["outlineNumberingId"]
        .as_u64()
        .unwrap_or_else(|| panic!("outlineNumberingId 없음: {json}"))
}

/// 개요 1 문단 하나와 두 번째 번호 정의(가나다 + 괄호)를 둔 새 문서.
fn fixture() -> DocumentCore {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.ensure_default_numbering();
    assert_eq!(
        doc.create_numbering(r#"{"levelFormats":["^1)"],"numberFormats":[8],"startNumber":1}"#),
        2
    );
    let mut core = DocumentCore::new_empty();
    core.set_document(doc.document().clone());
    core.insert_text_native(0, 0, 0, "제목").unwrap();
    let outline = core
        .document()
        .doc_info
        .styles
        .iter()
        .position(|style| style.local_name == "개요 1")
        .expect("개요 1 스타일");
    core.apply_style_native(0, 0, outline).unwrap();
    core
}

#[test]
fn outline_numbering_id_is_settable_readable_and_saved() {
    let mut core = fixture();
    assert_eq!(outline_id(&core), 1);
    assert!(page_text(&core).contains("1.제목"), "{}", page_text(&core));

    core.set_section_def_native(0, r#"{"outlineNumberingId":2}"#)
        .expect("set outline numbering");
    assert_eq!(outline_id(&core), 2);
    assert!(page_text(&core).contains("가)제목"), "{}", page_text(&core));

    let hwpx = core.export_hwpx_native().expect("hwpx");
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen hwpx");
    assert_eq!(outline_id(&reopened), 2);
    assert!(page_text(&reopened).contains("가)제목"));

    let hwp = core.export_hwp_native().expect("hwp");
    let reopened = DocumentCore::from_bytes(&hwp).expect("reopen hwp");
    assert_eq!(outline_id(&reopened), 2);
    assert!(page_text(&reopened).contains("가)제목"));
}

#[test]
fn missing_outline_numbering_is_rejected_without_change() {
    let mut core = fixture();
    let before = core.get_section_def_native(0).unwrap();
    for id in ["3", "65536", "65537", "-1", "1.5", "\"2\"", "{}"] {
        let props = format!(r#"{{"outlineNumberingId":{id},"pageNum":5}}"#);
        assert!(core.set_section_def_all_native(&props).is_err(), "{props}");
        assert_eq!(core.get_section_def_native(0).unwrap(), before, "{props}");
    }
}
