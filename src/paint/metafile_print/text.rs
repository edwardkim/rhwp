//! 공급한 실제 SFNT의 윤곽과 GDI 전진을 source 좌표에 준비한다.

use resvg::usvg;

use super::{frame::Frame, metrics::FontMetrics, xml};
use crate::paint::metafile_fonts::{family, requests, MetafileFonts};

pub(super) const ATTRIBUTE: &str = "data-rhwp-print-wmf-text";

pub(super) struct Marker {
    pub(super) pdf: [f64; 4],
    pub(super) window: [f64; 8],
    pub(super) em: f64,
    pub(super) synthetic_bold: bool,
}

impl Marker {
    fn encode(&self) -> String {
        std::iter::once(1.0)
            .chain(self.pdf)
            .chain(self.window)
            .chain([self.em, f64::from(u8::from(self.synthetic_bold))])
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(any(feature = "native-skia", target_arch = "wasm32"))]
    pub(super) fn read(value: &str) -> Option<Self> {
        let values = xml::numbers::<15>(value)?;
        if values[0] != 1.0 || values[13] <= 0.0 || ![0.0, 1.0].contains(&values[14]) {
            return None;
        }
        Some(Self {
            pdf: values[1..5].try_into().ok()?,
            window: values[5..13].try_into().ok()?,
            em: values[13],
            synthetic_bold: values[14] == 1.0,
        })
    }
}

fn contents(node: roxmltree::Node<'_, '_>) -> String {
    node.descendants()
        .filter(|node| node.is_text())
        .filter_map(|node| node.text())
        .collect()
}

pub(super) fn prepare(
    svg: &[u8],
    frame: Frame,
    preceding_polygon: bool,
    fonts: &MetafileFonts,
) -> Option<Vec<u8>> {
    let source = std::str::from_utf8(svg).ok()?;
    let xml = roxmltree::Document::parse(source).ok()?;
    let database = fonts.database();
    let mut edits = Vec::new();
    let mut markers = Vec::new();
    for node in xml.descendants().filter(|node| node.has_tag_name("text")) {
        // 원본의 회전·글자 clip은 기존 SVG가 소유한다. 제거해 출력하지 않는다.
        if node.ancestors().any(|parent| {
            ["transform", "clip-path", "mask", "opacity"]
                .iter()
                .any(|name| parent.has_attribute(*name))
        }) {
            return None;
        }
        let id = node.attribute("id")?.to_owned();
        let window = xml::numbers::<8>(node.attribute("data-rhwp-wmf-window")?)?;
        let source_font = xml::numbers::<6>(node.attribute("data-rhwp-wmf-font")?)?;
        let em = xml::number(node, "font-size")?;
        let compact =
            preceding_polygon || node.attribute("data-rhwp-wmf-polygon-seen") == Some("1");
        let pdf = frame.text_matrix(window, compact)?;
        let family_names = svgtypes::parse_font_families(node.attribute("font-family")?).ok()?;
        let names = family_names
            .into_iter()
            .map(|name| match name {
                svgtypes::FontFamily::Named(value) => value,
                other => other.to_string(),
            })
            .collect::<Vec<_>>();
        let families = names.iter().map(|name| family(name)).collect::<Vec<_>>();
        let requested_weight = if source_font[4] == 0.0 {
            400
        } else {
            source_font[4] as u16
        };
        let face_id = database.query(&usvg::fontdb::Query {
            families: &families,
            weight: usvg::fontdb::Weight(requested_weight),
            style: if source_font[5] == 0.0 {
                usvg::fontdb::Style::Normal
            } else {
                usvg::fontdb::Style::Italic
            },
            ..Default::default()
        })?;
        let spans = node
            .children()
            .filter(|child| child.has_tag_name("tspan"))
            .collect::<Vec<_>>();
        let positions = spans
            .iter()
            .map(|span| xml::number(*span, "x"))
            .collect::<Option<Vec<_>>>();
        let selected = database.with_face_data(face_id, |bytes, index| {
            let face = ttf_parser::Face::parse(bytes, index).ok()?;
            // writer가 별도 defs에 보관하는 색 글꼴·가변 윤곽은 아직 이
            // 부분 추출 계약이 아니다. 다른 face의 누락 glyph도 혼합하지 않는다.
            if face.is_variable()
                || face.tables().glyf.is_none()
                || [*b"COLR", *b"SVG ", *b"CBDT", *b"sbix"].iter().any(|tag| {
                    face.raw_face()
                        .table(ttf_parser::Tag::from_bytes(tag))
                        .is_some()
                })
                || contents(node)
                    .chars()
                    .filter(|ch| !ch.is_control() && !ch.is_whitespace())
                    .any(|ch| !face.glyph_index(ch).is_some_and(|glyph| glyph.0 != 0))
            {
                return None;
            }
            let synthetic_bold = requested_weight >= 600 && face.weight().to_number() < 600;
            let advances = (|| {
                if node.attribute("data-rhwp-wmf-text-align") != Some("start auto 0") {
                    return None;
                }
                let metrics = FontMetrics::new(
                    bytes,
                    index,
                    source_font,
                    frame.device_scale((window[2], window[3]))?,
                )?;
                let positions = positions.as_ref()?;
                spans
                    .iter()
                    .zip(positions.windows(2))
                    .map(|(span, pair)| metrics.advance(&contents(*span), pair[1] - pair[0]))
                    .collect::<Option<Vec<_>>>()
            })();
            Some((advances, synthetic_bold))
        })??;
        if let (Some(advances), Some(positions)) = (selected.0, positions) {
            if let Some(first) = positions.first() {
                let mut current = *first;
                for (index, span) in spans.iter().enumerate() {
                    edits.extend(xml::attributes(
                        source,
                        *span,
                        &[("x", current.to_string())],
                    )?);
                    current += advances.get(index).copied().unwrap_or(0.0);
                }
            }
        }
        if selected.1 {
            edits.extend(xml::attributes(
                source,
                node,
                &[
                    (
                        "stroke",
                        node.attribute("fill").unwrap_or("black").to_owned(),
                    ),
                    ("stroke-width", (em * 0.02).to_string()),
                    ("stroke-linejoin", "round".into()),
                ],
            )?);
        }
        markers.push((
            id,
            node.range(),
            contents(node),
            Marker {
                pdf,
                window,
                em,
                synthetic_bold: selected.1,
            },
        ));
    }
    if markers.is_empty() {
        return Some(svg.to_vec());
    }
    let measured = xml::apply(source, edits)?;
    // 누락 glyph를 완료된 윤곽으로 캐시하지 않는다. Native의 시스템 font도
    // 실제 선택한 바이트로 coverage를 확인한 뒤 같은 database를 사용한다.
    fonts.coverage_fingerprint(&requests(measured.as_bytes())?)?;
    let options = usvg::Options {
        fontdb: database,
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(&measured, &options).ok()?;
    let written = tree.to_string(&usvg::WriteOptions {
        preserve_text: false,
        coordinates_precision: 12,
        transforms_precision: 12,
        ..Default::default()
    });
    if written.len() > xml::MAX_SVG_BYTES {
        return None;
    }
    let outlined = roxmltree::Document::parse(&written).ok()?;
    let groups = outlined
        .descendants()
        .filter_map(|node| Some((node.attribute("id")?, node)))
        .collect::<std::collections::HashMap<_, _>>();
    let mut replacements = Vec::new();
    for (id, range, text, marker) in markers {
        let body = if let Some(group) = groups.get(id.as_str()) {
            written.get(group.range())?
        } else if text.trim().is_empty() {
            "<g/>"
        } else {
            return None;
        };
        // 그림·mask defs는 원래 SVG를 보존한다. writer의 전체 문서를 다시
        // 사용하면 원본 pattern 격자와 공유 image/use의 프레임이 사라진다.
        replacements.push((
            range,
            format!("<g {ATTRIBUTE}=\"{}\">{body}</g>", marker.encode()),
        ));
    }
    Some(xml::apply(source, replacements)?.into_bytes())
}
