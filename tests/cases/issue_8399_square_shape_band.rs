//! 검사 유형: 관계형(저장 줄 기록 대조 + 렌더 포함·겹침 관계).
//! 2026-10-31 측정 raw(96dpi) — 기준 pdf/issue8399/*-2024.pdf(한/글 2024 13.0.0.564 Print,
//! 같은 HWP·HWPX 각각 인쇄). samples/issue8399 A·D·E·P 의 HWP·HWPX 1쪽 Native·fresh WASM 모두
//! 100.00%. 절대 px 대신 원본 속성(오프셋 + 바깥 여백)에서 계산한 관계와 한/글 재저장 HWP 의
//! 줄 기록을 검사한다.
//!
//! [#8399] 어울림(Square·Tight·Through) 비-TAC 글상자가 본문 줄과 겹치면 줄을 깎는다.
//!
//! 결함: 배제 밴드의 소유자(`layout_picture_band`)가 비-TAC 그림만 받아 글상자는 모든 경로에서
//! 전폭 줄을 냈다. 같은 장치가 구역 첫 문단(구역 나눔 표지)을 host 로 받지 않았고, 밴드가 구역
//! 마지막 문단 아래로 이어지면 결과를 버렸다. HWPX 는 rhwp 가 계산한 줄을 싣지 않아 다시 열면
//! 렌더 프레임이 전폭으로 채웠다. 글상자 잉크는 위 바깥 여백만큼 위에 그렸다.
//!
//! 독립 기준: 한/글 2024 가 같은 HWPX·HWP 를 열어 다시 저장한 HWP 의 줄 기록과 Print PDF.
//! 글상자 60×20mm, 바깥 여백 283, 본문 왼쪽 8504·위 9922, 본문 폭 42520.
//!
//! | 변형 | host / 가로·세로(용지) | 한/글 줄 기록 |
//! |---|---|---|
//! | D | 180자 / 8504·13000 | host 줄 0–3 `cs=17574 sw=24946`, 줄 4–5 전폭 |
//! | A | 「앞 글 뒤 글」 / 8504·15000 | host 전폭, 다음 문단 `cs=17574 sw=24946` |
//! | E | 「앞 글 뒤 글」 / 36000·9922 | host·다음 문단 `sw=27496` |
//! | B | 「앞 글 뒤 글」 / 8504·0 | 밴드가 본문 위 — 전폭 |
//!
//! `cs=17574` 는 오프셋이 바깥 여백 상자의 원점이라는 뜻이다(0 + 283 + 17008 + 283). 한/글은
//! 잉크를 오프셋 + 283 에 그린다(D 테두리 117·177px = 8787·13283HU). Tight·Through 글상자도
//! 한/글은 Square 로 저장하고 줄 기록이 같다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const GROUP: &str = "가나다라마바사아자차카타파하 거너더러머버서어저처커터퍼허 고노도로모보소오조초코토포호 구누두루무부수우주추쿠투푸후 그느드르므브스으즈츠크트프흐 기니디리미비시이지치키티피히 ";
const SHORT: &str = "앞 글 뒤 글";
const BOX_W: u32 = 17008;
const BOX_H: u32 = 5669;
const OUTER_MARGIN: i32 = 283;
/// 한/글이 겹친 줄에 남긴 오른쪽 조각.
const NARROW: (i32, i32) = (17574, 24946);
const FULL: (i32, i32) = (0, 42520);

fn json_usize(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect(key)
}

/// #8282 와 같은 편집: host 둘째 글자 자리에 글자처럼 그림, 문단 끝에 용지 기준 글상자.
fn edited(host: &str, wrap: &str, horz: u32, vert: u32) -> DocumentCore {
    let host_chars = host.chars().count();
    let jpg = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/images/moogung.jpg"),
    )
    .expect("jpg");
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, host).expect("text");
    core.split_paragraph_native(0, 0, host_chars, None)
        .expect("split");
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
    core.create_shape_control_native(
        0,
        0,
        host_chars + 1,
        BOX_W,
        BOX_H,
        horz,
        vert,
        false,
        wrap,
        "textbox",
        false,
        false,
        &[],
    )
    .expect("float textbox");
    core
}

/// 문단별 저장 줄 `(text_start, vertical_pos, line_height, column_start, segment_width)`.
fn rows(core: &DocumentCore) -> Vec<Vec<(u32, i32, i32, i32, i32)>> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| {
            paragraph
                .line_segs
                .iter()
                .map(|seg| {
                    (
                        seg.text_start,
                        seg.vertical_pos,
                        seg.line_height,
                        seg.column_start,
                        seg.segment_width,
                    )
                })
                .collect()
        })
        .collect()
}

fn reopen_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("hwp reopen")
}

fn reopen_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("hwpx")).expect("hwpx reopen")
}

/// 편집 직후·HWP 저장본·HWPX 저장본이 모두 한/글 줄 기록을 낸다.
fn assert_hancom_rows(
    label: &str,
    core: &DocumentCore,
    expected: &[Vec<(u32, i32, i32, (i32, i32))>],
) {
    let expected: Vec<Vec<(u32, i32, i32, i32, i32)>> = expected
        .iter()
        .map(|paragraph| {
            paragraph
                .iter()
                .map(|&(text_start, vpos, height, (start, width))| {
                    (text_start, vpos, height, start, width)
                })
                .collect()
        })
        .collect();
    for (route, doc) in [
        ("편집 직후", None),
        ("HWP 다시 열기", Some(reopen_hwp(core))),
        ("HWPX 다시 열기", Some(reopen_hwpx(core))),
    ] {
        let actual = rows(doc.as_ref().unwrap_or(core));
        assert_eq!(
            actual, expected,
            "{label} {route}: 줄 기록 (text_start, vpos, lh, cs, sw)이 한/글 재저장본과 같아야 한다"
        );
    }
}

#[test]
fn square_textbox_over_host_rows_narrows_them_like_hancom() {
    // D: 글상자 바깥 상자 세로 3078..9313 이 host 줄 0–3 과 겹친다.
    let row = |ts, vpos, lh, span| (ts, vpos, lh, span);
    assert_hancom_rows(
        "D",
        &edited(&GROUP.repeat(2), "Square", 8504, 13000),
        &[
            vec![
                row(0, 0, 4447, NARROW),
                row(41, 5047, 1000, NARROW),
                row(67, 6647, 1000, NARROW),
                row(93, 8247, 1000, NARROW),
                row(119, 9847, 1000, FULL),
                row(164, 11447, 1000, FULL),
            ],
            vec![row(0, 13047, 1000, FULL)],
        ],
    );
}

#[test]
fn square_like_textbox_below_host_narrows_next_paragraph() {
    for wrap in ["Square", "Tight", "Through"] {
        assert_hancom_rows(
            &format!("A {wrap}"),
            &edited(SHORT, wrap, 8504, 15000),
            &[vec![(0, 0, 4447, FULL)], vec![(0, 5047, 1000, NARROW)]],
        );
    }
}

#[test]
fn textbox_on_the_right_trims_rows_to_its_outer_left_edge() {
    // E: 오프셋 36000 − 본문 왼쪽 8504 = 27496 — 바깥 상자 왼쪽에서 줄이 끝난다.
    assert_hancom_rows(
        "E",
        &edited(SHORT, "Square", 36000, 9922),
        &[
            vec![(0, 0, 4447, (0, 27496))],
            vec![(0, 5047, 1000, (0, 27496))],
        ],
    );
}

/// 반례: 바깥 상자가 본문 위에서 끝나면 어느 줄도 깎지 않고, 밴드 줄도 발행하지 않는다.
#[test]
fn textbox_above_the_body_leaves_rows_full_width() {
    let core = edited(SHORT, "Square", 8504, 0);
    let expected = vec![
        vec![(0, 0, 4447, 0, 42520)],
        vec![(0, 5047, 1000, 0, 42520)],
    ];
    assert_eq!(rows(&core), expected, "B 편집 직후");
    assert_eq!(rows(&reopen_hwp(&core)), expected, "B HWP 다시 열기");
    // HWPX 는 rhwp 가 계산한 host 줄을 싣지 않는다. 밴드가 없으니 로드도 줄을 만들지 않고, 렌더
    // 프레임이 전폭으로 채운다(#8282). 다음 문단은 원본 저장 줄을 지닌다.
    let hwpx = reopen_hwpx(&core);
    let hwpx_rows = rows(&hwpx);
    assert!(hwpx_rows[0].is_empty(), "B HWPX host 는 저장 줄 없음");
    assert_eq!(hwpx_rows[1], expected[1], "B HWPX 다음 문단");
    let tree = hwpx.build_page_render_tree(0).expect("page 1");
    let mut lines = Vec::new();
    body_lines(&tree.root, &mut lines);
    let host = lines
        .iter()
        .find(|(_, text)| text.contains("뒤 글"))
        .expect("host 줄");
    assert!(
        (host.0.x - px(8504)).abs() <= 0.5,
        "B HWPX host 줄은 본문 왼쪽에서 시작 — x {:.2}",
        host.0.x
    );
}

/// 글상자를 옮기면 밴드를 다시 투영한다(그림의 #6204 와 같은 계약).
#[test]
fn moving_the_textbox_reprojects_its_band() {
    let mut core = edited(SHORT, "Square", 8504, 0);
    let shape_index = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, rhwp::model::control::Control::Shape(_)))
        .expect("글상자");
    core.set_shape_properties_native(0, 0, shape_index, r#"{"vertOffset":15000}"#)
        .expect("move");
    assert_hancom_rows(
        "B→A 이동",
        &core,
        &[vec![(0, 0, 4447, FULL)], vec![(0, 5047, 1000, NARROW)]],
    );
}

/// 구역 첫 문단(구역 나눔 표지)의 어울림 **그림**도 밴드를 얻는다 — host 앞의 나눔은 밴드
/// 밖의 경계다. 한/글 P: 그림 60×20mm·바깥 여백 0, 용지 8504·13000, 줄 2–5 `cs=17008 sw=25512`.
#[test]
fn square_picture_in_the_first_paragraph_narrows_rows_like_hancom() {
    let host = GROUP.repeat(2);
    let host_chars = host.chars().count();
    let jpg = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/images/moogung.jpg"),
    )
    .expect("jpg");
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, &host).expect("text");
    core.split_paragraph_native(0, 0, host_chars, None)
        .expect("split");
    core.insert_text_native(0, 1, 0, "다음 문단")
        .expect("text 2");
    let pic = core
        .insert_picture_native(
            0,
            0,
            host_chars,
            &[],
            &jpg,
            BOX_W,
            BOX_H,
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
        r#"{"treatAsChar":false,"textWrap":"Square","horzRelTo":"Paper","vertRelTo":"Paper","horzAlign":"Left","vertAlign":"Top","horzOffset":8504,"vertOffset":13000}"#,
    )
    .expect("square picture");
    assert_eq!(
        core.document().sections[0].paragraphs[0].column_type,
        rhwp::model::paragraph::ColumnBreakType::Section,
        "전제: 구역 첫 문단은 구역 나눔 표지를 지닌다"
    );
    let (cut_start, cut_width) = (17008, 25512);
    let expected = vec![
        vec![
            (0, 0, 1000, 0, 42520),
            (61, 1600, 1000, 0, 42520),
            (106, 3200, 1000, cut_start, cut_width),
            (132, 4800, 1000, cut_start, cut_width),
            (159, 6400, 1000, cut_start, cut_width),
            (186, 8000, 1000, cut_start, cut_width),
        ],
        vec![(0, 9600, 1000, 0, 42520)],
    ];
    assert_eq!(rows(&core), expected, "P 편집 직후");
    assert_eq!(rows(&reopen_hwp(&core)), expected, "P HWP 다시 열기");
    // 저장 줄 없는 어울림 그림(HWPX 다시 열기)은 기존 렌더 경로가 소유한다 — 이 변경의 범위 밖이다.
}

fn find_rect(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Rectangle(_)) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(find_rect)
}

fn text_of(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text_of(child));
    }
    out
}

fn body_lines(node: &RenderNode, out: &mut Vec<(BoundingBox, String)>) {
    if matches!(
        node.node_type,
        RenderNodeType::Rectangle(_) | RenderNodeType::Table(_)
    ) {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push((node.bbox, text_of(node)));
        return;
    }
    for child in &node.children {
        body_lines(child, out);
    }
}

fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 글상자 잉크는 바깥 여백 안쪽(오프셋 + 283)에 있고, 깎인 줄의 글자는 그 바깥 상자와 겹치지 않는다.
#[test]
fn textbox_ink_sits_inside_its_outer_box_and_text_clears_it() {
    for (label, core) in [
        ("D", edited(&GROUP.repeat(2), "Square", 8504, 13000)),
        (
            "D HWPX",
            reopen_hwpx(&edited(&GROUP.repeat(2), "Square", 8504, 13000)),
        ),
        ("A", edited(SHORT, "Square", 8504, 15000)),
    ] {
        let tree = core.build_page_render_tree(0).expect("page 1");
        let rect = find_rect(&tree.root).expect("글상자 사각형");
        let vert = if label == "A" { 15000 } else { 13000 };
        assert!(
            (rect.x - px(8504 + OUTER_MARGIN)).abs() <= 0.5
                && (rect.y - px(vert + OUTER_MARGIN)).abs() <= 0.5,
            "{label}: 글상자 잉크 원점 ({:.2}, {:.2}) = 오프셋 + 바깥 여백 ({:.2}, {:.2})",
            rect.x,
            rect.y,
            px(8504 + OUTER_MARGIN),
            px(vert + OUTER_MARGIN)
        );
        let outer_right = rect.x + rect.width + px(OUTER_MARGIN);
        let outer_bottom = rect.y + rect.height + px(OUTER_MARGIN);
        let mut lines = Vec::new();
        body_lines(&tree.root, &mut lines);
        let overlapped: Vec<_> = lines
            .iter()
            .filter(|(bbox, text)| {
                !text.trim().is_empty()
                    && bbox.y < outer_bottom
                    && bbox.y + bbox.height > rect.y - px(OUTER_MARGIN)
            })
            .collect();
        assert!(
            !overlapped.is_empty(),
            "{label}: 글상자 높이에 걸친 본문 줄"
        );
        for (bbox, text) in overlapped {
            assert!(
                bbox.x + 0.5 >= outer_right,
                "{label}: 글상자 높이의 줄 「{text}」은 바깥 상자 오른쪽({outer_right:.2}) 뒤에서 시작 — x {:.2}",
                bbox.x
            );
        }
    }
}

/// 한/글 Print 기준과 대조한 HWPX·HWP fixture 를 열어도 한/글 줄 기록을 낸다.
#[test]
fn printed_fixtures_open_with_hancom_rows() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue8399");
    for (name, expected_next) in [("A", NARROW), ("E", (0, 27496))] {
        for ext in ["hwpx", "hwp"] {
            let bytes = std::fs::read(root.join(format!("{name}.{ext}"))).expect("fixture");
            let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
            let rows = rows(&core);
            assert_eq!(
                rows[1].iter().map(|row| (row.3, row.4)).collect::<Vec<_>>(),
                vec![expected_next],
                "{name}.{ext}: 다음 문단 줄 조각"
            );
        }
    }
    for (name, exts, host_spans) in [
        (
            "D",
            ["hwpx", "hwp"].as_slice(),
            vec![NARROW, NARROW, NARROW, NARROW, FULL, FULL],
        ),
        // P.hwpx(저장 줄 없는 어울림 그림)는 기존 렌더 경로가 소유한다 — 범위 밖이다.
        (
            "P",
            ["hwp"].as_slice(),
            vec![
                FULL,
                FULL,
                (17008, 25512),
                (17008, 25512),
                (17008, 25512),
                (17008, 25512),
            ],
        ),
    ] {
        for ext in exts {
            let bytes = std::fs::read(root.join(format!("{name}.{ext}"))).expect("fixture");
            let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
            let spans: Vec<_> = rows(&core)[0].iter().map(|row| (row.3, row.4)).collect();
            assert_eq!(spans, host_spans, "{name}.{ext}: host 줄 조각");
        }
    }
}
