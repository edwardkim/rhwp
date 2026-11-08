//! Native 문자 장식과 탭 리더가 공유하는 선 그리기.
use skia_safe::{paint, Canvas, Color, Paint, PathEffect};
pub(super) struct TextLines<'a> {
    pub canvas: &'a Canvas,
}
impl TextLines<'_> {
    pub fn top_underline(
        &self,
        x1: f32,
        baseline: f32,
        x2: f32,
        color: Color,
        geometry: &crate::renderer::text_decoration::TopUnderlineGeometry,
        phase_delta: f64,
    ) {
        for stroke in &geometry.strokes {
            let dash: Vec<f32> = stroke.dash.iter().map(|x| *x as f32).collect();
            self.styled_with_phase(
                x1,
                baseline + stroke.offset as f32,
                x2,
                color,
                stroke.width as f32,
                &dash,
                false,
                geometry.phase as f32 + phase_delta as f32,
            );
        }
        if let Some(dot) = &geometry.dots {
            self.styled_with_phase(
                x1,
                baseline + dot.offset as f32,
                x2,
                color,
                (dot.radius * 2.0) as f32,
                &[0.0, dot.step as f32],
                true,
                geometry.phase as f32 + phase_delta as f32,
            );
        }
    }

    pub fn styled(
        &self,
        x1: f32,
        y: f32,
        x2: f32,
        color: Color,
        width: f32,
        dash: &[f32],
        round: bool,
    ) {
        self.styled_with_phase(x1, y, x2, color, width, dash, round, 0.0);
    }
    #[allow(clippy::too_many_arguments)]
    fn styled_with_phase(
        &self,
        x1: f32,
        y: f32,
        x2: f32,
        color: Color,
        width: f32,
        dash: &[f32],
        round: bool,
        phase: f32,
    ) {
        if x2 <= x1 {
            return;
        }
        let mut line_paint = Paint::default();
        line_paint.set_anti_alias(true);
        line_paint.set_style(paint::Style::Stroke);
        line_paint.set_stroke_width(width);
        line_paint.set_color(color);
        if round {
            line_paint.set_stroke_cap(paint::Cap::Round);
        }
        if !dash.is_empty() {
            if let Some(effect) = PathEffect::dash(dash, phase) {
                line_paint.set_path_effect(effect);
            }
        }
        self.canvas.draw_line((x1, y), (x2, y), &line_paint);
    }
    pub fn shape(&self, x1: f32, y: f32, x2: f32, color: Color, shape: u8) {
        match shape {
            7 => {
                self.styled(x1, y - 1.0, x2, color, 0.7, &[], false);
                self.styled(x1, y + 1.0, x2, color, 0.7, &[], false);
            }
            8 => {
                self.styled(x1, y - 1.2, x2, color, 0.5, &[], false);
                self.styled(x1, y + 0.8, x2, color, 1.2, &[], false);
            }
            9 => {
                self.styled(x1, y - 0.8, x2, color, 1.2, &[], false);
                self.styled(x1, y + 1.2, x2, color, 0.5, &[], false);
            }
            10 => {
                self.styled(x1, y - 1.5, x2, color, 0.5, &[], false);
                self.styled(x1, y, x2, color, 0.5, &[], false);
                self.styled(x1, y + 1.5, x2, color, 0.5, &[], false);
            }
            11 | 12 => {
                if ![x1, y, x2].iter().all(|value| value.is_finite()) || x2 <= x1 {
                    return;
                }
                if x2 - x1 > 6.0 * 4096.0 {
                    self.styled(x1, y, x2, color, 1.0, &[], false);
                    return;
                }
                let offsets: &[f32] = if shape == 12 { &[-1.0, 1.0] } else { &[0.0] };
                for offset in offsets {
                    let mut path = skia_safe::PathBuilder::new();
                    path.move_to((x1, y + offset));
                    let segments = ((x2 - x1) / 6.0).ceil() as usize;
                    for index in 0..segments {
                        let left = x1 + index as f32 * 6.0;
                        let right = (left + 6.0).min(x2);
                        let control_y = y + offset + if index % 2 == 0 { -1.5 } else { 1.5 };
                        path.quad_to(((left + right) / 2.0, control_y), (right, y + offset));
                    }
                    let mut wave = Paint::default();
                    wave.set_anti_alias(true);
                    wave.set_style(paint::Style::Stroke);
                    wave.set_stroke_width(if shape == 12 { 0.5 } else { 0.7 });
                    wave.set_color(color);
                    self.canvas.draw_path(&path.detach(), &wave);
                }
            }
            1 => self.styled(x1, y, x2, color, 1.0, &[3.0, 3.0], false),
            2 => self.styled(x1, y, x2, color, 1.0, &[1.0, 2.0], false),
            3 => self.styled(x1, y, x2, color, 1.0, &[6.0, 2.0, 1.0, 2.0], false),
            4 => self.styled(
                x1,
                y,
                x2,
                color,
                1.0,
                &[6.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                false,
            ),
            5 => self.styled(x1, y, x2, color, 1.0, &[8.0, 4.0], false),
            6 => self.styled(x1, y, x2, color, 1.0, &[0.1, 2.5], true),
            _ => self.styled(x1, y, x2, color, 1.0, &[], false),
        }
    }
}
