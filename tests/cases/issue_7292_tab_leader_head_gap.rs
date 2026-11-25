//! 관계형·고정형: KTX 목차 쪽번호의 셀 소속·순서·정렬과 첫 점 중심을 보호한다.
//! 주석 갱신 2026-11-26T03:57:48+09:00. 직전 파일 커밋 2026-10-24T15:25:31+09:00, 3918f2d22c.
//! 2026-11-26 한컴 2020 Print, 96 DPI·PDFium·2px raw 실루엣.
//! 2쪽 Native 100.00%, fresh WASM 100.00%. 전 27쪽 최저 Native 99.99%, fresh WASM 99.87%.
//! 근거는 mydocs/pr/assets/issue7445/ktx-toc27/devel-32014/README.md에 있다.
//!
//! #7292: 첫 점의 실제 중심을 독립 Print와 비교해 앞 글자와의 간격을 보호한다.
//! pdf/KTX-2022.pdf p2: 첫 U+00B7의 origin=164.28pt, size=15pt.
//! T3 Encoding code00→HFT1, CharProc xref24의 윤곽 중심은 192/1000em이다.
//! 따라서 첫 점 중심은 (164.28 + 15*0.192)*96/72 = 222.88px이다.
//! 이전 검사는 glyph advance 원점 219.07px를 SVG dash 선의 x1과 비교했다.
//! 개별 원을 그리는 출력에서는 실제 점 중심을 확인하며, 선 태그 존재로 대신하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn toc_bytes() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue7445/ktx-toc27/KTX.hwp");
    std::fs::read(path).expect("KTX 27쪽 원본")
}

mod svg_tab_leaders {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/common/svg_tab_leaders.rs"
    ));
}

#[test]
fn toc_first_dot_center_preserves_independent_head_gap() {
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&toc_bytes()).expect("문서 로드");
    let svg = doc.render_page_svg_native(1).expect("목차 쪽 SVG");
    let leader = svg_tab_leaders::dot_leaders(&svg)
        .into_iter()
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("실제 점끌기");
    let first_center = leader.start + leader.diameter / 2.0;
    assert!(
        (first_center - 222.88).abs() <= 0.6,
        "첫 점 중심 {first_center:.2}px — 독립 Print의 실제 glyph 중심 222.88px"
    );
}

#[test]
fn toc_page_numbers_share_the_print_right_edge_inside_the_toc_cell() {
    fn collect<'a>(
        node: &'a RenderNode,
        cell: Option<&'a RenderNode>,
        numbers: &mut Vec<(u8, &'a RenderNode, &'a RenderNode)>,
    ) {
        let cell = if matches!(&node.node_type, RenderNodeType::TableCell(_)) {
            Some(node)
        } else {
            cell
        };
        if let (Some(cell), RenderNodeType::TextRun(run)) = (cell, &node.node_type) {
            if let Ok(number) = run.text.trim().parse() {
                numbers.push((number, node, cell));
            }
        }
        for child in &node.children {
            collect(child, cell, numbers);
        }
    }

    let mut core = DocumentCore::from_bytes(&toc_bytes()).expect("목차 문서 로드");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 27, "한컴 Print의 전체 쪽수");

    let tree = core.build_page_render_tree(1).expect("목차 쪽 렌더 트리");
    let mut numbers = Vec::new();
    collect(&tree.root, None, &mut numbers);
    numbers.sort_by(|a, b| a.1.bbox.y.total_cmp(&b.1.bbox.y));

    let actual: Vec<_> = numbers.iter().map(|&(number, _, _)| number).collect();
    assert_eq!(
        actual,
        [3, 4, 4, 6, 7, 8, 9, 14, 16, 17, 17, 20, 21, 23, 24, 25, 27],
        "장과 절의 대응 쪽번호가 순서대로 한 번씩 표시된다"
    );
    let (_, first_number, toc_cell) = numbers[0];
    let first_right = first_number.bbox.x + first_number.bbox.width;
    let cell_right = toc_cell.bbox.x + toc_cell.bbox.width;
    for (number, node, cell) in numbers {
        assert!(std::ptr::eq(cell, toc_cell), "쪽번호 {number}의 셀 소속");
        assert!(node.bbox.x >= toc_cell.bbox.x);
        let right = node.bbox.x + node.bbox.width;
        assert!(right < cell_right, "쪽번호 {number}가 목차 셀 안에 있다");
        assert!(
            (right - first_right).abs() <= 0.5,
            "쪽번호 {number}의 오른쪽 끝 {right:.2}px, 첫 번호 {first_right:.2}px"
        );
        assert!(
            (right - 701.64).abs() <= 1.0,
            "쪽번호 {number}의 오른쪽 끝 {right:.2}px, 독립 Print 701.64px"
        );
    }
}
