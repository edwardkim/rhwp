//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8486의 정식 회귀.
//! 주석 갱신: 2026-11-05T20:40:00+09:00; 직전 파일 변경: 2026-11-05T16:57:51+09:00 (`d188a6bea337263efcbbd3e2718158965be85456`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8468/156454114-p4g-opening-remarks.hwp`; 독립 Print `pdf/issue8468/156454114-p4g-opening-remarks-2022.pdf`.
//! 전체 8쪽, 전쪽 최저 Native 99.99% / fresh WASM 100.00%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 4쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 주장 5쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8468/line_reserve_F20_L30_A30_R12.hwp`; 독립 Print `pdf/issue8468/line_reserve_F20_L30_A30_R12-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 99.99% / fresh WASM 99.99%.
//! 주장 1쪽 raw: Native 99.99% / fresh WASM 99.99%.
//! 입력 `samples/issue8468/line_reserve_F20_L30_A40_R12.hwp`; 독립 Print `pdf/issue8468/line_reserve_F20_L30_A40_R12-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8468/line_reserve_F20_L30_A46_R12.hwp`; 독립 Print `pdf/issue8468/line_reserve_F20_L30_A46_R12-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8486_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T21:10:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-02 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! head ced211a7ba Native skia 18:46+09:00 / fresh WASM 19:11+09:00):
//! - 156454114 P4G 개회사(한컴 8쪽 = rhwp 8쪽): 각주 쪽 3·4·5쪽 Native 99.40/99.24/99.29%,
//!   fresh WASM 99.98/99.98/99.97%; 전8쪽 최저 Native 98.53%(1쪽) / WASM 99.86%(7쪽).
//! - line_reserve A30(2쪽): Native 99.97/100.00%, WASM 99.98/100.00%; 최저 99.97% / 99.98%.
//! - line_reserve A40(2쪽): Native·WASM 각 100.00/100.00%.
//! - line_reserve A46(2쪽, 대조군): Native·WASM 각 100.00/100.00%.
//! - 수정 전(base de2295d430) Native: P4G 9쪽(6쪽 4.05%), A30 2쪽 43.37%, A40 2쪽 59.27%.
//!
//! 앞 조각 각주 비용 후속 측정(2026-11-05, 제품 코드 head `d188a6bea3`·base `6e1de40620`, 96 DPI
//! `visual_sweep.py --silhouette-only --compat 2022`, `RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`,
//! Native Skia / fresh WASM Canvas, `silhouette_raw_match_percent`):
//! - line_reserve A10 T20(전체 2쪽 = PDF 2쪽): head Native 99.99/100.00%, fresh WASM 100.00/100.00%.
//!   수정 전 base Native 40.89/64.36%(앞부분만 들어간 각주가 쪽 나머지를 막아 뒤 줄이 2쪽으로 밀림).
//! - line_reserve A30 R4 B4(전체 2쪽 = PDF 2쪽, 각주 안 위아래 배치 표): head Native 99.96/100.00%,
//!   fresh WASM 99.99/100.00%. base 도 같은 값이다. 앞 조각을 줄 합으로 재던 중간 커밋은 1쪽에
//!   0..18줄을 두어 뒤 본문이 각주 위에 겹쳤다(회귀 검사로만 고정, 시각 측정 없음).
//! - A30·A40·A46(각 2쪽): head Native 최저 99.97/99.98/99.99%, fresh WASM 99.99/100.00/100.00%.
//!   P4G(8쪽): head Native 최저 96.63%(1쪽), fresh WASM 99.81%(7쪽).
//!
//! 근거: mydocs/pr/assets/issue_8468/{head,base}/{native,wasm}/*_silhouette.tsv,
//! 후속 측정은 `mydocs/pr/assets/issue_8468/followup/{head,base}/{native,wasm}/` 와 `followup/png/`.
//!
//! 본문 각주는 그 참조 줄 뒤의 본문 줄보다 먼저 쪽 아래를 차지한다(#8468).
//!
//! 한컴 Print(Hwp 2022 12.0.0.4547) 실측 규칙:
//! - 각주 영역은 본문 영역 안에 있다. 꼬리말 밴드를 쓰지 않는다.
//! - 본문 줄은 이미 받아들인 앞 줄들의 각주 위에 들어가야 한다.
//! - 참조 줄은 그 각주가 다 들어가지 않아도 놓일 수 있고, 각주는 들어가는 줄까지 이 쪽에,
//!   나머지는 다음 쪽으로 이어진다.
//!
//! 수정 전 rhwp 는 각주가 있는 쪽마다 본문을 40px(3000HU) 덜 쓰고, 꼬리말 없는 구역에서는
//! 꼬리말 밴드 높이만큼 본문을 돌려줬다. 각주는 문단 배치가 끝난 뒤 통째로 등록돼, 앞 줄이
//! 참조한 각주 자리를 뒤 줄이 차지했다.
//! - 156454114 P4G 개회사(8쪽): 40px 여유 때문에 5쪽 마지막 문단(각주 4 참조)을 6쪽으로
//!   밀어 9쪽이 됐다.
//! - `line_reserve_*`(합성, `samples/issue8468/gen_line_reserve.py` 로 만든 HWPX 를 한/글 2020 이
//!   다시 저장한 HWP): 30줄 문단의 문장 AT 뒤에 긴 각주 하나. 수정 전 rhwp 는 세 입력 모두 첫 쪽에
//!   23줄을 두고 각주를 다음 쪽으로 밀었다. 한/글은 참조 줄 뒤 줄이 각주 위에 들어가지 않으면
//!   그 줄부터 다음 쪽에 둔다(A30: 18줄, A40: 20줄). A46 은 참조 줄이 첫 쪽 끝 줄이고 각주가
//!   들어갈 자리가 없어 통째로 다음 쪽에 가는 대조군이다(수정 전후 같다).
//! - `line_reserve_F20_L30_A10_R1_T20`(같은 생성기 `TABLE_ROWS=20`): 각주가 글 한 줄 + 글자처럼
//!   취급한 20행 표. 참조 줄 뒤에 글 줄만 들어가고 표 줄은 들어가지 않는다. 한/글은 글 줄만 1쪽에
//!   두고 뒤 본문 줄로 그 위를 채운다(쪽 reset 줄 21, 표는 2쪽 각주). 앞부분만 들어간 각주가 쪽
//!   나머지를 막는다고 보면 뒤 줄이 모두 2쪽으로 밀린다(실문서 i838 90쪽 각주 273 의 형상).
//!
//! 기대값은 같은 원본의 독립 한컴 Print PDF(`pdf/issue8468/*-2022.pdf`, `*-2020.pdf`)와
//! 한/글이 저장한 쪽 reset 줄에서 정했다(쪽수, 쪽마다 줄 머리 `N)` 으로 시작하는 각주 번호,
//! 긴 문단의 쪽 경계 줄).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("parse sample")
}

fn footnote_area(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(footnote_area)
}

/// 각주 영역에서 번호(`N)`)를 그린 각주(= 이 쪽에서 시작한 각주)의 번호.
/// 이어지는 조각은 번호를 그리지 않는다.
fn started_notes(node: &RenderNode, out: &mut Vec<u16>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(number) = run
            .display_text
            .as_deref()
            .and_then(|text| text.strip_suffix(')'))
            .and_then(|digits| digits.parse::<u16>().ok())
        {
            out.push(number);
        }
    }
    for child in &node.children {
        started_notes(child, out);
    }
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

/// 본문 영역 안, 각주 영역 밖 글줄의 아래 끝 중 가장 큰 값.
fn lowest_body_line(node: &RenderNode) -> f64 {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return f64::MIN;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        return node.bbox.y + node.bbox.height;
    }
    node.children
        .iter()
        .map(lowest_body_line)
        .fold(f64::MIN, f64::max)
}

/// 쪽(1부터)마다 시작한 각주 번호. 각주가 시작하지 않은 쪽은 빠진다.
/// 모든 쪽에서 본문 글줄이 각주 영역 위 끝을 넘지 않는지도 함께 확인한다.
fn note_starts_without_overlap(sample: &str, core: &DocumentCore) -> Vec<(u32, Vec<u16>)> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("page render tree");
        let Some(area) = footnote_area(&tree.root) else {
            continue;
        };
        let bottom = body(&tree.root).map_or(f64::MIN, lowest_body_line);
        assert!(
            bottom <= area.bbox.y + 0.5,
            "{sample} {}쪽: 본문 글줄 아래 끝 {bottom:.1}px 가 각주 영역 위 끝 {:.1}px 와 겹친다",
            page + 1,
            area.bbox.y
        );
        let mut numbers = Vec::new();
        started_notes(area, &mut numbers);
        if !numbers.is_empty() {
            out.push((page + 1, numbers));
        }
    }
    out
}

fn expected(rows: &[(u32, &[u16])]) -> Vec<(u32, Vec<u16>)> {
    rows.iter()
        .map(|(page, notes)| (*page, notes.to_vec()))
        .collect()
}

#[test]
fn page_tail_paragraph_keeps_its_note_without_safety_margin() {
    let sample = "samples/issue8468/156454114-p4g-opening-remarks.hwp";
    let core = core(sample);
    assert_eq!(core.page_count(), 8, "{sample}: 한컴 Print 8쪽");
    assert_eq!(
        note_starts_without_overlap(sample, &core),
        expected(&[(3, &[1]), (4, &[2, 3]), (5, &[4])]),
        "{sample}: 쪽마다 시작하는 각주 번호가 한컴 Print 와 같다"
    );
}

/// 긴 문단(pi=21) 조각들: (쪽, 시작 줄, 끝 줄).
fn fragments(core: &DocumentCore) -> Vec<(u32, u64, u64)> {
    let pages = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for (page, value) in pages.as_array().unwrap().iter().enumerate() {
        for column in value["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                if item["paraIndex"].as_u64() == Some(21) && item["kind"] == "partialParagraph" {
                    out.push((
                        page as u32 + 1,
                        item["startLine"].as_u64().unwrap(),
                        item["endLine"].as_u64().unwrap(),
                    ));
                }
            }
        }
    }
    out
}

#[test]
fn earlier_line_note_reserves_space_before_later_lines() {
    // (입력, 한/글 2020 이 저장한 쪽 reset 줄, 각주 1 이 시작하는 쪽)
    for (sample, page_break_line, note_page) in [
        // 참조 줄 뒤 줄들이 각주 위에 들어가지 않아 다음 쪽으로 간다.
        ("samples/issue8468/line_reserve_F20_L30_A30_R12.hwp", 18, 1),
        // 참조 줄이 첫 쪽 마지막 줄이고 각주는 그 쪽에 다 들어간다.
        ("samples/issue8468/line_reserve_F20_L30_A40_R12.hwp", 20, 1),
        // 대조군: 참조 줄은 첫 쪽 마지막 줄에 놓이지만 각주는 한 줄도 들어가지 않아
        // 통째로 다음 쪽에서 시작한다.
        ("samples/issue8468/line_reserve_F20_L30_A46_R12.hwp", 23, 2),
    ] {
        let core = core(sample);
        assert_eq!(core.page_count(), 2, "{sample}: 한컴 Print 2쪽");
        assert_eq!(
            fragments(&core),
            vec![(1, 0, page_break_line), (2, page_break_line, 29)],
            "{sample}: 긴 문단은 한/글 저장 쪽 reset 줄에서 나뉜다"
        );
        assert_eq!(
            note_starts_without_overlap(sample, &core),
            expected(&[(note_page, &[1])]),
            "{sample}: 각주 1 이 한컴 Print 와 같은 쪽에서 시작한다"
        );
    }
}

/// 각주 영역에 표(각주 본문 안의 표)가 있는가.
fn has_note_table(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::Table(_)) || node.children.iter().any(has_note_table)
}

#[test]
fn partial_note_with_unsplittable_table_keeps_later_lines_above_its_prefix() {
    // 참조 줄(5번째 줄쯤) 뒤에 각주 글 한 줄은 들어가지만 나눌 수 없는 표 줄은 들어가지 않는다.
    // 한/글은 각주 글 줄만 1쪽에 두고 표는 2쪽 각주로 넘기며, 참조 줄 뒤 본문 줄은 그 글 줄 위
    // 남는 공간을 계속 채운다(쪽 reset 줄 21). 각주가 앞부분만 들어가면 쪽 나머지를 막는다고
    // 보면 뒤 줄이 모두 2쪽으로 밀린다(i838 90쪽 각주 273 과 같은 형상).
    let sample = "samples/issue8468/line_reserve_F20_L30_A10_R1_T20.hwp";
    let core = core(sample);
    assert_eq!(core.page_count(), 2, "{sample}: 한컴 Print 2쪽");
    assert_eq!(
        fragments(&core),
        vec![(1, 0, 21), (2, 21, 29)],
        "{sample}: 긴 문단은 한/글 저장 쪽 reset 줄 21 에서 나뉜다"
    );
    assert_eq!(
        note_starts_without_overlap(sample, &core),
        expected(&[(1, &[1])]),
        "{sample}: 각주 1 은 참조 줄이 있는 1쪽에서 시작한다"
    );
    let table_pages: Vec<u32> = (0..core.page_count())
        .filter(|&page| {
            let tree = core.build_page_render_tree(page).expect("page render tree");
            footnote_area(&tree.root).is_some_and(has_note_table)
        })
        .map(|page| page + 1)
        .collect();
    assert_eq!(
        table_pages,
        vec![2],
        "{sample}: 각주 1 의 표 줄은 1쪽에 들어가지 않아 2쪽 각주 영역에 이어진다"
    );
}

#[test]
fn partial_note_prefix_with_block_table_counts_table_height() {
    // 각주 = 글 문단 + 글자처럼 취급하지 않은 위아래 배치 표(4행) + 긴 글 문단. 표는 host 줄의
    // 줄 높이·줄간격 밖을 차지하므로 앞 조각 높이는 줄 합보다 표 높이만큼 크다(실문서 i628
    // 95쪽 각주 56 의 형상). 한/글은 표와 뒤 글 일부까지 1쪽 각주에 두고 참조 줄 뒤 본문 줄은
    // 그 앞 조각 위에 남는 만큼만 받는다(쪽 reset 줄 15). 앞 조각을 줄 합으로 재면 각주 영역이
    // 작게 잡혀 본문 줄 3개가 더 1쪽에 들어가고 각주 위에 겹친다.
    let sample = "samples/issue8468/line_reserve_F20_L30_A30_R4_B4.hwp";
    let core = core(sample);
    assert_eq!(core.page_count(), 2, "{sample}: 한컴 Print 2쪽");
    assert_eq!(
        fragments(&core),
        vec![(1, 0, 15), (2, 15, 29)],
        "{sample}: 긴 문단은 한/글 저장 쪽 reset 줄 15 에서 나뉜다"
    );
    assert_eq!(
        note_starts_without_overlap(sample, &core),
        expected(&[(1, &[1])]),
        "{sample}: 각주 1 은 참조 줄이 있는 1쪽에서 시작하고 본문 줄과 겹치지 않는다"
    );
    let table_pages: Vec<u32> = (0..core.page_count())
        .filter(|&page| {
            let tree = core.build_page_render_tree(page).expect("page render tree");
            footnote_area(&tree.root).is_some_and(has_note_table)
        })
        .map(|page| page + 1)
        .collect();
    assert_eq!(
        table_pages,
        vec![1],
        "{sample}: 각주 1 의 블록 표는 앞 조각에 들어 1쪽 각주 영역에 그려진다"
    );
}
