//! 관계형: PR #8530의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-05T01:10:21+09:00 (`18987f5dd4d63dcf37266c0d8157732ae4a21f6d`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8529/rowbreak_exact_fit.hwpx`; 독립 Print `pdf/issue8529_rowbreak_exact_fit-2020.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.94% / fresh WASM 99.96%.
//! 주장 1쪽 raw: Native 99.94% / fresh WASM 99.96%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8530_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-04T20:47:18+09:00; 신규 검사.
//! 측정일 2026-11-04, 코드 dd0ac546e3389f1703e67cb86f1c5bdf615ff0c9.
//! 96 DPI, 독립 한컴 Print와 Native Skia/fresh WASM Canvas의 색상 경계 보정 전 2px 실루엣.
//! 전체 3쪽, Native 최저 99.94% / fresh WASM 최저 99.96%; 1쪽도 각각 99.94% / 99.96%.
//! 2·3쪽은 두 경로 모두 100.00%. 첫 프레임·끝 행과 이어받기 영역을 직접 판독했다.
//! 입력: samples/issue8529/rowbreak_exact_fit.hwpx.
//! 독립 Print: pdf/issue8529_rowbreak_exact_fit-2020.pdf (한컴 2022, 3쪽).
//! 입력 SHA-256: 6f29966a3e061fb1763b60d8b4bc8c4b0898a53b66989ab13449ecc147d05bf7.
//! PDF SHA-256: c04a7cc89855a61a83c978d59686026adc68c672000e38deb9687397ec78a81f.
//! 원본 선언 68,454 HU는 독립 Print의 첫 물리 프레임이며 전체 표의 내용 높이가 아니다.
//! 절대 페이지 좌표 대신 원본 HU 프레임과 실제 조각의 크기·셀 포함·내용 소유를 대조한다.
//! 양쪽 98% 이상인 완료 페이지의 ±2px 조건을 충족하며, 설치 글꼴의 존재를 요구하지 않는다.
//! 전쪽 TSV와 같은 쪽 review/overlay·제품 해시: mydocs/pr/pr_8530_review.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn accepted_source_frame_is_not_extended_into_the_page_bottom_band() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue8529/rowbreak_exact_fit.hwpx"
    ))
    .expect("독립 Print와 같은 원본");
    core.set_dpi(96.0);
    core.set_hangul2024_compat(false);
    let source = core.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 31 && table.col_count == 17 => Some(table),
            _ => None,
        })
        .expect("규제 개요 원본 표");
    let declared_height = f64::from(source.common.height) * 96.0 / 7200.0;
    assert!(declared_height > 0.0, "원본이 선언한 첫 물리 프레임");
    let first = core.build_page_render_tree(0).expect("첫 조각");
    let mut nodes = Vec::new();
    walk(&first.root, &mut nodes);
    let tables: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(1) && table.cell_context.is_none())
        })
        .collect();
    assert_eq!(tables.len(), 1, "첫 쪽은 원본 표의 첫 조각 하나를 소유");
    let frame = tables[0];
    assert!(
        (frame.bbox.height - declared_height).abs() <= 2.0,
        "독립 Print의 원본 첫 프레임에 쪽 끝 여유를 더하지 않는다: 실제={}, 원본={declared_height}",
        frame.bbox.height
    );
    let mut checked_cells = 0;
    let mut owns_cut_row = false;
    for cell in &frame.children {
        if let RenderNodeType::TableCell(owner) = &cell.node_type {
            checked_cells += 1;
            owns_cut_row |= owner.row == 7;
            assert!(
                cell.bbox.y >= frame.bbox.y - 2.0
                    && cell.bbox.y + cell.bbox.height <= frame.bbox.y + frame.bbox.height + 2.0,
                "첫 조각을 닫을 때 그 조각이 소유한 셀도 같은 물리 경계 안에 있다"
            );
        }
    }
    assert!(
        checked_cells > 0 && owns_cut_row,
        "실제 끝 행을 포함한 셀 소유를 검사"
    );
    const SECOND: &str = "별도행정처분기준마련을통한정수기품질향상";
    const THIRD: &str = "이와더불어용출안전성검사부적합정수기의";
    let first_text = text(frame);
    let next = core.build_page_render_tree(1).expect("이어받기 조각");
    let next_text = text(&next.root);
    assert_eq!(
        first_text.matches(SECOND).count(),
        1,
        "수용한 둘째 문단 보존"
    );
    assert!(!first_text.contains(THIRD), "셋째 문단은 다음 조각 소유");
    assert_eq!(
        next_text.matches(THIRD).count(),
        1,
        "이어받은 문단 누락·중복 없음"
    );
    assert!(
        !next_text.contains(SECOND),
        "이미 수용한 문단을 다시 내보내지 않는다"
    );
}
