//! [Issue #5717] 쪽 배경/테두리 "구역 첫 쪽에만 표시" (HWP5 구역정의 flags bit 8/9,
//! HWPX `hp:visibility` `border`/`fill`=`SHOW_FIRST`).
//!
//! 성북구 자원순환집행계획(HWP5, flags=0x00300200) 실측 — 한글 2022 는 남색 쪽
//! 배경(#1c3d62)을 1쪽에만 칠하는데 rhwp 는 172쪽 전부에 칠했다. 판별자는
//! PageBorderFill 레코드가 아니라 구역정의 속성 bit9 다(같은 [X,1,1] 슬롯의
//! 테두리 문서 156494214 는 bit 가 꺼져 있고 한글도 3/3쪽 전부에 그린다 — COM
//! PDF 양성대조). 한글 2022 가 같은 문서를 HWPX 로 저장하면
//! `fill="SHOW_FIRST"` 로 내보낸다(SaveAs 실측).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, Fill, FillType, SolidFill};
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

const FIXTURE_TEXT: &[u8] = include_bytes!("../../samples/hwpx/ref/ref_text.hwpx");
/// 렌더 게이트용 다문단 픽스처 — ref_text 는 문단 1개가 secPr 를 품고 있어
/// 문단 복제가 구역까지 복제해 버린다. 컨트롤 없는 본문 문단이 있는 문서를 쓴다.
const FIXTURE_MULTI: &[u8] = include_bytes!("../../samples/hwpx/business_overview.hwpx");

/// #1c3d62 남색 — COLORREF(0x00BBGGRR) 표기.
const NAVY: u32 = 0x0062_3D1C;

/// 남색 단색 채우기 BorderFill 을 추가하고 1-based id 를 돌려준다.
fn push_navy_border_fill(doc: &mut rhwp::model::document::Document) -> u16 {
    doc.doc_info.border_fills.push(BorderFill {
        fill: Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: NAVY,
                pattern_color: 0,
                pattern_type: -1,
            }),
            ..Default::default()
        },
        ..Default::default()
    });
    doc.doc_info.border_fills.len() as u16
}

/// 본문 문단을 복제해 2쪽 이상으로 늘린다. 구역정의 등 컨트롤을 품은 문단을
/// 복제하면 구역까지 복제되므로 컨트롤 없는 순수 텍스트 문단만 고른다.
fn grow_to_multiple_pages(doc: &mut rhwp::model::document::Document) {
    let body = doc.sections[0]
        .paragraphs
        .iter()
        .find(|p| !p.text.is_empty() && p.controls.is_empty())
        .expect("컨트롤 없는 본문 문단")
        .clone();
    for _ in 0..160 {
        doc.sections[0].paragraphs.push(body.clone());
    }
}

#[test]
fn issue_5717_show_first_survives_hwpx_roundtrip() {
    let mut doc = parse_hwpx(FIXTURE_TEXT).expect("parse ref_text");
    {
        let sd = &mut doc.sections[0].section_def;
        assert!(!sd.first_page_border, "기본값은 제한 없음");
        assert!(!sd.first_page_fill, "기본값은 제한 없음");
        sd.first_page_border = true;
        sd.first_page_fill = true;
        sd.flags |= 0x0300;
    }
    let bytes = serialize_hwpx(&doc).expect("serialize");
    let reparsed = parse_hwpx(&bytes).expect("reparse");
    let sd = &reparsed.sections[0].section_def;
    assert!(
        sd.first_page_border,
        "border=SHOW_FIRST 가 왕복에서 살아남아야 한다"
    );
    assert!(
        sd.first_page_fill,
        "fill=SHOW_FIRST 가 왕복에서 살아남아야 한다"
    );
    assert_eq!(sd.flags & 0x0300, 0x0300, "HWP5 flags bit 8/9 동기");
}

#[test]
fn issue_5717_default_visibility_stays_show_all() {
    let doc = parse_hwpx(FIXTURE_TEXT).expect("parse ref_text");
    let bytes = serialize_hwpx(&doc).expect("serialize");
    let xml = {
        // section0.xml 만 확인 — visibility 어휘가 SHOW_ALL 로 남아야 한다.
        let cursor = std::io::Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor).expect("zip");
        let mut s = String::new();
        std::io::Read::read_to_string(
            &mut zip.by_name("Contents/section0.xml").expect("section0"),
            &mut s,
        )
        .expect("read");
        s
    };
    assert!(
        xml.contains(r#"border="SHOW_ALL""#) && xml.contains(r#"fill="SHOW_ALL""#),
        "제한 없는 문서는 SHOW_ALL 을 유지해야 한다"
    );
    assert!(!xml.contains("SHOW_FIRST"));
}

/// 렌더 게이트 — fill=SHOW_FIRST 문서는 커스텀 쪽 배경이 구역 첫 쪽에만 칠해진다.
#[test]
fn issue_5717_fill_show_first_paints_first_page_only() {
    let mut doc = parse_hwpx(FIXTURE_MULTI).expect("parse fixture");
    let navy_bfid = push_navy_border_fill(&mut doc);
    {
        let sd = &mut doc.sections[0].section_def;
        sd.page_border_fill.border_fill_id = navy_bfid;
        sd.first_page_fill = true;
        sd.flags |= 0x0200;
    }
    grow_to_multiple_pages(&mut doc);

    let bytes = serialize_hwpx(&doc).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let first = core.render_page_svg_native(0).expect("page 1 svg");
    assert!(
        first.contains("#1c3d62"),
        "구역 첫 쪽에는 남색 배경이 칠해져야 한다"
    );
    let second = core.render_page_svg_native(1).expect("page 2 svg");
    assert!(
        !second.contains("#1c3d62"),
        "fill=SHOW_FIRST 문서의 2쪽에는 남색 배경이 없어야 한다 (#5717)"
    );
}

/// 회귀 가드 — 플래그가 꺼진 문서(코퍼스 [X,1,1] 테두리 11건과 동형)는 전 쪽 유지.
#[test]
fn issue_5717_without_show_first_fill_paints_every_page() {
    let mut doc = parse_hwpx(FIXTURE_MULTI).expect("parse fixture");
    let navy_bfid = push_navy_border_fill(&mut doc);
    doc.sections[0].section_def.page_border_fill.border_fill_id = navy_bfid;
    grow_to_multiple_pages(&mut doc);

    let bytes = serialize_hwpx(&doc).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let second = core.render_page_svg_native(1).expect("page 2 svg");
    assert!(
        second.contains("#1c3d62"),
        "SHOW_FIRST 가 아닌 쪽 배경은 종전대로 전 쪽에 칠해져야 한다 (156494214 계약)"
    );
}

/// 남색 실선 4방향 테두리 BorderFill 을 추가하고 1-based id 를 돌려준다.
fn push_navy_border_lines(doc: &mut rhwp::model::document::Document) -> u16 {
    let line = BorderLine {
        line_type: BorderLineType::Solid,
        width: 2,
        color: NAVY,
    };
    doc.doc_info.border_fills.push(BorderFill {
        borders: [line.clone(), line.clone(), line.clone(), line],
        ..Default::default()
    });
    doc.doc_info.border_fills.len() as u16
}

/// 렌더 게이트 — border=SHOW_FIRST 문서는 쪽 테두리가 구역 첫 쪽에만 그려진다.
///
/// 배경 축(bit 9)과 달리 테두리 축(bit 8)은 `build_page_borders` 게이트를 탄다.
/// 두 축이 같은 구역 정의 상태에서 갈라져 나오는지를 여기서 잠근다.
#[test]
fn issue_5717_border_show_first_paints_first_page_only() {
    let mut doc = parse_hwpx(FIXTURE_MULTI).expect("parse fixture");
    let navy_bfid = push_navy_border_lines(&mut doc);
    {
        let sd = &mut doc.sections[0].section_def;
        sd.page_border_fill.border_fill_id = navy_bfid;
        sd.first_page_border = true;
        sd.flags |= 0x0100;
    }
    grow_to_multiple_pages(&mut doc);

    let bytes = serialize_hwpx(&doc).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let first = core.render_page_svg_native(0).expect("page 1 svg");
    assert!(
        first.contains("#1c3d62"),
        "구역 첫 쪽에는 남색 쪽 테두리가 그려져야 한다"
    );
    let second = core.render_page_svg_native(1).expect("page 2 svg");
    assert!(
        !second.contains("#1c3d62"),
        "border=SHOW_FIRST 문서의 2쪽에는 쪽 테두리가 없어야 한다 (#5717)"
    );
}

/// 회귀 가드 — bit 8 이 꺼진 문서(156494214 계약)는 쪽 테두리를 전 쪽에 유지한다.
#[test]
fn issue_5717_without_show_first_border_paints_every_page() {
    let mut doc = parse_hwpx(FIXTURE_MULTI).expect("parse fixture");
    let navy_bfid = push_navy_border_lines(&mut doc);
    doc.sections[0].section_def.page_border_fill.border_fill_id = navy_bfid;
    grow_to_multiple_pages(&mut doc);

    let bytes = serialize_hwpx(&doc).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let second = core.render_page_svg_native(1).expect("page 2 svg");
    assert!(
        second.contains("#1c3d62"),
        "SHOW_FIRST 가 아닌 쪽 테두리는 종전대로 전 쪽에 그려져야 한다 (156494214 계약)"
    );
}
