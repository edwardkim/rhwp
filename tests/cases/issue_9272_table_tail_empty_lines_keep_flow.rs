//! 관계형·고정형 — [#9272] 나뉜 표 끝 조각 바로 아래의 빈 줄은 그 쪽에서 줄 상자를 차지하고, 되감긴 다음 표는 다음 쪽에 놓인다.
//! 주석 갱신: 2026-11-21 09:13 KST (새 파일)
//!
//! 입력: 산업통상자원부 입법예고 공개 문서(「불공정무역행위조사 및 산업피해구제에 관한 법률 시행령
//! 일부개정령(안)」, HWPX, 한컴오피스 2020 저장, 7쪽)
//! `samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx`.
//! 기준: 한컴 Print PDF `pdf/issue9272/trade-remedy-decree-table-tail-empty-lines-2020.pdf`(7쪽).
//!
//! 문단 31 은 15×2 RowBreak 신구조문 대비표이고 6쪽에 행 12..15 끝 조각이 584.2px 에서 끝난다.
//! 문단 32~42 는 빈 문단 11개이며 저장 `vpos` 가 43813HU(= 584.2px)부터 2560HU 간격이다 — 표 끝
//! 조각 바로 아래에 조판된 줄이다. 문단 43 은 용지 기준 자리차지 표(3×2 「의안 소관 부서명」)의
//! host 이고 저장 `vpos=2560` 으로 새 쪽에서 한 줄 아래다. 본문 높이 70012HU 에 문단 41(66853+1600)
//! 까지 들고 문단 42(69413+1600)는 7쪽 맨 위 줄이 된다.
//! 수정 전 rhwp 는 뒤 앵커(문단 43)가 되감겼다는 이유로 문단 32~42 를 옛 좌표의 안내 줄로 보고
//! 높이 없이 감춰, 문단 43 의 표가 6쪽 대비표 위에 겹치고 7쪽이 비었다.
//! - 고정형: 전체 쪽수 7(= PDF).
//! - 관계형: 6쪽에 문단 32~41 의 줄 상자가 하나씩 있고 대비표 끝 조각 아래에서 위→아래 순서로
//!   겹치지 않으며 쪽 상자 안에 있다. 문단 43 의 표는 6쪽에 없고 7쪽에 있으며, 7쪽에서 문단 42 의
//!   줄이 문단 44 의 줄보다 위에 있다.
//!
//! 측정(2026-11-21, 이 파일과 같은 커밋의 코드, base = upstream/devel `2480af27df`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 7쪽 / 7쪽, 6쪽 98.66(base 85.58), 7쪽 99.59(base 0.62), 전쪽 최저 97.38(3쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 7쪽 / 7쪽, 6쪽 99.94, 7쪽 99.82, 전쪽 최저 99.82(7쪽)
//! - 증거: `mydocs/pr/assets/issue_9272/`(native·wasm 전쪽 TSV, 6·7쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9272/trade-remedy-decree-table-tail-empty-lines.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 7;
/// 쪽을 넘어온 신구조문 대비표(15×2)의 문단.
const SPLIT_TABLE_PARA: usize = 31;
/// 대비표 끝 조각 아래 6쪽에 남는 빈 문단.
const KEPT_EMPTY_PARAS: std::ops::RangeInclusive<usize> = 32..=41;
/// 7쪽 맨 위 줄이 되는 빈 문단.
const NEXT_PAGE_FIRST_EMPTY_PARA: usize = 42;
/// 용지 기준 자리차지 표(3×2)의 host 문단.
const REWOUND_TABLE_PARA: usize = 43;
/// 문단 43 다음의 빈 문단.
const AFTER_TABLE_EMPTY_PARA: usize = 44;

struct TableBox {
    para: Option<usize>,
    bbox: BoundingBox,
    text: String,
}

struct LineBox {
    para: usize,
    bbox: BoundingBox,
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

/// 본문 흐름의 표와 줄을 모은다. 표 안 칸의 줄은 본문 문단 번호가 아니므로 내려가지 않는다.
fn collect_body(node: &RenderNode, tables: &mut Vec<TableBox>, lines: &mut Vec<LineBox>) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            let mut text = String::new();
            collect_text(node, &mut text);
            tables.push(TableBox {
                para: table.para_index,
                bbox: node.bbox,
                text,
            });
            return;
        }
        RenderNodeType::Header | RenderNodeType::Footer => return,
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                lines.push(LineBox {
                    para,
                    bbox: node.bbox,
                });
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect_body(child, tables, lines);
    }
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    core
}

#[test]
fn empty_lines_right_below_the_table_tail_keep_their_line_boxes() {
    let core = open();
    let page = core.build_page_render_tree(5).expect("6쪽");
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    collect_body(&page.root, &mut tables, &mut lines);

    let tail = tables
        .iter()
        .find(|t| t.para == Some(SPLIT_TABLE_PARA))
        .expect("6쪽에 문단 31 대비표의 끝 조각이 있어야 한다");
    assert!(
        tables.iter().all(|t| t.para != Some(REWOUND_TABLE_PARA)),
        "문단 43 의 표는 6쪽에 그려지지 않아야 한다"
    );

    let page_box = page.root.bbox;
    let mut previous_bottom = tail.bbox.y + tail.bbox.height;
    for para in KEPT_EMPTY_PARAS {
        let owned: Vec<_> = lines.iter().filter(|line| line.para == para).collect();
        assert_eq!(
            owned.len(),
            1,
            "빈 문단 {para} 는 6쪽에서 줄 상자 하나를 차지해야 한다"
        );
        let line = owned[0];
        assert!(
            line.bbox.y + 0.5 >= previous_bottom,
            "빈 문단 {para} 의 줄 위 {:.2} 가 앞 내용 아래 {previous_bottom:.2} 보다 위다",
            line.bbox.y
        );
        let bottom = line.bbox.y + line.bbox.height;
        assert!(
            bottom <= page_box.y + page_box.height,
            "빈 문단 {para} 의 줄 아래 {bottom:.2} 가 쪽 상자를 넘는다"
        );
        previous_bottom = bottom;
    }
    assert!(
        lines
            .iter()
            .all(|line| line.para != NEXT_PAGE_FIRST_EMPTY_PARA),
        "빈 문단 42 는 6쪽 본문에 들지 않으므로 6쪽에 줄을 두지 않아야 한다"
    );
}

#[test]
fn rewound_table_after_the_empty_lines_starts_on_the_next_page() {
    let core = open();
    let page = core.build_page_render_tree(6).expect("7쪽");
    let (mut tables, mut lines) = (Vec::new(), Vec::new());
    collect_body(&page.root, &mut tables, &mut lines);

    let rewound: Vec<_> = tables
        .iter()
        .filter(|t| t.para == Some(REWOUND_TABLE_PARA))
        .collect();
    assert_eq!(
        rewound.len(),
        1,
        "문단 43 의 표는 7쪽에 정확히 한 번 그려져야 한다"
    );
    assert!(
        rewound[0].text.contains("산업통상자원부") && rewound[0].text.contains("연"),
        "문단 43 표의 칸 글이 7쪽에 그려져야 한다"
    );
    assert!(
        tables.iter().all(|t| t.para != Some(SPLIT_TABLE_PARA)),
        "문단 31 대비표는 6쪽에서 끝나 7쪽에 조각이 없어야 한다"
    );

    let line_top = |para: usize| {
        lines
            .iter()
            .find(|line| line.para == para)
            .map(|line| line.bbox.y)
            .unwrap_or_else(|| panic!("빈 문단 {para} 의 줄이 7쪽에 있어야 한다"))
    };
    let first = line_top(NEXT_PAGE_FIRST_EMPTY_PARA);
    let after = line_top(AFTER_TABLE_EMPTY_PARA);
    assert!(
        first < after,
        "7쪽에서 빈 문단 42 의 줄 {first:.2} 은 문단 44 의 줄 {after:.2} 보다 위여야 한다"
    );
}
