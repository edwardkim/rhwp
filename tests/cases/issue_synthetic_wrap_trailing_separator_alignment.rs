//! 합성 줄의 말미 구분 공백은 보존하되 마지막 가시 글자가 줄 오른쪽 끝에 맞는다.
//!
//! 독립 한/글 Print `pdf/issue7816/base_charpr0_20pt-2020.pdf`의 첫 긴 문단은
//! 같은 세 줄이고, 앞 두 줄의 마지막 한글 글리프가 본문 오른쪽 끝에 맞는다.
//! 바탕 글리프의 advance는 1em이다(독립 PDF subset/실제 Batang 윤곽·hmtx 대조).
//! source ab5112의 Native/fresh WASM 전쪽 최저 100%, 대표 review 확대 판독을
//! `pr_7825_review_impl.md`에 연결한다. 절대 원점이나 픽셀 영역을 고정하지 않는다.

use rhwp::document_core::DocumentCore;

const INPUT: &[u8] = include_bytes!("../../samples/issue7816/base_charpr0_20pt.hwpx");

fn owned_lines(node: &serde_json::Value, lines: &mut Vec<serde_json::Value>) {
    if node["type"] == "TextLine" && node["pi"] == 2 {
        lines.push(node.clone());
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            owned_lines(child, lines);
        }
    }
}

#[test]
fn wrapped_visible_glyphs_align_and_terminal_separator_remains_owned() {
    let core = DocumentCore::from_bytes(INPUT).expect("문서 로드");
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 한 쪽");
    let tree = core.build_page_render_tree(0).expect("render tree");
    let root: serde_json::Value = serde_json::from_str(&tree.root.to_json()).expect("tree JSON");
    let mut lines = Vec::new();
    owned_lines(&root, &mut lines);
    assert_eq!(lines.len(), 3, "독립 Print의 첫 긴 문단 세 줄");

    let svg = core.render_page_svg_native(0).expect("SVG");
    let svg = roxmltree::Document::parse(&svg).expect("SVG XML");
    for line in lines.iter().take(2) {
        let text: String = line["children"]
            .as_array()
            .expect("줄의 run")
            .iter()
            .filter_map(|run| run["text"].as_str())
            .collect();
        assert!(text.ends_with(' '), "말미 구분 공백의 줄 소유 보존");
        let last_char = text.trim_end().chars().last().expect("가시 글자");
        assert!(('\u{AC00}'..='\u{D7A3}').contains(&last_char));

        let top = line["bbox"]["y"].as_f64().expect("줄 위");
        let bottom = top + line["bbox"]["h"].as_f64().expect("줄 높이");
        // 소유 줄 상자 내부의 baseline만 선택한다. 고정 페이지 좌표가 아니다.
        let last = svg
            .descendants()
            .rfind(|n| {
                n.has_tag_name("text")
                    && n.attribute("y")
                        .and_then(|s| s.parse::<f64>().ok())
                        .is_some_and(|y| y >= top && y <= bottom)
                    && n.text().is_some_and(|s| !s.trim().is_empty())
            })
            .expect("소유 줄의 마지막 가시 glyph");
        assert_eq!(last.text(), Some(last_char.to_string().as_str()));
        let x: f64 = last.attribute("x").expect("glyph x").parse().unwrap();
        let em: f64 = last
            .attribute("font-size")
            .expect("glyph em")
            .parse()
            .unwrap();
        let right = line["bbox"]["x"].as_f64().unwrap() + line["bbox"]["w"].as_f64().unwrap();
        assert!(
            (x + em - right).abs() < em / 32.0,
            "가시 glyph 끝과 소유 줄 끝의 정렬 관계: x={x}, em={em}, right={right}, text={text:?}"
        );
    }
}
