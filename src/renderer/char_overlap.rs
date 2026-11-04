//! 원문 겹침의 글자모양과 펼침을 해소하고 모든 출력 경로에 같은 배치를 전달한다.
//! 덧말도 같은 제어 문자(0x17) 한 칸에 본말과 덧말을 함께 그리므로 이 배치를 쓴다.

use super::composer::char_overlap_size_ratio;
use super::layout::{resolved_to_text_style, EmbeddedTextMeasurer, TextMeasurer};
use super::render_tree::{BoundingBox, TextRunNode};
use super::style_resolver::{detect_lang_category, ResolvedStyleSet};
use super::TextStyle;
use crate::model::control::{CharOverlap, Ruby};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Composition {
    border: Option<char>,
    owner_style: TextStyle,
    characters: Vec<(char, TextStyle)>,
    spread: bool,
    size_ratio: f64,
    /// 덧말이면 `characters`는 본말이고, 이 글자들을 본말 위나 아래에 작게 쓴다.
    ruby: Option<Box<RubyText>>,
    /// 테두리 없는 겹침이 전각 한 칸을 차지하는지(HWP 전각 공백 표지).
    full_width_cell: bool,
}

crate::storage::clone_fields!(Composition, {border, owner_style, characters, spread, size_ratio, ruby, full_width_cell});

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RubyText {
    characters: Vec<(char, TextStyle)>,
    below: bool,
}

crate::storage::clone_fields!(RubyText, { characters, below });

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

/// 채운 모양은 안쪽 글자를 테두리에서 비워 낸다.
fn reversed_border(kind: u8) -> bool {
    matches!(kind, 2 | 4 | 6 | 9 | 13)
}

/// 안쪽 글자 기준선을 줄이는 비율의 em 정렬점. 한컴 12.30 Print에서
/// 원·재활용 모양은 0.4, 삼각형은 0.3, 나머지는 1/3이다.
fn inner_alignment(kind: u8) -> f64 {
    match kind {
        1 | 11 | 12 | 13 => 0.4,
        5 | 6 => 0.3,
        _ => 1.0 / 3.0,
    }
}

impl Composition {
    pub(crate) fn from_model(
        co: &CharOverlap,
        styles: &ResolvedStyleSet,
        host_id: u32,
    ) -> Option<Box<Self>> {
        if co.chars.len() > 10 || co.border_type > 13 {
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
        let border = CharOverlap::border_glyph(co.border_type);
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
            ruby: None,
            // 한컴은 펼친 글자끼리 겹치기를 전각 한 칸에 그린다(HWP 원문은 그 칸을 전각 공백으로 둔다).
            full_width_cell: border.is_none() && co.expansion == 0,
        }))
    }

    /// 본말은 놓인 자리의 글자 모양으로, 덧말은 고른 스타일(없으면 본말)의 글꼴로 쓴다.
    /// 한컴 도움말처럼 덧말 크기는 스타일과 상관없이 본말의 `szRatio`%(0이면 절반)다.
    pub(crate) fn from_ruby(
        ruby: &Ruby,
        styles: &ResolvedStyleSet,
        host_id: u32,
    ) -> Option<Box<Self>> {
        if ruby.main_text.is_empty() {
            return None;
        }
        let owner_style = resolved_to_text_style(styles, host_id, 0);
        let characters: Vec<_> = ruby
            .main_text
            .chars()
            .filter(|c| !c.is_control())
            .map(|c| {
                (
                    c,
                    resolved_to_text_style(styles, host_id, detect_lang_category(c)),
                )
            })
            .collect();
        // option 셋째 비트(4)는 한컴 12.30이 덧말 스타일을 고를 때 켠다.
        let ruby_shape = (ruby.option & 4 != 0)
            .then(|| {
                styles
                    .style_char_shapes
                    .get(usize::from(ruby.style_id_ref))
                    .copied()
            })
            .flatten()
            .filter(|&id| (id as usize) < styles.char_styles.len())
            .unwrap_or(host_id);
        let ratio = if ruby.sz_ratio == 0 {
            0.5
        } else {
            f64::from(ruby.sz_ratio) / 100.0
        };
        let ruby_size = owner_style.font_size * ratio;
        let ruby_characters = ruby
            .ruby_text
            .chars()
            .filter(|c| !c.is_control())
            .map(|c| {
                let mut style = resolved_to_text_style(styles, ruby_shape, detect_lang_category(c));
                style.font_size = ruby_size;
                (c, style)
            })
            .collect();
        Some(Box::new(Self {
            border: None,
            owner_style,
            characters,
            spread: false,
            size_ratio: 1.0,
            ruby: Some(Box::new(RubyText {
                characters: ruby_characters,
                below: ruby.pos_type == 1,
            })),
            full_width_cell: false,
        }))
    }

    /// 덧말 본말의 글자별 전진. 본말 글자 모양의 자간을 함께 둔다.
    fn ruby_main_advances(&self) -> Vec<f64> {
        self.characters
            .iter()
            .map(|(c, style)| {
                let positions = EmbeddedTextMeasurer.compute_char_positions(&c.to_string(), style);
                positions.last().copied().unwrap_or(0.0)
            })
            .collect()
    }

    /// 줄 나눔이 재는 한 칸의 전진(px).
    pub(crate) fn advance_px(&self) -> f64 {
        self.advance()
    }

    fn advance(&self) -> f64 {
        if self.ruby.is_some() {
            // 한컴은 덧말이 길어도 본말 폭만 차지하고 덧말을 줄여 쓴다.
            return self.ruby_main_advances().iter().sum();
        }
        let character = self
            .border
            .or_else(|| self.full_width_cell.then_some('\u{3000}'))
            .or_else(|| self.characters.first().map(|v| v.0))
            // 빈 겹침도 한컴 저장본의 전각 캐리어 한 칸을 점유한다.
            .unwrap_or('\u{3000}');
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

/// 한컴 12.30 Print: 덧말은 본말 글자 상자(기준선 위 0.85em, 아래 0.15em)에 맞붙어 가운데
/// 정렬된다. 본말보다 길면 장평을 절반까지 줄이고, 그래도 넘치면 본말 왼쪽에서 시작한다.
fn project_ruby(
    owner: &TextRunNode,
    bbox: BoundingBox,
    composition: &Composition,
    ruby: &RubyText,
) -> Projection {
    let baseline = bbox.y + owner.baseline;
    let main_size = composition.owner_style.effective_font_size();
    let mut interior = Vec::with_capacity(composition.characters.len() + ruby.characters.len());
    let mut cursor = bbox.x;
    for ((character, style), advance) in composition
        .characters
        .iter()
        .zip(composition.ruby_main_advances())
    {
        let mut drawn = glyph(owner, *character, style.clone(), 0.0, baseline);
        drawn.bbox.x = cursor;
        interior.push(drawn);
        cursor += advance;
    }
    let main_width = cursor - bbox.x;
    let ruby_size = ruby
        .characters
        .first()
        .map_or(0.0, |(_, style)| style.effective_font_size());
    let ruby_baseline = if ruby.below {
        baseline + main_size * 0.15 + ruby_size * 0.85
    } else {
        baseline - main_size * 0.85 - ruby_size * 0.15
    };
    let natural: Vec<_> = ruby
        .characters
        .iter()
        .map(|(c, style)| {
            glyph(owner, *c, style.clone(), 0.0, ruby_baseline)
                .bbox
                .width
        })
        .collect();
    let natural_width: f64 = natural.iter().sum();
    let scale = if natural_width > main_width && natural_width > 0.0 {
        (main_width / natural_width).max(0.5)
    } else {
        1.0
    };
    let mut cursor = bbox.x + ((main_width - natural_width * scale) / 2.0).max(0.0);
    for ((character, original), width) in ruby.characters.iter().zip(natural) {
        let mut style = original.clone();
        style.ratio = if style.ratio > 0.0 { style.ratio } else { 1.0 } * scale;
        let mut drawn = glyph(owner, *character, style, 0.0, ruby_baseline);
        drawn.bbox.x = cursor;
        interior.push(drawn);
        cursor += width * scale;
    }
    let bounds = interior.iter().fold(bbox, |bounds, g| {
        let left = bounds.x.min(g.bbox.x);
        let top = bounds.y.min(g.bbox.y);
        let right = (bounds.x + bounds.width).max(g.bbox.x + g.bbox.width);
        let bottom = (bounds.y + bounds.height).max(g.bbox.y + g.bbox.height);
        BoundingBox::new(left, top, right - left, bottom - top)
    });
    Projection {
        border: None,
        interior,
        reversed: false,
        bounds,
    }
}

pub(crate) fn project(owner: &TextRunNode, bbox: BoundingBox) -> Option<Projection> {
    if owner.is_vertical {
        return None;
    }
    let overlap = owner.char_overlap.as_ref()?;
    let composition = overlap.composition.as_ref()?;
    if let Some(ruby) = &composition.ruby {
        return Some(project_ruby(owner, bbox, composition, ruby));
    }
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
    let reversed = reversed_border(overlap.border_type);
    let ratio = composition.size_ratio;
    if ratio <= 0.0 {
        return None;
    }
    // 한컴 Print의 테두리 모양은 내부 기준선을 서로 다른 em 정렬점으로 공유한다.
    let alignment = inner_alignment(overlap.border_type);
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
