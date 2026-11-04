//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8476의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:40:17+09:00 (`158ebac0d80eb3caabb3007df2a13a06ef464410`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8471/pushed_none.hwpx`; 독립 Print `pdf/issue8471/pushed_none-2020.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 99.96% / fresh WASM 100.00%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 3쪽 raw: Native 99.97% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8476_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02 19:16 KST (신규 파일).
//! 2026-11-02 측정 raw(`silhouette_raw_match_percent`, 96dpi, PDFium, `--compat 2022`, 한/글 2020 엔진 Print 정본):
//! pushed_none.hwpx: 3쪽(N1 표) Native 99.97% / fresh WASM 100.00%; 5쪽(N2 표) Native 99.96% / fresh WASM 100.00%;
//! 전8쪽(독립 PDF 8쪽과 일치) 최저 99.96% / 100.00%.
//! 근거: mydocs/pr/assets/issue_8471/{native,wasm}/pushed_none_silhouette.tsv,
//! mydocs/pr/assets/issue_8471/png/pushed_none_p{3,5}_{review,overlay}_{native,wasm}.png.
//!
//! #8471 다음 쪽으로 통째로 넘긴 「나누지 않음」 표 뒤의 쪽 나누기 문단은 그 표 쪽 다음에서 시작한다.
//!
//! 독립 기준은 같은 HWPX 의 한/글 2020 엔진 Print PDF(`pdf/issue8471/pushed_none-2020.pdf`)다. 생성기는
//! `samples/issue8471/gen_pushed_none_table.py` 이며, 아래 쪽 소속은 PDF 본문 글자에서 읽었다.
//! 한/글 8쪽: 2 `N1 끝` · 3 N1 표 · 4 `N2 끝` · 5 N2 표 · 6 N3 채움·머리·끝 · 7 N3 표 · 8 N4.
//! - N1·N2: 쪽 나누기 빈 문단 안 본문보다 긴 표. 앞 표가 넘어간 쪽에 다음 쪽 나누기 문단과 그 표가
//!   함께 놓이면 안 된다(수정 전 rhwp 는 3쪽에 두 표를 겹쳐 6쪽).
//! - N3·N4: 본문보다 짧은 표도 넘어간 쪽을 혼자 차지하고, 뒤 쪽 나누기 문단은 그다음 쪽에서 시작한다.
//!
//! 검사는 쪽 소속(어느 행·문단이 어느 쪽에 있나)·누락·중복과 독립 PDF 전체 쪽수다. 좌표는 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn line_texts(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                text.push_str(&run.text);
            }
            stack.extend(n.children.iter().rev());
        }
        out.push(text.trim().to_string());
        return;
    }
    for child in &node.children {
        line_texts(child, out);
    }
}

fn page_lines(core: &DocumentCore) -> Vec<Vec<String>> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            let mut lines = Vec::new();
            line_texts(&tree.root, &mut lines);
            lines
        })
        .collect()
}

/// 표 `label` 의 행 번호를 쪽마다 모은다: `(쪽 번호, 행 번호들)`.
fn rows_by_page(pages: &[Vec<String>], label: &str) -> Vec<(usize, Vec<u32>)> {
    let prefix = format!("{label} 행");
    pages
        .iter()
        .enumerate()
        .filter_map(|(page, lines)| {
            let mut rows: Vec<u32> = lines
                .iter()
                .filter_map(|line| line.strip_prefix(&prefix)?.trim().parse().ok())
                .collect();
            rows.sort_unstable();
            (!rows.is_empty()).then_some((page, rows))
        })
        .collect()
}

fn page_of(pages: &[Vec<String>], text: &str) -> usize {
    let found: Vec<usize> = (0..pages.len())
        .filter(|&page| pages[page].iter().any(|line| line == text))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "`{text}` 는 한 쪽에 한 번 나와야 한다: {found:?}"
    );
    found[0]
}

/// 표 `label` 이 나뉘지 않고 한 쪽에 모두 있으면 그 쪽을 돌려준다.
fn whole_table_page(pages: &[Vec<String>], label: &str, rows: u32) -> usize {
    let found = rows_by_page(pages, label);
    assert_eq!(found.len(), 1, "{label} 은 나뉘지 않는다: {found:?}");
    assert_eq!(
        found[0].1,
        (1..=rows).collect::<Vec<_>>(),
        "{label} 행 누락·중복 없음"
    );
    found[0].0
}

#[test]
fn page_break_after_pushed_none_table_starts_after_that_table_page() {
    let core = open("samples/issue8471/pushed_none.hwpx");
    let pages = page_lines(&core);

    let n1_end = page_of(&pages, "N1 끝");
    let n1 = whole_table_page(&pages, "N1", 40);
    let n2_end = page_of(&pages, "N2 끝");
    let n2 = whole_table_page(&pages, "N2", 40);
    let n3_head = page_of(&pages, "N3 머리");
    let n3 = whole_table_page(&pages, "N3", 30);
    let n4_head = page_of(&pages, "N4 머리");

    // 넘긴 표는 앵커 문단 다음 쪽을 혼자 차지한다.
    assert_eq!(n1, n1_end + 1, "N1 표는 N1 끝 다음 쪽");
    assert_eq!(n2_end, n1 + 1, "N2 쪽 나누기는 N1 표 쪽 다음에서 시작");
    assert_eq!(n2, n2_end + 1, "N2 표는 N2 끝 다음 쪽");
    assert_eq!(n3_head, n2 + 1, "N3 쪽 나누기는 N2 표 쪽 다음에서 시작");
    assert_eq!(
        page_of(&pages, "N3 끝"),
        n3_head,
        "N3 끝은 채움 쪽에 남는다"
    );
    assert_eq!(n3, n3_head + 1, "짧은 N3 표도 다음 쪽으로 통째로 넘어간다");
    assert_eq!(n4_head, n3 + 1, "N4 쪽 나누기는 N3 표 쪽 다음에서 시작");
    for (label, page) in [("N1", n1), ("N2", n2), ("N3", n3)] {
        assert!(
            pages[page].iter().all(|line| !line.ends_with(" 끝")
                && !line.ends_with(" 머리")
                && !line.contains(" 채움 ")),
            "{label} 표 쪽에는 다른 문단이 없다: {:?}",
            pages[page]
        );
    }
    // 독립 PDF 전체 쪽수(한/글 8쪽).
    assert_eq!(core.page_count(), 8u32);
}
