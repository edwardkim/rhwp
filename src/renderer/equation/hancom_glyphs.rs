//! 한컴 수식의 실제 face·글리프·논리 전진을 모든 출력 경로에 함께 전달한다.
//! HYhwpEQ의 cmap/hmtx는 설치 HYHWPEQ.TTF(UPM 1024)에서 확인했다.
//! HancomEQN은 별도 GDI 메트릭을 사용한다. 두 글꼴을 서로 치환하지 않는다.

pub const FONT_FAMILY: &str = "HyhwpEQ";

#[derive(Debug, Clone, serde::Serialize)]
pub struct GlyphRun {
    pub font_family: &'static str,
    pub glyphs: String,
    pub positions: Vec<f64>,
    pub shears: Vec<f64>,
    pub width: f64,
    pub baseline_shift: f64,
}

#[derive(Clone, Copy)]
pub(super) enum Role {
    Variable { italic: bool },
    Number,
    Operator,
    Greek,
    BigOperator,
    LegacyLiteral,
}

pub(super) fn is_operator(text: &str) -> bool {
    matches!(
        text,
        "+" | "-" | "=" | ":" | "<" | ">" | "×" | "÷" | "≤" | "≥"
    )
}

const ITALIC_LOWER: [u16; 26] = [
    541, 439, 442, 532, 476, 553, 487, 590, 352, 421, 532, 305, 899, 614, 496, 515, 457, 462, 480,
    369, 585, 496, 732, 585, 502, 476,
];
const ITALIC_UPPER: [u16; 26] = [
    768, 725, 739, 782, 696, 667, 803, 768, 369, 526, 795, 640, 938, 768, 795, 696, 795, 753, 568,
    739, 768, 768, 1051, 768, 768, 625,
];
const ROMAN_LOWER: [u16; 26] = [
    512, 554, 512, 554, 554, 384, 554, 554, 298, 298, 554, 298, 853, 554, 554, 554, 554, 426, 512,
    384, 554, 554, 810, 597, 597, 469,
];
const ROMAN_UPPER: [u16; 26] = [
    768, 682, 682, 725, 682, 640, 725, 768, 384, 469, 768, 640, 938, 768, 725, 640, 725, 682, 640,
    768, 768, 725, 981, 682, 682, 640,
];
const NUMBERS: [u16; 10] = [512, 512, 512, 512, 512, 512, 512, 512, 512, 512];

fn glyph(ch: char, role: Role) -> Option<(char, u16)> {
    let (code, advance) = match role {
        Role::Variable { italic } if ch.is_ascii_lowercase() => {
            let index = ch as usize - 'a' as usize;
            if italic {
                (0xe0e5 + index as u32, ITALIC_LOWER[index])
            } else {
                (ch as u32, ROMAN_LOWER[index])
            }
        }
        Role::Variable { italic } if ch.is_ascii_uppercase() => {
            let index = ch as usize - 'A' as usize;
            if italic {
                (0xe000 + index as u32, ITALIC_UPPER[index])
            } else {
                (ch as u32, ROMAN_UPPER[index])
            }
        }
        Role::Number if ch.is_ascii_digit() => {
            let index = ch as usize - '0' as usize;
            let code = if index == 0 {
                0xe03d
            } else {
                0xe033 + index as u32
            };
            (code, NUMBERS[index])
        }
        Role::Operator => match ch {
            '+' => (0xe048, 795),
            '-' => (0xe046, 774),
            '=' => (0xe047, 795),
            ':' => (0xe04f, 283),
            '<' => (0xe055, 864),
            '>' => (0xe056, 864),
            '×' => (0xd7, 831),
            '÷' => (0xf7, 831),
            '≤' => (0x2264, 1024),
            '≥' => (0x2265, 1024),
            // 독립 두 face·네 크기 Print의 논리 기호는 각 원본 face의 1em이다.
            // 이항 연산자로 분류하거나 명시적 ~의 여백을 여기서 더하지 않는다.
            '∴' => (0x2234, 1024),
            '∵' => (0x2235, 1024),
            ',' => (0xe052, 283),
            // 독립 Print의 일반 괄호는 ASCII glyph가 아닌 수학 PUA다.
            '(' => (0xe044, 398),
            ')' => (0xe045, 398),
            _ => return None,
        },
        Role::Greek => {
            if let Some(index) = "αβγδεζηθικλμνξοπρστυφχψω".chars().position(|c| c == ch)
            {
                (0xe09d + index as u32, 0)
            } else {
                let index = "ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ".chars().position(|c| c == ch)?;
                (0xe085 + index as u32, 0)
            }
        }
        Role::BigOperator if ch == '∑' => (0xe067, 739),
        // A legacy equation's literal triangle and one-dot leader occupy
        // full symbol slots in Hancom Print. The equation font has no U+2024
        // cmap entry; its mathematical dot is U+22C5, with that same advance.
        // Keep this projection in the shared glyph result, not in a painter.
        Role::LegacyLiteral => match ch {
            '△' => (0x25b3, 1024),
            '․' => (0x22c5, 1024),
            _ => return None,
        },
        _ => return None,
    };
    Some((char::from_u32(code)?, advance))
}

pub(super) fn measure(
    text: &str,
    role: Role,
    fs: f64,
    dpi: f64,
    pad: f64,
    font_family: &'static str,
) -> Option<GlyphRun> {
    if font_family != "HancomEQN" && matches!(role, Role::Greek | Role::BigOperator) {
        return None;
    }
    if text.is_empty() || !fs.is_finite() || fs <= 0.0 || !dpi.is_finite() || dpi <= 0.0 {
        return None;
    }
    let logical_size = (fs * 48.0 / dpi).round().max(1.0) * 2.0;
    let mut glyphs = String::new();
    let mut positions = Vec::new();
    let mut shears = Vec::new();
    let mut width = pad;
    for ch in text.chars() {
        let (rendered, advance) = glyph(ch, role)?;
        positions.push(width);
        // 독립 Print는 정체 PUA 대문자에 이 행렬을 적용한다.
        // 소문자 PUA는 글리프 자체에 기울임 윤곽이 있다.
        shears.push(
            if matches!(role, Role::Variable { italic: true }) && ch.is_ascii_uppercase() {
                0.333
            } else {
                0.0
            },
        );
        glyphs.push(rendered);
        width += if font_family == "HancomEQN" {
            super::hancom_eqn_metrics::advance(rendered, fs, dpi)?
        } else if matches!(role, Role::Variable { italic: false }) {
            // 같은 face의 기존 독립 GDI 전진을 실제 glyph 원점에도 소비한다.
            // hmtx의 단순 반올림은 힌팅된 정체 D 등의 다음 글자를 앞당긴다.
            super::layout::measured_roman_advance(&ch.to_string(), fs, dpi).unwrap_or_else(|| {
                (logical_size * f64::from(advance) / 1024.0).round() * dpi / 96.0
            })
        } else {
            (logical_size * f64::from(advance) / 1024.0).round() * dpi / 96.0
        };
        if matches!(role, Role::Operator) && ch == ',' {
            // 독립 Print의 네 크기에서 확인한 수학 쉼표의 오른쪽 간격.
            // glyph 자체의 GDI 전진과 구분하며 쉼표의 표시 원점은 보존한다.
            width += fs * 0.07;
        }
    }
    Some(GlyphRun {
        font_family,
        glyphs,
        positions,
        shears,
        width: width + pad,
        // 정체·ASCII 연산자의 glyph 기준선과 수식 기준선을 구분한다.
        // 저장 컨트롤의 기준선 이동은 개체 전체에 별도로 적용한다.
        baseline_shift: if matches!(role, Role::Variable { italic: false })
            || matches!(role, Role::LegacyLiteral)
            || matches!(role, Role::Operator) && text.is_ascii()
        {
            fs * if font_family == "HancomEQN" {
                -0.05
            } else {
                820.0 / 1024.0 - 0.85
            }
        } else {
            0.0
        },
    })
}

/// 독립 확인한 선형 글리프 줄은 자연 크기로 표시한다.
pub(crate) fn is_linear(layout: &super::layout::LayoutBox) -> bool {
    use super::layout::LayoutKind;
    match &layout.kind {
        LayoutKind::HancomGlyphs(_) => true,
        LayoutKind::Row(children) => !children.is_empty() && children.iter().all(is_linear),
        LayoutKind::FontStyle {
            style: super::symbols::FontStyleKind::Roman | super::symbols::FontStyleKind::Italic,
            body,
        } => is_linear(body),
        _ => false,
    }
}

pub(crate) fn is_used(layout: &super::layout::LayoutBox) -> bool {
    !used_font_families(layout).is_empty()
}

pub(crate) fn used_font_families(
    layout: &super::layout::LayoutBox,
) -> std::collections::BTreeSet<&'static str> {
    use super::layout::LayoutKind;
    let mut pending = vec![layout];
    let mut families = std::collections::BTreeSet::new();
    while let Some(item) = pending.pop() {
        match &item.kind {
            LayoutKind::HancomGlyphs(run) => {
                families.insert(run.font_family);
            }
            LayoutKind::Row(children) => pending.extend(children),
            LayoutKind::Fraction { numer, denom, .. } => {
                pending.push(numer);
                pending.push(denom);
            }
            LayoutKind::Atop { top, bottom } => {
                pending.push(top);
                pending.push(bottom);
            }
            LayoutKind::Sqrt { index, body } => {
                pending.extend(index.as_deref());
                pending.push(body);
            }
            LayoutKind::Superscript { base, sup } => {
                pending.push(base);
                pending.push(sup);
            }
            LayoutKind::Subscript { base, sub } => {
                pending.push(base);
                pending.push(sub);
            }
            LayoutKind::SubSup { base, sub, sup } => {
                pending.push(base);
                pending.push(sub);
                pending.push(sup);
            }
            LayoutKind::BigOp { sub, sup, .. } => {
                pending.extend(sub.as_deref());
                pending.extend(sup.as_deref());
            }
            LayoutKind::Limit { name, sub, .. } => {
                pending.extend(name.as_deref());
                pending.extend(sub.as_deref());
            }
            LayoutKind::Matrix { cells, .. } => pending.extend(cells.iter().flatten()),
            LayoutKind::Rel { arrow, over, under } => {
                pending.push(arrow);
                pending.push(over);
                pending.extend(under.as_deref());
            }
            LayoutKind::EqAlign { rows } => {
                for (left, right) in rows {
                    pending.push(left);
                    pending.push(right);
                }
            }
            LayoutKind::Paren { body, .. }
            | LayoutKind::Decoration { body, .. }
            | LayoutKind::FontStyle { body, .. } => pending.push(body),
            _ => {}
        }
    }
    families
}
