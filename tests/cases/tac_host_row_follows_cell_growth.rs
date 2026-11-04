//! 관계형: 칸 글이 늘어난 글자처럼 취급 표는 제자리에서 아래로 자라고, 뒤 표는 그만큼 내려가며, 저장·다시 열기 뒤에도 같다.
//! 주석 갱신: 2026-11-04 12:20 KST(+09:00).
//! 측정: 2026-11-04 한/글 12.30(Mac)에서 `samples/footnote-01.hwp` 제목 표 둘째 문단 앞에 "표안한글"을 넣었다.
//! 한/글은 제목 표 윗변을 그대로 두고 표를 한 줄만큼 아래로 키웠으며, 제목 표와 다음 개요 표 사이 빈 자리도
//! 그대로 두었다(같은 편집을 한 rhwp 저장본을 한/글로 열어도 같다). 한/글이 저장한 파일은 표 sz 6942→8362,
//! 표 줄(둘째 줄) lh·th 7508→8928, 첫 줄 lh 7508→8928(th 1600 유지), 뒤 문단 vpos 14028→15448이었다.
//! 수정 전 rhwp는 편집 직후 제목 표를 첫 줄 자리(2560HU 위)로 올리고 개요 표를 그대로 두었으며, 저장본을
//! 다시 열면 개요 표가 58.7px 올라와 제목 표와 겹쳤다. 쪽·픽셀 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/footnote-01.hwp";
const TYPED: &str = "표안한글";
const TOLERANCE: f64 = 0.5;

/// 쪽 1의 제목 표(문단 0)와 개요 표(문단 1)의 (윗변, 아랫변).
#[derive(Debug, Clone, Copy)]
struct Tables {
    title: (f64, f64),
    outline: (f64, f64),
}

fn collect(node: &RenderNode, out: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.cell_context.is_none() {
            if let Some(para) = table.para_index {
                out.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn tables(doc: &DocumentCore) -> Tables {
    let tree = doc.build_page_render_tree(0).expect("쪽 1 렌더 트리");
    let mut found = Vec::new();
    collect(&tree.root, &mut found);
    let find = |para: usize| {
        found
            .iter()
            .find(|(owner, _, _)| *owner == para)
            .map(|(_, top, bottom)| (*top, *bottom))
            .unwrap_or_else(|| panic!("쪽 1에 문단 {para}의 표가 없다: {found:?}"))
    };
    Tables {
        title: find(0),
        outline: find(1),
    }
}

fn title_control(doc: &DocumentCore) -> usize {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .expect("제목 표")
}

fn close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "{what}: {actual:.2} != {expected:.2}"
    );
}

fn assert_same(actual: Tables, expected: Tables, what: &str) {
    close(
        actual.title.0,
        expected.title.0,
        &format!("{what} 제목 표 윗변"),
    );
    close(
        actual.title.1,
        expected.title.1,
        &format!("{what} 제목 표 아랫변"),
    );
    close(
        actual.outline.0,
        expected.outline.0,
        &format!("{what} 개요 표 윗변"),
    );
}

fn open(bytes: &[u8]) -> DocumentCore {
    DocumentCore::from_bytes(bytes).expect("문서 열기")
}

#[test]
fn grown_title_table_keeps_its_top_and_pushes_the_next_table_across_save() {
    let mut doc = open(&std::fs::read(SAMPLE).expect("견본"));
    let before = tables(&doc);
    let gap = before.outline.0 - before.title.1;
    assert!(gap > 10.0, "견본의 두 표 사이에 빈 자리가 있다: {before:?}");

    let control = title_control(&doc);
    doc.insert_text_in_cell_native(0, 0, control, 0, 1, 0, TYPED)
        .expect("제목 표 칸에 입력");
    let typed = tables(&doc);

    // 한/글처럼 윗변은 그대로, 아래로만 자란다.
    close(typed.title.0, before.title.0, "입력 뒤 제목 표 윗변");
    assert!(
        typed.title.1 > before.title.1 + 10.0,
        "칸에 한 줄이 늘어 표가 자란다: {before:?} -> {typed:?}"
    );
    // 뒤 표는 자란 만큼 내려가 빈 자리를 지킨다.
    close(typed.outline.0 - typed.title.1, gap, "입력 뒤 두 표 사이");

    // 저장한 파일을 다시 열어도 같은 자리다.
    let hwp = open(&doc.export_hwp_native().expect("HWP 저장"));
    assert_same(tables(&hwp), typed, "HWP 다시 열기");
    let hwpx = open(&doc.export_hwpx_native().expect("HWPX 저장"));
    assert_same(tables(&hwpx), typed, "HWPX 다시 열기");

    // 넣은 글을 지우면 처음 자리로 돌아온다.
    doc.delete_text_in_cell_native(0, 0, control, 0, 1, 0, TYPED.chars().count())
        .expect("넣은 글 지우기");
    assert_same(tables(&doc), before, "지운 뒤");
    let restored = open(&doc.export_hwp_native().expect("지운 뒤 저장"));
    assert_same(tables(&restored), before, "지운 뒤 다시 열기");
}

#[test]
fn saved_host_row_holds_the_table_it_draws() {
    let mut doc = open(&std::fs::read(SAMPLE).expect("견본"));
    let control = title_control(&doc);
    let rows = |doc: &DocumentCore| doc.document().sections[0].paragraphs[0].line_segs.clone();
    let next_gap = |doc: &DocumentCore| {
        let paragraphs = &doc.document().sections[0].paragraphs;
        let row = paragraphs[0].line_segs.last().expect("표 줄");
        let next = paragraphs[1].line_segs.first().expect("뒤 문단 첫 줄");
        next.vertical_pos - (row.vertical_pos + row.text_height + row.line_spacing)
    };
    let before = rows(&doc);
    let gap = next_gap(&doc);
    doc.insert_text_in_cell_native(0, 0, control, 0, 1, 0, TYPED)
        .expect("제목 표 칸에 입력");

    let saved = open(&doc.export_hwp_native().expect("HWP 저장"));
    let Some(Control::Table(table)) = saved.document().sections[0].paragraphs[0]
        .controls
        .get(control)
    else {
        panic!("저장본의 제목 표");
    };
    let margins = f64::from(table.outer_margin_top) + f64::from(table.outer_margin_bottom);
    let after = rows(&saved);
    let row = after.last().expect("표 줄");
    // 표가 놓인 저장 줄은 다시 연 문서가 그린 표와 바깥 여백을 담는다.
    let drawn = tables(&saved).title;
    let dpi = 96.0;
    close(
        f64::from(row.text_height) * dpi / 7200.0,
        drawn.1 - drawn.0 + margins * dpi / 7200.0,
        "표 줄 높이와 그린 표 외곽",
    );
    // 표 앞 줄은 다시 짜지 않는다.
    assert_eq!(after.len(), before.len(), "저장 줄 수");
    assert_eq!(after[0].vertical_pos, before[0].vertical_pos, "첫 줄 위치");
    assert_eq!(after[0].text_height, before[0].text_height, "첫 줄 높이");
    assert_eq!(
        row.vertical_pos,
        before.last().unwrap().vertical_pos,
        "표 줄 위치"
    );
    // 표 줄 끝에서 뒤 문단까지의 저장 간격도 그대로다.
    assert_eq!(next_gap(&saved), gap, "표 줄 끝과 뒤 문단 사이");
}
