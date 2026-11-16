//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-14T00:30:00+09:00 (새 파일, 직전 파일 커밋 없음).
//!
//! 입력: 공개 결재문서 36400525(한컴오피스 2020 저장 HWPX, 4쪽)
//! `samples/issue9055/36400525-page-band-empty-host-line.hwpx`.
//! 기준: 한컴 engine 2020 Print `pdf/issue9055/36400525-page-band-empty-host-line-2020.pdf`(4쪽).
//! - 고정형: 전체 쪽수 4.
//! - 관계형([#9055], 1쪽): 쪽 기준 자리차지 머리 표(문단 0)의 빈 host 줄은 띠 아래의 독립 줄이다.
//!   문단 1 위끝 = 띠 아래끝 + host 줄 전진(`lh 1000 + ls 600`), 그 뒤 빈 문단들과 쪽 아래
//!   글자처럼 취급 표(문단 15)는 저장 vpos 차이를 그대로 따른다.
//! - 관계형([#9056], 2·3쪽): 저장 줄이 없는 글자처럼 취급 표 문단(16·17, 34) 뒤의 원본 줄은
//!   단 위끝에서 저장 vpos 만큼 떨어져 있다(표 줄 뒤 간격 전량).
//!
//! 측정(2026-11-14, 코드 커밋 a1a041e5fb(base devel 9e27947967),
//! `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, raw = `silhouette_raw_match_percent`).
//! 글꼴 공급은 두 경로 같게 맑은 고딕(Regular·Bold)·바탕(Windows/Office 글꼴)과 저장소
//! `ttfs/opensource` 다. 문서 face(HY중고딕 등)는 대체 글꼴이다.
//! - Native: 4/4쪽, 1쪽 95.81 · 2쪽 97.96 · 3쪽 96.02 · 4쪽 99.94, 전쪽 최저 95.81.
//! - fresh WASM(wasm sha256 91107a4d5167a5b2…): 4/4쪽, 1쪽 97.75 · 2쪽 93.57 · 3쪽 93.82 ·
//!   4쪽 99.90, 전쪽 최저 93.57.
//! - `ttfs/opensource` 만 공급하면 fresh WASM 2·3쪽이 89.36 · 88.39 다(이전 head 측정). macOS
//!   Chrome 이 대체 사다리의 Apple SD Gothic Neo 를 골라 `*` 글리프가 Native 와 약 4px 어긋난다.
//!   render tree 는 두 경로가 같다.
//! - 수정 전(devel dd7f453898) Native: 1쪽 81.97 · 2쪽 60.35 · 3쪽 28.53 · 4쪽 99.94.
//!
//! 증거: `mydocs/pr/assets/issue_9055/`(Native·fresh WASM·수정 전 TSV, 1·3쪽 review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9055/36400525-page-band-empty-host-line.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

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

    fn line_top(&self, para: usize) -> f64 {
        self.lines
            .iter()
            .filter(|(p, _)| *p == para)
            .map(|(_, y)| *y)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or_else(|| panic!("문단 {para} 글줄 없음: {:?}", self.lines))
    }

    fn table(&self, para: usize) -> (f64, f64) {
        self.tables
            .iter()
            .find(|(p, _, _)| *p == para)
            .map(|(_, top, bottom)| (*top, *bottom))
            .unwrap_or_else(|| panic!("문단 {para} 표 없음: {:?}", self.tables))
    }
}

fn first_seg(core: &DocumentCore, para: usize) -> LineSeg {
    core.document().sections[0].paragraphs[para]
        .line_segs
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("문단 {para} 줄 없음"))
}

fn is_stored(seg: &LineSeg) -> bool {
    seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
}

fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= TOLERANCE_PX,
        "{what}: {actual:.2}px ≠ {expected:.2}px"
    );
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(
        document().page_count(),
        4,
        "한컴 engine 2020 Print PDF 는 4쪽"
    );
}

/// [#9055] 쪽 기준 자리차지 표 띠에 걸린 빈 host 줄은 띠 아래에서 줄높이 + 줄간격을 차지한다.
#[test]
fn empty_host_line_sits_below_page_relative_band() {
    let core = document();
    let host = first_seg(&core, 0);
    let next = first_seg(&core, 1);
    // 원본 전제: 띠 아래끝 26732(= 오프셋 2 + 바깥 위 138 + 높이 26454 + 바깥 아래 138) 안의
    // host 저장 줄, 다음 문단은 띠 아래끝 + host 줄 전진.
    assert!(is_stored(&host) && is_stored(&next), "원본 전제: 저장 줄");
    assert_eq!(
        (host.vertical_pos, host.line_height, host.line_spacing),
        (0, 1000, 600)
    );
    assert_eq!(next.vertical_pos, 26732 + 1000 + 600);

    let page = Page::of(&core, 0);
    let (_, band_table_bottom) = page.table(0);
    let band_bottom = band_table_bottom + hu_to_px(138);
    assert_close(
        page.line_top(1) - band_bottom,
        hu_to_px(host.line_height + host.line_spacing),
        "띠 아래끝→문단 1 위끝(host 줄 전진)",
    );
    // 띠 아래 빈 문단들은 저장 사다리대로 이어진다.
    for para in 1..14 {
        assert_close(
            page.line_top(para + 1) - page.line_top(para),
            hu_to_px(first_seg(&core, para + 1).vertical_pos - first_seg(&core, para).vertical_pos),
            &format!("문단 {para}→{} 위끝 차이", para + 1),
        );
    }
    // 쪽 아래 글자처럼 취급 표(문단 15)도 같은 축: 표 위끝 = 저장 줄 위끝 + 바깥 위 여백 140.
    let (table_top, _) = page.table(15);
    assert_close(
        table_top - page.line_top(14),
        hu_to_px(first_seg(&core, 15).vertical_pos - first_seg(&core, 14).vertical_pos + 140),
        "문단 14 위끝→문단 15 표 위끝",
    );
}

/// [#9056] 단 중간의 저장 줄 없는 표 줄 뒤 원본 줄은 단 위끝에서 저장 vpos 만큼 떨어진다.
#[test]
fn stored_line_after_reflowed_tac_rows_keeps_full_trailing_gap() {
    let core = document();
    for para in [16, 17] {
        assert!(
            !is_stored(&first_seg(&core, para)),
            "원본 전제: 문단 {para} 는 저장 줄 없는 표 문단"
        );
    }
    let stored = first_seg(&core, 18);
    assert!(is_stored(&stored), "원본 전제: 문단 18 은 저장 줄");
    // 원본 전제: 두 표 줄의 `lh + ls` 합이 다음 원본 줄의 쪽 로컬 vpos 다.
    let advance: i32 = [16, 17]
        .iter()
        .map(|&para| {
            let seg = first_seg(&core, para);
            seg.line_height + seg.line_spacing
        })
        .sum();
    assert_eq!(stored.vertical_pos, advance);

    let page = Page::of(&core, 1);
    // 단 위끝 = 첫 표 위끝 − 바깥 위 여백 141.
    let column_top = page.table(16).0 - hu_to_px(141);
    assert_close(
        page.line_top(18) - column_top,
        hu_to_px(stored.vertical_pos),
        "2쪽 단 위끝→문단 18 위끝",
    );
    assert!(
        page.table(17).1 <= page.line_top(18),
        "문단 17 표는 문단 18 글줄 위에 있다"
    );
}

/// [#9056] 3쪽: 저장 줄 표(문단 33) 뒤 저장 줄 없는 표(문단 34), 그 뒤 원본 줄(문단 35).
#[test]
fn third_page_stored_line_follows_reflowed_table_row() {
    let core = document();
    assert!(is_stored(&first_seg(&core, 33)) && !is_stored(&first_seg(&core, 34)));
    let stored = first_seg(&core, 35);
    assert!(is_stored(&stored));
    assert_eq!(
        first_seg(&core, 33).vertical_pos,
        0,
        "원본 전제: 문단 33 이 쪽을 연다"
    );

    let page = Page::of(&core, 2);
    // 단 위끝 = 문단 33 표 위끝 − 바깥 위 여백 283.
    let column_top = page.table(33).0 - hu_to_px(283);
    assert_close(
        page.line_top(35) - column_top,
        hu_to_px(stored.vertical_pos),
        "3쪽 단 위끝→문단 35 위끝",
    );
    assert!(page.table(34).1 <= page.line_top(35));
}
