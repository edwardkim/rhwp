//! Issue #1271: HWPX 글뒤로 paper-anchored RowBreak 표가 본문 흐름을 밀어
//! 앞쪽 페이지와 바탕쪽 홀짝 적용을 함께 어긋나게 하는 회귀 가드.
//!
//! 재현 문서: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`.
//! 한컴 PDF 기준 앞쪽 페이지 배치는 page 2 = MEMO, page 3 = 1주차 표지,
//! page 4 = 본문 시작이다. 현재 TypesetEngine 은 section 0 paragraph 3 의
//! 글뒤로 표를 `PartialTable` 로 분할해 page 2 를 차지시키므로, 이후 페이지가
//! 한 쪽씩 밀리고 Odd/Even 바탕쪽 적용도 반대로 보인다.

use std::fs;
use std::path::Path;

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let rel = "samples/hwpx/[2027] 온새미로 1 본교재.hwpx";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

#[test]
fn onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment() {
    let doc = load_doc();

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("\"MEMO\""),
        "PDF 기준 page 2 는 MEMO 쪽이어야 한다. 글뒤로 표 조각이 끼어 있으면 \
         이후 page_num 과 바탕쪽 홀짝이 밀린다.\n{page2}"
    );
    assert!(
        !page2.contains("PartialTable   pi=3 ci=0"),
        "section 0 paragraph 3 의 글뒤로 표는 본문 흐름을 차지하는 \
         PartialTable 로 분할되면 안 된다.\n{page2}"
    );

    let page3 = doc.dump_page_items(Some(2));
    assert!(
        page3.contains("Shape          pi=5 ci=0"),
        "PDF 기준 page 3 은 1주차 표지 도형들이 있는 쪽이어야 한다.\n{page3}"
    );
    assert!(
        !page3.contains("\"MEMO\""),
        "MEMO 쪽이 page 3 으로 밀리면 바탕쪽 홀짝 적용도 함께 어긋난다.\n{page3}"
    );

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("section=1, page_num=4"),
        "PDF 기준 page 4 에서 section 1 본문이 page_num=4 로 시작해야 한다.\n{page4}"
    );
    assert!(
        page4.contains("\"강의 01.\""),
        "PDF 기준 page 4 는 강의 01 본문 시작 쪽이어야 한다.\n{page4}"
    );
}
