//! [Issue #6896] 잘린 EMF 를 먼저 잡아 **쓸 수 있는 WMF 미리보기를 안 채워** OLE 개체가
//! 자리표시자로만 남던 결함의 가드.
//!
//! ## 개체 안 구조
//!
//! ```text
//!   BinData/ole5.ole  12,618,756 bytes = [u32 길이 접두] + CFB
//!     \x01CompObj        90
//!     \x01Ole            20
//!     \x02OlePres000     12,076,826     ← 미리보기 (clipFormat 14 = CF_ENHMETAFILE)
//!     CONTENTS           439,387
//! ```
//!
//! `OlePres000` 은 헤더 40바이트 뒤부터 **표준 WMF** 이고(`01 00 09 00 00 03` —
//! mtType=1 · mtHeaderSize=9 · mtVersion=0x0300), 그 WMF 가 EMF 를 ` WMFC` 주석
//! 레코드로 감싼 EMF-in-WMF 형식이다.
//!
//! ## 왜 자리표시자만 남았나
//!
//! `strip_ole_presentation_header` 가 offset 102 의 `EMR_HEADER` 를 잡는다(offset 142 에
//! `" EMF"`). 그런데 그 EMF 는 끝이 잘려 있다.
//!
//! ```text
//!   rec0 type=1  size=108        (EMR_HEADER, nBytes 선언 6,022,292 · nRecords 7)
//!   rec5 type=81 size=6,022,104  (EMR_STRETCHDIBITS)
//!   rec6 type=0xFFFFFFFF          ← 여기서 깨진다
//!   걸은 바이트 6,022,272 / 선언 6,022,292 — 남은 20바이트가 전부 0xFF
//! ```
//!
//! 종전 코드는 `preview_emf` 가 `Some` 이면 `preview_wmf` 를 **아예 안 채웠다**. EMF
//! 파싱이 실패해도 폴백이 비어 있어 렌더가 자리표시자로 끝났다.
//!
//! ## 수정
//!
//! `preview_wmf` 를 조건 없이 채운다. 렌더는 `OOXML 차트 → EMF → WMF → 자리표시자` 순으로
//! 내려가므로, 둘 다 들고 있으면 EMF 실패가 자연히 WMF 로 이어진다.
//!
//! ```text
//!   전   Placeholder kind="ole"  x=100.7 y=541.5 w=602.4 h=844.7
//!   후   RawSvg      pi=59 ci=0  x=100.7 y=541.5 w=602.4 h=844.7   (내부 SVG 406,671B)
//! ```
//!
//! ## 메인터너 보정: 세로 위치도 검증
//!
//! 정본(engine 2020)은 같은 그림을 **y=128.3** 에 둔다. 빈 anchor 한 줄의 저장 높이가
//! TopAndBottom 개체를 포함한다고 오인하면 가운데 정렬에서 413px 아래로 밀린다.
//! 저장 extent가 실제 flow band를 담는 경우에만 신뢰하여, 미리보기와 쪽 안 배치를 함께 잠근다.
//! 위치는 #6912의 셀 윗선 대비 11.53px로 검증한다. 표 전체의 기존 쪽 좌표 차이를
//! 이 개체의 셀 내부 정렬 계약과 섞지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6896/156564340-ip-dispute-mediation.hwpx";
/// OLE 개체가 있는 쪽 (0-based).
const PAGE: u32 = 3;

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()))
}

fn page_tree(page: u32) -> RenderNode {
    let document = HwpDocument::from_bytes(&read(SAMPLE)).expect("문서 로드");
    document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"))
        .root
}

fn collect(node: &RenderNode, svgs: &mut Vec<usize>, placeholders: &mut Vec<String>) {
    match &node.node_type {
        RenderNodeType::RawSvg(raw) => svgs.push(raw.svg.len()),
        RenderNodeType::Placeholder(placeholder) => placeholders.push(placeholder.label.clone()),
        _ => {}
    }
    for child in &node.children {
        collect(child, svgs, placeholders);
    }
}

/// OLE 개체가 미리보기로 그려진다 — 자리표시자로 끝나지 않는다.
#[test]
fn ole_preview_renders_instead_of_a_placeholder() {
    let root = page_tree(PAGE);
    let mut svgs = Vec::new();
    let mut placeholders = Vec::new();
    collect(&root, &mut svgs, &mut placeholders);

    assert!(
        placeholders.is_empty(),
        "OLE 자리표시자가 남았다 — 잘린 EMF 때문에 WMF 폴백에 못 닿는 회귀다. \
         got {placeholders:?}"
    );
    assert_eq!(
        svgs.len(),
        1,
        "OLE 미리보기 RawSvg 가 하나 있어야 한다 — got {svgs:?}"
    );
    // WMF 안 비트맵을 감싼 SVG 라 수만 바이트가 나온다. 빈 조각(<svg/>)이 아님을 잠근다.
    assert!(
        svgs[0] > 10_000,
        "미리보기 SVG 가 사실상 비었다 — {} bytes",
        svgs[0]
    );
}

/// 미리보기 상자는 선언 크기를 지킨다 — 정본 601.9 × 843.7px.
#[test]
fn ole_preview_keeps_the_declared_box() {
    let root = page_tree(PAGE);
    fn find(node: &RenderNode, cell_top: Option<f64>) -> Option<(f64, f64, f64, f64, f64)> {
        let cell_top = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            Some(node.bbox.y)
        } else {
            cell_top
        };
        if matches!(node.node_type, RenderNodeType::RawSvg(_)) {
            return Some((
                node.bbox.x,
                node.bbox.y,
                node.bbox.width,
                node.bbox.height,
                cell_top?,
            ));
        }
        node.children.iter().find_map(|child| find(child, cell_top))
    }
    let (x, y, w, h, cell_top) = find(&root, None).expect("셀 안 OLE RawSvg");
    assert!(
        (x - 100.6).abs() <= 2.0 && (y - cell_top - 11.53).abs() <= 0.5,
        "미리보기의 셀 상대 위치가 정본에서 벗어났다: x={x:.1}, y={y:.1}, cell_top={cell_top:.1}"
    );
    assert!(
        y + h <= root.bbox.y + root.bbox.height,
        "복원한 OLE 미리보기가 용지 아래로 이탈했다"
    );
    assert!(
        (598.0..=606.0).contains(&w) && (840.0..=849.0).contains(&h),
        "미리보기 상자가 정본(601.9 × 843.7)에서 벗어났다 — {w:.1} × {h:.1}"
    );
}
