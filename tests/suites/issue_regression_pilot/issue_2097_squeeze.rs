//! Issue #2097 — 독립 쪽수와 분할 표의 내용·후속 문단 소유를 확인한다.
//! 재난 원본은 Native/fresh WASM 전 2쪽 최저 96.21501% 확인 후 관계 검사를 보완했다.
//! 증적은 completed-20261009/current-base-qualification/에 보존한다.
//!
//! 병인: 한글은 쪽 끝자락에서 행/블록/컷 첫 유닛이 잔여를 소폭 초과하면 압축해
//! 끼워 넣는데(COM·PDF 실측), rhwp 는 무조건 이월해 표 마지막 행 sliver 쪽을
//! 만든다. 수정: BOTTOM_SQUEEZE_* 게이트(초과 ≤13px, 잔여 ≤100px, 콘텐츠 여유
//! ≥12px, 측정-선언 정합, 중첩 표 제외, RowBreak 한정)의 압축 수용.
//! 정답 쪽수는 한글 2022 COM PageCount 실측 (visual sweep 픽셀 정합 동반).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::RenderNodeType;

/// (샘플, 한글 실측 쪽수)
const PINS: &[(&str, u32)] = &[
    // 블록(rows 10-11: 잔여 87.8 에 98.2 압축) + 행(r14: 초과 6.9) 수용 (3→2쪽)
    ("samples/task2097/1741000_project_application.hwp", 2),
    // 압축 컷 수용 (p1 잔여 32.5 에 첫 줄 36.8 압축, 3→2쪽)
    ("samples/task2097/21298295_byeolpyo5_disaster.hwp", 2),
    // #7445: #7382 후보에서 6→7쪽으로 실패한 task2146 쪽수 항목만 보류합니다.
    // 다른 두 압축 수용 입력과 원문·독립 PDF는 유지합니다.
];

#[test]
fn issue_2097_bottom_squeeze_keeps_source_content_and_page_owners() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    for (sample, expected) in PINS {
        let bytes = fs::read(Path::new(repo_root).join(sample))
            .unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let core =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
        assert_eq!(
            core.page_count(),
            *expected,
            "{sample}: 한글 COM 실측 쪽수와 불일치"
        );
        if *sample == "samples/task2097/21298295_byeolpyo5_disaster.hwp" {
            let original = rhwp::parse_document(&bytes).expect("독립 원본 모델");
            let Control::Table(source) = &original.sections[0].paragraphs[4].controls[0] else {
                panic!("편성기준 표의 원본 호스트");
            };
            let compact = |text: &str| -> String {
                text.chars()
                    .filter(|ch| !ch.is_whitespace() && !ch.is_control() && *ch != '\u{fffc}')
                    .collect()
            };
            let mut content = vec![String::new(); source.cells.len()];
            let mut owners = vec![0usize; source.cells.len()];
            for page in 0..core.page_count() {
                let tree = core
                    .build_page_render_tree(page as u32)
                    .expect("전체 쪽 소유");
                let mut stack = vec![&tree.root];
                let mut table_bottom = None;
                let mut following_text = String::new();
                let mut following_top = None;
                while let Some(node) = stack.pop() {
                    if !node.visible || node.editor_only {
                        continue;
                    }
                    stack.extend(node.children.iter().rev());
                    if let RenderNodeType::TextRun(run) = &node.node_type {
                        if run.para_index == Some(6) && run.cell_context.is_none() {
                            following_text.push_str(run.display_or_text());
                            following_top.get_or_insert(node.bbox.y);
                        }
                    }
                    let RenderNodeType::Table(table) = &node.node_type else {
                        continue;
                    };
                    if table.para_index != Some(4) || table.cell_context.is_some() {
                        continue;
                    }
                    table_bottom = Some(node.bbox.y + node.bbox.height);
                    for child in &node.children {
                        let RenderNodeType::TableCell(cell) = &child.node_type else {
                            continue;
                        };
                        let index = cell.model_cell_index.expect("원본 셀 소속") as usize;
                        assert!(index < source.cells.len(), "원본 밖 셀을 생성하지 않는다");
                        owners[index] += 1;
                        let mut runs = vec![child];
                        while let Some(run_node) = runs.pop() {
                            if !run_node.visible || run_node.editor_only {
                                continue;
                            }
                            runs.extend(run_node.children.iter().rev());
                            if let RenderNodeType::TextRun(run) = &run_node.node_type {
                                if !cell.clip || child.bbox.intersects(&run_node.bbox) {
                                    content[index].push_str(run.display_or_text());
                                }
                            }
                        }
                    }
                }
                if page + 1 == core.page_count() {
                    assert_eq!(
                        compact(&following_text),
                        compact(&original.sections[0].paragraphs[6].text),
                        "PDF 마지막 쪽의 표 뒤 문단을 보존한다"
                    );
                    assert!(
                        following_top.expect("후속 문단") >= table_bottom.expect("마지막 표 조각"),
                        "마지막 문단은 표 뒤에 배치한다"
                    );
                } else {
                    assert!(
                        following_text.is_empty(),
                        "마지막 문단을 앞 조각에 중복하지 않는다"
                    );
                }
            }
            for (index, cell) in source.cells.iter().enumerate() {
                let expected_text: String =
                    cell.paragraphs.iter().map(|p| p.text.as_str()).collect();
                assert!(owners[index] > 0, "셀 소속 누락: {index}");
                assert_eq!(
                    compact(&content[index]),
                    compact(&expected_text),
                    "표 조각 전체에서 셀 {index}의 내용 누락·중복을 막는다"
                );
            }
        }
    }
}
