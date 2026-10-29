//! [#8282] HWPX 로 저장한 뒤 다시 열어도 글자처럼 그림 + 어울림 부동 도형 문단의 배치가 같다.
//!
//! 결함: HWPX 저장은 편집한 문단의 `linesegarray` 를 뺀다. rhwp 가 다시 열면 그 문단을 저장 줄 없이
//! 현재 프레임에서 재조판하는데, 그림 밴드 프레임은 어울림 **그림** 하나만 받고 어울림 **도형**은
//! 받지 않았다. 그래서 합성 줄(글자 높이)만 남아 글자처럼 그림의 높이가 줄에서 빠지고, 뒤 문단이
//! 그만큼 위로 올라왔다(#8271 의 줄 폭 중립 배치와 같은 경로).
//!
//! 독립 기준: 같은 문서의 저장 전 배치와 HWP 저장본(저장 줄 보존)을 다시 연 배치가 서로 같다.
//! 저장·재열기는 배치를 바꾸지 않아야 한다. 어울림 도형은 어울림 그림과 같은 부동 개체 하나로
//! 같은 프레임이 받는다(대조군: 어울림 그림은 종전에도 같았다).
//!
//! 이 검사는 저장·재열기의 일관성만 고정한다. 어울림 개체가 그림 줄과 겹칠 때 줄 폭을 얼마나
//! 줄이는지는 한컴 Print 와 Visual Sweep 으로 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn find(node: &RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<BoundingBox> {
    if pred(node) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

/// 본문 줄(글상자·표 안 제외)을 위에서부터 (상자, 글자) 로 모은다.
fn body_lines(node: &RenderNode, out: &mut Vec<(BoundingBox, String)>) {
    if matches!(
        node.node_type,
        RenderNodeType::TextBox | RenderNodeType::Table(_)
    ) {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox, text(node)));
        return;
    }
    for child in &node.children {
        body_lines(child, out);
    }
}

fn json_usize(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect(key)
}

const SHORT: &str = "앞 글 뒤 글";
const LONG: &str = "앞 글 뒤에 이어지는 이 문단은 어울림 글상자 옆으로 여러 줄에 걸쳐 흐르도록 길게 쓴 문장입니다. 글상자가 줄과 겹치는 세로 범위에서는 줄 폭이 줄어듭니다. 끝.";

fn edited(body: &str, horz: u32, vert: u32, picture_float: bool) -> DocumentCore {
    let jpg = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/images/moogung.jpg"),
    )
    .expect("jpg");
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, body).expect("text");
    let end = body.chars().count();
    core.split_paragraph_native(0, 0, end, None).expect("split");
    core.insert_text_native(0, 1, 0, "다음 문단")
        .expect("text 2");
    let pic = core
        .insert_picture_native(
            0,
            0,
            2,
            &[],
            &jpg,
            8504,
            4447,
            1000,
            523,
            "jpg",
            "그림",
            None,
            None,
        )
        .expect("picture");
    core.set_picture_properties_native(
        0,
        json_usize(&pic, "paraIdx"),
        json_usize(&pic, "controlIdx"),
        r#"{"treatAsChar":true}"#,
    )
    .expect("tac");
    if picture_float {
        let f = core
            .insert_picture_native(
                0,
                0,
                end + 1,
                &[],
                &jpg,
                17008,
                5669,
                1000,
                523,
                "jpg",
                "부동",
                None,
                None,
            )
            .expect("float picture");
        core.set_picture_properties_native(
            0,
            json_usize(&f, "paraIdx"),
            json_usize(&f, "controlIdx"),
            &format!(r#"{{"treatAsChar":false,"textWrap":"Square","vertRelTo":"Paper","horzRelTo":"Paper","vertOffset":{vert},"horzOffset":{horz}}}"#),
        )
        .expect("square picture");
    } else {
        core.create_shape_control_native(
            0,
            0,
            end + 1,
            17008,
            5669,
            horz,
            vert,
            false,
            "Square",
            "textbox",
            false,
            false,
            &[],
        )
        .expect("square textbox");
    }
    core
}

struct Placement {
    picture: BoundingBox,
    lines: Vec<(BoundingBox, String)>,
}

fn placement(core: &DocumentCore) -> Placement {
    let tree = core.build_page_render_tree(0).expect("page 1");
    let mut lines = Vec::new();
    body_lines(&tree.root, &mut lines);
    Placement {
        picture: find(&tree.root, &|n| {
            matches!(n.node_type, RenderNodeType::Image(_)) && n.bbox.width < 120.0
        })
        .expect("글자처럼 그림"),
        lines,
    }
}

fn reopen_hwpx(core: &DocumentCore) -> DocumentCore {
    let reopened =
        DocumentCore::from_bytes(&core.export_hwpx_native().expect("hwpx")).expect("reopen");
    assert!(
        reopened.document().sections[0].paragraphs[0]
            .line_segs
            .is_empty(),
        "전제: HWPX 저장본의 host 문단은 저장 줄 없이 재조판된다"
    );
    reopened
}

fn reopen_hwp(core: &DocumentCore) -> DocumentCore {
    let reopened =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("hwp reopen");
    assert!(
        !reopened.document().sections[0].paragraphs[0]
            .line_segs
            .is_empty(),
        "HWP 저장본은 host 줄을 보존한다"
    );
    reopened
}

const PICTURE_HEIGHT_PX: f64 = 4447.0 * 96.0 / 7200.0;

/// 그림 위치, 본문 줄의 수·글자·세로 위치·높이가 같고, 그림 host 줄이 그림을 담는다.
fn assert_same_placement(label: &str, expected: &Placement, actual: &Placement) {
    let host = &actual.lines.first().expect("host 줄").0;
    assert!(
        host.height + 0.5 >= PICTURE_HEIGHT_PX,
        "{label}: host 줄 높이({:.1})가 그림 높이({PICTURE_HEIGHT_PX:.1}) 이상",
        host.height
    );
    assert!(
        actual.picture.y + 0.5 >= host.y
            && actual.picture.y + actual.picture.height <= host.y + host.height + 0.5,
        "{label}: 그림({:.1}–{:.1})이 host 줄({:.1}–{:.1}) 안에 있다",
        actual.picture.y,
        actual.picture.y + actual.picture.height,
        host.y,
        host.y + host.height
    );
    assert!(
        (expected.picture.y - actual.picture.y).abs() <= 0.5,
        "{label}: 그림 위 — 기준 {:.1}, 다시 열기 {:.1}",
        expected.picture.y,
        actual.picture.y
    );
    let texts = |p: &Placement| p.lines.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>();
    assert_eq!(texts(expected), texts(actual), "{label}: 줄마다 같은 글자");
    for ((a, text), (b, _)) in expected.lines.iter().zip(&actual.lines) {
        assert!(
            (a.y - b.y).abs() <= 0.5 && (a.height - b.height).abs() <= 0.5,
            "{label}: 「{text}」 줄 — 기준 y={:.1} h={:.1}, 다시 열기 y={:.1} h={:.1}",
            a.y,
            a.height,
            b.y,
            b.height
        );
    }
}

/// 이슈 재현: 어울림 글상자가 host 줄 아래(종이 기준 세로 15000HU)에 있어 줄 폭과 겹치지 않는다.
#[test]
fn square_textbox_below_keeps_tac_picture_line_after_hwpx_reopen() {
    let mut blank = DocumentCore::new_empty();
    blank.create_blank_document_native().expect("blank");
    let margin_left = blank.document().sections[0]
        .section_def
        .page_def
        .margin_left;
    let core = edited(SHORT, margin_left, 15000, false);
    let before = placement(&core);
    assert_same_placement(
        "어울림 글상자(줄 아래)",
        &before,
        &placement(&reopen_hwpx(&core)),
    );
    assert_same_placement("HWP 저장본", &before, &placement(&reopen_hwp(&core)));
}

/// 어울림 글상자가 그림 줄과 둘째 줄의 오른쪽을 덮는다. 저장 줄 없이 다시 연 HWPX 와 저장 줄을
/// 보존한 HWP 가 같은 줄 구성을 낸다.
#[test]
fn square_textbox_overlapping_lines_reopens_like_hwp() {
    let core = edited(LONG, 34016, 9000, false);
    assert_same_placement(
        "어울림 글상자(줄과 겹침)",
        &placement(&reopen_hwp(&core)),
        &placement(&reopen_hwpx(&core)),
    );
}

/// 대조군: 어울림 그림은 종전에도 그림 밴드 프레임이 받았다.
#[test]
fn square_picture_host_keeps_its_lines_after_hwpx_reopen() {
    for (label, body, horz, vert) in [
        ("어울림 그림(줄 아래)", SHORT, 8504u32, 15000u32),
        ("어울림 그림(줄과 겹침)", LONG, 34016, 9000),
    ] {
        let core = edited(body, horz, vert, true);
        assert_same_placement(
            label,
            &placement(&reopen_hwp(&core)),
            &placement(&reopen_hwpx(&core)),
        );
    }
}

/// 커밋한 합성 HWPX(`make_inputs.mjs`)의 host 는 저장 줄이 없고, 같은 편집을 메모리에서 저장·재열기한
/// 배치와 같다. 한컴 Print 대조는 이 fixture 로 한다.
#[test]
fn committed_fixtures_match_the_in_memory_reopen() {
    let mut blank = DocumentCore::new_empty();
    blank.create_blank_document_native().expect("blank");
    let margin_left = blank.document().sections[0]
        .section_def
        .page_def
        .margin_left;
    for (path, body, horz, vert, picture_float) in [
        (
            "samples/issue8282/tac_picture_square_textbox.hwpx",
            SHORT,
            margin_left,
            15000,
            false,
        ),
        (
            "samples/issue8282/tac_picture_square_textbox_overlap.hwpx",
            LONG,
            34016,
            9000,
            false,
        ),
        (
            "samples/issue8282/tac_picture_square_picture_overlap.hwpx",
            LONG,
            34016,
            9000,
            true,
        ),
    ] {
        let fixture = DocumentCore::from_bytes(
            &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("fixture"),
        )
        .expect("parse fixture");
        assert!(
            fixture.document().sections[0].paragraphs[0]
                .line_segs
                .is_empty(),
            "{path}: host 문단은 저장 줄이 없다"
        );
        assert_same_placement(
            path,
            &placement(&reopen_hwpx(&edited(body, horz, vert, picture_float))),
            &placement(&fixture),
        );
    }
}
