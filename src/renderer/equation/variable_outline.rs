//! 한컴 수학 변수의 공통 공개 윤곽과 독립 Print 표시 계약.
//! 공개 Latin Modern Math의 정체·이탤릭 윤곽만 사용한다. 사설 윤곽·바이트는 복제하지 않는다.
//! 파생 글꼴 출처와 GUST 라이선스는 assets/fonts의 해당 manifest를 따른다.

use std::sync::LazyLock;

use super::GlyphOutlineCommand as Command;

static FACE: LazyLock<ttf_parser::Face<'static>> = LazyLock::new(|| {
    ttf_parser::Face::parse(
        include_bytes!("../../../assets/fonts/LatinModernMath-VariableOutline-subset.otf"),
        0,
    )
    .expect("공개 변수 윤곽 subset은 유효한 OpenType 글꼴이다")
});

// UPM 1024, HYhwpEQ cmap U+E000..U+E019의 em 잉크 프레임.
// 설치 원본 SHA-256: a3a3cd992a89ac38e7707b58143ebdb1da9511d3c462f019cda7f92c30467807.
// 독립 Print의 해당 glyph와 메트릭·기울임을 대조한다. 화면 좌표가 아니다.
const TARGET: [[f64; 4]; 26] = [
    [32.0, 2.0, 734.0, 735.0],               // A
    [37.0, 2.0, 666.0, 702.0],               // B
    [57.0, -20.0, 681.0, 724.0],             // C
    [36.0, 2.0, 723.0, 702.0],               // D
    [34.0, 2.0, 668.0, 702.0],               // E
    [34.0, 2.0, 625.0, 702.0],               // F
    [57.0, -20.0, 753.0, 724.0],             // G
    [34.0, 2.0, 734.0, 702.0],               // H
    [29.0, 2.0, 341.0, 702.0],               // I
    [42.0, -20.0, 476.0, 702.0],             // J
    [34.0, 2.0, 754.0, 702.0],               // K
    [34.0, 2.0, 596.0, 702.0],               // L
    [38.0, 2.0, 899.0, 703.0],               // M
    [34.0, 0.304347826086957, 734.0, 702.0], // N
    [57.0, -20.0, 738.0, 724.0],             // O
    [36.0, 2.0, 639.0, 702.0],               // P
    [57.0, -197.0, 744.0, 724.0],            // Q
    [36.0, -20.0, 749.0, 702.0],             // R
    [56.0, -20.0, 511.0, 724.0],             // S
    [37.0, 2.0, 701.0, 702.0],               // T
    [34.0, -20.0, 734.0, 702.0],             // U
    [20.0, -20.0, 748.0, 702.0],             // V
    [19.0, -20.0, 1033.0, 702.0],            // W
    [25.0, 2.0, 743.0, 702.0],               // X
    [11.0, 2.0, 755.0, 702.0],               // Y
    [56.0, 2.0, 573.0, 702.0],               // Z
];

// 정체 A..Z/a..z와 이탤릭 a..z의 같은 UPM 잉크 메트릭.
const ROMAN: [[f64; 4]; 52] = [
    [37.0, -69.0, 731.0, 684.0],
    [38.0, -69.0, 623.0, 668.0],
    [57.0, -86.0, 630.0, 684.0],
    [38.0, -69.0, 667.0, 668.0],
    [38.0, -69.0, 625.0, 668.0],
    [38.0, -69.0, 600.0, 668.0],
    [57.0, -85.0, 687.0, 684.0],
    [43.0, -69.0, 724.0, 668.0],
    [53.0, -69.0, 329.0, 668.0],
    [42.0, -86.0, 430.0, 668.0],
    [42.0, -69.0, 723.0, 668.0],
    [38.0, -69.0, 601.0, 668.0],
    [41.0, -77.0, 896.0, 668.0],
    [38.0, -74.0, 729.0, 668.0],
    [56.0, -85.0, 667.0, 684.0],
    [38.0, -69.0, 584.0, 668.0],
    [53.0, -183.0, 699.0, 684.0],
    [38.0, -69.0, 648.0, 668.0],
    [56.0, -86.0, 568.0, 684.0],
    [55.0, -71.0, 714.0, 668.0],
    [38.0, -85.0, 729.0, 668.0],
    [30.0, -86.0, 695.0, 668.0],
    [30.0, -86.0, 951.0, 668.0],
    [31.0, -69.0, 650.0, 668.0],
    [31.0, -69.0, 651.0, 668.0],
    [48.0, -69.0, 575.0, 668.0],
    [63.0, -86.0, 492.0, 427.0],
    [32.0, -86.0, 489.0, 675.0],
    [63.0, -86.0, 459.0, 427.0],
    [63.0, -86.0, 521.0, 675.0],
    [63.0, -86.0, 489.0, 427.0],
    [61.0, -69.0, 379.0, 680.0],
    [45.0, -213.0, 527.0, 432.0],
    [36.0, -69.0, 518.0, 675.0],
    [46.0, -69.0, 247.0, 684.0],
    [1.0, -213.0, 209.0, 684.0],
    [34.0, -69.0, 531.0, 675.0],
    [45.0, -69.0, 250.0, 675.0],
    [40.0, -69.0, 813.0, 427.0],
    [36.0, -69.0, 517.0, 427.0],
    [61.0, -86.0, 494.0, 427.0],
    [34.0, -213.0, 490.0, 427.0],
    [63.0, -213.0, 520.0, 427.0],
    [32.0, -69.0, 387.0, 437.0],
    [68.0, -85.0, 443.0, 428.0],
    [34.0, -86.0, 343.0, 583.0],
    [36.0, -86.0, 517.0, 427.0],
    [31.0, -86.0, 523.0, 419.0],
    [32.0, -86.0, 777.0, 419.0],
    [32.0, -69.0, 564.0, 419.0],
    [41.0, -213.0, 551.0, 419.0],
    [41.0, -69.0, 435.0, 419.0],
];
const ITALIC_LOWER: [[f64; 4]; 26] = [
    [41.0, -9.0, 509.1666666666667, 454.0],
    [48.0, -9.0, 425.0, 713.5],
    [42.0, -9.0, 440.44444444444446, 454.0],
    [41.0, -9.0, 527.3333333333334, 713.1666666666666],
    [47.0, -9.0, 439.2307692307692, 454.0],
    [-5.148148148148148, -217.0, 575.1379310344828, 714.0],
    [15.0, -208.0, 484.5, 454.0],
    [57.0, -9.0, 558.5, 713.5],
    [29.666666666666668, -9.0, 300.0, 679.0],
    [-12.0, -208.0, 408.0, 679.0],
    [57.0, -9.0, 521.0, 713.0],
    [45.17142857142857, -9.0, 265.0, 713.0],
    [29.75, -9.0, 867.5, 454.0],
    [29.75, -9.0, 583.5, 454.0],
    [42.0, -9.0, 480.0, 454.0],
    [-31.0, -197.0, 503.0, 454.0],
    [41.0, -197.0, 463.0, 454.0],
    [30.0, -9.23076923076923, 447.0, 454.0],
    [53.0, -9.0, 430.0, 454.0],
    [24.0, -9.0, 338.0, 643.0],
    [29.666666666666668, -9.0, 555.0, 454.0],
    [29.666666666666668, -9.0, 479.0, 454.0],
    [29.666666666666668, -9.0, 707.0, 454.0],
    [30.0, -9.0, 540.0, 454.0],
    [29.666666666666668, -208.0, 502.4, 454.0],
    [44.0, -9.0, 478.1666666666667, 455.5],
];

struct Builder {
    commands: Vec<Command>,
    source: ttf_parser::Rect,
    target: [f64; 4],
    scale: f64,
    shear: f64,
    origin: f64,
    current: (f64, f64),
    contour_start: (f64, f64),
}

impl Builder {
    fn point(&self, x: f32, y: f32) -> (f64, f64) {
        let x = self.target[0]
            + (f64::from(x) - f64::from(self.source.x_min)) * (self.target[2] - self.target[0])
                / f64::from(self.source.x_max - self.source.x_min);
        let y = self.target[1]
            + (f64::from(y) - f64::from(self.source.y_min)) * (self.target[3] - self.target[1])
                / f64::from(self.source.y_max - self.source.y_min);
        // 대문자는 정체 윤곽에 Print의 기울임을 한 번 적용한다.
        // 실제 이탤릭 윤곽인 소문자에는 이 변환을 적용하지 않는다.
        (
            self.scale * (x + self.shear * y),
            self.origin - self.scale * y,
        )
    }
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current = self.point(x, y);
        self.contour_start = self.current;
        self.commands.push(Command::Move(self.current));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current = self.point(x, y);
        self.commands.push(Command::Line(self.current));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = self.point(x1, y1);
        let end = self.point(x, y);
        let first = (
            self.current.0 + (control.0 - self.current.0) * 2.0 / 3.0,
            self.current.1 + (control.1 - self.current.1) * 2.0 / 3.0,
        );
        let second = (
            end.0 + (control.0 - end.0) * 2.0 / 3.0,
            end.1 + (control.1 - end.1) * 2.0 / 3.0,
        );
        self.commands.push(Command::Cubic([first, second, end]));
        self.current = end;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let first = self.point(x1, y1);
        let second = self.point(x2, y2);
        let end = self.point(x, y);
        self.commands.push(Command::Cubic([first, second, end]));
        self.current = end;
    }

    fn close(&mut self) {
        self.commands.push(Command::Close);
        self.current = self.contour_start;
    }
}

pub(super) fn commands(
    ch: char,
    fs: f64,
    dpi: f64,
    baseline: f64,
    italic: bool,
) -> Option<Vec<Command>> {
    if !ch.is_ascii_alphabetic() {
        return None;
    }
    let upper = ch.is_ascii_uppercase();
    let index = (u32::from(ch) - u32::from(if upper { 'A' } else { 'a' })) as usize;
    let display = if italic && !upper {
        if ch == 'h' {
            '\u{210E}'
        } else {
            char::from_u32(0x1D44E + index as u32)?
        }
    } else {
        ch
    };
    let glyph = FACE.glyph_index(display)?;
    let source = FACE.glyph_bounding_box(glyph)?;
    if source.x_max <= source.x_min || source.y_max <= source.y_min {
        return None;
    }
    let target = match (italic, upper) {
        (true, true) => TARGET[index],
        (true, false) => ITALIC_LOWER[index],
        (false, true) => ROMAN[index],
        (false, false) => ROMAN[26 + index],
    };
    let roman_rise = if italic {
        0.0
    } else {
        // 정체 ASCII 프로그램의 기준선 차이. em 단위와 같은 Print 격자를 사용한다.
        (fs * 600.0 / dpi * 70.0 / 1024.0).floor() * dpi / 600.0
    };
    let mut builder = Builder {
        commands: Vec::new(),
        source,
        target,
        scale: fs / 1024.0,
        shear: if italic && upper { 0.333 } else { 0.0 },
        origin: baseline - super::negation_geometry::math_origin_shift(fs, dpi) - roman_rise,
        current: (0.0, 0.0),
        contour_start: (0.0, 0.0),
    };
    FACE.outline_glyph(glyph, &mut builder)?;
    (!builder.commands.is_empty()).then_some(builder.commands)
}
