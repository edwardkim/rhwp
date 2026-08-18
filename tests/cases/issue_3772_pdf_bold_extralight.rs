//! #3772: PDF bold run 이 Noto Sans KR ExtraLight 폴백으로 떨어져 굵기가 소실되지 않게 한다.
//!
//! Task #1224 ExtraLight 는 regular 본문 획 두께용이다. ExtraLight 는 별도 family
//! 이름이라 `font-weight="bold"` 가 Bold/Regular 로 넘어가지 않고, svg2pdf 는
//! faux-bold 를 합성하지 않는다. bold 체인에서 ExtraLight 를 빼면 Noto Sans KR
//! Regular/Bold 가 매칭된다.

const EXTRALIGHT: &str = "Noto Sans KR ExtraLight";

#[test]
fn issue_3772_bold_sans_chain_drops_extralight() {
    let regular = rhwp::renderer::render_font_family_chain("함초롬돋움");
    let bold = rhwp::renderer::render_font_family_chain_for_weight("함초롬돋움", true);
    assert!(
        regular.contains("'Noto Sans KR ExtraLight','Noto Sans KR'"),
        "regular 는 #1224 ExtraLight 계약을 유지해야 한다: {regular}"
    );
    assert!(
        !bold.contains(EXTRALIGHT),
        "bold 체인이 ExtraLight 에 떨어지면 svg2pdf 가 굵기를 잃는다: {bold}"
    );
    assert!(
        bold.contains("'Noto Sans KR'"),
        "bold 는 Noto Sans KR Regular/Bold 를 남겨야 한다: {bold}"
    );

    let light = rhwp::renderer::generic_fallback("KoPub돋움체 Light");
    let light_bold = rhwp::renderer::generic_fallback_for_weight("KoPub돋움체 Light", true);
    assert!(light.starts_with("'Noto Sans KR ExtraLight','Malgun Gothic'"));
    assert!(
        !light_bold.contains(EXTRALIGHT),
        "KoPub Light + bold 도 ExtraLight 를 쓰면 안 된다: {light_bold}"
    );
    assert!(light_bold.starts_with("'Malgun Gothic'"));
}

#[test]
fn issue_3772_pdf_prepare_path_strips_bold_extralight_only() {
    let options = rhwp::renderer::pdf::PdfExportOptions::default();
    let svg = concat!(
        r#"<svg>"#,
        r#"<text font-family="'Malgun Gothic','Noto Sans KR ExtraLight','Noto Sans KR'" font-weight="bold">제목</text>"#,
        r#"<text font-family="'Malgun Gothic','Noto Sans KR ExtraLight','Noto Sans KR'">본문</text>"#,
        r#"<text font-weight="700" font-family="&apos;Noto Sans KR ExtraLight&apos;,&apos;Noto Sans KR&apos;">강조</text>"#,
        r#"<text font-weight="500" font-family="'Noto Sans KR ExtraLight','Noto Sans KR'">중고딕</text>"#,
        r#"</svg>"#,
    );
    let out = rhwp::renderer::pdf::apply_pdf_font_options(svg, &options);

    let texts: Vec<&str> = out
        .split("<text")
        .skip(1)
        .map(|part| part.split("</text>").next().unwrap_or(part))
        .collect();
    assert_eq!(texts.len(), 4, "text 노드 수: {out}");

    assert!(
        !texts[0].contains(EXTRALIGHT),
        "bold 제목이 ExtraLight 에 떨어졌다: {}",
        texts[0]
    );
    assert!(
        texts[0].contains("Noto Sans KR"),
        "bold 제목은 Regular/Bold family 를 남겨야 한다: {}",
        texts[0]
    );
    assert!(
        texts[1].contains(EXTRALIGHT),
        "regular 본문은 #1224 ExtraLight 계약을 유지해야 한다: {}",
        texts[1]
    );
    assert!(
        !texts[2].contains(EXTRALIGHT),
        "font-weight=700 도 ExtraLight 를 쓰면 안 된다: {}",
        texts[2]
    );
    assert!(
        texts[3].contains(EXTRALIGHT),
        "font-weight=500 은 #3772 범위가 아니다: {}",
        texts[3]
    );
}
