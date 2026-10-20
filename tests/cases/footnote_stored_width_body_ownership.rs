//! 새 각주의 저장 줄 폭이 실제 각주 영역보다 넓으면 그 캐시로 본문 예산을 고정하지 않는다.
//!
//! 같은 편집 저장본의 한컴2020 Print는 location.replace 사용 예를 첫 쪽에,
//! navigator 제목과 마지막 screen 속성들을 둘째 쪽에 둔다. Native/fresh 전체 두 쪽은
//! 95.47353%/92.07644%이고, 해당 본문·다음 쪽 소속을 같은 배율로 직접 판독했다.
//! 현재 위치의 px 핀 대신 실제 인쇄 경로의 쪽 소속·내용 보존을 검사한다.
#![cfg(not(target_arch = "wasm32"))]

mod output {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/wrapper_note_render_output.rs"
    ));
}

const INPUT: &str = "mydocs/pr/assets/semanticist21-20261008/current-print-inputs-v175/additional-edited-note-style-missing-named-style-v175.hwpx";

#[test]
fn invalid_footnote_width_does_not_push_the_body_and_final_content_to_extra_pages() {
    let pages = output::pages(INPUT);
    assert_eq!(pages.len(), 2, "동일 편집 저장본의 독립 Print 전체 쪽수");
    let text: Vec<_> = pages
        .iter()
        .map(|svg| output::text(&roxmltree::Document::parse(svg).expect("실제 인쇄 SVG")))
        .collect();
    let usage = "location.replace(\"URL\")";
    assert_eq!(text[0].matches(usage).count(), 1, "첫 쪽의 본문 끝 사용 예");
    assert_eq!(
        text[1].matches(usage).count(),
        0,
        "첫 쪽 본문이 뒤 쪽에 중복되지 않는다"
    );
    assert!(
        !text[0].contains("navigator"),
        "다음 쪽 본문을 앞 쪽으로 당기지 않는다"
    );
    assert!(text[1].contains("navigator객체.브라우저에관한정보"));
    let final_properties = [
        "width스크린의가로",
        "height스크린의세로",
        "colorDepth칼라비트수",
        "availHeight출력할수있는세로크기",
        "availWidth출력할수있는가로크기",
    ];
    let mut previous = 0;
    for property in final_properties {
        assert_eq!(
            text[1].matches(property).count(),
            1,
            "마지막 본문의 누락/중복: {property}"
        );
        let start = text[1].find(property).unwrap();
        assert!(start >= previous, "마지막 표의 속성 순서가 보존된다");
        previous = start + property.len();
    }
    assert!(text[0].contains("1)"), "새 각주의 참조를 보존한다");
}
