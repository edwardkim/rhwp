//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-08 21:40:40 KST. 직전 변경: 2026-11-08T20:27:19+09:00, 1b159d1f855c82a28de8b909b6f15a9b1d65c7b7.
//! 프로그램 선택은 관계형 계약이며 실문서 raw는 비해당이다. Linux·Mac은 MIT 합성 TTF 두 개만 공급한다.
//! Linux는 자식의 Fontconfig, macOS는 CoreText의 process 범위 등록으로 실제 OS 선택을 대조한다.
//! Windows는 DirectWrite의 기본 face를 독립 조회한 뒤 같은 프로그램의 명시 공급 출력과 대조한다.
//! Windows에서는 GDI의 private 등록을 DirectWrite 등록으로 간주하지 않으며 개인·한컴 글꼴을 요구하지 않는다.
//! macOS의 사용자·시스템 글꼴을 설치·삭제하지 않으며, 운영체제 사이의 raster 바이트는 비교하지 않는다.
//! 2026-10-28 측정 raw(96dpi, PDFium): calendar_year.hwp 1쪽 Native 98.82866% / fresh WASM 98.50891%; 전1쪽 최저 98.82866% / 98.50891%.
//! sungeo.hwp 34쪽 Native 99.97528% / fresh WASM 99.98763%; 전94쪽 최저 99.85164% / 99.92219%.
//! 합성 글리프 검사는 독립 원본 문서가 없는 raster 격자 검사다.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json.

#![cfg(feature = "native-skia")]

//! 생성 번호의 소수 원점과 일반 본문의 기존 raster 격자를 함께 보호한다.
//! 장치 한 칸보다 가는 raster 획의 최소 굵기(#8125)도 같은 Native raster 격자 계약이라 여기서 검사한다.

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterPixels;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::TextStyle;

fn glyph_at(renderer: &SkiaLayerRenderer, y: f64, fixed_glyph_advances: bool) -> RasterPixels {
    let run = TextRunNode {
        text: "H".into(),
        style: TextStyle {
            font_family: "Noto Sans KR".into(),
            font_size: 32.0,
            color: 0,
            fixed_glyph_advances,
            ..Default::default()
        },
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
    let tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 64.0),
            None,
            vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
        ),
    );
    renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .expect("글리프를 투명 RGBA에 재생한다")
}

/// 검사 유형: 고정형(독립 Print PDF의 192DPI 화소 위치 ±2px, 채널 차이 32 이하).
/// 주석 갱신: 2026-11-07 03:33 KST.
/// interword-space-{wide,edge,tight}.hwpx 전1쪽 Native/fresh WASM 각각 100%에서 추가했다.
/// 기준: pdf/issue4138/interword-space-tight-2024.pdf의 왼쪽 용지 테두리.
/// PDFium 192DPI의 y=1250, x=37/38 회색 값 60/194는 글자 없는 구간의 독립 관측이다.
/// 표 괘선의 최소 장치 폭 검사는 위의 기존 모듈에 유지하고, 이 검사는 용지 테두리의
/// 소수 원점이 정수 화소 중심으로 옮겨져 두 화소의 coverage가 사라지는 회귀를 검출한다.
mod issue_4138_page_frame_fractional_coverage {
    use super::*;
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::render_tree::RenderNodeType;
    use rhwp::DocumentCore;

    #[test]
    fn paper_frame_preserves_independent_print_fractional_edge_coverage() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut core = DocumentCore::from_bytes(
            &std::fs::read(root.join("pdf/issue4138/interword-space-tight.hwpx"))
                .expect("독립 Print와 동일한 입력"),
        )
        .expect("HWPX 입력");
        core.set_hangul2024_compat(true);
        let page = core
            .build_page_render_tree(0)
            .expect("용지 테두리 생산 경로");
        // 글꼴 공급과 무관하게 실제 조판기가 만든 네 변을 그대로 재생한다.
        let ops: Vec<_> = page
            .root
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::Line(line) => Some(PaintOp::line(node.bbox, line.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(ops.len(), 4, "원본 용지 테두리 네 변");
        let bounds = page.root.bbox;
        let tree = PageLayerTree::with_profile(
            bounds.width,
            bounds.height,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, bounds.width, bounds.height),
                None,
                ops,
            ),
            RenderProfile::Print,
        );
        let pixels = SkiaLayerRenderer::new()
            .render_pixels_with_options(
                &tree,
                RasterRenderOptions {
                    scale: 2.0,
                    transparent: false,
                    background_color: Some(0x00FF_FFFF),
                    ..Default::default()
                },
            )
            .expect("글꼴 없는 Native 용지 테두리 재생");
        let observed = |x: usize| {
            let at = 1250 * pixels.stride + x * 4;
            [pixels.bytes[at], pixels.bytes[at + 1], pixels.bytes[at + 2]]
        };
        let matched = (-2i32..=2).any(|shift| {
            [(37i32, 60i32), (38, 194)]
                .into_iter()
                .all(|(x, expected)| {
                    observed((x + shift) as usize)
                        .into_iter()
                        .all(|value| (i32::from(value) - expected).abs() <= 32)
                })
        });
        assert!(
            matched,
            "독립 PDF의 두 화소 coverage [60, 194]가 ±2px 안에 있어야 한다: x=37/38 {:?}/{:?}",
            observed(37),
            observed(38),
        );
    }
}

fn ink_center(image: &RasterPixels) -> (f64, f64) {
    let mut mass = 0.0;
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
        for (x, pixel) in row[..image.width as usize * 4].chunks_exact(4).enumerate() {
            let weight = f64::from(pixel[3]);
            mass += weight;
            x_sum += x as f64 * weight;
            y_sum += y as f64 * weight;
        }
    }
    assert!(mass > 0.0, "공백/누락을 같은 위치의 잉크로 판정하지 않는다");
    (x_sum / mass, y_sum / mass)
}

#[test]
fn fixed_advance_glyph_preserves_quarter_pixel_baseline_translation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[root.join("ttfs/opensource")]);
    let original = ink_center(&glyph_at(&renderer, 4.1, true));
    let quarter = ink_center(&glyph_at(&renderer, 4.35, true));
    let whole = ink_center(&glyph_at(&renderer, 5.1, true));
    // 입력은 정확히1/4px 이동이다. alpha 양자화만 허용하며, 같은 정수 기준선으로
    // 반올림돼 이동이0이 되는 기존 raster 동작을 검출한다.
    let quarter_shift = quarter.1 - original.1;
    assert!(
        (quarter_shift - 0.25).abs() <= 0.08,
        "소수 기준선 이동이 잉크에 보존돼야 한다: {quarter_shift}"
    );
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}

#[test]
fn ordinary_ttf_keeps_existing_raster_baseline_grid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[root.join("ttfs/opensource")]);
    let original = ink_center(&glyph_at(&renderer, 4.1, false));
    let quarter = ink_center(&glyph_at(&renderer, 4.35, false));
    let whole = ink_center(&glyph_at(&renderer, 5.1, false));
    // 일반 TTF는 기존 화면 격자를 쓴다. 생성 번호의 별도 계약을 모든 본문에
    // 확장해 정상 쪽의 잉크 위치까지 바꾸는 회귀를 검출한다.
    assert!((quarter.1 - original.1).abs() <= 0.02);
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}

/// Native raster 의 장치 한 칸보다 가는 획은 한 칸 굵기로 그린다.
///
/// 한컴 Print PDF 를 raster 로 읽으면 0.1 mm(약 0.38 px) 표 괘선도 장치 한 칸을 채운다.
/// Native Skia 가 선언 굵기 그대로 anti-alias 하면 괘선의 덮임이 38% 안팎에 그쳐 거의 보이지
/// 않는다. `samples/basic/calendar_year.hwp`(한 쪽, 기준 `pdf/basic/calendar_year-2022.pdf`)는
/// 이 차이로 Native 실루엣이 88.25 였고, 한 칸 굵기로 그리면 98.87(fresh WASM 93.80)이다.
///
/// 조판 좌표·선언 굵기는 그대로이고 raster 획 굵기만 바뀌므로, render tree 의 실제 괘선 위치에서
/// 그 괘선을 가로지르는 화소 중 가장 진한 화소가 선 색 대비의 절반 이상인지 본다.
mod subpixel_stroke_hairline {
    use rhwp::document_core::DocumentCore;
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};

    fn thin_horizontal_lines(node: &RenderNode, out: &mut Vec<LineNode>) {
        if let RenderNodeType::Line(line) = &node.node_type {
            let horizontal = (line.y1 - line.y2).abs() < 0.01 && (line.x2 - line.x1).abs() > 20.0;
            if horizontal && line.style.width > 0.0 && line.style.width < 1.0 {
                out.push(line.clone());
            }
        }
        for child in &node.children {
            thin_horizontal_lines(child, out);
        }
    }

    #[test]
    fn subpixel_table_rules_cover_one_device_pixel() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/basic/calendar_year.hwp");
        let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
        let tree = core.build_page_render_tree(0).expect("page");
        let mut lines = Vec::new();
        thin_horizontal_lines(&tree.root, &mut lines);
        assert!(lines.len() >= 10, "가는 가로 괘선 — {}개", lines.len());

        let width = tree.root.bbox.width.round() as u32;
        let pixels = core
            .render_page_pixels_native_at_width(0, width, RenderProfile::Print, &[])
            .expect("raster");
        let scale = f64::from(pixels.width) / tree.root.bbox.width;
        let gray = |x: i32, y: i32| -> f64 {
            let at = y as usize * pixels.stride + x as usize * 4;
            let p = &pixels.bytes[at..at + 3];
            (f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2])) / 3.0
        };

        let mut faint = Vec::new();
        for line in &lines {
            let color = line.style.color;
            let ink = (f64::from((color & 0xff) as u8)
                + f64::from(((color >> 8) & 0xff) as u8)
                + f64::from(((color >> 16) & 0xff) as u8))
                / 3.0;
            let contrast = 255.0 - ink;
            if contrast < 64.0 {
                continue;
            }
            // 괘선 양 끝(교차 괘선)을 피해 가운데 세 지점을 본다.
            for t in [0.3, 0.5, 0.7] {
                let x = ((line.x1 + (line.x2 - line.x1) * t) * scale) as i32;
                let cy = line.y1 * scale;
                let darkest = ((cy - 2.0) as i32..=(cy + 2.0) as i32)
                    .filter(|y| *y >= 0 && *y < pixels.height)
                    .map(|y| 255.0 - gray(x, y))
                    .fold(0.0, f64::max);
                if darkest < contrast * 0.5 {
                    faint.push((line.y1, line.style.width, darkest / contrast));
                }
            }
        }
        assert!(
            faint.is_empty(),
            "장치 한 칸을 덮지 못한 괘선 {}곳 (y, 굵기, 최대 덮임) — {:?}",
            faint.len(),
            &faint[..faint.len().min(6)]
        );
    }

    fn thin_lines(node: &RenderNode, out: &mut Vec<LineNode>) {
        if let RenderNodeType::Line(line) = &node.node_type {
            let axis = (line.y1 - line.y2).abs() < 0.01 || (line.x1 - line.x2).abs() < 0.01;
            let long = (line.x2 - line.x1).hypot(line.y2 - line.y1) > 100.0;
            if axis && long && line.style.width > 0.0 && line.style.width < 1.0 {
                out.push(line.clone());
            }
        }
        for child in &node.children {
            thin_lines(child, out);
        }
    }

    /// 단일 실선의 인쇄 coverage 계획은 점선을 원래 굵기로 둔다. 점선 쪽 테두리도 장치 한 칸을
    /// 덮어야 한다 — `samples/basic/sungeo.hwp` 34쪽(기준 `pdf/basic/sungeo-2022.pdf`)의 점선 쪽
    /// 테두리 네 변은 선언 굵기 그대로면 거의 보이지 않아 Native raw 실루엣이 78.75 였고, 한 칸
    /// 굵기로 그리면 99.58(fresh WASM 99.99)이다.
    ///
    /// 같은 PDF 를 96DPI 로 raster 하면(poppler) 네 변 모두 회색 63 의 내용 화소다. Visual Sweep 의
    /// 내용 화소 판정(최소 채널 < 232)으로 rhwp 도 각 변에 내용 화소가 있는지 본다. 점선의 빈 칸을
    /// 피하려고 선 방향 ±20px 창에서 선을 가로지르는 ±2px 화소 중 가장 진한 값을 쓴다.
    #[test]
    fn subpixel_dashed_page_border_covers_one_device_pixel() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp");
        let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
        let page = 33;
        let tree = core.build_page_render_tree(page).expect("page");
        let mut lines = Vec::new();
        thin_lines(&tree.root, &mut lines);
        assert!(
            lines.len() >= 4,
            "쪽 테두리 네 변의 가는 선 — {}개",
            lines.len()
        );

        let width = tree.root.bbox.width.round() as u32;
        let pixels = core
            .render_page_pixels_native_at_width(page, width, RenderProfile::Print, &[])
            .expect("raster");
        let scale = f64::from(pixels.width) / tree.root.bbox.width;
        let min_channel = |x: i32, y: i32| -> u8 {
            let at = y as usize * pixels.stride + x as usize * 4;
            pixels.bytes[at..at + 3]
                .iter()
                .copied()
                .min()
                .unwrap_or(255)
        };
        let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < pixels.width && y < pixels.height;

        let mut faint = Vec::new();
        for line in &lines {
            let horizontal = (line.y1 - line.y2).abs() < 0.01;
            for t in [0.3, 0.5, 0.7] {
                let along = if horizontal {
                    (line.x1 + (line.x2 - line.x1) * t) * scale
                } else {
                    (line.y1 + (line.y2 - line.y1) * t) * scale
                };
                let across = if horizontal { line.y1 } else { line.x1 } * scale;
                let mut darkest = 255u8;
                for a in (along - 20.0) as i32..=(along + 20.0) as i32 {
                    for c in (across - 2.0) as i32..=(across + 2.0) as i32 {
                        let (x, y) = if horizontal { (a, c) } else { (c, a) };
                        if inside(x, y) {
                            darkest = darkest.min(min_channel(x, y));
                        }
                    }
                }
                if darkest >= 232 {
                    faint.push((horizontal, across / scale, darkest));
                }
            }
        }
        assert!(
            faint.is_empty(),
            "내용 화소가 없는 점선 쪽 테두리 {}곳 (가로, 위치, 가장 진한 최소 채널) — {:?}",
            faint.len(),
            &faint[..faint.len().min(6)]
        );
    }
}

// 기존 Native CI 진입점으로 실제 glyph 프로그램 선택 계약도 실행한다.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
mod system_program_preference {
    use rhwp::{
        paint::RenderProfile,
        renderer::{
            font_environment::FontEnvironment, layer_renderer::RasterRenderOptions,
            skia::SkiaLayerRenderer,
        },
        DocumentCore,
    };
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    use std::path::PathBuf;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const FAMILY: &str = "RHWP Host Fixture";
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const CHILD: &str = "RHWP_SYSTEM_PROGRAM_PREFERENCE_CHILD";
    #[cfg(target_os = "macos")]
    mod coretext {
        use std::{
            ffi::c_void,
            os::unix::ffi::OsStrExt,
            path::{Path, PathBuf},
        };

        #[link(name = "CoreFoundation", kind = "framework")]
        unsafe extern "C" {
            fn CFURLCreateFromFileSystemRepresentation(
                allocator: *const c_void,
                buffer: *const u8,
                length: isize,
                is_directory: u8,
            ) -> *const c_void;
            fn CFRelease(value: *const c_void);
        }
        #[link(name = "CoreText", kind = "framework")]
        unsafe extern "C" {
            fn CTFontManagerRegisterFontsForURL(
                url: *const c_void,
                scope: u32,
                error: *mut *const c_void,
            ) -> bool;
            fn CTFontManagerUnregisterFontsForURL(
                url: *const c_void,
                scope: u32,
                error: *mut *const c_void,
            ) -> bool;
        }

        // Apple의 kCTFontManagerScopeProcess: 현재 자식 프로세스에만 노출한다.
        // https://developer.apple.com/documentation/coretext/ctfontmanagerscope/process
        const PROCESS: u32 = 1;
        pub(super) struct Fonts(Vec<*const c_void>);
        impl Fonts {
            pub(super) fn register(directory: &Path) -> Self {
                Self::register_paths(&[
                    directory.join("a-other.ttf"),
                    directory.join("z-preferred.ttf"),
                ])
            }
            pub(super) fn register_paths(paths: &[PathBuf]) -> Self {
                let mut fonts = Self(Vec::new());
                for path in paths {
                    let bytes = path.as_os_str().as_bytes();
                    // CFURL은 등록 해제까지 보존하며, 원본 파일도 그동안 유지한다.
                    let url = unsafe {
                        CFURLCreateFromFileSystemRepresentation(
                            std::ptr::null(),
                            bytes.as_ptr(),
                            bytes.len() as isize,
                            0,
                        )
                    };
                    assert!(!url.is_null(), "합성 글꼴 파일의 CoreFoundation URL");
                    let registered = unsafe {
                        CTFontManagerRegisterFontsForURL(url, PROCESS, std::ptr::null_mut())
                    };
                    if !registered {
                        unsafe { CFRelease(url) };
                    }
                    assert!(registered, "CoreText process 글꼴 등록: {}", path.display());
                    fonts.0.push(url);
                }
                fonts
            }
        }
        impl Drop for Fonts {
            fn drop(&mut self) {
                for url in self.0.drain(..).rev() {
                    let removed = unsafe {
                        let removed =
                            CTFontManagerUnregisterFontsForURL(url, PROCESS, std::ptr::null_mut());
                        CFRelease(url);
                        removed
                    };
                    assert!(removed, "이 자식이 등록한 CoreText 글꼴만 해제한다");
                }
            }
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    struct Directory(PathBuf);
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    impl Drop for Directory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).expect("이 검사만 만든 자료 정리");
        }
    }
    fn render_family(renderer: &SkiaLayerRenderer, family: &str) -> Vec<u8> {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        let mut doc = core.document().clone();
        for font in doc.doc_info.font_faces.iter_mut().flatten() {
            font.name = family.into();
            font.alt_type = 1;
            font.native_program = None;
            font.is_embedded = false;
            font.subst_font = None;
        }
        core.set_document(doc);
        core.insert_text_native(0, 0, 0, "A").unwrap();
        let environment = serde_json::json!({
            "id": "fixture", "substitutions": {family: family},
            "trueTypeFaces": [family], "preserveSourceMetrics": false,
        });
        core.set_font_environment(Some(
            FontEnvironment::from_json(&environment.to_string()).unwrap(),
        ))
        .unwrap();
        renderer.prepare_declared_font_fallbacks(&mut core).unwrap();
        renderer.prepare_native_pua_fonts(&mut core).unwrap();
        renderer.prepare_native_otf_fonts(&mut core).unwrap();
        let tree = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        renderer
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    dpi: Some(96.0),
                    ..Default::default()
                },
            )
            .unwrap()
            .bytes
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn render(renderer: &SkiaLayerRenderer) -> Vec<u8> {
        render_family(renderer, FAMILY)
    }
    #[cfg(target_os = "linux")]
    fn xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub(super) fn check_system_program_preference() {
        if std::env::var_os(CHILD).is_some() {
            #[cfg(target_os = "macos")]
            let _fonts = coretext::Fonts::register(&PathBuf::from(
                std::env::var_os(CHILD).expect("현재 자식에만 공급할 글꼴 디렉터리"),
            ));
            let manager = skia_safe::FontMgr::default();
            let names: Vec<_> = manager.family_names().collect();
            #[cfg(target_os = "linux")]
            assert_eq!(
                names,
                vec![FAMILY.to_string()],
                "공개 합성 family 외의 시스템 폰트를 사용하지 않는다"
            );
            #[cfg(target_os = "macos")]
            assert!(
                names.iter().any(|name| name == FAMILY),
                "CoreText 등록을 실제 Skia가 조회한다"
            );
            let chosen = manager
                .match_family_style(FAMILY, skia_safe::FontStyle::normal())
                .unwrap();
            #[cfg(target_os = "linux")]
            assert_eq!(
                chosen.post_script_name().as_deref(),
                Some("RHWPHostFixture-Preferred"),
                "OS가 선택한 프로그램을 독립 확인"
            );
            let chosen_name = chosen
                .post_script_name()
                .expect("선택한 합성 프로그램의 이름");
            assert!(matches!(
                chosen_name.as_str(),
                "RHWPHostFixture-Preferred" | "RHWPHostFixture-Regular"
            ));
            #[cfg(target_os = "linux")]
            let data = {
                let (data, index) = chosen.to_existing_font_data().unwrap();
                assert_eq!(index, 0);
                data.as_bytes().to_vec()
            };
            #[cfg(target_os = "macos")]
            let data = {
                // CoreText의 process face는 파일 스트림을 반환하지 않을 수 있다.
                // 이름만 믿지 않고 실제 cmap·윤곽·폭 테이블을 독립 fixture와 대조한다.
                let source = if chosen_name == "RHWPHostFixture-Preferred" {
                    include_bytes!("../fixtures/fonts/RHWPHostFixture-Preferred.ttf").as_slice()
                } else {
                    include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf").as_slice()
                };
                let face = ttf_parser::Face::parse(source, 0).unwrap();
                assert_eq!(chosen.units_per_em(), Some(i32::from(face.units_per_em())));
                for tag in [b"cmap", b"glyf", b"hmtx"] {
                    let actual = chosen.copy_table_data(u32::from_be_bytes(*tag)).unwrap();
                    let expected = face
                        .raw_face()
                        .table(ttf_parser::Tag::from_bytes(tag))
                        .unwrap();
                    assert_eq!(
                        actual.as_bytes(),
                        expected,
                        "CoreText가 선택한 실제 프로그램 테이블 {tag:?}"
                    );
                }
                source.to_vec()
            };
            let actual = render(&SkiaLayerRenderer::new());
            let expected = render(
                &SkiaLayerRenderer::new()
                    .try_with_font_data(&[data])
                    .unwrap(),
            );
            assert!(
                actual == expected,
                "같은 형식의 다른 TTF가 OS의 프로그램 선택을 덮어쓰지 않는다"
            );
            let other = if chosen_name == "RHWPHostFixture-Preferred" {
                include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf").as_slice()
            } else {
                include_bytes!("../fixtures/fonts/RHWPHostFixture-Preferred.ttf").as_slice()
            };
            assert!(
                actual
                    != render(
                        &SkiaLayerRenderer::new()
                            .try_with_font_data(&[other.to_vec()])
                            .unwrap()
                    ),
                "빈 출력이나 같은 윤곽으로 프로그램 선택 검사가 무력화되지 않는다"
            );
            println!("OS={} selected={chosen_name}; 시스템 출력=해당 프로그램 명시 공급 출력, 다른 프로그램과는 다름", std::env::consts::OS);
            return;
        }
        let directory = Directory(std::env::temp_dir().join(format!(
                "rhwp-program-preference-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        let fonts = directory.0.join("fonts");
        let cache = directory.0.join("cache");
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::create_dir(&cache).unwrap();
        std::fs::write(
            fonts.join("a-other.ttf"),
            include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf"),
        )
        .unwrap();
        let preferred = fonts.join("z-preferred.ttf");
        std::fs::write(
            &preferred,
            include_bytes!("../fixtures/fonts/RHWPHostFixture-Preferred.ttf"),
        )
        .unwrap();
        #[cfg(target_os = "linux")]
        let config = directory.0.join("fonts.conf");
        #[cfg(target_os = "linux")]
        std::fs::write(&config,format!(r#"<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig><dir>{}</dir><cachedir>{}</cachedir><match target="pattern"><test name="family" compare="eq"><string>{FAMILY}</string></test><edit name="file" mode="assign"><string>{}</string></edit></match></fontconfig>"#,xml(&fonts.to_string_lossy()),xml(&cache.to_string_lossy()),xml(&preferred.to_string_lossy()))).unwrap();
        let name = std::thread::current().name().unwrap().to_string();
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, &fonts)
            .env_remove("FONTCONFIG_PATH")
            .env_remove("RHWP_FONT_PATH");
        #[cfg(target_os = "linux")]
        command.env("FONTCONFIG_FILE", config);
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "격리된 Native 프로그램 선택 실패:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn coretext_system_cff_keeps_selected_primary_program_identity() {
        use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
        use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
        use sha2::{Digest, Sha256};
        const MARKER: &str = "RHWP_CORETEXT_PRIMARY_CFF_CHILD";
        const CFF: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
        const NAME: &str = "RHWP HFT Outline Fixture";
        if let Some(path) = std::env::var_os(MARKER) {
            let _fonts = coretext::Fonts::register_paths(&[PathBuf::from(path)]);
            let mut hash = Sha256::new();
            hash.update(CFF);
            hash.update(0u32.to_be_bytes());
            let id: String = hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            fn style(node: &RenderNode) -> Option<&rhwp::renderer::TextStyle> {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    if run.text.contains('가') {
                        return Some(&run.style);
                    }
                }
                node.children.iter().find_map(style)
            }
            let mut positions = Vec::new();
            for bold in [false, true] {
                let mut core = DocumentCore::new_empty();
                core.create_blank_document_native().unwrap();
                let mut doc = core.document().clone();
                doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwp5;
                for font in doc.doc_info.font_faces.iter_mut().flatten() {
                    font.name = NAME.into();
                    font.alt_type = 2;
                    font.native_program = None;
                    font.subst_font = None;
                }
                for shape in &mut doc.doc_info.char_shapes {
                    shape.bold = bold;
                    shape.base_size = 1500;
                    shape.ratios = [100; 7];
                    shape.spacings = [0; 7];
                }
                let supplied = serde_json::json!([{
                    "query": NAME, "family": NAME, "programId": id,
                    "trueType": false, "openTypeCff": true,
                }]);
                let environment =
                    FontEnvironment::from_available_programs(&doc.doc_info, &supplied.to_string())
                        .unwrap()
                        .unwrap();
                core.set_document(doc);
                core.insert_text_native(0, 0, 0, "가가").unwrap();
                core.set_font_environment(Some(environment)).unwrap();
                // 명시 custom 공급으로 시스템/CoreText의 재포장 경계를 우회하지 않는다.
                let renderer = SkiaLayerRenderer::new();
                renderer.prepare_native_otf_fonts(&mut core).unwrap();
                let tree = core.build_page_render_tree(0).unwrap();
                let style = style(&tree.root).unwrap();
                assert!(
                    style.native_otf_metrics.is_some(),
                    "실제 시스템 CFF가 공급됐다"
                );
                positions.push(EmbeddedTextMeasurer.compute_char_positions("가가", style));
            }
            assert_eq!(
                positions[0], positions[1],
                "같은 주 CFF의 CoreText 컨테이너 재포장이 HFT 대체 굵기 전진을 추가하지 않는다"
            );
            return;
        }
        let directory = Directory(std::env::temp_dir().join(format!(
                "rhwp-coretext-cff-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        std::fs::create_dir(&directory.0).unwrap();
        let font = directory.0.join("primary.otf");
        std::fs::write(&font, CFF).unwrap();
        let name = std::thread::current().name().unwrap().to_owned();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(MARKER, font)
            .env_remove("RHWP_FONT_PATH")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "CoreText의 실제 주 프로그램 계약:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(target_os = "windows")]
    pub(super) fn check_system_program_preference() {
        // GDI AddFontResourceEx(FR_PRIVATE)는 DirectWrite의 시스템 collection이 아니다.
        // HKCU/HKLM·사용자 Fonts를 바꾸지 않고 OS의 기본 face를 독립적인 기대값으로 쓴다.
        // https://learn.microsoft.com/en-us/windows/win32/directwrite/custom-font-collections
        let manager = skia_safe::FontMgr::default();
        let chosen = manager
            .legacy_make_typeface(None::<&str>, skia_safe::FontStyle::normal())
            .expect("DirectWrite의 OS 기본 글꼴은 필수이며 없으면 검사 실패");
        let family = chosen.family_name();
        assert!(!family.is_empty());
        let (data, index) = chosen
            .to_existing_font_data()
            .map(|(data, index)| (data.as_bytes().to_vec(), index))
            .or_else(|| chosen.to_font_bytes())
            .expect("DirectWrite의 실제 프로그램과 face index");
        let face = ttf_parser::Face::parse(&data, index).unwrap();
        assert!(
            face.tables().glyf.is_some(),
            "이 검사는 같은 TrueType 형식 선택을 보호한다"
        );
        assert!(face.glyph_index('A').is_some());
        let actual = render_family(&SkiaLayerRenderer::new(), &family);
        let expected = render_family(
            &SkiaLayerRenderer::new()
                .try_with_font_data(&[data])
                .unwrap(),
            &family,
        );
        assert!(
            actual == expected,
            "DirectWrite가 선택한 실제 face의 윤곽을 Native가 그대로 사용한다"
        );
        let fixture = SkiaLayerRenderer::new()
            .try_with_font_data(&[include_bytes!(
                "../fixtures/fonts/RHWPHostFixture-Preferred.ttf"
            )
            .to_vec()])
            .unwrap()
            .with_font_family_alias(&family, "RHWP Host Fixture")
            .unwrap();
        assert!(
            actual != render_family(&fixture, &family),
            "다른 프로그램을 같은 출력으로 세지 않는다"
        );
        println!("OS=windows family={family} face_index={index}; 실제 DirectWrite 선택과 명시 공급 출력 일치");
    }
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn same_format_system_program_keeps_os_preference() {
    system_program_preference::check_system_program_preference();
}
