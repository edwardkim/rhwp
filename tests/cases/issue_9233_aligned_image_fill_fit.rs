//! 관계형·고정형 — [#9233] 정렬 배치 그림 채우기는 영역보다 큰 축을 영역 크기로 줄여 그린다.
//! 주석 갱신: 2026-11-20 18:35 KST (새 파일)
//!
//! 입력: 서울시교육청 공개 문서(「2018학년도 후기고등학교 입학전 전학 시행 계획(안)」, HWP5,
//! 한컴오피스 2010 저장, 22쪽) `samples/issue9233/high-school-transfer-plan-cover-band.hwp`.
//! 기준: 한컴 Print PDF `pdf/issue9233/high-school-transfer-plan-cover-band-2020.pdf`(22쪽).
//! 1쪽 표지 표(3×1)의 첫 행·셋째 행 칸은 글 없이 칸 배경 그림 채우기만 가진다 — 칸
//! 46775×2596HU(623.7×34.6px), 그림 344×131px, 유형 `LeftTop`(첫 행)·`RightBottom`(셋째 행).
//! 한/글은 그림을 폭 344px 그대로 두고 높이만 칸 높이로 줄여 곡선 띠 전체를 그린다(한컴 PDF
//! 96dpi 래스터 대비 평균 절대 오차: 줄여 그리기 3.7, 잘라 내기 44.4). 수정 전 rhwp 는 그림을
//! 344×131px 로 놓고 칸 밖을 잘라 그림 위쪽(아래쪽) 34.6px 만 보였다.
//! 같은 쪽의 도형 채우기(`Center`, 영역 209.8×214.2px, 그림 200×200px)는 그림이 영역보다 작은
//! 반례다 — 줄이지 않고 원래 크기로 가운데에 놓는다.
//! - 고정형: 전체 쪽수 22(= PDF), 1쪽의 정렬 배치 채우기 그림 3개.
//! - 관계형: 표지 띠 두 그림은 자기 clip(칸) 사각형 안에 들어 있고, 높이는 칸 높이와 같으며 폭은
//!   그림 픽셀 폭(칸보다 좁음) 그대로다. 첫 그림은 칸 왼쪽, 둘째 그림은 칸 오른쪽에 붙는다. 도형
//!   채우기 그림은 영역보다 작은 원래 크기로 영역 가운데에 있다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `0a437a7eef`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 22쪽 / 22쪽, 1쪽 94.14(base 85.91), 전쪽 최저 91.23(16쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 22쪽 / 22쪽, 1쪽 91.94, 전쪽 최저 91.15(16쪽)
//! - 증거: `mydocs/pr/assets/issue_9233/`(native·wasm 전쪽 TSV, 1쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::{DocumentCore, PrintJobContext};
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use std::path::Path;

const SAMPLE: &str = "samples/issue9233/high-school-transfer-plan-cover-band.hwp";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 22;
/// 칸 배경 그림(BinData 2·3)의 픽셀 폭. 칸 폭(623.7px)보다 좁다.
const IMAGE_WIDTH_PX: f64 = 344.0;

/// `name="값"` 속성 값을 실수로 읽는다.
fn attr(tag: &str, name: &str) -> f64 {
    let key = format!(" {name}=\"");
    let start = tag
        .find(&key)
        .unwrap_or_else(|| panic!("{name} 속성 없음: {tag}"))
        + key.len();
    let end = tag[start..].find('"').expect("속성 끝") + start;
    tag[start..end]
        .parse()
        .unwrap_or_else(|e| panic!("{name} 값 해석 실패({e}): {tag}"))
}

/// 채우기 clip 사각형과 그 안에 그린 그림 사각형 `(x, y, width, height)` 쌍.
fn clipped_fill_images(svg: &str) -> Vec<([f64; 4], [f64; 4])> {
    let rect = |tag: &str| {
        [
            attr(tag, "x"),
            attr(tag, "y"),
            attr(tag, "width"),
            attr(tag, "height"),
        ]
    };
    let mut pairs = Vec::new();
    let mut rest = svg;
    while let Some(at) = rest.find("<g clip-path=\"url(#fill-clip-") {
        rest = &rest[at..];
        let id_start = "<g clip-path=\"url(#".len();
        let id_end = rest.find(")\"").expect("clip 참조 끝");
        let id = &rest[id_start..id_end];
        let image_at = rest.find("<image ").expect("clip 안의 그림");
        // 큰 data URI 앞의 기하 속성만 읽는다.
        let image_tag_end = rest[image_at..].find(" href=").expect("그림 href") + image_at;
        let image = rect(&rest[image_at..image_tag_end]);
        let clip_key = format!("<clipPath id=\"{id}\"><rect ");
        let clip_at = svg.find(&clip_key).expect("clip 정의") + clip_key.len() - "<rect ".len();
        let clip_end = svg[clip_at..].find("/>").expect("clip rect 끝") + clip_at;
        pairs.push((rect(&svg[clip_at..clip_end]), image));
        rest = &rest[image_at..];
    }
    pairs
}

#[test]
fn aligned_image_fill_shrinks_the_overflowing_axis_to_the_area() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let svg = core.render_page_svg_native(0).expect("1쪽 SVG");
    let fills = clipped_fill_images(&svg);
    assert_eq!(fills.len(), 3, "1쪽의 정렬 배치 채우기 그림은 3개여야 한다");
    for (index, (clip, image)) in fills.iter().take(2).enumerate() {
        let [cx, cy, cw, ch] = *clip;
        let [ix, iy, iw, ih] = *image;
        assert!(
            ix + 0.01 >= cx && iy + 0.01 >= cy && ix + iw <= cx + cw + 0.01 && iy + ih <= cy + ch + 0.01,
            "그림 {index} 사각형 ({ix:.2}, {iy:.2}, {iw:.2}, {ih:.2}) 이 칸 ({cx:.2}, {cy:.2}, {cw:.2}, {ch:.2}) 밖으로 나간다"
        );
        assert!(
            (ih - ch).abs() < 0.01,
            "그림 {index} 높이 {ih:.2} 는 칸 높이 {ch:.2} 로 줄어야 한다"
        );
        assert!(
            (iw - IMAGE_WIDTH_PX).abs() < 0.01 && iw < cw,
            "그림 {index} 폭 {iw:.2} 는 그림 픽셀 폭 {IMAGE_WIDTH_PX} 그대로여야 한다(칸 폭 {cw:.2})"
        );
    }
    let ([cx, _, _, _], [ix, _, _, _]) = fills[0];
    assert!(
        (ix - cx).abs() < 0.01,
        "첫 그림(LeftTop)은 칸 왼쪽에 붙어야 한다: 그림 {ix:.2}, 칸 {cx:.2}"
    );
    let ([cx, _, cw, _], [ix, _, iw, _]) = fills[1];
    assert!(
        (ix + iw - (cx + cw)).abs() < 0.01,
        "둘째 그림(RightBottom)은 칸 오른쪽에 붙어야 한다: 그림 끝 {:.2}, 칸 끝 {:.2}",
        ix + iw,
        cx + cw
    );

    // 반례: 영역보다 작은 그림은 줄이지 않고 가운데에 놓는다.
    let ([cx, cy, cw, ch], [ix, iy, iw, ih]) = fills[2];
    assert!(
        iw < cw && ih < ch,
        "도형 채우기 그림 {iw:.2}×{ih:.2} 은 영역 {cw:.2}×{ch:.2} 보다 작아야 한다"
    );
    assert!(
        ((ix - cx) - (cx + cw - ix - iw)).abs() < 0.01
            && ((iy - cy) - (cy + ch - iy - ih)).abs() < 0.01,
        "도형 채우기 그림은 영역 가운데에 있어야 한다: 그림 ({ix:.2}, {iy:.2}, {iw:.2}, {ih:.2}), 영역 ({cx:.2}, {cy:.2}, {cw:.2}, {ch:.2})"
    );
    assert!(
        (iw - ih).abs() < 0.01,
        "도형 채우기 그림은 원래 종횡비(정사각형)를 유지해야 한다: {iw:.2}×{ih:.2}"
    );
}

/// 고정형 좌표와 관계 검사. 한컴 2024 Print 2쪽의 96 DPI 기준이며,
/// Native·fresh WASM raw 모두 100.00%다. 2쪽은 문서의 쪽 번호 재시작으로 '1쪽'이다.
#[test]
fn automatic_footer_follows_the_print_job_without_changing_the_document() {
    const FILE_NAME: &str = "hwp-no-scripts-5b26f65dc7954a6da84dac2a1373c8ee.hwp";
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue9233/high-school-transfer-plan-cover-band.hwp"
    ))
    .expect("인쇄 fixture 열기");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), PDF_PAGES);

    let page = 1;
    let original = core.export_hwp_native().expect("원본 저장 사본");
    let render = |core: &DocumentCore, profile| {
        core.render_page_svg_layer_with_profile_native(page, profile)
            .expect("2쪽 SVG")
    };
    let original_print = render(&core, RenderProfile::Print);
    let original_screen = render(&core, RenderProfile::Screen);
    let job = PrintJobContext::from_json(&format!(
        r#"{{"fileName":"{FILE_NAME}","printedAt":"2026-11-20T20:57:56+09:00"}}"#
    ))
    .expect("수용한 독립 Print의 작업 정보");
    core.set_print_job_context(Some(job));

    let expected = [
        (FILE_NAME, 75.52),
        ("2026-11-20 오후 8:57", 330.24),
        ("22쪽 중 1쪽", 652.96),
    ];
    let layer = core
        .build_page_layer_tree_with_profile(page, RenderProfile::Print)
        .expect("2쪽 인쇄 레이어");
    fn text_positions(node: &LayerNode, expected: &str, positions: &mut Vec<(f64, f64)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    text_positions(child, expected, positions);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                text_positions(child, expected, positions);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    match op {
                        PaintOp::TextRun { bbox, run, .. } if run.text == expected => {
                            positions.push((bbox.x, bbox.y + run.baseline));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    for (text, reference_x) in expected {
        let mut positions = Vec::new();
        text_positions(&layer.root, text, &mut positions);
        assert_eq!(positions.len(), 1, "자동 바닥글 항목 소유: {text}");
        let (x, baseline) = positions[0];
        assert!((x - reference_x).abs() <= 1.0, "{text} 시작 x={x}");
        assert!(
            (baseline - 1095.04).abs() <= 1.0,
            "{text} 기준선 y={baseline}"
        );
    }
    assert_eq!(render(&core, RenderProfile::Screen), original_screen);
    assert_eq!(
        core.export_hwp_native().expect("인쇄 중 저장 사본"),
        original
    );

    core.set_print_job_context(None);
    assert_eq!(render(&core, RenderProfile::Print), original_print);
}
