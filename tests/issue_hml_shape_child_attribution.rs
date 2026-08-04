//! HML 리소스 자식 요소 귀속 검증.
//!
//! CHARSHAPE/PARASHAPE 는 `Id` 속성 위치에 배치되는데(set_indexed), 자식 요소
//! (FONTID/RATIO/CHARSPACING/RELSIZE/CHAROFFSET, PARAMARGIN/PARABORDER)는
//! `last_mut()` 로 귀속된다. Id 가 내림차순이거나 상한 초과로 건너뛴 경우
//! 자식 값이 엉뚱한 리소스에 덮어써진다.

use rhwp::parser::hml::{parse_hml, parse_hml_with_limits, HmlLimits};

fn wrap_head(mapping_table: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Version="2.91">
  <HEAD SecCnt="1"><MAPPINGTABLE>{mapping_table}</MAPPINGTABLE></HEAD>
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>x</CHAR></TEXT></P></SECTION></BODY>
  <TAIL />
</HWPML>"#
    )
    .into_bytes()
}

/// Id 가 내림차순인 CHARSHAPE 목록: 각 도형의 RATIO 는 자기 CHARSHAPE 에 귀속되어야 한다.
#[test]
fn charshape_children_follow_out_of_order_ids() {
    let xml = wrap_head(
        r#"<CHARSHAPELIST Count="2">
             <CHARSHAPE Id="1" Height="1100">
               <RATIO Hangul="90" Latin="90" Hanja="90" Japanese="90" Other="90" Symbol="90" User="90"/>
             </CHARSHAPE>
             <CHARSHAPE Id="0" Height="1000">
               <RATIO Hangul="50" Latin="50" Hanja="50" Japanese="50" Other="50" Symbol="50" User="50"/>
             </CHARSHAPE>
           </CHARSHAPELIST>"#,
    );
    let result = parse_hml(&xml).expect("parse should succeed");
    let shapes = &result.document.doc_info.char_shapes;
    assert_eq!(shapes[1].ratios, [90; 7], "Id=1 의 RATIO 는 90 이어야 한다");
    assert_eq!(shapes[0].ratios, [50; 7], "Id=0 의 RATIO 는 50 이어야 한다");
}

/// 상한 초과로 건너뛴 CHARSHAPE 의 자식이 직전 정상 CHARSHAPE 를 오염시키면 안 된다.
#[test]
fn skipped_charshape_children_do_not_pollute_previous_shape() {
    let xml = wrap_head(
        r#"<CHARSHAPELIST Count="2">
             <CHARSHAPE Id="0" Height="1000">
               <RATIO Hangul="100" Latin="100" Hanja="100" Japanese="100" Other="100" Symbol="100" User="100"/>
             </CHARSHAPE>
             <CHARSHAPE Id="9" Height="1000">
               <RATIO Hangul="10" Latin="10" Hanja="10" Japanese="10" Other="10" Symbol="10" User="10"/>
             </CHARSHAPE>
           </CHARSHAPELIST>"#,
    );
    let limits = HmlLimits {
        max_resource_id: 4,
        ..HmlLimits::default()
    };
    let result = parse_hml_with_limits(&xml, &limits).expect("parse should succeed");
    let shapes = &result.document.doc_info.char_shapes;
    assert_eq!(
        shapes[0].ratios, [100; 7],
        "건너뛴 CHARSHAPE Id=9 의 RATIO 가 Id=0 을 덮어쓰면 안 된다"
    );
}

/// Id 가 내림차순인 PARASHAPE: PARAMARGIN 은 자기 PARASHAPE 에 귀속되어야 한다.
#[test]
fn parashape_margin_follows_out_of_order_ids() {
    let xml = wrap_head(
        r#"<PARASHAPELIST Count="2">
             <PARASHAPE Id="1" Align="Left">
               <PARAMARGIN Left="700" Right="700" LineSpacing="150"/>
             </PARASHAPE>
             <PARASHAPE Id="0" Align="Justify">
               <PARAMARGIN Left="300" Right="300" LineSpacing="200"/>
             </PARASHAPE>
           </PARASHAPELIST>"#,
    );
    let result = parse_hml(&xml).expect("parse should succeed");
    let shapes = &result.document.doc_info.para_shapes;
    assert_eq!(shapes[1].margin_left, 700, "Id=1 의 PARAMARGIN Left");
    assert_eq!(shapes[1].line_spacing, 150, "Id=1 의 LineSpacing");
    assert_eq!(shapes[0].margin_left, 300, "Id=0 의 PARAMARGIN Left");
    assert_eq!(shapes[0].line_spacing, 200, "Id=0 의 LineSpacing");
}
