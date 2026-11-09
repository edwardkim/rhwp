//! 관계형: 도형 그림 채우기 배치 모드의 그림 크기와 「한컴 윤고딕 250」 글자 위치를 독립 한/글 출력에 맞춘다.
//! 주석 작성: 2026-11-04 23:20 KST (새 파일). 측정: 2026-11-04 KST, ai101.
//!
//! # 측정 근거
//!
//! - 방법: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 판정값
//!   `silhouette_raw_match_percent`(색상 경계 보정 전 2px 실루엣). 기준 PDF 는
//!   `pdf/issue8575/shape_image_fill-2020.pdf`(한/글 2020 Print, 2쪽). Native 글꼴은 64MiB 적재
//!   예산(#8597) 때문에 쓰는 face 만 든 폴더를 `--font-path` 맨 앞에 두었다.
//! - 원본 `samples/issue8575/shape_image_fill.hwpx` 전체 2쪽(PDF 2쪽):
//!   수정 후(fa62121230) 1쪽 Native 100.00% · fresh WASM 100.00%, 2쪽 Native 100.00% ·
//!   fresh WASM 100.00%, 전쪽 최저 Native 100.00% · fresh WASM 100.00%.
//!   수정 전(base 140cb02b7f) 1쪽 Native 61.74% · fresh WASM 61.76%, 2쪽 100.00% · 100.00%.
//! - 증거: `mydocs/pr/assets/issue_8575/{native,wasm}/fixture_{base,fix}_silhouette.tsv`,
//!   `mydocs/pr/assets/issue_8575/png/`. 저장본 `shape_image_fill-2020.hwp` 는 같은 HWPX 를
//!   한/글 2020 이 HWP 로 저장한 것이다(같은 PDF 대상, 별도 측정 없음).
//! - 이 검사는 좌표를 고정하지 않고 그림 크기와 상자·그림 픽셀 크기의 관계, 같은 줄 글자 시작
//!   사이 거리만 본다.
//!
//! # 무엇이 깨져 있었나
//!
//! 1. 도형(사각형 등)의 그림 채우기가 배치 모드(가운데로·왼쪽 아래로 등)일 때 그림 크기로
//!    도형의 원래 크기(`shape_attr.original_width/height`)를 썼다. 쪽 크기로 원래 크기가
//!    저장된 배경 도형은 그림이 용지만큼 부풀어 잘렸고, 원래 크기가 상자보다 큰 도형은
//!    그림이 상자 밖으로 넘쳐 잘렸다.
//! 2. 「한컴 윤고딕 250」 이름이 같은 메트릭 표(`Haan YGodic 250`)와 설치 face 에 연결되지
//!    않아, 글꼴 파일이 없을 때 글자 전진폭이 대체 글꼴로 계산됐다(230·240 은 연결돼 있었다).
//!
//! # 기대값의 출처
//!
//! 공개 합성 입력 `samples/issue8575/shape_image_fill.hwpx`(생성기 `gen_shape_image_fill.py`)의
//! 한/글 2020 Print PDF `pdf/issue8575/shape_image_fill-2020.pdf`(96dpi 래스터 측정):
//! - 그림은 240x160px PNG(pHYs 300dpi). 사례 A(320x240 상자)는 240x160 으로 가운데에 놓인다 —
//!   그림의 DPI 정보는 쓰지 않는다.
//! - 사례 B(160x240)·E(160x240, 가운데 아래로)는 가로만 상자 폭으로 줄고 세로는 160px,
//!   사례 C(320x100)는 세로만 100px 로 줄고 가로는 240px, 사례 D(120x80)는 상자에 맞는다.
//! - 사례 F(현재 320x240, 원래 640x480)는 도형 배율과 무관하게 240x160 이다.
//! - 2쪽(22pt 한컴 윤고딕 250, PDF 내장 HaanYGodic25) 낱말 시작 사이 거리: 「2」→「9」 83.07pt,
//!   →「윤」 151.96pt, →「한」 283.29pt. 실폰트 HANYGO250.ttf 전진폭과 같다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::style::ImageFillMode;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8575/shape_image_fill.hwpx";
const SAVED_HWP: &str = "samples/issue8575/shape_image_fill-2020.hwp";
/// 채움 그림의 픽셀 크기(1px = 1/96in).
const NATIVE: (f64, f64) = (240.0, 160.0);
const PT_TO_PX: f64 = 96.0 / 72.0;

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("재현물 {sample} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// (상자, 채움 모드, 그림 크기) — 도형 그림 채우기 노드.
fn image_fills(node: &RenderNode, out: &mut Vec<(BoundingBox, ImageFillMode, (f64, f64))>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if let (Some(mode), Some(size)) = (image.fill_mode, image.original_size) {
            out.push((node.bbox, mode, size));
        }
    }
    for child in &node.children {
        image_fills(child, out);
    }
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 0.5
}

fn assert_placement_sizes(sample: &str) {
    let doc = load(sample);
    assert_eq!(doc.page_count(), 2, "{sample}: 한/글 Print 는 2쪽");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut fills = Vec::new();
    image_fills(&tree.root, &mut fills);
    assert_eq!(
        fills.len(),
        6,
        "{sample}: 사례 A~F 의 채움 그림이 하나씩 있다: {fills:?}"
    );
    fills.sort_by(|a, b| (a.0.y, a.0.x).partial_cmp(&(b.0.y, b.0.x)).unwrap());
    for (bbox, mode, (w, h)) in &fills {
        assert!(
            matches!(mode, ImageFillMode::Center | ImageFillMode::CenterBottom),
            "{sample}: 배치 모드만 쓴다: {mode:?}"
        );
        // 축마다 그림 픽셀 크기와 상자 중 작은 쪽 — 도형의 원래 크기·배율과 무관하다.
        let expected = (NATIVE.0.min(bbox.width), NATIVE.1.min(bbox.height));
        assert!(
            close(*w, expected.0) && close(*h, expected.1),
            "{sample}: 상자 {:.0}x{:.0} 의 그림 {w:.1}x{h:.1}, 기대 {:.0}x{:.0}",
            bbox.width,
            bbox.height,
            expected.0,
            expected.1
        );
    }
    // 한 축만 넘치는 사례가 실제로 있어 축별 판정이 검사된다.
    assert!(fills
        .iter()
        .any(|(b, _, (w, h))| close(*w, b.width) && close(*h, NATIVE.1) && b.height > NATIVE.1));
    assert!(fills
        .iter()
        .any(|(b, _, (w, h))| close(*h, b.height) && close(*w, NATIVE.0) && b.width > NATIVE.0));
}

#[test]
fn placement_fill_uses_image_pixels_clamped_per_axis() {
    assert_placement_sizes(SAMPLE);
}

#[test]
fn hancom_saved_hwp_keeps_the_same_placement_sizes() {
    assert_placement_sizes(SAVED_HWP);
}

/// 2쪽 SVG 의 보이는 글자별 x 시작(문서 순서).
fn glyph_starts(doc: &DocumentCore) -> Vec<(char, f64)> {
    let svg = doc.render_page_svg_native(1).expect("2쪽 SVG");
    let mut out = Vec::new();
    for piece in svg.split("<text ").skip(1) {
        let Some(x) = piece
            .strip_prefix("x=\"")
            .and_then(|rest| rest.split('"').next())
            .and_then(|x| x.parse::<f64>().ok())
        else {
            continue;
        };
        let Some(text) = piece
            .split_once('>')
            .and_then(|(_, rest)| rest.split_once("</text>"))
            .map(|(text, _)| text)
        else {
            continue;
        };
        let mut chars = text.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            out.push((ch, x));
        }
    }
    out
}

#[test]
fn ygodic250_line_uses_its_own_advances() {
    let doc = load(SAMPLE);
    let glyphs = glyph_starts(&doc);
    let start = |ch: char| {
        glyphs
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, x)| *x)
            .unwrap_or_else(|| panic!("2쪽에 「{ch}」가 없다: {glyphs:?}"))
    };
    let origin = start('2');
    // 한/글 Print 낱말 시작 사이 거리(pt): 「2021.」→「9.」 83.07, →「윤고딕」 151.96, →「한글」 283.29.
    for (ch, pt) in [('9', 83.07), ('윤', 151.96), ('한', 283.29)] {
        let actual = start(ch) - origin;
        let expected = pt * PT_TO_PX;
        assert!(
            (actual - expected).abs() <= 1.5,
            "「2」→「{ch}」 {actual:.2}px, 한/글 Print {expected:.2}px"
        );
    }
}
