//! 한컴 Print와 Native/fresh WASM 전1쪽을 확인한 표의 내용·소유·표시 정렬 계약.
//! TextRun 전진 상자의 끝에는 음수 자간의 후행 전진도 포함된다.
//! 마지막 글자 뒤 전진과 실제 표시 폭을 구분하여 셀 가운데 정렬을 확인한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn collect<'a>(
    node: &'a RenderNode,
    accept: fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if accept(node) {
        out.push(node);
    }
    for child in &node.children {
        collect(child, accept, out);
    }
}

#[test]
fn issue7642_table_text_preserves_cells_content_and_visible_center_alignment() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/table-text.hwpx");
    let doc = HwpDocument::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 파싱");
    assert_eq!(doc.page_count(), 1, "독립 Print의 전체 쪽수");
    let tree = doc.build_page_render_tree(0).expect("실제 첫 쪽");
    let mut tables = Vec::new();
    collect(
        &tree.root,
        |n| matches!(n.node_type, RenderNodeType::Table(_)),
        &mut tables,
    );
    assert_eq!(tables.len(), 1);
    let RenderNodeType::Table(table) = &tables[0].node_type else {
        unreachable!()
    };
    assert_eq!((table.row_count, table.col_count), (3, 8));
    let mut cells = Vec::new();
    collect(
        tables[0],
        |n| matches!(n.node_type, RenderNodeType::TableCell(_)),
        &mut cells,
    );
    assert_eq!(cells.len(), 18, "병합 두 칸과 하단 두 행의 소유 칸");
    let expected = [
        (0, 0, "기부 금액(원, %)"),
        (0, 4, "기부 건수(건, %)"),
        (1, 0, "2023년"),
        (1, 1, "2024년"),
        (1, 2, "2025년"),
        (1, 3, "2024년 대비 증감"),
        (1, 4, "2023년"),
        (1, 5, "2024년"),
        (1, 6, "2025년"),
        (1, 7, "2024년 대비 증감"),
        (2, 0, "65,063,026,600"),
        (2, 1, "87,804,677,338"),
        (2, 2, "151,459,074,040"),
        (2, 3, "72.5"),
        (2, 4, "526.278"),
        (2, 5, "772,712"),
        (2, 6, "1,391,874"),
        (2, 7, "80.0"),
    ];
    for (row, col, text) in expected {
        let owners: Vec<_> = cells
            .iter()
            .copied()
            .filter(|n| {
                matches!(&n.node_type,
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
            })
            .collect();
        assert_eq!(owners.len(), 1, "({row},{col}) 소유는 한 번만 방출한다");
        let cell = owners[0];
        let mut runs = Vec::new();
        collect(
            cell,
            |n| matches!(n.node_type, RenderNodeType::TextRun(_)),
            &mut runs,
        );
        let content: String = runs
            .iter()
            .map(|n| match &n.node_type {
                RenderNodeType::TextRun(run) => run.display_or_text(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(content.trim(), text, "칸의 내용·순서·누락/중복");
        let tolerance = cell.bbox.width.min(cell.bbox.height) * 0.005;
        for run in &runs {
            assert!(
                run.bbox.x >= cell.bbox.x - tolerance
                    && run.bbox.x + run.bbox.width <= cell.bbox.x + cell.bbox.width + tolerance
                    && run.bbox.y >= cell.bbox.y - tolerance
                    && run.bbox.y + run.bbox.height <= cell.bbox.y + cell.bbox.height + tolerance,
                "({row},{col}) 글자 전진은 소유 칸 내부에 있다"
            );
        }
        if row == 2 || (row == 1 && (col == 3 || col == 7)) {
            let left = runs.iter().map(|n| n.bbox.x).fold(f64::INFINITY, f64::min);
            let last = runs
                .iter()
                .max_by(|a, b| (a.bbox.x + a.bbox.width).total_cmp(&(b.bbox.x + b.bbox.width)))
                .expect("마지막 표시 런");
            let RenderNodeType::TextRun(style_run) = &last.node_type else {
                unreachable!()
            };
            // 독립 Print의 MalgunGothic CIDFont /W: 숫자는 550/1000em,
            // 제목 끝 한글은 1000/1000em이다. 자간은 해당 글자의 폭 비례다.
            // 전각 자간을 숫자에도 적용하면 정상 Print의 표시 끝을 오판한다.
            let final_char = style_run.display_or_text().chars().last().expect("끝 글자");
            let glyph_advance_em = if final_char.is_ascii_digit() {
                550.0 / 1000.0
            } else {
                assert!(('\u{ac00}'..='\u{d7a3}').contains(&final_char));
                1000.0 / 1000.0
            };
            let trailing_advance = style_run.style.letter_spacing * glyph_advance_em
                + style_run.style.extra_char_spacing;
            let visible_right = last.bbox.x + last.bbox.width - trailing_advance;
            let center = cell.bbox.x + cell.bbox.width / 2.0;
            assert!(
                ((left + visible_right) / 2.0 - center).abs() <= tolerance,
                "({row},{col}) 수치·제목의 표시 범위가 소유 칸 가운데에 있다"
            );
        }
    }
}
