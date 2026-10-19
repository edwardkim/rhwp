//! 독립 한컴 Print에서 확인한 그리스 기호의 의미·face·전진을 함께 전달한다.
//! rm/it/bold는 라틴 변수 스타일이며 그리스 기호의 표시 glyph를 바꾸지 않는다.
//! 논리 메트릭은 설치 HYhwpEQ의 UPM 1024 hmtx, glyph 대응은 v8–v10 Print다.
//! 사설 글꼴 바이트·윤곽은 포함하지 않고 공개 수학 glyph 또는 설치 대체 face를 쓴다.

use super::{source_font_text, source_roman_advance, LayoutBox, LayoutKind};

pub(super) fn command(text: &str, fs: f64, dpi: f64) -> Option<LayoutBox> {
    let mut chars = text.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if ch == 'ϱ' {
        return fallback(ch, fs, dpi);
    }
    greek(ch, fs, dpi)
}

pub(super) fn literal(text: &str, fs: f64, dpi: f64) -> Option<LayoutBox> {
    if text.is_empty() {
        return None;
    }
    let mut children = Vec::new();
    let mut x = 0.0;
    for ch in text.chars() {
        // 직접 입력한 변형 Unicode는 명령 glyph와 다른 Print 대체 face다.
        let mut child = fallback(ch, fs, dpi).or_else(|| greek(ch, fs, dpi))?;
        child.x = x;
        x += child.width;
        children.push(child);
    }
    if children.len() == 1 {
        return children.pop();
    }
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: x,
        height: fs,
        baseline: fs * 0.8,
        kind: LayoutKind::Row(children),
    })
}

fn greek(ch: char, fs: f64, dpi: f64) -> Option<LayoutBox> {
    let (units, display) = match ch {
        'α' => (655, '\u{1D6FC}'),
        'β' => (579, '\u{1D6FD}'),
        'γ' => (529, '\u{1D6FE}'),
        'δ' => (455, '\u{1D6FF}'),
        'ε' => (415, '\u{1D716}'),
        'ζ' => (447, '\u{1D701}'),
        'η' => (508, '\u{1D702}'),
        'θ' => (480, '\u{1D703}'),
        'ι' => (362, '\u{1D704}'),
        'κ' => (590, '\u{1D705}'),
        'λ' => (597, '\u{1D706}'),
        'μ' => (616, '\u{1D707}'),
        'ν' => (505, '\u{1D708}'),
        'ξ' => (447, '\u{1D709}'),
        'ο' => (496, '\u{1D70A}'),
        'π' => (584, '\u{1D70B}'),
        'ρ' => (529, '\u{1D70C}'),
        'σ' => (585, '\u{1D70E}'),
        'τ' => (447, '\u{1D70F}'),
        'υ' => (553, '\u{1D710}'),
        'φ' => (609, '\u{1D719}'),
        'χ' => (640, '\u{1D712}'),
        'ψ' => (666, '\u{1D713}'),
        'ω' => (628, '\u{1D714}'),
        'Α' => (768, '\u{1D6E2}'),
        'Β' => (777, '\u{1D6E3}'),
        'Γ' => (630, '\u{1D6E4}'),
        'Δ' => (853, '\u{1D6E5}'),
        'Ε' => (756, '\u{1D6E6}'),
        'Ζ' => (698, '\u{1D6E7}'),
        'Η' => (851, '\u{1D6E8}'),
        'Θ' => (780, '\u{1D6E9}'),
        'Ι' => (450, '\u{1D6EA}'),
        'Κ' => (870, '\u{1D6EB}'),
        'Λ' => (711, '\u{1D6EC}'),
        'Μ' => (993, '\u{1D6ED}'),
        'Ν' => (823, '\u{1D6EE}'),
        'Ξ' => (759, '\u{1D6EF}'),
        'Ο' => (780, '\u{1D6F0}'),
        'Π' => (851, '\u{1D6F1}'),
        'Ρ' => (657, '\u{1D6F2}'),
        'Σ' => (798, '\u{1D6F4}'),
        'Τ' => (598, '\u{1D6F5}'),
        'Υ' => (594, '\u{1D6F6}'),
        'Φ' => (682, '\u{1D6F7}'),
        'Χ' => (848, '\u{1D6F8}'),
        'Ψ' => (626, '\u{1D6F9}'),
        'Ω' => (790, '\u{1D6FA}'),
        'ϕ' => (670, '\u{1D711}'),
        'ϵ' => (461, '\u{1D700}'),
        'ϑ' => (587, '\u{1D717}'),
        'ϖ' => (848, '\u{1D71B}'),
        'ς' => (370, '\u{1D70D}'),
        'ϒ' => (635, '\u{3D2}'),
        _ => return None,
    };
    let em = source_roman_advance::em_advance(fs, dpi).unwrap_or(fs);
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: source_font_text::logical_advance(units, em, dpi),
        height: fs,
        baseline: fs * 0.8,
        kind: if display == 'ϒ' {
            LayoutKind::MathSymbol(display.to_string())
        } else {
            LayoutKind::Text(display.to_string())
        },
    })
}

fn fallback(ch: char, fs: f64, dpi: f64) -> Option<LayoutBox> {
    // 독립 Print와 설치 Haansoft Batang의 UPM 1024 hmtx를 대조했다.
    let units = match ch {
        'ϱ' => 528,
        'ϕ' => 554,
        'ϵ' => 461,
        'ϰ' => 645,
        'ϑ' => 619,
        _ => return None,
    };
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: source_font_text::logical_advance(units, fs, dpi),
        height: fs,
        baseline: fs * 0.8,
        kind: LayoutKind::FontGlyph {
            text: ch.to_string(),
            font_family: "Haansoft Batang, 한컴바탕, Batang, 바탕, serif",
            outline: None,
            paint: None,
        },
    })
}
