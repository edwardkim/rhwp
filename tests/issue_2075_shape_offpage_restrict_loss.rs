//! Issue #2075: floating 도형(shape) off-page 좌표 완전 소실 — restrictInPage 클램프 부재.
//!
//! 배경: PR #2033(이슈 #2032)에서 그림(layout_body_picture)에는 restrictInPage
//! (flow_with_text, HWP5 attr bit 13) 하단 클램프를 넣었으나, 도형 경로
//! (`src/renderer/layout/shape_layout.rs`)에는 동일 클램프가 빠져 있다. 주석에
//! "layout_body_picture와 동일 로직"이라 적혀 있지만 그 로직이 갱신되지 않았다.
//! → vert=Para + 큰 vertOffset + restrictInPage=on 인 floating 도형이 페이지 캔버스
//!   밖 좌표로 계산되면 어느 페이지에서도 그려지지 않는다(완전 소실).
//!
//! 한컴 실제 동작(#2032 그림과 동일): restrictInPage=on 이면 개체를 쪽 영역 안으로
//! 끌어들여 소실이 없다. restrictInPage=off 는 쪽 영역 이탈 허용.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/mix-shape-01.hwp";
/// mix-shape-01 의 사각형(control 3, para 0) — 유일한 Rectangle.
const RECT_PARA: usize = 0;
const RECT_CTRL: usize = 3;
/// A4 세로 용지 높이(HWPUNIT).
const A4_HEIGHT_HU: i32 = 84188;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn collect_rects(node: &RenderNode, out: &mut Vec<Rect>) {
    if let RenderNodeType::Rectangle(_) = &node.node_type {
        out.push(Rect {
            x: node.bbox.x,
            y: node.bbox.y,
            w: node.bbox.width,
            h: node.bbox.height,
        });
    }
    for c in &node.children {
        collect_rects(c, out);
    }
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("load {SAMPLE}: {e}"))
}

fn set_shape(core: &mut DocumentCore, json: &str) {
    core.set_shape_properties_native(0, RECT_PARA, RECT_CTRL, json)
        .unwrap_or_else(|e| panic!("set_shape_properties_native({json}): {e}"));
}

/// 전 페이지에서 유일한 Rectangle 노드를 찾아 (페이지, 페이지 wh, Rect) 반환.
fn find_rect(core: &mut DocumentCore) -> Option<(u32, (f64, f64), Rect)> {
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree page {page}: {e}"));
        let mut rects = Vec::new();
        collect_rects(&tree.root, &mut rects);
        if let Some(r) = rects.first() {
            return Some((page, (tree.root.bbox.width, tree.root.bbox.height), *r));
        }
    }
    None
}

/// restrictInPage=on + 페이지 높이를 넘는 vertOffset:
/// 도형이 쪽 영역 안으로 끌려와 페이지에 보여야 한다(완전 소실 금지).
#[test]
fn shape_restrict_on_offpage_offset_stays_in_page() {
    let mut core = load();
    set_shape(
        &mut core,
        r#"{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","vertAlign":"Top","vertOffset":90000,"restrictInPage":true}"#,
    );
    let (page, (pw, ph), r) = find_rect(&mut core).unwrap_or_else(|| {
        panic!("restrictInPage=on 인데 Rectangle 노드가 어느 페이지에도 없음(소실)")
    });
    let visible = r.x < pw && r.x + r.w > 0.0 && r.y < ph && r.y + r.h > 0.0;
    assert!(
        visible,
        "restrictInPage=on 도형이 페이지 {page} 캔버스({pw:.0}x{ph:.0}) 밖으로 소실: {r:?}"
    );
    assert!(
        r.y + r.h <= ph + 0.5,
        "restrictInPage=on 도형 하단이 페이지 하단 초과(클램프 미적용): bottom={:.1} ph={ph:.1}",
        r.y + r.h
    );
}

/// restrictInPage=on + 하단 걸침: 도형 전체가 쪽 영역 안으로 들어와야 한다.
#[test]
fn shape_restrict_on_partial_overflow_clamped() {
    let mut core = load();
    let offset = A4_HEIGHT_HU - 5000; // 하단 근처 앵커 + 도형 높이로 하단 초과
    set_shape(
        &mut core,
        &format!(
            r#"{{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","vertAlign":"Top","vertOffset":{offset},"restrictInPage":true}}"#
        ),
    );
    let (_, (_, ph), r) = find_rect(&mut core)
        .unwrap_or_else(|| panic!("restrictInPage=on 인데 Rectangle 노드 소실"));
    assert!(
        r.y + r.h <= ph + 0.5,
        "restrictInPage=on 도형 하단이 페이지 하단 초과: bottom={:.1} ph={ph:.1}",
        r.y + r.h
    );
}

/// restrictInPage=off + 페이지 높이를 넘는 vertOffset:
/// 한컴 정합 — 클램프 없이 쪽 영역을 벗어날 수 있어야 한다(과잉 클램프 방지).
/// 렌더가 emit 되면 좌표는 원값(페이지 하단 아래)이어야 한다.
#[test]
fn shape_restrict_off_offpage_offset_not_clamped() {
    let mut core = load();
    set_shape(
        &mut core,
        r#"{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","vertAlign":"Top","vertOffset":90000,"restrictInPage":false}"#,
    );
    if let Some((page, (_, ph), r)) = find_rect(&mut core) {
        assert!(
            r.y + r.h > ph,
            "restrictInPage=off 인데 도형이 페이지 {page} 안으로 클램프됨(off 는 쪽 영역 제한이 없어야 함): {r:?}"
        );
    }
}
