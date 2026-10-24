//! Issue #3837: 저장 `vpos` 가 되돌아가는 자리를 쪽 경계로 인정한다.
//!
//! 한글이 어떤 항목을 다음 쪽 맨 위에 뒀다면 그 항목의 저장 `vpos` 는 쪽 상단 값으로
//! **되돌아간다**. rhwp 는 그 신호를 쪽 경계로 쓰지 않아, 잔여 공간에 들어가기만 하면
//! 현재 쪽 끝에 얹었다. 쪽 총수는 맞는 채 문단 하나만 어긋나므로 쪽수 지표로는 침묵한다.
//!
//! ```text
//! samples/issue3837/stored_vpos_rewind_form.hwp — 응시원서 서식 (한글 4쪽)
//!   1쪽 마지막 항목 pi=12  vpos=41645
//!   그 다음   항목 pi=13  vpos=1000   <- 되돌아감. 한글은 2쪽 맨 위에 뒀다
//!   수정 전   rhwp 는 1쪽 잔여(10.9px 슬랙)에 42.5px 표를 얹었다
//! ```
//!
//! 판별력은 r29 서베이 `PI_MISMATCH` n=1 코호트 66건으로 쟀다 — 어긋난 항목의 36% 가
//! 되돌아감인데, 같은 문서 **다른 쪽**의 마지막 항목은 1,134개 중 2개(0.2%)뿐이다.
//! 쪽이 90% 이상 찼을 때만 인정한다(쪽 중간 인정은 연쇄 발동으로 쪽수를 늘린다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::DocumentCore;
use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/issue3837/stored_vpos_rewind_form.hwp";
/// 되돌아간 표가 있는 문단. 한글은 2쪽(1-based)에 둔다.
const REWOUND_PARA: &str = "pi=13 ";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn dump_pages() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["dump-pages", sample().to_str().unwrap()])
        .output()
        .expect("rhwp 실행 실패");
    assert!(
        out.status.success(),
        "dump-pages 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 항목이 놓인 쪽(1-based)을 찾는다.
fn page_of(dump: &str, marker: &str) -> Option<usize> {
    dump.split("=== 페이지 ")
        .skip(1)
        .position(|block| block.contains(marker))
        .map(|i| i + 1)
}

/// 되돌아간 항목은 다음 쪽에서 시작해야 한다.
#[test]
fn rewound_item_starts_the_next_page() {
    let dump = dump_pages();
    let page = page_of(&dump, REWOUND_PARA).expect("되돌아간 문단을 어느 쪽에서도 못 찾았다");

    assert_eq!(
        page, 2,
        "되돌아간 표({REWOUND_PARA})가 {page}쪽에 놓였다 — 저장 vpos 는 1000 으로 \n         \
         되돌아가 한글이 2쪽 맨 위에 뒀음을 말한다. 잔여 공간에 들어간다는 이유로 \n         \
         1쪽 끝에 얹으면 안 된다."
    );

    // 전4쪽 Native/fresh WASM 검증 뒤 저장 줄과 실제 표의 관계를 보호한다.
    // 절대 y를 고정하지 않는다. 독립 PDF의 3쪽 마지막 표는 바로 앞 빈 줄의
    // 줄 상자·간격·원본 표 바깥여백을 지나 시작하며, 마지막 표는 4쪽 소속이다.
    let input = std::fs::read(sample()).expect("보존된 응시원서 원본");
    let source = rhwp::parse_document(&input).expect("원본 저장 속성");
    let core = DocumentCore::from_bytes(&input).expect("응시원서 로드");
    assert_eq!(core.page_count(), 4, "독립 Print의 네 서식 쪽을 보존한다");
    assert_eq!(page_of(&dump, "pi=40 "), Some(3));
    assert_eq!(page_of(&dump, "pi=41 "), Some(4));
    let tree = core.build_page_render_tree(2).expect("세 번째 서식 쪽");
    let blank = body_node(&tree.root, 39, false).expect("표 앞 원본 빈 글줄");
    let painted_table = body_node(&tree.root, 40, true).expect("3쪽 마지막 표");
    let preceding = &source.sections[0].paragraphs[39];
    assert!(preceding.text.is_empty());
    let [line] = preceding.line_segs.as_slice() else {
        panic!("표 앞 원본 한 줄");
    };
    let Control::Table(table) = &source.sections[0].paragraphs[40].controls[0] else {
        panic!("원본 마지막 표");
    };
    let owned_gap = hwpunit_to_px(
        line.line_height + line.line_spacing + i32::from(table.outer_margin_top),
        DEFAULT_DPI,
    );
    let tolerance = painted_table.bbox.y.abs() * 1e-9;
    assert!(
        (painted_table.bbox.y - blank.bbox.y - owned_gap).abs() <= tolerance,
        "실제 표는 앞 빈 줄의 상자·간격과 원본 바깥여백 뒤에 시작한다"
    );

    // Print 원본에서도 사진 칸의 '2매' 위에 ㊞ 개체가 겹친다. 글자 쌍의
    // 겹침 건수 대신 두 사진 칸과 각 칸 안의 원본 도장을 함께 보존한다.
    let first = core.build_page_render_tree(0).expect("응시원서 첫 쪽");
    let mut photos = Vec::new();
    let mut stamps = Vec::new();
    photo_nodes(&first.root, &mut photos, &mut stamps);
    assert_eq!(photos.len(), 2, "본서식과 부본의 사진 안내를 보존한다");
    for photo in photos {
        let inside = stamps
            .iter()
            .filter(|stamp| {
                stamp.bbox.x >= photo.bbox.x
                    && stamp.bbox.y >= photo.bbox.y
                    && stamp.bbox.x + stamp.bbox.width <= photo.bbox.x + photo.bbox.width
                    && stamp.bbox.y + stamp.bbox.height <= photo.bbox.y + photo.bbox.height
            })
            .count();
        assert_eq!(inside, 1, "사진 칸마다 원본 도장 한 개가 포함된다");
    }
}

fn body_node(node: &RenderNode, para: usize, table: bool) -> Option<&RenderNode> {
    let hit = match &node.node_type {
        RenderNodeType::Table(data) => {
            table
                && data.para_index == Some(para)
                && data.section_index == Some(0)
                && data.cell_context.is_none()
        }
        RenderNodeType::TextLine(data) => {
            !table && data.para_index == Some(para) && data.section_index == Some(0)
        }
        _ => false,
    };
    if hit {
        return Some(node);
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    node.children
        .iter()
        .find_map(|child| body_node(child, para, table))
}

fn node_text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => node.children.iter().map(node_text).collect(),
    }
}

fn photo_nodes<'a>(
    node: &'a RenderNode,
    photos: &mut Vec<&'a RenderNode>,
    stamps: &mut Vec<&'a RenderNode>,
) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        let text = node_text(node);
        if text.contains("2매") && text.contains("동일원판") {
            photos.push(node);
        }
    }
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.display_or_text() == "㊞") {
        stamps.push(node);
    }
    for child in &node.children {
        photo_nodes(child, photos, stamps);
    }
}

/// 표본이 계약을 시험하는 형태인지 못박는다.
///
/// 되돌아감은 **직전 항목의 vpos 가 충분히 큰 자리**에서만 신호가 된다. 표본의 1쪽이
/// 그 형태가 아니면 이 테스트는 수정 전에도 통과한다.
#[test]
fn the_fixture_has_a_large_preceding_vpos() {
    let dump = dump_pages();
    let prior = dump
        .lines()
        .find(|l| l.contains("pi=12 ") && l.contains("Table"))
        .expect("직전 표(pi=12)를 못 찾았다");
    let vpos: i64 = prior
        .split("vpos=")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse().ok())
        .expect("직전 항목 vpos 파싱 실패");

    assert!(
        vpos > 5000,
        "직전 항목의 저장 vpos 가 {vpos} 다 — 5000 이하면 되돌아감 가드가 애초에 \n         \
         발동하지 않아 계약을 시험하지 못한다."
    );
}
