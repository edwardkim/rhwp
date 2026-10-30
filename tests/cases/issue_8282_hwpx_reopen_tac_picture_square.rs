//! [#8282] HWPX 로 저장한 뒤 다시 열어도 글자처럼 그림 + 어울림 부동 도형 문단의 배치가 같고,
//! 어울림 개체가 줄 폭을 한/글처럼 줄인다.
//!
//! 결함: HWPX 저장은 편집한 문단의 `linesegarray` 를 뺀다. rhwp 가 다시 열면 그 문단을 저장 줄 없이
//! 현재 프레임에서 재조판하는데, 그림 밴드 프레임은 어울림 **그림** 하나만 받고 어울림 **도형**은
//! 받지 않았다. 그래서 합성 줄(글자 높이)만 남아 글자처럼 그림의 높이가 줄에서 빠지고, 뒤 문단이
//! 그만큼 위로 올라왔다(#8271 의 줄 폭 중립 배치와 같은 경로). 또 저장 줄 없는 문단의 어울림
//! 개체는 자기 문단과 뒤 문단의 줄을 줄이지 않았다.
//!
//! 독립 기준:
//! - 같은 문서의 저장 전 배치와 HWP 저장본을 다시 연 배치가 HWPX 재열기와 같다.
//! - 커밋한 합성 HWPX 의 한/글 2020 Print PDF(`pdf/issue8282/*-2020.pdf`)에서 읽은 줄 나눔과
//!   개체 회피 관계: 어울림 개체와 겹치는 줄은 개체 바깥 여백 상자 왼쪽에서 끝나고, 개체 아래
//!   줄은 전폭이며, 개체 옆의 뒤 문단은 개체 오른쪽에서 시작한다. 글상자 윗변은 오프셋에 위
//!   바깥여백을 더한 자리다.

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

fn find_all(node: &RenderNode, pred: &dyn Fn(&RenderNode) -> bool, out: &mut Vec<BoundingBox>) {
    if pred(node) {
        out.push(node.bbox);
    }
    for child in &node.children {
        find_all(child, pred, out);
    }
}

const HU_PX: f64 = 96.0 / 7200.0;
/// 글상자 바깥여백(283 HU)·크기(17008×5669 HU).
const BOX_MARGIN_PX: f64 = 283.0 * HU_PX;
const BOX_WIDTH_PX: f64 = 17008.0 * HU_PX;
const BOX_HEIGHT_PX: f64 = 5669.0 * HU_PX;

fn fixture(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("fixture"),
    )
    .expect("parse fixture")
}

/// 글상자 잉크(사각형). 그림 fixture 에는 없다.
fn textbox(core: &DocumentCore) -> BoundingBox {
    let tree = core.build_page_render_tree(0).expect("page 1");
    let mut boxes = Vec::new();
    find_all(
        &tree.root,
        &|n| matches!(n.node_type, RenderNodeType::Rectangle(_)) && n.bbox.width > 200.0,
        &mut boxes,
    );
    assert_eq!(boxes.len(), 1, "글상자 하나");
    boxes[0]
}

/// 한/글 2020 Print: 줄과 겹치는 어울림 개체 왼쪽에서 앞 두 줄이 끝나고(「…어울」「…쓴 문」),
/// 개체 아래 셋째 줄은 전폭이다. 같은 관계를 편집 직후·HWP·HWPX 가 모두 낸다.
#[test]
fn square_textbox_carves_its_own_host_lines_like_hancom_print() {
    let path = "samples/issue8282/tac_picture_square_textbox_overlap.hwpx";
    let core = fixture(path);
    let ink = textbox(&core);
    // 용지 기준 오프셋(34016, 9000 HU) + 바깥여백.
    assert!(
        (ink.x - (34016.0 * HU_PX + BOX_MARGIN_PX)).abs() <= 0.5
            && (ink.y - (9000.0 * HU_PX + BOX_MARGIN_PX)).abs() <= 0.5,
        "{path}: 글상자 잉크({:.1},{:.1})는 오프셋+바깥여백에 놓인다",
        ink.x,
        ink.y
    );
    let outer_left = ink.x - BOX_MARGIN_PX;
    let outer_bottom = ink.y + BOX_HEIGHT_PX + BOX_MARGIN_PX;
    let lines = placement(&core).lines;
    let texts: Vec<_> = lines
        .iter()
        .map(|(_, t)| t.trim_end().to_string())
        .collect();
    assert_eq!(texts.len(), 4, "{path}: host 세 줄 + 다음 문단 — {texts:?}");
    assert!(
        texts[0].ends_with("어울") && texts[1].ends_with("쓴 문") && texts[2].ends_with("끝."),
        "{path}: 한/글 Print 와 같은 줄 나눔 — {texts:?}"
    );
    for (bbox, text) in &lines[..2] {
        assert!(
            bbox.y < outer_bottom && bbox.x + bbox.width <= outer_left + 0.5,
            "{path}: 「{text}」 줄({:.1}–{:.1})은 글상자 바깥 왼쪽({outer_left:.1})에서 끝난다",
            bbox.x,
            bbox.x + bbox.width
        );
    }
    assert!(
        lines[2].0.y >= outer_bottom - 0.5 && lines[2].0.x + lines[2].0.width > outer_left,
        "{path}: 글상자 아래 셋째 줄은 전폭이다"
    );
    let edited = edited(LONG, 34016, 9000, false);
    assert_same_placement("편집 직후", &placement(&core), &placement(&edited));
    assert_same_placement(
        "HWP 저장본",
        &placement(&core),
        &placement(&reopen_hwp(&edited)),
    );
}

/// 한/글 2020 Print: host 줄 아래 어울림 글상자 옆에서 뒤 문단이 글상자 오른쪽으로 비켜 선다.
/// 뒤 문단의 저장 줄은 저장 줄 없는 host 의 개체를 증언하지 않는다(전폭 저장 줄).
#[test]
fn square_textbox_of_uncached_host_moves_next_paragraph_beside_it() {
    let path = "samples/issue8282/tac_picture_square_textbox.hwpx";
    let core = fixture(path);
    assert!(
        !core.document().sections[0].paragraphs[1]
            .line_segs
            .is_empty(),
        "전제: 뒤 문단은 저장 줄이 있다"
    );
    let ink = textbox(&core);
    let outer_right = ink.x + BOX_WIDTH_PX + BOX_MARGIN_PX;
    let lines = placement(&core).lines;
    let (next, text) = lines.last().expect("다음 문단");
    assert_eq!(text.trim_end(), "다음 문단");
    assert!(
        next.y < ink.y + BOX_HEIGHT_PX && next.y + next.height > ink.y - BOX_MARGIN_PX,
        "{path}: 다음 문단 줄은 글상자 세로 범위에 있다"
    );
    assert!(
        next.x >= outer_right - 0.5,
        "{path}: 다음 문단({:.1})은 글상자 바깥 오른쪽({outer_right:.1}) 뒤에서 시작한다",
        next.x
    );
}

/// 대조군: 어울림 그림이 첫 줄만 덮으면 둘째 줄부터 전폭이다(한/글 2020 Print 「…어울」 뒤
/// 「…겹치는 세로」).
#[test]
fn square_picture_carves_only_the_lines_it_overlaps() {
    let path = "samples/issue8282/tac_picture_square_picture_overlap.hwpx";
    let texts: Vec<_> = placement(&fixture(path))
        .lines
        .iter()
        .map(|(_, t)| t.trim_end().to_string())
        .collect();
    assert!(
        texts.len() == 4 && texts[0].ends_with("어울") && texts[1].ends_with("겹치는 세로"),
        "{path}: 한/글 Print 와 같은 줄 나눔 — {texts:?}"
    );
}
