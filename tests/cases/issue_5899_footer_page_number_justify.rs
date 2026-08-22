//! Issue #5899: 양쪽정렬 꼬리말의 쪽번호가 종이 밖으로 밀려 사라지던 회귀 가드.
//!
//! `samples/hwp3-sample11-hwpx.hwpx` 의 꼬리말은 `DCT Technology Inc.` + 공백 74개 +
//! `<hp:autoNum numType="PAGE">` 다. #3216 규약대로 필드 치환은 `run.text`(공백
//! placeholder)를 보존하고 `display_text` 만 쪽번호로 바꾼다. 그런데 양쪽정렬 슬랙
//! 분배가 공백을 **모델 텍스트**로 세는 바람에 줄이 "후행 공백 75개로 끝난다"고
//! 판정되어 슬랙이 내부 공백 2개에만 나뉘었고, 그 여분이 렌더 단계에서 **표시
//! 텍스트의 공백 76개 전부**에 붙어 쪽번호가 x≈20,163px(종이 폭 793.7px)로 밀려났다.
//!
//! 한글 2020 정본 `pdf/hwp3-sample11-2020.pdf` p116 은 `115` 를 x=523.00pt
//! (=697.3px, 본문 오른쪽 끝 538.92pt=718.6px 바로 안쪽)에 그린다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwp3-sample11-hwpx.hwpx";

/// 꼬리말 쪽번호가 그려지는 쪽들(0-based). 1쪽(표지)에는 꼬리말이 없다.
const PAGES: &[(u32, &str)] = &[(1, "1"), (115, "115"), (150, "150")];

fn render(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    doc.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("render page {page}: {e}"))
}

fn paper_width(svg: &str) -> f64 {
    let head = &svg[..svg.find('>').expect("svg 루트 태그")];
    let at = head.find("width=\"").expect("svg width 속성") + 7;
    let rest = &head[at..];
    let end = rest.find('"').expect("width 값 종료");
    rest[..end].parse().expect("width 수치")
}

/// `<text x=".." y="..">내용</text>` 을 (x, y, 내용) 으로 모은다.
fn text_nodes(svg: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start + 6..];
        let Some(gt) = rest.find('>') else { break };
        let attrs = &rest[..gt];
        let body_rest = &rest[gt + 1..];
        let Some(close) = body_rest.find("</text>") else {
            break;
        };
        let body = &body_rest[..close];
        rest = &body_rest[close + 7..];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let at = attrs.find(&key)? + key.len();
            let tail = &attrs[at..];
            let end = tail.find('"')?;
            tail[..end].parse().ok()
        };
        let (Some(x), Some(y)) = (attr("x"), attr("y")) else {
            continue;
        };
        out.push((x, y, body.to_string()));
    }
    out
}

/// 어떤 글자도 종이 밖(x > 종이 폭)에 놓이면 안 된다.
#[test]
fn issue_5899_no_glyph_is_drawn_outside_the_paper() {
    for &(page, _) in PAGES {
        let svg = render(page);
        let paper = paper_width(&svg);
        let worst = text_nodes(&svg)
            .into_iter()
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .expect("쪽에 글자가 있어야 한다");
        assert!(
            worst.0 <= paper,
            "p{}: 글자 {:?} 가 x={:.1} 로 종이 폭 {:.1} 밖에 그려졌다",
            page + 1,
            worst.2,
            worst.0,
            paper
        );
    }
}

/// 꼬리말 쪽번호는 꼬리말 마지막 줄 오른쪽 끝(본문 오른쪽 여백 안쪽)에 있어야 한다.
///
/// 한컴 2020 정본 p116: `115` 가 523.00~539.50pt = 697.3~719.3px, 본문 오른쪽 끝은
/// 538.92pt = 718.6px. 왼쪽으로 붙여 버리는 회귀(슬랙 전량 제거)도 함께 막는다.
#[test]
fn issue_5899_footer_page_number_sits_at_the_right_margin() {
    for &(page, expected) in PAGES {
        let svg = render(page);
        let paper = paper_width(&svg);
        let nodes = text_nodes(&svg);
        let footer_y = nodes
            .iter()
            .map(|(_, y, _)| *y)
            .fold(f64::MIN, |acc, y| acc.max(y));
        let mut footer: Vec<_> = nodes
            .iter()
            .filter(|(_, y, _)| (*y - footer_y).abs() < 0.5)
            .cloned()
            .collect();
        footer.sort_by(|a, b| a.0.total_cmp(&b.0));
        let drawn: String = footer.iter().map(|(_, _, t)| t.as_str()).collect();
        assert!(
            drawn.ends_with(expected),
            "p{}: 꼬리말 마지막 줄이 쪽번호 {expected} 로 끝나야 한다 (실제 {drawn:?})",
            page + 1
        );
        let first_digit_x = footer[footer.len() - expected.chars().count()].0;
        assert!(
            first_digit_x > paper * 0.8 && first_digit_x < paper,
            "p{}: 쪽번호 {expected} 가 오른쪽 여백(0.8×{:.1} ~ {:.1}px) 안에 와야 한다 (실제 x={:.1})",
            page + 1,
            paper,
            paper,
            first_digit_x
        );
    }
}
