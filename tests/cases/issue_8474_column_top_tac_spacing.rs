//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8489의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:41:06+09:00 (`d1a9e1a71bdce6e90288691326377a8bd651b6a6`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8474/section_top_spacing-2020.hwp`; 독립 Print `pdf/issue8474/section_top_spacing-2020.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 8쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8474/section_top_spacing.hwpx`; 독립 Print `pdf/issue8474/section_top_spacing-2020.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 8쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8474/top_tac_cases-2020.hwp`; 독립 Print `pdf/issue8474/top_tac_cases-2020.pdf`.
//! 전체 4쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8474/top_tac_cases.hwpx`; 독립 Print `pdf/issue8474/top_tac_cases-2020.pdf`.
//! 전체 4쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8470/42565_입법예고문.hwp`; 독립 Print `pdf/issue8470/42565_입법예고문-2020.pdf`.
//! 전체 16쪽, 전쪽 최저 Native 94.92% / fresh WASM 96.96%.
//! 주장 5쪽 raw: Native 99.27% / fresh WASM 99.24%.
//! 주장 13쪽 raw: Native 99.88% / fresh WASM 99.84%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8489_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 02:20 KST (신규 파일).
//! 2026-11-02 측정 raw(`silhouette_raw_match_percent`, 96dpi, PDFium, `--compat 2022`, 한/글 2020 엔진 Print 정본, head `cf8947160d`):
//! section_top_spacing-2020.hwp: 8쪽(S4 표) Native 100.00% / fresh WASM 100.00%; 전8쪽 최저 100.00% / 100.00%.
//! top_tac_cases-2020.hwp: 3쪽(T 표) Native 100.00% / fresh WASM 100.00%; 4쪽(U 표) 100.00% / 100.00%; 전4쪽 최저 100.00% / 100.00%.
//! 근거: mydocs/pr/assets/issue_8474/{native,wasm}/{section_top_spacing,top_tac_cases}_silhouette.tsv,
//! mydocs/pr/assets/issue_8474/png/{section_top_spacing_p8,top_tac_cases_p3}_{review,overlay}_{native,wasm}.png.
//!
//! #8474 쪽 맨 위의 글자처럼 취급 표 host 도 저장 첫 줄이 증언하는 문단 위 간격을 받는다.
//!
//! 독립 기준은 같은 HWP 의 한/글 2020 엔진 Print PDF(`pdf/issue8474/section_top_spacing-2020.pdf`)다.
//! 입력은 `samples/issue8474/gen_section_top_spacing.py` 로 만든 HWPX 를 한/글 2020 이 HWP 로 저장한
//! `section_top_spacing-2020.hwp` 이고(저장 줄 사다리 경로), 원 HWPX 도 함께 둔다.
//! 한/글은 쪽 맨 위에서도 문단 위 간격을 적용한다 — 저장 첫 줄 vpos 가 위 간격과 같고
//! (쪽 나누기·자연 쪽 넘김·구역 첫 쪽 모두), 정본의 첫 줄 위끝이 위 간격만큼 내려간다.
//! - S1·S2·S3: 구역 첫 문단 글줄, 위 간격 500·1000·1500HU(원본 저장값 1000·2000·3000).
//! - S4: 구역 첫 문단이 위 간격 1000HU 의 글자처럼 취급 표. 수정 전 rhwp 는 이 표만 위 간격을
//!   버려 본문 위끝에 붙였다(정본 대비 13.3px 위, docs#333 입법예고문 5·13쪽과 같은 꼴).
//!
//! 검사는 본문 위끝에서 잰 첫 내용의 거리 관계다 — S4 표 위끝 = S2 첫 줄 위끝 + 표 바깥 위 여백,
//! S1 < S2 < S3 의 위 간격 순서, 표 뒤 문단은 표 아래. 절대 좌표는 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn line_text(node: &RenderNode) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            text.push_str(&run.text);
        }
        stack.extend(n.children.iter().rev());
    }
    text.trim().to_string()
}

#[derive(Default)]
struct PageProbe {
    body_top: Option<f64>,
    /// (글줄 위끝, 글줄 아래끝, 글자) — 본문 단의 글줄만.
    lines: Vec<(f64, f64, String)>,
    /// 본문 단 최상위 표의 (위끝, 아래끝).
    tables: Vec<(f64, f64)>,
}

fn probe(node: &RenderNode, in_table: bool, out: &mut PageProbe) {
    match &node.node_type {
        RenderNodeType::Body { .. } => out.body_top = Some(node.bbox.y),
        RenderNodeType::Table(_) if !in_table => {
            out.tables
                .push((node.bbox.y, node.bbox.y + node.bbox.height));
            for child in &node.children {
                probe(child, true, out);
            }
            return;
        }
        RenderNodeType::TextLine(_) if !in_table => {
            out.lines
                .push((node.bbox.y, node.bbox.y + node.bbox.height, line_text(node)));
            return;
        }
        _ => {}
    }
    for child in &node.children {
        probe(child, in_table, out);
    }
}

fn pages(core: &DocumentCore) -> Vec<PageProbe> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            let mut out = PageProbe::default();
            probe(&tree.root, false, &mut out);
            out
        })
        .collect()
}

/// `text` 로 시작하는 글줄이 첫 내용인 쪽에서 본문 위끝부터 그 줄 위끝까지의 거리.
fn first_line_offset(pages: &[PageProbe], text: &str) -> f64 {
    let page = pages
        .iter()
        .find(|p| p.lines.first().is_some_and(|(_, _, t)| t.starts_with(text)))
        .unwrap_or_else(|| panic!("`{text}` 로 시작하는 쪽이 없다"));
    page.lines[0].0 - page.body_top.expect("본문 영역")
}

#[test]
fn column_top_tac_table_keeps_stored_spacing_before() {
    let core = open("samples/issue8474/section_top_spacing-2020.hwp");
    let pages = pages(&core);
    assert_eq!(pages.len(), 8, "독립 PDF 8쪽");

    let s1 = first_line_offset(&pages, "S1 첫");
    let s2 = first_line_offset(&pages, "S2 첫");
    let s3 = first_line_offset(&pages, "S3 첫");
    let p = first_line_offset(&pages, "P 쪽나누기");
    // 위 간격 500 < 1000 < 1500HU 순서와 같은 간격(쪽 나누기 = 구역 첫 = 1000HU).
    assert!(
        s1 < s2 && s2 < s3,
        "구역 첫 글줄 위 간격 순서: {s1:.2} {s2:.2} {s3:.2}"
    );
    assert!(
        (s2 - p).abs() <= 0.5,
        "같은 위 간격이면 같은 자리: S2 {s2:.2} / 쪽 나누기 {p:.2}"
    );
    assert!(
        ((s3 - s2) - (s2 - s1)).abs() <= 0.5,
        "위 간격 500HU 마다 같은 거리만큼 내려간다: {s1:.2} {s2:.2} {s3:.2}"
    );

    // S4: 구역 첫 문단의 글자처럼 취급 표. 위 간격이 S2 와 같으므로 표 위끝 = S2 첫 줄 위끝 +
    // 표 바깥 위 여백이다.
    let s4 = pages
        .iter()
        .find(|p| p.lines.iter().any(|(_, _, t)| t == "S4 둘째"))
        .expect("S4 쪽");
    let body_top = s4.body_top.expect("본문 영역");
    let &(table_top, table_bottom) = s4.tables.first().expect("S4 표");
    let outer_top = core.document().sections[4].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table.outer_margin_top),
            _ => None,
        })
        .expect("S4 표 컨트롤");
    let outer_top_px = f64::from(outer_top) * 96.0 / 7200.0;
    assert!(
        ((table_top - body_top) - (s2 + outer_top_px)).abs() <= 0.5,
        "S4 표 위끝 {:.2} = S2 첫 줄 위끝 {s2:.2} + 바깥 위 여백 {outer_top_px:.2}",
        table_top - body_top
    );
    let after = s4
        .lines
        .iter()
        .find(|(_, _, t)| t == "S4 둘째")
        .expect("S4 둘째");
    assert!(after.0 >= table_bottom - 0.5, "표 뒤 문단은 표 아래");
}

/// 쪽 맨 위 첫 최상위 표의 위끝 − 본문 위끝.
fn first_table_offset(page: &PageProbe) -> f64 {
    page.tables.first().expect("쪽 맨 위 표").0 - page.body_top.expect("본문 영역")
}

#[test]
fn natural_and_section_top_tac_tables_share_the_spacing_rule() {
    // N: 자연 쪽 넘김 쪽 맨 위 표(위 간격 1000HU), T: 구역 첫 큰 표(1200HU, 60000HU),
    // U: 구역 첫 작은 표(1200HU). 한/글 정본은 셋 다 위 간격 + 바깥 위 여백 아래에 표를 둔다.
    let core = open("samples/issue8474/top_tac_cases-2020.hwp");
    let pages = pages(&core);
    assert_eq!(pages.len(), 4, "독립 PDF 4쪽");
    let n = first_table_offset(&pages[1]);
    let t = first_table_offset(&pages[2]);
    let u = first_table_offset(&pages[3]);
    assert!(
        (t - u).abs() <= 0.5,
        "같은 위 간격이면 표 크기와 무관하게 같은 자리: T {t:.2} / U {u:.2}"
    );
    let step = 200.0 * 96.0 / 7200.0;
    assert!(
        ((t - n) - step).abs() <= 0.5,
        "구역 첫 표(1200HU)는 자연 쪽 넘김 표(1000HU)보다 200HU 아래: N {n:.2} / T {t:.2}"
    );
}
