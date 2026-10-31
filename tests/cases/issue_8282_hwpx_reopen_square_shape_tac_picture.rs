//! 검사 유형: 관계형.
//! 2026-10-31 측정 raw(96dpi): samples/issue8282/square_textbox_long_host.hwpx 1쪽
//! Native 91.96% · fresh WASM 92.15% — 기준 pdf/issue8282/square_textbox_long_host-2024.pdf
//! (한/글 2024 13.0.0.564 Print). 98% 미만이라 절대 px 좌표는 고정하지 않고, 한/글 재저장본의
//! 줄 기록에서 온 줄 사이 관계만 검사한다. 남은 차이는 글상자 위치(이 변경 전후 같음)다.
//!
//! [#8282] HWPX 로 저장한 뒤 다시 열어도 글자처럼 그림 + 어울림(Square·Tight·Through) 부동
//! 글상자 문단의 그림 줄 높이가 남는다.
//!
//! 결함: HWPX 저장은 편집한 문단의 `linesegarray` 를 뺀다. 다시 열면 저장 줄 없는 host 를 현재
//! 프레임에서 재조판하는데, 어울림 도형이 있으면 프레임이 문단을 받지 않았다(#8271 은 줄 폭에
//! 중립인 배치만 받았다). 합성 줄(글자 높이)만 남아 그림 높이가 줄에서 빠지고 뒤 내용이 올라왔다.
//!
//! 독립 기준: 한/글 2024 가 같은 HWPX 를 열어 다시 저장한 HWP 의 host 줄 기록 —
//! 줄 0 = vpos 0·lh 4447(그림 높이)·전폭, 줄 1~4 = vpos 5047·6647·8247·9847(lh 1000),
//! 다음 문단 vpos 11447. 글상자가 host 줄과 겹치지 않는 배치라 한/글도 줄을 좁히지 않는다.
//! Tight·Through 글상자도 한/글은 Square 로 저장하며 host 줄 기록이 같다.
//! 글상자가 host 줄과 겹칠 때 줄을 좁히는 규칙은 rhwp 의 어느 경로에도 없어 이 검사 밖이다.

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

/// 본문 줄(글상자·표 안 제외)을 위에서부터.
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

fn image(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(image)
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

const PICTURE_HEIGHT_HU: f64 = 4447.0;
const GROUP: &str = "가나다라마바사아자차카타파하 거너더러머버서어저처커터퍼허 고노도로모보소오조초코토포호 구누두루무부수우주추쿠투푸후 그느드르므브스으즈츠크트프흐 기니디리미비시이지치키티피히 ";

fn px(hu: f64) -> f64 {
    hu * 96.0 / 7200.0
}

/// 180자 host 의 둘째 글자 자리에 글자처럼 그림, 문단 끝에 종이 기준 (0,0) 의 부동 글상자(`wrap`).
fn edited(wrap: &str) -> DocumentCore {
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
            2,
            &[],
            &jpg,
            8504,
            PICTURE_HEIGHT_HU as u32,
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
        17008,
        5669,
        0,
        0,
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

struct Placement {
    host: Vec<BoundingBox>,
    picture: BoundingBox,
    next_line: BoundingBox,
    host_text: String,
}

fn placement(core: &DocumentCore) -> Placement {
    let tree = core.build_page_render_tree(0).expect("page 1");
    let mut lines = Vec::new();
    body_lines(&tree.root, &mut lines);
    let next = lines
        .iter()
        .position(|(_, text)| text.contains("다음 문단"))
        .expect("다음 문단 줄");
    Placement {
        host: lines[..next].iter().map(|(bbox, _)| *bbox).collect(),
        picture: image(&tree.root).expect("그림"),
        next_line: lines[next].0,
        host_text: lines[..next]
            .iter()
            .map(|(_, text)| text.as_str())
            .collect(),
    }
}

fn reopen_hwpx(core: &DocumentCore) -> DocumentCore {
    let bytes = core.export_hwpx_native().expect("hwpx");
    let reopened = DocumentCore::from_bytes(&bytes).expect("reopen");
    assert!(
        reopened.document().sections[0].paragraphs[0]
            .line_segs
            .is_empty(),
        "전제: HWPX 저장본의 host 문단은 저장 줄 없이 재조판된다"
    );
    reopened
}

/// 한/글 2024 재저장본의 host 줄 기록과 줄 사이 관계가 같다.
fn assert_hancom_rows(label: &str, p: &Placement) {
    assert_eq!(p.host.len(), 5, "{label}: host 줄 수(한/글 5줄)");
    let top = p.host[0].y;
    assert!(
        p.host[0].height + 0.5 >= px(PICTURE_HEIGHT_HU),
        "{label}: 첫 줄 높이 {:.1} ≥ 그림 높이 {:.1}",
        p.host[0].height,
        px(PICTURE_HEIGHT_HU)
    );
    let line_tops: Vec<f64> = p.host.iter().map(|b| b.y - top).collect();
    // 줄 상자 위는 줄 높이 차(4447 vs 1000)만큼 내려가 있으니 줄 아래끝 간격으로 견준다.
    let bottoms: Vec<f64> = p.host.iter().map(|b| b.y + b.height - top).collect();
    for (row, hancom_bottom) in [4447.0, 6047.0, 7647.0, 9247.0, 10847.0].iter().enumerate() {
        assert!(
            (bottoms[row] - px(*hancom_bottom)).abs() <= 1.0,
            "{label}: 줄 {row} 아래끝 {:.2} (한/글 {:.2}), 줄 위 {:?}",
            bottoms[row],
            px(*hancom_bottom),
            line_tops
        );
    }
    assert!(
        p.next_line.y + 0.5 >= p.host[4].y + p.host[4].height,
        "{label}: 다음 문단({:.1})이 host 마지막 줄 아래({:.1})에서 시작",
        p.next_line.y,
        p.host[4].y + p.host[4].height
    );
    assert!(
        p.picture.y + 0.5 >= top && p.picture.y + p.picture.height <= top + p.host[0].height + 0.5,
        "{label}: 그림이 첫 줄 상자 안"
    );
}

fn assert_same_placement(label: &str, before: &Placement, after: &Placement) {
    assert_eq!(
        before.host_text, after.host_text,
        "{label}: host 줄 내용·순서"
    );
    assert_eq!(before.host.len(), after.host.len(), "{label}: host 줄 수");
    for (row, (a, b)) in before.host.iter().zip(&after.host).enumerate() {
        assert!(
            (a.y - b.y).abs() <= 0.5 && (a.height - b.height).abs() <= 0.5,
            "{label}: host 줄 {row} — 저장 전 y {:.1} h {:.1}, 다시 열기 y {:.1} h {:.1}",
            a.y,
            a.height,
            b.y,
            b.height
        );
    }
    for (what, a, b) in [
        ("그림 위", before.picture.y, after.picture.y),
        ("다음 문단 위", before.next_line.y, after.next_line.y),
    ] {
        assert!(
            (a - b).abs() <= 0.5,
            "{label}: {what} — 저장 전 {a:.1}, 다시 열기 {b:.1}"
        );
    }
}

#[test]
fn square_like_float_textbox_keeps_tac_picture_line_after_hwpx_reopen() {
    for wrap in ["Square", "Tight", "Through"] {
        let core = edited(wrap);
        let before = placement(&core);
        assert_hancom_rows(&format!("{wrap} 저장 전"), &before);
        let after = placement(&reopen_hwpx(&core));
        assert_hancom_rows(&format!("{wrap} HWPX 다시 열기"), &after);
        assert_same_placement(wrap, &before, &after);
    }
}

/// 같은 편집의 HWP 저장본(저장 줄 보존)과 HWPX 저장본(저장 줄 없음)이 같은 배치를 낸다.
#[test]
fn hwpx_and_hwp_reopen_agree_for_square_float_host() {
    let core = edited("Square");
    let hwp =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("hwp")).expect("hwp reopen");
    assert!(
        !hwp.document().sections[0].paragraphs[0]
            .line_segs
            .is_empty(),
        "HWP 저장본은 host 줄을 보존한다"
    );
    assert_same_placement(
        "HWP 대 HWPX",
        &placement(&hwp),
        &placement(&reopen_hwpx(&core)),
    );
}

/// 한/글 Print 기준과 대조한 실제 HWPX(rhwp 저장본)를 열어도 같은 줄 관계다.
#[test]
fn printed_fixture_keeps_hancom_host_rows() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8282/square_textbox_long_host.hwpx");
    let fixture =
        DocumentCore::from_bytes(&std::fs::read(path).expect("fixture")).expect("parse fixture");
    assert!(
        fixture.document().sections[0].paragraphs[0]
            .line_segs
            .is_empty(),
        "전제: fixture host 는 저장 줄이 없다"
    );
    assert_hancom_rows("fixture", &placement(&fixture));
}
