//! 검사 유형: 관계형. 실제 원본 프로그램의 존재 확인과 자연 전진 선택은 종이 프레임을 따로 소유한다.
//! 주석 갱신: 2026-11-09T09:51:29+09:00. 신규 검사.
//! 공개 합성 CFF와 HWPML의 선택 계약이며 실물 페이지 좌표를 고정하지 않는다.
//! 원본 프로그램 조회·다른 대체 조회·자연 전진 요청을 독립 대조한다.
use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
fn frames(node: &RenderNode, out: &mut Vec<bool>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains('가') {
            out.push(run.style.hft_print_frame)
        }
    }
    for c in &node.children {
        frames(c, out)
    }
}
fn core() -> DocumentCore {
    let langs = [
        "Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User",
    ];
    let fonts=langs.iter().map(|l|format!("<FONTFACE Count=\"1\" Lang=\"{l}\"><FONT Id=\"0\" Name=\"명조\" Type=\"hft\"/></FONTFACE>")).collect::<String>();
    let xml=format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><HWPML Version=\"2.91\"><HEAD SecCnt=\"1\"><MAPPINGTABLE><FACENAMELIST>{fonts}</FACENAMELIST><CHARSHAPELIST Count=\"1\"><CHARSHAPE Id=\"0\" Height=\"2400\"/></CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id=\"0\"><P ParaShape=\"0\"><TEXT CharShape=\"0\"><CHAR>가</CHAR></TEXT></P></SECTION></BODY></HWPML>");
    DocumentCore::from_bytes(xml.as_bytes()).unwrap()
}
#[test]
fn original_hft_program_keeps_its_print_frame_when_source_metrics_are_preserved() {
    for (preserve, query, expected) in [
        (true, "명조", true),
        (false, "명조", false),
        (true, "다른 대체 프로그램", false),
    ] {
        let mut c = core();
        let env = format!(
            r#"{{"id":"source-hft-frame","preserveSourceMetrics":{preserve},"substitutions":{{"명조":"RHWP HFT Outline Fixture"}},"selectedPrograms":{{"명조":{{"query":"{query}","family":"RHWP HFT Outline Fixture","openTypeCff":true,"programId":"sweep-c3eb4ea18997f2a79442f80e51e421bfe496edb431a9247f27dafd4fcc305dda"}}}}}}"#
        );
        c.set_font_environment(Some(FontEnvironment::from_json(&env).unwrap()))
            .unwrap();
        c.prepare_native_otf_fonts_with(|_| {
            Some((
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/fonts/RHWPHftOutline.otf"
                ))
                .to_vec(),
                0,
            ))
        })
        .unwrap();
        let t = c.build_page_render_tree(0).unwrap();
        let mut f = vec![];
        frames(&t.root, &mut f);
        assert!(!f.is_empty());
        assert!(
            f.iter().all(|v| *v == expected),
            "preserve={preserve} query={query} frames={f:?}"
        );
    }
}
