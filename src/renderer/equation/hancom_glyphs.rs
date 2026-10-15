//! HYhwpEQ Latin glyph selection and logical advance shared by all painters.
//! The cmap and hmtx values below come from the installed HYHWPEQ.TTF (1024 UPEM).
//! Independent Hancom Print probes confirm the italic PUA/roman cmap distinction.
//! Latin advance rounds the 48 dpi logical size first, then the glyph advance.

pub const FONT_FAMILY: &str = "HyhwpEQ";

#[derive(Debug, Clone, serde::Serialize)]
pub struct GlyphRun {
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
}

pub(super) fn is_operator(text: &str) -> bool {
    matches!(text, "+" | "-" | "=" | "<" | ">" | "×" | "÷" | "≤" | "≥")
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
            '<' => (0xe055, 864),
            '>' => (0xe056, 864),
            '×' => (0xd7, 831),
            '÷' => (0xf7, 831),
            '≤' => (0x2264, 1024),
            '≥' => (0x2265, 1024),
            _ => return None,
        },
        _ => return None,
    };
    Some((char::from_u32(code)?, advance))
}

pub(super) fn measure(text: &str, role: Role, fs: f64, dpi: f64, pad: f64) -> Option<GlyphRun> {
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
        // Independent Print uses this text matrix for upright PUA capitals.
        // Lowercase PUA glyphs already contain their italic outline.
        shears.push(
            if matches!(role, Role::Variable { italic: true }) && ch.is_ascii_uppercase() {
                0.333
            } else {
                0.0
            },
        );
        glyphs.push(rendered);
        width += (logical_size * f64::from(advance) / 1024.0).round() * dpi / 96.0;
    }
    Some(GlyphRun {
        glyphs,
        positions,
        shears,
        width: width + pad,
        // Roman glyphs use the font ascent, rather than the formula baseline.
        // Moving the stored control baseline moves the entire object separately.
        baseline_shift: if matches!(role, Role::Variable { italic: false })
            || matches!(role, Role::Operator) && text.is_ascii()
        {
            fs * (820.0 / 1024.0 - 0.85)
        } else {
            0.0
        },
    })
}

/// Only the independently verified linear Latin path has natural glyph scale.
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
    use super::layout::LayoutKind;
    let mut pending = vec![layout];
    while let Some(item) = pending.pop() {
        match &item.kind {
            LayoutKind::HancomGlyphs(_) => return true,
            LayoutKind::Row(children) => pending.extend(children),
            LayoutKind::Fraction { numer, denom } => {
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
            LayoutKind::Limit { sub, .. } => pending.extend(sub.as_deref()),
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
    false
}
