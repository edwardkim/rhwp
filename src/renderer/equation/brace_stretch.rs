//! 공개 MATH variant/assembly로 늘림 중괄호의 공통 filled outline을 만든다.
//! 슬롯 폭·행 점유는 layout이 소유하며 출력 backend는 같은 명령만 소비한다.

use super::GlyphOutlineCommand as Command;
#[path = "brace_stretch_data.rs"]
mod data;

#[derive(Clone, Copy)]
struct Glyph {
    advance: f64,
    min_y: f64,
    max_y: f64,
    outline: &'static [Command],
}
#[derive(Clone, Copy)]
struct Part {
    glyph: Glyph,
    full: f64,
    start: f64,
    end: f64,
    extender: bool,
}
fn project(out: &mut Vec<Command>, glyph: Glyph, map: impl Fn((f64, f64)) -> (f64, f64)) {
    out.extend(glyph.outline.iter().map(|command| match *command {
        Command::Move(p) => Command::Move(map(p)),
        Command::Line(p) => Command::Line(map(p)),
        Command::Cubic(points) => Command::Cubic(points.map(&map)),
        Command::Close => Command::Close,
    }));
}

pub(crate) fn commands(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fs: f64,
    right: bool,
) -> Vec<Command> {
    commands_at_frame(x, y, width, height, fs, right, height)
}

/// 조립 부품의 글줄 프레임과 실제 잉크 끝을 구분한다.
/// 원 face의 cap 위끝 606·tail 아래끝 12 / UPM 1024와 독립 줄 프레임을 소비한다.
/// 한/두 행의 일반 glyph와 지원하지 않는 font frame은 기존 전체 높이를 보존한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn source_commands(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fs: f64,
    dpi: f64,
    right: bool,
) -> Vec<Command> {
    let assembly = height * 1000.0 / fs > data::VARIANTS.last().expect("공개 variant").0;
    if let Some((line_height, _)) = assembly
        .then(|| super::font_frame_metrics::line_at_dpi(fs, dpi))
        .flatten()
    {
        let ink_height = height - line_height + fs * (606.0 - 12.0) / 1024.0;
        let inset = (height - ink_height) / 2.0;
        return commands_at_frame(x, y + inset, width, ink_height, fs, right, height);
    }
    commands(x, y, width, height, fs, right)
}

#[allow(clippy::too_many_arguments)]
fn commands_at_frame(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fs: f64,
    right: bool,
    selection_height: f64,
) -> Vec<Command> {
    if fs <= 0.0 || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let target = height * 1000.0 / fs;
    let map_x = |gx: f64, advance: f64| {
        let offset = gx * width / advance;
        x + if right { width - offset } else { offset }
    };
    let mut out = Vec::new();
    let selection = selection_height * 1000.0 / fs;
    if let Some((_, glyph)) = data::VARIANTS.iter().find(|(size, _)| *size >= selection) {
        project(&mut out, *glyph, |(gx, gy)| {
            (
                map_x(gx, glyph.advance),
                y + (glyph.max_y - gy) * height / (glyph.max_y - glyph.min_y),
            )
        });
        return out;
    }
    // extender는 필요한 높이일 때만 넣는다. cap·middle·tail만으로 맞는
    // 원 face의 잉크 프레임도 공개 MATH의 connector 범위 안에서 연결한다.
    // 위·아래 extender의 수는 같아야 중괄호의 가운데 축을 보존한다.
    let mut repeats = 0;
    let parts = loop {
        let parts: Vec<_> = data::PARTS
            .iter()
            .flat_map(|part| std::iter::repeat_n(*part, if part.extender { repeats } else { 1 }))
            .collect();
        let maximum = parts.iter().map(|part| part.full).sum::<f64>()
            - data::MIN_OVERLAP * (parts.len() - 1) as f64;
        if maximum >= target {
            break parts;
        }
        repeats += 1;
    };
    let overlap =
        (parts.iter().map(|part| part.full).sum::<f64>() - target) / (parts.len() - 1) as f64;
    let mut bottom = 0.0;
    for (index, part) in parts.iter().enumerate() {
        project(&mut out, part.glyph, |(gx, gy)| {
            (
                map_x(gx, part.glyph.advance),
                y + height - (bottom + gy) * fs / 1000.0,
            )
        });
        if let Some(next) = parts.get(index + 1) {
            debug_assert!(overlap >= data::MIN_OVERLAP);
            debug_assert!(overlap <= part.end.min(next.start));
            bottom += part.full - overlap;
        }
    }
    out
}
