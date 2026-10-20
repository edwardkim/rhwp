//! 이미 방출한 저장 호스트 줄과 흐름 밖 그림의 좌표 기준 소유를 확인한다.

use crate::model::{control::Control, paragraph::Paragraph, shape::TextWrap};
use crate::renderer::render_tree::{RenderNode, RenderNodeType};

pub(super) struct ShapeHostFlow {
    pub before: f64,
    pub after: f64,
    pub spacing_after: f64,
    pub dpi: f64,
    pub textless_host_end: Option<f64>,
}

/// 호스트 줄의 전진이 끝난 뒤 그림만 출력한 항목은 저장 사다리의 원점을 바꾸지 않는다.
/// 가시 문자열 여부와 줄 점유를 구별하며, 실제 방출한 마지막 줄과 전진 결과를 대조한다.
pub(super) fn preserves_line_origin(
    paragraph: &Paragraph,
    para_index: usize,
    control_index: usize,
    column: &RenderNode,
    flow: ShapeHostFlow,
) -> bool {
    let ShapeHostFlow {
        before: flow_before,
        after: flow_after,
        spacing_after,
        dpi,
        textless_host_end,
    } = flow;
    let common = match paragraph.controls.get(control_index) {
        Some(Control::Picture(picture)) => &picture.common,
        Some(Control::Shape(shape)) => shape.common(),
        _ => return false,
    };
    if common.treat_as_char
        || !matches!(
            common.text_wrap,
            TextWrap::Square
                | TextWrap::TopAndBottom
                | TextWrap::Tight
                | TextWrap::Through
                | TextWrap::InFrontOfText
                | TextWrap::BehindText
        )
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.cell_format_vpos_dirty
        || !flow_before.is_finite()
        || !flow_after.is_finite()
    {
        return false;
    }
    // 픽셀 관용치를 두지 않는다. 같은 흐름 원점의 부동소수점 반올림만 허용한다.
    let roundoff = (flow_before.abs() + flow_after.abs()) * f64::EPSILON;
    if (flow_after - flow_before).abs() > roundoff {
        return false;
    }
    // 글자가 없는 호스트는 TextLine을 그리지 않아도 줄 공간을 예약한다.
    // 그 분기에서 저장 사다리로 실제 수용한 끝을 재사용하며 예약량을 재추정하지 않는다.
    if let Some(end) = textless_host_end {
        return same_coordinate(flow_before, end);
    }
    let Some(first) = paragraph.line_segs.first() else {
        return false;
    };
    let mut origin = None;
    let mut end = None;
    // 저장 태그나 마지막 줄의 존재만으로 현재 배치가 그 사다리를 썼다고
    // 추정하지 않는다. 재조판한 본문 줄은 같은 인덱스라도 메트릭이 다를 수 있다.
    for (index, stored) in paragraph.line_segs.iter().enumerate() {
        if stored.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || stored.vertical_pos < first.vertical_pos
        {
            return false;
        }
        let Ok(index) = u32::try_from(index) else {
            return false;
        };
        // 셀/글상자 안의 줄이나 앞 조각만으로 호스트의 완료를 추정하지 않는다.
        let Some(actual) = column.children.iter().find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
                if line.para_index == Some(para_index) && line.line_index == Some(index))
        }) else {
            return false;
        };
        let origin = *origin.get_or_insert(actual.bbox.y);
        let expected_top =
            origin + crate::renderer::hwpunit_to_px(stored.vertical_pos - first.vertical_pos, dpi);
        let expected_height = crate::renderer::hwpunit_to_px(stored.line_height, dpi);
        if !same_coordinate(actual.bbox.y, expected_top)
            || !same_coordinate(actual.bbox.height, expected_height)
        {
            return false;
        }
        end = Some(
            actual.bbox.y
                + expected_height
                + crate::renderer::hwpunit_to_px(stored.line_spacing, dpi)
                + spacing_after,
        );
    }
    // 호스트 줄의 후행 간격까지 실제로 소비한 경우만 원점이 확립됐다.
    // 저장 간격의 일부만 예약하는 호환 경로는 기존 기준 복구가 필요하다.
    end.is_some_and(|end| same_coordinate(flow_before, end))
}

fn same_coordinate(actual: f64, expected: f64) -> bool {
    let roundoff = (actual.abs() + expected.abs() + 1.0) * f64::EPSILON * 8.0;
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= roundoff
}
