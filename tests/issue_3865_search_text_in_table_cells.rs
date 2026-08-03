//! [#3865] 표 셀 안 텍스트가 찾기에서 잡히지 않던 회귀.
//!
//! 제보: 웹 데모에서 1쪽 표 안의 문장이 어떤 단어로도 검색되지 않는다.
//!
//! 파싱은 정상이었다 — `search_all` 도 CLI `grep` 도 표 셀을 순회한다. 문제는 찾기
//! 대화상자가 쓰는 `search_text_native` 가 셀·글상자 매치를 **옵션 없이** 걸러낸 것이었다.
//! 당시 사유는 "커서 이동 불가"였지만, 그 뒤 편집기가 셀 좌표를 다루게 되어
//! (`getCursorRectInCell`, `DocumentPosition.cellIndex` 계열) 더는 성립하지 않는다.
//!
//! 계약: `include_cells` 는 옵트인이다.
//! - 끄면 종전대로 본문만 — 셀 이동을 못 하는 호출자가 무회귀로 남는다.
//! - 켜면 셀 매치를 돌려주고, 셀 안 위치를 `cellContext` 로 함께 싣는다. 이게 없으면
//!   호출자가 "찾았다"는 결과를 받고도 어디로 가야 할지 모른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 셀 안에만 있는 단어(`셀전용단어`)와 본문에만 있는 단어(`본문전용단어`)를 함께 둔다.
/// 두 단어를 나눠 두어야 "셀을 켜서 찾은 것"과 "원래 본문에서 찾히던 것"이 구별된다.
const TABLE_HML: &str = r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
  <P><TEXT><CHAR>본문전용단어</CHAR></TEXT></P>
  <P><TEXT><TABLE RowCount="1" ColCount="1">
    <SHAPEOBJECT><SIZE Width="4000" Height="1200"/></SHAPEOBJECT>
    <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST>
      <P><TEXT><CHAR>셀전용단어</CHAR></TEXT></P>
    </PARALIST></CELL></ROW>
  </TABLE></TEXT></P>
</SECTION></BODY><TAIL/></HWPML>"#;

fn core() -> DocumentCore {
    DocumentCore::from_bytes(TABLE_HML.as_bytes()).expect("표 픽스처가 열려야 한다")
}

/// 끈 상태의 동작을 먼저 고정한다 — 이게 무너지면 아래 판정이 공허하다.
#[test]
fn body_only_search_still_ignores_table_cells() {
    let core = core();

    let body = core
        .search_text_native("본문전용단어", 0, 0, 0, true, true, false)
        .expect("검색 실패");
    assert!(
        body.contains("\"found\":true"),
        "본문 단어는 종전대로 찾혀야 한다: {body}"
    );

    let cell = core
        .search_text_native("셀전용단어", 0, 0, 0, true, true, false)
        .expect("검색 실패");
    assert!(
        cell.contains("\"found\":false"),
        "include_cells 를 끄면 셀 매치는 나오지 않아야 한다(종전 동작 유지): {cell}"
    );
}

/// #3865 본체 — 켜면 셀 안 단어가 잡히고, 갈 곳(cellContext)이 함께 온다.
#[test]
fn opting_in_finds_text_inside_table_cells_with_navigable_context() {
    let core = core();

    let found = core
        .search_text_native("셀전용단어", 0, 0, 0, true, true, true)
        .expect("검색 실패");

    assert!(
        found.contains("\"found\":true"),
        "표 셀 안 텍스트가 검색되지 않는다 — #3865 의 증상 그대로다: {found}"
    );
    assert!(
        found.contains("\"cellContext\""),
        "셀 매치인데 cellContext 가 없으면 호출자가 커서를 옮길 수 없다: {found}"
    );
    for key in ["parentPara", "ctrlIdx", "cellIdx", "cellPara"] {
        assert!(
            found.contains(key),
            "cellContext 에 {key} 가 없다 — 셀 좌표가 불완전하다: {found}"
        );
    }
}

/// 켜도 본문 매치는 그대로여야 한다 — 옵션이 본문 검색을 바꾸면 안 된다.
#[test]
fn opting_in_does_not_change_body_matches() {
    let core = core();

    let off = core
        .search_text_native("본문전용단어", 0, 0, 0, true, true, false)
        .expect("검색 실패");
    let on = core
        .search_text_native("본문전용단어", 0, 0, 0, true, true, true)
        .expect("검색 실패");

    assert_eq!(
        off, on,
        "본문에만 있는 단어의 결과가 include_cells 에 따라 달라졌다"
    );
    assert!(
        !off.contains("\"cellContext\""),
        "본문 매치에 셀 좌표가 붙었다: {off}"
    );
}
