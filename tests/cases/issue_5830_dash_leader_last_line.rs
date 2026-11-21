//! 관계형·고정형: 원본 HWPX의 dash 글리프·전진과 셀 내부 포함을 보호한다.
//! 주석 갱신: 2026-11-22T03:03:17+09:00; 직전 파일 변경 커밋: 2026-10-04T19:03:36+09:00 731de9e1b4bb946d76f35108ed7e186ebe4ebecb.
//! 측정: 2026-11-22 KST, Print profile, compat2024, 96DPI, PDFium, 보정 전2px raw.
//! 전64쪽 최저 Native98.69% / fresh WASM98.69%; 쪽수도 독립 Print와 같다.
//! 대상33쪽 Native99.99/WASM99.98%,34쪽 양쪽99.97%. 같은 영역의 직접 판독 근거는 source32/README.md.
//! PDF33쪽 dash 전진0.49857~0.56977em,34쪽0.56960~0.57260em을 독립 재측정했다.
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md와 issue7765-hwpx의 전쪽 TSV.
//! 실제 선택 프로그램 metadata만 공급하고 시스템 글꼴·전체 SVG hash·고정 px 영역을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
mod regulatory_svg {
    include!("../common/regulatory_dash_svg.rs");
}
use regulatory_svg::hyphen_runs;
fn render_page_svg(page: &str) -> String {
    regulatory_svg::render_page_svg(page.parse().unwrap())
}

#[test]
fn last_line_dash_leader_advance_stays_in_the_oracle_band() {
    for page_arg in ["32", "33"] {
        let svg = render_page_svg(page_arg);
        let runs = hyphen_runs(&svg, 8);
        assert!(
            runs.len() >= 3,
            "p{page_arg}: 8자 이상 dash 런이 3개는 있어야 한다 (실측 {})",
            runs.len()
        );
        for run in &runs {
            let em = run.advance() / run.font_size();
            assert!(
                (0.45..=0.75).contains(&em),
                "p{page_arg} y={:.1} {}자 런의 advance {:.3}em — 정본 대역(0.45~0.75em) 밖 \
                 (0.300em 은 마지막 줄 클램프 잔존, 0.75em 초과는 슬랙 과신장)",
                run.baseline,
                run.pts.len(),
                em
            );
        }
    }
}

/// dash leader 는 채움이지만 **여백을 넘지는** 않는다 — 확장 후에도 줄의 마지막
/// 글리프가 본문 오른쪽 경계 안에 있어야 한다.
#[test]
fn stretched_dash_leader_stays_inside_the_text_area() {
    // 전역707.1px 핀 대신 같은 원본 셀의 포함 관계로 보호한다.

    for page_arg in ["32", "33"] {
        let svg = render_page_svg(page_arg);
        let cells = cell_frames(page_arg.parse().unwrap());
        for run in hyphen_runs(&svg, 8) {
            let last_x = run.pts[run.pts.len() - 1].0;
            assert!(
                cells.iter().any(|cell| run.pts[0].0 >= cell[0] - 0.1
                    && last_x <= cell[2] + 0.1
                    && run.baseline >= cell[1] - 0.1 && run.baseline <= cell[3] + 0.1),
                "p{page_arg} y={:.1} {}자 런이 본문 경계를 넘는다: 마지막 글리프 원점 {last_x:.1}px",
                run.baseline,
                run.pts.len(),
            );
        }
    }
}

fn cell_frames(page: u32) -> Vec<[f64; 4]> {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(node: &RenderNode, out: &mut Vec<[f64; 4]>) {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            out.push([
                node.bbox.x,
                node.bbox.y,
                node.bbox.x + node.bbox.width,
                node.bbox.y + node.bbox.height,
            ]);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let page = regulatory_svg::core().build_page_render_tree(page).unwrap();
    let mut result = Vec::new();
    walk(&page.root, &mut result);
    result
}
