//! [#7330] 같은 문단의 문단 기준 자리차지 표가 host 첫 글줄을 민 경우의 **경계**.
//!
//! 글줄 원점은 `max(문단 상단 + 앞 간격, 표 점유 끝)` 이다. 측정(typeset)이 표 점유 끝 위에
//! 남는 앞 간격을 확정해 단 결과에 남기고, 배치(layout)는 같은 값을 확정 앞 간격으로 소비한다.
//! 이 시험은 그 결과가 실제 출력과 쪽 소속에서 한/글과 같은지 본다.
//!
//! 공개 합성 입력(생성기 `samples/float_push_spacing_before/make_float_push_spacing_before_boundaries.py`):
//! - `float_push_spacing_before_boundaries.hwpx`: 생성기 출력 그대로(저장 줄 없음 → 재조판 경로).
//! - `float_push_spacing_before_boundaries.hwp`: 위 HWPX 를 한/글 2020(hwp2024Convert MCP
//!   engine 2020)이 HWP 로 저장한 것 — 저장 LineSeg 는 한/글이 쓴 값이다.
//! - 정본 `pdf/float_push_spacing_before/float_push_spacing_before_boundaries-2020.pdf`
//!   (HWP 의 Print PDF. HWPX 를 같은 엔진으로 출력한 PDF 와 줄 위치가 같다).
//!
//! 형상(쪽):
//! - 1쪽 pi=2(M): 앞 간격 4000HU > 표 점유(한 행 1000HU, 바깥 여백 0). 저장 vpos 7200 =
//!   문단 상단 3200 + 앞 간격 4000 — 글줄은 표 아래가 아니라 문단 상단 + 앞 간격에 선다.
//!   대조: pi=4(같은 앞 간격, 표 없음), pi=6(앞 간격 없는 host).
//! - 2쪽 pi=8(T): 같은 host 가 쪽 나눔 뒤 쪽 맨 위. 저장 vpos 4000 = 앞 간격(쪽 맨 위에서도 남는다).
//!   뒤를 채움 문단 42개로 채워 한/글은 39개를 2쪽에, 나머지를 3쪽에 둔다.
//! - 4쪽 pi=53(L): 앞 간격 1400HU < 표 점유. 표 뒤 글 3줄 — 첫 줄은 표 바깥 아래 여백 바로 밑,
//!   둘째 줄부터는 문단 안의 줄이라 앞 간격과 무관하다.
//!
//! 기대값은 한/글 저장 vpos 와 정본 PDF 의 쪽 소속에서 가져온다. 구현값을 재인용하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HWP: &str = "samples/float_push_spacing_before/float_push_spacing_before_boundaries.hwp";
const HWPX: &str = "samples/float_push_spacing_before/float_push_spacing_before_boundaries.hwpx";
const HU_PER_PX: f64 = 75.0;
/// 정본 PDF 의 쪽 수.
const HANCOM_PAGES: usize = 4;

#[derive(Debug, Clone)]
struct Line {
    para: usize,
    y: f64,
    bottom: f64,
    vpos: Option<i32>,
    text: String,
}

#[derive(Default)]
struct Page {
    body_top: f64,
    body_bottom: f64,
    /// 본문(셀 밖) 표의 (문단, 상단, 하단).
    tables: Vec<(usize, f64, f64)>,
    lines: Vec<Line>,
}

fn collect(node: &RenderNode, in_table: bool, page: &mut Page) {
    let mut in_table = in_table;
    match &node.node_type {
        RenderNodeType::Body { .. } => {
            page.body_top = node.bbox.y;
            page.body_bottom = node.bbox.y + node.bbox.height;
        }
        RenderNodeType::Table(t) => {
            if !in_table {
                page.tables.push((
                    t.para_index.unwrap_or(usize::MAX),
                    node.bbox.y,
                    node.bbox.y + node.bbox.height,
                ));
            }
            in_table = true;
        }
        RenderNodeType::TextLine(tl) if !in_table => {
            if let Some(para) = tl.para_index {
                let text = node
                    .children
                    .iter()
                    .filter_map(|c| match &c.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect::<String>();
                page.lines.push(Line {
                    para,
                    y: node.bbox.y,
                    bottom: node.bbox.y + node.bbox.height,
                    vpos: tl.vpos,
                    text,
                });
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, in_table, page);
    }
}

fn pages(sample: &str) -> Vec<Page> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    (0..core.page_count())
        .map(|i| {
            let tree = core
                .build_page_render_tree(i)
                .unwrap_or_else(|e| panic!("{sample} {}쪽: {e:?}", i + 1));
            let mut page = Page::default();
            collect(&tree.root, false, &mut page);
            page
        })
        .collect()
}

fn first_line(page: &Page, para: usize) -> &Line {
    page.lines
        .iter()
        .find(|l| l.para == para)
        .unwrap_or_else(|| panic!("pi={para} 본문 줄이 이 쪽에 있어야 한다"))
}

fn table(page: &Page, para: usize) -> (f64, f64) {
    page.tables
        .iter()
        .find(|(p, _, _)| *p == para)
        .map(|(_, top, bottom)| (*top, *bottom))
        .unwrap_or_else(|| panic!("pi={para} 본문 표가 이 쪽에 있어야 한다"))
}

/// 한/글이 저장한 쪽 상대 vpos 가 곧 본문 상단 기준 글줄 위치다(이 입력의 각 쪽은 vpos 0 에서 시작).
fn stored_y(page: &Page, line: &Line) -> f64 {
    let vpos = line
        .vpos
        .unwrap_or_else(|| panic!("pi={} 저장 vpos 가 있어야 한다", line.para));
    page.body_top + f64::from(vpos) / HU_PER_PX
}

#[test]
fn spacing_before_larger_than_float_occupancy_is_measured_from_paragraph_top() {
    let pages = pages(HWP);
    let p1 = &pages[0];
    let host = first_line(p1, 2);
    let (table_top, table_bottom) = table(p1, 2);
    let para_top = first_line(p1, 1).y + 1600.0 / HU_PER_PX;
    assert!(
        (table_top - para_top).abs() < 1.0,
        "입력 전제: 표는 문단 상단에 붙는다: table_top={table_top:.1}, para_top={para_top:.1}"
    );
    assert!(
        host.y > table_bottom + 1.0,
        "입력 전제: 앞 간격이 표 점유보다 커서 글줄은 표 아래에서 떨어진다"
    );
    let expected = stored_y(p1, host);
    assert!(
        (host.y - expected).abs() < 1.0,
        "pi=2 첫 글줄 = 문단 상단 + 앞 간격(표 점유 끝에 앞 간격을 더하지 않는다): \
         y={:.1}, 한/글 저장 vpos 기준 {expected:.1}",
        host.y
    );
}

#[test]
fn column_top_host_keeps_spacing_before_larger_than_float_occupancy() {
    let pages = pages(HWP);
    let p2 = &pages[1];
    let host = first_line(p2, 8);
    let (table_top, _) = table(p2, 8);
    assert!(
        (table_top - p2.body_top).abs() < 1.0,
        "입력 전제: 2쪽 맨 위 표: table_top={table_top:.1}, body_top={:.1}",
        p2.body_top
    );
    assert_eq!(
        host.vpos,
        Some(4000),
        "입력 전제: 한/글 저장 vpos = 앞 간격"
    );
    let expected = stored_y(p2, host);
    assert!(
        (host.y - expected).abs() < 1.0,
        "쪽 맨 위 host 첫 글줄도 문단 상단 + 앞 간격: y={:.1}, 저장 vpos 기준 {expected:.1}",
        host.y
    );
}

#[test]
fn multi_line_host_only_first_line_consumes_paragraph_spacing() {
    let pages = pages(HWP);
    let p4 = &pages[3];
    let host_lines: Vec<&Line> = p4.lines.iter().filter(|l| l.para == 53).collect();
    assert_eq!(host_lines.len(), 3, "입력 전제: 표 뒤 host 글 3줄");
    for line in &host_lines {
        let expected = stored_y(p4, line);
        assert!(
            (line.y - expected).abs() < 1.0,
            "pi=53 줄 '{}': y={:.1}, 저장 vpos 기준 {expected:.1}",
            line.text,
            line.y
        );
    }
    let (_, table_bottom) = table(p4, 53);
    let gap = host_lines[0].y - table_bottom;
    assert!(
        (gap - 852.0 / HU_PER_PX).abs() < 1.0,
        "앞 간격(1400HU)이 표 점유보다 작으면 첫 글줄은 표 바깥 아래 여백 바로 밑: gap={gap:.1}"
    );
}

#[test]
fn controls_without_pushing_float_keep_their_spacing() {
    let pages = pages(HWP);
    let p1 = &pages[0];
    // 표 없는 같은 앞 간격 문단(pi=4), 앞 간격 없는 host(pi=6)와 그 뒤 — 바로 앞 문단과의
    // 거리가 저장 vpos 간격 그대로다(앞 문단의 위치와 무관하게 이 문단들의 간격만 본다).
    for para in [4, 5, 6, 7] {
        let (prev, line) = (first_line(p1, para - 1), first_line(p1, para));
        let stored = f64::from(line.vpos.unwrap() - prev.vpos.unwrap()) / HU_PER_PX;
        assert!(
            (line.y - prev.y - stored).abs() < 1.0,
            "pi={para}: 앞 문단과 {:.1}px, 저장 vpos 간격 {stored:.1}px",
            line.y - prev.y
        );
    }
    assert!(
        first_line(p1, 4).vpos.unwrap() - first_line(p1, 3).vpos.unwrap() > 4000,
        "입력 전제: 표 없는 대조 문단의 저장 간격은 앞 간격을 포함한다"
    );
    let (_, table_bottom) = table(p1, 6);
    assert!(
        (first_line(p1, 6).y - table_bottom).abs() < 1.0,
        "앞 간격 없는 host 첫 글줄은 표 바로 밑(바깥 여백 0)"
    );
}

/// 측정과 배치가 같은 첫 글줄 원점을 소비하는지 쪽 소속과 본문 하단으로 본다.
/// 저장 줄이 없는 HWPX 는 저장 사다리 스냅 없이 측정이 쪽 나눔을 정하므로, 측정이 앞 간격을
/// 빠뜨리면 2쪽에 채움 문단이 더 남고 그 줄이 본문 아래로 넘친다.
fn assert_page_ownership_matches_hancom(sample: &str) {
    let pages = pages(sample);
    assert_eq!(pages.len(), HANCOM_PAGES, "{sample}: 정본 쪽 수");
    let fillers = |page: &Page| -> Vec<String> {
        page.lines
            .iter()
            .filter(|l| l.text.starts_with("T 채움 "))
            .map(|l| l.text.clone())
            .collect()
    };
    let p2 = fillers(&pages[1]);
    let p3 = fillers(&pages[2]);
    assert_eq!(
        (p2.len(), p2.last().map(String::as_str)),
        (39, Some("T 채움 39")),
        "{sample}: 정본 2쪽은 채움 1~39"
    );
    assert_eq!(
        p3,
        vec!["T 채움 40", "T 채움 41", "T 채움 42"],
        "{sample}: 정본 3쪽은 채움 40~42"
    );
    for (i, page) in pages.iter().enumerate() {
        for line in &page.lines {
            assert!(
                line.bottom <= page.body_bottom + 0.5,
                "{sample} {}쪽 '{}' 하단 {:.1} 이 본문 하단 {:.1} 을 넘는다",
                i + 1,
                line.text,
                line.bottom,
                page.body_bottom
            );
        }
    }
}

#[test]
fn reflow_measurement_reserves_the_painted_first_line_origin() {
    assert_page_ownership_matches_hancom(HWPX);
}

#[test]
fn stored_measurement_reserves_the_painted_first_line_origin() {
    assert_page_ownership_matches_hancom(HWP);
}

#[test]
fn reflow_output_matches_hancom_saved_layout() {
    // 같은 입력을 한/글이 저장한 HWP 의 저장 vpos 가 재조판 경로의 독립 기대값이다.
    let saved = pages(HWP);
    let reflow = pages(HWPX);
    assert_eq!(reflow.len(), saved.len(), "쪽 수");
    for (i, (s, r)) in saved.iter().zip(&reflow).enumerate() {
        let st: Vec<&str> = s.lines.iter().map(|l| l.text.as_str()).collect();
        let rt: Vec<&str> = r.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(rt, st, "{}쪽 본문 줄 소속·순서", i + 1);
        for (sl, rl) in s.lines.iter().zip(&r.lines) {
            let expected = stored_y(s, sl);
            assert!(
                (rl.y - expected).abs() < 1.0,
                "{}쪽 '{}': 재조판 y={:.1}, 한/글 저장 vpos 기준 {expected:.1}",
                i + 1,
                rl.text,
                rl.y
            );
        }
    }
}
