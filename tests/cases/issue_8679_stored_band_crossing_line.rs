//! [#8679] 관계형·고정형 회귀 — 문단 기준 자리차지(글 위아래) 표의 띠 위끝에 걸친 저장 호스트
//! 줄은 띠 끝으로 내려가고, 같은 저장 프레임의 다음 문단은 그 줄 뒤에서 시작한다.
//!
//! 주석 갱신: 2026-11-06 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8679/stored_band_crossing_line.hwpx` — 서울시 결재문서 원본
//!   (36385506, 결재 시스템 저장 HWPX, 2쪽) 그대로. 문단 1 의 저장 줄 0(vpos 440, 높이 1100)
//!   뒤에 제목 표(1×1, 문단 기준 세로 오프셋 2001, 높이 2282, 띠 2441..4723)가 있고, 저장
//!   줄 1(vpos 2200, 높이 300, 줄 간격 180)이 띠 위끝에 걸친다. 문단 2 는 4723 에 저장돼 있다.
//! - 독립 기준 1: 같은 입력을 한/글 2022 로 다시 저장하면 줄 1 이 4723(띠 끝), 문단 2 가
//!   5203(= 4723 + 300 + 180)이 된다.
//! - 독립 기준 2: 같은 입력의 한/글 2022 Print PDF
//!   `pdf/issue8679_stored_band_crossing_line-2020.pdf`(2쪽). 96dpi 래스터의 머리 표 잉크 띠가
//!   146..191px 로, 수정 전 rhwp(139..185)보다 6.4px(480HU) 아래다.
//! - 수정 전 rhwp(devel 170b886f16): 저장 줄 1 을 띠와 겹친 자리에 두고 문단 2 를 저장값
//!   4723 에 둬서 머리 표 이하 본문 전체가 6.4px 위.
//!
//! 측정(2026-11-06, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | 쪽 | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|---|
//! | stored_band_crossing_line | 2 | 1 | 99.98 | 100.00 | 56.86 |
//! | stored_band_crossing_line | 2 | 2 | 100.00 | 100.00 | 100.00 |
//!
//! 두 경로 전쪽 최저: Native 99.98, fresh WASM 100.00.
//!
//! 증거: `mydocs/pr/assets/issue_8679/{native,native_base,wasm}/*_silhouette.tsv`,
//! `review/*.png`. 검사는 저장 줄과 띠의 원본 관계(전제), 제목 표 아래 끝과 다음 문단 표 위끝
//! 사이가 걸친 줄의 높이·줄 간격과 다음 표 위 바깥여백의 합인 관계(관계형), 독립 PDF 전체
//! 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8679/stored_band_crossing_line.hwpx";

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.control_index == Some(control) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn first_table(core: &DocumentCore, para: usize) -> (usize, &Table) {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .enumerate()
        .find_map(|(ci, c)| match c {
            Control::Table(t) => Some((ci, t.as_ref())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {para} 에 표"))
}

#[test]
fn stored_line_crossing_own_topbottom_band_moves_below_it() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 2, "한컴 Print PDF 2쪽");

    // 전제: 원본 저장 줄 1 이 제목 표 띠의 위끝에 걸친다(띠는 줄 0 기준 세로 오프셋).
    let host = &core.document().sections[0].paragraphs[1];
    let (title_ci, title) = first_table(&core, 1);
    assert!(
        !title.common.treat_as_char,
        "제목 표는 글자처럼 취급하지 않는다"
    );
    let [line0, line1] = host.line_segs.as_slice() else {
        panic!("문단 1 은 저장 줄 둘");
    };
    let band_top = i64::from(line0.vertical_pos) + i64::from(title.common.vertical_offset as i32);
    let band_bottom = band_top
        + i64::from(title.outer_margin_top)
        + i64::from(title.common.height)
        + i64::from(title.outer_margin_bottom);
    assert!(
        i64::from(line0.vertical_pos + line0.line_height) <= band_top,
        "줄 0 은 띠 위에서 끝난다"
    );
    assert!(
        i64::from(line1.vertical_pos) < band_top
            && i64::from(line1.vertical_pos + line1.line_height) > band_top,
        "줄 1 이 띠 위끝 {band_top} 에 걸친다"
    );
    let next = &core.document().sections[0].paragraphs[2];
    assert_eq!(
        i64::from(next.line_segs[0].vertical_pos),
        band_bottom,
        "원본은 다음 문단을 띠 끝에 저장한다"
    );

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let title_box = find_table(&tree.root, 1, title_ci).expect("제목 표를 그린다");
    let (next_ci, next_table) = first_table(&core, 2);
    let next_box = find_table(&tree.root, 2, next_ci).expect("다음 문단 표를 그린다");

    // 걸친 줄은 띠 끝에서 시작하고, 다음 문단은 그 줄의 높이와 줄 간격 뒤에서 시작한다.
    let expected_gap = px(i64::from(title.outer_margin_bottom))
        + px(i64::from(line1.line_height) + i64::from(line1.line_spacing))
        + px(i64::from(next_table.outer_margin_top));
    let gap = next_box.y - (title_box.y + title_box.height);
    assert!(
        (gap - expected_gap).abs() <= 0.5,
        "제목 표 아래 끝→다음 표 위끝 {gap:.2} = 아래 바깥여백 + 걸친 줄 높이·줄 간격 + 위 바깥여백 {expected_gap:.2}"
    );
    assert!(
        next_box.y > title_box.y + title_box.height,
        "다음 표는 제목 표 아래"
    );
}
