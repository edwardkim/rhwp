//! 검사 유형: 관계형 + 고정형(독립 한/글 Print 전체 쪽수).
//! 주석 갱신: 2026-11-05 08:20 KST (새 파일, 직전 파일 커밋 없음).
//! 2026-11-05 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! compat 2022, base e96b500e3e + 이 수정, Native skia / fresh WASM):
//! - 25128 국회입법조사처 보도자료 2019-05-17(HWP, 한컴 1쪽 = rhwp 1쪽): 1쪽 Native 99.95 ·
//!   WASM 99.43(base 55.42 · 54.09). 전쪽 최저 Native 99.95 · WASM 99.43.
//! - 36849 국회입법조사처 보도자료 2021-11-16(HWP, 한컴 3쪽 = rhwp 3쪽): 1쪽 Native 98.51 ·
//!   WASM 98.62(base 52.73 · 52.09), 2·3쪽은 base 와 같음. 전쪽 최저 Native 96.37 · WASM 96.78.
//!
//! 측정 글꼴: 문서별 face 폴더(휴먼명조 HMKMM·HY헤드라인M·HY견고딕·HY울릉도M, 36849 는 KoPub
//! 돋움 Light·Medium 추가) + `ttfs/hwp`·`ttfs/windows`. HY중고딕·Times New Roman 은 없어 대체된다.
//! 이 검사는 글꼴과 무관하게 저장 높이·저장 vpos 사이 관계만 본다.
//!
//! 근거: mydocs/pr/assets/issue_8610/{head,base}/{native,wasm}/{i318,i319}_silhouette.tsv.
//!
//! 칸 안의 빈 host 문단이 비-TAC 글 위아래(TopAndBottom) 표를 품어 흐름을 표 바깥 상자 하단까지
//! 밀면, 뒤 문단의 문단 앞 간격은 host 글줄 끝에서 재며 표 띠 안에 겹친다(#8610). 수정 전
//! rhwp 는 표 하단 뒤에 앞 간격(900 HU = 12 px)을 다시 더해 칸 본문 전체가 내려갔다.
//! 한 쪽짜리 표는 일반 칸 배치, 여러 쪽으로 나뉘는 표는 부분 표 배치 경로를 탄다 — 두 원본으로
//! 두 경로를 함께 검사한다.
//!
//! 기대값은 원본 저장값과 독립 한/글 Print(`pdf/issue8610/*-2022.pdf`)로 정한다.
//! - 25128: 큰 칸(행 7) 문단 0 = 빈 host + 1×1 제목 표(높이 5372, 바깥여백 140/140), 문단 1
//!   (앞 간격 900) 저장 vpos 5652 = 표 바깥 상자 하단, 문단 2 저장 vpos 8792. 한/글 Print 에서
//!   문단 2 첫 줄은 제목 표 아래 괘선보다 hu(8792 − 5652 + 140) 아래다.
//! - 36849: 같은 형상, 표 높이 4564, 문단 1 vpos 4844, 문단 2 vpos 6420.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

#[derive(Default)]
struct BodyCell {
    /// 바깥 표 칸의 글줄 (문단, 줄, 위끝). 중첩 표 안 글줄은 뺀다.
    lines: Vec<(usize, u32, f64)>,
    /// 칸 안 중첩 표의 (위끝, 아래끝).
    nested: Vec<(f64, f64)>,
}

fn collect(node: &RenderNode, table_depth: usize, out: &mut BodyCell) {
    let depth = table_depth + usize::from(matches!(node.node_type, RenderNodeType::Table(_)));
    if matches!(node.node_type, RenderNodeType::Table(_)) && depth == 2 {
        out.nested
            .push((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if let (1, Some(p), Some(l)) = (depth, line.para_index, line.line_index) {
            out.lines.push((p, l, node.bbox.y));
        }
    }
    for child in &node.children {
        collect(child, depth, out);
    }
}

fn check(sample: &str, pages: u32, table_h: f64, p1_vpos: f64, p2_vpos: f64) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        pages,
        "{sample}: 독립 한/글 Print 전체 쪽수"
    );

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut cell = BodyCell::default();
    collect(&tree.root, 0, &mut cell);
    let line = |para: usize| {
        cell.lines
            .iter()
            .find(|l| l.0 == para && l.1 == 0)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 큰 칸 문단 {para} 첫 줄 없음 {:?}", cell.lines))
    };
    let &[(title_top, title_bottom)] = cell.nested.as_slice() else {
        panic!("{sample}: 1쪽 중첩 제목 표는 하나 {:?}", cell.nested);
    };
    // 제목 표는 저장 높이 그대로, 문단 1·2 는 그 아래 순서대로 놓인다.
    assert!(
        (title_bottom - title_top - hu(table_h)).abs() <= 1.0,
        "{sample}: 제목 표 높이 {:.2} ≠ 저장 {:.2}",
        title_bottom - title_top,
        hu(table_h)
    );
    assert!(
        title_bottom <= line(1).2 && line(1).2 < line(2).2,
        "{sample}: 제목 표 → 문단 1 → 문단 2 순서가 아님"
    );
    // 문단 1 첫 줄 = 제목 표 아래 바깥여백(140) 바로 아래 — 앞 간격 900 은 표 띠에 흡수.
    let lead = line(1).2 - title_bottom;
    assert!(
        (lead - hu(140.0)).abs() <= 1.0,
        "{sample}: 제목 표 → 문단 1 간격 {lead:.2} ≠ 바깥여백 {:.2} (앞 간격 이중 가산)",
        hu(140.0)
    );
    // 문단 2 는 저장 사다리대로 문단 1 뒤에 이어진다.
    let gap = line(2).2 - title_bottom;
    let expected = hu(p2_vpos - p1_vpos + 140.0);
    assert!(
        (gap - expected).abs() <= 1.0,
        "{sample}: 제목 표 → 문단 2 간격 {gap:.2} ≠ 저장 사다리 {expected:.2}"
    );
}

#[test]
fn top_bottom_table_band_absorbs_successor_spacing_single_page_cell() {
    check(
        "samples/issue8610/25128-nars-press-release-20190517.hwp",
        1,
        5372.0,
        5652.0,
        8792.0,
    );
}

#[test]
fn top_bottom_table_band_absorbs_successor_spacing_split_table_cell() {
    check(
        "samples/issue8610/36849-nars-press-release-20211116.hwp",
        3,
        4564.0,
        4844.0,
        6420.0,
    );
}
