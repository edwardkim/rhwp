//! 검사 유형: 관계형. 명시한 글리프 공급자와 OS 별칭의 선택 순서를 검사한다.
//! 주석 갱신: 2026-11-12T15:02:00+09:00.
//! 공개 합성 style 계약이므로 독립 PDF·raw 일치율·절대 px 고정은 비해당이다.
//! 보호 문서의 공급 CFF에 없는 ○를 Mac 설치 TTF 별칭이 선점한 소비 경로의 반례다.
//! 글꼴 설치 여부와 무관하게 실제 Canvas 측정·paint가 공유하는 descriptor를 검사한다.

use rhwp::renderer::canvas_text_font::CanvasTextFont;
use rhwp::renderer::TextStyle;

fn descriptor(explicit: bool, source_ttf: bool) -> String {
    CanvasTextFont::for_positioned_text(
        &TextStyle {
            font_family: "한양신명조,Myeongjo,Batang".into(),
            glyph_fallback_advance_explicit: explicit,
            source_font_is_ttf: source_ttf,
            ..Default::default()
        },
        0.0,
    )
    .descriptor()
    .to_owned()
}

#[test]
fn explicit_glyph_supplier_precedes_unselected_os_aliases() {
    let font = descriptor(true, false);
    let source = font.find("\"한양신명조\"").unwrap();
    let supplier = font.find("\"Myeongjo\"").unwrap();
    assert!(source < supplier, "주 face의 실제 글리프는 먼저 유지한다");
    if let Some(alias) = font.find("\"HYSinMyeongJo-Medium\"") {
        assert!(
            supplier < alias,
            "명시 공급자보다 OS 별칭이 먼저 나오면 안 된다: {font}"
        );
    }
}

#[test]
fn ordinary_selection_preserves_installed_alias_priority() {
    let font = descriptor(false, false);
    assert!(
        font.find("\"HYSinMyeongJo-Medium\"").unwrap() < font.find("\"Myeongjo\"").unwrap(),
        "명시 환경이 없는 일반 글꼴 선택은 보존한다"
    );
}

#[test]
fn explicit_supplier_does_not_replace_the_verified_primary_ttf() {
    let font = descriptor(true, true);
    assert!(
        font.find("'__rhwp_ttf__한양신명조'").unwrap() < font.find("\"한양신명조\"").unwrap(),
        "확인된 원 TTF 프로그램은 계속 먼저 소비한다"
    );
}
