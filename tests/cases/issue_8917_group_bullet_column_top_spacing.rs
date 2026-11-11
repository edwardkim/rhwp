//! 관계형·고정형 — [#8917] 저장 글줄 안 작은 글자처럼 묶음 도형 문단이 쪽 맨 위에서 저장 위 간격을 유지한다.
//! 주석 갱신: 2026-11-11 02:15 KST (새 파일)
//!
//! 입력: 공공기관이 공개한 문서 두 건.
//! - `44907-group-bullet-column-top.hwp`(경찰청 도로교통법 시행령 개정 내용, 한컴오피스 2020 저장, 4쪽).
//!   2쪽 맨 위 `pi=8`, 4쪽 맨 위 `pi=20` 은 글머리 묶음 도형(글자처럼, 앞으로) 하나와 글을 가진 한 줄
//!   문단이며 저장 첫 줄 vpos 는 문단 위 간격 1000HU 다.
//! - `1471000-group-bullet-column-top.hwpx`(식약처 과업내용서, 3쪽). 3쪽 맨 위 `pi=30` 은 같은 모양의
//!   묶음 글머리 문단이며 저장 첫 줄 vpos 는 위 간격 1500HU 다.
//!
//! 한/글은 쪽 첫 문단의 첫 줄 vpos 에 문단 위 간격을 저장하고(쪽 첫 원점 = vpos − 위 간격),
//! 뒤 문단들은 같은 저장 축에서 놓인다. 묶음 도형이 저장 글줄의 기준선 상자 안에 들어가면 단일
//! 그리기 개체와 같은 글줄이므로 이 쪽 원점을 써야 한다. 수정 전에는 묶음이 `drawing()` 이 없다는
//! 이유로 거부되어 쪽 원점이 0 으로 잡히고, 뒤 문단들이 쪽 첫 위 간격만큼 위로 올라왔다.
//!
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8917/44907-group-bullet-column-top-2020.pdf`(4쪽),
//! `pdf/issue8917/1471000-group-bullet-column-top-2020.pdf`(3쪽).
//! - 고정형: 전체 쪽수 4 · 3.
//! - 관계형: 대상 문단 첫 글줄 위끝 − 본문 영역 위끝 = 저장 첫 줄 vpos(HU→px), 묶음 도형은 그 글줄 상자 안.
//! - 관계형: 그 쪽의 모든 본문 글줄 위끝 − 대상 문단 첫 글줄 위끝 = 저장 vpos 차(HU→px). 수정 전에는
//!   44907 2쪽 `pi=9` 이후 9줄이 −13.33px, 1471000 3쪽 `pi=32` 이후 17줄이 −20.00px 어긋났다.
//!
//! 측정(2026-11-11, base upstream/devel 5ea1227505 + 이 변경의 src, `scripts/visual_sweep.py --silhouette-only
//! --compat 2022 --dpi 96`, raw = `silhouette_raw_match_percent`, 글꼴은 사용자 글꼴·`ttfs/hwp`·`ttfs/windows` 공급):
//! - 44907(4쪽): Native 99.35 · 99.76 · 99.99 · 99.99, fresh WASM 99.35 · 99.99 · 99.99 · 99.99.
//!   두 경로 전쪽 최저 99.35(1쪽). 대상 2쪽 Native 99.76 / WASM 99.99, 4쪽 99.99 / 99.99.
//! - 1471000(3쪽): Native 95.47 · 96.86 · 96.69, fresh WASM 100.00 · 100.00 · 100.00.
//!   두 경로 전쪽 최저 95.47(Native 1쪽). 대상 3쪽 Native 96.69 / WASM 100.00.
//! - 수정 전 Native(devel 5ea1227505): 44907 2쪽 58.61 · 4쪽 63.52, 1471000 3쪽 19.31.
//! - 1471000 3쪽 남은 차이는 「정당한 이유 없이」 줄의 글자 폭(가로)이며 줄 위치는 PDF 와 같다.
//! - 증거: `mydocs/pr/assets/issue_8917/`(Native·WASM·수정 전 TSV 원본, review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HWP: &str = "samples/issue8917/44907-group-bullet-column-top.hwp";
const HWPX: &str = "samples/issue8917/1471000-group-bullet-column-top.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn document(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {sample}: {e:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn first_line<'a>(all: &[&'a RenderNode], para: usize) -> Option<&'a RenderNode> {
    all.iter().copied().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.section_index == Some(0) && line.para_index == Some(para) && line.line_index == Some(0))
    })
}

/// 쪽 맨 위 묶음 글머리 문단의 첫 글줄이 저장 vpos(=위 간격) 위치에 놓이고 묶음이 그 줄 안에 있으며,
/// 그 쪽의 본문 줄이 모두 같은 저장 축에 놓인다.
fn assert_group_bullet_page_top(core: &DocumentCore, sample: &str, page: u32, para: usize) {
    let paragraphs = &core.document().sections[0].paragraphs;
    let paragraph = &paragraphs[para];
    assert!(
        paragraph
            .controls
            .iter()
            .any(|control| matches!(control, Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Group(_)))),
        "{sample} pi={para} 는 묶음 도형 문단이어야 한다"
    );
    let stored_first = paragraph.line_segs.first().expect("저장 줄").vertical_pos;
    assert!(
        stored_first > 0,
        "{sample} pi={para} 저장 첫 줄 vpos 는 위 간격(양수)"
    );

    let tree = core.build_page_render_tree(page).expect("쪽");
    let all = nodes(&tree.root);
    let body_top = all
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .map(|node| node.bbox.y)
        .expect("본문 영역");
    let line = first_line(&all, para)
        .unwrap_or_else(|| panic!("{sample} {}쪽 pi={para} 첫 글줄", page + 1));
    let offset = line.bbox.y - body_top;
    let expected = hu_to_px(stored_first);
    assert!(
        (offset - expected).abs() <= TOLERANCE_PX,
        "{sample} {}쪽 맨 위 pi={para} 첫 글줄이 본문 위끝에서 {offset:.2}px — 저장 위 간격 {expected:.2}px 이어야 한다",
        page + 1
    );

    let group = all
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Group(group)
                if group.section_index == Some(0) && group.para_index == Some(para))
        })
        .unwrap_or_else(|| panic!("{sample} {}쪽 pi={para} 묶음", page + 1));
    let line_bottom = line.bbox.y + line.bbox.height;
    assert!(
        group.bbox.y >= line.bbox.y - TOLERANCE_PX
            && group.bbox.y + group.bbox.height <= line_bottom + TOLERANCE_PX,
        "{sample} {}쪽 pi={para} 묶음({:.2}..{:.2})이 글줄 상자({:.2}..{line_bottom:.2}) 안에 있어야 한다",
        page + 1,
        group.bbox.y,
        group.bbox.y + group.bbox.height,
        line.bbox.y
    );

    // 쪽의 모든 본문 저장 줄은 쪽 첫 줄과 같은 저장 축에 있다. 칸 안 줄은 표가 소유하므로 제외한다.
    let mut body_lines = Vec::new();
    body_text_lines(&tree.root, &mut body_lines);
    assert!(body_lines.len() > 1, "{sample} {}쪽 본문 줄", page + 1);
    let mismatches: Vec<_> = body_lines
        .iter()
        .filter_map(|(top, line_para, line_index, vpos)| {
            let drift = (top - line.bbox.y) - hu_to_px(vpos - stored_first);
            (drift.abs() > TOLERANCE_PX)
                .then(|| format!("pi={line_para} 줄{line_index} {drift:+.2}px"))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{sample} {}쪽 본문 줄이 쪽 첫 줄 pi={para} 에서 저장 vpos 차만큼 떨어져야 한다: {mismatches:?}",
        page + 1
    );
}

/// 표 칸 안을 제외한 본문 글줄의 (위끝, 문단, 줄, 저장 vpos).
fn body_text_lines(node: &RenderNode, out: &mut Vec<(f64, usize, u32, i32)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => return,
        RenderNodeType::TextLine(line) if line.section_index == Some(0) => {
            if let (Some(para), Some(index), Some(vpos)) =
                (line.para_index, line.line_index, line.vpos)
            {
                out.push((node.bbox.y, para, index, vpos));
            }
        }
        _ => {}
    }
    for child in &node.children {
        body_text_lines(child, out);
    }
}

#[test]
fn hwp_group_bullet_keeps_stored_spacing_at_page_top() {
    let core = document(HWP);
    assert_eq!(core.page_count(), 4, "한컴 engine 2020 Print PDF 는 4쪽");
    assert_group_bullet_page_top(&core, HWP, 1, 8);
    assert_group_bullet_page_top(&core, HWP, 3, 20);
}

#[test]
fn hwpx_group_bullet_keeps_stored_spacing_at_page_top() {
    let core = document(HWPX);
    assert_eq!(core.page_count(), 3, "한컴 engine 2020 Print PDF 는 3쪽");
    assert_group_bullet_page_top(&core, HWPX, 2, 30);
}
