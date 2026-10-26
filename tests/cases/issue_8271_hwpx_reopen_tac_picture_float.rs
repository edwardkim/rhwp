//! [#8271] HWPX 로 저장한 뒤 다시 열어도 글자처럼 그림 + 줄 폭 중립 부동 개체 문단의 배치가 같다.
//!
//! 결함: HWPX 저장은 편집한 문단의 `linesegarray` 를 뺀다(한/글이 다시 계산하게 하는 설계).
//! rhwp 가 다시 열면 그 문단을 저장 줄 없이 현재 프레임에서 재조판하는데, 같은 문단에 글자처럼
//! 취급하지 않은 그림·도형이 하나라도 있으면 프레임이 문단을 받지 않았다. 그래서 합성 줄(글자
//! 높이)만 남아 글자처럼 그림의 높이가 줄에서 빠지고, 뒤 문단이 그만큼 위로 올라왔다.
//!
//! 독립 기준: 같은 문서의 저장 전 배치와 HWP 저장본(저장 줄 보존)을 다시 연 배치가 서로 같다.
//! 저장·재열기는 배치를 바꾸지 않아야 한다. 자리차지·글 뒤·글 앞 부동 개체는 줄 폭을 바꾸지
//! 않으므로(표의 같은 계약 #6102) 글자처럼 그림의 줄 구성에 관여하지 않는다.

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

/// 본문 줄(글상자·표 안 제외) 중 `needle` 을 담은 줄.
fn line_with(node: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if matches!(
        node.node_type,
        RenderNodeType::TextBox | RenderNodeType::Table(_)
    ) {
        return None;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle) {
        return Some(node.bbox);
    }
    node.children
        .iter()
        .find_map(|child| line_with(child, needle))
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

/// #8168 재현과 같은 편집: 「앞 글 뒤 글」 둘째 글자 자리에 글자처럼 그림, 같은 문단 끝에
/// 종이 기준 세로 15000HU 의 부동 글상자(`wrap`). `wrap` 이 None 이면 그림만 넣는다.
fn edited(wrap: Option<&str>) -> DocumentCore {
    let jpg = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("rhwp-studio/public/samples/images/moogung.jpg"),
    )
    .expect("jpg");
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank");
    core.insert_text_native(0, 0, 0, "앞 글 뒤 글")
        .expect("text");
    core.split_paragraph_native(0, 0, 7, None).expect("split");
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
    if let Some(wrap) = wrap {
        let margin_left = core.document().sections[0].section_def.page_def.margin_left;
        core.create_shape_control_native(
            0,
            0,
            7,
            17008,
            5669,
            margin_left,
            15000,
            false,
            wrap,
            "textbox",
            false,
            false,
            &[],
        )
        .expect("float textbox");
    }
    core
}

struct Placement {
    host_line: BoundingBox,
    picture: BoundingBox,
    next_line: BoundingBox,
}

fn placement(core: &DocumentCore) -> Placement {
    let tree = core.build_page_render_tree(0).expect("page 1");
    Placement {
        host_line: line_with(&tree.root, "앞").expect("그림 host 줄"),
        picture: find(&tree.root, &|n| {
            matches!(n.node_type, RenderNodeType::Image(_))
        })
        .expect("그림"),
        next_line: line_with(&tree.root, "다음 문단").expect("다음 문단 줄"),
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

fn assert_same_placement(label: &str, before: &Placement, after: &Placement) {
    let picture_h = PICTURE_HEIGHT_HU * 96.0 / 7200.0;
    assert!(
        after.host_line.height + 0.5 >= picture_h,
        "{label}: 다시 연 host 줄 높이({:.1})가 그림 높이({picture_h:.1}) 이상",
        after.host_line.height
    );
    for (what, a, b) in [
        ("host 줄 위", before.host_line.y, after.host_line.y),
        (
            "host 줄 높이",
            before.host_line.height,
            after.host_line.height,
        ),
        ("그림 위", before.picture.y, after.picture.y),
        ("다음 문단 위", before.next_line.y, after.next_line.y),
    ] {
        assert!(
            (a - b).abs() <= 0.5,
            "{label}: {what} — 저장 전 {a:.1}, HWPX 다시 열기 {b:.1}"
        );
    }
    assert!(
        after.next_line.y + 0.5 >= after.host_line.y + after.host_line.height,
        "{label}: 다음 문단({:.1})이 그림 host 줄 아래({:.1})에서 시작",
        after.next_line.y,
        after.host_line.y + after.host_line.height
    );
}

#[test]
fn topandbottom_float_keeps_tac_picture_line_after_hwpx_reopen() {
    let core = edited(Some("TopAndBottom"));
    assert_same_placement(
        "위아래 글상자",
        &placement(&core),
        &placement(&reopen_hwpx(&core)),
    );
}

#[test]
fn behind_text_float_keeps_tac_picture_line_after_hwpx_reopen() {
    let core = edited(Some("BehindText"));
    assert_same_placement(
        "글 뒤 글상자",
        &placement(&core),
        &placement(&reopen_hwpx(&core)),
    );
}

/// 대조군: 부동 개체가 없는 글자처럼 그림 문단은 종전에도 같았다.
#[test]
fn picture_only_host_keeps_its_line_after_hwpx_reopen() {
    let core = edited(None);
    assert_same_placement("그림만", &placement(&core), &placement(&reopen_hwpx(&core)));
}

/// 같은 편집의 HWP 저장본(저장 줄 보존)과 HWPX 저장본(저장 줄 없음)이 같은 배치를 낸다.
#[test]
fn hwpx_and_hwp_reopen_agree_for_topandbottom_float_host() {
    let core = edited(Some("TopAndBottom"));
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
