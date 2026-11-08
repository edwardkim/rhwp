//! 문서 source 순서에서 중첩 그림의 최초 전달 품질과 선행 그리기 상태를 확정한다.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::renderer::image_resolver::{decode_image_with_format_limited, encode_print_jpeg};
use crate::wmf::converter::{SVGPlayer, WMFConverter};

use super::xml::{self, MAX_SVG_BYTES};

const MAX_SOURCES: usize = 512;
const MAX_USES: usize = 4096;
const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;

pub(crate) struct SourceUse {
    pub(crate) data: Arc<[u8]>,
    pub(crate) order_known: bool,
    /// 검증 가능한 원본 배치 폭(96dpi). 상대 크기나 미해석 그룹은 None이다.
    pub(crate) width: Option<f64>,
    pub(crate) crop: Option<(i32, i32, i32, i32)>,
    pub(crate) crop_reference: Option<(u32, u32)>,
}

pub(super) struct Source {
    pub(super) svg: Arc<Vec<u8>>,
    /// 같은 source를 선행 상태가 다른 곳에 쓰면 하나의 상태로 추측하지 않는다.
    pub(super) preceding_polygon: Option<bool>,
}

pub(crate) struct Catalogue {
    sources: HashMap<[u8; 32], Source>,
    qualities: HashMap<[u8; 32], Option<u8>>,
    bytes: usize,
    pub(super) fingerprint: [u8; 32],
}

thread_local! {
    static CACHE: std::cell::RefCell<VecDeque<([u8; 32], Arc<Catalogue>)>> =
        const { std::cell::RefCell::new(VecDeque::new()) };
}

/// 원본의 불투명 고색수 자원만 사진 전달 대상이다. 투명·저색수는 기존 PNG다.
pub(super) fn raster_identity(data: &[u8]) -> Option<([u8; 32], u32)> {
    let header = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())?;
    let decoded = decode_image_with_format_limited(data, image::ImageFormat::Png)?.to_rgba8();
    if decoded.dimensions() != (header.width, header.height) {
        return None;
    }
    let mut colors = HashSet::with_capacity(32);
    let mut hash = blake3::Hasher::new();
    hash.update(&header.width.to_le_bytes());
    hash.update(&header.height.to_le_bytes());
    for pixel in decoded.pixels() {
        if pixel[3] != 255 {
            return None;
        }
        if colors.len() < 32 {
            colors.insert([pixel[0], pixel[1], pixel[2]]);
        }
    }
    // alpha는 위에서 모두 255임을 확인했다. 같은 RGB의 opaque RGBA를
    // 한 번에 해시해 작은 channel 조각마다 hasher를 호출하지 않는다.
    hash.update(decoded.as_raw());
    (colors.len() >= 32).then_some((*hash.finalize().as_bytes(), header.width))
}

pub(super) fn image_uses<'a, 'input>(
    document: &'a roxmltree::Document<'input>,
) -> Option<Vec<(roxmltree::Node<'a, 'input>, roxmltree::Node<'a, 'input>)>> {
    let root = document.root_element();
    let definitions = root
        .descendants()
        .filter(|node| node.has_tag_name("image"))
        .filter_map(|node| Some((node.attribute("id")?, node)))
        .collect::<HashMap<_, _>>();
    let mut result = Vec::new();
    for node in root.descendants().filter(|node| {
        node.is_element()
            && !node.has_attribute("data-rhwp-wmf-mask-grid")
            && !node
                .ancestors()
                .any(|parent| parent.has_tag_name("defs") || parent.has_attribute("mask"))
    }) {
        let image = if node.has_tag_name("image") {
            Some(node)
        } else if node.has_tag_name("use") {
            xml::href(node)
                .and_then(|value| value.strip_prefix('#'))
                .and_then(|name| definitions.get(name).copied())
        } else {
            None
        };
        if let Some(image) = image {
            if result.len() >= MAX_USES {
                return None;
            }
            result.push((node, image));
        }
    }
    Some(result)
}

impl Catalogue {
    pub(crate) fn from_uses(uses: &[SourceUse]) -> Option<Arc<Self>> {
        if uses.len() > MAX_USES {
            return None;
        }
        let mut hash = blake3::Hasher::new();
        for usage in uses {
            hash.update(blake3::hash(&usage.data).as_bytes());
            hash.update(&[u8::from(usage.order_known)]);
            hash.update(&usage.width.unwrap_or(f64::NAN).to_bits().to_le_bytes());
            if let Some(value) = usage.crop.map(|(l, t, r, b)| [l, t, r, b]) {
                for item in value {
                    hash.update(&item.to_le_bytes());
                }
            }
            hash.update(&[u8::from(usage.crop.is_some())]);
            if let Some((width, height)) = usage.crop_reference {
                hash.update(&width.to_le_bytes());
                hash.update(&height.to_le_bytes());
            }
            hash.update(&[u8::from(usage.crop_reference.is_some())]);
        }
        let key = *hash.finalize().as_bytes();
        if let Some(hit) = CACHE.with(|cache| {
            cache
                .borrow()
                .iter()
                .find(|(value, _)| *value == key)
                .map(|(_, catalogue)| catalogue.clone())
        }) {
            return Some(hit);
        }
        let mut catalogue = Self::build(uses)?;
        catalogue.fingerprint = key;
        let catalogue = Arc::new(catalogue);
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            while cache.len() >= 4
                || cache.iter().map(|(_, item)| item.bytes).sum::<usize>() + catalogue.bytes
                    > MAX_CACHE_BYTES
            {
                if cache.pop_front().is_none() {
                    break;
                }
            }
            if catalogue.bytes <= MAX_CACHE_BYTES {
                cache.push_back((key, catalogue.clone()));
            }
        });
        Some(catalogue)
    }

    fn build(uses: &[SourceUse]) -> Option<Self> {
        let mut result = Self {
            sources: HashMap::new(),
            qualities: HashMap::new(),
            bytes: 0,
            fingerprint: [0; 32],
        };
        let mut preceding_polygon = Some(false);
        let mut known_resources = true;
        for usage in uses {
            let key = *blake3::hash(&usage.data).as_bytes();
            if !usage.order_known {
                if let Some(source) = result.sources.get_mut(&key) {
                    source.preceding_polygon = None;
                }
                preceding_polygon = None;
                known_resources = false;
                continue;
            }
            let svg = if let Some(source) = result.sources.get_mut(&key) {
                if source.preceding_polygon != preceding_polygon {
                    source.preceding_polygon = None;
                }
                source.svg.clone()
            } else {
                if result.sources.len() >= MAX_SOURCES {
                    return None;
                }
                // enhanced payload는 기존 EMF 소비자가 소유한다. 실패한 source의
                // 선행 상태나 이미지 목록을 성공한 것처럼 건너뛰지 않는다.
                let converted =
                    crate::parser::ole_container::emf_from_wmf_comment_chunks(&usage.data)
                        .is_none()
                        .then(|| {
                            WMFConverter::new(
                                usage.data.as_ref(),
                                SVGPlayer::new().with_source_contract(),
                            )
                            .run()
                            .ok()
                        })
                        .flatten();
                let Some(svg) = converted.filter(|svg| svg.len() <= MAX_SVG_BYTES) else {
                    preceding_polygon = None;
                    known_resources = false;
                    continue;
                };
                result.bytes = result.bytes.checked_add(svg.len())?;
                if result.bytes > MAX_CACHE_BYTES {
                    return None;
                }
                let svg = Arc::new(svg);
                result.sources.insert(
                    key,
                    Source {
                        svg: svg.clone(),
                        preceding_polygon,
                    },
                );
                svg
            };
            let xml = roxmltree::Document::parse(std::str::from_utf8(&svg).ok()?).ok()?;
            let root = xml.root_element();
            let width = xml::number(root, "width")?;
            let height = xml::number(root, "height")?;
            if width <= 0.0 {
                return None;
            }
            let full_crop = usage.crop.is_none_or(|crop| {
                crate::renderer::svg::compute_image_crop_src(
                    crop,
                    crate::renderer::metafile_geometry::wmf_crop_reference(usage.crop_reference),
                    width,
                    height,
                ) == (0.0, 0.0, width, height)
            });
            let unscaled_window = root
                .attribute("data-rhwp-wmf-window")
                .and_then(xml::numbers::<8>)
                .is_some_and(|window| {
                    (window[2], window[3]) == (width, height) && window[4..] == [1.0, 1.0, 0.0, 0.0]
                });
            for (node, image) in image_uses(&xml)? {
                let Some(("image/png", bytes)) = xml::href(image).and_then(xml::image_bytes) else {
                    continue;
                };
                let Some((identity, pixels)) = raster_identity(&bytes) else {
                    continue;
                };
                let quality = (|| {
                    if !known_resources || !full_crop || !unscaled_window {
                        return None;
                    }
                    let nominal = usage.width?;
                    let (rect, _, _) = super::images::placement(node, image)?;
                    let image_width = rect[2];
                    let device_width = (image_width * nominal / width).floor();
                    // 96dpi 폭의 floor가 0이어도 600dpi 원본 blit은 유효할
                    // 수 있다. 품질식의 하한 35를 그대로 소비한다.
                    if !device_width.is_finite() || device_width < 0.0 {
                        return None;
                    }
                    Some(
                        (75.0 * device_width / f64::from(pixels))
                            .floor()
                            .clamp(35.0, 75.0) as u8,
                    )
                })();
                result.qualities.entry(identity).or_insert(quality);
            }
            if root.attribute("data-rhwp-wmf-polygon-seen") == Some("1") {
                preceding_polygon = Some(true);
            }
        }
        Some(result)
    }

    pub(super) fn source(&self, data: &[u8]) -> Option<&Source> {
        self.sources
            .get(blake3::hash(data).as_bytes())
            .filter(|source| source.preceding_polygon.is_some())
    }

    pub(super) fn jpeg(&self, data: &[u8]) -> Option<Vec<u8>> {
        let (key, _) = raster_identity(data)?;
        let quality = self.qualities.get(&key).copied().flatten()?;
        encode_print_jpeg(data, quality, false)
    }
}
