//! HWP5 표 33·35의 글자 그림자를 한 번 해소해 출력 경로가 함께 재생한다.

use crate::renderer::TextStyle;

/// 실제 글자 효과가 내장 SFNT 메트릭을 요구하는지 원본 종류로 판정한다.
pub(crate) fn needs_font_metrics(node: &crate::renderer::render_tree::RenderNode) -> bool {
    use crate::renderer::render_tree::RenderNodeType;
    matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.style.shadow_type == 2 && !run.is_vertical)
        || node.children.iter().any(needs_font_metrics)
}

/// 인쇄 장치에서 해소한 연속 그림자. 원문·글줄·전진 폭에는 참여하지 않는다.
#[derive(Clone, Debug, PartialEq)]
pub struct ContinuousTextShadow {
    delta: [i32; 2],
    device_pixel: f64,
    /// 실제 거리 생산에 사용한 프로그램을 식별한다. 글꼴 파일은 보관하지 않는다.
    pub source_hash: [u8; 32],
    pub face_index: u32,
}

impl ContinuousTextShadow {
    pub fn offsets(&self) -> impl Iterator<Item = [f64; 2]> + '_ {
        let count = i64::from(self.delta[0])
            .abs()
            .max(i64::from(self.delta[1]).abs())
            + 1;
        (1..=count).map(move |step| {
            self.delta.map(|delta| {
                // 독립 Print의 혼합 축·음수 대조: 부호 있는 정수 나눗셈이다.
                (step * i64::from(delta) / count) as f64 * self.device_pixel
            })
        })
    }

    pub fn extent(&self) -> [f64; 2] {
        self.delta.map(|value| f64::from(value) * self.device_pixel)
    }
}

/// 세 backend가 같은 순서의 그림자를 소비한다. 원문은 이 반복 뒤에 한 번만 그린다.
pub(crate) fn offsets(style: &TextStyle) -> impl Iterator<Item = [f64; 2]> + '_ {
    // 편집·출력 투영에서 종류가 바뀌면 이전 세션 계획으로 효과를 되살리지 않는다.
    let continuous = style
        .continuous_shadow
        .as_ref()
        .filter(|_| style.shadow_type == 2)
        .map(|plan| plan.offsets());
    let endpoint = (style.shadow_type > 0 && continuous.is_none())
        .then_some([style.shadow_offset_x, style.shadow_offset_y]);
    continuous.into_iter().flatten().chain(endpoint)
}

/// 전송 소비자는 SFNT 메트릭이나 보간을 다시 계산하지 않고 확정된 표본을 재생한다.
pub(super) fn write_offsets_json(buf: &mut String, kind: u8, plan: Option<&ContinuousTextShadow>) {
    use std::fmt::Write;
    let Some(plan) = plan.filter(|_| kind == 2) else {
        return;
    };
    buf.push_str(",\"shadowOffsets\":[");
    for (index, [x, y]) in plan.offsets().enumerate() {
        if index != 0 {
            buf.push(',');
        }
        let _ = write!(buf, "[{x},{y}]");
    }
    buf.push(']');
}

/// 레이아웃의 본문 clip은 흐름 콘텐츠를 포함한다. paint에서 해소한 그림자도 같은 소유다.
/// 셀·글상자·곡선 clip이나 부동 변환 그룹의 경계를 넘어 확장하지 않는다.
pub(super) fn flow_bounds(
    node: &super::LayerNode,
) -> Option<crate::renderer::render_tree::BoundingBox> {
    use super::{GroupKind, LayerNodeKind, PaintOp};
    use crate::renderer::render_tree::BoundingBox;
    let union = |a: BoundingBox, b: BoundingBox| {
        let x = a.x.min(b.x);
        let y = a.y.min(b.y);
        BoundingBox::new(
            x,
            y,
            (a.x + a.width).max(b.x + b.width) - x,
            (a.y + a.height).max(b.y + b.height) - y,
        )
    };
    match &node.kind {
        LayerNodeKind::Leaf { ops } => ops
            .iter()
            .filter_map(|op| {
                let PaintOp::TextRun {
                    bbox,
                    run,
                    paint_transform: None,
                    ..
                } = op
                else {
                    return None;
                };
                if run.style.shadow_type != 2
                    || run.rotation != 0.0
                    || run.is_vertical
                    || crate::renderer::text_glyph_visibility::is_nonprinting_cluster(
                        run.display_or_text(),
                    )
                {
                    return None;
                }
                let [dx, dy] = run.style.continuous_shadow.as_ref()?.extent();
                Some(union(
                    *bbox,
                    BoundingBox::new(bbox.x + dx, bbox.y + dy, bbox.width, bbox.height),
                ))
            })
            .reduce(union),
        LayerNodeKind::Group {
            group_kind: GroupKind::TextBox | GroupKind::Group(_),
            ..
        }
        | LayerNodeKind::ClipRect { .. }
        | LayerNodeKind::ClipPath { .. } => None,
        LayerNodeKind::Group { children, .. } => {
            children.iter().filter_map(flow_bounds).reduce(union)
        }
    }
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn signed_at(bytes: &[u8], offset: usize) -> Option<i16> {
    u16_at(bytes, offset).map(|value| value as i16)
}

/// OpenType VDMX의 첫 대응 aspect group만 수용한다. 없는 크기를 보간하지 않는다.
fn vdmx_height(bytes: &[u8], ppem: u16) -> Option<i32> {
    if u16_at(bytes, 0)? > 1 {
        return None;
    }
    let ratios = usize::from(u16_at(bytes, 4)?);
    let offsets = 6usize.checked_add(ratios.checked_mul(4)?)?;
    bytes.get(..offsets.checked_add(ratios.checked_mul(2)?)?)?;
    for index in 0..ratios {
        let ratio = 6 + index * 4;
        let x = bytes[ratio + 1];
        let low = bytes[ratio + 2];
        let high = bytes[ratio + 3];
        if !((x == 0 && low == 0 && high == 0) || (low <= x && x <= high)) {
            continue;
        }
        let group = usize::from(u16_at(bytes, offsets + index * 2)?);
        let count = usize::from(u16_at(bytes, group)?);
        for entry in 0..count {
            let at = group.checked_add(4)?.checked_add(entry.checked_mul(6)?)?;
            let size = u16_at(bytes, at)?;
            if size == ppem {
                let height =
                    i32::from(signed_at(bytes, at + 2)?) - i32::from(signed_at(bytes, at + 4)?);
                return (height > 0).then_some(height);
            }
            if size > ppem {
                return None;
            }
        }
        return None;
    }
    None
}

fn font_cell(face: &ttf_parser::Face<'_>, ppem: u16) -> Option<i32> {
    // 이번 계약은 수평 TrueType의 실제 Print 대조다. CFF·세로 쓰기로 확대하지 않는다.
    face.raw_face()
        .table(ttf_parser::Tag::from_bytes(b"glyf"))?;
    let height = face
        .raw_face()
        .table(ttf_parser::Tag::from_bytes(b"VDMX"))
        .and_then(|bytes| vdmx_height(bytes, ppem))
        .or_else(|| {
            let os2 = face
                .raw_face()
                .table(ttf_parser::Tag::from_bytes(b"OS/2"))?;
            let ascent = u16_at(os2, 74)?;
            let descent = u16_at(os2, 76)?;
            let units = f64::from(face.units_per_em());
            (ascent != 0 || descent != 0).then(|| {
                (f64::from(ascent) * f64::from(ppem) / units).round() as i32
                    + (f64::from(descent) * f64::from(ppem) / units).round() as i32
            })
        })?;
    (height > 0).then_some(height + (height - i32::from(ppem)))
}

pub(super) fn prepare(
    fonts: &super::metafile_fonts::MetafileFonts,
    text: &str,
    style: &TextStyle,
    layout_dpi: f64,
) -> Option<ContinuousTextShadow> {
    if style.shadow_type != 2
        || crate::renderer::text_glyph_visibility::is_nonprinting_cluster(text)
    {
        return None;
    }
    let percent = style.shadow_offset_percent?;
    let (size, _) = style.script_draw_metrics(style.font_size, 0.0);
    let em = (size * super::metafile::HANCOM_2024_PRINT_DEVICE_DPI / layout_dpi).round();
    if !em.is_finite() || !(1.0..=f64::from(u16::MAX)).contains(&em) {
        return None;
    }
    let database = fonts.database();
    for id in fonts.supplied_faces_for_style(style) {
        let plan = database
            .with_face_data(id, |bytes, face_index| {
                let face = ttf_parser::Face::parse(bytes, face_index).ok()?;
                if !text
                    .chars()
                    .filter(|ch| {
                        !crate::renderer::text_glyph_visibility::is_nonprinting_character(*ch)
                    })
                    .all(|ch| face.glyph_index(ch).is_some_and(|glyph| glyph.0 != 0))
                {
                    return None;
                }
                let cell = font_cell(&face, em as u16)?;
                // 원본 signed 백분율은 HWPUNIT에서 먼저 절삭한 뒤 장치로 환산한다.
                let basis_hu = (f64::from(cell) * 7200.0
                    / super::metafile::HANCOM_2024_PRINT_DEVICE_DPI)
                    .round();
                let delta = percent.map(|gap| {
                    let logical = (basis_hu * f64::from(gap) / 100.0).trunc();
                    (logical * super::metafile::HANCOM_2024_PRINT_DEVICE_DPI / 7200.0).round()
                        as i32
                });
                Some(ContinuousTextShadow {
                    delta,
                    device_pixel: layout_dpi / super::metafile::HANCOM_2024_PRINT_DEVICE_DPI,
                    source_hash: *blake3::hash(bytes).as_bytes(),
                    face_index,
                })
            })
            .flatten();
        if plan.is_some() {
            return plan;
        }
    }
    None
}
