//! 칸 배경 그림 채우기 `None`(이진 채우기 유형 15)은 칸에 맞춰 축소해 가운데 놓는다 (#7235).
//!
//! # 결함
//!
//! `render_image_node`(`renderer/svg.rs`)의 채우기 유형 분기에 `ImageFillMode::None` 팔이
//! 없어 `_` 의 **배치 모드**로 떨어졌다. 배치 모드는 그림을 **원본 픽셀 크기**로 상자
//! 왼쪽 위에 놓고 상자 clip 으로 자른다. 그래서 253.4x57.1px 칸에 1628x563 로고가
//! 왼쪽 위 기준으로 놓여 칸에는 로고의 흰 여백만 남고 로고가 사라졌다.
//!
//! 같은 `None` 을 쪽 배경 경로(`render_page_background_image`)는 늘려 채우기로 묶어 두었다 —
//! 한 의미를 두 경로가 다르게 처리하고 있었다.
//!
//! # 기대값의 근거 (구현과 독립)
//!
//! 한/글 출력에서 이 칸의 로고는 **칸에 맞춰 종횡비를 지키며 축소되어 가운데** 그려진다.
//! `#7235` 본문의 실측은 가로 511~676px 이다. 칸 상자(x=466.613, w=253.373, h=57.107)와
//! 원본 종횡비 1628/563 로 "비율 유지 축소 + 가운데" 를 계산하면
//!
//! ```text
//!   폭   = 57.107 x (1628/563) = 165.16
//!   왼쪽 = 466.613 + (253.373 - 165.16)/2 = 510.72
//!   오른쪽 = 675.88
//! ```
//!
//! 세 수가 모두 정답지와 맞는다. 늘려 채우기라면 466.61~720.0(폭 253.37)이 되어 어긋난다.
//! 즉 `None` 은 `Zoom`(HWPX `imgBrush mode="ZOOM"`, `#6310`)과 같은 결과다.
//!
//! # 입력
//!
//! `samples/issue7235/156467175_press_release_header_logo_p1.hwp` — 원본
//! `156467175_210823(조간) 대구염색공단 발주 전기통신설비공사 입찰담합 제재.hwp`(코퍼스,
//! 6쪽 10.3MB)에서 `rhwp extract-pages --from 1 --to 1` 로 1쪽만 남긴 발췌본이다
//! (233KB, BinData 7 -> 3). 발췌 전후 모두 같은 칸 상자(466.613, 100.267, 253.373x57.107)와
//! 같은 결함(1628x563 배치)을 내는 것을 확인했다.
//!
//! 이 검사는 **그려지는 사각형**을 본다. render tree 의 bbox 는 수정 전에도 칸 상자였고
//! (결함은 paint 단계에 있다) 그래서 트리 단언으로는 이 결함이 잡히지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue7235/156467175_press_release_header_logo_p1.hwp";

/// 칸 상자 — 수정 전후 불변이며 `export-render-tree` 의 Image 노드 bbox 와 같다.
const CELL_X: f64 = 466.613;
const CELL_Y: f64 = 100.267;
const CELL_W: f64 = 253.373;
const CELL_H: f64 = 57.107;

/// 원본 로고 픽셀 크기.
const IMG_W: f64 = 1628.0;
const IMG_H: f64 = 563.0;

fn page_svg() -> String {
    let bytes = std::fs::read(SAMPLE).expect("fixture 읽기 실패");
    let doc = DocumentCore::from_bytes(&bytes).expect("문서 로드 실패");
    doc.render_page_svg_native(0).expect("1쪽 SVG 렌더 실패")
}

/// `<image ...>` 태그에서 x·y·width·height·preserveAspectRatio 를 뽑는다.
fn image_tags(svg: &str) -> Vec<(f64, f64, f64, f64, String)> {
    let mut out = Vec::new();
    for tag in svg.split("<image ").skip(1) {
        let head = &tag[..tag.find('>').unwrap_or(tag.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let rest = head.split(&key).nth(1)?;
            rest[..rest.find('"')?].parse::<f64>().ok()
        };
        let par = head
            .split("preserveAspectRatio=\"")
            .nth(1)
            .and_then(|r| r.find('"').map(|e| r[..e].to_string()))
            .unwrap_or_default();
        if let (Some(x), Some(y), Some(w), Some(h)) =
            (attr("x"), attr("y"), attr("width"), attr("height"))
        {
            out.push((x, y, w, h, par));
        }
    }
    out
}

fn cell_fill_image(svg: &str) -> (f64, f64, f64, f64, String) {
    let found: Vec<_> = image_tags(svg)
        .into_iter()
        .filter(|(x, y, _, _, _)| (x - CELL_X).abs() < 1.0 && (y - CELL_Y).abs() < 1.0)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "칸 원점({CELL_X}, {CELL_Y})에서 시작하는 <image> 가 1개여야 한다: {found:?}"
    );
    found.into_iter().next().unwrap()
}

#[test]
fn issue_7235_cell_image_fill_none_is_not_drawn_at_original_size() {
    let svg = page_svg();
    let (_, _, w, h, _) = cell_fill_image(&svg);
    // 수정 전: 1628x563 (원본 픽셀 크기). 이 단언이 결함을 드러낸다.
    assert!(
        (w - IMG_W).abs() > 1.0 && (h - IMG_H).abs() > 1.0,
        "칸 채우기 그림이 원본 픽셀 크기로 그려졌다: {w} x {h}"
    );
    // 배치 모드는 상자 clip 을 만든다 — 그 흔적도 없어야 한다.
    assert!(
        !svg.contains("fill-clip"),
        "칸 채우기가 여전히 배치 모드(fill-clip)로 그려진다"
    );
}

#[test]
fn issue_7235_cell_image_fill_none_contains_and_centers() {
    let svg = page_svg();
    let (x, y, w, h, par) = cell_fill_image(&svg);
    assert_eq!(
        par, "xMidYMid meet",
        "칸 채우기는 종횡비를 지키며 영역에 맞춰야 한다"
    );
    // 상자는 칸이고, `meet` 이 그 안에서 축소·가운데를 맡는다.
    assert!((x - CELL_X).abs() < 0.01, "x={x}");
    assert!((y - CELL_Y).abs() < 0.01, "y={y}");
    assert!((w - CELL_W).abs() < 0.01, "width={w}");
    assert!((h - CELL_H).abs() < 0.01, "height={h}");
}

#[test]
fn issue_7235_drawn_logo_matches_hancom_geometry() {
    let svg = page_svg();
    let (x, y, w, h, par) = cell_fill_image(&svg);
    assert_eq!(par, "xMidYMid meet");
    // `xMidYMid meet` 의 결과를 직접 계산해 한/글 실측(가로 511~676px)과 대조한다.
    let scale = (w / IMG_W).min(h / IMG_H);
    let drawn_w = IMG_W * scale;
    let drawn_h = IMG_H * scale;
    let left = x + (w - drawn_w) / 2.0;
    let right = left + drawn_w;
    assert!((drawn_w - 165.16).abs() < 0.05, "그려지는 폭={drawn_w}");
    assert!((left - 510.72).abs() < 0.05, "왼쪽={left}");
    assert!((right - 675.88).abs() < 0.05, "오른쪽={right}");
    // 높이는 칸을 꽉 채운다(칸이 더 납작하므로 높이가 제한 축이다).
    assert!((drawn_h - CELL_H).abs() < 0.01, "그려지는 높이={drawn_h}");
    // 한/글 실측 구간(511~676) 안에 있다.
    assert!(left > 510.0 && right < 677.0, "{left}..{right}");
}

#[test]
fn issue_7235_other_images_on_the_page_are_untouched() {
    let svg = page_svg();
    // 같은 쪽의 일반 그림(문단 그림, 77.47/105.46 에 132.41x126.72)은 이 수정 대상이 아니다.
    let others: Vec<_> = image_tags(&svg)
        .into_iter()
        .filter(|(x, y, _, _, _)| (x - CELL_X).abs() >= 1.0 || (y - CELL_Y).abs() >= 1.0)
        .collect();
    assert!(
        !others.is_empty(),
        "대조군 그림이 사라졌다 — 이 수정은 칸 채우기만 바꾼다"
    );
    for (x, y, w, h, par) in others {
        assert_eq!(
            par, "none",
            "대조군 그림의 채우기 방식이 바뀌었다: ({x}, {y}) {w}x{h} par={par}"
        );
    }
}
