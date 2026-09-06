//! [Issue #6782] 조각 표의 셀 앵커 그림에서, **그림을 자기 칸 밖으로 통째로 밀어내는**
//! 세로 오프셋이 그대로 실려 그림이 쪽 위쪽 밖(음수 y)으로 나가 소실되던 결함의 가드.
//!
//! ## 계약 — 부호도 크기도 아니라 **칸과 겹치는가**
//!
//! 한/글은 셀 앵커 그림의 세로 오프셋을 대체로 그대로 적용하지만, 그 결과가 그림을 자기
//! 칸 밖으로 완전히 내보내면 쓰지 않는다. 한/글 2020 오라클 실측이 그 축을 가른다.
//!
//! ```text
//!   문제 그림 (h≈65.7)   한/글 y = 235.9     rhwp 종전 y = −233.4  ← 용지 밖, 소실
//!                                            이 수정   y =  238.7  (한/글과 2.8px)
//!
//!   voff −70,819HU = −944.25px, 셀 valign=Center
//!   232.1 + (79.05 − 65.8 − 944.25) / 2 = −233.4
//! ```
//!
//! ⚠ **음수라고 버리면 안 된다.** `#5734`(156684746 9쪽 왼쪽 칸)의 첫 그림도 저장 vpos 가
//! 0이라 같은 폴백 갈래로 오는데, 거기서는 `−1,079HU`(14.4px)가 **적용되는 것이 정답**이고
//! `issue_5734_cell_float_stack_stored_vpos` 가 그 값을 잠근다. 실제로 「음수 → 0」 판과
//! 「결과 바닥을 칸 상단으로」 판은 **둘 다 그 핀을 깨뜨렸다**(전수 9020 중 그 1건).
//!
//! ```text
//!   #6782  y + h = −167.6 ≤ 칸 상단 232.1   → 칸 밖   → 오프셋 무시
//!   #5734  y + h =  701.1 >  칸 상단 631.0   → 겹침    → 오프셋 적용
//! ```
//!
//! ## 재현물이 코퍼스에 있는 이유
//!
//! `extract-pages` 로 두 쪽만 잘라도 **5.1MB** 다 — BinData(그림)가 걷히지 않아 범위와
//! 무관하게 크기가 거의 그대로다(76~77쪽 5,104KB · 77쪽만 5,103KB). 저장소 fixture
//! (`issue6718` 54KB · `issue6697` 322KB)로 담기에 과해서 `issue_6599` 와 같은 방식으로
//! 코퍼스에서 찾고 없으면 건너뛴다. `RHWP_ISSUE6782_SAMPLE` 로 경로를 덮어쓸 수 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// 0-based — 문제 그림이 있는 물리 77쪽(인쇄 쪽번호 56).
const PAGE_INDEX: u32 = 76;
const PAGE_HEIGHT_PX: f64 = 1122.5;

fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE6782_SAMPLE") {
        return std::fs::read(path).ok();
    }
    for base in [
        concat!(r"C:\Users\planet\hwpdocs_10k_share", r"\prism_downloads"),
        concat!(r"D:\hwpdocs_10k_share", r"\prism_downloads"),
    ] {
        for path in walk(std::path::Path::new(base), 0) {
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("1480000-201900042") && name.ends_with(".hwp"))
            {
                return std::fs::read(path).ok();
            }
        }
    }
    None
}

/// 코퍼스는 부처별 하위 폴더로 나뉜다 — 두 단계까지만 훑는다.
fn walk(dir: &std::path::Path, depth: usize) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if depth > 2 {
        return out;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path, depth + 1));
        } else {
            out.push(path);
        }
    }
    out
}

fn collect_cell_images<'a>(
    node: &'a RenderNode,
    cell: Option<&'a RenderNode>,
    out: &mut Vec<(f64, f64, f64)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        if let Some(cell) = cell {
            out.push((cell.bbox.y, node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        collect_cell_images(child, cell, out);
    }
}

#[test]
fn offset_that_pushes_a_cell_image_out_of_its_cell_is_not_applied() {
    let Some(bytes) = sample() else {
        eprintln!("코퍼스 재현물 없음 — 건너뛴다 (RHWP_ISSUE6782_SAMPLE 로 지정 가능)");
        return;
    };
    let document = HwpDocument::from_bytes(&bytes).expect("parse 1480000-201900042");
    assert_eq!(document.page_count(), 104, "쪽수는 104쪽이어야 한다");

    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p77");
    let mut images = Vec::new();
    collect_cell_images(&tree.root, None, &mut images);

    assert!(
        images.len() >= 10,
        "77쪽 표에 칸 안 그림이 10장 이상이어야 한다 — 표본이 어긋났다. got {}",
        images.len()
    );

    for (cell_y, image_y, image_h) in &images {
        assert!(
            *image_y >= 0.0 && *image_y <= PAGE_HEIGHT_PX,
            "칸 안 그림이 용지 밖으로 나갔다 — 그림 y={image_y:.1}px (용지 0..{PAGE_HEIGHT_PX:.1}). \
             회귀 시 −233.4px 로 인쇄에서 소실된다"
        );
        // 오프셋이 살아 있어도 그림은 자기 칸과 **겹쳐야** 한다. 한/글은 칸 밖으로
        // 통째로 내보내는 오프셋을 쓰지 않는다.
        assert!(
            *image_y + *image_h > *cell_y,
            "칸 안 그림이 자기 칸 위로 통째로 벗어났다 — 칸 {cell_y:.1}px, \
             그림 {image_y:.1}+{image_h:.1}px"
        );
    }
}
