//! 원문 겹침의 글자모양과 펼침을 해소하고 모든 출력 경로에 같은 배치를 전달한다.

use super::composer::char_overlap_size_ratio;
use super::layout::{resolved_to_text_style, EmbeddedTextMeasurer, TextMeasurer};
use super::render_tree::{BoundingBox, TextRunNode};
use super::style_resolver::{detect_lang_category, ResolvedStyleSet};
use super::TextStyle;
use crate::model::control::CharOverlap;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Composition {
    border: Option<char>,
    owner_style: TextStyle,
    characters: Vec<(char, TextStyle)>,
    spread: bool,
    size_ratio: f64,
}

pub(crate) struct Glyph {
    pub bbox: BoundingBox,
    pub run: TextRunNode,
}

pub(crate) struct Projection {
    pub border: Option<Glyph>,
    pub interior: Vec<Glyph>,
    pub reversed: bool,
    pub bounds: BoundingBox,
}

fn border_character(kind: u8) -> Option<char> {
    match kind {
        1 => Some('◯'),
        2 => Some('●'),
        3 => Some('□'),
        4 => Some('■'),
        _ => None,
    }
}

impl Composition {
    pub(crate) fn from_model(
        co: &CharOverlap,
        styles: &ResolvedStyleSet,
        host_id: u32,
    ) -> Option<Box<Self>> {
        if co.chars.len() > 10 || co.border_type > 4 {
            return None;
        }
        // 폰트 인코딩의 PUA 조합 숫자는 기존의 별도 해독 계약을 사용한다.
        if co
            .chars
            .iter()
            .any(|c| matches!(u32::from(*c), 0xe000..=0xf8ff | 0xf0000..=0xffffd))
        {
            return None;
        }
        let border = border_character(co.border_type);
        let has_prefix = co.chars.first().is_some_and(|c| {
            Some(*c) == border
                || *c == '\0'
                || (border.is_none() && co.expansion == 0 && *c == '\u{3000}')
        });
        let resolve = |index: usize, character: char| {
            let lang = detect_lang_category(character);
            match co
                .char_shape_ids
                .get(index)
                .filter(|&&id| (id as usize) < styles.char_styles.len())
            {
                Some(&id) => resolved_to_text_style(styles, id, lang),
                // 한컴 저장본의 누락 글자모양은 문서가 가져온 첫 ID와 구분한 기본 글자모양이다.
                None => TextStyle {
                    font_family: "한컴바탕".into(),
                    font_size: 10.0 * styles.resolved_dpi.unwrap_or(super::DEFAULT_DPI) / 72.0,
                    ..TextStyle::default()
                },
            }
        };
        // 첫 저장 모양은 내부 서식의 기준 스냅샷이다. 현재 문단과 다르면 한컴은
        // 예전 내부 서식을 적용하지 않고 현재 run의 서식을 전체에 적용한다.
        let reference_id = co.char_shape_ids.first().copied();
        let reference_valid = reference_id.is_some_and(|id| {
            match (
                styles.source_char_shapes.get(id as usize),
                styles.source_char_shapes.get(host_id as usize),
            ) {
                (Some(reference), Some(host)) => reference == host,
                _ => id == host_id,
            }
        });
        let owner_style = resolved_to_text_style(styles, host_id, 0);
        let characters = co
            .chars
            .iter()
            .skip(usize::from(has_prefix))
            .enumerate()
            .filter(|(_, c)| !c.is_control())
            .map(|(i, &c)| {
                (
                    c,
                    if reference_valid {
                        resolve(i + 1, c)
                    } else {
                        resolved_to_text_style(styles, host_id, detect_lang_category(c))
                    },
                )
            })
            .collect::<Vec<_>>();
        if characters.len() > 9 {
            return None;
        }
        Some(Box::new(Self {
            border,
            owner_style,
            characters,
            spread: co.expansion == 0,
            size_ratio: char_overlap_size_ratio(
                co.border_type,
                styles.char_overlap_inner_size(co.inner_char_size),
            ),
        }))
    }

    fn advance(&self) -> f64 {
        let character = self
            .border
            .or_else(|| self.characters.first().map(|v| v.0))
            .unwrap_or(' ');
        EmbeddedTextMeasurer.estimate_text_width(&character.to_string(), &self.owner_style)
    }
}

/// 논리 한 칸의 전진은 테두리 또는 기준 글자의 실제 전진 폭이다. 원문 문자 수와 다르다.
pub(crate) fn advance(
    text: &str,
    overlap: &super::composer::CharOverlapInfo,
    style: &TextStyle,
) -> f64 {
    overlap.composition.as_ref().map_or_else(
        || style.effective_font_size() * usize::from(!text.is_empty()) as f64,
        |c| c.advance(),
    )
}

fn glyph(
    owner: &TextRunNode,
    character: char,
    mut style: TextStyle,
    center: f64,
    baseline: f64,
) -> Glyph {
    style.letter_spacing = 0.0;
    let (draw_size, baseline) = style.script_draw_metrics(style.effective_font_size(), baseline);
    style.font_size = draw_size;
    style.font_position = None;
    style.superscript = false;
    style.subscript = false;
    let text = character.to_string();
    let positions = EmbeddedTextMeasurer.compute_char_positions(&text, &style);
    let width = *positions.last().unwrap_or(&0.0);
    let size = style.effective_font_size();
    let bbox = BoundingBox::new(center - width / 2.0, baseline - size * 0.85, width, size);
    let mut run = owner.clone();
    run.text = text;
    run.style = style;
    run.char_overlap = None;
    run.display_text = None;
    run.char_start = None;
    run.baseline = size * 0.85;
    run.layout_positions = Some(positions);
    run.rotation = 0.0;
    run.is_para_end = false;
    run.is_line_break_end = false;
    Glyph { bbox, run }
}

pub(crate) fn project(owner: &TextRunNode, bbox: BoundingBox) -> Option<Projection> {
    if owner.is_vertical {
        return None;
    }
    let overlap = owner.char_overlap.as_ref()?;
    let composition = overlap.composition.as_ref()?;
    let base_size = composition.owner_style.effective_font_size();
    let baseline = bbox.y + owner.baseline;
    let width = composition.advance();
    let border = composition.border.map(|c| {
        glyph(
            owner,
            c,
            composition.owner_style.clone(),
            bbox.x + width / 2.0,
            baseline,
        )
    });
    let reversed = matches!(overlap.border_type, 2 | 4);
    let ratio = composition.size_ratio;
    if ratio <= 0.0 {
        return None;
    }
    // 한컴 Print의 원/사각 테두리는 내부 기준선을 서로 다른 em 정렬점으로 공유한다.
    let alignment = if matches!(overlap.border_type, 1 | 2) {
        0.4
    } else {
        1.0 / 3.0
    };
    let inner_baseline = baseline + base_size * (ratio - 1.0) * alignment;
    let count = composition.characters.len();
    let mut interior = Vec::with_capacity(count);
    // 펼침은 동일한 중심 간격이 아니다. 원 글자의 자연 전진에 비례해 내부
    // 80% 폭을 나누고, 실제로 그린 글자들의 전체 경계를 가운데에 놓는다.
    // 크기 비율은 그릴 글자만 바꾸며 원 글자 사이에 배분한 전진은 보존한다.
    let natural_widths: Vec<_> = composition
        .characters
        .iter()
        .map(|(character, style)| {
            glyph(owner, *character, style.clone(), 0.0, inner_baseline)
                .bbox
                .width
        })
        .collect();
    let total_width: f64 = natural_widths.iter().sum();
    let mut cursor = 0.0;
    for (i, (character, original)) in composition.characters.iter().enumerate() {
        let mut style = original.clone();
        style.font_size *= ratio;
        if composition.spread {
            let share = if total_width > 0.0 {
                natural_widths[i] / total_width
            } else {
                1.0 / count as f64
            };
            let advance = width * 0.8 * share;
            let natural = glyph(owner, *character, style.clone(), 0.0, inner_baseline)
                .bbox
                .width;
            let base_scale = (1.0 / count as f64).min(if composition.border.is_none() {
                0.8
            } else {
                1.0
            });
            let minimum_scale = if natural > 0.0 {
                (advance / natural).min(1.0)
            } else {
                0.0
            };
            style.ratio =
                if style.ratio > 0.0 { style.ratio } else { 1.0 } * base_scale.max(minimum_scale);
            let mut drawn = glyph(owner, *character, style, 0.0, inner_baseline);
            drawn.bbox.x = cursor;
            interior.push(drawn);
            cursor += advance;
        } else {
            interior.push(glyph(
                owner,
                *character,
                style,
                bbox.x + width / 2.0,
                inner_baseline,
            ));
        }
    }
    if composition.spread {
        let extent = interior
            .iter()
            .map(|g| g.bbox.x + g.bbox.width)
            .fold(0.0, f64::max);
        let origin = bbox.x + (width - extent) / 2.0;
        for g in &mut interior {
            g.bbox.x += origin;
        }
    }
    let mut bounds = border.as_ref().map_or(bbox, |g| g.bbox);
    if !reversed {
        for g in &interior {
            let left = bounds.x.min(g.bbox.x);
            let top = bounds.y.min(g.bbox.y);
            let right = (bounds.x + bounds.width).max(g.bbox.x + g.bbox.width);
            let bottom = (bounds.y + bounds.height).max(g.bbox.y + g.bbox.height);
            bounds = BoundingBox::new(left, top, right - left, bottom - top);
        }
    }
    Some(Projection {
        border,
        interior,
        reversed,
        bounds,
    })
}

/// 실제로 회전해 그리는 잉크의 같은 영역을 본문 clip과 paint 기록에 제공한다.
pub(crate) fn ink_bounds(owner: &TextRunNode, bbox: BoundingBox) -> Option<BoundingBox> {
    let bounds = project(owner, bbox)?.bounds;
    if owner.rotation == 0.0 {
        return Some(bounds);
    }
    let (sin, cos) = owner.rotation.to_radians().sin_cos();
    let (cx, cy) = (bbox.x + bbox.width / 2.0, bbox.y + bbox.height / 2.0);
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for (x, y) in [
        (bounds.x, bounds.y),
        (bounds.x + bounds.width, bounds.y),
        (bounds.x, bounds.y + bounds.height),
        (bounds.x + bounds.width, bounds.y + bounds.height),
    ] {
        let dx = x - cx;
        let dy = y - cy;
        let x = cx + dx * cos - dy * sin;
        let y = cy + dx * sin + dy * cos;
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
    }
    Some(BoundingBox::new(left, top, right - left, bottom - top))
}
