//! 독립 Print의 대시 복원 한도와 같은 줄 낱말 간격을 함께 보호한다.
//! PII 원본66쪽 C250 Native/fresh WASM 전쪽 최저94.12098/93.77937%, 미달0.
//! C253 보정 후4쪽 Native100/fresh WASM98.89537%, 양쪽 review/overlay 직접 판독 완료.
//! DPI192, 좌표CSS96. 독립 Print 기호 원점(544.96,193.28)을 ±2CSSpx로 보호한다.
//! 입력/PDF/producer SHA·전264쪽TSV·대표PNG: mydocs/pr/assets/issue7445/
//! completed-20261009/dash-word-spacing-20261018/. 내용 소속 관계도 계속 검사한다.
//! 같은 p0/charPr8/paraPr47의 좌우 셀을 대조한다. 독립 Print에서 오른쪽 줄의
//! '-' 전진은 약0.834em, 내부 공백은 약0.65em이다. 여유 전부를 대시에 넣어
//! 0.884em으로 늘리고 공백을0.5em에 두면 '①'가 왼쪽으로 이동한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::{
        font_environment::FontEnvironment,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

fn belongs(run: &TextRunNode, cell: usize) -> bool {
    run.section_index == Some(0)
        && run.cell_context.as_ref().is_some_and(|context| {
            context.parent_para_index == 15
                && context.path.len() == 1
                && context.path[0].control_index == 0
                && context.path[0].cell_index == cell
                && context.path[0].cell_para_index == 0
        })
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_control())
        .collect()
}

#[test]
fn dash_restoration_keeps_word_spaces_and_original_cell_content() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(&std::fs::read_to_string(root.join(
        "mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json",
    )).unwrap()).unwrap())).unwrap();
    assert_eq!(core.page_count(), 66, "전쪽 독립 Print의 쪽수");
    let Control::Table(table) = &core.document().sections[0].paragraphs[15].controls[0] else {
        panic!("원본 조문 대비표");
    };
    let tree = core.build_page_render_tree(3).unwrap();
    for cell in [2, 3] {
        let rendered: String = nodes(&tree.root)
            .into_iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if belongs(run, cell) => Some(run.display_or_text()),
                _ => None,
            })
            .collect();
        assert_eq!(
            compact(&rendered),
            compact(&table.cells[cell].paragraphs[0].text),
            "같은 원본 셀·문단의 내용 순서·누락·중복 보존"
        );
        assert_eq!(rendered.matches('①').count(), 1, "문단 기호의 소유는 한 번");
    }
    let runs = nodes(&tree.root)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .find_map(|line| {
            let runs: Vec<_> = line
                .children
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run) if belongs(run, 3) => Some(run),
                    _ => None,
                })
                .collect();
            let text: String = runs.iter().map(|run| run.display_or_text()).collect();
            (text.contains("공개방법") && text.contains('①') && text.contains("---"))
                .then_some(runs)
        })
        .expect("같은 줄에 낱말·문단 기호·대시가 있는 원본 대비표 줄");
    let mut dash_count = 0;
    let mut space_count = 0;
    for run in runs {
        let text = run.display_or_text();
        let measured;
        let positions = if let Some(positions) = &run.layout_positions {
            positions
        } else {
            measured = EmbeddedTextMeasurer.compute_char_positions(text, &run.style);
            &measured
        };
        let chars: Vec<_> = text.chars().collect();
        assert_eq!(positions.len(), chars.len() + 1);
        for (index, ch) in chars.iter().enumerate() {
            let advance = positions[index + 1] - positions[index];
            if *ch == '-' {
                dash_count += 1;
                assert!(
                    advance <= run.style.font_size * 0.85,
                    "독립 Print의 자연 대시 전진 범위를 넘지 않는다: {advance} / {}",
                    run.style.font_size
                );
            } else if *ch == ' ' {
                space_count += 1;
                assert!(
                    advance > run.style.font_size * 0.6,
                    "대시 복원 뒤 남은 여유는 같은 줄 낱말 사이에도 배분한다: {advance}"
                );
            }
        }
    }
    // 원본 p0의 기호는 위 내용 소속 검사에서 정확히 한 번임을 확인했다.
    // 기대 좌표는 구현 계산값이 아니라 독립 Print의 H2mjsM 글리프 원점이다.
    let symbol = nodes(&tree.root)
        .into_iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if belongs(run, 3) => run
                .display_or_text()
                .chars()
                .position(|ch| ch == '①')
                .map(|index| (node, run, index)),
            _ => None,
        })
        .expect("완료된4쪽의 원본 셀 항목 기호");
    let (node, run, index) = symbol;
    let measured;
    let positions = if let Some(positions) = &run.layout_positions {
        positions
    } else {
        measured = EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style);
        &measured
    };
    let actual = (node.bbox.x + positions[index], node.bbox.y + run.baseline);
    assert!(
        (actual.0 - 544.96).abs() <= 2.0 && (actual.1 - 193.28).abs() <= 2.0,
        "독립 Print의 항목 기호 원점 ±2CSSpx: {actual:?}"
    );
    assert_eq!(dash_count, 9, "저장 줄의 대시 소속·개수");
    assert_eq!(
        space_count, 3,
        "언어 슬롯으로 나뉜 런 사이에도 공백 소속 유지"
    );
}
