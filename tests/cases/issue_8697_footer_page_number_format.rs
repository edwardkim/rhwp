//! 고정형(쪽 번호 문자열·쪽수)·관계형(`〈` 전진/글자 크기): 꼬리말 쪽 번호 자동 번호가 저장된 번호 모양(표 134)을 따라 앞부분은 소문자 로마 숫자, 본문은 아라비아 숫자로 찍히는지, 전체 쪽수가 독립 PDF 와 같은지, 표 목차의 `〈`(기호 슬롯 HFT 한양중고딕)가 독립 Print 와 같은 전진으로 그려지는지 검사한다.
//! 주석 갱신: 2026-11-07T10:13:21+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-07 KST, base = #8685 head 34368de8cf(upstream/devel 45684640be + #8677 + #8685), head = base + 이 PR 의 src 변경.
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 독립 한컴 Print(Hwp 2022 12.0.0.4547 생성).
//! 원본 `samples/issue8685/1311000-201100125_saemaul_oda_pilot_report.hwp` / `pdf/issue8685/1311000-201100125_saemaul_oda_pilot_report-2020.pdf`, 전체 209쪽=209쪽.
//! 5~8쪽(목차) 수정 전 Native 83.55·83.42·84.04·85.10 / fresh WASM 95.77·94.94·96.88·95.13
//! → 수정 후 Native 99.91·99.85·99.91·99.84 / fresh WASM 97.85·97.67·97.94·98.22.
//! 수정 후 전쪽 최저 Native 93.94·fresh WASM 95.05(90% 미만 0쪽, 수정 전 Native 4쪽).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8697/{base,head}/{native,wasm}/saemaul_oda_pilot_report_silhouette.tsv`, `png/`.
//! 독립 기대값: 같은 PDF 의 꼬리말 글자(2~8쪽 「i」~「vii」, 10쪽 「2」, 11쪽 「3」, 20쪽 「12」)와 6쪽 「〈표」의 `〈` 다음 글자 원점(83 단위 글자 중 79 단위 = 0.952em).
//! 확대 판독: 6쪽 꼬리말 「v」가 한/글과 같은 자리, 목차 `〈`·`〉`·`‧` 와 「4-13」 하이픈이 한/글과 같은 간격(수정 전 `〈` 잉크가 「표」에 겹치고 하이픈 뒤 빈칸).
//! 같은 assertion 이 수정 전 FAIL(꼬리말 「1」≠「i」, `〈` 0.476em) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8685/1311000-201100125_saemaul_oda_pilot_report.hwp";
const PDF_PAGES: u32 = 209;

fn text_of(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // 머리말·꼬리말 필드는 모델 1자를 화면 N자로 보이므로 표시 텍스트가 정본이다.
        out.push_str(run.display_text.as_deref().unwrap_or(&run.text));
    }
    for child in &node.children {
        text_of(child, out);
    }
}

fn footer_text(node: &RenderNode) -> Option<String> {
    if matches!(node.node_type, RenderNodeType::Footer) {
        let mut text = String::new();
        text_of(node, &mut text);
        return Some(text.split_whitespace().collect::<String>());
    }
    node.children.iter().find_map(footer_text)
}

#[test]
fn footer_page_number_follows_stored_number_format() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let footer = |page: u32| {
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        footer_text(&tree.root).unwrap_or_default()
    };

    // 앞부분(목차) 꼬리말: 번호 모양 3(소문자 로마 숫자) 하나만 있다.
    for (page, roman) in (1..=7).zip(["i", "ii", "iii", "iv", "v", "vi", "vii"]) {
        assert_eq!(footer(page), roman, "{}쪽 꼬리말 쪽 번호", page + 1);
    }
    // 본문 꼬리말: 번호 모양 0(아라비아 숫자) 뒤에 쪽 제목이 온다.
    for (page, digits) in [(9, "2"), (10, "3"), (19, "12")] {
        let text = footer(page);
        assert!(
            text.starts_with(digits)
                && !text[digits.len()..].starts_with(|c: char| c.is_ascii_digit()),
            "{}쪽 꼬리말은 「{digits}」로 시작한다: {text:?}",
            page + 1
        );
    }
}

fn attr<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let at = tag.find(&format!(" {key}=\""))? + key.len() + 3;
    let rest = &tag[at..];
    Some(&rest[..rest.find('"')?])
}

/// paint 결과(layer SVG)의 한 글자 `<text>` 위치: (y, x, 글자 크기, 글자).
fn painted_glyphs(svg: &str) -> Vec<(f64, f64, f64, String)> {
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
        let text = body[..close].to_string();
        let position = match (attr(tag, "x"), attr(tag, "y")) {
            (Some(x), Some(y)) => x.parse::<f64>().ok().zip(y.parse::<f64>().ok()),
            _ => attr(tag, "transform").and_then(|t| {
                let inner = t.strip_prefix("translate(")?;
                let (x, y) = inner[..inner.find(')')?].split_once(',')?;
                x.trim()
                    .parse::<f64>()
                    .ok()
                    .zip(y.trim().parse::<f64>().ok())
            }),
        };
        if let (Some((x, y)), Some(size)) = (
            position,
            attr(tag, "font-size").and_then(|v| v.parse::<f64>().ok()),
        ) {
            if text.chars().count() == 1 {
                out.push((y, x, size, text));
            }
        }
        rest = &body[close..];
    }
    out
}

/// 6쪽 표 목차 「〈표 4-13〉…」의 `〈` 는 기호 슬롯 HFT 한양중고딕(장평 100%, 자간 −5%)으로
/// 재고 그린다. 독립 Print 는 이 글자를 Type3 폭 1000/1000 으로 그리고 다음 글자가 83 단위
/// 글자 크기 중 79 단위 뒤에 온다(0.952em). 수정 전 rhwp 의 paint 는 반각(0.476em)이라 뒤
/// 글자들이 당겨지고 `〈` 잉크가 「표」에 겹쳤다.
#[test]
fn toc_angle_bracket_takes_full_hft_advance() {
    let mut core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    // CLI·Visual Sweep 의 기본 Native 웹폰트 환경과 같은 꼴: HFT 를 표시용 TTF 로 대체하되
    // 원본 HFT 메트릭을 유지한다(preserveSourceMetrics). 글꼴 파일은 공급하지 않는다.
    let environment = Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(
            r#"{"id":"issue8697-native-webfont-shape","preserveSourceMetrics":true,
                "substitutions":{"한양중고딕":"NotoSansKR-Regular","신명 중고딕":"NotoSansKR-Regular"}}"#,
        )
        .expect("환경 JSON"),
    );
    core.set_font_environment(environment).expect("환경 적용");
    let svg = core
        .render_page_svg_layer_with_profile_native(5, rhwp::paint::RenderProfile::Print)
        .expect("SVG");
    let mut glyphs = painted_glyphs(&svg);
    glyphs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    // 같은 줄(같은 y)에서 `〈` 바로 뒤 글자의 시작까지가 `〈` 의 전진이다.
    let advances: Vec<(f64, f64)> = glyphs
        .windows(2)
        .filter(|pair| pair[0].3 == "〈" && (pair[0].0 - pair[1].0).abs() < 0.5)
        .map(|pair| (pair[1].1 - pair[0].1, pair[0].2))
        .collect();
    assert!(advances.len() >= 20, "6쪽 표 목차의 〈: {}", advances.len());
    for (advance, size) in advances {
        let em = advance / size;
        assert!(
            (0.94..=0.96).contains(&em),
            "〈 전진 {advance:.3}px / 글자 크기 {size:.3}px = {em:.3}em, 독립 Print 0.952em"
        );
    }
}
