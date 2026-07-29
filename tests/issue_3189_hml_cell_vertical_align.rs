//! Issue #3189 회귀 가드 — HML 표 셀 세로 정렬(`PARALIST@VertAlign`) 왕복.
//!
//! 종전엔 네 겹으로 값이 사라졌다.
//! 1. reader 의 `start_element` 디스패치에 `PARALIST` 처리가 아예 없어 속성을 읽지 않음
//! 2. adapter 의 `into_table()` 이 모든 셀을 `VerticalAlign::Center` 로 하드코딩
//! 3. serializer 의 `write_cell()` 이 `PARALIST` 를 속성 없이 방출
//! 4. preflight 의 `validate_cell()` 이 비-Center 셀을 blocker 로 막아 저장 자체를 차단
//!
//! 그래서 `parse → IR → serialize → re-parse` 전 구간을 한 테스트에서 확인한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::VerticalAlign;
use rhwp::parser::hml::parse_hml;

const FIXTURE: &str = include_str!("../samples/hml/formatting_table.hml");

/// fixture(1행 1열 표)의 CELL 원문. 세 번 복제해 1행 3열로 늘린다.
const FIXTURE_CELL: &str = r#"<CELL BorderFill="3" ColAddr="0" ColSpan="1" Dirty="false" Editable="false" HasMargin="false" Header="false" Height="282" Protect="false" RowAddr="0" RowSpan="1" Width="41956"><CELLMARGIN Bottom="141" Left="510" Right="510" Top="141"/><PARALIST LineWrap="Break" LinkListID="0" LinkListIDNext="0" TextDirection="0" VertAlign="Center"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>table</CHAR></TEXT></P></PARALIST></CELL>"#;

const EXPECTED_ALIGNS: [VerticalAlign; 3] = [
    VerticalAlign::Top,
    VerticalAlign::Center,
    VerticalAlign::Bottom,
];

/// 실물 fixture 를 Top/Center/Bottom 세 셀짜리 1행 3열 표로 바꾼다.
/// 저장 경로(preflight)를 통과하는 것이 확인된 문서를 최소 수술만 해서 쓴다.
fn fixture_with_three_aligned_cells() -> Vec<u8> {
    assert!(
        FIXTURE.contains(FIXTURE_CELL),
        "fixture 의 CELL 원문을 찾지 못함 — 샘플이 바뀌었다면 상수를 갱신할 것"
    );
    let mut cells = String::new();
    for (col, align) in ["Top", "Center", "Bottom"].iter().enumerate() {
        cells.push_str(
            &FIXTURE_CELL
                .replacen(r#"ColAddr="0""#, &format!(r#"ColAddr="{col}""#), 1)
                .replacen(
                    r#"VertAlign="Center""#,
                    &format!(r#"VertAlign="{align}""#),
                    1,
                ),
        );
    }
    FIXTURE
        .replacen(r#"ColCount="1""#, r#"ColCount="3""#, 1)
        .replacen(FIXTURE_CELL, &cells, 1)
        .into_bytes()
}

fn cell_aligns(document: &rhwp::model::document::Document) -> Vec<VerticalAlign> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤이 있어야 함")
        .cells
        .iter()
        .map(|cell| cell.vertical_align)
        .collect()
}

#[test]
fn cell_vertical_align_survives_hml_save_and_reload() {
    let bytes = fixture_with_three_aligned_cells();

    let core = DocumentCore::from_bytes(&bytes).expect("주입 HML 은 파싱되어야 함");
    assert_eq!(
        cell_aligns(core.document()),
        EXPECTED_ALIGNS,
        "PARALIST@VertAlign 이 IR 로 올라오지 않음"
    );

    // 종전엔 여기서 "cell fields cannot round-trip through HML" blocker 로 막혔다.
    let exported = core
        .export_hml_native()
        .expect("Top/Bottom 정렬 셀도 저장할 수 있어야 함");
    let xml = std::str::from_utf8(&exported).expect("HML 출력은 UTF-8");
    for align in ["Top", "Center", "Bottom"] {
        assert!(
            xml.contains(&format!(r#"<PARALIST VertAlign="{align}">"#)),
            "PARALIST VertAlign={align} 이 방출되지 않음"
        );
    }

    let reparsed = DocumentCore::from_bytes(&exported).expect("저장한 HML 은 다시 열려야 함");
    assert_eq!(
        cell_aligns(reparsed.document()),
        EXPECTED_ALIGNS,
        "저장 후 되읽기에서 셀 세로 정렬이 유실됨"
    );
}

#[test]
fn absent_vert_align_folds_to_center_not_default_top() {
    // `VerticalAlign::default()` 는 Top 이지만 HML 경로의 실효 기본값은 Center 다
    // (종전 adapter 하드코딩). 속성 없는 PARALIST 와 PARALIST 자체가 없는 셀 모두
    // Center 로 접혀야 기존 문서의 정렬이 바뀌지 않는다.
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><TABLE RowCount="1" ColCount="2">
        <ROW>
          <CELL ColAddr="0" RowAddr="0"><PARALIST><P><TEXT><CHAR>a</CHAR></TEXT></P></PARALIST></CELL>
          <CELL ColAddr="1" RowAddr="0"/>
        </ROW>
      </TABLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("최소 표 HML 은 파싱되어야 함");

    assert_eq!(
        cell_aligns(&parsed.document),
        vec![VerticalAlign::Center, VerticalAlign::Center],
        "속성이 없는 셀은 Center 로 접혀야 함"
    );
}

#[test]
fn textbox_paralist_does_not_hijack_enclosing_cell_alignment() {
    // PARALIST 는 글상자(DRAWTEXT) 아래에도 나온다. 셀 안 글상자의 PARALIST 가
    // 뒤늦게 등장한다는 이유로 바깥 셀의 정렬을 덮어쓰면 안 된다 — 문단 귀속(#2723)과
    // 같은 `nearest_paragraph_owner_is_cell` 판정을 공유한다.
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><TABLE RowCount="1" ColCount="1">
        <ROW><CELL ColAddr="0" RowAddr="0" Width="4000" Height="1200"><PARALIST VertAlign="Bottom"><P><TEXT><RECTANGLE X0="0" X1="1000" X2="1000" X3="0" Y0="0" Y1="0" Y2="500" Y3="500">
          <SHAPEOBJECT><SIZE Width="1000" Height="500"/></SHAPEOBJECT>
          <DRAWINGOBJECT><DRAWTEXT><PARALIST VertAlign="Top">
            <P><TEXT><CHAR>BOXTEXT</CHAR></TEXT></P>
          </PARALIST></DRAWTEXT></DRAWINGOBJECT>
        </RECTANGLE></TEXT></P></PARALIST></CELL></ROW>
      </TABLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let parsed = parse_hml(xml).expect("셀 안 글상자 HML 은 파싱되어야 함");

    assert_eq!(
        cell_aligns(&parsed.document),
        vec![VerticalAlign::Bottom],
        "글상자 PARALIST 가 바깥 셀 정렬을 덮어씀"
    );
}
