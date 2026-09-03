//! [Issue #6599 축 ②] 분할 조각의 칸 상자가 자기 내용보다 짧아 중첩 표 밑줄이 바깥 표
//! 밑줄을 넘던 결함의 가드.
//!
//! `cell_units` 는 중첩 표를 품은 호스트 문단을 두 갈래로 올린다.
//!
//! * **atom** — 호스트 문단 하나를 한 유닛으로. 저장 줄높이가 캡션을 이미 품는다.
//! * **행 유닛**(`nested_row = Some(ri)`) — 중첩 표를 행 단위로 쪼갠다. 여기서
//!   **캡션 높이가 통째로 빠졌다.**
//!
//! `2181727` 7쪽 조각(한 칸, 문단 7개) 실측 — 페인트 전진량 vs 유닛 합(px):
//!
//! ```text
//!   p3 atom      187.08 / 187.08   ✔
//!   p5 행유닛     146.33 / 122.77   ← +23.56 = `<표2>` 캡션 25.43
//!   p6 atom      197.53 / 207.01   (유닛이 큼 — 안전)
//!   p9 행유닛     162.48 / 169.39   (캡션 없음, 유닛이 큼 — 안전)
//! ```
//!
//! 조각 칸 상자는 `Σ unit.height + 안 여백` 이라 그만큼 짧아지고, 마지막 중첩 표
//! 밑줄이 바깥 표 밑줄보다 **아래**로 나가 두 선이 겹쳤다.
//!
//! ```text
//!               바깥 표 밑줄   중첩 표 밑줄
//!   종전          1002.59       1006.00   ← 역전(겹침)
//!   교정 후       1019.24       1006.00   ← 정상
//! ```
//!
//! ⚠ 이 시험은 **겹침 축만** 잠근다. 같은 이슈의 다른 축(한/글은 이어지는 조각을
//! 본문 하한까지 늘리고 그 안에서 내용을 가운데에 둔다 — 40px)은 열려 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/admrul_downloads/고용노동부/2181727_[별표 1의2] 프레스 또는
/// 전단기 방호장치의 시험방법(제4조 관련)(방호장치 안전인증 고시).hwp`
///
/// ⚠ `.hwp` 재현본을 `samples/` 에 넣으면 `ir_field_sweep_baseline` 이 `samples/`
/// **전체**를 스윕하는 탓에 이 수정과 무관한 직렬화 발산까지 끌고 온다(실측: 이 문서
/// 하나로 `list_header_width_ref` 2행). 그래서 코퍼스에서 찾고, 없으면 건너뛴다.
/// `RHWP_ISSUE6599_SAMPLE` 로 경로를 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE6599_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(
            r"C:\Users\planet\hwpdocs_10k_share",
            r"\admrul_downloads\고용노동부"
        ),
        concat!(r"D:\hwpdocs_10k_share", r"\admrul_downloads\고용노동부"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("2181727") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// **자기 표의** 가로 괘선만 모은다 — 중첩 `Table` 아래로는 내려가지 않는다.
/// (재귀하면 바깥 표의 괘선 집합이 중첩 표 괘선을 삼켜 두 값이 항상 같아진다.)
fn own_horizontal_rules(node: &RenderNode, out: &mut Vec<f64>, inside_table: bool) {
    if inside_table && matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    if let RenderNodeType::Line(l) = &node.node_type {
        if (l.y1 - l.y2).abs() < 0.5 && (l.x2 - l.x1).abs() > 60.0 {
            out.push(l.y1);
        }
    }
    for child in &node.children {
        own_horizontal_rules(child, out, true);
    }
}

/// 7쪽에서 **가장 아래 가로 괘선은 바깥 표의 밑줄**이어야 한다 — 중첩 표 밑줄이
/// 그보다 아래에 있으면 두 선이 겹쳐 보인다.
#[test]
fn fragment_cell_box_contains_its_nested_table() {
    let Some(bytes) = sample() else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(6).expect("7쪽 render tree");

    // 바깥 표는 Column 의 직계 자식이고, 중첩 표는 그 안에 있다.
    let mut outer_bottom = f64::NAN;
    let mut nested_bottom = f64::NAN;
    fn walk(n: &RenderNode, depth_tables: usize, outer: &mut f64, nested: &mut f64) {
        let mut depth = depth_tables;
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            depth += 1;
            let mut rules = Vec::new();
            own_horizontal_rules(n, &mut rules, false);
            let bottom =
                rules
                    .iter()
                    .copied()
                    .fold(f64::NAN, |a: f64, y| if a.is_nan() { y } else { a.max(y) });
            if !bottom.is_nan() {
                let slot: &mut f64 = if depth == 1 { outer } else { nested };
                if slot.is_nan() || bottom > *slot {
                    *slot = bottom;
                }
            }
        }
        for c in &n.children {
            walk(c, depth, outer, nested);
        }
    }
    walk(&page.root, 0, &mut outer_bottom, &mut nested_bottom);

    assert!(
        outer_bottom.is_finite() && nested_bottom.is_finite(),
        "7쪽에서 바깥 표/중첩 표 괘선을 못 찾았다 — 시험 설정 오류. \
         outer={outer_bottom} nested={nested_bottom}"
    );
    assert!(
        outer_bottom >= nested_bottom - 0.5,
        "중첩 표 밑줄({nested_bottom:.2})이 바깥 표 밑줄({outer_bottom:.2})보다 아래다 \
         — #6599 회귀. 조각 칸 상자가 자기 내용보다 짧다(행 유닛이 중첩 표 캡션을 \
         안 세면 종전처럼 1002.59 vs 1006.00 으로 역전한다)."
    );
}
