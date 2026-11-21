//! 관계형: 표시용 분할 프로그램의 원본 HFT 폭 보존과 실제 glyph paint를 비교한다.
//! 주석 갱신: 2026-11-22T07:19:13+09:00.
//! 입력은 MIT RHWPHftOutline.otf의 cmap/hmtx만 메모리에서 바꾼 합성 font-only 계약이다.
//! 실제 문서 좌표·golden·시각 측정값을 고정하지 않는다. 합성 래스터에는 독립 PDF가 없다.
//! 근거: mydocs/pr/assets/issues7754_7765/source35/metric-provenance/README.md.
//! 실제 #7754의 Native/fresh WASM 전쪽 측정은 해당 근거에서 별도로 구분한다.

use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    native_pua_fonts::NativePuaFonts,
    TextStyle,
};
use std::sync::Arc;

const CLUSTER: &str = "ᄒᆞᆫ";
const COMPATIBLE: char = '\u{f53a}';

fn table(bytes: &[u8], tag: &[u8; 4]) -> usize {
    (0..usize::from(u16::from_be_bytes([bytes[4], bytes[5]])))
        .find_map(|i| {
            let p = 12 + i * 16;
            (&bytes[p..p + 4] == tag)
                .then(|| u32::from_be_bytes(bytes[p + 8..p + 12].try_into().unwrap()) as usize)
        })
        .expect("licensed fixture table")
}

// The generated fixture has a format-4 cmap with one non-sentinel segment.
// No installed/private font or outline bytes are added to the repository.
fn font(character: char, advance: u16) -> Vec<u8> {
    let mut bytes = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf").to_vec();
    let cmap = table(&bytes, b"cmap");
    let sub = cmap + u32::from_be_bytes(bytes[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
    assert_eq!(
        u16::from_be_bytes(bytes[sub..sub + 2].try_into().unwrap()),
        4
    );
    for offset in [sub + 14, sub + 20] {
        bytes[offset..offset + 2].copy_from_slice(&(character as u16).to_be_bytes());
    }
    bytes[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub(character as u16).to_be_bytes());
    let hmtx = table(&bytes, b"hmtx");
    bytes[hmtx + 4..hmtx + 6].copy_from_slice(&advance.to_be_bytes());
    bytes
}

fn original_style() -> TextStyle {
    TextStyle {
        font_family: "HY Sinmyeongjo".into(),
        font_size: 20.0,
        source_font_is_hft: true,
        hft_font_face: true,
        hft_hangul_face: true,
        original_glyph_metric_family: Some(Arc::new("한양신명조".into())),
        ..Default::default()
    }
}

fn supply(style: &mut TextStyle, programs: &[Vec<u8>]) {
    let mut fonts = NativePuaFonts::default();
    for program in programs {
        fonts
            .register_otf("display", false, false, program, 0)
            .expect("licensed supplied program");
    }
    style.native_otf_metrics = fonts.get_otf("display", false, false);
}

fn width(style: &TextStyle) -> f64 {
    *EmbeddedTextMeasurer
        .compute_char_positions(CLUSTER, style)
        .last()
        .expect("cluster end")
}

#[test]
fn display_program_preserves_original_hft_advance() {
    let mut style = original_style();
    let original = width(&style);
    supply(&mut style, &[font(COMPATIBLE, 900)]);
    assert_eq!(
        width(&style),
        original,
        "display CFF must not own HFT width"
    );

    style.natural_program_metrics = true;
    let natural = width(&style);
    assert!(
        natural < original,
        "explicit natural metrics consume supplied hmtx"
    );
    style.source_font_is_hft = false;
    assert_eq!(width(&style), natural);
    style.natural_program_metrics = false;
    assert_eq!(
        width(&style),
        natural,
        "non-HFT keeps actual program metrics"
    );
}

#[test]
fn earlier_original_jamo_is_not_replaced_by_later_compatible_program() {
    let first = font('\u{1112}', 700);
    let later = font(COMPATIBLE, 900);
    let mut style = original_style();
    style.natural_program_metrics = true;
    supply(&mut style, std::slice::from_ref(&first));
    let before = width(&style);
    supply(&mut style, &[first, later]);
    assert_eq!(
        width(&style),
        before,
        "original cmap keeps ordered ownership"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn native_paint_reaches_visible_compatible_glyph_in_later_fragment() {
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::{
        layer_renderer::RasterRenderOptions,
        render_tree::{BoundingBox, TextRunNode},
        skia::SkiaLayerRenderer,
    };

    let render = |text: &str, glyph: char| {
        let programs = [font('X', 1000), font(glyph, 1000)];
        let mut style = original_style();
        style.font_family = "RHWP HFT Outline Fixture".into();
        style.font_size = 32.0;
        supply(&mut style, &programs);
        let run = TextRunNode {
            text: text.into(),
            style,
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 32.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        };
        let mut tree = PageLayerTree::new(
            64.0,
            64.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 40.0, 40.0),
                    run,
                )],
            ),
        );
        tree.profile = RenderProfile::Print;
        SkiaLayerRenderer::new()
            .try_with_font_data(&programs)
            .expect("explicit licensed font bytes")
            .render_pixels_with_options(&tree, RasterRenderOptions::default())
            .expect("actual Native text replay")
    };
    let actual = render(CLUSTER, COMPATIBLE);
    let control = render("가", '가');
    assert!(control
        .bytes
        .chunks_exact(4)
        .any(|p| p[3] > 0 && p[..3] != [255; 3]));
    assert!(
        actual == control,
        "same original outline must paint through split lookup"
    );
}
