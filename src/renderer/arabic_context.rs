//! 확인한 글꼴의 아랍어 결합형·합자 전진을 원문 scalar 주소에 투영한다.
//! 윤곽선 shaping은 각 backend가 맡고 측정·배치는 같은 전진 결과를 소비한다.
use super::{font_metrics_data, TextStyle};

#[path = "arabic_context/face_0.rs"]
mod face_0;
#[path = "arabic_context/face_1.rs"]
mod face_1;
#[path = "arabic_context/face_2.rs"]
mod face_2;
#[path = "arabic_context/face_3.rs"]
mod face_3;
#[path = "arabic_context/joining.rs"]
mod joining;

// 원문 codepoint, isolated/initial/medial/final glyph, hmtx, GDEF mark 비트.
type Row = (u32, [u16; 4], [u16; 4], u8);
// lookup, 입력 glyph들, 출력 glyph, hmtx, IgnoreMarks, 출력 GDEF mark 여부.
type Rule = (u16, &'static [u16], u16, u16, bool, bool);

struct Source {
    rows: &'static [Row],
    rules: &'static [Rule],
}

impl Source {
    fn row(&self, ch: char) -> Option<&'static Row> {
        self.rows
            .binary_search_by_key(&(ch as u32), |row| row.0)
            .ok()
            .map(|i| &self.rows[i])
    }
}

fn source(style: &TextStyle) -> Option<Source> {
    let family = style
        .metric_font_family
        .as_deref()
        .unwrap_or(&style.font_family);
    let primary = family.split(',').next()?.trim().trim_matches(['\'', '"']);
    if !matches!(
        primary,
        "HCR Batang" | "함초롬바탕" | "HCR Dotum" | "함초롬돋움"
    ) {
        return None;
    }
    let metric = font_metrics_data::find_metric(primary, style.bold, style.italic)?.metric;
    if metric.em_size != 1000 {
        return None;
    }
    let (rows, rules) = match (primary, metric.bold) {
        ("HCR Batang" | "함초롬바탕", false) => (face_0::ROWS, face_0::RULES),
        ("HCR Batang" | "함초롬바탕", true) => (face_1::ROWS, face_1::RULES),
        ("HCR Dotum" | "함초롬돋움", false) => (face_2::ROWS, face_2::RULES),
        ("HCR Dotum" | "함초롬돋움", true) => (face_3::ROWS, face_3::RULES),
        _ => return None,
    };
    Some(Source { rows, rules })
}

fn neighbor_kind(chars: &[char], index: usize, forward: bool, source: &Source) -> u8 {
    let mut at = index;
    loop {
        at = if forward {
            let Some(next) = at.checked_add(1).filter(|&i| i < chars.len()) else {
                return 0;
            };
            next
        } else {
            let Some(previous) = at.checked_sub(1) else {
                return 0;
            };
            previous
        };
        let ch = chars[at];
        let kind = joining::kind(ch);
        if kind == 4 {
            continue;
        }
        // ZWJ는 자체 glyph 없이 결합을 요청한다. 다른 face의 글자에 걸쳐 합치지 않는다.
        return if ch == '\u{200d}' || source.row(ch).is_some() {
            kind
        } else {
            0
        };
    }
}

struct Glyph {
    id: Option<u16>,
    width: f64,
    mark: bool,
    owners: Vec<(usize, f64)>,
}

fn apply_lookup(glyphs: &mut Vec<Glyph>, rules: &[Rule]) {
    let mut index = 0;
    while index < glyphs.len() {
        for &(_, inputs, output, width, ignore_marks, mark) in rules {
            if glyphs[index].id != inputs.first().copied() {
                continue;
            }
            let mut matched = vec![index];
            let mut at = index + 1;
            for input in &inputs[1..] {
                while ignore_marks && glyphs.get(at).is_some_and(|glyph| glyph.mark) {
                    at += 1;
                }
                if glyphs.get(at).and_then(|glyph| glyph.id) != Some(*input) {
                    break;
                }
                matched.push(at);
                at += 1;
            }
            if matched.len() != inputs.len() {
                continue;
            }
            let mut owners = Vec::new();
            for &at in &matched {
                owners.extend_from_slice(&glyphs[at].owners);
            }
            glyphs[index] = Glyph {
                id: Some(output),
                width: f64::from(width),
                mark,
                owners,
            };
            for &at in matched.iter().skip(1).rev() {
                glyphs.remove(at);
            }
            break;
        }
        index += 1;
    }
}

/// 반환 단위는 em이다. None인 원문 글자는 기존 측정 계약을 유지한다.
pub(crate) fn advances(chars: &[char], style: &TextStyle) -> Option<Vec<Option<f64>>> {
    if !chars.iter().any(|ch| matches!(*ch as u32, 0x0600..=0x08ff)) {
        return None;
    }
    let source = source(style)?;
    let mut glyphs = Vec::new();
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '\u{200d}' {
            continue;
        }
        let Some(row) = source.row(ch) else {
            glyphs.push(Glyph {
                id: None,
                width: 0.0,
                mark: joining::kind(ch) == 4,
                owners: Vec::new(),
            });
            continue;
        };
        let kind = joining::kind(ch);
        let previous = kind & 1 != 0 && neighbor_kind(chars, index, false, &source) & 2 != 0;
        let next = kind & 2 != 0 && neighbor_kind(chars, index, true, &source) & 1 != 0;
        let form = match (previous, next) {
            (false, false) => 0,
            (false, true) => 1,
            (true, true) => 2,
            (true, false) => 3,
        };
        let width = f64::from(row.2[form]);
        glyphs.push(Glyph {
            id: Some(row.1[form]),
            width,
            mark: row.3 & (1 << form) != 0,
            owners: vec![(index, width)],
        });
    }
    // GSUB의 rlig → liga 순서. 같은 lookup의 규칙은 한 번의 순회로 적용한다.
    let mut start = 0;
    while start < source.rules.len() {
        let end =
            start + source.rules[start..].partition_point(|rule| rule.0 == source.rules[start].0);
        apply_lookup(&mut glyphs, &source.rules[start..end]);
        start = end;
    }
    let mut advances = vec![None; chars.len()];
    for glyph in glyphs {
        let sum: f64 = glyph.owners.iter().map(|(_, width)| *width).sum();
        for &(owner, original_width) in &glyph.owners {
            // 합자의 scalar 수와 순서는 보존하며 총 전진을 원래 기여 비율로 배분한다.
            let share = if sum > 0.0 {
                original_width / sum
            } else {
                1.0 / glyph.owners.len() as f64
            };
            advances[owner] = Some(glyph.width * share / 1000.0);
        }
    }
    Some(advances)
}
