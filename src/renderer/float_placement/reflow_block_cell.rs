//! 저장 앵커가 없는 빈 표 호스트의 줄·개체·흐름 점유를 분리한다.

use super::{hwpunit_to_px, signed_hwpunit};
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::{HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo},
    table::{Cell, Table},
};
use crate::renderer::style_resolver::ResolvedStyleSet;

#[derive(Clone, Copy)]
pub(crate) struct ReflowBlockOrigin {
    /// 제한이 꺼진 부동 표는 부모 셀과 별개인 외부 호스트 문단을 참조한다.
    pub(crate) host_relative: bool,
    /// 제한을 켜면 내용 원점 기준이며 표의 위 바깥여백까지 포함한다.
    pub(crate) top: f64,
}

pub(crate) struct ReflowBlockCellFlow {
    pub(crate) paragraph_origins: Vec<f64>,
    pub(crate) origins: Vec<Vec<Option<ReflowBlockOrigin>>>,
    /// 부동 자식의 별도 프레임을 제외한 호스트 글줄의 점유 끝.
    pub(crate) host_bottom: f64,
    pub(crate) bottom: f64,
    /// 외부 호스트 앵커를 공유하는 자식들의 점유 끝. 부모 줄 전진과 합산하지 않는다.
    pub(crate) external_bottom: Option<f64>,
    pub(crate) external_child_splittable: bool,
}

impl ReflowBlockCellFlow {
    /// 선언 최소와 실제 부모 글줄이 소유하는 동일한 물리 프레임.
    pub(crate) fn parent_frame_height(&self, cell: &Cell, table: &Table, dpi: f64) -> Option<f64> {
        let declared = i32::try_from(cell.height).ok()?;
        let padding = cell.effective_padding(&table.padding);
        Some(hwpunit_to_px(declared, dpi).max(
            self.bottom + hwpunit_to_px(i32::from(padding.top) + i32::from(padding.bottom), dpi),
        ))
    }
}

/// 하나의 물리 부모 프레임과 공동 앵커 외부 자식의 전량 점유를 구분한다.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReflowExternalTableFlow {
    pub(crate) parent_frame_height: f64,
    pub(crate) external_bottom: f64,
    pub(crate) effective_height: f64,
    pub(crate) child_splittable: bool,
}

/// 온전한 호스트와 부분 조각이 같은 빈 글줄의 점유 높이를 사용한다.
pub(crate) fn reflow_host_line_height(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    last: bool,
) -> Option<f64> {
    let em = para
        .char_shapes
        .first()
        .and_then(|shape| styles.char_styles.get(shape.char_shape_id as usize))?
        .font_size;
    if em <= 0.0 {
        return None;
    }
    Some(if last {
        em
    } else {
        styles
            .para_styles
            .get(para.para_shape_id as usize)
            .map_or(em, |style| {
                crate::renderer::corrected_line_height(
                    hwpunit_to_px(400, dpi),
                    em,
                    style.line_spacing_type,
                    style.line_spacing,
                )
            })
    })
}

/// 저장된 선행 글줄 뒤의 NO_LS 호스트도 동일한 재조판 프레임을 연다.
/// 다른 개체와 혼합된 호스트·저장 위치로 돌아오는 후속 문단은 각각의 기존
/// 줄 구성/저장 프레임 계약을 유지한다. 자식 높이는 바깥여백을 제외한 실제 본체다.
pub(crate) fn has_reflow_block_cell_host(paragraphs: &[Paragraph]) -> bool {
    let mut reflow_started = false;
    let mut has_host = false;
    for para in paragraphs {
        let no_ls = crate::renderer::para_has_no_stored_line_segs(para);
        if para.column_type != ColumnBreakType::None {
            return false;
        }
        if no_ls {
            reflow_started = true;
        } else if reflow_started
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.line_segs.iter().any(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.vertical_pos < 0
                    || line.line_height <= 0
            })
        {
            return false;
        }
        for control in &para.controls {
            match control {
                Control::Table(table)
                    if no_ls
                        && !table.common.treat_as_char
                        && table.common.text_wrap == TextWrap::TopAndBottom
                        && table.common.vert_rel_to == VertRelTo::Para
                        && table.common.vert_align == VertAlign::Top
                        && matches!(
                            table.common.horz_rel_to,
                            HorzRelTo::Para | HorzRelTo::Column
                        )
                        && table.common.horz_align == HorzAlign::Left
                        && table.caption.is_none() =>
                {
                    has_host = true;
                }
                Control::ColumnDef(columns) if columns.column_count == 1 => {}
                _ => return false,
            }
        }
    }
    has_host
}

pub(crate) fn reflow_block_cell_flow(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    table_body_height: impl FnMut(&Table) -> f64,
) -> Option<ReflowBlockCellFlow> {
    reflow_block_cell_flow_with_heights(paragraphs, styles, dpi, None, table_body_height)
}

/// 가시 NO_LS 문단은 실제 셀 폭에서 구성한 글줄의 점유를 받아야 한다.
pub(super) fn reflow_block_cell_flow_with_heights(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    text_heights: Option<&[f64]>,
    mut table_body_height: impl FnMut(&Table) -> f64,
) -> Option<ReflowBlockCellFlow> {
    match reflow_block_cell_flow_with_heights_with_storage(
        paragraphs,
        styles,
        dpi,
        text_heights,
        |table| Ok(table_body_height(table)),
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(super) fn reflow_block_cell_flow_with_heights_with_storage<S: crate::storage::Storage>(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    text_heights: Option<&[f64]>,
    mut table_body_height: impl FnMut(&Table) -> Result<f64, S::Error>,
    storage: &S,
) -> Result<Option<ReflowBlockCellFlow>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !has_reflow_block_cell_host(paragraphs) {
            return None;
        }

        // 혼합 재조판 프레임의 저장 prefix는 첫 줄을 셀의 앞 간격에 연결한다.
        // 원래 본문에서 옮겨 온 첫 vpos를 새 셀의 빈 물리 띠로 더하지 않는다.
        // 저장 줄 사이의 차이와 원본 속성에서 입증된 앞 간격은 그대로 보존한다.
        let stored_origin = paragraphs.first().map_or(0.0, |para| {
            let first = para
                .line_segs
                .first()
                .map_or(0.0, |line| hwpunit_to_px(line.vertical_pos, dpi));
            let before = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map_or(0.0, |style| style.spacing_before);
            first - crate::renderer::cell_first_para_stored_lead(para, before, dpi)
        });

        let mut flow = ReflowBlockCellFlow {
            paragraph_origins: crate::storage::optional(storage.vec(paragraphs.len()), failure)?,
            origins: crate::storage::optional(
                storage.try_collect(
                    paragraphs
                        .iter()
                        .map(|p| storage.filled(p.controls.len(), None)),
                ),
                failure,
            )?,
            host_bottom: 0.0,
            bottom: 0.0,
            external_bottom: None,
            external_child_splittable: false,
        };
        let mut cursor = 0.0_f64;
        for (pi, para) in paragraphs.iter().enumerate() {
            let style = styles.para_styles.get(para.para_shape_id as usize);
            let last = pi + 1 == paragraphs.len();
            let before = crate::renderer::cell_paragraph_spacing::paragraph_before(
                para,
                pi,
                style.map_or(0.0, |style| style.spacing_before),
                dpi,
            )
            .occupied_before();
            let after = if last {
                0.0
            } else {
                style.map_or(0.0, |style| style.spacing_after)
            };
            cursor += before;
            flow.host_bottom += before;
            let no_ls = crate::renderer::para_has_no_stored_line_segs(para);
            if !no_ls {
                let top = hwpunit_to_px(para.line_segs.first()?.vertical_pos, dpi) - stored_origin;
                if top < cursor {
                    return None;
                }
                cursor = top;
                flow.host_bottom = top;
                crate::storage::optional(
                    storage.push(&mut flow.paragraph_origins, cursor),
                    failure,
                )?;
                for (li, line) in para.line_segs.iter().enumerate() {
                    let end =
                        hwpunit_to_px(line.vertical_pos.saturating_add(line.line_height), dpi)
                            - stored_origin
                            + if !last || li + 1 < para.line_segs.len() {
                                hwpunit_to_px(line.line_spacing, dpi)
                            } else {
                                0.0
                            };
                    if end < cursor {
                        return None;
                    }
                    cursor = end;
                    flow.host_bottom = end;
                }
            } else {
                crate::storage::optional(
                    storage.push(&mut flow.paragraph_origins, cursor),
                    failure,
                )?;
                let line_height = if para.text.is_empty() {
                    reflow_host_line_height(para, styles, dpi, last)?
                } else {
                    let height = *text_heights?.get(pi)?;
                    if !height.is_finite() || height <= 0.0 {
                        return None;
                    }
                    height
                };
                let mut block_height = 0.0_f64;
                for group in crate::storage::optional(
                    super::nested_table_groups_with_storage(para, false, storage),
                    failure,
                )? {
                    let mut group_height = 0.0_f64;
                    for ci in group.controls {
                        let Control::Table(table) = &para.controls[ci] else {
                            continue;
                        };
                        let offset =
                            hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
                        let restricted = table.common.flow_with_text;
                        let preceding = if group.side_by_side {
                            0.0
                        } else {
                            group_height
                        };
                        flow.origins[pi][ci] = Some(ReflowBlockOrigin {
                            host_relative: !restricted,
                            top: if restricted {
                                cursor
                                    + block_height
                                    + preceding
                                    + offset
                                    + hwpunit_to_px(i32::from(table.outer_margin_top), dpi)
                            } else {
                                offset
                            },
                        });
                        let height = offset
                            + crate::storage::optional(table_body_height(table), failure)?
                            + hwpunit_to_px(
                                i32::from(table.outer_margin_top)
                                    + i32::from(table.outer_margin_bottom),
                                dpi,
                            );
                        if restricted {
                            group_height = if group.side_by_side {
                                group_height.max(height)
                            } else {
                                group_height + height
                            };
                        } else {
                            flow.external_bottom = Some(
                                flow.external_bottom
                                    .map_or(height, |previous| previous.max(height)),
                            );
                            flow.external_child_splittable |=
                                table.page_break != crate::model::table::TablePageBreak::None;
                        }
                    }
                    block_height += group_height;
                }
                let occupied = line_height.max(block_height);
                cursor += occupied;
                flow.host_bottom += line_height;
            }
            cursor += after;
            flow.host_bottom += after;
        }
        flow.bottom = cursor;
        Some(flow)
    })
}

/// 부모 행의 물리 프레임과 공동 앵커 자손들의 최대 점유를 함께 발행한다.
/// 일반 텍스트 형제 칸은 부모 프레임만 소유하며 외부 자식 스트림을 만들지 않는다.
pub(crate) fn supports_reflow_external_table_frame(table: &Table) -> bool {
    super::is_para_topbottom_float(&table.common)
        && !table.common.flow_with_text
        && table.common.vert_align == VertAlign::Top
        && table.caption.is_none()
        && table.row_count == 1
        && !table.cells.is_empty()
}

pub(crate) fn reflow_external_table_flow(
    table: &Table,
    dpi: f64,
    parent_frame_height: f64,
    mut cell_flow: impl FnMut(&Cell) -> Option<ReflowBlockCellFlow>,
) -> Option<ReflowExternalTableFlow> {
    match reflow_external_table_flow_with_storage(
        table,
        dpi,
        parent_frame_height,
        |cell| Ok(cell_flow(cell)),
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn reflow_external_table_flow_with_storage<S: crate::storage::Storage>(
    table: &Table,
    dpi: f64,
    parent_frame_height: f64,
    mut cell_flow: impl FnMut(&Cell) -> Result<Option<ReflowBlockCellFlow>, S::Error>,
    storage: &S,
) -> Result<Option<ReflowExternalTableFlow>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !supports_reflow_external_table_frame(table)
            || !parent_frame_height.is_finite()
            || parent_frame_height <= 0.0
        {
            return None;
        }
        let mut external_bottom = None::<f64>;
        let mut child_splittable = false;
        for cell in &table.cells {
            if cell.row_span != 1 || cell.text_direction != 0 {
                return None;
            }
            let children = cell
                .paragraphs
                .iter()
                .flat_map(|para| &para.controls)
                .filter(|control| matches!(control, Control::Table(_)))
                .count();
            if children == 0
                && cell
                    .paragraphs
                    .iter()
                    .all(crate::renderer::cell_paragraph_has_only_text_metadata)
            {
                continue;
            }
            // 한 칸의 여러 독립 자식은 일반 컷 원장이 계속 소유한다.
            if children != 1 {
                return None;
            }
            let flow = crate::storage::optional(cell_flow(cell), failure)??;
            if flow
                .origins
                .iter()
                .flatten()
                .flatten()
                .any(|origin| !origin.host_relative)
            {
                return None;
            }
            let child_end = flow.external_bottom?;
            external_bottom = Some(external_bottom.map_or(child_end, |end| end.max(child_end)));
            child_splittable |= flow.external_child_splittable;
        }
        let external_bottom = external_bottom?;
        let outside = hwpunit_to_px(
            i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
            dpi,
        );
        let offset = hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
        external_bottom
            .is_finite()
            .then_some(ReflowExternalTableFlow {
                parent_frame_height,
                external_bottom,
                effective_height: parent_frame_height.max(external_bottom - offset - outside),
                child_splittable,
            })
    })
}
