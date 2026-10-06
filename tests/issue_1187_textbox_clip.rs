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

// ── Issue #6974: 비인라인 글상자의 자식 도형 세로 확장 ──────────────────────
//
// 비인라인 글상자 안의 자식 도형·그림이 글상자 선언 높이를 넘으면, 한글은
// 글상자를 내용 높이에 맞춰 세로로 늘린다. rhwp 는 선언 높이 그대로 두어
// 하단 장식이 잘리거나 글상자 아래 내용이 위로 붙어 겹쳤다.
//
// 수정 전에는 확장이 없어 특정 글상자 높이가 선언값에 머문다. 수정 후에는
// 자식 도형 하단 + 아래 여백까지 높이가 늘어난다. 이 픽스처의 한 글상자는
// 수정으로 약 +30px 커진다(자식 장식이 선언 높이를 넘는 형상).

const VGROW_SAMPLE: &str = "samples/issue6974/synth_textbox_vgrow.hwp";

#[test]
fn issue_6974_noninline_textbox_expands_to_child_object_height() {
    use rhwp::document_core::DocumentCore;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(VGROW_SAMPLE);
    let core =
        DocumentCore::from_bytes(&fs::read(&path).expect("read fixture")).expect("parse fixture");
    let tree = core.build_page_render_tree(0).expect("render tree");

    // 각 글상자에서 "자식 도형 하단"과 "글상자 하단"을 모은다. 확장이 일어나면
    // 글상자 하단이 자식 도형 하단 이상으로 내려온다. 확장이 없으면 자식 도형이
    // 글상자 하단보다 아래에 남는다(선언 높이 고정).
    fn object_bottom(n: &RenderNode) -> f64 {
        let own = match &n.node_type {
            RenderNodeType::Image(_)
            | RenderNodeType::Path(_)
            | RenderNodeType::Rectangle(_)
            | RenderNodeType::Ellipse(_) => n.bbox.y + n.bbox.height,
            _ => f64::NEG_INFINITY,
        };
        n.children.iter().fold(own, |m, c| m.max(object_bottom(c)))
    }
    fn walk(n: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
        if matches!(n.node_type, RenderNodeType::TextBox) {
            let cb = object_bottom(n);
            if cb.is_finite() {
                out.push((n.bbox.x, n.bbox.height, n.bbox.y + n.bbox.height, cb));
            }
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let mut boxes = Vec::new();
    walk(&tree.root, &mut boxes);
    assert!(
        !boxes.is_empty(),
        "픽스처에 자식 도형을 가진 글상자가 있어야 한다(드리프트 감지)"
    );

    // 이 픽스처의 한 글상자(문서 우하단, x≈562·바닥부 y≈633)는 자식 장식이
    // 선언 높이(약 205px)를 넘어, 수정 후 약 236px 로 확장된다. 확장이 없으면
    // 이 글상자가 선언 높이에 머물러 하단 장식이 잘리고 아래 표가 위로 붙는다.
    // 픽스처 기하는 make_synth_repro 익명화로 보존되므로 위치로 특정한다.
    // 문서 우측 열(x≈562)의 아래쪽 글상자는 자식 장식이 선언 높이(~205px)를
    // 넘어, 수정 후 ~236px 로 확장된다. 확장이 없으면 이 열의 어떤 글상자도
    // 220px 를 넘지 못한다(픽스처 기하는 make_synth_repro 익명화로 보존).
    let right_col_max_h = boxes
        .iter()
        .filter(|(x, _, _, _)| (*x - 562.5).abs() < 4.0)
        .map(|(_, h, _, _)| *h)
        .fold(0.0_f64, f64::max);
    assert!(
        right_col_max_h > 220.0,
        "우측 열 글상자가 자식 도형 높이로 확장되어야 한다(수정 후 ~236px, 수정 전 ~205px) — \
         실측 최대 {right_col_max_h:.1}px: {:?}",
        boxes
            .iter()
            .map(|(x, h, _, _)| (*x, *h))
            .collect::<Vec<_>>()
    );

    // 확장된 글상자에서 자식 도형 하단이 글상자 하단 안(±1px)에 들어온다.
    for (_, _, box_bottom, child_bottom) in &boxes {
        assert!(
            *child_bottom <= *box_bottom + 1.0,
            "글상자 자식 도형(하단 {child_bottom:.1})이 글상자 하단({box_bottom:.1})을 \
             넘었다"
        );
    }
}
