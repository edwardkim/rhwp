//! Issue #1187: `BookReview.hwp` 글상자 내용이 영역 밖으로 출력되는 회귀 가드.
//!
//! `samples/basic/BookReview.hwp` 1쪽의 큰 점선 글상자에는 뒤쪽 목차 문단의
//! `line_seg.vertical_pos` 가 글상자 내부 높이를 초과하는 데이터가 들어 있다.
//! 렌더러는 문단을 삭제하거나 재배치하지 않고, 글상자 콘텐츠를 글상자 내부 영역으로
//! clip 해야 한다.

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone)]
struct ClipRect {
    id: String,
    rect: Rect,
}

#[derive(Debug, Clone)]
struct SvgText {
    text: String,
    x: f64,
    y: f64,
}

fn render_bookreview_page1_svg() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/BookReview.hwp");
    let bytes = fs::read(&path).expect("read samples/basic/BookReview.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse BookReview.hwp");
    doc.render_page_svg_native(0).expect("render page 1 svg")
}

fn render_bookreview_page1_layer_json() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/BookReview.hwp");
    let bytes = fs::read(&path).expect("read samples/basic/BookReview.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse BookReview.hwp");
    doc.get_page_layer_tree_native(0)
        .expect("render page 1 layer tree")
}

fn attr_f64(tag: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

fn attr_string(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn textbox_clip_rects(svg: &str) -> Vec<ClipRect> {
    let mut rects = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<clipPath") {
        rest = &rest[start..];
        let Some(end) = rest.find("</clipPath>") else {
            break;
        };
        let clip = &rest[..end + "</clipPath>".len()];
        if let Some(id) = attr_string(clip, "id") {
            if !id.starts_with("textbox-clip-") {
                rest = &rest[end + "</clipPath>".len()..];
                continue;
            }
            if let Some(rect_start) = clip.find("<rect") {
                let rect_tail = &clip[rect_start..];
                if let Some(rect_end) = rect_tail.find('>') {
                    let tag = &rect_tail[..=rect_end];
                    if let (Some(x), Some(y), Some(width), Some(height)) = (
                        attr_f64(tag, "x"),
                        attr_f64(tag, "y"),
                        attr_f64(tag, "width"),
                        attr_f64(tag, "height"),
                    ) {
                        rects.push(ClipRect {
                            id,
                            rect: Rect {
                                x,
                                y,
                                width,
                                height,
                            },
                        });
                    }
                }
            }
        }
        rest = &rest[end + "</clipPath>".len()..];
    }
    rects
}

fn text_nodes_in_clip(svg: &str, clip_id: &str) -> Vec<SvgText> {
    let marker = format!("<g clip-path=\"url(#{clip_id})\">");
    let Some(group_start) = svg.find(&marker) else {
        return Vec::new();
    };
    let group_body = &svg[group_start + marker.len()..];
    let Some(group_end) = group_body.find("</g>") else {
        return Vec::new();
    };
    let mut rest = &group_body[..group_end];
    let mut nodes = Vec::new();

    while let Some(open) = rest.find("<text") {
        let after_open = &rest[open..];
        let Some(gt) = after_open.find('>') else {
            break;
        };
        let tag = &after_open[..=gt];
        let after_tag = &after_open[gt + 1..];
        let Some(close) = after_tag.find("</text>") else {
            break;
        };

        if let (Some(x), Some(y)) = (attr_f64(tag, "x"), attr_f64(tag, "y")) {
            nodes.push(SvgText {
                text: after_tag[..close].to_string(),
                x,
                y,
            });
        }
        rest = &after_tag[close + "</text>".len()..];
    }

    nodes
}

fn text_lines_in_clip(svg: &str, clip_id: &str) -> Vec<(f64, String)> {
    let mut rows = std::collections::BTreeMap::<i64, Vec<SvgText>>::new();
    for node in text_nodes_in_clip(svg, clip_id) {
        rows.entry((node.y * 100.0).round() as i64)
            .or_default()
            .push(node);
    }

    rows.into_iter()
        .map(|(y, mut nodes)| {
            nodes.sort_by(|a, b| a.x.total_cmp(&b.x));
            let text = nodes.into_iter().map(|n| n.text).collect::<String>();
            (y as f64 / 100.0, text)
        })
        .collect()
}

fn svg_text_sequence(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after_open = &rest[open..];
        if let Some(gt) = after_open.find('>') {
            let after_tag = &after_open[gt + 1..];
            if let Some(close) = after_tag.find("</text>") {
                out.push_str(&after_tag[..close]);
                rest = &after_tag[close + "</text>".len()..];
                continue;
            }
        }
        break;
    }
    out
}

fn compact_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '\u{00a0}')
        .collect()
}

#[test]
fn bookreview_textbox_content_is_clipped_without_losing_visible_toc_lines() {
    let svg = render_bookreview_page1_svg();

    let text = svg_text_sequence(&svg);
    let compact = compact_text(&text);
    assert!(
        compact.contains("강우신지음"),
        "우측 하단 저자 정보 글상자는 사라지면 안 됨. text sequence={text:?}"
    );
    assert!(
        compact.contains("원앤원북스"),
        "우측 하단 출판 정보 글상자는 사라지면 안 됨. text sequence={text:?}"
    );
    assert!(
        compact.contains("5장_중대형주택마련과자녀교육을위한40대의자산관리"),
        "정상 표시되어야 하는 5장 목차는 렌더 트리에 남아야 함. text sequence={text:?}"
    );
    assert!(
        compact.contains("6장_당당한인생2막을위한50대이후의자산관리"),
        "정상 표시되어야 하는 6장 목차는 렌더 트리에 남아야 함. text sequence={text:?}"
    );
    assert!(
        compact.contains("에필로그_월급쟁이가부자로당당하게은퇴하기위한10가지조언"),
        "정상 표시되어야 하는 에필로그 목차는 렌더 트리에 남아야 함. text sequence={text:?}"
    );

    let rects = textbox_clip_rects(&svg);
    assert!(
        !rects.is_empty(),
        "BookReview.hwp 글상자 콘텐츠에는 textbox clipPath 가 필요함"
    );

    let main_textbox_clip = rects.iter().find(|clip| {
        let r = &clip.rect;
        (45.0..=51.0).contains(&r.x)
            && (514.0..=520.0).contains(&r.y)
            && (680.0..=695.0).contains(&r.width)
            && (480.0..=495.0).contains(&r.height)
    });
    assert!(
        main_textbox_clip.is_some(),
        "큰 점선 글상자 내부 영역 clip 이 필요함. actual textbox clips={rects:?}"
    );

    let main_textbox_clip = main_textbox_clip.unwrap();
    let clip_bottom = main_textbox_clip.rect.y + main_textbox_clip.rect.height;
    let lines = text_lines_in_clip(&svg, &main_textbox_clip.id);
    for expected in [
        "5장_중대형주택마련과자녀교육을위한40대의자산관리",
        "6장_당당한인생2막을위한50대이후의자산관리",
        "에필로그_월급쟁이가부자로당당하게은퇴하기위한10가지조언",
    ] {
        let Some((line_y, line_text)) = lines
            .iter()
            .find(|(_, line)| compact_text(line).contains(expected))
        else {
            panic!("큰 글상자 안에서 목차 줄을 찾을 수 없음: {expected}. actual lines={lines:?}");
        };
        assert!(
            *line_y <= clip_bottom,
            "정상 표시되어야 하는 목차 줄이 clip 밖으로 밀리면 안 됨. expected={expected:?}, y={line_y}, clip_bottom={clip_bottom}, line={line_text:?}"
        );
    }
}

#[test]
fn bookreview_textbox_content_has_paint_layer_clip() {
    let json = render_bookreview_page1_layer_json();
    let textbox_clip_count = json.matches("\"clipKind\":\"textBox\"").count();

    assert!(
        textbox_clip_count >= 3,
        "BookReview.hwp 글상자 콘텐츠에는 paint layer textBox ClipRect 가 필요함. count={textbox_clip_count}"
    );
    assert!(
        json.contains("\"groupKind\":{\"kind\":\"textBox\"}"),
        "TextBox ClipRect child 는 TextBox groupKind 를 유지해야 함"
    );
}
