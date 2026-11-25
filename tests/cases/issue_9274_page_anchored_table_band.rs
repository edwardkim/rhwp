//! 관계형·고정형 — [#9274] 용지 기준 자리차지 표는 띠다: host 뒤 문단은 띠 위를 먼저 채우고, 띠에 닿는 줄부터 띠 아래로 간다.
//! 주석 갱신: 2026-11-22 13:57 KST (새 파일)
//!
//! 입력: #9272 fixture `samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx`
//! (산업통상자원부 입법예고 공개 문서, HWPX, 한컴오피스 2020 저장, 7쪽).
//! 기준: 한컴 Print PDF `pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2020.pdf`(7쪽).
//!
//! 7쪽 문단 43 은 용지 기준·세로 가운데(오프셋 0) 자리차지 표(3×2 「의안 소관 부서명」)의 host
//! 이고 저장 `vpos=2560` 이다. 한/글은 빈 문단 44~51 을 저장 `vpos` 5120~23040(2560HU 간격)으로
//! 표 띠 위에 채우고, 띠에 닿는 문단 52 부터 띠 아래(43051~63531)에 둔다. 마지막 줄 끝
//! 64531HU 는 본문 높이 70012HU 안이다.
//! 수정 전 rhwp 는 host 에서 흐름을 표 아래까지 한 번에 내려 문단 44~51 을 띠 아래에 두고,
//! 문단 52~60 을 본문 아래로 밀어 쪽 밖(1225.6px)까지 그렸다.
//! - 고정형: 전체 쪽수 7(= PDF).
//! - 관계형: 7쪽에서 문단 44~51 의 줄은 맨 위 문단 42 의 줄 아래, 표 위끝 위에 위→아래 순서로 겹치지 않고,
//!   문단 52~60 의 줄은 표 아래끝 아래에 같은 순서로 놓이며 본문 상자 안에 든다.
//!
//! 반례: `issue_8836_absolute_float_host_spacing` — host 줄 자체가 띠에 닿으면 띠 아래로 뛰는 동작은
//! 그대로 남는다. 그림 띠는 `issue_7696_page_anchored_topbottom_band` 가 지킨다.
//!
//! 측정(2026-11-22, 이 파일과 같은 커밋의 코드, base = #9278 head `083da55560`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 7쪽 / 7쪽, 7쪽 99.59(base 99.59 — 옮긴 줄이 빈 줄이라 실루엣이 같다), 전쪽 최저 97.38(3쪽,
//!   한양신명조 대체 글꼴의 글자 모양 차이만, 줄 위치·줄 나눔은 같다)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 7쪽 / 7쪽, 7쪽 99.82, 전쪽 최저 99.82(7쪽)
//! - 7쪽 빈 줄 위치는 문단부호 표시 비교 `p7_para_marks_base_vs_head.png` 로 확인했다.
//! - 증거: `mydocs/pr/assets/issue_9274/`(native·wasm·base 전쪽 TSV, 7쪽 review·overlay PNG, 문단부호 비교 PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 7;
/// 7쪽(0 기반 6).
const PAGE: u32 = 6;
/// 7쪽 맨 위 줄이 되는 빈 문단(표 host 앞).
const PAGE_FIRST_EMPTY_PARA: usize = 42;
/// 용지 기준 세로 가운데 자리차지 표(3×2)의 host 문단.
const BAND_TABLE_PARA: usize = 43;
/// 한/글이 표 띠 위에 채우는 빈 문단.
const ABOVE_BAND_PARAS: std::ops::RangeInclusive<usize> = 44..=51;
/// 띠에 닿아 띠 아래로 가는 빈 문단.
const BELOW_BAND_PARAS: std::ops::RangeInclusive<usize> = 52..=60;

struct LineBox {
    para: usize,
    bbox: BoundingBox,
}

#[derive(Default)]
struct PageBoxes {
    body: Option<BoundingBox>,
    tables: Vec<(Option<usize>, BoundingBox)>,
    lines: Vec<LineBox>,
}

/// 본문 흐름의 표와 줄을 모은다. 표 안 칸의 줄은 본문 문단 번호가 아니므로 내려가지 않는다.
fn collect(node: &RenderNode, out: &mut PageBoxes) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            out.tables.push((table.para_index, node.bbox));
            return;
        }
        RenderNodeType::Header | RenderNodeType::Footer => return,
        RenderNodeType::Body { .. } => out.body = Some(node.bbox),
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                out.lines.push(LineBox {
                    para,
                    bbox: node.bbox,
                });
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page_boxes() -> PageBoxes {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    let page = core.build_page_render_tree(PAGE).expect("7쪽");
    let mut out = PageBoxes::default();
    collect(&page.root, &mut out);
    out
}

/// 문단의 7쪽 줄 상자 하나. 빈 문단은 줄 하나를 차지한다.
fn only_line(boxes: &PageBoxes, para: usize) -> BoundingBox {
    let owned: Vec<_> = boxes.lines.iter().filter(|l| l.para == para).collect();
    assert_eq!(
        owned.len(),
        1,
        "빈 문단 {para} 는 7쪽에서 줄 상자 하나를 차지해야 한다"
    );
    owned[0].bbox
}

/// 줄들이 위→아래 순서로 겹치지 않는지 확인하고 마지막 줄 아래끝을 돌려준다.
fn assert_stacked(boxes: &PageBoxes, paras: std::ops::RangeInclusive<usize>, from: f64) -> f64 {
    let mut previous_bottom = from;
    for para in paras {
        let line = only_line(boxes, para);
        assert!(
            line.y + 0.5 >= previous_bottom,
            "빈 문단 {para} 의 줄 위 {:.2} 가 앞 줄 아래 {previous_bottom:.2} 보다 위다",
            line.y
        );
        previous_bottom = line.y + line.height;
    }
    previous_bottom
}

fn band_table(boxes: &PageBoxes) -> BoundingBox {
    let tables: Vec<_> = boxes
        .tables
        .iter()
        .filter(|(para, _)| *para == Some(BAND_TABLE_PARA))
        .collect();
    assert_eq!(tables.len(), 1, "문단 43 의 표는 7쪽에 한 번 그려진다");
    tables[0].1
}

#[test]
fn paragraphs_after_the_host_fill_the_space_above_the_band() {
    let boxes = page_boxes();
    let table = band_table(&boxes);
    let first = only_line(&boxes, PAGE_FIRST_EMPTY_PARA);
    let above_bottom = assert_stacked(&boxes, ABOVE_BAND_PARAS, first.y + first.height);
    assert!(
        above_bottom <= table.y + 0.5,
        "문단 51 의 줄 아래 {above_bottom:.2} 가 표 띠 위끝 {:.2} 를 넘는다 — 띠 위 문단이 표 아래로 밀렸다",
        table.y
    );
}

#[test]
fn the_line_that_meets_the_band_continues_below_it_inside_the_body() {
    let boxes = page_boxes();
    let table = band_table(&boxes);
    let body = boxes.body.expect("7쪽 본문 상자");
    let first_below = only_line(&boxes, *BELOW_BAND_PARAS.start());
    assert!(
        first_below.y + 0.5 >= table.y + table.height,
        "띠에 닿는 문단 52 의 줄 위 {:.2} 는 표 띠 아래끝 {:.2} 아래여야 한다",
        first_below.y,
        table.y + table.height
    );
    let last_bottom = assert_stacked(&boxes, BELOW_BAND_PARAS, table.y + table.height);
    assert!(
        last_bottom <= body.y + body.height + 0.5,
        "문단 60 의 줄 아래 {last_bottom:.2} 가 본문 아래끝 {:.2} 를 넘는다",
        body.y + body.height
    );
}
