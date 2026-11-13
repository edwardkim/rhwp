//! 검사 유형: 관계형 — 원본 중첩 구조의 깊이·소유·값과 검색·추출 결과를 대조한다.
//! 주석 갱신: 2026-11-13 05:01 KST.
//! Native/fresh WASM 일치율은 비해당이다. 이 파일은 렌더링 배치나 px 좌표를 검사하지 않는다.
//! 공개 CLI 재현 입력은 `tests/fixtures/issue8989/nested-values.hml`을 공유한다.
//! 기존 HML 상수와 바이트가 같은 입력이며, 기존 검색·추출 assertion은 변경하지 않는다.
//! 실제 CLI 증거: `mydocs/pr/assets/planet6897_clean20_20261113/final-d0cc4657/api-consumers/`.
//!
//! [#2792] 표 안의 표(중첩 표) 텍스트가 검색에 잡히지 않던 결함.
//!
//! `grep` 은 본문 문단의 컨트롤에서 `Control::Table` 로 들어가 셀 문단까지 훑었지만,
//! **셀 문단 안의 컨트롤**을 훑는 자리는 `Control::Equation` 만 봤다. 그래서 셀 안에
//! 들어 있는 표(중첩 표)는 어느 갈래에도 걸리지 않고 조용히 버려졌고, 그 안의 텍스트는
//! 순회 대상이 된 적이 없었다.
//!
//! 실패 모습이 나쁜 이유는 **오류가 아니기 때문**이다. `search` 는 exit 0 ·
//! `matchCount: 0` 으로 성공 응답하므로 호출자는 "문서에 그 말이 없다"로 읽는다.
//! 규제심사·검토보고 양식처럼 바깥 표 한 칸 안에 본문 표를 넣는 구조에서는 본문
//! 상당 부분이 통째로 검색에서 빠진다.
//!
//! 이 파일이 못 박는 계약.
//!
//! 1. 중첩 표 안 텍스트가 **찾힌다** — 2단계 아래까지.
//! 2. `nestedDepth` 가 실제 깊이를 알린다. 0(=중첩 아님)이면 **필드 자체가 없다** —
//!    중첩 없는 문서의 봉투는 종전과 바이트까지 같다.
//! 3. 중첩 매치의 `cell` 은 **그것을 담은 바깥 셀 문단**을 가리킨다(정확 위치가 아니다).
//!    깊이 1·2 매치는 같은 바깥 `cell.paragraph` 를 쓴다.
//! 4. 중첩 매치는 `page` 를 **싣지 않는다** — 바깥 행으로 계산한 쪽은 틀리기 때문이다.
//! 5. 본문·바깥 셀 매치의 동작은 종전 그대로다(순서 포함).
//! 6. `replace_all_native` 는 중첩 전용 토큰도 같은 검색 순회로 찾아 바꾼다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 3중 구조: 본문 → 바깥 표 셀 → 중첩 표 셀 → 더 깊은 표 셀.
/// 단어를 층마다 다르게 두어야 "어느 층까지 닿았는가"가 구별된다.
const NESTED_HML: &str = r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
  <P><TEXT><CHAR>본문전용단어</CHAR></TEXT></P>
  <P><TEXT><TABLE RowCount="1" ColCount="1">
    <SHAPEOBJECT><SIZE Width="8000" Height="4000"/></SHAPEOBJECT>
    <ROW><CELL ColAddr="0" RowAddr="0" Width="8000" Height="4000"><PARALIST>
      <P><TEXT><CHAR>바깥셀단어</CHAR></TEXT></P>
      <P><TEXT><TABLE RowCount="1" ColCount="1">
        <SHAPEOBJECT><SIZE Width="6000" Height="3000"/></SHAPEOBJECT>
        <ROW><CELL ColAddr="0" RowAddr="0" Width="6000" Height="3000"><PARALIST>
          <P><TEXT><CHAR>중첩셀단어</CHAR></TEXT></P>
          <P><TEXT><TABLE RowCount="1" ColCount="1">
            <SHAPEOBJECT><SIZE Width="4000" Height="2000"/></SHAPEOBJECT>
            <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="2000"><PARALIST>
              <P><TEXT><CHAR>더깊은셀단어</CHAR></TEXT></P>
            </PARALIST></CELL></ROW>
          </TABLE></TEXT></P>
        </PARALIST></CELL></ROW>
      </TABLE></TEXT></P>
    </PARALIST></CELL></ROW>
  </TABLE></TEXT></P>
</SECTION></BODY><TAIL/></HWPML>"#;

/// 중첩이 전혀 없는 대조군 — 봉투 무변경을 확인할 기준.
const FLAT_HML: &str = r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
  <P><TEXT><CHAR>본문전용단어</CHAR></TEXT></P>
  <P><TEXT><TABLE RowCount="1" ColCount="1">
    <SHAPEOBJECT><SIZE Width="4000" Height="1200"/></SHAPEOBJECT>
    <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST>
      <P><TEXT><CHAR>바깥셀단어</CHAR></TEXT></P>
    </PARALIST></CELL></ROW>
  </TABLE></TEXT></P>
</SECTION></BODY><TAIL/></HWPML>"#;

const TEXTBOX_TABLE_HML: &str = r#"<HWPML Version="2.91"><HEAD/><BODY><SECTION>
  <P><TEXT><RECTANGLE X0="0" X1="5000" X2="5000" X3="0" Y0="0" Y1="0" Y2="2400" Y3="2400">
    <SHAPEOBJECT><SIZE Width="5000" Height="2400"/></SHAPEOBJECT>
    <DRAWINGOBJECT><SHAPECOMPONENT XPos="0" YPos="0" OriWidth="5000" OriHeight="2400" CurWidth="5000" CurHeight="2400"/>
      <LINESHAPE Width="0" Style="Solid" EndCap="Flat" Alpha="0"/>
      <DRAWTEXT><TEXTMARGIN Left="0" Right="0" Top="0" Bottom="0"/><PARALIST>
        <P><TEXT><TABLE RowCount="1" ColCount="1">
          <SHAPEOBJECT><SIZE Width="4000" Height="1200"/></SHAPEOBJECT>
          <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST>
            <P><TEXT><CHAR>글상자표전용단어</CHAR></TEXT></P>
          </PARALIST></CELL></ROW>
        </TABLE></TEXT></P>
      </PARALIST></DRAWTEXT>
    </DRAWINGOBJECT>
  </RECTANGLE></TEXT></P>
</SECTION></BODY><TAIL/></HWPML>"#;

fn nested() -> DocumentCore {
    DocumentCore::from_bytes(NESTED_HML.as_bytes()).expect("중첩 표 픽스처가 열려야 한다")
}

fn flat() -> DocumentCore {
    DocumentCore::from_bytes(FLAT_HML.as_bytes()).expect("평면 표 픽스처가 열려야 한다")
}

fn textbox_table() -> DocumentCore {
    DocumentCore::from_bytes(TEXTBOX_TABLE_HML.as_bytes())
        .expect("글상자 안 표 픽스처가 열려야 한다")
}

#[test]
fn nested_table_text_is_found() {
    // 이 계약의 핵심. 종전에는 아래 두 단어가 어떤 질의로도 잡히지 않았다.
    let doc = nested();
    for word in ["본문전용단어", "바깥셀단어", "중첩셀단어", "더깊은셀단어"] {
        let hits = doc.grep(word, true, None);
        assert_eq!(
            hits.len(),
            1,
            "{word} 를 찾지 못했습니다 (중첩 표 순회 누락)"
        );
        assert!(hits[0].text.contains(word), "{:?}", hits[0]);
    }
}

#[test]
fn nested_depth_reports_actual_depth() {
    let doc = nested();
    // 본문 매치는 표 밖이라 깊이 0 이고 셀 좌표도 없다.
    let body = doc.grep("본문전용단어", true, None);
    assert_eq!(body[0].nested_depth, 0, "{:?}", body[0]);
    assert!(body[0].cell.is_none(), "{:?}", body[0]);

    // 바깥 표의 셀 = 중첩이 아니다(깊이 0).
    let outer = doc.grep("바깥셀단어", true, None);
    assert_eq!(outer[0].nested_depth, 0, "{:?}", outer[0]);
    assert!(outer[0].cell.is_some(), "{:?}", outer[0]);

    // 한 단계·두 단계 아래.
    assert_eq!(doc.grep("중첩셀단어", true, None)[0].nested_depth, 1);
    assert_eq!(doc.grep("더깊은셀단어", true, None)[0].nested_depth, 2);
}

#[test]
fn nested_match_addresses_the_containing_outer_cell() {
    // 중첩 안의 (cell, paragraph) 를 바깥 좌표계인 척 실으면 소비자가 엉뚱한 칸을
    // 고친다. 주소는 **담은 바깥 셀 문단**이어야 하고, 그 사실을 nestedDepth 가 알린다.
    let doc = nested();
    let deep = doc.grep("더깊은셀단어", true, None);
    let mid = doc.grep("중첩셀단어", true, None);
    let deep_cell = deep[0].cell.as_ref().expect("셀 좌표");
    let mid_cell = mid[0].cell.as_ref().expect("셀 좌표");

    // 두 중첩 매치는 같은 바깥 셀 문단이 담고 있다 — 바깥 표는 셀 하나뿐이고
    // 중첩 표는 그 셀의 두 번째 문단에 들어 있다.
    assert_eq!(deep_cell.control, mid_cell.control, "{deep:?} {mid:?}");
    assert_eq!(deep_cell.cell, mid_cell.cell, "{deep:?} {mid:?}");
    assert_eq!(
        mid_cell.paragraph, 1,
        "중첩 표를 담은 바깥 셀 문단을 가리켜야 합니다: {mid:?}"
    );
    assert_eq!(
        deep_cell.paragraph, mid_cell.paragraph,
        "깊이 2 매치도 같은 바깥 셀 문단이어야 합니다: {deep:?} {mid:?}"
    );
    assert_eq!(
        deep_cell.paragraph, 1,
        "깊이 2 매치의 cell.paragraph 는 바깥 호스트 1 이어야 합니다: {deep:?}"
    );
}

#[test]
fn nested_match_omits_page_rather_than_reporting_a_wrong_one() {
    // 바깥 셀의 row 로 쪽을 계산하면 1×1 바깥 표가 여러 쪽에 걸칠 때 행 범위의 첫 쪽이
    // 나온다 — 뒷쪽 텍스트가 앞쪽으로 보고된다. 없는 것보다 틀린 주소가 나쁘다.
    let doc = nested();
    for word in ["중첩셀단어", "더깊은셀단어"] {
        let hits = doc.grep(word, true, None);
        assert!(
            hits[0].page.is_none(),
            "중첩 매치에 쪽이 실렸습니다 (바깥 행 기준이라 틀릴 수 있음): {:?}",
            hits[0]
        );
    }
}

#[test]
fn flat_document_envelope_is_unchanged() {
    // 추가 전용이어야 한다 — 중첩이 없으면 nestedDepth 는 직렬화에서 아예 빠진다.
    let doc = flat();
    for word in ["본문전용단어", "바깥셀단어"] {
        let hits = doc.grep(word, true, None);
        assert_eq!(hits.len(), 1, "{word}");
        assert_eq!(hits[0].nested_depth, 0);
        let json = serde_json::to_string(&hits[0]).expect("직렬화");
        assert!(
            !json.contains("nestedDepth"),
            "중첩 없는 매치에 nestedDepth 가 실렸습니다: {json}"
        );
    }
}

#[test]
fn nested_match_serializes_depth_and_no_page() {
    let doc = nested();
    let hits = doc.grep("더깊은셀단어", true, None);
    let json = serde_json::to_string(&hits[0]).expect("직렬화");
    assert!(json.contains("\"nestedDepth\":2"), "{json}");
    assert!(
        !json.contains("\"page\""),
        "중첩 매치에 page 가 실렸습니다: {json}"
    );
    assert!(json.contains("\"cell\""), "{json}");
}

#[test]
fn body_and_outer_cell_matches_keep_their_order() {
    // 중첩분은 바깥 매치 **뒤에** 붙는다. 앞에 끼어들면 기존 소비자의 "첫 매치"가
    // 조용히 다른 것을 가리키게 된다.
    let doc = nested();
    // 세 층 모두에 있는 공통 문자열로 한 번에 훑는다.
    let hits = doc.grep("단어", true, None);
    let depths: Vec<usize> = hits.iter().map(|h| h.nested_depth).collect();
    assert!(
        depths.windows(2).all(|w| w[0] <= w[1]),
        "깊이가 오름차순이 아닙니다(중첩분이 앞에 끼어듦): {depths:?}"
    );
    assert_eq!(depths.first().copied(), Some(0), "{depths:?}");
    assert_eq!(depths.last().copied(), Some(2), "{depths:?}");
}

#[test]
fn limit_still_stops_early_with_nesting() {
    // 상한은 중첩 순회에서도 그대로 걸려야 한다 — 안 걸리면 대형 문서에서 상한이
    // 무력화된다.
    let doc = nested();
    let hits = doc.grep("단어", true, Some(2));
    assert_eq!(hits.len(), 2, "{hits:?}");
}

#[test]
fn grep_finds_table_text_inside_textbox() {
    let doc = textbox_table();
    let hits = doc.grep("글상자표전용단어", true, None);
    assert_eq!(
        hits.len(),
        1,
        "글상자 안 표 텍스트를 직접 검색이 놓치면 안 됩니다: {hits:?}"
    );
    assert!(
        hits[0].textbox.is_some(),
        "글상자 안 표 매치는 최소한 글상자 호스트를 알려야 합니다: {:?}",
        hits[0]
    );
}

#[test]
fn replace_all_native_replaces_nested_only_token() {
    // grep 과 치환 엔진은 중첩 표 전용 토큰을 같은 범위로 다룬다.
    let mut doc = nested();
    assert_eq!(doc.grep("더깊은셀단어", true, None).len(), 1);
    let result = doc
        .replace_all_native("더깊은셀단어", "치환됨", true)
        .expect("replace_all_native 는 실패하지 않아야 한다");
    let v: serde_json::Value =
        serde_json::from_str(&result).unwrap_or_else(|e| panic!("{e}: {result}"));
    assert_eq!(v["ok"], true, "{result}");
    assert_eq!(
        v["count"], 1,
        "중첩 전용 토큰도 한 번 치환되어야 합니다: {result}"
    );
    assert!(doc.grep("더깊은셀단어", true, None).is_empty());
    assert_eq!(doc.grep("치환됨", true, None).len(), 1);
}

// ---------------------------------------------------------------------------
// [#8989] 중첩 칸의 정확한 경로(`cellPath`) — search·extract-data 공통 계약.
//
// `cell` 은 종전대로 최외곽 칸 문단이고(호환), 정확한 칸은 `cellPath` 의 마지막
// 단계다. 꼴은 `fields` 의 `location.nested` 와 같다(바깥 → 안쪽).
// 중첩 표 하나에 칸이 여럿이어야 "같은 바깥 칸 안의 다른 안쪽 칸"이 구별되는지 본다.
// ---------------------------------------------------------------------------

/// 본문 → 바깥 1×2 표 칸 1 → (문단 1) 안쪽 1×3 표, 각 칸에 서로 다른 값.
const NESTED_VALUES_HML: &str = include_str!("../fixtures/issue8989/nested-values.hml");

fn nested_values() -> DocumentCore {
    DocumentCore::from_bytes(NESTED_VALUES_HML.as_bytes()).expect("중첩 값 픽스처가 열려야 한다")
}

fn steps(
    path: &[rhwp::document_core::queries::grep::CellPathStep],
) -> Vec<(&str, usize, Option<usize>, usize)> {
    path.iter()
        .map(|s| (s.kind, s.control, s.cell, s.paragraph))
        .collect()
}

#[test]
fn issue_8989_search_cell_path_names_the_inner_cell() {
    let doc = nested_values();
    let a = doc.grep("42,360명", true, None);
    let b = doc.grep("4,676건", true, None);
    assert_eq!((a.len(), b.len()), (1, 1), "{a:?} {b:?}");

    // 호환: cell 은 두 매치 모두 안쪽 표를 품은 바깥 칸 문단이다.
    let outer = |m: &rhwp::document_core::queries::grep::GrepMatch| {
        let c = m.cell.as_ref().expect("셀 좌표");
        (c.control, c.cell, c.paragraph)
    };
    assert_eq!(outer(&a[0]), (0, 1, 1));
    assert_eq!(outer(&b[0]), (0, 1, 1));

    // 정확한 칸: 바깥 칸 1 문단 1 → 안쪽 표(컨트롤 0) 칸 0 문단 1 / 칸 1 문단 0.
    assert_eq!(
        steps(&a[0].cell_path),
        vec![("tableCell", 0, Some(1), 1), ("tableCell", 0, Some(0), 1)]
    );
    assert_eq!(
        steps(&b[0].cell_path),
        vec![("tableCell", 0, Some(1), 1), ("tableCell", 0, Some(1), 0)]
    );
    let json = serde_json::to_string(&a[0]).expect("직렬화");
    assert!(json.contains("\"cellPath\":[{"), "{json}");
}

#[test]
fn issue_8989_one_level_and_flat_matches_have_no_cell_path() {
    // 한 단계 표 칸·본문 매치는 cell 이 곧 정확한 주소다 — 봉투가 종전과 같아야 한다.
    let doc = nested_values();
    let hit = doc.grep("목표", true, None);
    assert_eq!(hit.len(), 1);
    assert!(hit[0].cell_path.is_empty(), "{hit:?}");
    for word in ["본문전용단어", "바깥셀단어"] {
        let json = serde_json::to_string(&flat().grep(word, true, None)[0]).expect("직렬화");
        assert!(!json.contains("cellPath"), "{json}");
    }
}

#[test]
fn issue_8989_textbox_table_match_carries_textbox_step() {
    let doc = textbox_table();
    let hits = doc.grep("글상자표전용단어", true, None);
    assert_eq!(
        steps(&hits[0].cell_path),
        vec![("textBox", 0, None, 0), ("tableCell", 0, Some(0), 0)]
    );
}

#[test]
fn issue_8989_extract_data_finds_values_in_nested_cells() {
    let doc = nested_values();
    let items = doc.extract_data(&[]);
    let find = |raw: &str| {
        items
            .iter()
            .find(|i| i.raw == raw)
            .unwrap_or_else(|| panic!("{raw} 를 뽑지 못했습니다: {items:?}"))
    };

    // 한 단계 칸 값은 종전 그대로 — cellPath·nestedDepth 없음.
    let date = find("2026년 8월 2일");
    assert_eq!(date.nested_depth, 0);
    assert!(date.cell_path.is_empty(), "{date:?}");

    // 안쪽 칸 값은 칸마다 다른 경로로 나온다.
    let people = find("42,360명");
    let cases = find("4,676건");
    let money = find("1,234,000원");
    for item in [people, cases, money] {
        assert_eq!(item.nested_depth, 1, "{item:?}");
        let c = item.cell.as_ref().expect("셀 좌표");
        assert_eq!((c.control, c.cell, c.paragraph), (0, 1, 1), "{item:?}");
    }
    assert_eq!(
        steps(&people.cell_path),
        vec![("tableCell", 0, Some(1), 1), ("tableCell", 0, Some(0), 1)]
    );
    assert_eq!(steps(&cases.cell_path)[1], ("tableCell", 0, Some(1), 0));
    assert_eq!(steps(&money.cell_path)[1], ("tableCell", 0, Some(2), 0));
    let json = serde_json::to_string(people).expect("직렬화");
    assert!(
        json.contains("\"nestedDepth\":1") && json.contains("\"cellPath\""),
        "{json}"
    );
}

#[test]
fn issue_8989_extract_data_reads_tables_inside_textbox() {
    // search 와 같은 범위 — 글상자 안 표 칸의 값도 뽑는다.
    let hml = TEXTBOX_TABLE_HML.replace("글상자표전용단어", "12개");
    let doc = DocumentCore::from_bytes(hml.as_bytes()).expect("픽스처");
    let items = doc.extract_data(&[]);
    let item = items
        .iter()
        .find(|i| i.raw == "12개")
        .expect("글상자 안 표 값");
    assert!(item.textbox.is_some(), "{item:?}");
    assert_eq!(
        steps(&item.cell_path),
        vec![("textBox", 0, None, 0), ("tableCell", 0, Some(0), 0)]
    );
}
