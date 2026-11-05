//! 관계형 회귀: 셀 내용 증가 전후 표 윗변·뒤 표·다음 쪽 내용과 저장 재열기의 일관성. 비교 문서 간 관계를 검사한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! footnote-01.hwp (8572-original-footnote-hwp): 전 6쪽, Native 최저 100.0%, fresh WASM 최저 99.99572%.
//! fix-typed.hwp (8572-fix-typed-hwp-2022): 전 7쪽, Native 최저 100.0%, fresh WASM 최저 99.99572%.
//! actual-api-cell-typed.hwpx (api-8572-cell-typed-hwpx): 전 7쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 칸 글이 늘어난 글자처럼 취급 표는 제자리에서 아래로 자라고, 뒤 표는 그만큼 내려가며, 저장·다시 열기 뒤에도 같다.
//! 주석 갱신: 2026-11-04 16:10 KST(+09:00).
//! 측정: 2026-11-04 한/글 12.30(Mac)에서 `samples/footnote-01.hwp` 제목 표 둘째 문단 앞에 "표안한글"을 넣었다.
//! 한/글은 제목 표 윗변을 그대로 두고 표를 한 줄만큼 아래로 키웠으며, 제목 표와 다음 개요 표 사이 빈 자리도
//! 그대로 두었다(같은 편집을 한 rhwp 저장본을 한/글로 열어도 같다). 한/글이 저장한 파일은 표 sz 6942→8362,
//! 표 줄(둘째 줄) lh·th 7508→8928, 첫 줄 lh 7508→8928(th 1600 유지), 뒤 문단 vpos 14028→15448이었다.
//! 수정 전 rhwp는 편집 직후 제목 표를 첫 줄 자리(2560HU 위)로 올리고 개요 표를 그대로 두었으며, 저장본을
//! 다시 열면 개요 표가 58.7px 올라와 제목 표와 겹쳤다.
//! 쪽 나눔: 원본은 한/글에서 6쪽이고 1쪽이 "- 3D 프린터로…" 문단(문단 10)과 각주 둘로 끝난다. 편집 뒤에는
//! 그 문단이 각주 위에 들어가지 않아 한/글이 2쪽 맨 위로 넘기고 7쪽이 된다(rhwp 저장본을 한/글로 열어도
//! "3/7쪽"으로 같다). 한/글 저장본은 그 문단 첫 줄 vpos를 300(문단 위 간격)으로 적는다. 수정 전 rhwp는 편집
//! 화면이 7쪽인데 저장본을 다시 열면 6쪽이었다(옮긴 vpos가 앞 쪽에 이어 있어 저장 사다리를 따르는 조판이
//! 그 문단을 1쪽에 남겼다). 쪽·픽셀 좌표는 고정하지 않는다.
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

/// 쪽마다 그 쪽에서 시작하거나 이어지는 본문 문단 번호.
fn page_paragraphs(doc: &DocumentCore) -> Vec<Vec<usize>> {
    let pages = doc.dump_page_items_json(None);
    pages
        .as_array()
        .expect("쪽 목록")
        .iter()
        .map(|page| {
            let mut paras: Vec<usize> = page["columns"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|column| column["items"].as_array().into_iter().flatten())
                .filter_map(|item| item["paraIndex"].as_u64())
                .map(|index| index as usize)
                .collect();
            paras.dedup();
            paras
        })
        .collect()
}

fn page_of(pages: &[Vec<usize>], para: usize) -> usize {
    pages
        .iter()
        .position(|paras| paras.contains(&para))
        .unwrap_or_else(|| panic!("문단 {para}이 어느 쪽에도 없다: {pages:?}"))
}

/// 편집이 옮긴 흐름은 첫 명시적 쪽 나눔 앞까지다.
fn first_page_break(doc: &DocumentCore) -> usize {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .position(|para| para.column_type == rhwp::model::paragraph::ColumnBreakType::Page)
        .expect("명시적 쪽 나눔")
}

fn assert_same_pages(actual: &DocumentCore, expected: &[Vec<usize>], limit: usize, what: &str) {
    let pages = page_paragraphs(actual);
    assert_eq!(pages.len(), expected.len(), "{what} 쪽 수: {pages:?}");
    for para in 0..limit {
        assert_eq!(
            page_of(&pages, para),
            page_of(expected, para),
            "{what} 문단 {para}의 쪽"
        );
    }
}

#[test]
fn page_that_takes_the_pushed_paragraph_survives_save() {
    let mut doc = open(&std::fs::read(SAMPLE).expect("견본"));
    let original = page_paragraphs(&doc);
    let limit = first_page_break(&doc);
    let last = limit - 1;
    assert_eq!(
        page_of(&original, last),
        page_of(&original, last - 1),
        "견본에서는 문단 {last}이 앞 문단과 같은 쪽에 있다"
    );

    let control = title_control(&doc);
    doc.insert_text_in_cell_native(0, 0, control, 0, 1, 0, TYPED)
        .expect("제목 표 칸에 입력");
    let typed = page_paragraphs(&doc);
    // 한/글처럼 마지막 문단이 각주 위에 들어가지 않아 다음 쪽 맨 위로 간다.
    assert_eq!(
        page_of(&typed, last),
        page_of(&typed, last - 1) + 1,
        "편집 뒤 문단 {last}은 다음 쪽으로: {typed:?}"
    );
    assert_eq!(typed.len(), original.len() + 1, "편집 뒤 쪽 수: {typed:?}");

    // 저장본은 그 쪽 시작을 저장 줄에 담아 다시 열어도 같은 쪽에 둔다.
    let hwp = open(&doc.export_hwp_native().expect("HWP 저장"));
    assert_same_pages(&hwp, &typed, limit, "HWP 다시 열기");
    let hwpx = open(&doc.export_hwpx_native().expect("HWPX 저장"));
    assert_same_pages(&hwpx, &typed, limit, "HWPX 다시 열기");
    let paragraphs = &hwp.document().sections[0].paragraphs;
    let previous_end = paragraphs[last - 1]
        .line_segs
        .last()
        .map(|row| row.vertical_pos + row.line_height + row.line_spacing)
        .expect("앞 문단 줄");
    assert!(
        paragraphs[last].line_segs[0].vertical_pos < previous_end,
        "저장본의 문단 {last} 첫 줄은 새 쪽 원점에서 다시 시작한다"
    );

    // 편집 중 저장 줄은 앞 쪽에 이어 둔다. 쪽 원점은 저장본에만 적는다.
    let live = &doc.document().sections[0].paragraphs;
    assert!(
        live[last].line_segs[0].vertical_pos
            >= live[last - 1]
                .line_segs
                .last()
                .map(|row| row.vertical_pos + row.line_height + row.line_spacing)
                .expect("앞 문단 줄"),
        "편집 중 문단 {last} 저장 줄은 앞 문단에 이어 있다"
    );

    // 글을 지우면 그 문단이 앞 쪽으로 돌아오고 저장해도 같다.
    doc.delete_text_in_cell_native(0, 0, control, 0, 1, 0, TYPED.chars().count())
        .expect("넣은 글 지우기");
    assert_same_pages(&doc, &original, limit, "지운 뒤");
    let restored = open(&doc.export_hwp_native().expect("지운 뒤 저장"));
    assert_same_pages(&restored, &original, limit, "지운 뒤 다시 열기");
}
