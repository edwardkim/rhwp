//! [#7398] 상대 크기(relSz)는 글리프에만 곱한다 — 줄 상자는 선언 크기다.
//!
//! `samples/exam_eng.hwp` 는 charPr 다수가 `relSz.latin = 106` 이다. 한/글이 저장한 그
//! 라틴 문단의 LINE_SEG 는 `text_height` 가 선언 크기(base_size)와 같고, 한컴 2022 PDF
//! 의 줄 진행도 저장 vpos 와 같다. 즉 relSz 는 그리는 크기·전진폭에만 들어가고 줄 상자에는
//! 들어가지 않는다.
//!
//! PR #7505(7eddd19da)가 `TextStyle.font_size` 에 relSz 를 곱한 뒤, 렌더 줄 루프의
//! `max_fs` 와 재조판 토큰 크기가 그 값을 줄 상자로 소비해 저장 줄보다 커졌다
//! (2쪽 19번 본문 저장 1494HU = 19.92px → 20.2px, 편지 17.63 → 18.7px).
//!
//! 기대값은 구현이 아니라 **한/글 저장값**에서 온다.
//! - 저장 경로: 2쪽 19번 본문 문단의 렌더 줄 상단 거리 = 저장 vpos 거리(누적).
//! - 재조판 경로: 같은 문단을 편집해 다시 줄 나눔해도 줄 상자(text_height)는 한/글이 같은
//!   문단에 저장한 값(= 선언 크기) 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/exam_eng.hwp";
const PAGE: u32 = 1;
/// 2쪽 오른쪽 단 19번 본문("It was Valentine's Day …", 영문 11줄).
const BODY_19: usize = 112;
const LATIN: usize = 1;
const TOL_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).expect("required fixture samples/exam_eng.hwp");
    DocumentCore::from_bytes(&bytes).expect("exam_eng parse")
}

/// 본문 흐름(표 셀 밖) 문단의 줄 상단들, 위에서 아래로.
fn body_line_tops(node: &RenderNode, para: usize, out: &mut Vec<f64>) {
    match &node.node_type {
        RenderNodeType::Table(_) => return,
        RenderNodeType::TextLine(line) if line.para_index == Some(para) => {
            out.push(node.bbox.y);
        }
        _ => {}
    }
    for child in &node.children {
        body_line_tops(child, para, out);
    }
}

/// 문단 첫 글자모양의 (선언 크기 HU, 라틴 relSz %).
fn first_char_shape(core: &DocumentCore, para: usize) -> (i32, u8) {
    let doc = core.document();
    let p = &doc.sections[0].paragraphs[para];
    let id = p.char_shapes.first().expect("char shape").char_shape_id as usize;
    let cs = &doc.doc_info.char_shapes[id];
    (cs.base_size, cs.relative_sizes[LATIN])
}

#[test]
fn fixture_latin_paragraph_has_relative_size_but_declared_stored_line_box() {
    let core = core();
    let (base, rel) = first_char_shape(&core, BODY_19);
    assert_eq!(rel, 106, "pi{BODY_19} latin relSz");
    let segs = &core.document().sections[0].paragraphs[BODY_19].line_segs;
    assert!(segs.len() >= 10, "pi{BODY_19} stored lines: {}", segs.len());
    for seg in segs {
        assert_eq!(
            seg.text_height, base,
            "Hancom stored line box = declared size"
        );
    }
}

#[test]
fn stored_relative_size_paragraph_keeps_stored_line_pitch() {
    let core = core();
    let root = core.build_page_render_tree(PAGE).expect("page 2 tree").root;
    let mut tops = Vec::new();
    body_line_tops(&root, BODY_19, &mut tops);
    let segs = &core.document().sections[0].paragraphs[BODY_19].line_segs;
    assert_eq!(tops.len(), segs.len(), "every stored line is rendered once");
    let rendered = tops[tops.len() - 1] - tops[0];
    let stored = hu_to_px(segs[segs.len() - 1].vertical_pos - segs[0].vertical_pos);
    assert!(
        (rendered - stored).abs() < TOL_PX,
        "pi{BODY_19} first->last line: rendered {rendered:.2} vs stored {stored:.2} \
         (relSz leaking into the line box grows each line by ~0.29px)"
    );
}

#[test]
fn reflowed_relative_size_paragraph_keeps_declared_line_box() {
    let mut core = core();
    let (base, _) = first_char_shape(&core, BODY_19);
    core.insert_text_native(0, BODY_19, 0, "A")
        .expect("edit pi19 body");
    let segs = &core.document().sections[0].paragraphs[BODY_19].line_segs;
    assert!(!segs.is_empty());
    for (i, seg) in segs.iter().enumerate() {
        assert_eq!(
            seg.text_height, base,
            "reflowed line {i}: text_height {} must stay the declared {base}HU \
             (relSz scales glyphs, not the line box)",
            seg.text_height
        );
    }
}
