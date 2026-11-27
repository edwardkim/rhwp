//! 재조판하는 부동 개체의 잉크 없는 호스트 줄 점유.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{ShapeObject, VertAlign};
use crate::model::table::Table;

use super::composer::ComposedParagraph;
use super::float_placement::{is_non_flow_decoration, is_para_topbottom_float, signed_hwpunit};
use super::style_resolver::{ResolvedParaStyle, ResolvedStyleSet};

/// 저장 줄이 없는 부동 개체 앵커도 문단의 빈 글자 줄은 소유한다.
/// 파서가 만든 LineSeg는 어울림 안내 줄의 저장 증거가 아니다. 실제 저장된
/// 줄은 기존 사다리/안내 줄 경로에 남기고, 재조판 앵커의 글꼴·줄간격만 공유한다.
/// 개체의 페인트 크기는 이 글자 줄의 높이를 대신하지 않는다.
pub(crate) fn neutral_anchor_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    hwp3_legacy_caps: bool,
    dpi: f64,
) -> Option<(f64, f64, f64)> {
    // NoLS 표 host의 줄과 글꼴 선택은 새 base의 공통 결과를 그대로 소비한다.
    // 일반 부동 앵커의 첫 글자 모양으로 재계산하면 layout의 최대 글꼴 선택과 달라진다.
    if super::para_is_unstored_empty_overlay_table_host(para) {
        return super::unstored_overlay_table_host_line_metrics(para, styles, para_style);
    }
    if let Some(metrics) = hwp_topbottom_band_host_metrics(para, styles, para_style, dpi) {
        return Some(metrics);
    }
    if let Some(metrics) = word_compat_topbottom_band_host_metrics(para, styles, para_style, dpi) {
        return Some(metrics);
    }
    if !para.text.trim().is_empty()
        || para.controls.is_empty()
        || !super::empty_host_controls_are_flow_neutral(para)
        || !super::para_has_no_stored_line_segs(para)
    {
        return None;
    }
    let shape = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?;
    let char_style = styles.char_styles.get(shape as usize)?;
    // 가시 잉크가 없어도 문단 끝 문자는 마지막 줄의 글자 상자에 참여한다.
    // 재조판 프레임이 이미 계산한 종단 크기를 첫 공백 run의 크기로 낮추면
    // 측정·배치가 함께 짧아져 뒤 제목과 표가 앞당겨진다.
    let font_size = char_style
        .font_size
        .max(super::composer::paragraph_end_font_size_px(para, styles));
    if font_size <= 0.0 {
        return None;
    }
    if hwp3_legacy_caps
        && (font_size > super::hwpunit_to_px(1000, dpi) + 0.1
            || (!char_style.bold && font_size < super::hwpunit_to_px(800, dpi) - 0.1))
    {
        return None;
    }
    let spacing_type = para_style.map_or(crate::model::style::LineSpacingType::Percent, |style| {
        style.line_spacing_type
    });
    let spacing = para_style.map_or(160.0, |style| style.line_spacing);
    // [#8582] 워드 호환 문서는 빈 앵커 줄도 빈 문단과 같이 끝 표지 글꼴 배율로 잰다.
    let font_size =
        super::composer::word_compat_empty_paragraph_font_size(para, styles, spacing_type)
            .map_or(font_size, |scaled| font_size.max(scaled));
    let (height, leading) =
        super::corrected_line_metrics(0.0, 0.0, font_size, spacing_type, spacing);
    Some((height, leading, font_size))
}

/// [#8582] 워드 호환 문서에서 저장 줄 없는 빈 문단이 문단 기준 자리차지 그림·도형만 가지면,
/// 그 띠에 걸리는 줄을 띠 아래로 민다(앵커 문단의 줄 포함).
///
/// 데스크톱 한컴 인쇄(`samples/issue8582/word_band.hwpx`, 높이 120 HU 직선, 줄 1860 HU):
/// 세로 오프셋 0·600 이면 앵커 줄이 띠에 걸려 띠 아래(120·720)에서 시작하고, 2374 면 앵커 줄
/// 다음 빈 줄(1860~3720)이 띠에 걸려 띠 아래(2494)로 간다. 이를 앵커 줄의 전진으로 낸다:
/// 띠가 앵커 줄에 걸리면 `띠 아래 + 줄`, 다음 줄에 걸리면 `띠 아래` 까지. 띠가 더 아래면
/// 종전 계약에 맡긴다. 한/글 문서는 앵커 줄이 띠를 흡수하는 다른 규칙이라 대상이 아니다.
fn word_compat_topbottom_band_host_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    dpi: f64,
) -> Option<(f64, f64, f64)> {
    if !styles.word_compat_target
        || !para.text.trim().is_empty()
        || para.controls.is_empty()
        || !super::para_has_no_stored_line_segs(para)
    {
        return None;
    }
    let mut band: Option<(f64, f64)> = None;
    let mut object_flow = 0.0f64;
    for control in &para.controls {
        let common = match control {
            Control::Shape(shape) => shape.common(),
            Control::Picture(picture) => &picture.common,
            _ => return None,
        };
        if !is_para_topbottom_float(common) {
            return None;
        }
        let top = super::hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
        let height_px = super::hwpunit_to_px(common.height as i32, dpi);
        let bottom = top + height_px;
        object_flow = object_flow.max(height_px);
        band = Some(band.map_or((top, bottom), |(t, b)| (t.min(top), b.max(bottom))));
    }
    let (band_top, band_bottom) = band?;
    let shape = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?;
    let font_size = styles.char_styles.get(shape as usize)?.font_size;
    if font_size <= 0.0 {
        return None;
    }
    let spacing_type = para_style.map_or(crate::model::style::LineSpacingType::Percent, |style| {
        style.line_spacing_type
    });
    let spacing = para_style.map_or(160.0, |style| style.line_spacing);
    let font_size =
        super::composer::word_compat_empty_paragraph_font_size(para, styles, spacing_type)
            .map_or(font_size, |scaled| font_size.max(scaled));
    let (height, leading) =
        super::corrected_line_metrics(0.0, 0.0, font_size, spacing_type, spacing);
    let advance = height + leading;
    if band_bottom <= 0.0 {
        return None;
    }
    // 개체 항목이 이미 흐름에 자기 높이를 더하므로(종전 계약) 그만큼 뺀 앵커 줄 전진을 낸다.
    if band_top < advance {
        // 앵커 줄이 띠에 걸린다: 줄은 띠 아래에서 시작한다.
        return Some((
            (band_bottom + height - object_flow).max(height),
            leading,
            font_size,
        ));
    }
    if band_top < advance * 2.0 {
        // 앵커 줄 다음 줄이 띠에 걸린다: 다음 줄은 띠 아래에서 시작한다.
        return Some((
            (band_bottom - leading - object_flow).max(height),
            leading,
            font_size,
        ));
    }
    None
}

/// 한/글 문서(호환 대상이 MS 워드가 아님)에서 저장 줄 없는 빈 문단이 문단 기준 자리차지
/// 직선만 가지면, 그 문단은 자기 글자 줄을 그대로 갖고 띠는 걸리는 줄만 민다.
///
/// - 띠가 앵커 줄 상자 안에서 시작하면(세로 오프셋 < 줄 높이) 앵커 줄이 띠를 흡수한다. 다음
///   문단은 `max(앵커 줄 끝, 띠 아래끝)` 에서 시작한다(저장 줄 문서의
///   [`super::empty_host_line_absorbed_by_topbottom_float`] 와 같은 관계).
/// - 띠가 그보다 아래면 앵커 줄 뒤 줄들 중 띠에 처음 걸리는 줄만 띠 아래로 밀린다. 뒤 줄은
///   앵커 줄과 같은 줄 피치로 이어진다고 보고 그 밀림을 앵커 줄 전진에 싣는다.
///
/// 웹한글기안기 인쇄(`samples/issue8582/hwp_band.hwpx`, 높이 120 HU 직선, 줄 피치 1100 HU):
/// 세로 오프셋 600·0 은 밀림 없음, 2374 는 셋째 줄(2200~3300)이 띠(2374~2494)에 걸려 +294 HU.
/// 개체 항목이 흐름에 더하는 개체 높이(종전 계약)는 앵커 줄 전진에서 뺀다.
pub(crate) fn hwp_topbottom_band_host_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    dpi: f64,
) -> Option<(f64, f64, f64)> {
    if styles.word_compat_target
        || !para.text.trim().is_empty()
        || para.controls.is_empty()
        || !super::para_has_no_stored_line_segs(para)
    {
        return None;
    }
    let mut band: Option<(f64, f64)> = None;
    let mut object_flow = 0.0f64;
    for control in &para.controls {
        // 직선만 본다. 그림·글상자·묶음 개체·표는 종전 경로(앵커 줄을 접고 개체 높이만 흐름에
        // 더함)를 따른다.
        let common = match control {
            Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Line(_)) => {
                shape.common()
            }
            _ => return None,
        };
        if !is_para_topbottom_float(common) {
            return None;
        }
        let top = super::hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
        let height_px = super::hwpunit_to_px(common.height as i32, dpi);
        object_flow = object_flow.max(height_px);
        band = Some(band.map_or((top, top + height_px), |(t, b)| {
            (t.min(top), b.max(top + height_px))
        }));
    }
    let (band_top, band_bottom) = band?;
    if band_top < 0.0 || band_bottom <= band_top {
        return None;
    }
    let shape = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?;
    let font_size = styles.char_styles.get(shape as usize)?.font_size;
    if font_size <= 0.0 {
        return None;
    }
    let spacing_type = para_style.map_or(crate::model::style::LineSpacingType::Percent, |style| {
        style.line_spacing_type
    });
    let spacing = para_style.map_or(160.0, |style| style.line_spacing);
    let (height, leading) =
        super::corrected_line_metrics(0.0, 0.0, font_size, spacing_type, spacing);
    let pitch = height + leading;
    if pitch <= 0.0 {
        return None;
    }
    let flow = if band_top < height {
        // 앵커 줄 상자 안의 띠: 줄이 흡수하고 다음 문단은 줄 끝과 띠 아래끝 중 아래에서.
        pitch.max(band_bottom)
    } else {
        // 띠에 처음 걸리는 뒤 줄(상자 [k·피치, k·피치 + 줄 높이))만 띠 아래로 민다.
        let first_hit = (1..)
            .map(|k| k as f64 * pitch)
            .take_while(|line_top| *line_top < band_bottom)
            .find(|line_top| line_top + height > band_top);
        pitch + first_hit.map_or(0.0, |line_top| band_bottom - line_top)
    };
    Some(((flow - leading - object_flow).max(0.0), leading, font_size))
}

/// 양수 첫줄 들여쓰기는 텍스트 없이도 줄 안쪽을 차지한다.
/// 왼쪽 여백과 내어쓰기는 줄 밖의 기하이며, 빈 개체 줄을 만들지 않는다.
/// 자리차지 밴드 위에 줄 높이가 들어가면 그 줄은 표 뒤 꼬리가 아니다.
/// 공백을 포함한 실제 텍스트는 별도 문단 조각이 이미 소유한다. 표 꼬리에
/// 그 줄을 더하지 않는다. 저장 줄·글앞/글뒤 장식 외의 다른 개체도 기존 줄/밴드 경로를 유지한다.
pub(crate) fn trailing_advance(
    para: &Paragraph,
    table: &Table,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    if !super::para_has_no_stored_line_segs(para)
        || !is_para_topbottom_float(&table.common)
        || !table.common.flow_with_text
        || table.common.vert_align != VertAlign::Top
        || !para.text.is_empty()
        || para.controls.iter().any(|control| {
            !matches!(
                control,
                Control::Table(_) | Control::SectionDef(_) | Control::ColumnDef(_)
            ) && !is_non_flow_decoration(control)
        })
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return None;
    }
    let style_id = composed.map_or(para.para_shape_id, |paragraph| paragraph.para_style_id);
    let style = styles.para_styles.get(style_id as usize)?;
    // 글앞·글뒤 장식을 함께 가진 빈 문단은 표만 가진 개체 앵커가 아니라 자기 빈 줄을 갖는다.
    // 음수 오프셋 표는 host 원점 위로 올라가 재조판 표 상자 계약(오프셋 ≥ 0) 밖이다.
    let owns_line = signed_hwpunit(table.common.vertical_offset) >= 0
        && para.controls.iter().any(is_non_flow_decoration);
    let word_compat = styles.word_compat_target;
    if style.indent <= 0.0 && !owns_line && !word_compat {
        return Some(0.0);
    }
    // 종단 글자 모양이 빈 줄 상자를 소유한다. 개체 제어 문자의
    // 모양은 빈 종단 run보다 큰 글꼴을 우연히 가지고 있어도 줄 높이를 바꾸지 않는다.
    let end_shape = para
        .char_shapes
        .iter()
        .find(|shape| shape.start_pos == para.char_count.saturating_sub(1))
        .map(|shape| shape.char_shape_id)
        .or_else(|| para.char_shape_id_at(0))
        .or_else(|| para.char_shapes.last().map(|shape| shape.char_shape_id))?;
    let font_size = styles.char_styles.get(end_shape as usize)?.font_size;
    if font_size <= 0.0 {
        return None;
    }
    // [#8582] 호환 대상이 MS 워드인 문서는 표만 가진 빈 앵커 문단의 줄(문단 끝 표지)을
    // 들여쓰기와 무관하게 표 **아래**에 한 줄로 놓는다(데스크톱 한컴 인쇄: 7pt 함초롬바탕
    // 앵커 문단은 표 아래에 1184 HU = 700 × 1.691 이 붙는다. 한/글 문서는 0).
    // 줄 높이는 빈 문단과 같이 끝 표지 글꼴의 한 줄 높이로 잰다(고정 줄 간격은 배율 없음).
    if word_compat {
        let font_size = super::composer::word_compat_empty_paragraph_font_size(
            para,
            styles,
            style.line_spacing_type,
        )
        .map_or(font_size, |scaled| font_size.max(scaled));
        let (height, leading) = super::corrected_line_metrics(
            0.0,
            0.0,
            font_size,
            style.line_spacing_type,
            style.line_spacing,
        );
        return Some(height + leading);
    }
    let (height, leading) = super::corrected_line_metrics(
        0.0,
        0.0,
        font_size,
        style.line_spacing_type,
        style.line_spacing,
    );
    let band_top = super::hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    Some(if height <= band_top {
        0.0
    } else {
        height + leading
    })
}
