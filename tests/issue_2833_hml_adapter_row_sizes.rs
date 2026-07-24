//! Issue #2833 회귀 가드 — HML 파서 `into_table()` 의 `row_sizes` 계산이
//! `O(row_count × cells.len())` 이던 문제(#2751 의 import 경로 잔여).
//! `RowCount` 만 크게 부풀리고 `CELL` 은 정상 개수인 입력(뷰어가 여는 "멀쩡한"
//! 문서)이 파싱 자체를 느리게 만들지 않아야 한다.

use rhwp::parser::hml::parse_hml;
use std::time::Instant;

/// `RowCount` 를 크게, `CELL` 은 `cell_count`개(모두 `RowAddr="0"`)로 구성한
/// 최소 HML 표 문서.
fn hml_with_inflated_row_count(row_count: u32, cell_count: usize) -> Vec<u8> {
    let mut cells = String::new();
    for _ in 0..cell_count {
        cells.push_str(
            r#"<CELL BorderFill="1" ColAddr="0" ColSpan="1" Dirty="false" Editable="false" HasMargin="false" Header="false" Height="100" Protect="false" RowAddr="0" RowSpan="1" Width="1000"><CELLMARGIN Bottom="0" Left="0" Right="0" Top="0"/><PARALIST LineWrap="Break" LinkListID="0" LinkListIDNext="0" TextDirection="0" VertAlign="Center"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>x</CHAR></TEXT></P></PARALIST></CELL>"#,
        );
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1"><MAPPINGTABLE><BORDERFILLLIST Count="1"><BORDERFILL BackSlash="0" BreakCellSeparateLine="0" CenterLine="0" CounterBackSlash="0" CounterSlash="0" CrookedSlash="0" Id="1" Shadow="false" Slash="0" ThreeD="false"><LEFTBORDER Type="None" Width="0.1mm"/><RIGHTBORDER Type="None" Width="0.1mm"/><TOPBORDER Type="None" Width="0.1mm"/><BOTTOMBORDER Type="None" Width="0.1mm"/><DIAGONAL Type="Solid" Width="0.1mm"/></BORDERFILL></BORDERFILLLIST><CHARSHAPELIST Count="1"><CHARSHAPE BorderFillId="1" Height="1000" Id="0" ShadeColor="4294967295" SymMark="0" TextColor="0" UseFontSpace="false" UseKerning="false"/></CHARSHAPELIST><PARASHAPELIST Count="1"><PARASHAPE Align="Left" AutoSpaceEAsianEng="false" AutoSpaceEAsianNum="false" BreakLatinWord="KeepWord" BreakNonLatinWord="false" Condense="0" FontLineHeight="false" HeadingType="None" Id="0" KeepLines="false" KeepWithNext="false" Level="0" LineWrap="Break" PageBreakBefore="false" SnapToGrid="true" TabDef="0" VerAlign="Baseline" WidowOrphan="false"><PARAMARGIN Indent="0" Left="0" LineSpacing="160" LineSpacingType="Percent" Next="0" Prev="0" Right="0"/><PARABORDER BorderFill="1" Connect="false" IgnoreMargin="false"/></PARASHAPE></PARASHAPELIST><STYLELIST Count="1"><STYLE CharShape="0" EngName="Normal" Id="0" LangId="1042" LockForm="0" Name="바탕글" NextStyle="0" ParaShape="0" Type="Para"/></STYLELIST></MAPPINGTABLE></HEAD>
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>a</CHAR><TABLE BorderFill="1" CellSpacing="0" ColCount="1" PageBreak="Cell" RepeatHeader="true" RowCount="{row_count}"><SHAPEOBJECT InstId="1" Lock="false" NumberingType="Table" TextFlow="BothSides" ZOrder="0"><SIZE Height="100" HeightRelTo="Absolute" Protect="false" Width="1000" WidthRelTo="Absolute"/><POSITION AffectLSpacing="false" AllowOverlap="false" FlowWithText="true" HoldAnchorAndSO="false" HorzAlign="Left" HorzOffset="0" HorzRelTo="Para" TreatAsChar="true" VertAlign="Top" VertOffset="0" VertRelTo="Para"/><OUTSIDEMARGIN Bottom="0" Left="0" Right="0" Top="0"/></SHAPEOBJECT><INSIDEMARGIN Bottom="0" Left="0" Right="0" Top="0"/><ROW>{cells}</ROW></TABLE><CHAR>b</CHAR></TEXT></P></SECTION></BODY>
  <TAIL/>
</HWPML>
"#
    )
    .into_bytes()
}

#[test]
fn inflated_row_count_does_not_slow_down_parsing() {
    // 실측(이슈 본문): RowCount=60000, CELL=3000 이 수정 전 코드에서 O(row_count ×
    // cells.len()) = 1.8억 회 비교를 강제해 120~180ms/call 대였다. 수정 후에는
    // O(row_count + cells.len()) 이라 같은 입력이 훨씬 빨라야 한다. 느슨한 상한
    // (수 초)으로 회귀만 잡는다 — CI 환경 편차를 고려해 타이트하게 걸지 않는다.
    let bytes = hml_with_inflated_row_count(60_000, 3_000);

    let started = Instant::now();
    let parsed = parse_hml(&bytes).expect("RowCount 부풀림이 있어도 파싱은 성공해야 함");
    let elapsed = started.elapsed();

    assert!(
        elapsed.as_millis() < 500,
        "row_sizes 계산이 O(row_count × cells.len()) 로 되돌아가면 이 상한을 넘음 (실제 {elapsed:?})"
    );

    // row_sizes 값 자체도 정확해야 한다: row=0 에 cells.len() 개, 나머지는 0.
    use rhwp::model::control::Control;
    let table = parsed
        .document
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤이 있어야 함");
    assert_eq!(table.row_sizes.len(), 60_000);
    assert_eq!(table.row_sizes[0], 3_000);
    assert!(table.row_sizes[1..].iter().all(|&n| n == 0));
}
