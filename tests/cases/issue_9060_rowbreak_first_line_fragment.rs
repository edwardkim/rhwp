//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-14T14:30:00+09:00 (새 파일, 직전 파일 커밋 없음).
//!
//! 입력: 공개 입법예고 의견 86036 「(규제영향분석서) 공공재정 부정청구 금지 및 부정이익 환수 등에
//! 관한 법률 일부개정법률(안)」(한컴오피스 2020 저장 HWPX, 14쪽)
//! `samples/issue9060/86036-rowbreak-first-line-fragment.hwpx`.
//! 기준: 한컴 engine 2020 Print `pdf/issue9060/86036-rowbreak-first-line-fragment-2020.pdf`(14쪽).
//! - 고정형: 전체 쪽수 14.
//! - 관계형([#9060]): 문단 41 의 1×1 RowBreak 자리차지 표는 5쪽 아래와 6쪽 위에 나뉘어 놓인다.
//!   칸 문단의 저장 줄은 vpos `0, 0, 2252` 로 첫 줄 뒤에서 쪽을 다시 시작한다. 5쪽 조각은
//!   「3. 규제목표」(문단 39) 아래에 있고, 6쪽은 나머지 조각으로 시작한다.
//!
//! 측정(2026-11-14, 코드 커밋 ddcc876814(base devel 5f78bb8d02),
//! `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, raw = `silhouette_raw_match_percent`).
//! 글꼴 공급은 두 경로 같게 맑은 고딕(Regular·Bold)·바탕(Windows/Office 글꼴)과 저장소
//! `ttfs/opensource` 다.
//! - Native: 14/14쪽, 5쪽 99.20 · 6쪽 99.26, 전쪽 최저 95.47(4쪽).
//! - fresh WASM(wasm sha256 e8fb83753c35abb2…): 14/14쪽, 5쪽 100.00 · 6쪽 100.00, 전쪽 최저 99.54(4쪽).
//! - 수정 전(devel 5f78bb8d02): 15쪽(한컴 14쪽)이라 쪽수 불일치.
//!
//! 증거: `mydocs/pr/assets/issue_9060/`(Native·fresh WASM TSV, 5·6쪽 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9060/86036-rowbreak-first-line-fragment.hwpx";
const TABLE_PARA: usize = 41;

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

/// 표 밖 본문 글줄(문단, 위끝)과 최상위 표(문단, 위끝, 아래끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, f64, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index.filter(|_| !in_table) {
                tables.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
            }
            inside = true;
        }
        RenderNodeType::TextLine(line) if !in_table => {
            if let Some(para) = line.para_index {
                lines.push((para, node.bbox.y));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside, lines, tables);
    }
}

struct Page {
    lines: Vec<(usize, f64)>,
    tables: Vec<(usize, f64, f64)>,
}

impl Page {
    fn of(core: &DocumentCore, page: u32) -> Self {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
        let (mut lines, mut tables) = (Vec::new(), Vec::new());
        collect(&tree.root, false, &mut lines, &mut tables);
        Self { lines, tables }
    }

    fn table(&self, para: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .find(|(p, _, _)| *p == para)
            .map(|(_, top, bottom)| (*top, *bottom))
    }

    fn line_top(&self, para: usize) -> Option<f64> {
        self.lines
            .iter()
            .filter(|(p, _)| *p == para)
            .map(|(_, y)| *y)
            .min_by(|a, b| a.total_cmp(b))
    }
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(
        document().page_count(),
        14,
        "한컴 engine 2020 Print PDF 는 14쪽"
    );
}

/// 원본 전제: 칸 문단의 저장 줄이 첫 줄 뒤에서 쪽을 다시 시작한다.
#[test]
fn source_cell_lines_restart_the_page_after_the_first_line() {
    let core = document();
    let host = &core.document().sections[0].paragraphs[TABLE_PARA];
    let [Control::Table(table)] = host.controls.as_slice() else {
        panic!("문단 {TABLE_PARA} 컨트롤: {}", host.controls.len());
    };
    assert_eq!((table.row_count, table.col_count), (1, 1));
    assert!(!table.common.treat_as_char);
    let vpos: Vec<i32> = table.cells[0].paragraphs[0]
        .line_segs
        .iter()
        .map(|seg| seg.vertical_pos)
        .collect();
    assert_eq!(vpos, [0, 0, 2252]);
}

/// [#9060] 첫 줄 조각은 5쪽 아래에, 나머지는 6쪽 위에 놓인다.
#[test]
fn first_line_fragment_stays_at_the_bottom_of_the_page() {
    let core = document();
    let fifth = Page::of(&core, 4);
    let (first_top, _) = fifth
        .table(TABLE_PARA)
        .expect("5쪽에 문단 41 표의 첫 조각이 있다");
    let heading = fifth
        .line_top(39)
        .expect("5쪽에 문단 39 「3. 규제목표」가 있다");
    assert!(
        heading < first_top,
        "「3. 규제목표」 위끝 {heading:.2} < 표 첫 조각 위끝 {first_top:.2}"
    );

    let sixth = Page::of(&core, 5);
    let (rest_top, _) = sixth
        .table(TABLE_PARA)
        .expect("6쪽에 문단 41 표의 나머지 조각이 있다");
    // 나머지 조각이 6쪽 본문을 연다: 표 앞 문단(39·40)은 5쪽에 남고 6쪽에서 표 위에 오지 않는다.
    assert!(
        sixth.line_top(39).is_none(),
        "「3. 규제목표」는 5쪽에만 있다"
    );
    assert!(
        sixth
            .lines
            .iter()
            .filter(|(para, _)| *para > TABLE_PARA)
            .all(|(_, y)| rest_top <= *y),
        "6쪽의 표 뒤 본문 글줄은 표 나머지 조각 아래에 있다"
    );
    assert!(
        Page::of(&core, 6).table(TABLE_PARA).is_none(),
        "표는 7쪽까지 이어지지 않는다"
    );
}
