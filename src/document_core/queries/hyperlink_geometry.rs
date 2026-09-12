//! 출력에 실제 포함된 TextRun의 문자 경계와 clip을 사용한 링크 영역 질의 (#6963).
use crate::document_core::{hyperlink::HyperlinkTarget, DocumentCore};
use crate::error::HwpError;
use crate::model::{
    control::{Control, FieldType},
    hyperlink::command_uri,
};
use crate::paint::{GroupKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use crate::renderer::{
    hyperlinks::{clipped_rect, export_uri, PdfLink},
    render_tree::BoundingBox,
};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageHyperlink {
    pub target: HyperlinkTarget,
    pub field_id: u32,
    /// 이 사각형에 포함된 표시 문자 범위.
    pub start: usize,
    pub end: usize,
    pub uri: String,
    pub rect: BoundingBox,
}

impl PageHyperlink {
    pub fn pdf_link(&self) -> PdfLink {
        PdfLink {
            uri: self.uri.clone(),
            rect: self.rect,
        }
    }
}

impl DocumentCore {
    /// 0-based 페이지의 실제 출력 링크 조각. 줄바꿈·서식 분할은 각각의 사각형으로 반환한다.
    pub fn page_hyperlinks_native(
        &self,
        page: u32,
        profile: RenderProfile,
    ) -> Result<Vec<PageHyperlink>, HwpError> {
        self.hyperlinks_in_layer_tree(&self.build_page_layer_tree_with_profile(page, profile)?)
    }

    /// 이미 생성한 출력 트리를 재사용한다. glyph sidecar와 fallback을 이중으로 세지 않는다.
    pub fn hyperlinks_in_layer_tree(
        &self,
        tree: &PageLayerTree,
    ) -> Result<Vec<PageHyperlink>, HwpError> {
        let mut links = Vec::new();
        let page = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: tree.page_width,
            height: tree.page_height,
        };
        self.collect_layer_hyperlinks(&tree.root, page, &mut links)?;
        Ok(links)
    }

    fn collect_layer_hyperlinks(
        &self,
        node: &LayerNode,
        clip: BoundingBox,
        links: &mut Vec<PageHyperlink>,
    ) -> Result<(), HwpError> {
        match &node.kind {
            LayerNodeKind::Group {
                children,
                group_kind,
                ..
            } => {
                // 이 컨테이너들의 para_index는 본문과 다른 좌표계다. 본문 링크를 잘못 붙이지 않는다.
                if matches!(
                    group_kind,
                    GroupKind::Header
                        | GroupKind::Footer
                        | GroupKind::MasterPage
                        | GroupKind::FootnoteArea
                ) {
                    return Ok(());
                }
                for child in children {
                    self.collect_layer_hyperlinks(child, clip, links)?;
                }
            }
            LayerNodeKind::ClipRect {
                clip: child_clip,
                child,
                ..
            } => {
                if let Some(intersection) = clipped_rect(*child_clip, clip) {
                    self.collect_layer_hyperlinks(child, intersection, links)?;
                }
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    let PaintOp::TextRun { bbox, run, .. } = op else {
                        continue;
                    };
                    let (Some(section), Some(para), Some(char_start)) =
                        (run.section_index, run.para_index, run.char_start)
                    else {
                        continue;
                    };
                    let target = if let Some(cell) = &run.cell_context {
                        if cell.path.is_empty() {
                            continue;
                        }
                        HyperlinkTarget {
                            section,
                            para: cell.parent_para_index,
                            cell_path: cell
                                .path
                                .iter()
                                .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
                                .collect(),
                        }
                    } else {
                        HyperlinkTarget::body(section, para)
                    };
                    let Ok(paragraph) = self.hyperlink_paragraph(&target) else {
                        continue;
                    };
                    let count = run.text.chars().count();
                    let Some(run_end) = char_start.checked_add(count) else {
                        continue;
                    };
                    for range in &paragraph.field_ranges {
                        let Some(Control::Field(field)) = paragraph.controls.get(range.control_idx)
                        else {
                            continue;
                        };
                        if field.field_type != FieldType::Hyperlink {
                            continue;
                        }
                        let start = char_start.max(range.start_char_idx);
                        let end = run_end.min(range.end_char_idx);
                        if start >= end {
                            continue;
                        }
                        let Some(uri) = export_uri(&command_uri(&field.command)) else {
                            continue;
                        };
                        // 보이지 않는 반복 셀 조각에는 클릭 영역을 만들지 않는다.
                        if clipped_rect(*bbox, clip).is_none() {
                            continue;
                        }
                        if run.is_vertical
                            || run.rotation != 0.0
                            || run.char_overlap.is_some()
                            || run.display_or_text().chars().count() != count
                        {
                            return Err(HwpError::RenderError("회전·세로쓰기·표시값 치환 링크의 출력 영역은 아직 지원하지 않습니다".into()));
                        }
                        // K1은 layout owner의 실제 경계, K0은 SVG/Skia와 같은 replay 경계를 사용한다.
                        let positions = run.replay_positions_for(run.display_or_text());
                        let Some((&x1, &x2)) = positions
                            .get(start - char_start)
                            .zip(positions.get(end - char_start))
                        else {
                            continue;
                        };
                        let rect = BoundingBox {
                            x: bbox.x + x1,
                            y: bbox.y,
                            width: x2 - x1,
                            height: bbox.height,
                        };
                        if let Some(rect) = clipped_rect(rect, clip) {
                            links.push(PageHyperlink {
                                target: target.clone(),
                                field_id: field.field_id,
                                start,
                                end,
                                uri,
                                rect,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
