//! #8582: 호환 대상이 MS 워드인 문서는 표만 가진 빈 앵커 문단의 줄을 표 아래에 놓는다.
//!
//! 기준: 데스크톱 한컴 2024 인쇄 PDF(Hancom PDF 드라이버)의 글자 기준선.
//! 재현물은 `samples/test-image.hwpx` 에서 호환 대상과 문단 모양만 바꿨다. 7pt 빈 문단에
//! 글자처럼 아닌 자리차지 표를 두고 다음 문단에 `AFTER가` 를 세 번 되풀이한다.
//! 표 아래 끝 → `AFTER가` 기준선:
//! - 한글 문서 935 HU(= `AFTER가` 줄 기준선 0.85 × 1100)
//! - 워드 문서 2579 HU(= 앵커 줄 7pt 한 줄 700 × 1.691 ≈ 1184 + `AFTER가` 기준선 0.75 × 1860).
//!   칸 높이도 160% 칸 문단의 줄 간격을 담는다(표 높이 3258 HU, 한글 문서 1382 HU).
//!
//! 수정 전 rhwp 는 워드 문서도 앵커 줄을 더하지 않아 `AFTER가` 가 표에 붙었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

fn collect(node: &RenderNode, tables: &mut Vec<BoundingBox>, after: &mut Vec<f64>) {
    match &node.node_type {
        RenderNodeType::Table(_) => tables.push(node.bbox.clone()),
        RenderNodeType::TextRun(run)
            if run.cell_context.is_none() && run.text.contains("AFTER") =>
        {
            after.push(node.bbox.y + run.baseline)
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, after);
    }
}

/// 표마다 (표 아래 끝 → 바로 뒤 `AFTER가` 기준선) 간격과 표 높이(HU).
fn below_tables(name: &str) -> Vec<(f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("#8582 표 앵커 재현물");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut tables, mut after) = (Vec::new(), Vec::new());
    collect(&tree.root, &mut tables, &mut after);
    tables.sort_by(|a, b| a.y.total_cmp(&b.y));
    after.sort_by(|a, b| a.total_cmp(b));
    assert_eq!(tables.len(), 3, "표 3개");
    assert_eq!(after.len(), 3, "AFTER 줄 3개");
    tables
        .iter()
        .zip(&after)
        .map(|(table, baseline)| {
            (
                (baseline - (table.y + table.height)) * HU_PER_PX,
                table.height * HU_PER_PX,
            )
        })
        .collect()
}

#[test]
fn word_target_places_anchor_line_below_table() {
    for (name, gap_hu, table_hu) in [
        ("hwp_table_anchor_line", 935.0, 1382.0),
        ("word_table_anchor_line", 2579.0, 3258.0),
    ] {
        for (gap, height) in below_tables(name) {
            assert!(
                (gap - gap_hu).abs() <= 40.0,
                "{name}: 표 아래 → AFTER가 기준선 {gap:.0} HU, 데스크톱 한컴 {gap_hu:.0} HU"
            );
            assert!(
                (height - table_hu).abs() <= 40.0,
                "{name}: 표 높이 {height:.0} HU, 데스크톱 한컴 {table_hu:.0} HU"
            );
        }
    }
}
