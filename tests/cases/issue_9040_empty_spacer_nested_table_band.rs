//! 관계형·고정형 — [#9040] 나뉜 칸의 빈 문단 뒤 자리차지 중첩 표 띠를 한 번만 세고 표를 앞 흐름 뒤에 둔다.
//! 주석 갱신: 2026-11-13 12:28 KST (새 파일)
//!
//! 입력: 공개 정책연구용역과제 신청서(문화체육관광부, HWP, 10쪽) `samples/issue9040/1371000-policy-research-application.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9040/1371000-policy-research-application-2020.pdf`(10쪽).
//! 7~9쪽에 걸쳐 나뉜 RowBreak 표 칸(pi=107, 4행 둘째 칸, 문단 44개)은 9쪽 조각을 쪽 되감김 문단 p40 에서
//! 연다. p40·p41 은 빈 줄(1000+500HU), p42 는 글자 없는 자리차지 조직도 표(23×9, 26502 + 바깥 여백 141·141)
//! 호스트, p43 은 빈 줄이다. 저장값은 `p41 끝 3000 + 26784 = p42 줄 29784`, `p42 줄 + 1500 = p43 줄 31284`.
//! 수정 전 rhwp 는 p41 의 전진에 표 띠(377px)를 넣고 조직도 행을 다시 세어 조각을 771px 로 그렸고(11쪽),
//! 조직도를 칸 위끝에 그렸다.
//! - 고정형: 전체 쪽수 10(= PDF).
//! - 관계형: 9쪽 조직도 표는 칸 조각 안에 있고 앞 빈 줄(p41) 아래에서 시작한다. 9쪽은 「2. 유사 용역실적」과
//!   「창의도시 정책」 행까지, 10쪽은 「Pension Policies」 행부터 담는다.
//!
//! 측정(2026-11-13, 이 파일과 같은 커밋의 코드, base = upstream/devel 41c889cfd8,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 10쪽 / 10쪽, 9쪽 99.83 · 10쪽 99.73(base 21.65 · 27.04, base 11쪽), 최저 91.33(2쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 10쪽 / 10쪽, 9쪽 99.84 · 10쪽 99.76, 최저 91.92(2쪽)
//! - 증거: `mydocs/pr/assets/issue_9040/`(native·wasm TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9040/1371000-policy-research-application.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 10;
/// 조직도가 있는 쪽(0 기반, 9쪽).
const CHART_PAGE: u32 = 8;
/// 나뉜 바깥 표의 소유 문단과 조직도 앞 빈 줄 문단(칸 문단 번호).
const OUTER_TABLE_PARA: usize = 107;
const SPACER_PARA: usize = 41;

fn page_text(core: &DocumentCore, page: u32) -> String {
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut text = String::new();
    walk(&tree.root, &mut text);
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

#[test]
fn split_cell_counts_empty_host_table_band_once() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let tree = core
        .build_page_render_tree(CHART_PAGE)
        .expect("render tree");
    let outer = find(&tree.root, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(OUTER_TABLE_PARA) && t.cell_context.is_none())
    })
    .expect("9쪽에 나뉜 바깥 표 조각이 있어야 한다");
    let chart = find(outer, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t) if t.row_count == 23 && t.col_count == 9)
    })
    .expect("9쪽 조각에 조직도 표가 있어야 한다");
    let spacer = find(outer, &|n| {
        matches!(&n.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(SPACER_PARA))
    })
    .expect("9쪽 조각에 조직도 앞 빈 줄이 있어야 한다");

    let spacer_bottom = spacer.bbox.y + spacer.bbox.height;
    assert!(
        chart.bbox.y >= spacer_bottom,
        "조직도는 앞 빈 줄 아래에서 시작해야 한다: 표 {} < 빈 줄 끝 {spacer_bottom}",
        chart.bbox.y
    );
    let outer_bottom = outer.bbox.y + outer.bbox.height;
    assert!(
        chart.bbox.y + chart.bbox.height <= outer_bottom,
        "조직도는 칸 조각 안에 있어야 한다"
    );

    let chart_page = page_text(&core, CHART_PAGE);
    let next_page = page_text(&core, CHART_PAGE + 1);
    assert!(
        chart_page.contains("정부학연구소평의원회")
            && chart_page.contains("유사용역실적")
            && chart_page.contains("창의도시정책"),
        "9쪽은 조직도·「2. 유사 용역실적」·「창의도시 정책」 행을 담아야 한다: {chart_page}"
    );
    assert!(
        !chart_page.contains("PensionPolicies"),
        "「Pension Policies」 행은 10쪽에 있어야 한다(칸 조각 이중 계상 없음)"
    );
    assert!(
        next_page.contains("PensionPolicies"),
        "10쪽은 「Pension Policies」 행부터 담아야 한다: {next_page}"
    );
}
