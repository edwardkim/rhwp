//! 표 호스트 문단(본문 문단의 내용이 표 컨트롤)에 앵커된 각주가 페이지 하단
//! 각주 영역에 누락되는 결함 회귀 가드.
//!
//! 재현 문서: `samples/footnote-table-host-01.hwp`
//!   - para 1 = [Control::Table, Control::Footnote("안녕하세요")]
//!   - 표 셀 본문: "② And it is also important ... 중요합니다."
//!
//! 결함 본질: typeset.rs (main paginator) 의 공통 컨트롤 루프가
//! `Control::Footnote` 수집을 `if !has_table` 로 가드 — 표 문단의 셀 내부
//! 각주는 `typeset_table_paragraph` 가 수집하지만(FootnoteSource::TableCell),
//! 같은 문단 BODY 레벨의 각주 컨트롤은 어느 경로에서도 수집되지 않아 파일에는
//! 존재해도 화면에는 영원히 보이지 않음. (legacy engine.rs 에는 가드가 없어
//! 정상 수집 — feedback_image_renderer_paths_separate 의 두 경로 동기화 누락
//! 사례, issue #1052 와 동류.)

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn page_svg(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    doc.render_page_svg_native(page).expect("render_page_svg")
}

fn svg_text_sequence(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after_open = &rest[open..];
        if let Some(gt) = after_open.find('>') {
            let after_tag = &after_open[gt + 1..];
            if let Some(close) = after_tag.find("</text>") {
                out.push_str(&after_tag[..close]);
                rest = &after_tag[close + "</text>".len()..];
                continue;
            }
        }
        break;
    }
    out
}

/// 표 호스트 문단에 앵커된 각주 본문이 페이지 하단 각주 영역에 렌더되어야 한다.
#[test]
fn table_host_para_footnote_appears_in_footer_area() {
    let doc = load_doc("samples/footnote-table-host-01.hwp");
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("안녕하세요"),
        "표 호스트 문단의 body 각주 '안녕하세요' 가 페이지 하단 각주 영역에 표시되어야 함. \
         text sequence={:?}",
        seq
    );
}

/// 페이지 각주 메타(getPageFootnoteInfo)에도 잡혀야 한다 (수집 자체의 가드).
#[test]
fn table_host_para_footnote_collected_in_page_info() {
    let doc = load_doc("samples/footnote-table-host-01.hwp");
    let info = doc
        .get_page_footnote_info_native(0, 0)
        .expect("page 0 footnote 0 must exist");
    assert!(
        info.contains("\"sourceType\":\"body\""),
        "body-anchored footnote ref expected, got {info}"
    );
}

/// 각주 내부 텍스트가 searchAllText 에 잡혀야 한다 (noteContext 동반).
/// 에이전트가 insert_footnote 직후 doc.find 로 검증하는 경로 — 이전에는
/// search walk 가 노트 내부 문단을 걷지 않아 영원히 "[]" 였다.
#[test]
fn footnote_text_is_searchable_with_note_context() {
    let doc = load_doc("samples/footnote-table-host-01.hwp");
    let hits = doc
        .search_all_text("안녕하세요", false, true)
        .expect("searchAllText");
    assert!(
        hits.contains("\"noteContext\""),
        "footnote-interior match must carry noteContext, got {hits}"
    );
    assert!(
        hits.contains("\"kind\":\"footnote\""),
        "noteContext.kind must be footnote, got {hits}"
    );
}

/// 노트 내부 매치는 본문 전용 검색(include_cells=false)에서는 제외된다.
#[test]
fn footnote_text_excluded_from_body_only_search() {
    let doc = load_doc("samples/footnote-table-host-01.hwp");
    let hits = doc
        .search_all_text("안녕하세요", false, false)
        .expect("searchAllText");
    assert_eq!(hits, "[]", "body-only search must skip note interiors");
}

/// 표 셀 문단 앵커 각주: 삽입(+본문 텍스트 원샷) → 렌더/검색/왕복 검증.
/// 에이전트의 "셀 안 문장 끝에 각주" 시나리오 — 이전에는 wasm API 에 셀 변형이
/// 없어 표 호스트 본문 문단으로 re-anchor 됐다.
#[test]
fn footnote_in_cell_insert_render_search_roundtrip() {
    let mut doc = load_doc("samples/footnote-table-host-01.hwp");

    // 표: para 1 control 0, cell 0, cell-para 1 = "그리고 ... 중요합니다." 끝에 앵커
    let res = doc
        .insert_footnote_in_cell(0, 1, 0, 0, 1, 999, "셀각주본문")
        .expect("insert_footnote_in_cell");
    assert!(res.contains("\"ok\":true"), "insert reply: {res}");
    assert!(res.contains("\"noteControlIdx\""), "insert reply: {res}");

    // 1) 페이지 하단 각주 영역에 렌더 (typeset_table_paragraph 셀 각주 수집 경로)
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("셀각주본문"),
        "셀 앵커 각주 본문이 페이지 하단에 렌더되어야 함. seq={:?}",
        seq
    );

    // 2) 검색: cellContext + noteContext 동반
    let hits = doc
        .search_all_text("셀각주본문", false, true)
        .expect("searchAllText");
    assert!(hits.contains("\"cellContext\""), "hits={hits}");
    assert!(hits.contains("\"noteContext\""), "hits={hits}");

    // 3) HWP 직렬화 왕복 후에도 유지
    let bytes = doc.export_hwp_native().expect("export hwp");
    let doc2 = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("re-parse");
    let hits2 = doc2
        .search_all_text("셀각주본문", false, true)
        .expect("searchAllText after roundtrip");
    assert!(
        hits2.contains("\"noteContext\""),
        "각주가 HWP 왕복에서 유실되면 안 됨. hits={hits2}"
    );
}

/// 본문 문단 각주 회귀 부재: 표가 없는 문단의 각주 수집은 그대로 동작.
#[test]
fn plain_para_footnote_no_regression() {
    let doc = load_doc("samples/footnote-tbox-01.hwp");
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("일반문단내각주") || seq.contains("일반 문단내 각주"),
        "기존 본문 각주 렌더가 유지되어야 함. text sequence={:?}",
        seq
    );
}
