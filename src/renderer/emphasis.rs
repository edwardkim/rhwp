//! 한컴 Print의 강조점 글리프 역할과 표시 자소별 배치 결과.
use super::{render_tree::BoundingBox, text_direction::VisualTextPositions, TextStyle};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) const FONT_FAMILY: &str = "HCR Batang";

/// 원문 scalar 주소의 자소 소속이다. run 분할·글꼴·좌표로 소속을 추측하지 않는다.
pub(crate) struct SourceGraphemes {
    characters: Vec<char>,
    starts: Vec<bool>,
    rendered_sources: Vec<Option<usize>>,
}
impl SourceGraphemes {
    pub fn new(
        para: &crate::model::paragraph::Paragraph,
        styles: &super::style_resolver::ResolvedStyleSet,
    ) -> Self {
        use crate::model::control::Control;
        let characters = para.text.chars().collect::<Vec<_>>();
        let mut formatting_starts = vec![false; characters.len()];
        let runs = para.char_shape_runs(0, characters.len().min(para.char_offsets.len()));
        for pair in runs.windows(2) {
            let before = styles.char_styles.get(pair[0].char_shape_id as usize);
            let after = styles.char_styles.get(pair[1].char_shape_id as usize);
            // ID만 다른 동등한 모양은 한컴에서도 같은 강조점 구간이다.
            // 실제 서식 변경만 자소 소속을 끊으며 언어별 표시 런 분할은 제외한다.
            if before.zip(after).is_some_and(|(a, b)| a != b) {
                formatting_starts[pair[1].start_offset] = true;
            }
        }
        let positions = para.control_text_positions();
        let mut controls = para
            .controls
            .iter()
            .zip(positions)
            .filter(|(control, _)| control.is_logical_inline())
            .collect::<Vec<_>>();
        controls.sort_by_key(|(_, position)| *position);
        let mut controls = controls.into_iter().peekable();
        let mut surface = String::new();
        let mut surface_sources = Vec::new();
        let mut rendered_sources = Vec::new();
        for index in 0..=characters.len() {
            if formatting_starts.get(index).copied().unwrap_or(false) {
                // 표시·저장하지 않는 경계이므로 rendered_sources는 전진시키지 않는다.
                surface.push('\0');
                surface_sources.push(None);
            }
            while controls
                .peek()
                .is_some_and(|(_, position)| *position == index)
            {
                let (control, _) = controls.next().unwrap();
                // 객체는 자소를 끊는다. 선두 주석 번호는 줄 밖 prefix여서 본문
                // 런 주소에 없지만, 원문 자소 경계에서는 여전히 별도 소유자다.
                // U+FFFC에는 뒤 결합 문자가 붙으므로 자소 투영에는 GB4/GB5의
                // 제어 경계를 쓴다. 이 문자열은 표시·저장하지 않는다.
                surface.push('\0');
                surface_sources.push(None);
                if index != 0 || !matches!(control, Control::Footnote(_) | Control::Endnote(_)) {
                    rendered_sources.push(None);
                }
            }
            if let Some(&ch) = characters.get(index) {
                surface.push(ch);
                surface_sources.push(Some(index));
                rendered_sources.push(Some(index));
            }
        }
        let mut starts = vec![false; characters.len()];
        let mut offset = 0;
        for cluster in surface.graphemes(true) {
            if let Some(index) = surface_sources[offset] {
                starts[index] = true;
            }
            offset += cluster.chars().count();
        }
        Self {
            characters,
            starts,
            rendered_sources,
        }
    }

    /// 전용 인라인 표 경로는 원문 scalar 위치를 이미 보유한다.
    pub fn apply_source_style(&self, start: usize, style: &mut TextStyle) {
        if style.emphasis_dot != 0 {
            style.emphasis_skip_first_cluster = !self.starts.get(start).copied().unwrap_or(true);
        }
    }
    pub fn apply(&self, run: &mut super::render_tree::TextRunNode) {
        use super::render_tree::FieldMarkerType;
        if run.style.emphasis_dot == 0 || run.field_marker != FieldMarkerType::None {
            return;
        }
        let Some(start) = run
            .char_start
            .and_then(|start| self.rendered_sources.get(start).copied().flatten())
        else {
            return;
        };
        // 생성 표시값은 별도 소유자다. 실제 원문과 대응하는 run에만 적용한다.
        if self.characters.get(start).copied() == run.text.chars().next() {
            self.apply_source_style(start, &mut run.style);
        }
    }
}

pub(crate) fn glyph(kind: u8) -> Option<&'static str> {
    match kind {
        1 => Some("\u{0307}"),
        2 => Some("\u{030a}"),
        3 => Some("\u{030c}"),
        4 => Some("\u{0303}"),
        5 => Some("\u{302e}"),
        6 => Some("\u{302f}"),
        _ => None,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmphasisGeometry {
    pub font_family: &'static str,
    pub glyph: &'static str,
    pub font_size: f64,
    pub bold: bool,
    pub ratio: f64,
    pub skew_x: f64,
    pub baseline_offset: f64,
    pub positions: Vec<f64>,
    /// 브라우저의 방점 셰이핑에서 보조 원 삽입을 막는 잉크 없는 문맥.
    pub browser_text: String,
    pub browser_offset: f64,
    /// 빈 문맥 자소의 실제 전진을 문자열 끝점 정렬로 상쇄한다.
    pub browser_align_end: bool,
}

impl EmphasisGeometry {
    /// clip/paint 범위에만 쓰는 보수적 em 상자이며 글줄·캐럿을 늘리지 않는다.
    pub fn ink_bounds(&self) -> Option<BoundingBox> {
        let left = self.positions.iter().copied().reduce(f64::min)?;
        let right = self.positions.iter().copied().reduce(f64::max)?;
        let pad = self.font_size * (self.ratio + self.skew_x.abs());
        Some(BoundingBox::new(
            left - pad,
            self.baseline_offset - self.font_size,
            right - left + pad * 1.25,
            self.font_size * 1.25,
        ))
    }
}

pub(crate) fn geometry(
    text: &str,
    positions: &[f64],
    style: &TextStyle,
    font_size: f64,
) -> Option<EmphasisGeometry> {
    let glyph = glyph(style.emphasis_dot)?;
    let ratio = style.ratio;
    if positions.len() != text.chars().count() + 1
        || positions.iter().any(|value| !value.is_finite())
        || !font_size.is_finite()
        || font_size <= 0.0
        || !ratio.is_finite()
        || ratio <= 0.0
    {
        return None;
    }
    let tone = style.emphasis_dot >= 5;
    let visual = VisualTextPositions::for_style(text, positions, style);
    let mut origins = Vec::new();
    let mut start = 0;
    for cluster in text.graphemes(true) {
        let end = start + cluster.chars().count();
        let width = positions[end] - positions[start];
        if width >= 0.0
            && !(start == 0 && style.emphasis_skip_first_cluster)
            && cluster.chars().any(|ch| {
                !ch.is_whitespace()
                    && !ch.is_control()
                    && ch.general_category() != GeneralCategory::Format
                    && ch != '\u{fffc}'
            })
        {
            let left = visual
                .as_ref()
                .and_then(|v| v.span_origin(start, end))
                .unwrap_or(positions[start]);
            origins.push(
                left + if tone {
                    // 방점 원점은 물리 em 오프셋이며 glyph 장평과 별개다.
                    font_size * 0.89
                } else {
                    width / 2.0 + font_size * ratio * 0.225
                },
            );
        }
        start = end;
    }
    Some(EmphasisGeometry {
        font_family: FONT_FAMILY,
        glyph,
        font_size,
        bold: style.is_visually_bold(),
        ratio,
        skew_x: if style.italic {
            super::text_italic::SKEW_X
        } else {
            0.0
        },
        baseline_offset: if tone { 0.0 } else { -font_size * 0.37 },
        positions: origins,
        browser_text: if tone {
            format!("\u{115f}\u{1160}{glyph}")
        } else {
            glyph.to_owned()
        },
        browser_offset: 0.0,
        browser_align_end: tone,
    })
}
