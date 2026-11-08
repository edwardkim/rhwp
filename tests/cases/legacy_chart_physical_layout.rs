//! 독립 Print와 Native/fresh WASM 전쪽 증거는 assets의 README에 보존한다.
//! 제목·축·범례의 관계를 검사하고, 양쪽100% 완료 페이지의 축에는±2px 검사를 둔다.
use rhwp::ole_chart::{parse_ole_chart_contents, render_ole_chart_svg_fragment_with_contents};
use std::io::{Cursor, Read};
use std::path::Path;

const ASSETS: &str = "mydocs/pr/assets/legacy_chart_physical_layout_20261020/inputs";
fn svg(file: &str, size: [f64; 2]) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = if file == "143E433F503322BD33.hwpx" {
        root.join("samples/hwpx").join(file)
    } else {
        root.join(ASSETS).join(file)
    };
    let mut zip =
        zip::ZipArchive::new(Cursor::new(std::fs::read(path).expect("검증한 입력"))).unwrap();
    let mut ole = Vec::new();
    let names: Vec<_> = zip
        .file_names()
        .map(|name| name.expect("ZIP entry name").into_owned())
        .filter(|name| name.to_ascii_lowercase().ends_with(".ole"))
        .collect();
    assert_eq!(names.len(), 1, "입력의 실제 차트 payload 하나");
    zip.by_name(&names[0])
        .unwrap()
        .read_to_end(&mut ole)
        .unwrap();
    let offset = if ole.starts_with(&[0xd0, 0xcf, 0x11, 0xe0]) {
        0
    } else {
        4
    };
    let mut compound = cfb::CompoundFile::open(Cursor::new(&ole[offset..])).unwrap();
    let mut contents = Vec::new();
    compound
        .open_stream("Contents")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let chart = parse_ole_chart_contents(&contents).unwrap();
    render_ole_chart_svg_fragment_with_contents(&chart, &contents, [0., 0., size[0], size[1]], 1)
}
fn source(file: &str) -> String {
    svg(file, [302.36, 302.36])
}
fn number(n: roxmltree::Node<'_, '_>, key: &str) -> f64 {
    n.attribute(key).unwrap().parse().unwrap()
}
fn text<'a>(d: &'a roxmltree::Document<'a>, value: &str) -> roxmltree::Node<'a, 'a> {
    let values: Vec<_> = d
        .descendants()
        .filter(|n| n.has_tag_name("text") && n.text() == Some(value))
        .collect();
    assert_eq!(values.len(), 1, "글자의 누락·중복: {value}");
    values[0]
}
fn marker<'a>(d: &'a roxmltree::Document<'a>, value: &str) -> roxmltree::Node<'a, 'a> {
    let m = text(d, value).prev_sibling().unwrap();
    assert!(m.has_tag_name("rect"));
    m
}
#[test]
fn title_size_reserves_space_above_plot_without_moving_legend() {
    let a = source("pension-title-font-08.hwpx");
    let b = source("pension-title-font-20.hwpx");
    let a = roxmltree::Document::parse(&a).unwrap();
    let b = roxmltree::Document::parse(&b).unwrap();
    assert!(number(text(&b, "연금　재정　전망"), "y") > number(text(&a, "연금　재정　전망"), "y"));
    assert!(number(text(&b, "2000"), "y") > number(text(&a, "2000"), "y"));
    assert_eq!(number(text(&a, "0"), "y"), number(text(&b, "0"), "y"));
    for name in ["적립금", "수입", "지출"] {
        assert_eq!(number(marker(&a, name), "y"), number(marker(&b, name), "y"));
    }
}
#[test]
fn global_legend_font_owns_marker_size_row_packing_and_plot_reservation() {
    let a = source("pension-legend-08-font.hwpx");
    let b = source("pension-legend-20-font.hwpx");
    let a = roxmltree::Document::parse(&a).unwrap();
    let b = roxmltree::Document::parse(&b).unwrap();
    assert!(number(marker(&b, "적립금"), "height") > number(marker(&a, "적립금"), "height"));
    assert_eq!(
        number(marker(&a, "적립금"), "y"),
        number(marker(&a, "수입"), "y")
    );
    assert!(number(marker(&b, "적립금"), "y") < number(marker(&b, "수입"), "y"));
    assert_eq!(
        number(marker(&b, "적립금"), "x"),
        number(marker(&b, "수입"), "x")
    );
    assert!(number(text(&b, "0"), "y") < number(text(&a, "0"), "y"));
    for name in ["적립금", "수입", "지출"] {
        assert!(number(text(&b, name), "font-size") > number(text(&a, name), "font-size"));
        assert_eq!(
            text(&b, name).attribute("font-size"),
            text(&b, "적립금").attribute("font-size")
        );
    }
}
#[test]
fn series_cached_font_cannot_override_the_global_legend_font() {
    let a = source("143E433F503322BD33.hwpx");
    let b = source("pension-series-20-font.hwpx");
    let a = roxmltree::Document::parse(&a).unwrap();
    let b = roxmltree::Document::parse(&b).unwrap();
    for name in ["적립금", "수입", "지출"] {
        assert_eq!(
            text(&a, name).attribute("font-size"),
            text(&b, name).attribute("font-size")
        );
        assert_eq!(
            number(marker(&a, name), "height"),
            number(marker(&b, name), "height")
        );
    }
    assert_eq!(number(text(&a, "0"), "y"), number(text(&b, "0"), "y"));
    let center = number(text(&b, "적립금"), "y");
    let m = marker(&b, "적립금");
    assert!(
        center > number(m, "y") && center < number(m, "y") + number(m, "height"),
        "새 글꼴로 복원한 글자는 자기 셀 안에 놓인다"
    );
    assert!(
        center > number(text(&a, "적립금"), "y"),
        "독립 Print에서 확인한 캐시 재측정의 방향"
    );
}
#[test]
fn axis_font_size_changes_owned_plot_budget() {
    let original = source("143E433F503322BD33.hwpx");
    let value = source("pension-value-font-20.hwpx");
    let category = source("pension-category-font-20.hwpx");
    let a = roxmltree::Document::parse(&original).unwrap();
    let v = roxmltree::Document::parse(&value).unwrap();
    let c = roxmltree::Document::parse(&category).unwrap();
    assert!(number(text(&v, "0"), "x") > number(text(&a, "0"), "x"));
    assert!(number(text(&v, "2000"), "y") > number(text(&a, "2000"), "y"));
    assert_eq!(number(text(&v, "0"), "y"), number(text(&a, "0"), "y"));
    assert!(number(text(&c, "0"), "y") < number(text(&a, "0"), "y"));
    assert_eq!(number(text(&c, "2000"), "y"), number(text(&a, "2000"), "y"));
}
#[test]
fn narrow_legend_reserves_section_padding_before_packing() {
    let svg = svg("pension-narrow.hwpx", [17000. / 75., 302.36]);
    let d = roxmltree::Document::parse(&svg).unwrap();
    let first = marker(&d, "적립금");
    let second = marker(&d, "수입");
    let third = marker(&d, "지출");
    assert_eq!(number(first, "x"), number(second, "x"));
    assert_eq!(
        number(first, "x"),
        number(third, "x"),
        "독립 Print의 좁은 범례는 한 열 세 행이다"
    );
    assert!(number(first, "y") < number(second, "y") && number(second, "y") < number(third, "y"));
}
#[test]
fn short_plot_suppresses_axes_when_tick_label_lines_do_not_fit() {
    let svg = svg("pension-short.hwpx", [302.36, 17000. / 75.]);
    let d = roxmltree::Document::parse(&svg).unwrap();
    assert!(!d
        .descendants()
        .any(|n| n.has_tag_name("text") && matches!(n.text(), Some("2000" | "2010년"))));
    for name in ["적립금", "수입", "지출"] {
        let m = marker(&d, name);
        assert!(number(text(&d, name), "x") > number(m, "x") + number(m, "width"));
    }
}

#[test]
fn hidden_axis_fonts_margins_and_ticks_preserve_visible_bar_geometry() {
    fn bars(svg: &str) -> Vec<[f64; 4]> {
        let d = roxmltree::Document::parse(svg).unwrap();
        let markers: Vec<_> = ["적립금", "수입", "지출"]
            .map(|name| marker(&d, name).id())
            .into();
        d.descendants()
            .filter(|n| {
                n.has_tag_name("rect")
                    && n.attribute("fill").is_some_and(|v| v != "none")
                    && !markers.contains(&n.id())
            })
            .map(|n| {
                [
                    number(n, "x"),
                    number(n, "y"),
                    number(n, "width"),
                    number(n, "height"),
                ]
            })
            .collect()
    }
    let size = [302.36, 17000. / 75.];
    let reference = bars(&svg("pension-short.hwpx", size));
    assert_eq!(reference.len(), 12, "독립 Print의4범주·3계열 막대 보존");
    for file in [
        "pension-short-value-font-20.hwpx",
        "pension-short-category-font-20.hwpx",
        "pension-short-value-margin-0.hwpx",
        "pension-short-value-margin-200.hwpx",
        "pension-short-value-tick-0.hwpx",
        "pension-short-value-tick-200.hwpx",
    ] {
        let changed = bars(&svg(file, size));
        assert_eq!(changed.len(), reference.len(), "{file}");
        for (a, b) in reference.iter().flatten().zip(changed.iter().flatten()) {
            assert!(
                (a - b).abs() / size[0] <= 0.000001,
                "비표시 축의 속성이 실제 막대 기하를 바꾸면 안 된다: {file}: {a} -> {b}"
            );
        }
    }
}

#[test]
fn isolated_print_page_100_percent_keeps_important_axis_anchors_within_two_pixels() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn chart(n: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::RawSvg(raw) = &n.node_type {
            if raw.svg.contains("hwp-ole-chart") {
                return Some(n);
            }
        }
        n.children.iter().find_map(chart)
    }
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/143E433F503322BD33-ole.hwp"),
    )
    .unwrap();
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let node = chart(&tree.root).expect("실제 OLE 출력");
    let RenderNodeType::RawSvg(raw) = &node.node_type else {
        unreachable!()
    };
    let d = roxmltree::Document::parse(&raw.svg).unwrap();
    let axis = d
        .descendants()
        .filter(|n| n.has_tag_name("path"))
        .filter_map(|n| {
            let path = n.attribute("d")?;
            let values: Vec<f64> = path
                .strip_prefix('M')?
                .split('L')
                .flat_map(str::split_whitespace)
                .map(str::parse)
                .collect::<Result<_, _>>()
                .ok()?;
            (values.len() == 4 && values[0] == values[2]).then_some(values)
        })
        .max_by(|a, b| (a[3] - a[1]).abs().total_cmp(&(b[3] - b[1]).abs()))
        .expect("세로 값축");
    let x = node.bbox.x + axis[0];
    let top = node.bbox.y + axis[1].min(axis[3]);
    let bottom = node.bbox.y + axis[1].max(axis[3]);
    // 96DPI, Native/fresh WASM p1 모두 raw100%. 실제 Print 벡터의 독립 좌표다.
    for (actual, expected) in [(x, 296.96), (top, 222.3173), (bottom, 377.5067)] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "Print 축 경계: {actual} != {expected}"
        );
    }
    assert!(node.bbox.y + number(text(&d, "연금　재정　전망"), "y") < top);
    for name in ["적립금", "수입", "지출"] {
        assert!(
            node.bbox.y + number(marker(&d, name), "y") > bottom,
            "범례는 플롯 아래: {name}"
        );
    }
}

#[test]
fn real_picture_page_background_uses_print_rgb_once_without_mutating_source() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
    fn background(n: &LayerNode) -> Option<&[u8]> {
        match &n.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(background),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                background(child)
            }
            LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
                if let PaintOp::PageBackground { background, .. } = op {
                    background.image.as_ref().map(|i| i.data.as_slice())
                } else {
                    None
                }
            }),
        }
    }
    let input =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/143E433F503322BD33.hwp"))
            .unwrap();
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&input).unwrap();
    let source: Vec<_> = core
        .document()
        .bin_data_content
        .iter()
        .filter_map(|b| image::load_from_memory(&b.data.load_shared()).ok())
        .filter(|i| i.width() == 512 && i.height() == 512)
        .collect();
    assert_eq!(source.len(), 1, "원본의 쪽 배경 그림");
    let original = source[0].to_rgba8();
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let rendered = image::load_from_memory(background(&tree.root).expect("실제 쪽 배경 paint"))
        .unwrap()
        .to_rgba8();
    assert_eq!(rendered.dimensions(), original.dimensions());
    let mut compared = 0;
    for (x, y, pixel) in original.enumerate_pixels() {
        if pixel.0[..3] == [0, 118, 191] && pixel[3] == 255 {
            // 독립 Print 이미지 객체의 대응 파랑은 RGB197/255/255다. JPEG 허용 오차만 둔다.
            let actual = rendered.get_pixel(x, y);
            for (got, want) in actual.0.into_iter().zip([197_u8, 255, 255, 255]) {
                assert!(got.abs_diff(want) <= 2, "Print 색조: {actual:?}");
            }
            compared += 1;
        }
    }
    assert!(
        compared > 0,
        "원본 파랑 표본이 사라지면 검사하지 않은 것으로 본다"
    );
    let unchanged = core
        .document()
        .bin_data_content
        .iter()
        .filter_map(|b| image::load_from_memory(&b.data.load_shared()).ok())
        .any(|i| i.to_rgba8() == original);
    assert!(
        unchanged,
        "render-node 보정이 원본 BinData를 바꾸면 안 된다"
    );
}
