//! [#8596] 관계형·고정형 회귀 — 앞 글자처럼 취급(TAC) 표의 host 저장 줄이 끝나기 전에 저장된
//! 자리차지 표 프레임은 같은 쪽의 프레임이 아니다. 한/글은 vpos 를 되감아 그 표를 새 쪽에서
//! 시작한다. 앞 TAC 표가 이미 끝난 뒤의 저장 원점은 흐름 커서가 조금 아래여도 같은 쪽에 둔다.
//!
//! 주석 갱신: 2026-11-05 20:08 (KST) — 직전 파일 변경 커밋 `032dcb80e3`(2026-11-05 18:44:11 +0900,
//! devel `a95d1d5afa` rebase). 처음 작성: 2026-11-04 22:34 (KST).
//!
//! - 입력 1: `samples/issue8596/certificate_table.hwpx` — 부천시 지방공무원 인사 규칙
//!   [별표 5의2] 경력경쟁임용 자격증 구분표(한컴 2020 저장, 공개 자치법규 원본 그대로).
//!   문단 4 = TAC 표(저장 줄 vpos 0, 줄 높이 67341), 문단 5 = 위아래 자리차지 표(문단 기준,
//!   오프셋 0, 저장 줄 vpos 0), 문단 6 저장 줄 vpos 66277.
//! - 입력 2: `samples/issue8596/policy_committee_agenda.hwp` — 금융위원회 2013년 제3차
//!   정책연구심의위원회 안건(HWP5, 공개 원본 그대로). 문단 110 TAC 표 뒤 문단 111 자리차지 표가
//!   저장 vpos 0 에서 다시 시작한다.
//! - 입력 3(반례): `samples/issue8596/idol_goods_trimmed.hwpx` — 정책브리핑 보도자료 156645214
//!   (2024-08-12 조간)를 한컴 2022 로 HWPX 저장한 뒤 원 11쪽 머리(저장 vpos 0) 문단
//!   120~133 과 구역 정의만 남기고 한컴 2022 로 다시 저장한 축소본. 문단 6(원 126) TAC 표의 host
//!   줄 끝(26636+7910=34546HU) 아래 문단 12(원 132) 자리차지 표가 저장 vpos 45850 에 놓인다.
//!   조판 커서가 저장 원점보다 아래라 앞 표 종류와 무관하게 되돌림을 판정하면 넘어간다.
//! - 독립 기준: 같은 입력의 한컴오피스 2022 Print PDF `pdf/issue8596_*-2020.pdf`
//!   (입력 1: 9쪽, 입력 2: 14쪽, 입력 3: 2쪽).
//! - 수정 전 rhwp(devel 7fdd5441ff, 그리고 rebase base `a95d1d5afa`): 입력 1 은 8쪽(2쪽에
//!   문단 4·5 표가 같은 원점에 겹침), 입력 2 는 13쪽. 앞 표 종류만 넓힌 중간안은 입력 3 을
//!   3쪽으로 만들었다.
//!
//! 측정(2026-11-05, macOS, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! 판정값 `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣. Native 는 sweep
//! 기본 웹폰트 공급, fresh WASM 은 `--font-path ttfs/opensource`. code head `032dcb80e3`):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native 최저(쪽) | fresh WASM 최저(쪽) | 변경 쪽 Native / WASM |
//! |---|---|---|---|---|
//! | certificate_table | 9 | 99.87 (7) | 99.96 (6) | 2쪽 99.96 / 100.00, 3쪽 99.96 / 100.00 |
//! | policy_committee_agenda | 14 | 92.50 (7) | 95.51 (10) | 10쪽 93.49 / 95.51, 11쪽 96.87 / 99.20 |
//! | idol_goods_trimmed | 2 | 99.50 (1) | 99.64 (1) | 1쪽 99.50 / 99.64 |
//!
//! 두 경로 전쪽 최저: 92.50(policy_committee_agenda 7쪽 Native). 이 쪽은 수정 전 base 와 값이
//! 같아 이 변경의 영향 쪽이 아니다. 수정 전 base 는 입력 1 이 8쪽·입력 2 가 13쪽으로 한컴과
//! 쪽수가 달라 쪽별 대조가 성립하지 않는다. rebase 전 code head `846f6cccf6` 를 같은 macOS 환경에서
//! 측정한 값은 위 표와 전쪽 동일하다. 처음 기록(2026-11-04, Windows 측정, 입력 2 Native 최저 95.19)
//! 과의 차이는 측정 환경의 글꼴 공급 차이다.
//!
//! 증거: `mydocs/pr/assets/issue_8596/{native,wasm}/*_silhouette.tsv`(head),
//! `mydocs/pr/assets/issue_8596/base/{native,wasm}/*_silhouette.tsv`(base `a95d1d5afa`), `review/*.png`.
//! 검사는 표의 쪽 소속·같은 쪽 표끼리 겹치지 않음·반례의 위아래 순서를 보는 관계형과,
//! 독립 PDF 전체 쪽수를 고정하는 고정형이다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// 본문의 최상위 표(칸 안 중첩 표는 제외): (문단, 상자).
fn top_tables(node: &RenderNode, out: &mut Vec<(usize, BoundingBox)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if let Some(para) = table.para_index {
            out.push((para, node.bbox));
        }
        return;
    }
    for child in &node.children {
        top_tables(child, out);
    }
}

fn page_tables(core: &DocumentCore, page: u32) -> Vec<(usize, BoundingBox)> {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut out = Vec::new();
    top_tables(&tree.root, &mut out);
    out
}

/// 문단의 표가 놓인 쪽(0부터). 한 쪽에만 있어야 한다.
fn table_page(core: &DocumentCore, para: usize) -> u32 {
    let pages: Vec<u32> = (0..core.page_count())
        .filter(|&page| page_tables(core, page).iter().any(|(p, _)| *p == para))
        .collect();
    assert_eq!(pages.len(), 1, "문단 {para} 표의 쪽 {pages:?}");
    pages[0]
}

/// 같은 쪽의 서로 다른 문단 표는 세로로 겹치지 않는다.
fn assert_no_table_overlap(core: &DocumentCore, sample: &str) {
    for page in 0..core.page_count() {
        let tables = page_tables(core, page);
        for (i, (pa, a)) in tables.iter().enumerate() {
            for (pb, b) in tables.iter().skip(i + 1) {
                if pa == pb {
                    continue;
                }
                let overlap = a.y.max(b.y) < (a.y + a.height).min(b.y + b.height) - 1.0;
                assert!(
                    !overlap,
                    "{sample} {}쪽: 문단 {pa} 표({:.1}..{:.1})와 문단 {pb} 표({:.1}..{:.1})가 겹친다",
                    page + 1,
                    a.y,
                    a.y + a.height,
                    b.y,
                    b.y + b.height
                );
            }
        }
    }
}

#[test]
fn stored_float_frame_inside_previous_tac_line_starts_next_page() {
    let sample = "samples/issue8596/certificate_table.hwpx";
    let core = core(sample);
    assert_eq!(core.page_count(), 9, "한컴 Print PDF 9쪽");
    let tac = table_page(&core, 4);
    assert_eq!(
        table_page(&core, 5),
        tac + 1,
        "문단 5 자리차지 표는 문단 4 TAC 표 다음 쪽에서 시작"
    );
    assert_no_table_overlap(&core, sample);
}

#[test]
fn hwp5_stored_float_frame_rewinding_into_tac_line_starts_next_page() {
    let sample = "samples/issue8596/policy_committee_agenda.hwp";
    let core = core(sample);
    assert_eq!(core.page_count(), 14, "한컴 Print PDF 14쪽");
    let tac = table_page(&core, 110);
    assert_eq!(
        table_page(&core, 111),
        tac + 1,
        "문단 111 자리차지 표는 문단 110 TAC 표 다음 쪽에서 시작"
    );
    assert_no_table_overlap(&core, sample);
}

#[test]
fn stored_float_frame_below_finished_tac_line_stays_on_page() {
    let sample = "samples/issue8596/idol_goods_trimmed.hwpx";
    let core = core(sample);
    assert_eq!(core.page_count(), 2, "한컴 Print PDF 2쪽");
    let page = table_page(&core, 6);
    assert_eq!(
        table_page(&core, 12),
        page,
        "끝난 TAC 표 아래 저장 프레임은 같은 쪽"
    );
    let tables = page_tables(&core, page);
    let bbox = |para: usize| tables.iter().find(|(p, _)| *p == para).unwrap().1;
    let (tac, float) = (bbox(6), bbox(12));
    assert!(
        float.y >= tac.y + tac.height,
        "문단 12 표({:.1})는 문단 6 TAC 표 아래({:.1})",
        float.y,
        tac.y + tac.height
    );
    assert_no_table_overlap(&core, sample);
}
