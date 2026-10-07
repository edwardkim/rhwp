//! 한컴 classic 폼의 공통 글자 배치와 입체 외형.
//!
//! 기준은 정상 form-01의 한컴 PDF다. 저장 10pt 글자, 2px 명암 프레임,
//! 솟은 버튼/들어간 입력 프레임을 유지하고 backend별 여백 추측을 제거한다.
use serde::Serialize;

use super::form_caption::display_form_caption;
use super::layout::{estimate_text_width_exact, resolved_to_text_style};
use super::render_tree::{BoundingBox, FormObjectNode};
use super::style_resolver::ResolvedStyleSet;
use crate::model::control::{FormObject, FormType};

#[derive(Debug, Clone, Serialize)]
pub struct FormAppearance {
    pub font_family: String,
    pub font_size: f64,
    pub bold: bool,
    pub italic: bool,
    pub label_width: f64,
    pub draw_frame: bool,
    /// 96dpi classic UI pixel을 문서 출력 해상도로 변환한다.
    pub pixel_scale: f64,
}

impl Default for FormAppearance {
    fn default() -> Self {
        Self {
            font_family: "sans-serif".into(),
            font_size: 40.0 / 3.0,
            bold: false,
            italic: false,
            label_width: 0.0,
            draw_frame: true,
            pixel_scale: 1.0,
        }
    }
}

impl FormAppearance {
    pub(crate) fn resolve(
        form: &FormObject,
        styles: &ResolvedStyleSet,
        context_id: u32,
        dpi: f64,
    ) -> Self {
        let follow_context = form
            .properties
            .get("FollowContext")
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        let id = if follow_context {
            context_id
        } else {
            form.properties
                .get("CharShapeID")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        };
        let style = resolved_to_text_style(styles, id, 0);
        let display = FormObjectNode::form_display_text(form);
        let caption = display_form_caption(&form.caption);
        let label = match form.form_type {
            FormType::PushButton | FormType::CheckBox | FormType::RadioButton => caption.as_ref(),
            _ => display.as_deref().unwrap_or(&form.text),
        };
        Self {
            label_width: estimate_text_width_exact(label, &style),
            font_family: style.font_family,
            font_size: style.font_size,
            bold: style.bold,
            italic: style.italic,
            draw_frame: !form
                .properties
                .get("DrawFrame")
                .is_some_and(|v| v == "0" || v.eq_ignore_ascii_case("false")),
            pixel_scale: dpi / 96.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FormPrimitive {
    Rect {
        bbox: BoundingBox,
        color: String,
    },
    Circle {
        x: f64,
        y: f64,
        radius: f64,
        color: String,
    },
    Polyline {
        points: Vec<[f64; 2]>,
        color: String,
        width: f64,
        closed: bool,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormLabel {
    pub text: String,
    pub x: f64,
    /// 세 backend와 CanvasKit에서 그대로 사용하는 절대 기준선.
    pub baseline: f64,
    pub font_family: String,
    pub font_size: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FormDrawing {
    pub primitives: Vec<FormPrimitive>,
    pub label: Option<FormLabel>,
}

fn rect(out: &mut Vec<FormPrimitive>, b: BoundingBox, color: &str) {
    if b.width > 0.0 && b.height > 0.0 {
        out.push(FormPrimitive::Rect {
            bbox: b,
            color: color.into(),
        });
    }
}

/// 두 겹 명암: 솟은 프레임의 좌상 밝음/우하 어두움, 들어간 프레임은 그 반대.
fn bevel(out: &mut Vec<FormPrimitive>, b: BoundingBox, s: f64, raised: bool) {
    let colors = if raised {
        ["#fafafa", "#404040", "#f6f6f6", "#a0a0a0"]
    } else {
        ["#a0a0a0", "#fafafa", "#404040", "#f6f6f6"]
    };
    for i in 0..2 {
        let d = i as f64 * s;
        let x = b.x + d;
        let y = b.y + d;
        let w = b.width - 2.0 * d;
        let h = b.height - 2.0 * d;
        rect(out, BoundingBox::new(x, y, w - s, s), colors[i * 2]);
        rect(out, BoundingBox::new(x, y, s, h - s), colors[i * 2]);
        rect(out, BoundingBox::new(x + w - s, y, s, h), colors[i * 2 + 1]);
        rect(out, BoundingBox::new(x, y + h - s, w, s), colors[i * 2 + 1]);
    }
}

fn arc(out: &mut Vec<FormPrimitive>, x: f64, y: f64, r: f64, start: f64, color: &str, width: f64) {
    let points = (0..=20)
        .map(|i| {
            let angle = start + std::f64::consts::PI * i as f64 / 20.0;
            [x + r * angle.cos(), y + r * angle.sin()]
        })
        .collect();
    out.push(FormPrimitive::Polyline {
        points,
        color: color.into(),
        width,
        closed: false,
    });
}

/// 고정 bbox 안에서 UI 외형과 label 원점을 함께 만든다. 모델 선택값/본문 흐름은 바꾸지 않는다.
pub fn form_drawing(form: &FormObjectNode, b: BoundingBox) -> FormDrawing {
    let mut out = Vec::new();
    if b.width <= 0.0 || b.height <= 0.0 {
        return FormDrawing {
            primitives: out,
            label: None,
        };
    }
    let a = &form.appearance;
    let s = a.pixel_scale;
    let cy = b.y + b.height / 2.0;
    let mut tx = b.x + 3.0 * s;
    let caption = display_form_caption(&form.caption);
    let text = match form.form_type {
        FormType::PushButton | FormType::CheckBox | FormType::RadioButton => caption.as_ref(),
        _ => form.display_or_text(),
    };
    match form.form_type {
        FormType::PushButton => {
            rect(&mut out, b, &form.back_color);
            if a.draw_frame {
                bevel(&mut out, b, s, true);
            }
            tx = b.x + (b.width - a.label_width) / 2.0;
        }
        FormType::Edit | FormType::ComboBox => {
            rect(&mut out, b, &form.back_color);
            if a.draw_frame {
                bevel(&mut out, b, s, false);
            }
            if form.form_type == FormType::ComboBox {
                // classic dropdown 버튼은 프레임 안쪽에 놓이며 17 UI pixels 폭이다.
                let bw = (17.0 * s).min((b.width - 4.0 * s).max(0.0));
                let button = BoundingBox::new(
                    b.x + b.width - 2.0 * s - bw,
                    b.y + 2.0 * s,
                    bw,
                    b.height - 4.0 * s,
                );
                rect(&mut out, button, &form.back_color);
                if a.draw_frame {
                    bevel(&mut out, button, s, true);
                }
                let cx = button.x + button.width / 2.0;
                out.push(FormPrimitive::Polyline {
                    points: vec![[cx - 2.0 * s, cy - s], [cx + 2.0 * s, cy - s], [cx, cy + s]],
                    color: "#000000".into(),
                    width: 0.0,
                    closed: true,
                });
            }
        }
        FormType::CheckBox => {
            let size = (13.0 * s).min(b.height).min(b.width);
            let icon = BoundingBox::new(b.x, cy - size / 2.0, size, size);
            rect(&mut out, icon, "#ffffff");
            if a.draw_frame {
                bevel(&mut out, icon, s, false);
            }
            if form.value != 0 {
                out.push(FormPrimitive::Polyline {
                    points: vec![
                        [icon.x + 3.0 * s, cy],
                        [icon.x + 5.0 * s, cy + 2.0 * s],
                        [icon.x + 10.0 * s, cy - 3.0 * s],
                    ],
                    color: form.fore_color.clone(),
                    width: 2.0 * s,
                    closed: false,
                });
            }
            tx = b.x + (40.0 / 3.0 + 4.0) * s;
        }
        FormType::RadioButton => {
            let r = (5.0 * s).min(b.height / 2.0).min(b.width / 2.0);
            let cx = b.x + r;
            out.push(FormPrimitive::Circle {
                x: cx,
                y: cy,
                radius: r,
                color: "#ffffff".into(),
            });
            if a.draw_frame {
                arc(
                    &mut out,
                    cx,
                    cy,
                    r - s / 2.0,
                    -std::f64::consts::FRAC_PI_4,
                    "#fafafa",
                    s,
                );
                arc(
                    &mut out,
                    cx,
                    cy,
                    r - s / 2.0,
                    3.0 * std::f64::consts::FRAC_PI_4,
                    "#a0a0a0",
                    s,
                );
                arc(
                    &mut out,
                    cx,
                    cy,
                    r - 1.5 * s,
                    -std::f64::consts::FRAC_PI_4,
                    "#f6f6f6",
                    s,
                );
                arc(
                    &mut out,
                    cx,
                    cy,
                    r - 1.5 * s,
                    3.0 * std::f64::consts::FRAC_PI_4,
                    "#404040",
                    s,
                );
            }
            if form.value != 0 {
                out.push(FormPrimitive::Circle {
                    x: cx,
                    y: cy,
                    radius: r / 2.0,
                    color: form.fore_color.clone(),
                });
            }
            tx = b.x + (40.0 / 3.0 + 4.0) * s;
        }
    }
    let label = (!text.is_empty()).then(|| FormLabel {
        text: text.into(),
        x: tx,
        // classic single-line control의 글자 상자를 세로 중앙에 놓는다.
        // 독립 PDF Batang 10pt 기준선은 중심 + 0.35em이다.
        baseline: cy + a.font_size * 0.35,
        font_family: a.font_family.clone(),
        font_size: a.font_size,
        bold: a.bold,
        italic: a.italic,
        color: if form.enabled {
            form.fore_color.clone()
        } else {
            "#a0a0a0".into()
        },
    });
    FormDrawing {
        primitives: out,
        label,
    }
}
