//! 큰 둥근 괄호의 cap·직선 extender를 공개 MATH 부품으로 조립한다.
//! 논리 슬롯과 본체 높이는 layout의 것이며 SVG·Canvas에 같은 filled outline을 준다.

use super::GlyphOutlineCommand as Command;
#[path = "round_stretch_data.rs"]
mod data;

#[derive(Clone, Copy)]
struct Part {
    outline: &'static [Command],
    full: f64,
    start: f64,
    end: f64,
    extender: bool,
}

pub(super) fn source_commands(height: f64, fs: f64, dpi: f64, right: bool) -> Option<Vec<Command>> {
    if !height.is_finite()
        || !fs.is_finite()
        || fs <= 0.0
        || height * 1000.0 / fs <= data::MAX_VARIANT
    {
        return None;
    }
    let (line_height, _) = super::font_frame_metrics::line_at_dpi(fs, dpi)?;
    // HYhwpEQ UPM 1024의 cap(E07C)·tail(E07E) bbox 메트릭이다.
    // 사설 윤곽은 사용하지 않고 공개 cap 디자인을 이 잉크 프레임에 맞춘다.
    // 글줄 상자에서 cap 위끝 623과 tail 아래끝 5 사이의 여백만 제외한다.
    let ink_height = height - line_height + fs * (623.0 - 5.0) / 1024.0;
    let inset = (height - ink_height) / 2.0;
    let scale_y = fs * 826.0 / (1024.0 * 1495.0);
    let target = ink_height / scale_y;
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
    let mut commands = Vec::new();
    let mut bottom = 0.0;
    for (index, part) in parts.iter().enumerate() {
        let map = |(gx, gy): (f64, f64)| {
            // 직선 bbox 277..379를 원 face의 38..87(49/1024em)에 대응한다.
            // cap의 나머지 곡선은 원래 bbox 끝 443까지 보존한다.
            // 오른쪽 cap bbox -41..365는 같은 디자인을 402 축으로 반전한 것이다.
            let ink_x = if gx <= 379.0 {
                38.0 + (gx - 277.0) * 49.0 / (379.0 - 277.0)
            } else {
                87.0 + (gx - 379.0) * (443.0 - 87.0) / (823.0 - 379.0)
            };
            let x = fs * if right { 402.0 - ink_x } else { ink_x } / 1024.0;
            (x, inset + ink_height - (bottom + gy) * scale_y)
        };
        commands.extend(part.outline.iter().map(|command| match *command {
            Command::Move(p) => Command::Move(map(p)),
            Command::Line(p) => Command::Line(map(p)),
            Command::Cubic(points) => Command::Cubic(points.map(&map)),
            Command::Close => Command::Close,
        }));
        if let Some(next) = parts.get(index + 1) {
            debug_assert!(overlap >= data::MIN_OVERLAP);
            debug_assert!(overlap <= part.end.min(next.start));
            bottom += part.full - overlap;
        }
    }
    Some(commands)
}
