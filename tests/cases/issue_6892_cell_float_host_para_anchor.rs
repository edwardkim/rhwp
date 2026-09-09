//! [#6892] 칸 안 Square 그림이 **자기 호스트 줄을 두고 칸 첫 줄 자리로** 올라가 글자를 덮는다.
//!
//! `#2226` 은 "글자 없는 문단의 저장 `vpos > 0` 이면 그 줄은 **그림 자신이 밀어낸** 자리다
//! → 앵커를 칸 콘텐츠 상단으로 되돌린다"를 세웠다. `#6192` 가 overlay(글 뒤로/앞으로)를
//! 그 전제에서 뺐고, 이 이슈는 남은 구멍이다 — **앞에 다른 문단이 있으면** 빈 호스트 줄의
//! `vpos` 는 그 문단들이 만든 진짜 흐름 위치다.
//!
//! `156726122`(자원순환 보도자료) **8쪽** 실측:
//!
//! ```text
//!   칸 문단  cp_idx = 5        (앞에 글자 있는 문단 5개)
//!   그림     textWrap=SQUARE  treatAsChar=0  vertRelTo=PARA  vertOffset=0
//!            sz 40038×14908HU = 533.8×198.8px
//!
//!   호스트 빈 줄            382.7
//!   수정 전 그림 y          192.3   ← 칸 콘텐츠 상단(−189.8px), 앞 문단 글자를 덮는다
//!   수정 후 그림 y          382.7
//!   정본(engine 2020)       382.1
//! ```
//!
//! 갈림은 문서가 준다 — **칸의 첫 문단인가**. 문턱도 wrap 열거도 더하지 않는다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6892/156726122-recycling-press-release.hwpx";
/// 공정도 그림이 있는 쪽(0 기준).
const PAGE: u32 = 7;

fn collect_images(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push((node.bbox.y, node.bbox.height));
    }
    for child in &node.children {
        collect_images(child, out);
    }
}

fn page_images(sample: &str, page: u32) -> Vec<(f64, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core = DocumentCore::from_bytes(&fs::read(path).expect("정식 원본")).expect("문서 로드");
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    collect_images(&tree.root, &mut out);
    out
}

#[test]
fn issue_6892_cell_square_float_sits_on_its_host_line() {
    let images = page_images(SAMPLE, PAGE);
    assert_eq!(images.len(), 1, "8쪽 그림은 한 장이다: {images:?}");
    let (y, h) = images[0];

    // 정본 382.1 — 호스트 빈 줄(382.7)과 0.6px 안이다. 종전에는 192.3 이었다.
    assert!(
        (y - 382.7).abs() <= 1.0,
        "그림이 자기 호스트 줄에 있어야 한다: y={y:.1} (기대 382.7, 정본 382.1)"
    );
    assert!(
        (h - 198.8).abs() <= 0.5,
        "그림 높이는 선언 그대로여야 한다: {h:.1}"
    );
}

#[test]
fn issue_6892_first_para_cell_float_keeps_the_cell_top_anchor() {
    // **음성 대조** — `#2226` 이 겨냥한 형상은 그대로 둔다. `pic-in-table-01.hwp` 의 그림은
    // 칸의 **첫 문단**(`cp_idx == 0`)에 매달려 있어 종전 갈래를 계속 타야 한다. 수정 전
    // 바이너리로 잰 값과 같은 자리다.
    let images = page_images("samples/pic-in-table-01.hwp", 0);
    assert_eq!(images.len(), 2, "1쪽 그림은 두 장이다: {images:?}");
    for (actual, expected) in images.iter().zip([(39.7, 36.6), (50.9, 18.4)]) {
        assert!(
            (actual.0 - expected.0).abs() <= 0.5 && (actual.1 - expected.1).abs() <= 0.5,
            "칸 첫 문단 그림은 제자리여야 한다: {actual:?} (기대 {expected:?})"
        );
    }
}
