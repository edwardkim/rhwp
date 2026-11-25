//! 관계형·고정형 — [#8735] 이어지는 표 조각 뒤·단 첫머리의 문단 기준 표 host 는 저장 첫 줄 − 문단 위 간격을 문단 상단으로 쓴다.
//! 주석 갱신: 2026-11-25 KST_TIME KST (처음 작성)
//!
//! 입력: `samples/issue8732/1741000-202200045_second_interim_report.hwp`
//! (기준 `pdf/issue8732/1741000-202200045_second_interim_report-2020.pdf`, 138쪽).
//! - 66쪽: 앞 문단 pi825 표의 이어지는 조각 뒤 빈 host pi826(14×4 표, 문단 기준 오프셋 1089,
//!   바깥 위 여백 283, 문단 위 간격 1000). 저장 첫 줄 17616 = 조각 점유 끝이고, 한컴 표 윗변은
//!   17616 − 1000 + 1089 + 283 이다(PDF 머리 행 괘선 414~415px − 첫 행 43.55px). 수정 전 rhwp 는
//!   조각 끝을 문단 상단으로 써 표가 13.3px 아래였고 쪽 바닥 행 하나가 67쪽으로 밀렸다(65.75).
//! - 134쪽: 같은 형상의 pi1826(1×1 표, 오프셋 1668). 한컴 표 윗변 = 13048 − 1000 + 1668 + 283
//!   (PDF 괘선 318px). 수정 전 40.65.
//! - 73쪽: 단을 여는 빈 host pi887(17×12 표, 오프셋 0, 바깥 여백 141)의 저장 줄 1000 = 문단 위
//!   간격, 다음 문단 pi888 줄 40580 = 0 + 141 + 40298 + 141 이라 단의 저장 원점은 0 이다.
//!   수정 전 rhwp 는 원점을 1000 으로 읽어 뒤 문단이 13.3px 위였다(86.29).
//! - 자리차지 표는 앞 조각 띠와 겹치지 않아 표 상자는 저장 첫 줄 위로 가지 않는다(10k 반례
//!   19087555 4쪽 pi3: 앞 간격 500·오프셋 0, 표 윗변 = 저장 줄 + 바깥 위 여백, 한/글 Print 99.46).
//! - 관계형: 표 윗변 = 본문 위끝 + 저장 첫 줄 + max(오프셋 − 문단 위 간격, 0) + 바깥 위 여백,
//!   뒤 문단 줄 간격 = 저장 vpos 차. 고정형: 같은 위치를 독립 저장값으로 ±2px 고정(세 쪽 모두
//!   Native·fresh WASM 98 이상, 독립 PDF 확대 대조 완료).
//!
//! 측정: 2026-11-25, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! (`silhouette_raw_match_percent`, 색상 경계 보정 전 2px 내용 실루엣), 글꼴은 맑은 고딕·바탕 +
//! `ttfs/opensource` 를 Native·WASM 두 경로에 공급. source MEASURE_SHA(devel `0fbee2c04e` 위).
//! - 66쪽 Native N66·WASM W66, 134쪽 Native N134·WASM W134, 73쪽 Native N73·WASM W73.
//! - 전체 138쪽(기준 PDF 와 같음). 전쪽 최저 Native NMIN·WASM WMIN.
//! - 증거: `mydocs/pr/assets/issue_8735/{native,wasm,base-native}/second_interim_report_silhouette.tsv`, 대표 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8732/1741000-202200045_second_interim_report.hwp";
const PX_PER_HU: f64 = 96.0 / 7200.0;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 열기")
}

#[derive(Default)]
struct PageBoxes {
    body: Option<BoundingBox>,
    tables: Vec<(Option<usize>, BoundingBox)>,
    lines: Vec<(Option<usize>, BoundingBox)>,
}

fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Body { .. } => out.body = Some(node.bbox),
        RenderNodeType::Table(table) => {
            out.tables.push((table.para_index, node.bbox));
            return;
        }
        RenderNodeType::Header
        | RenderNodeType::Footer
        | RenderNodeType::MasterPage
        | RenderNodeType::FootnoteArea => return,
        RenderNodeType::TextLine(line) => out.lines.push((line.para_index, node.bbox)),
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page_boxes(core: &DocumentCore, page: u32) -> PageBoxes {
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut out = PageBoxes::default();
    collect(&tree.root, &mut out);
    out
}

fn find(items: &[(Option<usize>, BoundingBox)], para: usize, what: &str) -> BoundingBox {
    items
        .iter()
        .find(|(p, _)| *p == Some(para))
        .map(|(_, b)| *b)
        .unwrap_or_else(|| panic!("문단 {para} 의 {what} 이 있어야 한다"))
}

fn table_of(core: &DocumentCore, para: usize) -> &rhwp::model::table::Table {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {para} 에 표가 있어야 한다"))
}

fn stored_vpos(core: &DocumentCore, para: usize) -> i32 {
    core.document().sections[0].paragraphs[para].line_segs[0].vertical_pos
}

/// 원본 문단 위 간격(HU). HWP5 문단 모양은 2배 단위로 저장한다.
fn spacing_before_hu(core: &DocumentCore, para: usize) -> i32 {
    let doc = core.document();
    let shape = doc.sections[0].paragraphs[para].para_shape_id as usize;
    doc.doc_info.para_shapes[shape].spacing_before / 2
}

/// 이어지는 표 조각 바로 뒤 빈 host 의 표 윗변 = 본문 위끝 + 저장 첫 줄 − 문단 위 간격
/// + 세로 오프셋 + 바깥 위 여백.
fn assert_split_successor_table_top(page: u32, previous: usize, host: usize) {
    let core = core();
    let boxes = page_boxes(&core, page);
    assert!(
        boxes.tables.iter().any(|(p, _)| *p == Some(previous)),
        "전제: {page}쪽 첫머리에 앞 문단 {previous} 표의 이어지는 조각이 있다"
    );
    let table = table_of(&core, host);
    let before = spacing_before_hu(&core, host);
    assert!(before > 0, "전제: host 문단에 문단 위 간격이 있다");
    assert!(
        table.common.vertical_offset > 0,
        "전제: 표에 세로 오프셋이 있다"
    );
    let body = boxes.body.expect("본문 영역");
    let flow_end = boxes
        .tables
        .iter()
        .find(|(p, _)| *p == Some(previous))
        .map(|(_, b)| b.y + b.height)
        .unwrap()
        + f64::from(table_of(&core, previous).outer_margin_bottom) * PX_PER_HU;
    let line_top = body.y + f64::from(stored_vpos(&core, host)) * PX_PER_HU;
    assert!(
        (flow_end - line_top).abs() <= 0.5,
        "전제: 저장 첫 줄 {line_top:.2} 은 조각 점유 끝 {flow_end:.2} 이다"
    );
    // 자리차지 표는 앞 조각 띠와 겹치지 않으므로 표 상자는 저장 첫 줄 위로 올라가지 않는다.
    let expected = line_top
        + f64::from(
            (table.common.vertical_offset as i32 - before).max(0)
                + i32::from(table.outer_margin_top),
        ) * PX_PER_HU;
    let actual = find(&boxes.tables, host, "표");
    assert!(
        (actual.y - expected).abs() <= 2.0,
        "{page}쪽 문단 {host} 표 윗변 {:.2} 은 저장 첫 줄 − 문단 위 간격 + 오프셋 + 위 여백 {expected:.2} 이다",
        actual.y
    );
}

#[test]
fn table_after_continuation_fragment_measures_offset_from_paragraph_top_p66() {
    assert_split_successor_table_top(65, 825, 826);
}

#[test]
fn table_after_continuation_fragment_measures_offset_from_paragraph_top_p134() {
    assert_split_successor_table_top(133, 1825, 1826);
}

/// 73쪽: 단을 여는 빈 host 의 저장 줄이 원점 + 문단 위 간격이면 뒤 문단은 원점 0 기준 저장 자리다.
#[test]
fn column_opening_table_host_stored_line_after_spacing_keeps_zero_origin_p73() {
    const PAGE: u32 = 72;
    const HOST: usize = 887;
    let core = core();
    let table = table_of(&core, HOST);
    assert_eq!(table.common.vertical_offset, 0, "전제: 오프셋 없는 표");
    assert_eq!(
        stored_vpos(&core, HOST),
        spacing_before_hu(&core, HOST),
        "전제: host 저장 줄 = 문단 위 간격"
    );
    assert_eq!(
        i64::from(stored_vpos(&core, HOST + 1)),
        i64::from(table.outer_margin_top)
            + i64::from(table.common.height)
            + i64::from(table.outer_margin_bottom),
        "전제: 다음 줄 = 원점 0 + 표 상자"
    );
    let boxes = page_boxes(&core, PAGE);
    let body = boxes.body.expect("본문 영역");
    for para in [HOST + 1, HOST + 2] {
        let line = find(&boxes.lines, para, "줄");
        let expected = body.y + f64::from(stored_vpos(&core, para)) * PX_PER_HU;
        assert!(
            (line.y - expected).abs() <= 2.0,
            "문단 {para} 첫 줄 위끝 {:.2} 은 본문 위끝 + 저장 vpos {expected:.2} 다",
            line.y
        );
    }
    let gap = find(&boxes.lines, HOST + 2, "줄").y - find(&boxes.lines, HOST + 1, "줄").y;
    let stored_gap =
        f64::from(stored_vpos(&core, HOST + 2) - stored_vpos(&core, HOST + 1)) * PX_PER_HU;
    assert!(
        (gap - stored_gap).abs() <= 0.5,
        "문단 {} → {} 줄 간격 {gap:.2} 은 저장 vpos 차 {stored_gap:.2} 다",
        HOST + 1,
        HOST + 2
    );
}
