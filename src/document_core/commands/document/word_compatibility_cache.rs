//! HWPX를 쓴 뒤 한컴이 Word 기본값으로 다시 조판할 때 사라지는 HWP의 개별 줄 설정.
use crate::document_core::DocumentCore;
use crate::model::document::Document;
use crate::model::document_compatibility::{CompatibilityTarget, DocumentCompatibility};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::composer::composed_line_text_starts;
use crate::renderer::px_to_hwpunit;
use crate::renderer::render_tree::RenderNodeType;
use std::collections::BTreeMap;

fn needs_saved_rows(para: &Paragraph) -> bool {
    para.serializable_line_segs()
        .iter()
        .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
}

impl DocumentCore {
    /// 한컴 HWPX는 Word 대상을 읽을 때 자체 프리셋을 사용한다. 한컴의 HWP→HWPX처럼
    /// 다른 호환 설정에서 나온 본문 줄은 실제 쪽·단 좌표로 내보낸다. 문서 원본과
    /// 이미 저장된 줄은 보존한다. 구역 누적 합성 좌표를 HWPX 원본 좌표로 쓰지 않는다.
    pub(super) fn write_word_compatibility_rows(&self, document: &mut Document) {
        let settings = DocumentCompatibility::from_records(&document.doc_info);
        if settings.target != CompatibilityTarget::MsWord
            || settings == settings.target.preset()
            || !document
                .sections
                .iter()
                .flat_map(|section| &section.paragraphs)
                .any(needs_saved_rows)
        {
            return;
        }

        let hu = |px| px_to_hwpunit(px, self.dpi);
        let mut frames = BTreeMap::new();
        for page in 0..self.page_count() {
            let Ok(tree) = self.build_page_tree_cached(page) else {
                return;
            };
            for body in tree
                .root
                .children
                .iter()
                .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
            {
                for column in &body.children {
                    if !matches!(column.node_type, RenderNodeType::Column(_)) {
                        continue;
                    }
                    for node in &column.children {
                        let RenderNodeType::TextLine(line) = &node.node_type else {
                            continue;
                        };
                        let (Some(s), Some(p), Some(i)) =
                            (line.section_index, line.para_index, line.line_index)
                        else {
                            continue;
                        };
                        let Some(para) = document
                            .sections
                            .get(s)
                            .and_then(|section| section.paragraphs.get(p))
                        else {
                            continue;
                        };
                        if needs_saved_rows(para) {
                            let row = line.source_flow.map(|(char_start, spacing)| {
                                let height = hu(line.line_height);
                                let segment = LineSeg {
                                    text_start: 0,
                                    vertical_pos: hu(node.bbox.y - column.bbox.y),
                                    line_height: height,
                                    text_height: height,
                                    baseline_distance: hu(line.baseline),
                                    line_spacing: hu(spacing),
                                    column_start: hu(node.bbox.x - column.bbox.x),
                                    segment_width: hu(node.bbox.width),
                                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                                };
                                (char_start, segment)
                            });
                            frames
                                .entry((s, p))
                                .or_insert_with(Vec::new)
                                .push((i as usize, row));
                        }
                    }
                }
            }
        }
        for ((s, p), placed) in frames {
            // 같은 줄이 반복되거나 줄의 원본 소유권을 모르면 불완전한 캐시를 쓰지 않는다.
            let Some(rows): Option<Vec<_>> = placed
                .into_iter()
                .enumerate()
                .map(|(expected, (index, row))| row.filter(|_| index == expected))
                .collect()
            else {
                continue;
            };
            let para = &mut document.sections[s].paragraphs[p];
            let starts = composed_line_text_starts(
                para,
                rows.iter().map(|(start, _)| *start),
                &self.styles,
                self.dpi,
            );
            let rows: Vec<_> = rows
                .into_iter()
                .zip(starts)
                .map(|((_, mut row), start)| {
                    row.text_start = start;
                    row
                })
                .collect();
            para.source_line_seg_vertical_pos =
                Some(rows.iter().map(|line| line.vertical_pos).collect());
            para.layout_only_fill_lines = 0;
            para.line_segs = rows;
        }
    }
}
