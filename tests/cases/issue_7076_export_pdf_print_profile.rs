//! [#7076] 프로필을 안 받는 PDF 갈래가 `Screen` 으로 그려 **빈 누름틀 안내문이 인쇄된다**.
//!
//! ## 증상
//!
//! ```bash
//! rhwp export-pdf tests/fixtures/issue_7076/ship-collision-analysis-form.hwp -o out.pdf
//! ```
//!
//! 공식 별지 서식의 빈칸마다 작성 안내문(`홍길동`, `2021-충돌-001`, `부산-KICS-0000`,
//! `A선박` …)이 붉은 글씨로 찍힌다. 한/글로 같은 문서를 인쇄하면 그 칸들은 비어 있다.
//!
//! ## 근인
//!
//! `#3375` 가 그 안내문을 `editor_only` 로 표시해 인쇄에서 빼도록 만들어 두었고 기계는
//! 정상이다. 꺼져 있던 것은 **스위치**다 — `render_pages_pdf_native_with_options` 가
//! `render_page_svg_native`(= layer 경로 `RenderProfile::Screen`)를 불렀다. 같은 파일의
//! direct 백엔드(`render_pages_pdf_direct_native*`)는 모든 층이 처음부터 `Print` 였으니
//! 두 백엔드의 계약이 갈려 있었던 셈이다.
//!
//! ## 정본 대조 (한/글 2020 · `lastSavedWith` = hancom-office-2020)
//!
//! 1쪽 텍스트를 글자 멀티셋으로 갈랐다.
//!
//! ```text
//!   정본 202자 · 수정 전 기본(Screen) 353자 · 수정 후 기본(Print) 152자
//!   정본에 없는데 인쇄된 글자 : 수정 전 192자  →  수정 후 0자
//! ```
//!
//! 수정 후 글자는 정본의 부분집합이다(차집합 0). 남는 50자는 두 프로필 모두에서 같은
//! 방향으로 빠지는 PDF 텍스트 추출(ToUnicode) 축이라 이 결함과 다른 축이다 — 같은 쪽을
//! 래스터로 겹쳐 보면 수정 후는 정본과 같은 빈 서식이다.
//!
//! ## 이 시험이 재는 것
//!
//! ① 프로필 없는 갈래의 산출이 `Print` 갈래와 **바이트 동일**하고 `Screen` 과는 다르다.
//! ② 그 프로필 차이가 실제로 안내문을 지운다 — 안내문에만 나오는 글자(`홍`·`길`·`동`)가
//!    `Screen` SVG 에는 있고 `Print` SVG 에는 하나도 없다. PDF 는 자소를 서브셋 글꼴로
//!    싣기 때문에 바이트에서 글자를 찾을 수 없어, 같은 layer SVG 를 글자 단위로 센다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::renderer::pdf::PdfExportOptions;
use rhwp::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../fixtures/issue_7076/ship-collision-analysis-form.hwp");

/// 값이 채워지지 않은 누름틀 안내문에만 나오는 글자 — 서식 본문에는 없다.
/// (`홍길동` 은 `성명` 칸 예시, 나머지 칸도 같은 스위치를 탄다.)
const GUIDE_ONLY_CHARS: [char; 3] = ['홍', '길', '동'];

fn open() -> DocumentCore {
    DocumentCore::from_bytes(FIXTURE).expect("별지 서식을 열지 못했다")
}

/// layer SVG 는 글자마다 `<text>` 하나를 낸다 — 그 내용만 뽑는다.
fn glyphs(svg: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(open_at) = rest.find("<text") {
        rest = &rest[open_at..];
        let Some(gt) = rest.find('>') else { break };
        let body = &rest[gt + 1..];
        let Some(close_at) = body.find("</text>") else {
            break;
        };
        out.push(body[..close_at].to_string());
        rest = &body[close_at..];
    }
    out
}

fn glyph_count(svg: &str, needle: char) -> usize {
    glyphs(svg)
        .iter()
        .filter(|g| g.chars().eq(std::iter::once(needle)))
        .count()
}

#[test]
fn profileless_pdf_export_renders_with_the_print_profile() {
    let core = open();
    let pages = [0u32];
    let options = PdfExportOptions::default();

    let default_pdf = core
        .render_pages_pdf_native_with_options(&pages, &options)
        .expect("기본 PDF");
    let print_pdf = core
        .render_pages_pdf_native_with_profile_and_options(&pages, RenderProfile::Print, &options)
        .expect("Print PDF");
    let screen_pdf = core
        .render_pages_pdf_native_with_profile_and_options(&pages, RenderProfile::Screen, &options)
        .expect("Screen PDF");

    assert!(default_pdf.starts_with(b"%PDF-"), "PDF 헤더가 없다");
    assert_eq!(
        default_pdf,
        print_pdf,
        "프로필 없는 PDF 는 Print 산출과 같아야 한다 (기본 {} B · Print {} B)",
        default_pdf.len(),
        print_pdf.len()
    );
    assert_ne!(
        default_pdf, screen_pdf,
        "Screen 산출과 같다면 스위치가 여전히 꺼져 있다"
    );
}

#[test]
fn the_print_profile_drops_the_empty_field_guide_text() {
    let core = open();
    let screen = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Screen)
        .expect("Screen SVG");
    let print = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("Print SVG");

    for needle in GUIDE_ONLY_CHARS {
        let on_screen = glyph_count(&screen, needle);
        let on_print = glyph_count(&print, needle);
        assert!(
            on_screen > 0,
            "화면 프로필은 안내문을 그린다 — `{needle}` 이 없다면 재현체가 바뀐 것이다"
        );
        assert_eq!(
            on_print, 0,
            "인쇄 프로필에 안내문 글자 `{needle}` 이 {on_print}개 남았다"
        );
    }

    // 서식 본문은 그대로다 — 안내문만 빠진다.
    let printed = glyphs(&print).len();
    let shown = glyphs(&screen).len();
    assert!(
        printed >= 150 && printed < shown,
        "인쇄 글자 수 {printed} · 화면 글자 수 {shown}"
    );
}
