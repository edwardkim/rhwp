//! 한컴 Mac 12.30이 만든 덧말 문서를 그린다.
//!
//! `samples/dutmal/hancom-mac-dutmal.*`는 한컴이 만든 같은 문서의 HWP·HWPX와 HWP의 Print PDF다.
//! 본말(漢字·한·대한민국만세·글자·스타일)과 덧말이 모두 쪽에 그려져야 한다. 종전에는 둘 다 빠졌다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn page_text(path: &str) -> String {
    let path = format!("{}/{path}", env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

#[test]
fn hancom_ruby_main_and_ruby_text_are_painted() {
    for ext in ["hwp", "hwpx"] {
        let text = page_text(&format!("samples/dutmal/hancom-mac-dutmal.{ext}"));
        for word in [
            "漢字",
            "한자",
            "한국어사전",
            "대한민국만세",
            "대한",
            "글자",
            "덧",
            "스타일",
            "차례제목",
        ] {
            // 글자마다 따로 그리므로 글자 차례로 찾는다.
            let mut rest = text.as_str();
            for ch in word.chars() {
                let at = rest
                    .find(ch)
                    .unwrap_or_else(|| panic!("{ext}: {word}의 {ch} 없음"));
                rest = &rest[at + ch.len_utf8()..];
            }
        }
    }
}

#[test]
fn opengov_ruby_main_text_is_painted() {
    let text = page_text("samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx");
    // 본말 "팀단위 훈련"의 '위'와 덧말 "전술훈련 30% + 현지훈련 20%"의 '술'·'지'는 이 쪽의 다른 글에 없다.
    for ch in ['위', '술', '지'] {
        assert!(text.contains(ch), "{ch} 없음");
    }
}
