//! WMF의 장치 클립·표시 외곽과 CBTA 표지의 실제 로고를 보호한다.
//! 같은 원문/한컴2020 Print 전204쪽의 Native/fresh WASM 최저90% 이상,
//! 표지 각각99.99681/99.99361%와 직접 review 후 추가한다.
//! 실물 px는 독립 Print의 로고 잉크 네 변 ±2CSSpx이며, 나머지 생성 WMF의
//! 수치는 문서 화면 핀이 아닌 WMF 레코드의 논리 좌표·DC 상태 계약이다.
//! 근거: mydocs/pr/assets/issue8152/wmf-device-clip-20261025/validation.json.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine as _;
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};

struct Cover {
    frame: [f64; 4],
    svg: String,
    #[cfg(feature = "native-skia")]
    png: image::RgbImage,
}

fn cover() -> &'static Cover {
    static COVER: OnceLock<Cover> = OnceLock::new();
    COVER.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = root.join("output").join(format!(
            "wmf-clip-contract-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&out).unwrap();
        let binary = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
        let input =
            root.join("samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp");
        let export = |command: &str, folder: &str| {
            let destination = out.join(folder);
            let result = Command::new(&binary)
                .arg(command)
                .arg(&input)
                .args([
                    "--compat",
                    "2022",
                    "--profile",
                    "print",
                    "--page",
                    "0",
                    "-o",
                ])
                .arg(&destination)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let mut paths: Vec<_> = fs::read_dir(destination)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|p| p.extension().is_some_and(|ext| ext == folder))
                .collect();
            assert_eq!(paths.len(), 1, "표지 한 쪽의 실제 산출물");
            paths.pop().unwrap()
        };
        let source = fs::read_to_string(export("export-svg", "svg")).unwrap();
        let document = roxmltree::Document::parse(&source).unwrap();
        let images: Vec<_> = document
            .descendants()
            .filter(|node| node.has_tag_name("image"))
            .filter_map(|node| {
                let uri = node.attribute("href")?;
                let payload = uri.strip_prefix("data:image/svg+xml;base64,")?;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(payload)
                    .unwrap();
                let frame = ["x", "y", "width", "height"]
                    .map(|key| node.attribute(key).unwrap().parse::<f64>().unwrap());
                Some((frame, String::from_utf8(decoded).unwrap()))
            })
            .collect();
        assert_eq!(images.len(), 1, "원문에 있는 WMF 그림 소속을 유지한다");
        let (frame, svg) = images.into_iter().next().unwrap();
        #[cfg(feature = "native-skia")]
        let png = image::open(export("export-png", "png")).unwrap().to_rgb8();
        fs::remove_dir_all(out).unwrap();
        Cover {
            frame,
            svg,
            #[cfg(feature = "native-skia")]
            png,
        }
    })
}

#[test]
fn saved_cover_uses_the_source_clip_for_both_paint_and_intrinsic_extent() {
    let cover = cover();
    let document = roxmltree::Document::parse(&cover.svg).unwrap();
    let image = document
        .descendants()
        .find(|n| n.has_tag_name("image"))
        .unwrap();
    let id = image
        .attribute("clip-path")
        .expect("원본 INTERSECTCLIPRECT가 실제 그림에도 적용돼야 한다")
        .strip_prefix("url(#")
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    let clip = document
        .descendants()
        .find(|n| n.attribute("id") == Some(id))
        .unwrap();
    let rectangle = clip.children().find(|n| n.has_tag_name("rect")).unwrap();
    let number = |node: roxmltree::Node<'_, '_>, key: &str| {
        node.attribute(key).unwrap().parse::<f64>().unwrap()
    };
    // 원본 WMF: origin(2752,644), clip(2751,644,3111,752).
    assert_eq!(
        ["x", "y", "width", "height"].map(|k| number(rectangle, k)),
        [-1.0, 0.0, 360.0, 108.0]
    );
    let root = document.root_element();
    assert_eq!(root.attribute("viewBox"), Some("0 0 359 108"));
    assert!(number(root, "width") <= number(rectangle, "x") + number(rectangle, "width"));
    assert!(number(root, "height") <= number(rectangle, "y") + number(rectangle, "height"));
    // 잘린 원본 비트맵의 데이터와 배치는 보존하며 SVG viewport만 실제 표시 영역을 소비한다.
    assert!(number(image, "width") > number(root, "width"));
    assert!(number(image, "height") > number(root, "height"));
    assert!(image
        .attribute("href")
        .unwrap()
        .starts_with("data:image/png;base64,"));
}

#[cfg(feature = "native-skia")]
#[test]
fn saved_cover_keeps_the_independent_print_logo_ink_bounds() {
    let cover = cover();
    let [x, y, width, height] = cover.frame;
    let mut bounds = [u32::MAX, u32::MAX, 0, 0];
    let mut count = 0;
    for py in y.floor() as u32..(y + height).ceil() as u32 {
        for px in x.floor() as u32..(x + width).ceil() as u32 {
            let [r, g, b] = cover.png.get_pixel(px, py).0.map(u32::from);
            if 299 * r + 587 * g + 114 * b < 160_000 {
                count += 1;
                bounds[0] = bounds[0].min(px);
                bounds[1] = bounds[1].min(py);
                bounds[2] = bounds[2].max(px + 1);
                bounds[3] = bounds[3].max(py + 1);
            }
        }
    }
    assert!(count > 0, "그림을 지워서 클립 검사를 통과시키면 안 된다");
    // 독립2020 Print / PDFium96DPI, 같은 원문 bitmap의 명도<160 잉크 외곽.
    for (actual, expected) in bounds.into_iter().zip([518u32, 959, 704, 1003]) {
        assert!(
            actual.abs_diff(expected) <= 2,
            "독립 Print의 로고 잉크 네 변 ±2CSSpx: {bounds:?}"
        );
    }
}

fn record(function: u16, parameters: &[i16], bytes: &mut Vec<u8>) {
    let data: Vec<_> = parameters.iter().flat_map(|x| x.to_le_bytes()).collect();
    raw_record(function, &data, bytes);
}
fn raw_record(function: u16, data: &[u8], bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(3 + data.len() as u32 / 2).to_le_bytes());
    bytes.extend_from_slice(&function.to_le_bytes());
    bytes.extend_from_slice(data);
}
fn bitmap(x: i16, y: i16, width: i16, height: i16, bgr: [u8; 3], bytes: &mut Vec<u8>) {
    let mut data = 0x00CC_0020u32.to_le_bytes().to_vec();
    for value in [2i16, 2, 0, 0, height, width, y, x] {
        data.extend_from_slice(&value.to_le_bytes());
    }
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&2i32.to_le_bytes());
    data.extend_from_slice(&2i32.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&24u16.to_le_bytes());
    data.extend_from_slice(&[0; 24]);
    for _ in 0..2 {
        data.extend_from_slice(&bgr);
        data.extend_from_slice(&bgr);
        data.extend_from_slice(&[0; 2]);
    }
    raw_record(0x0B41, &data, bytes);
}
fn primitive(draw: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut bytes = 0x9AC6_CDD7u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for value in [0i16, 0, 100, 100, 96] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for value in [1u16, 9, 0x0300] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    draw(&mut bytes);
    record(0, &[], &mut bytes);
    String::from_utf8(
        WMFConverter::new(bytes.as_slice(), SVGPlayer::new())
            .run()
            .unwrap(),
    )
    .unwrap()
}
#[cfg(feature = "native-skia")]
fn ink_bounds(svg: &str) -> Option<[usize; 4]> {
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).unwrap();
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let mut bounds = [usize::MAX, usize::MAX, 0, 0];
    let mut painted = false;
    for (i, pixel) in pixmap.pixels().iter().enumerate() {
        if pixel.alpha() > 0 {
            painted = true;
            let x = i % size.width() as usize;
            let y = i / size.width() as usize;
            bounds[0] = bounds[0].min(x);
            bounds[1] = bounds[1].min(y);
            bounds[2] = bounds[2].max(x + 1);
            bounds[3] = bounds[3].max(y + 1);
        }
    }
    painted.then_some(bounds)
}

#[test]
fn primitive_clip_and_unclipped_extent_keep_distinct_source_contracts() {
    for clipped in [false, true] {
        let svg = primitive(|bytes| {
            record(0x020B, &[20, 40], bytes);
            record(0x020C, &[10, 20], bytes);
            if clipped {
                record(0x0416, &[30, 60, 20, 40], bytes);
            }
            bitmap(0, 0, 100, 100, [20, 40, 80], bytes);
        });
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let root = doc.root_element();
        let (width, height) = if clipped { (20, 10) } else { (60, 80) };
        assert_eq!(
            root.attribute("width").unwrap().parse::<usize>().unwrap(),
            width
        );
        assert_eq!(
            root.attribute("height").unwrap().parse::<usize>().unwrap(),
            height
        );
        #[cfg(feature = "native-skia")]
        assert_eq!(ink_bounds(&svg), Some([0, 0, width, height]));
    }
}

#[test]
fn primitive_disjoint_intersection_stays_empty_until_dc_restore() {
    let svg = primitive(|bytes| {
        record(0x020C, &[100, 100], bytes);
        record(0x0416, &[40, 40, 10, 10], bytes);
        record(0x001E, &[], bytes);
        record(0x0416, &[80, 80, 50, 50], bytes);
        bitmap(0, 0, 100, 100, [0, 0, 255], bytes);
        record(0x0127, &[-1], bytes);
        bitmap(0, 0, 100, 100, [255, 0, 0], bytes);
    });
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let images: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("image"))
        .collect();
    assert_eq!(images.len(), 2);
    assert_ne!(
        images[0].attribute("clip-path"),
        images[1].attribute("clip-path")
    );
    assert!(doc.descendants().any(|n| n.has_tag_name("rect")
        && n.attribute("width") == Some("0")
        && n.attribute("height") == Some("0")));
    #[cfg(feature = "native-skia")]
    assert_eq!(ink_bounds(&svg), Some([10, 10, 40, 40]));
}

#[test]
fn primitive_clip_is_frozen_in_device_space_for_both_region_routes_and_axes() {
    for inverted in [false, true] {
        for selected_region in [false, true] {
            let svg = primitive(|bytes| {
                let extent = if inverted { -100 } else { 100 };
                record(0x020B, &[100, 100], bytes);
                record(0x020C, &[extent, extent], bytes);
                let (start, end) = if inverted { (80, 100) } else { (100, 120) };
                if selected_region {
                    let mut region = vec![];
                    region.extend_from_slice(&0u16.to_le_bytes());
                    region.extend_from_slice(&6u16.to_le_bytes());
                    region.extend_from_slice(&1u32.to_le_bytes());
                    for value in [
                        34i16, 1, 2, start, start, end, end, 2, start, end, start, end, 2,
                    ] {
                        region.extend_from_slice(&value.to_le_bytes());
                    }
                    raw_record(0x06FF, &region, bytes);
                    record(0x012C, &[0], bytes);
                } else {
                    record(0x0416, &[end, end, start, start], bytes);
                }
                let origin = if inverted { 90 } else { 110 };
                record(0x020B, &[origin, origin], bytes);
                bitmap(origin, origin, extent, extent, [20, 40, 80], bytes);
            });
            let doc = roxmltree::Document::parse(&svg).unwrap();
            let rect = doc.descendants().find(|n| n.has_tag_name("rect")).unwrap();
            assert_eq!(
                ["x", "y", "width", "height"].map(|k| rect
                    .attribute(k)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()),
                [0.0, 0.0, 20.0, 20.0]
            );
            #[cfg(feature = "native-skia")]
            assert_eq!(ink_bounds(&svg), Some([0, 0, 20, 20]));
        }
    }
}
