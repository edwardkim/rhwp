//! 내부 사진의 source 배치와 최초 JPEG 자원을 최종 장치 표본에 연결한다.

use super::{catalog::Catalogue, frame::Frame, xml};
use crate::renderer::print_raster::PrintRasterFrame;

pub(super) const ATTRIBUTE: &str = "data-rhwp-print-wmf-image";

pub(super) struct Marker {
    pub(super) frame: PrintRasterFrame,
    pub(super) flip_x: bool,
    pub(super) flip_y: bool,
}

impl Marker {
    fn encode(&self) -> String {
        std::iter::once(1.0)
            .chain(self.frame.values())
            .chain([
                f64::from(u8::from(self.flip_x)),
                f64::from(u8::from(self.flip_y)),
            ])
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn read(value: &str) -> Option<Self> {
        let values = xml::numbers::<11>(value)?;
        if values[0] != 1.0 || ![0.0, 1.0].contains(&values[9]) || ![0.0, 1.0].contains(&values[10])
        {
            return None;
        }
        Some(Self {
            frame: PrintRasterFrame::from_values(values[1..9].try_into().ok()?)?,
            flip_x: values[9] == 1.0,
            flip_y: values[10] == 1.0,
        })
    }
}

fn coordinate(node: roxmltree::Node<'_, '_>, name: &str) -> Option<f64> {
    node.attribute(name)
        .map_or(Some(0.0), |value| Some(xml::numbers::<1>(value)?[0]))
}

pub(super) fn placement(
    node: roxmltree::Node<'_, '_>,
    image: roxmltree::Node<'_, '_>,
) -> Option<([f64; 4], bool, bool)> {
    // 상위 클립은 원본 SVG 그룹이 계속 소유한다. 클립은 비트맵의 배치와
    // 최초 JPEG 전달 격자를 바꾸지 않으며, 재생도 image/use만 교체하므로
    // 같은 그룹의 클립이 최종 장치 그림에 적용된다. 상위 변환이나 그림
    // 자체의 클립은 아직 이 좌표 계약에서 해소하지 않는다.
    if node
        .ancestors()
        .skip(1)
        .any(|parent| parent.has_attribute("transform"))
        || node.has_attribute("clip-path")
        || image.has_attribute("clip-path")
        || (node != image && image.has_attribute("transform"))
    {
        return None;
    }
    let (x, y) = if node == image {
        (coordinate(image, "x")?, coordinate(image, "y")?)
    } else {
        (
            coordinate(node, "x")? + coordinate(image, "x")?,
            coordinate(node, "y")? + coordinate(image, "y")?,
        )
    };
    let (w, h) = (xml::number(image, "width")?, xml::number(image, "height")?);
    let transform = node
        .attribute("transform")
        .map(str::parse::<svgtypes::Transform>)
        .transpose()
        .ok()?
        .unwrap_or_default();
    if transform.b != 0.0 || transform.c != 0.0 || transform.a == 0.0 || transform.d == 0.0 {
        return None;
    }
    let l = transform.a * x + transform.e;
    let t = transform.d * y + transform.f;
    let r = l + transform.a * w;
    let b = t + transform.d * h;
    let bounds = [l.min(r), t.min(b), (r - l).abs(), (b - t).abs()];
    (bounds.iter().all(|v| v.is_finite()) && bounds[2] > 0.0 && bounds[3] > 0.0).then_some((
        bounds,
        transform.a < 0.0,
        transform.d < 0.0,
    ))
}

pub(super) fn prepare(
    svg: &[u8],
    frame: Frame,
    viewport: (f64, f64),
    catalogue: &Catalogue,
) -> Option<Vec<u8>> {
    let source = std::str::from_utf8(svg).ok()?;
    let document = roxmltree::Document::parse(source).ok()?;
    let mut edits = Vec::new();
    let mut changed = std::collections::HashSet::new();
    for (node, image) in super::catalog::image_uses(&document)? {
        let Some(("image/png", bytes)) = xml::href(image).and_then(xml::image_bytes) else {
            continue;
        };
        if super::catalog::raster_identity(&bytes).is_none() {
            continue;
        }
        let jpeg = catalogue.jpeg(&bytes)?;
        let (rect, flip_x, flip_y) = placement(node, image)?;
        let marker = Marker {
            frame: frame.nested_raster(rect, viewport)?,
            flip_x,
            flip_y,
        };
        edits.extend(xml::attributes(
            source,
            node,
            &[(ATTRIBUTE, marker.encode())],
        )?);
        if changed.insert(image.range().start) {
            edits.extend(xml::attributes(
                source,
                image,
                &[("href", xml::uri("image/jpeg", &jpeg)?)],
            )?);
        }
    }
    Some(xml::apply(source, edits)?.into_bytes())
}
