//! [Issue #6976] 쪽을 끝내는 조각의 마지막 행 상자를 **배치 뒤에** 접는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 행 높이에는 조각 마지막 줄 **뒤의 줄간격**이 들어 있는데 한/글은 그 띠를 그리지 않는다.
//! 그래서 쪽을 끝내는 조각마다 상자가 내용보다 한 줄간격만큼 길었다.
//!
//! 그 여분은 **행 높이를 정하는 단계에서는 알 수 없다** — 실제로 얼마가 남는지는 칸 내용을
//! 배치해 봐야 나오고, 배치는 행 높이가 정해진 *뒤*에 일어난다. 저장 `LineSeg` 합으로
//! 예측하는 우회로는 정본이 있는 네 문서에서 한 번도 성립하지 않아 기각했다
//! ([#6976 계측](https://github.com/edwardkim/rhwp/issues/6976)).
//!
//! 그래서 흐름·예약이 쓰는 행 높이는 그대로 두고, **그리는 상자만** 배치 뒤에 접는다.
//! 조각이 쪽을 끝내므로(뒤 내용은 다음 쪽 소유) 이 축소는 뒤 내용을 밀지 않는다.
//!
//! # 독립 기대값 — 한/글 정본
//!
//! `pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf`
//! (115쪽 = rhwp 115쪽). 쪽 척도(1122.5/1121.33)를 걷은 1쪽의 표 괘선 상자다.
//!
//! ```text
//!   행 0    79.36 ..  99.03
//!   행 1    99.03 .. 118.87
//!   행 2   118.87 .. 1056.90   <- 쪽을 넘는 큰 칸, 이 조각이 끝내는 행
//!   표 전체 79.36 .. 1056.90   =  h 977.54
//!
//!   rhwp 수정 전   h 985.40   (+7.86 — 마지막 줄 줄간격 9.60 이 그대로 남았다)
//!   rhwp 수정 후   h 975.80   (−1.74)
//! ```
//!
//! # 잠그는 것
//!
//! ① 접기가 일어난다(정본 높이). ② 접기가 **실제로 남은 여분**을 넘겨 자르지 않는다. ③ 쪽이
//! 상자를 정한 조각(`#7095`·`#7063` 레인②)은 **접지 않는다**(비적용 경로). 셋 중 하나만
//! 되돌려도 깨진다 — ③의 가드를 빼면 `hwpx_sample2` 19쪽 조각이 정본보다 2.87px 짧아진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn page_root(sample: &str, page_index: u32) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(page_index)
        .expect("render tree")
        .root
}

/// 본문 최상위 표(칸 안 중첩 표 제외)를 `para_index` 로 찾는다.
fn top_level_table<'a>(
    node: &'a RenderNode,
    para_index: usize,
    inside: bool,
) -> Option<&'a RenderNode> {
    let mut inside = inside;
    if let RenderNodeType::Table(t) = &node.node_type {
        if !inside && t.para_index == Some(para_index) {
            return Some(node);
        }
        inside = true;
    }
    node.children
        .iter()
        .find_map(|child| top_level_table(child, para_index, inside))
}

/// 가시 글줄의 바닥들.
fn visible_text_line_bottoms(node: &RenderNode, out: &mut Vec<f64>) {
    if node.visible
        && matches!(node.node_type, RenderNodeType::TextLine(_))
        && node.bbox.height > 0.0
    {
        out.push(node.bbox.y + node.bbox.height);
    }
    for child in &node.children {
        visible_text_line_bottoms(child, out);
    }
}

/// ① 쪽을 끝내는 조각의 상자가 한/글 정본 높이까지 접힌다.
#[test]
fn the_page_ending_fragment_box_folds_to_the_hancom_height() {
    let root = page_root("samples/issue1949_giant_cell_nested_tables_perf.hwpx", 0);
    let table = top_level_table(&root, 0, false).expect("1쪽 표 pi=0 — 시험 설정");
    let height = table.bbox.height;
    assert!(
        (height - 977.54).abs() <= 2.5,
        "① 조각 상자 높이는 한/글 정본(977.54px) 2.5px 안이어야 한다 \
         (수정 전 985.40 — 마지막 줄 줄간격 9.60 이 남았다): {height:.2}"
    );

    // 접은 상자가 이 조각이 그린 내용을 자르지 않는다.
    let box_bottom = table.bbox.y + table.bbox.height;
    let mut bottoms = Vec::new();
    visible_text_line_bottoms(table, &mut bottoms);
    bottoms.sort_by(f64::total_cmp);
    let last = *bottoms.last().expect("조각 안 가시 글줄");
    assert!(
        last <= box_bottom + 0.5,
        "① 접은 상자(바닥 {box_bottom:.2})는 그린 마지막 글줄({last:.2}) 아래여야 한다"
    );
}

/// ② 접기의 하한 — «실제로 남은 여분» 을 넘겨 자르지 않는다.
///
/// 자르는 양은 «마지막 줄의 저장 줄간격»(①의 상한)과 «실제로 남은 여분»(②의 상한) 중
/// **작은 쪽**이다. 어떤 조각의 행 높이에는 그 줄간격의 *일부만* 들어 있어, ①만 쓰면 상자가
/// 내용 아래로 내려간다.
///
/// 독립 기대값 — `pdf/table_giant_cell_overfill-hwpx-2024.pdf`(48쪽 = rhwp 48쪽) 29쪽의
/// 바깥 표 괘선 상자는 `y 77.44 .. 1068.58`(h 991.15)이다.
///
/// ```text
///   rhwp 상한 ② 있음   h 990.40   (−0.75)
///   rhwp 상한 ② 없음   h 980.80   (−10.35 — 줄간격 9.60 을 통째로 잘랐다)
/// ```
///
/// 이 문서는 접기 **전에도** 칸 내용이 상자 밖으로 넘친다(이름 그대로 overfill, 29쪽 마지막
/// 글줄 바닥 1091.8 > 상자 1075.6). 그건 선행 결함이고 접기가 건드리지 않는다 — 29쪽의 가시
/// 글줄 29개와 그 좌표는 접기 전후가 완전히 같다. 그래서 여기서는 상자 높이만 잠근다.
#[test]
fn the_fold_stops_at_the_slack_the_layout_actually_left() {
    let root = page_root("samples/table_giant_cell_overfill.hwpx", 28);
    let table = top_level_table(&root, 0, false).expect("29쪽 표 pi=0 — 시험 설정");
    let height = table.bbox.height;
    assert!(
        (height - 991.15).abs() <= 2.5,
        "② 접은 상자 높이는 한/글 정본(991.15px) 2.5px 안이어야 한다 \
         (여분 상한을 빼면 980.80 — 9.60 을 통째로 자른다): {height:.2}"
    );
}

/// ③ 비적용 경로 — 쪽이 상자를 정한 조각은 접지 않는다.
///
/// `#7095`·`#7063` 레인②가 세운 갈래다. 그 높이는 내용이 아니라 **쪽**이 정한 값이고
/// 정본도 그 자리를 채운다(`pdf/hwpx_sample2-hwpx-2020.pdf` 19쪽 상자 89.91..1081.39).
/// 가드를 빼면 이 상자가 9.6px 접혀 정본보다 2.87px 짧아진다.
#[test]
fn a_page_pinned_fragment_is_not_folded() {
    let root = page_root("samples/hwpx_sample2.hwpx", 18);
    let table = top_level_table(&root, 182, false).expect("19쪽 표 pi=182 — 시험 설정");
    let bottom = table.bbox.y + table.bbox.height;
    assert!(
        (bottom - 1081.39).abs() <= 1.0,
        "③ 쪽이 정한 조각 상자는 접지 않고 정본(1081.39px) 1px 안에 있어야 한다 \
         (접으면 1078.5 안팎): {bottom:.2}"
    );
}
