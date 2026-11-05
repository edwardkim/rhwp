//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8497의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:41:10+09:00 (`69108dd44bdcd4f55e77285f98929f1f430477f5`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8478/span_page_bottom-2020.hwp`; 독립 Print `pdf/issue8478/span_page_bottom-2020.pdf`.
//! 전체 12쪽, 전쪽 최저 Native 99.77% / fresh WASM 100.00%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 9쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8470/42565_입법예고문.hwp`; 독립 Print `pdf/issue8470/42565_입법예고문-2020.pdf`.
//! 전체 16쪽, 전쪽 최저 Native 94.92% / fresh WASM 96.96%.
//! 주장 11쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8497_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 04:55 KST (신규 파일).
//! 2026-11-03 측정 raw(`silhouette_raw_match_percent`, 96dpi, PDFium, `--compat 2022`, 한/글 2020 엔진 Print 정본, head `6e9bcbb5ab`):
//! span_page_bottom-2020.hwp: 3쪽(Q) Native 100.00% / fresh WASM 100.00%; 9쪽(U) 100.00% / 100.00%; 전12쪽 최저 99.77% / 100.00%.
//! 근거: mydocs/pr/assets/issue_8478/{native,wasm}/span_page_bottom_silhouette.tsv,
//! mydocs/pr/assets/issue_8478/png/span_page_bottom_p{3,9}_{review,overlay}_{native,wasm}.png.
//!
//! #8478 병합 칸이 키운 표의 끝 행이 쪽 바닥에 걸리면 한/글은 그 행을 남은 높이까지 그리고 닫는다.
//!
//! 독립 기준은 같은 HWP 의 한/글 2020 엔진 Print PDF(`pdf/issue8478/span_page_bottom-2020.pdf`)다.
//! 입력은 `samples/issue8478/gen_span_page_bottom.py` 로 만든 HWPX 를 한/글 2020 이 HWP 로 저장한
//! `span_page_bottom-2020.hwp` 이다(저장 줄 사다리 경로). 원 HWPX 도 함께 둔다.
//! 쪽마다 간격 표 아래에 6행 x 3열 표가 있다. 행 선언은 모두 2048HU, 1~5행 0열 병합 칸 선언은 행 합 +
//! 849HU 라 끝 행(5행)이 병합 칸 몫만큼 커진다. 간격 표가 표 전체를 본문 바닥 너머로 d 만큼 민다.
//! - P(d=-150HU): 다 들어간다. 정본 끝 행 38.5px(칸 선언 27.3 + 병합 몫 11.3).
//! - Q(d=225)·R(d=450)·S(d=849): 정본은 세 표 모두 같은 쪽 바닥 위치에서 끝 행을 닫고(34.2·31.1·25.9px)
//!   남는 빈 띠를 다음 쪽으로 넘기지 않는다. 수정 전 rhwp 는 끝 행을 칸 선언 27.3px 에서 잘라 Q·R 표가
//!   정본보다 5.9·2.9px 짧았다(입법예고문 docs#333 11쪽과 같은 꼴: 34.0px 대신 27.3px).
//! - U(d=225)·V(d=450): 4행에 1행 칸이 없고 1·2열 4~5행 병합 칸이 끝 행을 키운다. 정본은 Q·R 과 같은
//!   쪽 바닥 위치에서 끝 행을 닫는다(행 선언이 덜 정해진 표도 같은 규칙).
//!
//! 검사는 표·칸 사이의 관계다 — Q·R·S·U·V 표 바닥은 같다, Q·R·U·V 끝 행은 칸 선언보다 크다, P 표는 선언 높이
//! 그대로다, 병합 칸은 표 바닥에서 닫힌다, 다음 쪽에는 표 조각이 없다. 절대 좌표는 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn text_of(node: &RenderNode) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            text.push_str(&run.text);
        }
        stack.extend(n.children.iter().rev());
    }
    text
}

/// 쪽의 최상위 표들과 그 칸들 — (표 위끝, 표 아래끝, [(칸 글자, 칸 위끝, 칸 아래끝)]).
type TableProbe = (f64, f64, Vec<(String, f64, f64)>);

fn tables(node: &RenderNode, out: &mut Vec<TableProbe>) {
    if let RenderNodeType::Table(_) = &node.node_type {
        let mut cells = Vec::new();
        for child in &node.children {
            if let RenderNodeType::TableCell(_) = &child.node_type {
                cells.push((
                    text_of(child).trim().to_string(),
                    child.bbox.y,
                    child.bbox.y + child.bbox.height,
                ));
            }
        }
        out.push((node.bbox.y, node.bbox.y + node.bbox.height, cells));
        return;
    }
    for child in &node.children {
        tables(child, out);
    }
}

struct Page {
    text: String,
    tables: Vec<TableProbe>,
}

fn pages(core: &DocumentCore) -> Vec<Page> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            let mut found = Vec::new();
            tables(&tree.root, &mut found);
            Page {
                text: text_of(&tree.root),
                tables: found,
            }
        })
        .collect()
}

struct Closing {
    table_height: f64,
    table_bottom: f64,
    last_row: f64,
    span_bottom: f64,
}

/// `label` 표가 놓인 쪽에서 표 높이·바닥, 끝 행(5행) 칸 높이, 병합 칸 바닥을 잰다.
/// 다음 쪽에는 그 표의 조각이 없고 표 뒤 문단만 있어야 한다.
fn closing(pages: &[Page], label: &str) -> Closing {
    let index = pages
        .iter()
        .position(|page| page.text.contains(&format!("{label} 머리")))
        .unwrap_or_else(|| panic!("`{label}` 표 쪽이 없다"));
    let (top, bottom, cells) = pages[index]
        .tables
        .iter()
        .find(|(_, _, cells)| cells.iter().any(|(t, _, _)| *t == format!("{label} 머리")))
        .expect("6행 표");
    let cell = |text: String| {
        cells
            .iter()
            .find(|(t, _, _)| *t == text)
            .unwrap_or_else(|| panic!("`{text}` 칸"))
    };
    let last = cell(format!("{label}5가"));
    let span = cell(format!("{label} 병합"));
    let next = pages.get(index + 1).expect("표 뒤 쪽");
    assert!(
        next.text.contains(&format!("{label} 뒤")) && next.tables.is_empty(),
        "`{label}` 표의 남는 빈 띠를 다음 쪽으로 넘기지 않는다"
    );
    Closing {
        table_height: bottom - top,
        table_bottom: *bottom,
        last_row: last.2 - last.1,
        span_bottom: span.2,
    }
}

#[test]
fn span_raised_closing_row_ends_at_page_bottom() {
    let core = open("samples/issue8478/span_page_bottom-2020.hwp");
    let pages = pages(&core);
    assert_eq!(pages.len(), 12, "독립 PDF 12쪽");

    let row_px = 2048.0 * 96.0 / 7200.0;
    let p = closing(&pages, "P");
    let q = closing(&pages, "Q");
    let r = closing(&pages, "R");
    let s = closing(&pages, "S");
    let u = closing(&pages, "U");
    let v = closing(&pages, "V");

    // P 는 다 들어가므로 끝 행이 병합 칸 몫까지 커지고 표는 선언 높이(행 합 + 849HU) 그대로다.
    let declared = (2048.0 * 6.0 + 849.0) * 96.0 / 7200.0;
    assert!(
        (p.table_height - declared).abs() <= 0.5,
        "P 표 높이 {:.2} = 선언 {declared:.2}",
        p.table_height
    );
    assert!(p.last_row > row_px + 0.5, "P 끝 행은 병합 칸 몫만큼 크다");

    // Q·R·S·U·V 는 같은 쪽 바닥에서 끝 행을 닫는다.
    for (name, c) in [("R", &r), ("S", &s), ("U", &u), ("V", &v)] {
        assert!(
            (c.table_bottom - q.table_bottom).abs() <= 0.5,
            "{name} 표 바닥 {:.2} = Q 표 바닥 {:.2}",
            c.table_bottom,
            q.table_bottom
        );
    }
    // 남은 높이가 칸 선언보다 크면 칸 선언에서 자르지 않는다.
    for (name, c) in [("Q", &q), ("R", &r), ("U", &u), ("V", &v)] {
        assert!(
            c.last_row > row_px + 0.5,
            "{name} 끝 행 {:.2} 은 칸 선언 {row_px:.2} 보다 크다",
            c.last_row
        );
        assert!(
            c.last_row < p.last_row - 0.5,
            "{name} 끝 행은 쪽 바닥에서 잘린다"
        );
    }
    // 끝 행과 병합 칸은 표 바닥에서 함께 닫힌다.
    for (name, c) in [
        ("P", &p),
        ("Q", &q),
        ("R", &r),
        ("S", &s),
        ("U", &u),
        ("V", &v),
    ] {
        assert!(
            (c.span_bottom - c.table_bottom).abs() <= 0.5,
            "{name} 병합 칸 바닥 {:.2} = 표 바닥 {:.2}",
            c.span_bottom,
            c.table_bottom
        );
    }
}
