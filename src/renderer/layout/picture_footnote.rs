//! 그림/캡션 레이아웃 + 각주 영역 레이아웃

use super::super::composer::{compose_paragraph, ComposedLine, ComposedParagraph};
use super::super::page_layout::LayoutRect;
use super::super::pagination::{FootnoteFragment, FootnoteRef, FootnoteSource};
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::super::{
    format_number, hwpunit_to_px, AutoNumberCounter, LineStyle, NumberFormat as NumFmt, StrokeDash,
    TextStyle,
};
use super::border_rendering::border_width_to_px;
use super::text_measurement::{estimate_text_width, resolved_to_text_style};
use super::utils::{
    extract_shape_transform, find_bin_data_bytes, picture_data_is_unusable, picture_display_size_hu,
};
use super::{footnote_separator_length_px, LayoutEngine, ParagraphVerticalSpacing};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::footnote::{FootnoteShape, NumberFormat};
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    Caption, CaptionDirection, CommonObjAttr, HorzAlign, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::{Alignment, LineSpacingType};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FootnoteParagraphRoute {
    Numbered,
    Plain,
    StoredEmpty,
}

struct FootnoteParagraphPlacement {
    route: FootnoteParagraphRoute,
    spacing: ParagraphVerticalSpacing,
}

fn fragment_line_bounds(fragment: Option<FootnoteFragment>, total_lines: usize) -> (usize, usize) {
    match fragment {
        Some(fragment) => {
            let start = fragment.start_line.min(total_lines);
            let end = fragment.end_line.clamp(start, total_lines);
            (start, end)
        }
        None => (0, total_lines),
    }
}

fn fragment_draws_separator(fragment: Option<FootnoteFragment>) -> bool {
    fragment
        .map(|fragment| fragment.draw_separator)
        .unwrap_or(true)
}

fn fragment_draws_number(fragment: Option<FootnoteFragment>) -> bool {
    fragment
        .map(|fragment| fragment.draw_number)
        .unwrap_or(true)
}

/// [#6034] 저장 LINE_SEG 가 없는 각주 문단은 45자 고정 휴리스틱(compose_lines
/// 폴백) 대신 **각주 영역 폭**으로 재조판해 compose 한다. 한글 6.x 대 저장본은
/// 각주 문단에 LINE_SEG 를 저장하지 않는데, 폴백 폭과 실제 영역 폭의 차가 각주
/// 블록을 부풀려(2912735: rhwp 15줄 vs 한글 9줄) bottom-anchor 인 영역 상단이
/// 본문 마지막 줄 위로 올라가 구분선이 글줄을 관통했다. 편집 경로
/// (footnote_ops)의 reflow 계약과 같은 상자를 쓴다. LINE_SEG 가 있으면 종전
/// 그대로 통과한다.
fn compose_footnote_paragraph(
    para: &Paragraph,
    content_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> crate::renderer::composer::ComposedParagraph {
    if para.line_segs.is_empty() && !para.text.is_empty() && content_width_px > 0.0 {
        let para_style = styles.para_styles.get(para.para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (content_width_px - margin_left - margin_right).max(0.0);
        if final_width > 0.0 {
            let mut owned = para.clone();
            crate::renderer::composer::reflow_line_segs(
                &mut owned,
                crate::renderer::composer::ParagraphBox::content_width_px(final_width, dpi),
                styles,
                dpi,
            );
            return crate::renderer::composer::compose_paragraph_in_context(&owned, styles);
        }
    }
    let mut composed = crate::renderer::composer::compose_paragraph_in_context(para, styles);
    // 각주 영역에 실제 등록된 원본 빈 줄도 저장 높이로 측정한다.
    if composed.lines.is_empty()
        && para.text.is_empty()
        && para.controls.is_empty()
        && !para.stored_text_partition_is_dirty()
    {
        if let [line] = para.line_segs.as_slice() {
            if line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.line_height > 0
            {
                composed.lines.push(ComposedLine {
                    runs: Vec::new(),
                    line_height: line.line_height,
                    baseline_distance: line.baseline_distance,
                    segment_width: line.segment_width as i32,
                    column_start: line.column_start,
                    line_spacing: line.line_spacing,
                    has_line_break: false,
                    char_start: 0,
                });
            }
        }
    }
    composed
}

/// 원본 각주 본문의 선두 자동 번호 슬롯과 형식을 보존한다.
fn stored_footnote_number_prefix(para: &Paragraph, number: u16) -> Option<(String, usize)> {
    if para.stored_text_partition_is_dirty()
        || para.line_segs.is_empty()
        || para.line_segs.iter().any(|line| {
            line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        })
    {
        return None;
    }
    let positions = para.control_text_positions();
    let mut prefix = String::new();
    for (index, control) in para.controls.iter().enumerate() {
        let Control::AutoNumber(auto) = control else {
            return None;
        };
        if auto.number_type != crate::model::control::AutoNumberType::Footnote
            || auto.superscript
            || auto.format > 8
            || positions.get(index) != Some(&index)
            || para.text.chars().nth(index) != Some(' ')
        {
            return None;
        }
        if auto.prefix_char != '\0' {
            prefix.push(auto.prefix_char);
        }
        prefix.push_str(&format_number(number, NumFmt::from_hwp_format(auto.format)));
        if auto.suffix_char != '\0' {
            prefix.push(auto.suffix_char);
        }
    }
    Some((prefix, para.controls.len()))
}

/// 각주 번호를 문단 첫 줄에 실은 조합 결과를 만든다.
///
/// 각주 문단은 번호 자리에 autoNum 컨트롤을 두고, 파서는 그 자리를 공백 1자로
/// 남긴다. 머리말·꼬리말 쪽번호(#3216)와 같이 모델 글자는 그대로 두고 `display_text`
/// 만 번호로 바꿔, 글자 인덱스와 줄 나눔을 유지한 채 일반 문단 경로가 내어쓰기·정렬을
/// 적용하게 한다. 자리표시 뒤의 원문 공백이 번호와 본문 사이를 맡으므로 번호 서식의
/// 끝 공백은 뺀다(한/글: `78) CFR`).
///
/// 자리표시가 없는 문단(편집으로 만든 각주 등)은 첫 줄 앞에 번호 run 을 붙이고, 글자
/// 모양은 종전 번호 경로와 같은 문단 첫 글자 모양을 쓴다. 첫 줄을 그리지 않는 조각
/// (`selected_start > 0`)이나 조합 줄이 없는 빈 문단은 `None` 으로 종전 경로에 맡긴다.
fn footnote_composed_with_number(
    para: &Paragraph,
    composed: &ComposedParagraph,
    number_text: &str,
    selected_start: usize,
    base_cs_id: u32,
) -> Option<ComposedParagraph> {
    if selected_start != 0
        || composed
            .lines
            .first()
            .is_none_or(|line| line.runs.is_empty())
    {
        return None;
    }
    let mut numbered = composed.clone();
    let line = numbered.lines.first_mut()?;
    let positions = para.control_text_positions();
    let placeholder = para
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::AutoNumber(_) => positions.get(index).copied(),
            _ => None,
        });
    let mut run_start = line.char_start;
    let mut target = None;
    if let Some(position) = placeholder {
        for (run_index, run) in line.runs.iter().enumerate() {
            let len = run.text.chars().count();
            if (run_start..run_start + len).contains(&position)
                && run.text.chars().nth(position - run_start) == Some(' ')
            {
                target = Some((run_index, position - run_start));
                break;
            }
            run_start += len;
        }
    }
    match target {
        Some((run_index, local)) => {
            let run = line.runs.remove(run_index);
            let chars: Vec<char> = run.text.chars().collect();
            let mut pieces = Vec::with_capacity(3);
            if local > 0 {
                let mut before = run.clone();
                before.text = chars[..local].iter().collect();
                before.display_text = None;
                pieces.push(before);
            }
            let mut number = run.clone();
            number.text = " ".to_string();
            number.display_text = Some(number_text.trim_end().to_string());
            pieces.push(number);
            if local + 1 < chars.len() {
                let mut after = run;
                after.text = chars[local + 1..].iter().collect();
                after.display_text = None;
                pieces.push(after);
            }
            for (offset, piece) in pieces.into_iter().enumerate() {
                line.runs.insert(run_index + offset, piece);
            }
        }
        None => {
            let mut number = line.runs.first().cloned().unwrap_or_default();
            number.text = number_text.to_string();
            number.char_style_id = base_cs_id;
            number.display_text = None;
            number.footnote_marker = None;
            number.char_overlap = None;
            number.inserted_control_text = true;
            line.runs.insert(0, number);
        }
    }
    Some(numbered)
}

fn footnote_composed_line_count(
    paragraphs: &[Paragraph],
    content_width_px: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> usize {
    paragraphs
        .iter()
        .map(|paragraph| {
            compose_footnote_paragraph(paragraph, content_width_px, styles, dpi)
                .lines
                .len()
                .max(1)
        })
        .sum()
}

/// [#6866] 회전 프레임 그림의 노드 상자 — `pic`(회전 전 비트맵)과 **중심이 같은**
/// `size` 상자를 만든다. `size` 가 `pic` 과 같으면 원래 상자 그대로다.
fn rotated_frame_node_box(
    pic_x: f64,
    pic_y: f64,
    pic_width: f64,
    pic_height: f64,
    size: (f64, f64),
) -> BoundingBox {
    let (width, height) = size;
    let center_x = pic_x + pic_width / 2.0;
    let center_y = pic_y + pic_height / 2.0;
    BoundingBox::new(
        center_x - width / 2.0,
        center_y - height / 2.0,
        width,
        height,
    )
}

impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_picture(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        bin_data_content: &[BinDataContent],
        alignment: Alignment,
        section_index: Option<usize>,
        para_index: Option<usize>,
        control_index: Option<usize>,
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
        // [#6284] 캡션 문단 조판에 필요하다.
        styles: &ResolvedStyleSet,
    ) {
        // [Task #825] 본문 picture 경로 — header_footer_ref = None
        self.layout_picture_full(
            tree,
            parent_node,
            picture,
            container,
            bin_data_content,
            alignment,
            section_index,
            para_index,
            control_index,
            None,
            cell_ctx,
            styles,
        );
    }

    /// [Task #825] 머리말/꼬리말 picture 전용 — outer Header/Footer 위치 marker 전달.
    /// `header_footer_ref` 가 `Some` 일 때 ImageNode 에 마커 설정 → rhwp-studio
    /// 머리말/꼬리말 그림 클릭 hit-test + 개체 속성 dialog dispatch 활성화.
    ///
    /// [Task #1151 v4] `cell_ctx` 가 `Some` 일 때 ImageNode 의 cell_index 설정 +
    /// tac=true 인 경우 `inline_shape_positions` 등록. studio findPictureAtClick 가
    /// cellIdx 인식하여 셀 안 picture 의 클릭 hit-test 정상 동작.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_picture_full(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        bin_data_content: &[BinDataContent],
        alignment: Alignment,
        section_index: Option<usize>,
        para_index: Option<usize>,
        control_index: Option<usize>,
        header_footer_ref: Option<crate::renderer::render_tree::HeaderFooterImageRef>,
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
        // [#6284] 캡션 문단 조판에 필요하다.
        styles: &ResolvedStyleSet,
    ) {
        // 그림 크기 (HWPUNIT → 픽셀)
        // 회전 picture에서 common.width/height는 한컴이 저장한 회전 후 외접 프레임이고
        // current_width/current_height는 실제로 회전시킬 원본 표시 크기다. common 프레임을
        // 다시 회전시키면 다단/셀 경계를 침범하므로, current 이미지를 common 프레임 중앙에 둔다.
        let rotation = picture.shape_attr.rotation_angle.rem_euclid(360);
        let uses_rotated_frame = rotation != 0
            && picture.shape_attr.current_width > 0
            && picture.shape_attr.current_height > 0
            && picture.common.width > 0
            && picture.common.height > 0;
        let (pic_width_hu, pic_height_hu) = if uses_rotated_frame {
            (
                picture.shape_attr.current_width as i32,
                picture.shape_attr.current_height as i32,
            )
        } else {
            picture_display_size_hu(picture)
        };
        let mut pic_width = hwpunit_to_px(pic_width_hu, self.dpi);
        let mut pic_height = hwpunit_to_px(pic_height_hu, self.dpi);
        let mut frame_width = if uses_rotated_frame {
            hwpunit_to_px(picture.common.width as i32, self.dpi)
        } else {
            pic_width
        };
        let mut frame_height = if uses_rotated_frame {
            hwpunit_to_px(picture.common.height as i32, self.dpi)
        } else {
            pic_height
        };

        // 글자처럼 표 셀 그림은 저장 frame으로 그린 뒤 TableCell clip에 맡긴다. 한/글은
        // 셀 안쪽 폭에 맞춰 비율 축소하지 않으며, 넘치는 오른쪽·아래만 셀 경계에서 자른다.
        // 먼저 축소하면 #7333 31쪽 스크린샷과 그 안의 주석 도형이 함께 작아진다. 그 밖의
        // 그림은 기존처럼 컨테이너를 넘지 않도록 비율 유지 축소한다.
        let preserve_inline_cell_frame = picture.common.treat_as_char && cell_ctx.is_some();
        if !preserve_inline_cell_frame {
            // 회전 프레임과 실제 이미지를 같은 비율로 축소해야 중심/회전축이 유지된다.
            if container.width > 0.0 && frame_width > container.width {
                let scale = container.width / frame_width;
                frame_width = container.width;
                frame_height *= scale;
                pic_width *= scale;
                pic_height *= scale;
            }
            if container.height > 0.0 && frame_height > container.height {
                let scale = container.height / frame_height;
                frame_height = container.height;
                frame_width *= scale;
                pic_height *= scale;
                pic_width *= scale;
            }
        }

        // [#6866] **회전 프레임 그림의 노드 상자는 `frame`(= 선언 상자)이다.**
        //
        // 페인터는 `ShapeTransform::effective_image_bbox` 로 90/270° 회전 그림의
        // bbox 가로세로를 **먼저 뒤집은 뒤** 회전을 건다(이중회전 방지, shot 05).
        // 즉 페인터는 노드 상자가 **회전 후** 상자라고 전제한다. 그런데 위에서
        // `pic_*`(= `current_*`, 회전 **전** 비트맵)로 상자를 내면 그 전제가 깨져
        // 뒤집기가 한 번 더 걸린다 — 선언 29.3×53.7 이 53.7×29.3 으로 눕는다
        // (156627451 12쪽 `angle=270`, 같은 문서 `angle=0` 15장은 정상).
        //
        // 두 상자는 중심이 같으므로, 노드에 `frame` 을 실으면 페인터의 뒤집기가
        // 정확히 `pic` 을 만들어 내고 회전이 다시 `frame` 으로 돌려놓는다.
        // 두 상자는 중심이 같으므로 원점도 그 중심에서 되짚는다.
        let node_box_size = if uses_rotated_frame {
            (frame_width, frame_height)
        } else {
            (pic_width, pic_height)
        };

        // [#6284] 캡션 띠 — 그림이 차지하는 블록은 그림 + 캡션이다.
        //
        // 이 계약은 형제 함수 `layout_body_picture` 에 이미 있었는데, 본문 그림을
        // 실제로 그리는 이 경로에는 없었다. 그래서 `side="TOP"` 캡션이 통째로
        // 사라지고 그림이 캡션 띠만큼(≈15.6~20px) 위로 올라왔다
        // (156562502: 캡션 17개 전부 TOP, 그림 9개가 3pt 초과로 어긋남).
        let caption_band_height =
            crate::renderer::composer::caption_height_px(&picture.caption, self.dpi);
        let caption_spacing = picture
            .caption
            .as_ref()
            .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
            .unwrap_or(0.0);
        let (caption_top_offset, caption_left_offset, caption_band_width) =
            match picture.caption.as_ref().map(|c| c.direction) {
                Some(CaptionDirection::Top) => (caption_band_height + caption_spacing, 0.0, 0.0),
                Some(CaptionDirection::Left) => {
                    let cw = picture
                        .caption
                        .as_ref()
                        .map(|c| hwpunit_to_px(c.width as i32, self.dpi))
                        .unwrap_or(0.0);
                    (0.0, cw + caption_spacing, cw + caption_spacing)
                }
                Some(CaptionDirection::Right) => {
                    let cw = picture
                        .caption
                        .as_ref()
                        .map(|c| hwpunit_to_px(c.width as i32, self.dpi))
                        .unwrap_or(0.0);
                    (0.0, 0.0, cw + caption_spacing)
                }
                _ => (0.0, 0.0, 0.0),
            };
        // Bottom 캡션은 그림을 밀지 않고 블록 높이만 늘린다.
        let caption_block_extra_height = match picture.caption.as_ref().map(|c| c.direction) {
            Some(CaptionDirection::Top) | Some(CaptionDirection::Bottom) => {
                caption_band_height + caption_spacing
            }
            _ => 0.0,
        };
        let block_width = frame_width + caption_band_width;
        let block_height = frame_height + caption_block_extra_height;

        // 그림 위치: non-TAC 이미지는 common 속성의 offset 적용
        // 머리말/꼬리말에서 vert=Paper는 상단여백(header area) 기준
        let (pic_x, pic_y) = if !picture.common.treat_as_char {
            let h_offset = hwpunit_to_px(picture.common.horizontal_offset as i32, self.dpi);
            let v_offset = hwpunit_to_px(picture.common.vertical_offset as i32, self.dpi);
            let frame_x = match picture.common.horz_align {
                HorzAlign::Left | HorzAlign::Inside => container.x + h_offset,
                HorzAlign::Center => container.x + (container.width - block_width) / 2.0 + h_offset,
                HorzAlign::Right | HorzAlign::Outside => {
                    container.x + container.width - block_width - h_offset
                }
            };
            let frame_y = match picture.common.vert_align {
                VertAlign::Top | VertAlign::Inside => container.y + v_offset,
                VertAlign::Center => {
                    container.y + (container.height - block_height) / 2.0 + v_offset
                }
                VertAlign::Bottom | VertAlign::Outside => {
                    container.y + container.height - block_height - v_offset
                }
            };
            (
                frame_x + caption_left_offset + (frame_width - pic_width) / 2.0,
                frame_y + caption_top_offset + (frame_height - pic_height) / 2.0,
            )
        } else {
            let frame_x = match alignment {
                Alignment::Center | Alignment::Distribute => {
                    container.x + (container.width - block_width).max(0.0) / 2.0
                }
                Alignment::Right => container.x + (container.width - block_width).max(0.0),
                _ => container.x,
            };
            (
                frame_x + caption_left_offset + (frame_width - pic_width) / 2.0,
                container.y + caption_top_offset + (frame_height - pic_height) / 2.0,
            )
        };

        // BinData에서 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let bin_data_id = picture.image_attr.bin_data_id;
        let image_data = find_bin_data_bytes(bin_data_content, bin_data_id);
        // [Task #2225] 그림 미지정(bin 참조 실패 + 외부 경로 없음): 한컴은 편집기
        // 에서만 점선 테두리+그림-없음 아이콘으로 표시하고 인쇄 등가 출력은
        // 미출력 — 의미 노드(MissingPicture)로 방출해 백엔드별 분기를 일원화.
        if picture_data_is_unusable(image_data.as_deref())
            && picture.image_attr.external_path.is_none()
        {
            let ph_id = tree.next_id();
            parent_node.children.push(RenderNode::new(
                ph_id,
                RenderNodeType::Placeholder(
                    // [Task #2230] 문서 좌표 + 셀 경로 배선 — 편집 뷰 클릭
                    // 선택·그림 지정의 대상 특정에 사용.
                    crate::renderer::render_tree::PlaceholderNode::missing_picture(
                        section_index,
                        para_index,
                        control_index,
                        cell_ctx.cloned(),
                    ),
                ),
                rotated_frame_node_box(pic_x, pic_y, pic_width, pic_height, node_box_size),
            ));
            return;
        }

        // 그림 자르기: crop 좌표를 그대로 저장 (렌더러에서 이미지 px 크기와 비교)
        let crop = {
            let c = &picture.crop;
            if c.right > c.left
                && c.bottom > c.top
                && (c.left != 0 || c.top != 0 || c.right != 0 || c.bottom != 0)
            {
                Some((c.left, c.top, c.right, c.bottom))
            } else {
                None
            }
        };

        // crop 좌표 기준 범위(imgDim). orgSz는 개체 크기이므로 사용하지 않는다.
        let original_size_hu = picture.crop_reference_size();

        // 이미지 노드 생성
        // [Task #1151 v7 항목 1] cell_ctx 의 3 필드 매핑은 CellContext::last_image_indices()
        // 로 통합 (이전: 각 필드마다 path.last() 호출 반복).
        let (cei, cpi, otci) = cell_ctx
            .map(|c| c.last_image_indices())
            .unwrap_or((None, None, None));
        let img_id = tree.next_id();
        let img_node = RenderNode::new(
            img_id,
            RenderNodeType::Image(ImageNode {
                section_index,
                para_index,
                control_index,
                crop,
                original_size_hu,
                effect: picture.image_attr.effect,
                brightness: picture.image_attr.brightness,
                contrast: picture.image_attr.contrast,
                opacity: picture.image_attr.opacity(),
                text_wrap: Some(picture.common.text_wrap),
                transform: extract_shape_transform(&picture.shape_attr),
                external_path: picture.image_attr.external_path.clone(),
                content_inset: crate::renderer::layout::utils::picture_content_inset(picture),
                header_footer_ref: header_footer_ref.clone(),
                cell_index: cei,
                cell_para_index: cpi,
                outer_table_control_index: otci,
                // [Task #1161] 전체 다단계 경로 보존(스칼라는 위 innermost 투영).
                cell_context: cell_ctx.cloned(),
                ..ImageNode::new(bin_data_id, image_data)
            }),
            rotated_frame_node_box(pic_x, pic_y, pic_width, pic_height, node_box_size),
        );

        parent_node.children.push(img_node);

        // [#6284] 캡션 렌더링 — 기하는 위에서 예약한 띠와 같은 산식이다
        // (`layout_body_picture` 의 캡션 방출부와 같은 규칙).
        if let Some(ref caption) = picture.caption {
            use crate::model::shape::CaptionVertAlign;
            let (cap_x, cap_w, cap_y) = match caption.direction {
                CaptionDirection::Top => (
                    pic_x,
                    pic_width,
                    (pic_y - caption_top_offset).max(container.y),
                ),
                CaptionDirection::Bottom => {
                    (pic_x, pic_width, pic_y + pic_height + caption_spacing)
                }
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    let cx = if caption.direction == CaptionDirection::Left {
                        pic_x - caption_left_offset
                    } else {
                        pic_x + pic_width + caption_spacing
                    };
                    let cy = match caption.vert_align {
                        CaptionVertAlign::Top => pic_y,
                        CaptionVertAlign::Center => {
                            pic_y + (pic_height - caption_band_height).max(0.0) / 2.0
                        }
                        CaptionVertAlign::Bottom => {
                            pic_y + (pic_height - caption_band_height).max(0.0)
                        }
                    };
                    (cx, cw, cy)
                }
            };
            let caption_cell_ctx = cell_ctx.cloned().or_else(|| {
                para_index.map(|pi| super::CellContext {
                    in_textbox: false,
                    parent_para_index: pi,
                    path: vec![super::CellPathEntry {
                        control_index: control_index.unwrap_or(0),
                        cell_index: 0,
                        cell_para_index: 0,
                        text_direction: 0,
                    }],
                })
            });
            self.layout_caption(
                tree,
                parent_node,
                caption,
                styles,
                container,
                cap_x,
                cap_w,
                cap_y,
                &mut self.auto_counter.borrow_mut(),
                bin_data_content,
                caption_cell_ctx,
                CaptionOwner::new(
                    section_index,
                    para_index,
                    control_index,
                    CaptionControlKind::Image,
                ),
            );
        }

        // [Task #1151 v4] tac=true 셀 안 picture 의 위치를 inline_shape_positions 에 등록 →
        // cursor_rect 의 hit-test 루프가 picture 클릭 인식. 셀 외부 / 본문 picture 는
        // 기존 register path (paragraph_layout) 가 처리하므로 cell_ctx Some + tac=true
        // 인 경우만.
        if picture.common.treat_as_char {
            if let (Some(sec), Some(para_for_layout), Some(ctrl)) =
                (section_index, para_index, control_index)
            {
                tree.set_inline_shape_position(sec, para_for_layout, ctrl, cell_ctx, pic_x, pic_y);
            }
        }

        // 그림 테두리(선) 렌더링
        self.render_picture_border(
            tree,
            parent_node,
            picture,
            pic_x,
            pic_y,
            pic_width,
            pic_height,
        );
    }

    /// 개체(Picture/Shape)의 절대 좌표 (x, y)를 계산한다.
    /// HWP 스펙에 따른 VertRelTo/HorzRelTo/VertAlign/HorzAlign/treat_as_char 처리를 통합한 단일 함수.
    /// 상대 크기(WidthCriterion/HeightCriterion)를 적용하여 실제 px 크기를 반환한다.
    pub(crate) fn resolve_object_size(
        &self,
        common: &CommonObjAttr,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
    ) -> (f64, f64) {
        use crate::model::shape::SizeCriterion;

        let raw_w = common.width as f64;
        let raw_h = common.height as f64;

        let obj_width = match common.width_criterion {
            SizeCriterion::Absolute => hwpunit_to_px(common.width as i32, self.dpi),
            SizeCriterion::Paper => paper_area.width * raw_w / 10000.0,
            SizeCriterion::Page => body_area.width * raw_w / 10000.0,
            SizeCriterion::Column => col_area.width * raw_w / 10000.0,
            SizeCriterion::Para => col_area.width * raw_w / 10000.0,
        };

        let obj_height = match common.height_criterion {
            SizeCriterion::Absolute => hwpunit_to_px(common.height as i32, self.dpi),
            SizeCriterion::Paper => paper_area.height * raw_h / 10000.0,
            SizeCriterion::Page => body_area.height * raw_h / 10000.0,
            _ => hwpunit_to_px(common.height as i32, self.dpi),
        };

        (obj_width, obj_height)
    }

    pub(crate) fn compute_object_position(
        &self,
        common: &CommonObjAttr,
        obj_width: f64,
        obj_height: f64,
        container: &LayoutRect,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        para_y: f64,
        alignment: Alignment,
    ) -> (f64, f64) {
        crate::renderer::float_placement::ObjectPlacementFrame {
            container,
            column: col_area,
            body: body_area,
            paper: paper_area,
            paragraph_y: para_y,
            alignment,
            dpi: self.dpi,
        }
        .position(common, obj_width, obj_height)
    }

    /// 본문 그림(Picture) 개체를 레이아웃하고 업데이트된 y_offset을 반환한다.
    pub(crate) fn layout_body_picture(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        col_area: &LayoutRect,
        body_area: &LayoutRect,
        paper_area: &LayoutRect,
        bin_data_content: &[BinDataContent],
        styles: &ResolvedStyleSet,
        alignment: Alignment,
        y_offset: f64,
        section_index: usize,
        para_index: usize,
        control_index: usize,
        // [Task #1079] 파일 vpos 가 이미 그림 공간을 반영(그림 para 줄 앞 gap ≥ 그림 높이)하면
        // 그림을 그 gap 안(바닥이 그림 para 줄에 정렬)에 그리고 flow 를 그림 높이만큼 추가
        // 진행하지 않는다. 이중 계상(gap + draw-advance) 방지.
        vpos_accounts_for_height: bool,
    ) -> f64 {
        // 그림 크기 (HWPUNIT → 픽셀)
        // [Issue #1230] 측면흐름 wrap 은 common(개체 틀) 프레임으로 그린다.
        let (pic_width_hu, pic_height_hu) = super::utils::picture_flow_frame_size_hu(picture);
        let pic_width = hwpunit_to_px(pic_width_hu, self.dpi);
        let pic_height = hwpunit_to_px(pic_height_hu, self.dpi);

        // 캡션 높이 및 간격 계산
        let caption_height = self.calculate_caption_height(&picture.caption, styles);
        let caption_spacing = if let Some(ref caption) = picture.caption {
            hwpunit_to_px(caption.spacing as i32, self.dpi)
        } else {
            0.0
        };

        // 캡션을 포함한 전체 크기 계산 (위치 결정에 사용)
        let (total_width, total_height) = if let Some(ref caption) = picture.caption {
            match caption.direction {
                CaptionDirection::Top | CaptionDirection::Bottom => {
                    (pic_width, pic_height + caption_height + caption_spacing)
                }
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    (pic_width + cw + caption_spacing, pic_height)
                }
            }
        } else {
            (pic_width, pic_height)
        };

        // [#6596] 바깥 여백은 개체 상자의 일부다. 한/글은 여백을 포함한 상자를 오프셋·정렬
        // 자리에 놓고 잉크(그림+캡션)를 그 안쪽 (왼쪽 여백, 위 여백) 에 그린다.
        // 코퍼스 실측(samples↔pdf 한컴 PDF 215문서): 여백 3.01mm 그림 45건 중 44건이
        // dx + 왼쪽 여백 ≈ 0, dy + 위 여백 ≈ 0 이고 Paper/Column/Para 기준과
        // Square/TopAndBottom/BehindText 를 가리지 않는다. 가운데 정렬은 좌우 여백이
        // 같아 가로가 상쇄되고 오른쪽 정렬은 오른쪽 여백만 잉크에 나타나므로, 정렬은
        // 상자 크기로 계산해야 둘 다 맞는다. 글자처럼 그림은 줄 안 상자라 이 규칙 밖이다.
        let (margin_left, margin_right, margin_top, margin_bottom) = if picture.common.treat_as_char
        {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let m = &picture.common.margin;
            (
                hwpunit_to_px(i32::from(m.left), self.dpi),
                hwpunit_to_px(i32::from(m.right), self.dpi),
                hwpunit_to_px(i32::from(m.top), self.dpi),
                hwpunit_to_px(i32::from(m.bottom), self.dpi),
            )
        };
        let box_width = total_width + margin_left + margin_right;
        let box_height = total_height + margin_top + margin_bottom;

        // 통합 좌표 계산 (여백을 포함한 상자 기준)
        let (box_x, base_y) = self.compute_object_position(
            &picture.common,
            box_width,
            box_height,
            container,
            col_area,
            body_area,
            paper_area,
            y_offset,
            alignment,
        );

        // [Issue #2032] restrictInPage(쪽 영역 안으로 제한, HWP5 attr bit 13 = HWPX pos@flowWithText):
        // vert=Para floating 그림의 하단이 쪽 영역을 벗어나면 쪽 영역 안으로 끌어올린다.
        // 미적용 시 앵커+offset 조합으로 좌표가 페이지 캔버스 밖이 되어 그림이 어느
        // 페이지에서도 보이지 않는다 (완전 소실). floating 표 동등 로직
        // (table_layout.rs compute_table_y 의 Para 클램프) 과 동일 시멘틱.
        // 상단(top bleed) 은 한컴도 허용하는 사례가 있어 하단 초과만 교정한다
        // (표의 allow_para_top_bleed 예외와 동일 취지).
        // vpos_accounts_for_height(파일 vpos 가 그림 공간을 이미 반영) 이면 그림은
        // base_y 위쪽 gap 안에 그려지므로 (frame 하단 = base_y) 클램프 비대상.
        let base_y = if !picture.common.treat_as_char
            && picture.common.flow_with_text
            && !vpos_accounts_for_height
            && matches!(picture.common.vert_rel_to, VertRelTo::Para)
        {
            let body_bottom = col_area.y + col_area.height - box_height;
            base_y.min(body_bottom.max(col_area.y))
        } else {
            base_y
        };

        // 캡션 방향에 따라 그림 위치 오프셋 계산
        let (caption_top_offset, caption_left_offset) = if let Some(ref caption) = picture.caption {
            match caption.direction {
                CaptionDirection::Top => (caption_height + caption_spacing, 0.0),
                CaptionDirection::Left => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    (0.0, cw + caption_spacing)
                }
                _ => (0.0, 0.0),
            }
        } else {
            (0.0, 0.0)
        };

        // 잉크 원점 = 상자 원점 + (왼쪽 여백, 위 여백). #3821 이 Column-왼쪽 Square 의
        // 왼쪽 여백에만 좁혀 적용하던 것을 위 실측대로 사방·전 기준으로 편다. wrap
        // exclusion 은 이미 source LINE_SEG 가 상자 기준으로 보유한다.
        let adjusted_pic_x = box_x + margin_left + caption_left_offset;
        // [Task #1079] already_accounted: 상자를 gap 안에 그림(상자 바닥이 base_y=그림 para
        // 줄에 정렬되도록 box_height 만큼 위로). flow 진행은 아래 return 에서 생략.
        let vpos_shift = if vpos_accounts_for_height {
            box_height
        } else {
            0.0
        };
        let content_top = base_y + margin_top - vpos_shift;
        let pic_y = content_top + caption_top_offset;

        // BinData에서 이미지 데이터 찾기 (bin_data_id는 1-indexed 순번)
        let bin_data_id = picture.image_attr.bin_data_id;
        let image_data = find_bin_data_bytes(bin_data_content, bin_data_id);
        // [Task #2225] 그림 미지정 — layout_picture_full 과 동일 분기.
        //
        // 여기서 곧장 되돌아가면(early return) 캡션 배치와 흐름 계산을 통째로 건너뛴다 —
        // 캡션 글자가 사라지고, 반환값도 "후속 y" 가 아니라 "높이" 가 되어 뒤 문단이 그림
        // 위로 올라탄다. 그래서 **노드만 바꿔 끼우고** 나머지 경로는 그대로 태운다.
        let picture_missing = picture_data_is_unusable(image_data.as_deref())
            && picture.image_attr.external_path.is_none();

        // 그림 자르기
        let crop = {
            let c = &picture.crop;
            if c.right > c.left && c.bottom > c.top {
                Some((c.left, c.top, c.right, c.bottom))
            } else {
                None
            }
        };

        let original_size_hu = picture.crop_reference_size();

        // 이미지 노드 생성 (그림 미지정이면 같은 자리·같은 bbox 의 placeholder 노드)
        let img_id = tree.next_id();
        let node_type = if picture_missing {
            RenderNodeType::Placeholder(
                // [Task #2230] 본문 picture — 셀 경로 없음(None).
                crate::renderer::render_tree::PlaceholderNode::missing_picture(
                    Some(section_index),
                    Some(para_index),
                    Some(control_index),
                    None,
                ),
            )
        } else {
            RenderNodeType::Image(ImageNode {
                section_index: Some(section_index),
                para_index: Some(para_index),
                control_index: Some(control_index),
                crop,
                original_size_hu,
                effect: picture.image_attr.effect,
                brightness: picture.image_attr.brightness,
                contrast: picture.image_attr.contrast,
                opacity: picture.image_attr.opacity(),
                text_wrap: Some(picture.common.text_wrap),
                transform: extract_shape_transform(&picture.shape_attr),
                external_path: picture.image_attr.external_path.clone(),
                content_inset: crate::renderer::layout::utils::picture_content_inset(picture),
                ..ImageNode::new(bin_data_id, image_data)
            })
        };
        let img_node = RenderNode::new(
            img_id,
            node_type,
            BoundingBox::new(adjusted_pic_x, pic_y, pic_width, pic_height),
        );

        parent_node.children.push(img_node);

        // 그림 테두리(선) 렌더링
        self.render_picture_border(
            tree,
            parent_node,
            picture,
            adjusted_pic_x,
            pic_y,
            pic_width,
            pic_height,
        );

        // 캡션 렌더링
        if let Some(ref caption) = picture.caption {
            use crate::model::shape::CaptionVertAlign;
            let (cap_x, cap_w, cap_y) = match caption.direction {
                CaptionDirection::Top => (adjusted_pic_x, pic_width, base_y + margin_top),
                CaptionDirection::Bottom => (
                    adjusted_pic_x,
                    pic_width,
                    pic_y + pic_height + caption_spacing,
                ),
                CaptionDirection::Left | CaptionDirection::Right => {
                    let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                    let cx = if caption.direction == CaptionDirection::Left {
                        box_x + margin_left
                    } else {
                        adjusted_pic_x + pic_width + caption_spacing
                    };
                    let cy = match caption.vert_align {
                        CaptionVertAlign::Top => pic_y,
                        CaptionVertAlign::Center => {
                            pic_y + (pic_height - caption_height).max(0.0) / 2.0
                        }
                        CaptionVertAlign::Bottom => pic_y + (pic_height - caption_height).max(0.0),
                    };
                    (cx, cw, cy)
                }
            };

            let cell_ctx = super::CellContext {
                in_textbox: false,
                parent_para_index: para_index,
                path: vec![super::CellPathEntry {
                    control_index,
                    cell_index: 0,
                    cell_para_index: 0,
                    text_direction: 0,
                }],
            };
            self.layout_caption(
                tree,
                parent_node,
                caption,
                styles,
                col_area,
                cap_x,
                cap_w,
                cap_y,
                &mut self.auto_counter.borrow_mut(),
                bin_data_content,
                Some(cell_ctx),
                CaptionOwner::new(
                    Some(section_index),
                    Some(para_index),
                    Some(control_index),
                    CaptionControlKind::Image,
                ),
            );
        }

        // y_offset 업데이트: Para 기준 그림만 높이만큼 진행
        // Page/Paper 기준 그림은 플로팅이므로 y_offset 변경 없음
        // Task #347: 글뒤로/글앞으로 그림은 본문 흐름을 점유하지 않으므로 y 미진행.
        // base_y는 vert_offset이 적용된 상자 상단 y이므로, base_y + box_height 가
        // 상자 하단(잉크 + 위·아래 여백) y가 된다. y_offset(앵커 단락 y) 대신 base_y를
        // 기준으로 반환해야 vert_offset이 있는 혼합 단락(텍스트+그림)에서 후속 단락이
        // 그림 위로 겹치지 않는다.
        match (picture.common.vert_rel_to, picture.common.text_wrap) {
            (VertRelTo::Para, TextWrap::BehindText | TextWrap::InFrontOfText) => y_offset,
            // [Task #1079] 파일 vpos 가 그림 공간을 이미 반영하면 그림은 gap 안에 그려졌고
            // 후속 문단은 파일 vpos(그림 para 줄)로 흐르므로 추가 진행 없이 base_y 반환.
            (VertRelTo::Para, _) if vpos_accounts_for_height => base_y,
            (VertRelTo::Para, _) => base_y + box_height,
            (VertRelTo::Page | VertRelTo::Paper, _) => y_offset,
        }
    }

    /// 캡션의 총 높이를 계산한다.
    ///
    /// 산식은 `composer::caption_height_px`가 단일 정의다(#4320) — height_measurer 의
    /// `measure_caption`도 같은 함수를 호출한다.
    pub(crate) fn calculate_caption_height(
        &self,
        caption: &Option<Caption>,
        _styles: &ResolvedStyleSet,
    ) -> f64 {
        super::super::composer::caption_height_px(caption, self.dpi)
    }

    /// 캡션을 레이아웃한다.
    pub(crate) fn layout_caption(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        caption: &Caption,
        styles: &ResolvedStyleSet,
        _col_area: &LayoutRect,
        content_x: f64,
        content_width: f64,
        y_start: f64,
        auto_counter: &mut AutoNumberCounter,
        bin_data_content: &[BinDataContent],
        cell_ctx: Option<super::CellContext>,
        caption_owner: Option<CaptionOwner>,
    ) {
        if caption.paragraphs.is_empty() {
            return;
        }

        let caption_area = LayoutRect {
            x: content_x,
            y: y_start,
            width: content_width,
            height: 0.0, // 높이는 동적
        };

        let mut para_y = y_start;
        for (pi, para) in caption.paragraphs.iter().enumerate() {
            let first_caption_node = parent_node.children.len();
            let para_y_before_layout = para_y;
            // 먼저 문단을 조합
            let mut composed =
                crate::renderer::composer::compose_paragraph_in_context(para, styles);

            // AutoNumber 컨트롤 처리: 조합된 텍스트에 번호 삽입
            self.apply_auto_numbers_to_composed(&mut composed, para, auto_counter);

            // cell_ctx에 cell_para_index 갱신
            let ctx = cell_ctx.as_ref().map(|c| {
                let mut cc = c.clone();
                if let Some(last) = cc.path.last_mut() {
                    last.cell_para_index = pi;
                }
                cc
            });

            para_y = self.layout_composed_paragraph(
                tree,
                parent_node,
                &composed,
                styles,
                &caption_area,
                para_y,
                0,
                composed.lines.len(),
                0,
                0,
                ctx.clone(),
                false,
                false,
                0.0,
                None,
                Some(para),
                Some(bin_data_content),
                None, // 캡션 컨텍스트 — wrap zone 무관
            );

            // Tag only lines emitted by this caption paragraph, not text inside
            // its nested controls or earlier siblings. Wrapped lines share pi.
            for node in &mut parent_node.children[first_caption_node..] {
                if let RenderNodeType::TextLine(line) = &mut node.node_type {
                    line.caption_owner = caption_owner.map(|owner| CaptionOwner {
                        caption_ordinal: pi,
                        ..owner
                    });
                }
            }

            self.layout_caption_topbottom_pictures(
                tree,
                parent_node,
                para,
                &caption_area,
                para_y_before_layout,
                bin_data_content,
                ctx.as_ref(),
                styles,
            );
        }
    }

    fn layout_caption_topbottom_pictures(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        para: &Paragraph,
        caption_area: &LayoutRect,
        para_y: f64,
        bin_data_content: &[BinDataContent],
        cell_ctx: Option<&super::CellContext>,
        // [#6284] 캡션 문단 조판에 필요하다.
        styles: &ResolvedStyleSet,
    ) {
        let anchor_y = para
            .line_segs
            .first()
            .filter(|seg| seg.vertical_pos >= 0)
            .map(|seg| caption_area.y + hwpunit_to_px(seg.vertical_pos, self.dpi))
            .unwrap_or(para_y);

        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            let Control::Picture(pic) = ctrl else {
                continue;
            };
            if !matches!(pic.common.text_wrap, TextWrap::TopAndBottom) {
                continue;
            }
            if tree
                .get_inline_shape_position(0, 0, ctrl_idx, cell_ctx)
                .is_some()
            {
                continue;
            }

            let (pic_width_hu, pic_height_hu) = picture_display_size_hu(pic);
            let pic_width = hwpunit_to_px(pic_width_hu, self.dpi);
            let pic_height = hwpunit_to_px(pic_height_hu, self.dpi);
            let placement_area = LayoutRect {
                x: caption_area.x,
                y: anchor_y,
                width: caption_area.width,
                height: pic_height.max(caption_area.height),
            };
            let mut placement_common = pic.common.clone();
            placement_common.treat_as_char = false;
            let (pic_x, pic_y) = self.compute_object_position(
                &placement_common,
                pic_width,
                pic_height,
                &placement_area,
                &placement_area,
                &placement_area,
                &placement_area,
                anchor_y,
                Alignment::Left,
            );
            let pic_area = LayoutRect {
                x: pic_x,
                y: pic_y,
                width: pic_width,
                height: pic_height,
            };
            let mut pic_for_layout = (**pic).clone();
            pic_for_layout.common.treat_as_char = false;
            pic_for_layout.common.horizontal_offset = 0;
            pic_for_layout.common.vertical_offset = 0;
            pic_for_layout.common.horz_align = HorzAlign::Left;
            pic_for_layout.common.vert_align = VertAlign::Top;

            self.layout_picture(
                tree,
                parent_node,
                &pic_for_layout,
                &pic_area,
                bin_data_content,
                Alignment::Left,
                Some(0),
                Some(0),
                Some(ctrl_idx),
                cell_ctx,
                styles,
            );

            if pic.common.treat_as_char {
                tree.set_inline_shape_position(0, 0, ctrl_idx, cell_ctx, pic_x, pic_y);
            }
        }
    }

    /// [#5708] 저장 LINE_SEG 가 없는 각주 문단의 합성 폴백 줄높이(400 HWPUNIT = 5.33px)를
    /// 문단 줄간격 설정으로 보정한다.
    ///
    /// 폴백값을 그대로 쓰면 줄 전진(5.3px)이 글자 크기(9pt = 12px)보다 작아 각주 줄이 서로
    /// 겹쳐 그려진다(00464 1쪽 하단 주석 8줄). 본문·표 경로는 #674 로 같은 보정을 이미
    /// 하고 있고, 각주 경로만 빠져 있었다. `corrected_line_metrics` 는 `raw_lh < max_fs`
    /// 일 때만 개입하므로 저장 LINE_SEG 를 가진 각주는 종전 그대로다(#2112 계약).
    fn footnote_line_metrics(
        &self,
        comp_line: &ComposedLine,
        para_style_id: u16,
        styles: &ResolvedStyleSet,
    ) -> (f64, f64) {
        let raw_lh = hwpunit_to_px(comp_line.line_height, self.dpi);
        let raw_ls = hwpunit_to_px(comp_line.line_spacing, self.dpi);
        let max_fs = comp_line
            .runs
            .iter()
            .map(|run| {
                let ts = run.text_style(styles);
                if ts.font_size > 0.0 {
                    ts.font_size
                } else {
                    12.0
                }
            })
            .fold(0.0f64, f64::max);
        let para_style = styles.para_styles.get(para_style_id as usize);
        let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
        let ls_type = para_style
            .map(|s| s.line_spacing_type)
            .unwrap_or(LineSpacingType::Percent);
        crate::renderer::corrected_line_metrics(raw_lh, raw_ls, max_fs, ls_type, ls_val)
    }

    /// 선택된 각주 문단의 실제 경로와 앞뒤 간격을 함께 확정한다.
    /// 번호 전용 첫 문단과 저장 빈 줄은 기존 간격 계약을 유지한다.
    fn footnote_paragraph_placement(
        &self,
        para: &Paragraph,
        composed: &ComposedParagraph,
        styles: &ResolvedStyleSet,
        fragment: Option<FootnoteFragment>,
        number_drawn: bool,
        selected: std::ops::Range<usize>,
        relative_y: f64,
    ) -> FootnoteParagraphPlacement {
        let route = if para.text.is_empty()
            && para.controls.is_empty()
            && !fragment_draws_number(fragment)
            && composed.lines.len() == 1
        {
            FootnoteParagraphRoute::StoredEmpty
        } else if fragment_draws_number(fragment) && !number_drawn {
            FootnoteParagraphRoute::Numbered
        } else {
            FootnoteParagraphRoute::Plain
        };
        let mut spacing = ParagraphVerticalSpacing {
            before: 0.0,
            after: 0.0,
        };
        if route == FootnoteParagraphRoute::Plain {
            let style = styles.para_styles.get(composed.para_style_id as usize);
            if selected.start == 0 && relative_y.abs() >= 1.0 {
                spacing.before = crate::renderer::hwp3_variant_flow_spacing_before(
                    style.map_or(0.0, |style| style.spacing_before),
                    self.use_hwp3_origin_flow_spacing_before.get(),
                )
                .max(0.0);
            }
            if selected.end == composed.lines.len().max(1) {
                spacing.after = style.map_or(0.0, |style| style.spacing_after).max(0.0);
            }
        }
        FootnoteParagraphPlacement { route, spacing }
    }

    pub(crate) fn estimate_footnote_area_height(
        &self,
        footnotes: &[FootnoteRef],
        paragraphs: &[Paragraph],
        shape: &FootnoteShape,
        styles: &ResolvedStyleSet,
        area_width: f64,
    ) -> f64 {
        self.estimate_footnote_area_height_with_metrics(
            footnotes,
            paragraphs,
            styles,
            area_width,
            hwpunit_to_px(shape.separator_above_margin_hu() as i32, self.dpi)
                + border_width_to_px(shape.separator_line_width).max(0.5)
                + hwpunit_to_px(shape.separator_below_margin_hu() as i32, self.dpi),
            hwpunit_to_px(shape.between_notes_margin_hu() as i32, self.dpi),
        )
    }

    /// 페이지 분할 큐와 최종 배치는 같은 선택 줄 메트릭을 소비한다.
    pub(crate) fn estimate_footnote_area_height_with_metrics(
        &self,
        footnotes: &[FootnoteRef],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        area_width: f64,
        separator_height: f64,
        between_notes: f64,
    ) -> f64 {
        if footnotes.is_empty() {
            return 0.0;
        }
        let mut total = if footnotes
            .iter()
            .any(|note| fragment_draws_separator(note.fragment))
        {
            separator_height
        } else {
            0.0
        };

        // 실제 `layout_footnote_area`와 같은 줄 높이 산식을 사용한다. 저장 LineSeg의
        // line_height만 더하면 renderer가 누적하는 trailing line_spacing이 빠져 긴
        // 각주가 예약 영역(및 footer)을 넘을 수 있다. 마지막 각주 문단의 마지막 줄은
        // layout 경로에서도 trailing spacing을 붙이지 않는다.
        for (i, fn_ref) in footnotes.iter().enumerate() {
            let fn_paras = get_footnote_paragraphs(fn_ref, paragraphs);
            let (start_line, end_line) = fragment_line_bounds(
                fn_ref.fragment,
                footnote_composed_line_count(fn_paras, area_width, styles, self.dpi),
            );
            let mut flat_line = 0usize;
            let mut number_drawn = false;
            for para in fn_paras {
                let composed = compose_footnote_paragraph(para, area_width, styles, self.dpi);
                let line_count = composed.lines.len().max(1);
                let para_start = flat_line;
                let selected_start = start_line.saturating_sub(para_start).min(line_count);
                let selected_end = end_line.saturating_sub(para_start).min(line_count);
                flat_line += line_count;
                if selected_start >= selected_end {
                    continue;
                }
                let placement = self.footnote_paragraph_placement(
                    para,
                    &composed,
                    styles,
                    fn_ref.fragment,
                    number_drawn,
                    selected_start..selected_end,
                    total,
                );
                total += placement.spacing.before;
                if placement.route == FootnoteParagraphRoute::StoredEmpty {
                    // 실제 빈 줄 경로는 저장 높이만 소비하고 후행 줄간격을 붙이지 않는다.
                    total += hwpunit_to_px(composed.lines[0].line_height, self.dpi);
                } else if composed.lines.is_empty() {
                    total += hwpunit_to_px(400, self.dpi);
                } else {
                    for (index, line) in composed.lines[selected_start..selected_end]
                        .iter()
                        .enumerate()
                    {
                        let (line_height, line_spacing_px) =
                            self.footnote_line_metrics(line, composed.para_style_id, styles);
                        total += line_height;
                        if para_start + selected_start + index + 1 < end_line {
                            total += line_spacing_px;
                        }
                    }
                }
                total += placement.spacing.after;
                number_drawn |= placement.route == FootnoteParagraphRoute::Numbered;
            }
            // 각주 간 간격
            if i + 1 < footnotes.len() {
                total += between_notes;
            }
        }
        total
    }

    /// 각주 영역 레이아웃 (구분선 + 각주 문단들)
    pub(crate) fn layout_footnote_area(
        &self,
        tree: &mut PageLayoutContext,
        fn_node: &mut RenderNode,
        footnotes: &[FootnoteRef],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        fn_area: &LayoutRect,
        shape: &FootnoteShape,
    ) {
        let mut y = fn_area.y;

        if footnotes
            .iter()
            .any(|footnote| fragment_draws_separator(footnote.fragment))
        {
            // (1) 구분선 위 여백
            y += hwpunit_to_px(shape.separator_above_margin_hu() as i32, self.dpi);

            // (2) 구분선
            let sep_length =
                footnote_separator_length_px(shape.separator_length, fn_area.width, self.dpi);
            let line_width = border_width_to_px(shape.separator_line_width).max(0.5);

            let sep_id = tree.next_id();
            let sep_line = LineNode::new(
                fn_area.x,
                y,
                fn_area.x + sep_length,
                y,
                LineStyle {
                    color: shape.separator_color,
                    width: line_width,
                    dash: StrokeDash::Solid,
                    ..Default::default()
                },
            );
            let sep_bbox = sep_line.ink_bbox();
            let sep_node = RenderNode::new(sep_id, RenderNodeType::Line(sep_line), sep_bbox);
            fn_node.children.push(sep_node);
            y += line_width;

            // (3) 구분선 아래 여백
            y += hwpunit_to_px(shape.separator_below_margin_hu() as i32, self.dpi);
        }

        // (4) 각 각주 렌더링
        // 각주 TextRun에 마커를 인코딩하여 히트테스트에서 식별 가능하도록 함
        // section_index = footnote_index (footnotes 배열 인덱스)
        // para_index = usize::MAX - 2000 - fn_para_idx (각주 내 문단 인덱스)
        for (i, fn_ref) in footnotes.iter().enumerate() {
            let fn_paras = get_footnote_paragraphs(fn_ref, paragraphs);
            let number_text = format_footnote_number(
                fn_ref.number,
                &shape.number_format,
                shape.prefix_char,
                shape.suffix_char,
            );
            let (fragment_start, fragment_end) = fragment_line_bounds(
                fn_ref.fragment,
                footnote_composed_line_count(fn_paras, fn_area.width, styles, self.dpi),
            );
            let mut flat_line = 0usize;
            let mut number_drawn = false;

            for (p_idx, para) in fn_paras.iter().enumerate() {
                let composed = compose_footnote_paragraph(para, fn_area.width, styles, self.dpi);
                let marker_section = i; // footnote_index
                let marker_para = usize::MAX - 2000 - p_idx; // 각주 내 문단 인덱스
                                                             // 각주 번호 스타일용 기본 char_shape_id (빈/비빈 문단 모두 동일)
                let base_cs_id = para
                    .char_shapes
                    .first()
                    .map(|cs| cs.char_shape_id as u32)
                    .unwrap_or(composed.para_style_id as u32);

                let line_count = composed.lines.len().max(1);
                let para_start = flat_line;
                let para_end = flat_line + line_count;
                let selected_start = fragment_start.saturating_sub(para_start).min(line_count);
                let selected_end = fragment_end.saturating_sub(para_start).min(line_count);
                flat_line = para_end;
                if selected_start >= selected_end {
                    continue;
                }
                let is_last_selected_line = para_start + selected_end == fragment_end;
                let placement = self.footnote_paragraph_placement(
                    para,
                    &composed,
                    styles,
                    fn_ref.fragment,
                    number_drawn,
                    selected_start..selected_end,
                    y - fn_area.y,
                );
                if placement.route == FootnoteParagraphRoute::StoredEmpty {
                    y += hwpunit_to_px(composed.lines[0].line_height, self.dpi);
                    continue;
                }

                // 첫 fragment의 첫 선택 줄에만 각주 번호를 싣는다. 번호는 문단의 autoNum
                // 자리표시 위에 얹어 일반 문단 경로로 그린다 — 그래야 번호 문단도 다른 문단과
                // 같은 내어쓰기·정렬을 받는다. 조합 줄이 없는 빈 문단만 종전 전용 경로를 쓴다.
                let draw_number = placement.route == FootnoteParagraphRoute::Numbered;
                number_drawn |= draw_number;
                let stored_prefix = (self.profile.get().hwpx_stored_layout()
                    || self.profile.get().hwp5_stored_pagination_layout())
                .then(|| stored_footnote_number_prefix(para, fn_ref.number))
                .flatten();
                let paragraph_number = stored_prefix
                    .as_ref()
                    .map_or(number_text.as_str(), |(text, _)| text.as_str());
                // 여러 자동 번호 슬롯은 기존 번호 경로가 원본 형식을 보존한다.
                let numbered = (draw_number
                    && stored_prefix.as_ref().is_none_or(|(_, slots)| *slots <= 1))
                .then(|| {
                    footnote_composed_with_number(
                        para,
                        &composed,
                        paragraph_number,
                        selected_start,
                        base_cs_id,
                    )
                })
                .flatten();
                if draw_number && numbered.is_none() {
                    y = self.layout_footnote_paragraph_with_number(
                        tree,
                        fn_node,
                        &composed,
                        para,
                        styles,
                        fn_area,
                        y,
                        &number_text,
                        fn_ref.number,
                        marker_section,
                        marker_para,
                        base_cs_id,
                        selected_start,
                        selected_end,
                        is_last_selected_line,
                    );
                } else {
                    let composed = numbered.as_ref().unwrap_or(&composed);
                    let returned_y = self.layout_composed_paragraph_in_frame(
                        tree,
                        fn_node,
                        composed,
                        styles,
                        fn_area,
                        y,
                        selected_start,
                        selected_end,
                        marker_section,
                        marker_para,
                        None,
                        false,
                        false,
                        0.0,
                        None,
                        None,
                        None,
                        None, // 각주 컨텍스트 — wrap zone 무관
                        false,
                        Some(placement.spacing),
                    );
                    if is_last_selected_line {
                        // fragment의 마지막 줄은 trailing line-spacing을 쓰지 않는다.
                        let trail_ls = composed
                            .lines
                            .get(selected_end.saturating_sub(1))
                            .map(|l| hwpunit_to_px(l.line_spacing, self.dpi))
                            .unwrap_or(0.0);
                        y = returned_y - trail_ls;
                    } else {
                        y = returned_y;
                    }
                }
            }

            // 각주 간 간격
            if i + 1 < footnotes.len() {
                y += hwpunit_to_px(shape.between_notes_margin_hu() as i32, self.dpi);
            }
        }
    }

    /// 각주 번호를 앞에 붙여 문단을 레이아웃
    /// marker_section: footnote_index, marker_para: 각주 내 문단 마커 (usize::MAX - 2000 - fn_para_idx)
    /// base_cs_id: 번호 스타일 결정용 기본 char_shape_id (문단의 char_shapes[0])
    pub(crate) fn layout_footnote_paragraph_with_number(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        composed: &ComposedParagraph,
        paragraph: &Paragraph,
        styles: &ResolvedStyleSet,
        area: &LayoutRect,
        y_start: f64,
        number_text: &str,
        note_number: u16,
        marker_section: usize,
        marker_para: usize,
        base_cs_id: u32,
        line_start: usize,
        line_end: usize,
        // fragment의 마지막 선택 줄은 trailing line_spacing을 누적하지 않는다.
        is_last_selected_line: bool,
    ) -> f64 {
        let mut y = y_start;
        let stored_prefix = (self.profile.get().hwpx_stored_layout()
            || self.profile.get().hwp5_stored_pagination_layout())
        .then(|| stored_footnote_number_prefix(paragraph, note_number))
        .flatten();
        let number_text = stored_prefix
            .as_ref()
            .map_or(number_text, |(text, _)| text.as_str());
        // 분할 큐는 빈 각주 문단도 한 줄의 virtual fragment로 센다. 실제 composed
        // 줄은 0개이므로 그 범위를 그대로 slice하면 panic 난다. 기존 비분할 경로처럼
        // 빈 문단 fallback까지 흘려보내 번호/높이 계약을 보존한다.
        let line_start = line_start.min(composed.lines.len());
        let line_end = line_end.min(composed.lines.len()).max(line_start);

        let para_style = styles.para_styles.get(composed.para_style_id as usize);
        let margin_left = para_style.map(|style| style.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|style| style.margin_right).unwrap_or(0.0);
        let indent = para_style.map(|style| style.indent).unwrap_or(0.0);

        for (offset, comp_line) in composed.lines[line_start..line_end].iter().enumerate() {
            // 이어받은 쪽에서도 원본 문단 안의 줄 번호로 들여쓰기를 결정한다.
            let line_indent = crate::renderer::equation_tac_flow::paragraph_line_indent_for_source(
                indent,
                line_start + offset,
                Some(paragraph),
                composed.lines.len(),
                true,
            );
            let line_x = area.x + margin_left + line_indent;
            let line_width = (area.width - margin_left - margin_right - line_indent).max(0.0);
            // LineSeg.line_height는 HWP에서 줄간격이 이미 반영된 값.
            // [#5708] 저장 LINE_SEG 가 없는 문단의 폴백(400 HWPUNIT = 5.33px)은 글자보다
            // 작아 줄이 겹치므로 문단 줄간격 설정으로 보정한다.
            let (line_height, corrected_line_spacing) =
                self.footnote_line_metrics(comp_line, composed.para_style_id, styles);
            let raw_baseline = hwpunit_to_px(comp_line.baseline_distance, self.dpi);
            // 베이스라인도 같은 비율로 따라간다 — 줄 상자만 키우면 글자가 상자 위로 뜬다.
            let baseline = if line_height > 0.0
                && raw_baseline > 0.0
                && hwpunit_to_px(comp_line.line_height, self.dpi) > 0.0
            {
                (raw_baseline * line_height / hwpunit_to_px(comp_line.line_height, self.dpi))
                    .min(line_height)
            } else {
                raw_baseline
            };

            let line_id = tree.next_id();
            let mut line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(line_height, baseline)),
                BoundingBox::new(line_x, y, line_width, line_height),
            );

            let mut x = line_x;

            // 첫 줄에 각주 번호 삽입
            if offset == 0 {
                // 각주 번호 스타일: 문단의 기본 char_shape로 고정 (크기 약간 축소)
                // 빈/비빈 문단 모두 동일한 base_cs_id 사용 → 리렌더링 시 폰트·폭 변동 방지
                let base_style = {
                    let mut ts = resolved_to_text_style(styles, base_cs_id, 0);
                    if stored_prefix.is_none() {
                        ts.font_size = (ts.font_size * 0.9).max(8.0);
                    }
                    ts
                };

                let num_width = estimate_text_width(number_text, &base_style);
                let num_id = tree.next_id();
                let num_node = RenderNode::new(
                    num_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: number_text.to_string(),
                        style: base_style,
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: Some(marker_section),
                        para_index: Some(marker_para),
                        char_start: None, // 번호 run은 char_start 없음
                        cell_context: None,
                        is_para_end: false,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                        layout_positions: None,
                        display_text: None,
                    }),
                    BoundingBox::new(x, y, num_width, line_height),
                );
                line_node.children.push(num_node);
                x += num_width;
            }

            // 원본 TextRun들
            let mut char_offset = comp_line.char_start;
            let mut tab_index = composed
                .lines
                .iter()
                .take(line_start + offset)
                .flat_map(|line| line.runs.iter())
                .flat_map(|run| run.text.chars())
                .filter(|&ch| ch == '\t')
                .count();
            for run in &comp_line.runs {
                let mut text_style = run.text_style(styles);
                let skipped = stored_prefix.as_ref().map_or(0, |(_, count)| {
                    count
                        .saturating_sub(char_offset)
                        .min(run.text.chars().count())
                });
                let text = run.text.chars().skip(skipped).collect::<String>();
                if stored_prefix.is_some() {
                    text_style.inline_tabs = composed
                        .tab_extended
                        .iter()
                        .skip(tab_index)
                        .copied()
                        .collect();
                }
                let width = estimate_text_width(&text, &text_style);
                tab_index += run.text.chars().filter(|&ch| ch == '\t').count();

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text,
                        style: text_style,
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: Some(marker_section),
                        para_index: Some(marker_para),
                        char_start: Some(char_offset),
                        cell_context: None,
                        is_para_end: false,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: run.char_overlap.clone(),
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                        layout_positions: None,
                        display_text: None,
                    }),
                    BoundingBox::new(x, y, width, line_height),
                );
                line_node.children.push(run_node);
                x += width;
                char_offset += run.text.chars().count();
            }

            parent.children.push(line_node);
            // [Issue #483] trailing line_spacing 추가 — layout_composed_paragraph:2560 과 정합.
            // 단, 각주의 마지막 paragraph 의 마지막 line 에서는 trailing line_spacing 을
            // 누적하지 않는다 — 다음 각주와의 간격은 between-notes 값이 책임하므로
            // 이중 합산을 피하기 위함.
            let is_last_line = offset + 1 >= line_end - line_start;
            if is_last_selected_line && is_last_line {
                y += line_height;
            } else {
                let line_spacing_px = corrected_line_spacing;
                y += line_height + line_spacing_px;
            }
        }

        // 빈 문단 fallback
        if composed.lines.is_empty() {
            let default_height = hwpunit_to_px(400, self.dpi);
            let line_id = tree.next_id();
            let line_node = RenderNode::new(
                line_id,
                RenderNodeType::TextLine(TextLineNode::new(default_height, default_height * 0.8)),
                BoundingBox::new(area.x, y, area.width, default_height),
            );
            parent.children.push(line_node);
            y += default_height;
        }

        y
    }

    /// 문단 내 각주/미주 컨트롤에 대해 윗첨자 참조 번호를 렌더링한다.
    ///
    /// 마지막 TextLine의 마지막 TextRun 우측에 윗첨자 번호를 추가한다.
    pub(crate) fn add_footnote_superscripts(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        para: &Paragraph,
        _styles: &ResolvedStyleSet,
    ) {
        // layout_composed_paragraph에서 이미 인라인 FootnoteMarker를 삽입한 경우 건너뜀
        let has_inline_markers = parent.children.iter().any(|line| {
            line.children
                .iter()
                .any(|n| matches!(n.node_type, RenderNodeType::FootnoteMarker(_)))
        });
        if has_inline_markers {
            return;
        }

        // 각주/미주의 (번호, 텍스트 위치) 수집 — ComposedParagraph에서 미리 계산된 위치 사용
        // 폴백: control_text_positions로 직접 계산
        let ctrl_positions = para.control_text_positions();
        let mut footnotes: Vec<(String, usize)> = Vec::new();
        for (ci, ctrl) in para.controls.iter().enumerate() {
            let marker_text = match ctrl {
                Control::Footnote(fn_ctrl) => Some(format_control_note_marker(
                    fn_ctrl.number,
                    fn_ctrl.number_shape,
                    fn_ctrl.before_decoration_letter,
                    fn_ctrl.after_decoration_letter,
                )),
                Control::Endnote(en_ctrl) => Some(format_control_note_marker(
                    en_ctrl.number,
                    en_ctrl.number_shape,
                    en_ctrl.before_decoration_letter,
                    en_ctrl.after_decoration_letter,
                )),
                _ => None,
            };
            if let Some(text) = marker_text {
                let pos = ctrl_positions.get(ci).copied().unwrap_or(usize::MAX);
                footnotes.push((text, pos));
            }
        }

        if footnotes.is_empty() {
            return;
        }

        // 각 각주 위첨자를 렌더링: char_start 기반으로 정확한 TextRun 위치에 삽입
        for (_fn_idx, (number_text, char_pos)) in footnotes.iter().enumerate() {
            let mut target_line_idx: Option<usize> = None;
            let mut insert_x = 0.0;
            let mut line_height = 18.0;
            let mut line_y = 0.0;
            let mut base_font_size = 12.0_f64;
            let mut base_font_family = "sans-serif".to_string();

            // TextRun의 char_start로 각주 위치 찾기
            if *char_pos < usize::MAX {
                'outer: for (li, line_node) in parent.children.iter().enumerate() {
                    if !matches!(line_node.node_type, RenderNodeType::TextLine(_)) {
                        continue;
                    }
                    // 이 줄의 char_start 범위 확인: 첫 run의 char_start ~ 마지막 run의 (char_start + len)
                    let mut line_min_cs = usize::MAX;
                    let mut line_max_end = 0usize;
                    for run_node in &line_node.children {
                        if let RenderNodeType::TextRun(ref run) = run_node.node_type {
                            if let Some(cs) = run.char_start {
                                line_min_cs = line_min_cs.min(cs);
                                line_max_end = line_max_end.max(cs + run.text.chars().count());
                            }
                        }
                    }
                    // 각주 위치가 이 줄에 포함되지 않으면 다음 줄
                    if *char_pos > line_max_end || line_min_cs == usize::MAX {
                        continue;
                    }

                    for run_node in &line_node.children {
                        if let RenderNodeType::TextRun(ref run) = run_node.node_type {
                            if let Some(cs) = run.char_start {
                                let run_len = run.text.chars().count();
                                let run_end = cs + run_len;
                                if *char_pos >= cs && *char_pos <= run_end {
                                    let chars_before = char_pos - cs;
                                    let partial_text: String =
                                        run.text.chars().take(chars_before).collect();
                                    let partial_width =
                                        estimate_text_width(&partial_text, &run.style);
                                    insert_x = run_node.bbox.x + partial_width;
                                    line_height = line_node.bbox.height;
                                    line_y = line_node.bbox.y;
                                    base_font_size = run.style.font_size;
                                    base_font_family = run.style.font_family.clone();
                                    target_line_idx = Some(li);
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }

            // 최종 폴백: 마지막 TextLine 끝
            if target_line_idx.is_none() {
                if let Some(li) = parent
                    .children
                    .iter()
                    .rposition(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
                {
                    let line = &parent.children[li];
                    insert_x = line
                        .children
                        .last()
                        .map(|c| c.bbox.x + c.bbox.width)
                        .unwrap_or(line.bbox.x);
                    line_height = line.bbox.height;
                    line_y = line.bbox.y;
                    if let Some(last_run) = line.children.last() {
                        if let RenderNodeType::TextRun(ref run) = last_run.node_type {
                            base_font_size = run.style.font_size;
                            base_font_family = run.style.font_family.clone();
                        }
                    }
                    target_line_idx = Some(li);
                }
            }

            if let Some(line_idx) = target_line_idx {
                let sup_font_size = (base_font_size * 0.6).max(7.0);
                let sup_y_offset = line_height * 0.35;
                let style = TextStyle {
                    font_size: sup_font_size,
                    font_family: base_font_family,
                    ..Default::default()
                };
                let width = estimate_text_width(&number_text, &style);

                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: number_text.clone(),
                        style,
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: None,
                        char_start: None,
                        cell_context: None,
                        is_para_end: false,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline: line_height,
                        field_marker: FieldMarkerType::None,
                        layout_positions: None,
                        display_text: None,
                    }),
                    BoundingBox::new(insert_x, line_y - sup_y_offset, width, line_height),
                );

                let line_mut = &mut parent.children[line_idx];
                line_mut.children.push(run_node);
            }
        }
    }
}

fn get_footnote_paragraphs<'a>(
    fn_ref: &FootnoteRef,
    paragraphs: &'a [Paragraph],
) -> &'a [Paragraph] {
    match &fn_ref.source {
        FootnoteSource::Body {
            para_index,
            control_index,
        } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Footnote(footnote)) = para.controls.get(*control_index) {
                    return &footnote.paragraphs;
                }
            }
            &[]
        }
        FootnoteSource::TableCell {
            para_index,
            table_control_index,
            cell_index,
            cell_para_index,
            cell_control_index,
        } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Table(table)) = para.controls.get(*table_control_index) {
                    if let Some(cell) = table.cells.get(*cell_index) {
                        if let Some(cp) = cell.paragraphs.get(*cell_para_index) {
                            if let Some(Control::Footnote(footnote)) =
                                cp.controls.get(*cell_control_index)
                            {
                                return &footnote.paragraphs;
                            }
                        }
                    }
                }
            }
            &[]
        }
        FootnoteSource::ShapeTextBox {
            para_index,
            shape_control_index,
            tb_para_index,
            tb_control_index,
        } => {
            if let Some(para) = paragraphs.get(*para_index) {
                if let Some(Control::Shape(shape_obj)) = para.controls.get(*shape_control_index) {
                    if let Some(text_box) = shape_obj.drawing().and_then(|d| d.text_box.as_ref()) {
                        if let Some(tp) = text_box.paragraphs.get(*tb_para_index) {
                            if let Some(Control::Footnote(footnote)) =
                                tp.controls.get(*tb_control_index)
                            {
                                return &footnote.paragraphs;
                            }
                        }
                    }
                }
            }
            &[]
        }
    }
}

fn note_number_format_from_hwp_code(code: u8) -> NumFmt {
    match code {
        0 => NumFmt::Digit,
        1 => NumFmt::CircledDigit,
        2 => NumFmt::RomanUpper,
        3 => NumFmt::RomanLower,
        4 => NumFmt::LatinUpper,
        5 => NumFmt::LatinLower,
        8 => NumFmt::HangulGaNaDa,
        12 => NumFmt::HangulNumber,
        13 => NumFmt::HanjaNumber,
        _ => NumFmt::Digit,
    }
}

fn note_decoration_char(value: u16) -> Option<char> {
    if value == 0 {
        None
    } else {
        char::from_u32(value as u32).filter(|ch| *ch != '\0')
    }
}

fn format_control_note_marker(
    number: u16,
    number_shape: u32,
    before_decoration_letter: u16,
    after_decoration_letter: u16,
) -> String {
    let number = format_number(number, note_number_format_from_hwp_code(number_shape as u8));
    let prefix = note_decoration_char(before_decoration_letter)
        .map(|ch| ch.to_string())
        .unwrap_or_default();
    let suffix = note_decoration_char(after_decoration_letter)
        .unwrap_or(')')
        .to_string();
    format!("{}{}{}", prefix, number, suffix)
}

/// 각주 번호 포맷 (NumberFormat에 따른 변환)
fn format_footnote_number(
    number: u16,
    format: &NumberFormat,
    prefix: char,
    suffix: char,
) -> String {
    let num_str = match format {
        NumberFormat::Digit => number.to_string(),
        NumberFormat::CircledDigit => {
            // ① ~ ⑳
            if number >= 1 && number <= 20 {
                char::from_u32(0x2460 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        NumberFormat::LowerAlpha => {
            if number >= 1 && number <= 26 {
                char::from_u32(b'a' as u32 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        NumberFormat::UpperAlpha => {
            if number >= 1 && number <= 26 {
                char::from_u32(b'A' as u32 + (number - 1) as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| number.to_string())
            } else {
                number.to_string()
            }
        }
        _ => number.to_string(), // 기타 형식은 숫자로 fallback
    };

    let prefix_str = if prefix != '\0' {
        prefix.to_string()
    } else {
        String::new()
    };
    let suffix_str = if suffix != '\0' {
        suffix.to_string()
    } else {
        ")".to_string()
    };

    format!("{}{}{} ", prefix_str, num_str, suffix_str)
}

impl LayoutEngine {
    /// 그림 테두리(선) 렌더링
    /// border_attr의 bit 0~5가 선 종류, border_width가 두께 (0이면 기본 0.1mm)
    pub(crate) fn render_picture_border(
        &self,
        tree: &mut PageLayoutContext,
        parent: &mut RenderNode,
        picture: &crate::model::image::Picture,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) {
        let line_type = picture.border_attr.attr & 0x3F;
        // 선 종류 0 = 없음
        if line_type == 0 {
            return;
        }
        let border_w = if picture.border_width > 0 {
            hwpunit_to_px(picture.border_width, self.dpi)
        } else {
            // 기본 선 두께 0.1mm
            0.1 / 25.4 * self.dpi
        };
        let stroke_dash = match line_type {
            2 => super::super::StrokeDash::Dot,        // 점선
            3 => super::super::StrokeDash::Dash,       // 긴 점선 (파선)
            4 => super::super::StrokeDash::DashDot,    // 일점쇄선
            5 => super::super::StrokeDash::DashDotDot, // 이점쇄선
            _ => super::super::StrokeDash::Solid,      // 1=실선, 기타
        };
        let style = super::super::ShapeStyle {
            fill_color: None,
            pattern: None,
            stroke_color: Some(picture.border_color),
            stroke_width: border_w,
            stroke_dash,
            opacity: 1.0,
            shadow: None,
        };
        let border_id = tree.next_id();
        let border_node = RenderNode::new(
            border_id,
            RenderNodeType::Rectangle(RectangleNode::new(0.0, style, None)),
            BoundingBox::new(x, y, w, h),
        );
        parent.children.push(border_node);
    }
}
