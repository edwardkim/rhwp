//! #8582: 호환 대상이 MS 워드인 문서의 **표 칸 안** 저장 줄 없는 문단도 본문과 같은 배율로
//! 줄을 세운다.
//!
//! 재현물 `samples/issue8582/{hwp,word}_cellparas.hwpx` 는 `samples/test-image.hwpx` 에서 호환 대상만
//! 바꾼 문서다. 1행 2열 표의 오른쪽 칸에 한 줄짜리 11pt 문단 5개(`C0`~`C4`, 함초롬바탕, 비율 100%),
//! 표 뒤 본문에 같은 문단 5개(`B0`~`B4`)를 둔다. 생성: `make_8582_more.py`(rhwp-review/tools 의
//! `make_b2_fixture.py` 조합 방식, 저장 줄 없음).
//!
//! 기준: 데스크톱 한컴 2024 인쇄 PDF(Hancom PDF 드라이버)의 글자 기준선 간격(HU)
//! - `hwp_cellparas`: 칸 1100, 본문 1100
//! - `word_cellparas`: 칸 1860, 본문 1860 (= 1100 × 1.691). 글자처럼 취급한 표의 줄은 표 아래에
//!   끝 표지의 아래쪽(465 HU)이 붙어 `C4` → `B0` 가 2472 HU(= 1860 + 아래 칸 여백 141 + 471)다.
//!
//! 수정 전 rhwp 는 워드 문서의 칸 문단만 배율 없이 1100 HU 로 그렸다(칸 높이는 배율로 재어
//! 칸 아래가 비었다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("#8582 재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

/// 글자 런의 (글자, 줄 위끝 px). 칸 안 런과 본문 런을 나눠 모은다.
fn runs(
    node: &RenderNode,
    line_top: f64,
    cell: &mut Vec<(String, f64)>,
    body: &mut Vec<(String, f64)>,
) {
    let line_top = match &node.node_type {
        RenderNodeType::TextLine(_) => node.bbox.y,
        _ => line_top,
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.text.trim().to_string();
        if text.len() == 2 && (text.starts_with('C') || text.starts_with('B')) {
            if run.cell_context.is_some() {
                cell.push((text, line_top));
            } else {
                body.push((text, line_top));
            }
        }
    }
    for child in &node.children {
        runs(child, line_top, cell, body);
    }
}

fn pitches(name: &str) -> (f64, f64, f64) {
    let core = load(name);
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut cell, mut body) = (Vec::new(), Vec::new());
    runs(&tree.root, 0.0, &mut cell, &mut body);
    assert_eq!(cell.len(), 5, "{name}: 칸 문단 5개 {cell:?}");
    assert_eq!(body.len(), 5, "{name}: 본문 문단 5개 {body:?}");
    let pitch = |rows: &[(String, f64)]| (rows[4].1 - rows[0].1) / 4.0 * HU_PER_PX;
    let gap = (body[0].1 - cell[4].1) * HU_PER_PX;
    (pitch(&cell), pitch(&body), gap)
}

#[test]
fn hangul_target_cell_paragraph_pitch_is_font_size() {
    let (cell, body, _) = pitches("hwp_cellparas");
    assert!(
        (cell - 1100.0).abs() <= 10.0,
        "한글 문서 칸 피치 {cell:.0} HU, 데스크톱 한컴 1100"
    );
    assert!(
        (body - 1100.0).abs() <= 10.0,
        "한글 문서 본문 피치 {body:.0} HU, 데스크톱 한컴 1100"
    );
}

#[test]
fn word_target_cell_paragraph_pitch_uses_font_factor() {
    let (cell, body, gap) = pitches("word_cellparas");
    assert!(
        (body - 1860.0).abs() <= 12.0,
        "워드 문서 본문 피치 {body:.0} HU, 데스크톱 한컴 1860"
    );
    assert!(
        (cell - 1860.0).abs() <= 12.0,
        "워드 문서 칸 피치 {cell:.0} HU, 데스크톱 한컴 1860"
    );
    assert!(
        (gap - 2472.0).abs() <= 24.0,
        "워드 문서 C4→B0 {gap:.0} HU, 데스크톱 한컴 2472 (표 줄 아래 끝 표지 아래쪽)"
    );
}
