//! Issue #2743 회귀 가드 — HML 리소스 `Id` 가 할당 크기가 되는 것을 상한으로 막는다.
//!
//! `set_indexed()` 가 `resize_with(Id + 1, ..)` 를 검증 없이 호출해, 파일에서 온
//! `Id` 값에 선형 비례하는 예약이 일어났다. 이 결함은 두 구간을 가진다.
//!
//! - **조용한 구간**: `Id="1000000"` → 힙 최대 120,009,531 바이트를 쓰고도
//!   `parse_hml` 이 `Ok` 를 반환한다. 오류도 경고도 없어 호출자가 알 수 없다.
//!   따라서 아래 가드는 "죽지 않음"이 아니라 **결과 테이블 길이와 경고**를 단언한다
//!   (그렇게 해야 수정 전에 red 가 된다).
//! - **abort 구간**: `Id="2000000000"` → 240,000,000,120 바이트 요구 →
//!   `handle_alloc_error` → abort. 테스트 실패가 아니라 바이너리 전체가 죽는다.

use rhwp::parser::hml::{parse_hml, HmlWarningCode};

/// 상한(기본 65,535)을 훨씬 넘는 `Id` 하나만 든 최소 HML.
fn hml_with_charshape_id(id: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1"><MAPPINGTABLE><CHARSHAPELIST Count="1"><CHARSHAPE Id="{id}" Height="1000"/></CHARSHAPELIST></MAPPINGTABLE></HEAD>
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>A</CHAR></TEXT></P></SECTION></BODY>
  <TAIL />
</HWPML>
"#
    )
    .into_bytes()
}

fn invalid_reference_warnings(result: &rhwp::parser::hml::HmlParseResult) -> usize {
    result
        .warnings
        .iter()
        .filter(|w| w.code == HmlWarningCode::InvalidReference)
        .count()
}

/// 조용한 구간 — 수정 전에는 `Ok` 이면서 테이블이 1,000,001칸이 된다.
/// 길이와 경고를 단언해야 red 가 성립한다.
#[test]
fn hml_resource_id_beyond_limit_is_skipped_with_warning() {
    let bytes = hml_with_charshape_id("1000000");
    assert_eq!(bytes.len(), 382, "재현 입력 크기 고정");

    let parsed = parse_hml(&bytes).expect("상한 초과 Id 여도 문서는 열려야 함");
    let len = parsed.document.doc_info.char_shapes.len();

    assert!(
        len < 1_000,
        "상한 초과 CHARSHAPE 는 테이블을 늘리지 않아야 함 (실제 {len}칸)"
    );
    assert_eq!(
        invalid_reference_warnings(&parsed),
        1,
        "건너뛴 리소스는 경고로 보고되어야 함 (조용히 사라지면 안 됨)"
    );
}

/// abort 구간 — 상한을 크게 넘는 Id 도 테이블을 늘리지 않아야 한다.
///
/// [메인테이너 보정] 종전에는 Id `2000000000` 을 썼다. 가드가 살아 있으면 안전하지만,
/// **가드가 회귀로 사라지면 `CharShape` 120B × 20억 = 240GB** 를 요구해 테스트가 실패하는
/// 대신 러너가 죽는다. 원저자도 "수정 전에는 테스트 실패가 아니라 프로세스가 죽는다" 고
/// 주석에 적어 두었는데, 그것이 바로 CI 에서 허용될 수 없는 성질이다.
///
/// 상한(65,535)을 크게 넘는다는 성질은 Id `2_000_000` 으로도 동일하게 검증되며, 회귀 시
/// 요구량은 240MB 로 진단 가능한 범위에 머문다. 경계 자체는 아래
/// `hml_resource_id_boundary_accepts_limit_and_rejects_above` 가 65535/65536 으로 고정한다.
#[test]
fn hml_resource_id_far_beyond_limit_does_not_abort() {
    let bytes = hml_with_charshape_id("2000000");
    assert_eq!(bytes.len(), 382, "재현 입력 크기 고정");

    let parsed = parse_hml(&bytes).expect("거대 테이블을 요구하지 말고 정상 파싱되어야 함");
    assert!(parsed.document.doc_info.char_shapes.len() < 1_000);
    assert_eq!(invalid_reference_warnings(&parsed), 1);
}

/// 여섯 개 호출부(FONT/BORDERFILL/CHARSHAPE/PARASHAPE/TABDEF/STYLE) 전부 유계여야 한다.
#[test]
fn hml_all_six_resource_kinds_are_bounded() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1"><MAPPINGTABLE>
    <FACENAMELIST><FONTFACE Count="1" Lang="Hangul"><FONT Id="9000000" Name="X" Type="ttf"/></FONTFACE></FACENAMELIST>
    <BORDERFILLLIST Count="1"><BORDERFILL Id="9000000"/></BORDERFILLLIST>
    <CHARSHAPELIST Count="1"><CHARSHAPE Id="9000000" Height="1000"/></CHARSHAPELIST>
    <TABDEFLIST Count="1"><TABDEF Id="9000000"/></TABDEFLIST>
    <PARASHAPELIST Count="1"><PARASHAPE Id="9000000" Align="Left"/></PARASHAPELIST>
    <STYLELIST Count="1"><STYLE Id="9000000" Name="s"/></STYLELIST>
  </MAPPINGTABLE></HEAD>
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>A</CHAR></TEXT></P></SECTION></BODY>
  <TAIL />
</HWPML>
"#;

    let parsed = parse_hml(xml).expect("여섯 종류 전부 상한 초과여도 문서는 열려야 함");
    let d = &parsed.document.doc_info;

    for (name, len) in [
        ("char_shapes", d.char_shapes.len()),
        ("para_shapes", d.para_shapes.len()),
        ("styles", d.styles.len()),
        ("border_fills", d.border_fills.len()),
        ("tab_defs", d.tab_defs.len()),
    ] {
        assert!(len < 1_000, "{name} 가 상한 초과 Id 로 늘어남 ({len}칸)");
    }
    for faces in &d.font_faces {
        assert!(
            faces.len() < 1_000,
            "font_faces 가 늘어남 ({}칸)",
            faces.len()
        );
    }

    assert_eq!(
        invalid_reference_warnings(&parsed),
        6,
        "여섯 호출부가 각각 경고를 남겨야 함"
    );
}

/// 경계값 — 상한 이하는 종전대로 수용, 상한 초과만 건너뛴다.
#[test]
fn hml_resource_id_boundary_accepts_limit_and_rejects_above() {
    let accepted = parse_hml(&hml_with_charshape_id("65535")).expect("상한값은 수용되어야 함");
    assert_eq!(
        accepted.document.doc_info.char_shapes.len(),
        65_536,
        "Id=65535 는 종전대로 65,536칸을 만들어야 함"
    );
    assert_eq!(invalid_reference_warnings(&accepted), 0, "경고가 없어야 함");

    let rejected = parse_hml(&hml_with_charshape_id("65536")).expect("상한 초과도 열려야 함");
    assert!(
        rejected.document.doc_info.char_shapes.len() < 1_000,
        "Id=65536 는 건너뛰어야 함"
    );
    assert_eq!(invalid_reference_warnings(&rejected), 1);
}

/// 정상 범위 Id 는 동작이 완전히 불변이어야 한다 (수정 전후 모두 통과하는 가드).
#[test]
fn hml_resource_ids_within_limit_are_unchanged() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<HWPML Style="embed" SubVersion="9.0.1.0" Version="2.9">
  <HEAD SecCnt="1"><MAPPINGTABLE>
    <FACENAMELIST><FONTFACE Count="2" Lang="Hangul"><FONT Id="0" Name="굴림" Type="ttf"/><FONT Id="1" Name="바탕" Type="ttf"/></FONTFACE></FACENAMELIST>
    <BORDERFILLLIST Count="2"><BORDERFILL Id="1"/><BORDERFILL Id="2"/></BORDERFILLLIST>
    <CHARSHAPELIST Count="3"><CHARSHAPE Id="0" Height="1000"/><CHARSHAPE Id="1" Height="1100"/><CHARSHAPE Id="2" Height="1200"/></CHARSHAPELIST>
    <TABDEFLIST Count="2"><TABDEF Id="0"/><TABDEF Id="1"/></TABDEFLIST>
    <PARASHAPELIST Count="2"><PARASHAPE Id="0" Align="Left"/><PARASHAPE Id="1" Align="Center"/></PARASHAPELIST>
    <STYLELIST Count="2"><STYLE Id="0" Name="바탕글"/><STYLE Id="1" Name="본문"/></STYLELIST>
  </MAPPINGTABLE></HEAD>
  <BODY><SECTION Id="0"><P ParaShape="0" Style="0"><TEXT CharShape="0"><CHAR>가</CHAR></TEXT></P></SECTION></BODY>
  <TAIL />
</HWPML>
"#
    .as_bytes();

    let parsed = parse_hml(xml).expect("정상 HML 은 그대로 파싱되어야 함");
    let d = &parsed.document.doc_info;

    assert_eq!(d.char_shapes.len(), 3);
    assert_eq!(d.para_shapes.len(), 2);
    assert_eq!(d.styles.len(), 2);
    assert_eq!(d.border_fills.len(), 2);
    assert_eq!(d.tab_defs.len(), 2);
    assert_eq!(d.font_faces[0].len(), 2);
    assert_eq!(d.char_shapes[1].base_size, 1100, "값도 그대로여야 함");
    assert_eq!(d.char_shapes[2].base_size, 1200);
    assert_eq!(
        invalid_reference_warnings(&parsed),
        0,
        "정상 파일에는 경고가 없어야 함"
    );
}
