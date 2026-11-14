//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-14T15:04:24+09:00 (독립 Print의 끝 글줄 위치 보완).
//!
//! 입력: `samples/issue9062/ws-host-tac-above-offset-float.hwpx`. 공개 결재문서 36390766(저장 줄
//! 없는 HWPX, 한컴 16쪽)에서 구역 첫 문단과 마지막 「붙임5」 문단 넷(원본 문단 134~137)만 남긴
//! 축소본이다. 원본 문단을 지우기만 했고 남긴 문단의 속성·내용은 그대로다(끝 문단의 짝 없는
//! 누름틀 끝 표지 하나만 뺐다). 원본 전체는 4~8·11·12쪽이 다른 결함으로 90% 미만이라 쓰지 않는다.
//! 기준: 축소본을 한컴 engine 2020 으로 Print 한 `pdf/issue9062/ws-host-tac-above-offset-float-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2, 독립 Print로 확인한 2쪽 「끝.」 글줄 위끝(±2px).
//! - 관계형(1쪽): 문단 1(글 「  」)의 글자처럼 취급 제목 표(컨트롤 1) → 문단 2 「(단위 : 천원)」
//!   → 문단 1 의 자리차지 표(컨트롤 0, 문단 기준 세로 오프셋 6198) 순으로 놓인다. 제목 표
//!   위끝에서 자리차지 표 위끝까지는 원본 오프셋 6198 − 문단 위 간격 500 − 제목 표 바깥 위 여백 140 이다
//!   (한컴 PDF 래스터 74px).
//! - 관계형(2쪽): 이어진 표 조각 뒤에 「끝.」(문단 4)이 온다.
//!
//! 측정(2026-11-14, 코드 커밋 ad794d2cc6(base devel 9e27947967),
//! `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, raw = `silhouette_raw_match_percent`).
//! 글꼴 공급은 두 경로 같게 맑은 고딕(Regular·Bold)·바탕(Windows/Office 글꼴)과 저장소
//! `ttfs/opensource` 다. 문서 face 중 맑은 고딕만 원본이고 HY신명조·HY헤드라인M·한컴돋움·함초롬바탕은
//! 대체 글꼴이다.
//! - Native: 2/2쪽, 1쪽 92.51 · 2쪽 99.12, 전쪽 최저 92.51.
//! - fresh WASM(wasm sha256 4443cee60120af88…): 2/2쪽, 1쪽 98.35 · 2쪽 99.86, 전쪽 최저 98.35.
//! - `ttfs/opensource` 만 공급하면 Native 는 같고(92.51 · 99.12) fresh WASM 은 86.05 · 96.67 이다.
//!   macOS Chrome 이 대체 사다리의 Apple SD Gothic Neo(시스템 글꼴)를 골라 `*` 글리프가 Native 의
//!   Noto Sans KR 보다 약 4px 위에 그려진다. 배치(render tree)는 두 경로가 같다.
//! - 수정 전(devel 9e27947967 의 `src`): 3쪽(한컴 2쪽)이라 쪽수 불일치. 자리차지 표가 1쪽 맨 위에서 시작하고
//!   제목 표는 2쪽 표 뒤, 「(단위 : 천원)」·「끝.」은 3쪽에 놓였다.
//!
//! 증거: `mydocs/pr/assets/issue_9062/`(Native·fresh WASM TSV, 1쪽 review·overlay PNG).
//! 추가 검증(2026-11-14): 통합 후보의 2쪽 raw는 Native 98.76%, fresh WASM 99.86%다.
//! 같은 입력의 독립 한컴 2020 재저장 HWP에서 빈 문단 3과 「끝.」 문단 4의 실제 저장
//! vpos는 68224·70404HU다. 본문 위 여백 5668HU를 더한 「끝.」 줄 위끝은 1014.293px다.
//! 원본 Print와 같은 페이지의 review·overlay에서 잔여 차이가 이 글줄에만 있음을 확인했다.
//! 형상·절대 좌표 검사는 양쪽 raw 98% 이상인 이 2쪽에만 추가한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9062/ws-host-tac-above-offset-float.hwpx";
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

/// 표 밖 본문 글줄(문단, 위끝)과 최상위 표(문단, 컨트롤, 위끝, 아래끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, usize, f64, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let (Some(para), Some(control), false) =
                (table.para_index, table.control_index, in_table)
            {
                tables.push((para, control, node.bbox.y, node.bbox.y + node.bbox.height));
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
    tables: Vec<(usize, usize, f64, f64)>,
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

    fn line_top(&self, para: usize) -> Option<f64> {
        self.lines
            .iter()
            .filter(|(p, _)| *p == para)
            .map(|(_, y)| *y)
            .min_by(|a, b| a.total_cmp(b))
    }

    fn table(&self, para: usize, control: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .find(|(p, c, _, _)| *p == para && *c == control)
            .map(|(_, _, top, bottom)| (*top, *bottom))
    }
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(
        document().page_count(),
        2,
        "한컴 engine 2020 Print PDF 는 2쪽"
    );
}

/// 원본 전제: 문단 1 은 저장 줄이 없고, 양수 오프셋 자리차지 표와 글자처럼 취급 표를 함께 가진다.
#[test]
fn source_host_has_offset_float_and_tac_table_without_stored_lines() {
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::LineSeg;

    let core = document();
    let host = &core.document().sections[0].paragraphs[1];
    assert!(host.text.chars().all(char::is_whitespace) && !host.text.is_empty());
    assert!(host
        .line_segs
        .iter()
        .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0));
    let [Control::Table(float), Control::Table(title)] = host.controls.as_slice() else {
        panic!("문단 1 컨트롤: {}", host.controls.len());
    };
    assert!(!float.common.treat_as_char && float.common.vertical_offset == 6198);
    assert!(title.common.treat_as_char && title.outer_margin_top == 140);
}

/// [#9062] 제목 표 → 틈의 다음 문단 → 오프셋 자리차지 표 순서와 표의 원본 오프셋.
#[test]
fn title_and_gap_paragraph_sit_above_the_offset_float_table() {
    let core = document();
    let page = Page::of(&core, 0);
    let (title_top, title_bottom) = page.table(1, 1).expect("1쪽에 제목 표(컨트롤 1)가 있다");
    let (float_top, _) = page
        .table(1, 0)
        .expect("1쪽에 자리차지 표(컨트롤 0)가 있다");
    let unit_top = page
        .line_top(2)
        .expect("1쪽에 문단 2 「(단위 : 천원)」가 있다");

    assert!(
        title_bottom <= unit_top && unit_top < float_top,
        "제목 표 아래끝 {title_bottom:.2} ≤ 문단 2 위끝 {unit_top:.2} < 자리차지 표 위끝 {float_top:.2}"
    );
    // 자리차지 표의 오프셋 원점은 문단 위 간격(500) 앞의 문단 위끝이고, 제목 표는 그 간격과
    // 바깥 위 여백(140) 뒤에서 시작한다. 한컴 Print PDF 96dpi 래스터에서도 제목 상자 위끝에서
    // 표 위끝까지 74px(= 5558HU)다.
    let gap = float_top - title_top;
    let expected = hu_to_px(6198 - 500 - 140);
    assert!(
        (gap - expected).abs() <= TOLERANCE_PX,
        "제목 표 위끝→자리차지 표 위끝 {gap:.2}px ≠ 원본 오프셋 − 문단 위 간격 − 바깥 위 여백 {expected:.2}px"
    );
}

/// [#9062] 이어진 표 조각 뒤에 「끝.」이 같은 쪽에 온다. 제목 표는 2쪽에 다시 나오지 않는다.
#[test]
fn closing_line_follows_the_continued_table_on_the_second_page() {
    let core = document();
    let page = Page::of(&core, 1);
    let (_, continued_bottom) = page.table(1, 0).expect("2쪽에 이어진 표 조각이 있다");
    let closing = page.line_top(4).expect("2쪽에 문단 4 「끝.」이 있다");
    // 저장 줄이 없는 공백 host의 표가 앞 간격 띠를 이미 차지한 뒤에는,
    // 이어받기 끝에서 빈 후속 문단의 앞 간격을 다시 더하지 않는다.
    let empty = page.line_top(3).expect("2쪽에 빈 후속 문단 3이 있다");
    assert!(
        (empty - continued_bottom).abs() <= TOLERANCE_PX,
        "빈 후속 줄 {empty:.2}는 이어받은 표 하단 {continued_bottom:.2}에서 시작한다"
    );
    let print_closing_top = hu_to_px(5668 + 70404);
    assert!(
        (closing - print_closing_top).abs() <= 2.0,
        "「끝.」 위끝 {closing:.3}px는 독립 한컴 Print의 {print_closing_top:.3}px와 ±2px 안이다"
    );
    assert!(
        continued_bottom <= closing,
        "이어진 표 아래끝 {continued_bottom:.2} ≤ 「끝.」 위끝 {closing:.2}"
    );
    assert!(page.table(1, 1).is_none(), "제목 표는 1쪽에만 있다");
    assert!(page.line_top(2).is_none(), "문단 2 는 1쪽에만 있다");
}
