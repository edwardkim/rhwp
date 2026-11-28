//! 관계형·고정형 #7813 거대 셀의 지연 편집, 페이지 소유권과 실제 원본 115쪽.
//! 갱신 2026-11-28 17:04 KST.
//! 측정 2026-11-28 KST, 96dpi의 색상 경계 보정 전 2px 실루엣.
//! HWP 한컴 2024 Print 전115쪽 Native 98.29% / fresh WASM 98.29% 이상.
//! HWPX 한컴 2020 Print 전115쪽 Native 98.29% / fresh WASM 98.29% 이상.
//! 양쪽 원본과 전쪽 TSV는 mydocs/pr/assets/issues7813_8100/stage3/README.md에 보존한다.

use super::HwpDocument;
use crate::document_core::DeferredPaginationJobState;
use crate::renderer::pagination::PageItem;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;
use std::fs;
use std::path::Path;

const PRINT_PAGES: u32 = 115;
const ORIGINAL_END: usize = 130;
const INSERTIONS: usize = 56;

#[derive(Debug, PartialEq, Eq)]
struct Fragment {
    start_row: usize,
    end_row: usize,
    start_cut: Vec<usize>,
    end_cut: Vec<usize>,
}

fn open(format: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/issue1949_giant_cell_nested_tables_perf.{format}"
    ));
    HwpDocument::from_bytes(&fs::read(path).expect("읽을 수 있는 원본")).expect("원본 파싱")
}

fn fragments(doc: &HwpDocument) -> Vec<Fragment> {
    let pages: Vec<_> = doc
        .core
        .pagination
        .iter()
        .flat_map(|section| &section.pages)
        .collect();
    assert_eq!(
        pages.len(),
        doc.page_count() as usize,
        "공개 쪽수와 조각 목록"
    );
    pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let mut items = page.column_contents.iter().flat_map(|column| &column.items);
            let fragment = items
                .find_map(|item| match item {
                    PageItem::PartialTable {
                        para_index: 0,
                        control_index: 2,
                        start_row,
                        end_row,
                        start_cut,
                        end_cut,
                        is_continuation,
                        is_block_split,
                        ..
                    } => {
                        assert!(!is_block_split, "문서의 거대 셀 한 행을 이어받는다");
                        assert_eq!(*is_continuation, index > 0);
                        assert!(start_row < end_row);
                        Some(Fragment {
                            start_row: *start_row,
                            end_row: *end_row,
                            start_cut: start_cut.clone(),
                            end_cut: end_cut.clone(),
                        })
                    }
                    _ => None,
                })
                .expect("쪽마다 원본 표 0.0.2의 조각이 하나 있다");
            assert!(!items.any(|item| matches!(
                item,
                PageItem::PartialTable {
                    para_index: 0,
                    control_index: 2,
                    ..
                }
            )));
            fragment
        })
        .collect()
}

fn assert_contiguous(cuts: &[Fragment]) {
    assert!(cuts.first().unwrap().start_cut.is_empty());
    assert!(cuts.last().unwrap().end_cut.is_empty());
    for (index, fragment) in cuts.iter().enumerate() {
        if let Some(next) = cuts.get(index + 1) {
            let split_row = !fragment.end_cut.is_empty();
            assert_eq!(next.start_row, fragment.end_row - usize::from(split_row));
            assert_eq!(
                fragment.end_cut,
                next.start_cut,
                "{}→{}쪽 유닛 소속",
                index + 1,
                index + 2
            );
        }
        if fragment.end_row == fragment.start_row + 1
            && !fragment.start_cut.is_empty()
            && !fragment.end_cut.is_empty()
        {
            assert_eq!(fragment.end_cut.len(), fragment.start_cut.len());
            assert!(fragment
                .end_cut
                .iter()
                .zip(&fragment.start_cut)
                .all(|(end, start)| end >= start));
            assert!(fragment
                .end_cut
                .iter()
                .zip(&fragment.start_cut)
                .any(|(end, start)| end > start));
        }
    }
}

fn target_ranges(doc: &HwpDocument) -> Vec<(usize, usize)> {
    fn collect(node: &RenderNode, ranges: &mut Vec<(usize, usize)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if let (Some(start), Some(context)) = (run.char_start, &run.cell_context) {
                if context.parent_para_index == 0
                    && context.path.as_slice().first().is_some_and(|entry| {
                        context.path.len() == 1
                            && entry.control_index == 2
                            && entry.cell_index == 2
                            && entry.cell_para_index == 5
                    })
                    && !run.text.is_empty()
                {
                    assert!(run.char_overlap.is_none());
                    ranges.push((start, start + run.text.encode_utf16().count()));
                }
            }
        }
        for child in &node.children {
            collect(child, ranges);
        }
    }
    let mut ranges = Vec::new();
    collect(&doc.build_page_render_tree(0).unwrap().root, &mut ranges);
    ranges.sort_unstable();
    let mut end = 0;
    for &(start, next) in &ranges {
        assert_eq!(start, end, "따뜻한 트리의 UTF-16 범위 누락·중복");
        end = next;
    }
    ranges
}

fn cursor(doc: &HwpDocument, offset: usize) -> Value {
    serde_json::from_str(
        &doc.get_cursor_rect_in_cell_native(0, 0, 2, 2, 5, offset)
            .unwrap(),
    )
    .unwrap()
}

fn insert(doc: &mut HwpDocument, offset: usize, text: &str) -> Value {
    serde_json::from_str(
        &doc.insert_text_in_cell_native_deferred_pagination(0, 0, 2, 2, 5, offset, text)
            .unwrap(),
    )
    .unwrap()
}

fn finish_incrementally(doc: &mut HwpDocument, published: &[Fragment]) {
    let begin = doc.core.begin_deferred_pagination(1);
    assert_eq!(begin.state, DeferredPaginationJobState::Pending);
    assert_eq!(fragments(doc), published);
    let mut processed = 0;
    for _ in 0..published.len() + 1 {
        let step = doc.core.step_deferred_pagination(1);
        assert!(step.fragments_processed <= 1);
        processed += step.fragments_processed;
        match step.state {
            DeferredPaginationJobState::Pending => assert_eq!(fragments(doc), published),
            DeferredPaginationJobState::Complete => {
                assert_eq!(processed, fragments(doc).len());
                return;
            }
            state => panic!("예상하지 않은 지연 쪽 나눔 상태 {state:?}"),
        }
    }
    panic!("조각을 모두 처리한 뒤에도 페이지를 발행하지 않았다");
}

#[test]
fn scoped_edit_refreshes_the_warm_cell_and_preserves_published_cuts() {
    for format in ["hwp", "hwpx"] {
        let mut doc = open(format);
        assert_eq!(doc.page_count(), PRINT_PAGES, "한컴 Print 전쪽");
        assert_eq!(target_ranges(&doc).last().unwrap().1, ORIGINAL_END);
        let _ = cursor(&doc, ORIGINAL_END);
        let published = fragments(&doc);
        assert_contiguous(&published);

        let mut changed_flow = false;
        for inserted in 0..INSERTIONS {
            changed_flow |=
                insert(&mut doc, ORIGINAL_END + inserted, "1")["cellFlowChanged"] == true;
        }
        assert!(changed_flow, "입력이 실제 새 줄을 만들었다");
        assert_eq!(
            fragments(&doc),
            published,
            "편집만으로 공개 쪽 나눔을 바꾸지 않는다"
        );
        let end = ORIGINAL_END + INSERTIONS;
        let warm = target_ranges(&doc);
        assert_eq!(warm.last().unwrap().1, end);
        let warm_cursor = cursor(&doc, end);

        assert_eq!(
            doc.core.flush_deferred_pagination().state,
            DeferredPaginationJobState::Complete
        );
        let flushed = fragments(&doc);
        assert_contiguous(&flushed);
        assert_eq!(
            published[0], flushed[0],
            "첫 물리 조각 안에서 새 줄을 흡수한다"
        );
        assert_ne!(
            published[1..],
            flushed[1..],
            "뒤 조각은 발행 시점에 다시 나눈다"
        );
        assert_eq!(target_ranges(&doc), warm);
        assert_eq!(cursor(&doc, end), warm_cursor);
    }
}

#[test]
fn insert_and_delete_publish_the_same_cuts_as_a_synchronous_flush() {
    for format in ["hwp", "hwpx"] {
        let mut progressive = open(format);
        let mut reference = open(format);
        let inserted = "1".repeat(INSERTIONS);
        for doc in [&mut progressive, &mut reference] {
            assert_eq!(
                insert(doc, ORIGINAL_END, &inserted)["cellFlowChanged"],
                true
            );
        }
        let original = fragments(&progressive);
        finish_incrementally(&mut progressive, &original);
        assert_eq!(
            reference.core.flush_deferred_pagination().state,
            DeferredPaginationJobState::Complete
        );
        let expanded = fragments(&progressive);
        assert_contiguous(&expanded);
        assert_eq!(expanded, fragments(&reference));
        assert_ne!(expanded, original);

        for doc in [&mut progressive, &mut reference] {
            let value = doc
                .delete_text_in_cell_native_deferred_pagination(
                    0,
                    0,
                    2,
                    2,
                    5,
                    ORIGINAL_END,
                    INSERTIONS,
                )
                .unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&value).unwrap()["cellFlowChanged"],
                true
            );
        }
        assert_eq!(fragments(&progressive), expanded);
        finish_incrementally(&mut progressive, &expanded);
        assert_eq!(
            reference.core.flush_deferred_pagination().state,
            DeferredPaginationJobState::Complete
        );
        let restored = fragments(&progressive);
        assert_contiguous(&restored);
        assert_eq!(restored, fragments(&reference));
        assert_ne!(restored, expanded);
    }
}

#[test]
fn new_edit_stales_the_previous_job_and_flush_uses_the_latest_edit() {
    for format in ["hwp", "hwpx"] {
        let mut doc = open(format);
        let mut reference = open(format);
        let inserted = "1".repeat(INSERTIONS);
        insert(&mut doc, ORIGINAL_END, &inserted);
        let first = doc.core.begin_deferred_pagination(1);
        assert_eq!(first.state, DeferredPaginationJobState::Pending);
        assert_eq!(
            doc.core.step_deferred_pagination(1).state,
            DeferredPaginationJobState::Pending
        );
        let published = fragments(&doc);
        insert(&mut doc, ORIGINAL_END + INSERTIONS, "1");
        let stale = doc.core.step_deferred_pagination(1);
        assert_eq!(stale.state, DeferredPaginationJobState::Stale);
        assert_eq!(stale.revision, first.revision);
        assert_eq!(fragments(&doc), published);
        let second = doc.core.begin_deferred_pagination(1);
        assert_eq!(second.state, DeferredPaginationJobState::Pending);
        assert!(second.revision > first.revision);
        assert!(doc.core.cancel_deferred_pagination());
        assert!(!doc.core.cancel_deferred_pagination());
        let flushed = doc.core.flush_deferred_pagination();
        assert_eq!(flushed.state, DeferredPaginationJobState::Complete);
        assert_eq!(flushed.revision, second.revision);

        insert(&mut reference, ORIGINAL_END, &inserted);
        insert(&mut reference, ORIGINAL_END + INSERTIONS, "1");
        assert_eq!(
            reference.core.flush_deferred_pagination().state,
            DeferredPaginationJobState::Complete
        );
        let cuts = fragments(&doc);
        assert_contiguous(&cuts);
        assert_eq!(cuts, fragments(&reference));
        assert_eq!(target_ranges(&doc), target_ranges(&reference));
    }
}
