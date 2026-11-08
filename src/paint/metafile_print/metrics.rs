//! 실제 SFNT 평균 폭으로 실현한 GDI 고정 pitch와 원본 Dx를 연결한다.

use ttf_parser::Face;

pub(super) struct FontMetrics<'a> {
    face: Face<'a>,
    em: f64,
    average: f64,
    pixel_average: f64,
    sx: f64,
    sy: f64,
    synthetic_bold: bool,
}

impl<'a> FontMetrics<'a> {
    pub(super) fn new(
        bytes: &'a [u8],
        index: u32,
        source_font: [f64; 6],
        scale: (f64, f64),
    ) -> Option<Self> {
        let face = Face::parse(bytes, index).ok()?;
        // 명시 폭·회전·italic·가변 face의 실현을 같은 규칙으로 추측하지 않는다.
        if source_font[0] >= 0.0
            || source_font[1..4] != [0.0; 3]
            || source_font[5] != 0.0
            || !face.is_monospaced()
            || face.is_variable()
            || face.tables().glyf.is_none()
            || scale.0 <= 0.0
            || scale.1 <= 0.0
        {
            return None;
        }
        // OS/2의 xAvgCharWidth는 version 다음 signed int16이다.
        // 프로그램 바이트를 직접 읽으며 family 이름별 평균 폭 표를 만들지 않는다.
        let os2 = face
            .raw_face()
            .table(ttf_parser::Tag::from_bytes(b"OS/2"))?;
        let average = f64::from(i16::from_be_bytes(os2.get(2..4)?.try_into().ok()?));
        let em = -source_font[0];
        let pixel_average = (em * scale.0 * average / f64::from(face.units_per_em())).round();
        if average <= 0.0 || pixel_average <= 0.0 {
            return None;
        }
        let synthetic_bold = source_font[4] >= 600.0 && face.weight().to_number() < 600;
        Some(Self {
            face,
            em,
            average,
            pixel_average,
            sx: scale.0,
            sy: scale.1,
            synthetic_bold,
        })
    }

    pub(super) fn advance(&self, text: &str, dx: f64) -> Option<f64> {
        let mut chars = text.chars();
        let glyph = self.face.glyph_index(chars.next()?)?;
        if chars.next().is_some() {
            return None;
        }
        let source_width = f64::from(self.face.glyph_hor_advance(glyph)?);
        if source_width <= 0.0 || !dx.is_finite() {
            return None;
        }
        let physical_width = (self.pixel_average * source_width / self.average).round()
            + f64::from(u8::from(self.synthetic_bold));
        let gdi_width = (physical_width / self.sx).round();
        let intrinsic = (1000.0 * source_width / f64::from(self.face.units_per_em())).round();
        if dx == gdi_width {
            return Some(self.em * intrinsic / 1000.0);
        }
        let normalized = (gdi_width * self.sx / self.sy * 1000.0 / self.em).round();
        // PDF의 %.1f 직렬화와 같은 최근접 짝수 반올림이다. 원본 Dx의
        // 정수 곱셈을 먼저 수행해 1/4 경계에 나눗셈 오차를 더하지 않는다.
        let adjustment = ((normalized - dx * 1000.0 / self.em) * 10.0).round_ties_even() / 10.0;
        Some(self.em * (intrinsic - adjustment) / 1000.0)
    }
}
