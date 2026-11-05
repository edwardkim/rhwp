//! 관계형: 세로쓰기 칸의 글자 보존·한 세로줄·셀 내부 포함·내용 순서와 합성 줄 소유를 검사한다.
//! 주석 갱신: 2026-11-05T12:06:08+09:00 (Asia/Seoul).
//! 직전 파일 변경: 2026-11-05T09:48:14+09:00, commit `4a0af7e86731466fa5202d806c3adf23b7d9c8e0`. 측정 시각과 구분한다.
//! 실제 측정: 2026-11-05 KST, source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`.
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 기존 한컴 2020 Print.
//! vertical_fit.hwpx: 전체 5쪽, Native/fresh WASM 각각 전쪽 최저 100.00%, 대상 5쪽도 각각 100.00%.
//! vertical_oversize.hwpx: 전체 5쪽, Native/fresh WASM 각각 전쪽 최저 100.00%, 대상 5쪽도 각각 100.00%.
//! 원본 `samples/issue8470/`, 독립 PDF `pdf/issue8470/`의 동명 `-2020.pdf`.
//! 정식 TSV·입력/PDF·제품 SHA와 같은 쪽의 review·별도 overlay는
//! `mydocs/pr/assets/planet6897_20261103/rebase8609/vertical-cell-claims.md`에 연결한다.
//! 같은 두 쪽의 Native/fresh WASM과 실제 Studio Canvas2D/CanvasKit 화면에서 다섯 글자를 직접 확인했다.
//! 같은 assertion의 수정 전 2 FAIL → 수정 후 2 PASS와 정상 대조 5 PASS는
//! `mydocs/pr/assets/planet6897_20261103/pr_8475/vertical-axis-focused-37e5bba300.md`에 보존했다.
//! 전체 최소 글꼴 nextest 12,583 PASS·Native Skia lib 4,040 PASS 및 CI Native 21개 모두 exit 0.
//! 설치 글꼴·특정 glyph 폭·독립 절대 px 좌표를 가정하지 않는다. raw 100.00%는 RGB 동일을 뜻하지 않는다.

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
