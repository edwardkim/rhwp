//! 장치 펜의 경로를 출력 좌표로 옮겨 논리 좌표 확대와 선 폭을 분리한다.

use std::fmt::Write;

pub(super) type Matrix = [f64; 6];
pub(super) const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// b를 먼저, a를 나중에 적용한다. SVG의 열벡터 좌표계다.
pub(super) fn compose(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn inverse(m: Matrix) -> Option<Matrix> {
    let det = m[0] * m[3] - m[1] * m[2];
    if !m.iter().all(|v| v.is_finite()) || !det.is_finite() || det == 0.0 {
        return None;
    }
    let inv = [
        m[3] / det,
        -m[1] / det,
        -m[2] / det,
        m[0] / det,
        (m[2] * m[5] - m[3] * m[4]) / det,
        (m[1] * m[4] - m[0] * m[5]) / det,
    ];
    inv.iter().all(|v| v.is_finite()).then_some(inv)
}

fn number(node: roxmltree::Node<'_, '_>, name: &str, default: f64) -> Option<f64> {
    node.attribute(name)
        .map_or(Some(default), |s| s.parse().ok())
}

/// 이 변환기가 생성한 SVG 도형을 절대 경로로 바꾼다.
fn path_data(node: roxmltree::Node<'_, '_>) -> Option<String> {
    let n = |name, default| number(node, name, default);
    let path = match node.tag_name().name() {
        "path" => node.attribute("d")?.to_string(),
        "line" => format!(
            "M{} {} L{} {}",
            n("x1", 0.0)?,
            n("y1", 0.0)?,
            n("x2", 0.0)?,
            n("y2", 0.0)?
        ),
        "polyline" | "polygon" => {
            let points = node.attribute("points")?;
            format!(
                "M{points}{}",
                if node.has_tag_name("polygon") {
                    " Z"
                } else {
                    ""
                }
            )
        }
        "ellipse" => {
            let (cx, cy, rx, ry) = (n("cx", 0.0)?, n("cy", 0.0)?, n("rx", 0.0)?, n("ry", 0.0)?);
            format!(
                "M{} {cy} A{rx} {ry} 0 1 0 {} {cy} A{rx} {ry} 0 1 0 {} {cy} Z",
                cx + rx,
                cx - rx,
                cx + rx
            )
        }
        "rect" => {
            let (x, y, w, h) = (
                n("x", 0.0)?,
                n("y", 0.0)?,
                n("width", 0.0)?,
                n("height", 0.0)?,
            );
            if w < 0.0 || h < 0.0 {
                return None;
            }
            let (right, bottom) = (x + w, y + h);
            let mut rx = n("rx", n("ry", 0.0)?)?.max(0.0).min(w / 2.0);
            let mut ry = n("ry", n("rx", 0.0)?)?.max(0.0).min(h / 2.0);
            if rx == 0.0 || ry == 0.0 {
                rx = 0.0;
                ry = 0.0;
            }
            if rx == 0.0 {
                format!("M{x} {y} L{right} {y} L{right} {bottom} L{x} {bottom} Z")
            } else {
                format!("M{} {y} L{} {y} A{rx} {ry} 0 0 1 {right} {} L{right} {} A{rx} {ry} 0 0 1 {} {bottom} L{} {bottom} A{rx} {ry} 0 0 1 {x} {} L{x} {} A{rx} {ry} 0 0 1 {} {y} Z", x+rx, right-rx, y+ry, bottom-ry, right-rx, x+rx, bottom-ry, y+ry, x+rx)
            }
        }
        _ => return None,
    };
    Some(path)
}

/// 바깥 root/page/world 변환을 경로 점에 반영하고 선 폭은 출력 단위로 남긴다.
/// 역변환 그룹은 바깥 변환만 상쇄한다. 기존 clip은 같은 바깥 좌표계를 유지한다.
pub(super) fn normalize(node: &str, matrix: Matrix, width: f64) -> Option<String> {
    let inv = inverse(matrix)?;
    let xml = roxmltree::Document::parse(node).ok()?;
    let element = xml.root_element();
    let raw = path_data(element)?;
    let point = |x: f64, y: f64| -> Option<(f64, f64)> {
        let (x, y) = (
            matrix[0] * x + matrix[2] * y + matrix[4],
            matrix[1] * x + matrix[3] * y + matrix[5],
        );
        (x.is_finite() && y.is_finite()).then_some((x, y))
    };
    let mut d = String::with_capacity(raw.len());
    for segment in svgtypes::SimplifyingPathParser::from(raw.as_str()) {
        use svgtypes::SimplePathSegment as S;
        match segment.ok()? {
            S::MoveTo { x, y } => {
                let (x, y) = point(x, y)?;
                write!(d, "M{x:.9} {y:.9} ").ok()?;
            }
            S::LineTo { x, y } => {
                let (x, y) = point(x, y)?;
                write!(d, "L{x:.9} {y:.9} ").ok()?;
            }
            S::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let (x1, y1) = point(x1, y1)?;
                let (x2, y2) = point(x2, y2)?;
                let (x, y) = point(x, y)?;
                write!(d, "C{x1:.9} {y1:.9} {x2:.9} {y2:.9} {x:.9} {y:.9} ").ok()?;
            }
            S::Quadratic { x1, y1, x, y } => {
                let (x1, y1) = point(x1, y1)?;
                let (x, y) = point(x, y)?;
                write!(d, "Q{x1:.9} {y1:.9} {x:.9} {y:.9} ").ok()?;
            }
            S::ClosePath => d.push_str("Z "),
        }
    }
    let mut out = format!("<g transform=\"matrix({:.9} {:.9} {:.9} {:.9} {:.9} {:.9})\"><path d=\"{}\" data-rhwp-emf-device-stroke=\"1\" stroke-width=\"{width:.9}\"", inv[0],inv[1],inv[2],inv[3],inv[4],inv[5],d.trim());
    for attribute in element.attributes() {
        if matches!(
            attribute.name(),
            "fill"
                | "fill-rule"
                | "stroke"
                | "stroke-linecap"
                | "stroke-linejoin"
                | "stroke-miterlimit"
                | "opacity"
        ) {
            write!(
                out,
                " {}=\"{}\"",
                attribute.name(),
                super::svg::escape_xml(attribute.value())
            )
            .ok()?;
        }
    }
    out.push_str("/></g>");
    Some(out)
}
