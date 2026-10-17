//! [#7092] 한양 계열 이름을 TTF 로 선언한 문서의 가운뎃점 `·` — 한/글은 **문서가 적은 대체
//! 글꼴**로 실현을 가른다.
//!
//! - 대체가 TTF `HY중고딕`·`HY신명조` 면 그 TrueType 으로 그린다(정본 PDF 자원 `H2gtrM`,
//!   `·` 1.000~1.010em: 78494 n=61 · issue1891_external_bindata_link n=14 · 75544 n=26).
//! - 대체가 HFT(같은 이름)면 자기 HFT 글리프로 그린다(정본 Type3, `·` 0.381~0.386em:
//!   80168 의 2020·2022·2024 출력 전부).
//! - `HY견고딕` 을 직접 TTF 로 선언한 문서도 글꼴 파일 그대로 전각이다
//!   (`trade_report_forced_break_object-2024` 1.000em).
//!
//! 수정 전 rhwp 는 선언 형식과 무관하게 한양중고딕 HFT 실측 표(390/1024)를 써서 대체 TTF
//! 문서의 `·` 가 0.38em 이었고, `HY견고딕` 은 `.notdef` 덮어쓰기로 0.30em 이었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;

/// layer SVG 의 글자 하나짜리 `<text>`. `·` 는 합성 `<circle>` 뒤에 위치용 `<text>` 가 따라오며
/// `font-family` 가 없으므로, 같은 줄 바로 뒤 글자의 글꼴을 run 의 글꼴로 본다.
#[derive(Debug, Clone)]
struct Glyph {
    x: f64,
    y: f64,
    size: f64,
    text: String,
    family: Option<String>,
}

fn attr<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let at = tag.find(&format!(" {key}=\""))? + key.len() + 3;
    let rest = &tag[at..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn glyphs(svg: &str) -> Vec<Glyph> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(at) = rest.find("<text") {
        rest = &rest[at..];
        let Some(gt) = rest.find('>') else { break };
        let tag = &rest[..=gt];
        let body = &rest[gt + 1..];
        let Some(close) = body.find("</text>") else {
            break;
        };
        let text = body[..close].replace("&#183;", "\u{00B7}");
        if let (Some(x), Some(y), Some(size)) = (
            attr(tag, "x").and_then(|v| v.parse::<f64>().ok()),
            attr(tag, "y").and_then(|v| v.parse::<f64>().ok()),
            attr(tag, "font-size").and_then(|v| v.parse::<f64>().ok()),
        ) {
            if text.chars().count() == 1 && !text.trim().is_empty() {
                let family = attr(tag, "font-family").map(|f| {
                    f.split(',')
                        .next()
                        .unwrap_or(f)
                        .trim()
                        .trim_matches('\'')
                        .replace("&apos;", "")
                });
                out.push(Glyph {
                    x,
                    y,
                    size,
                    text,
                    family,
                });
            }
        }
        rest = &body[close..];
    }
    out
}

/// 같은 줄에서 `·` 다음 글자까지의 전진폭(em)과 그 뒤 글자의 글꼴. 뒤 글자가 공백이나 또 다른
/// `·` 면 전진폭이 아니라 제외한다.
fn dot_advances(svg: &str) -> Vec<(String, f64)> {
    let mut g = glyphs(svg);
    g.sort_by(|a, b| {
        (a.y * 10.0)
            .round()
            .total_cmp(&(b.y * 10.0).round())
            .then(a.x.total_cmp(&b.x))
    });
    g.windows(2)
        .filter(|w| {
            w[0].text == "\u{00B7}"
                && w[1].text != "\u{00B7}"
                && !w[1].text.trim().is_empty()
                && (w[0].y - w[1].y).abs() < 0.5
                && w[0].size > 0.0
        })
        .map(|w| {
            (
                w[1].family.clone().unwrap_or_default(),
                (w[1].x - w[0].x) / w[0].size,
            )
        })
        .filter(|(_, adv)| *adv > 0.0)
        .collect()
}

fn page_svg(path: &str, page: u32) -> String {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&full).expect("read sample")).expect("open sample");
    core.render_page_svg_layer_with_profile_native(page, RenderProfile::Print)
        .expect("SVG")
}

fn advances_of(svg: &str, family: &str) -> Vec<f64> {
    dot_advances(svg)
        .into_iter()
        .filter(|(f, _)| f == family)
        .map(|(_, adv)| adv)
        .collect()
}

fn assert_all_near(label: &str, advances: &[f64], expected: f64, tolerance: f64, min_count: usize) {
    assert!(
        advances.len() >= min_count,
        "{label}: `·` 가 {min_count}회 이상 나와야 한다 ({}회)",
        advances.len()
    );
    for adv in advances {
        assert!(
            (adv - expected).abs() <= tolerance,
            "{label}: `·` 전진폭은 {expected:.3}em 이어야 한다 — {adv:.3}em. 전체: {advances:?}"
        );
    }
}

#[test]
fn hanyang_face_with_ttf_substitute_measures_middle_dot_as_the_substitute_truetype() {
    // HWPX: `<hh:font face="한양중고딕" type="TTF"><hh:substFont face="HY중고딕" type="TTF"/>`.
    // 정본 `issue1891_external_bindata_link-2020.pdf` 2쪽 `H2gtrM` n=14 중앙값 1.010em.
    let svg = page_svg("samples/issue1891_external_bindata_link.hwpx", 1);
    assert_all_near(
        "issue1891_external_bindata_link 2쪽 한양중고딕",
        &advances_of(&svg, "한양중고딕"),
        1.0,
        0.05,
        3,
    );

    // HWP5: FACE_NAME 대체 글꼴 TTF `HY중고딕`(`alt_name`). 정본
    // `78494-virtual-convergence-industry-decree-2020.pdf` 2쪽 `H2gtrM` 1.000em.
    let svg = page_svg(
        "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx",
        1,
    );
    assert_all_near(
        "78494 2쪽 한양중고딕",
        &advances_of(&svg, "한양중고딕"),
        1.0,
        0.05,
        2,
    );
}

#[test]
fn hanyang_face_with_hft_substitute_keeps_the_hft_middle_dot() {
    // 80168: `한양중고딕` TTF 선언이지만 대체 글꼴이 HFT `한양중고딕` — 한/글은 자기 HFT
    // 글리프로 그린다. 정본 2020·2022·2024 출력 모두 Type3 `/W` 0.381em(2쪽 「결정·변경」
    // 0.380 · 「시·도조례」 0.386). 대체 TTF 규칙이 여기로 번지면 안 된다.
    let svg = page_svg("samples/80168_regulatory_analysis.hwp", 1);
    assert_all_near(
        "80168 2쪽 한양중고딕(HFT 대체)",
        &advances_of(&svg, "한양중고딕"),
        0.381,
        0.03,
        2,
    );
}

/// 합성 통제 실험 (`samples/issue7092/gen_subst_font.py`, 한/글 2024 13.0.0.564 Print →
/// `pdf/issue7092/substitute-font-*-2024.pdf`). 같은 본문·같은 선언에서 대체 글꼴만 다르다.
///
/// ```text
///   ttf   한양중고딕 TTF + substFont HY중고딕 TTF    → 정본 H2gtrM TrueType  · 1.000em (n=24)
///   hft   한양중고딕 TTF + substFont 한양중고딕 HFT  → 정본 Type3            · 0.380em (n=24)
///   none  한양중고딕 TTF, 대체 글꼴 없음            → 정본 Type3            · 0.380em (n=24)
///   ttf 2쪽 HY견고딕 직접 · 3쪽 휴먼명조 직접         → H2gtrE · 휴먼명조    · 1.000em
/// ```
#[test]
fn synthetic_variants_follow_the_declared_substitute_font() {
    let svg = page_svg("samples/issue7092/substitute-font-ttf.hwpx", 0);
    assert_all_near(
        "합성 ttf 1쪽 한양중고딕",
        &advances_of(&svg, "한양중고딕"),
        1.0,
        0.05,
        20,
    );
    let svg = page_svg("samples/issue7092/substitute-font-ttf.hwpx", 1);
    assert_all_near(
        "합성 ttf 2쪽 HY견고딕",
        &advances_of(&svg, "HY견고딕"),
        1.0,
        0.05,
        20,
    );
    let svg = page_svg("samples/issue7092/substitute-font-ttf.hwpx", 2);
    assert_all_near(
        "합성 ttf 3쪽 휴먼명조",
        &advances_of(&svg, "휴먼명조"),
        1.0,
        0.05,
        20,
    );

    let svg = page_svg("samples/issue7092/substitute-font-hft.hwpx", 0);
    assert_all_near(
        "합성 hft 1쪽 한양중고딕",
        &advances_of(&svg, "한양중고딕"),
        0.380,
        0.03,
        20,
    );
    let svg = page_svg("samples/issue7092/substitute-font-none.hwpx", 0);
    assert_all_near(
        "합성 none 1쪽 한양중고딕",
        &advances_of(&svg, "한양중고딕"),
        0.380,
        0.03,
        20,
    );
}

#[test]
fn directly_declared_hy_gothic_extra_measures_middle_dot_full_width() {
    // `HY견고딕` TTF 직접 선언. 정본 `issue6300/trade_report_forced_break_object-2024.pdf`
    // 10쪽 「국내·외」 1.000em (2020 출력 0.989). `H2GTRE.TTF` periodcentered 1024/1024.
    let svg = page_svg("samples/issue6300/trade_report_forced_break_object.hwp", 9);
    assert_all_near(
        "trade_report 10쪽 HY견고딕",
        &advances_of(&svg, "HY견고딕"),
        1.0,
        0.05,
        1,
    );
}

/// HWP5 `FACE_NAME` 의 대체 글꼴은 이름과 **유형**이 한 쌍이다. 종전 파서는 유형 바이트를 버리고
/// 직렬화는 글꼴 자신의 유형을 되썼으며(80168: 2 → 1), HWPX 저장은 HWP5 의 대체 글꼴을 아예
/// 싣지 않았다(`issue_1939` 왕복 렌더 935px 변위의 근인). 두 형식 모두 쌍을 보존해야 한다.
#[test]
fn hwp5_substitute_font_type_survives_hwp_and_hwpx_roundtrip() {
    fn substitute(doc: &rhwp::model::document::Document, face: &str) -> (String, u8) {
        let font = doc.doc_info.font_faces[0]
            .iter()
            .find(|font| font.name == face)
            .unwrap_or_else(|| panic!("한글 슬롯에 {face} 가 있어야 한다"));
        let subst = font
            .subst_font
            .as_ref()
            .unwrap_or_else(|| panic!("{face} 의 대체 글꼴이 subst_font 에 실려야 한다"));
        (subst.face.clone(), subst.font_type)
    }
    let read = |path: &str| {
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read")
    };
    for (path, expected) in [
        (
            "samples/80168_regulatory_analysis.hwp",
            ("한양중고딕".to_string(), 2u8),
        ),
        (
            "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx",
            ("HY중고딕".to_string(), 1u8),
        ),
    ] {
        let doc = rhwp::parser::parse_hwp(&read(path)).expect("HWP5 파싱");
        assert_eq!(substitute(&doc, "한양중고딕"), expected, "{path} 원본");

        let hwp = rhwp::serializer::serialize_hwp(&doc).expect("HWP5 직렬화");
        let again = rhwp::parser::parse_hwp(&hwp).expect("HWP5 재파싱");
        assert_eq!(
            substitute(&again, "한양중고딕"),
            expected,
            "{path} HWP5 왕복"
        );

        let hwpx = rhwp::serializer::serialize_hwpx(&doc).expect("HWPX 직렬화");
        let again = rhwp::parser::hwpx::parse_hwpx(&hwpx).expect("HWPX 재파싱");
        assert_eq!(
            substitute(&again, "한양중고딕"),
            expected,
            "{path} HWPX 왕복"
        );
    }
}
