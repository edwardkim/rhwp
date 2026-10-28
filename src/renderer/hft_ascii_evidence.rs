//! [#7051] HFT 한글 전용 face 의 ASCII 반각 조판을 **문서 자신의 저장 줄**로 판정한다.
//!
//! HWP3 시절 HFT 한글 전용 글꼴(`명조`·`한양신명조` 등)은 ASCII 를 한글 em 의 절반으로
//! 그린다. HWP5 변환본은 `HwpSummaryInformation` 의 HWP3 시대 연도로 그 계보를 안다
//! (`Document::is_hwp3_variant`). 같은 문서를 한컴이 HWPX 로 저장하면 그 신호가 사라진다 —
//! `content.hpf` 메타데이터는 자리표시자이고 `compatibleDocument`·`layoutCompatibility` 도
//! 일반 HWPX 와 같다(samples 452개 전부 `HWP201X` + 빈 호환 목록).
//!
//! 남는 근거는 한컴이 저장한 줄 사다리다. 저장 줄 하나가 담은 글자를 **비례 폭**으로 재면
//! 줄 폭(`segment_width`)을 넘고 **반각**으로 재면 들어가면, 그 줄은 반각 조판의 증인이다.
//! 거꾸로 다음 줄 첫 낱말이 반각으로는 들어가는데 한컴이 끊었다면 비례 폭의 증인이다.
//! 같은 HFT 이름을 쓰는 진짜 HWP5(`exam_kor`: `6` 을 0.645em 으로 그린다)는 두 번째
//! 증인을 남기거나 첫 번째 증인을 남기지 않는다.
//!
//! 판정은 증인 수의 비교다. 측정은 글자별 폭의 합이라 자간·커닝 오차가 있으므로 줄 폭의
//! [`FIT_TOLERANCE_RATIO`] 안쪽은 어느 쪽 증인으로도 세지 않는다.

use crate::model::document::Document;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::hwpunit_to_px;
use crate::renderer::layout::{estimate_text_width_unrounded, resolved_to_text_style};
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};

/// 줄 폭 대비 판정 유보 띠. 글자별 합산 측정의 오차를 덮는다.
const FIT_TOLERANCE_RATIO: f64 = 0.02;
/// 반각 증인이 이만큼은 있어야 문서 단위 판정을 내린다.
const MIN_HALFWIDTH_WITNESSES: usize = 3;
/// 반각 증인이 비례 증인의 이 배수 이상이어야 반각 문서로 본다.
const DOMINANCE: usize = 4;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HftAsciiWitnesses {
    pub halfwidth: usize,
    pub proportional: usize,
}

impl HftAsciiWitnesses {
    pub(crate) fn proves_halfwidth(self) -> bool {
        self.halfwidth >= MIN_HALFWIDTH_WITNESSES
            && self.halfwidth >= DOMINANCE * self.proportional.max(1)
    }
}

/// 본문 문단의 저장 줄을 비례·반각 두 측정으로 대조해 증인 수를 센다.
pub(crate) fn count_hft_ascii_witnesses(
    document: &Document,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> HftAsciiWitnesses {
    let mut witnesses = HftAsciiWitnesses::default();
    if !styles
        .char_styles
        .iter()
        .any(|cs| cs.font_families_hft_hangul.iter().any(|hft| *hft))
    {
        return witnesses;
    }
    for section in &document.sections {
        for para in &section.paragraphs {
            count_paragraph(para, styles, dpi, &mut witnesses);
        }
    }
    witnesses
}

fn stored_lines(para: &Paragraph) -> Vec<&LineSeg> {
    para.line_segs
        .iter()
        .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        .collect()
}

/// 글자 하나의 폭 — 비례(`false`)·반각(`true`) 두 해석.
fn char_widths(styles: &ResolvedStyleSet, shape_id: u32, ch: char) -> (f64, f64, bool) {
    let lang = detect_lang_category(ch);
    let mut style = resolved_to_text_style(styles, shape_id, lang);
    let hft_ascii = ch.is_ascii_graphic()
        && styles
            .char_styles
            .get(shape_id as usize)
            .is_some_and(|cs| cs.hft_hangul_face_for_lang(lang));
    let text = ch.to_string();
    style.hft_hangul_face = false;
    let proportional = estimate_text_width_unrounded(&text, &style);
    if !hft_ascii {
        return (proportional, proportional, false);
    }
    style.hft_hangul_face = true;
    (
        proportional,
        estimate_text_width_unrounded(&text, &style),
        true,
    )
}

fn count_paragraph(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    witnesses: &mut HftAsciiWitnesses,
) {
    let lines = stored_lines(para);
    if lines.len() < 2 || para.char_offsets.len() != para.text.chars().count() {
        return;
    }
    let chars: Vec<char> = para.text.chars().collect();
    // 글자 i 의 (비례 폭, 반각 폭, HFT ASCII 여부). 제어 문자가 섞인 문단은 폭 해석이
    // 갈리므로 건너뛴다.
    if chars.iter().any(|ch| (*ch as u32) < 0x20) {
        return;
    }
    let widths: Vec<(f64, f64, bool)> = chars
        .iter()
        .enumerate()
        .map(|(index, ch)| {
            let shape_id = para.char_shape_id_at(index).unwrap_or(0);
            char_widths(styles, shape_id, *ch)
        })
        .collect();
    let line_range = |line_index: usize| -> (usize, usize) {
        let start_pos = lines[line_index].text_start;
        let end_pos = lines
            .get(line_index + 1)
            .map(|next| next.text_start)
            .unwrap_or(u32::MAX);
        let start = para.char_offsets.partition_point(|pos| *pos < start_pos);
        let end = para.char_offsets.partition_point(|pos| *pos < end_pos);
        (start, end)
    };
    // 마지막 줄은 꽉 차지 않으므로 증인에서 뺀다.
    for line_index in 0..lines.len() - 1 {
        let line = lines[line_index];
        if line.segment_width <= 0 {
            continue;
        }
        let box_width = hwpunit_to_px(line.segment_width, dpi);
        let tolerance = box_width * FIT_TOLERANCE_RATIO;
        let (start, end) = line_range(line_index);
        if start >= end {
            continue;
        }
        // 줄 끝 공백은 한컴도 폭에 넣지 않는다.
        let mut ink_end = end;
        while ink_end > start && chars[ink_end - 1] == ' ' {
            ink_end -= 1;
        }
        let line_widths = &widths[start..ink_end];
        if !line_widths.iter().any(|(_, _, hft)| *hft) {
            continue;
        }
        let proportional: f64 = line_widths.iter().map(|w| w.0).sum();
        let halfwidth: f64 = line_widths.iter().map(|w| w.1).sum();
        if proportional > box_width + tolerance && halfwidth <= box_width - tolerance {
            witnesses.halfwidth += 1;
            continue;
        }
        // 다음 줄 첫 낱말이 반각으로는 들어가는데 끊었다 — 비례 조판의 증인.
        let (next_start, next_end) = line_range(line_index + 1);
        let word_end = (next_start..next_end)
            .find(|index| chars[*index] == ' ')
            .unwrap_or(next_end);
        if word_end <= next_start {
            continue;
        }
        let gap: f64 = widths[ink_end..end].iter().map(|w| w.1).sum();
        let next_word = &widths[next_start..word_end];
        let half_with_word = halfwidth + gap + next_word.iter().map(|w| w.1).sum::<f64>();
        let prop_with_word = proportional + gap + next_word.iter().map(|w| w.0).sum::<f64>();
        if half_with_word <= box_width - tolerance && prop_with_word > box_width + tolerance {
            witnesses.proportional += 1;
        }
    }
}
