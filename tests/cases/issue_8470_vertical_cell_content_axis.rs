//! 관계형: 세로쓰기 칸의 글줄은 칸 높이를 사용하며 모든 글자를 칸 안에 보존한다.
//! 주석 갱신: 2026-11-05T07:24:27+09:00; 신규 파일이며 직전 변경 커밋은 없다.
//! 선행 측정: 2026-11-04~05 KST, 제품 `4b149b6cdf` 및 Native `c183a1ac61`.
//! 96 DPI, 색상 경계 보정 전 2px 내용 실루엣 raw, 기존 한컴 2020 Print.
//! vertical_fit.hwpx: 전체 5쪽, 전쪽 최저와 대상 5쪽 Native 98.75% / fresh WASM 98.72%.
//! vertical_oversize.hwpx: 전체 5쪽, 전쪽 최저와 대상 5쪽 Native 98.94% / fresh WASM 98.92%.
//! 입력 `samples/issue8470/`, 독립 PDF `pdf/issue8470/`의 동명 `-2020.pdf`.
//! TSV·review·overlay·입력/PDF 해시는 `mydocs/pr/pr_8475_review.md`에 연결한다.
//! 직접 판독에서 병합 칸의 마지막 글자 소실을 확인했다. 이 개발 중 검사는 수정 전 RED를
//! 확인하려는 것이며, 보정 후 fresh WASM·Native 판독 완료나 최종 승인을 뜻하지 않는다.
//! 화면 좌표·글꼴 프로그램·특정 글리프 폭을 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn collect_text_runs<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        result.push(node);
    }
    for child in &node.children {
        collect_text_runs(child, result);
    }
}

fn inspect_vertical_cell(node: &RenderNode, found: &mut usize) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.text_direction != 0 {
            let mut runs = Vec::new();
            collect_text_runs(node, &mut runs);
            let text: String = runs
                .iter()
                .filter_map(|run| match &run.node_type {
                    RenderNodeType::TextRun(text) => Some(text.text.as_str()),
                    _ => None,
                })
                .collect();
            if text == "임용전경력" {
                *found += 1;
                assert_eq!(runs.len(), text.chars().count(), "세로 글자의 누락·중복");
                let first = runs[0].bbox;
                let precision =
                    f64::EPSILON * node.bbox.x.abs().max(node.bbox.y.abs()).max(1.0) * 16.0;
                let one_column = runs
                    .iter()
                    .all(|run| (run.bbox.x - first.x).abs() <= precision);
                let contained = runs.iter().all(|run| {
                    run.bbox.x + precision >= node.bbox.x
                        && run.bbox.y + precision >= node.bbox.y
                        && run.bbox.x + run.bbox.width <= node.bbox.x + node.bbox.width + precision
                        && run.bbox.y + run.bbox.height
                            <= node.bbox.y + node.bbox.height + precision
                });
                assert!(
                    one_column && contained,
                    "독립 Print는 다섯 글자를 같은 세로줄의 칸 안에 보존한다: \
                     one_column={one_column}, contained={contained}, cell={:?}, runs={:?}",
                    node.bbox,
                    runs.iter().map(|run| run.bbox).collect::<Vec<_>>()
                );
                for pair in runs.windows(2) {
                    assert!(
                        pair[1].bbox.y + precision >= pair[0].bbox.y + pair[0].bbox.height,
                        "위에서 아래로 내용 순서를 보존하고 앞 글자와 겹치지 않는다"
                    );
                }
            }
        }
    }
    for child in &node.children {
        inspect_vertical_cell(child, found);
    }
}

fn assert_vertical_cell_content(path: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 열기");
    let mut found = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 쪽 배치");
        inspect_vertical_cell(&tree.root, &mut found);
    }
    assert_eq!(found, 1, "목표 세로쓰기 칸은 전체 출력에 한 번만 존재한다");
    // Print의 한 세로줄은 그 입력을 발행한 합성 캐시에도 적용한다.
    // 가로 폭으로 만든 다줄 캐시를 측정에 남겨 두고 paint만 고치지 못하게 한다.
    let source_labels: Vec<_> = core
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table),
            _ => None,
        })
        .flat_map(|table| &table.cells)
        .filter(|cell| cell.text_direction != 0)
        .flat_map(|cell| &cell.paragraphs)
        .filter(|paragraph| paragraph.text == "임용전경력")
        .collect();
    assert_eq!(source_labels.len(), 1, "원본 세로쓰기 라벨의 소유 칸");
    assert_eq!(
        source_labels[0].line_segs.len(),
        1,
        "Print에서 한 세로줄인 라벨을 가로 폭으로 쪼갠 캐시로 측정하지 않는다"
    );
}

#[test]
fn fitted_vertical_table_keeps_all_label_glyphs_in_one_cell_column() {
    assert_vertical_cell_content("samples/issue8470/vertical_fit.hwpx");
}

#[test]
fn overflowing_vertical_table_keeps_all_label_glyphs_in_one_cell_column() {
    assert_vertical_cell_content("samples/issue8470/vertical_oversize.hwpx");
}
