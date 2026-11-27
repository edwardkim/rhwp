//! 관계형·고정형 — [#9000] 칸 안 중첩 표가 한/글이 저장한 쪽 프레임 경계에서 이어지는 큰 표의 쪽 컷·행 조각 상자.
//!
//! 주석 갱신: 2026-11-25 10:05 KST(+09:00), 새 파일(devel 0fbee2c04e rebase 재측정 반영).
//!
//! - 입력: `samples/issue9000/kaf_table_section.hwp` — 10k 공개 코퍼스의 KAF 총회 회의자료(docs#881,
//!   SHA-256 `8014ae9b…f6c9e5`, 66쪽, 저장소에 넣지 않음)에서 rhwp `edit delete-paragraph` 로
//!   구역 1 문단 33~244(표 뒤 45~66쪽 내용)를 지운 파생본(44쪽). 1~44쪽 조판 항목은 원본과 같고
//!   44쪽 끝에 원본 마지막 빈 문단만 남는다. 원본의 부록 쪽(48·50~52·54·64쪽)은 표와 무관한
//!   별개 결함(#9379)으로 90% 미만이라 파생본으로 검증한다.
//! - 기준: `pdf/issue9000/kaf_table_section-2020.pdf`(한/글 2020 11.0.0.1623, `Hancom PDF` Print,
//!   `hancom2020_pdf_driver_one_up`, 44쪽, SHA-256 `3c526bf9…2c09c62ca182b55d9ed899`).
//! - 문단 1.32 의 11×6 RowBreak 표가 33~44쪽에 걸친다. 칸 r·c5 의 중첩 표 행은 한/글이 쪽마다
//!   되감은 저장 프레임(vpos 0 재시작) 유닛에서 다음 쪽으로 이어진다.
//!
//! 검사:
//! - 고정형(쪽 소속): 한컴 PDF 의 쪽 경계 근처 글(`Standard : KAF address standard`·`landmark`·
//!   `surname, given` 등)이 PDF 와 같은 쪽에 같은 횟수로 있다. 수정 전 rhwp 는 이어받는 조각의
//!   행 높이를 크게 재(쪽을 넘어 닫힌 블록 표 뒤 문단 앞 간격 + 이어받는 자식 위 바깥 여백 이중
//!   계상) 한 유닛 일찍 끊어 이 글들을 한 쪽 뒤로 밀었다. 전체 쪽수는 한컴 44쪽이다.
//! - 관계형(행 조각 상자): 다음 쪽으로 이어지는 행 조각(34~42쪽)의 표 아래 경계는 모두 같은
//!   쪽 물리 프레임 끝이고, 앞 조각이 선언 행 높이를 거의 다 쓴 43쪽은 그보다 위 내용 끝에서
//!   닫힌다(한컴 PDF 34~42쪽 695px, 43쪽 681px). 수정 전 rhwp 는 내용 끝에서 멈춰 쪽마다 달랐다.
//! - 두 검사 모두 수정 전(devel `9ee9a5b0b1`·`2c0c1d205a`)에 실패한다(35쪽 아래 경계 674.55px ≠ 34쪽 679.72px,
//!   `Standard : KAF address standard` 가 34쪽).
//!
//! 측정(2026-11-24, base = devel `9ee9a5b0b1`, 후보 = base + 이 수정. devel `2c0c1d205a` 와 그 위로 rebase 한
//! 후보의 재측정 TSV 도 바이트 단위로 같다. devel `0fbee2c04e` 와 그 위로 rebase 한 후보(충돌 해소 뒤)의 재측정
//! TSV 4개도 바이트 단위로 같고, 두 검사는 `0fbee2c04e` 에서 실패하고 후보에서 통과한다.
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴은 스크립트 기본 공급):
//! - Native: 44쪽 / 44쪽. 33~44쪽 base 92.95·32.84·48.94·40.85·53.35·40.09·56.76·47.85·38.18·59.09·
//!   62.36·59.20 → 100.00·99.97·99.19·98.35·99.96·98.96·99.98·100.00·99.96·99.99·100.00·99.98.
//!   전쪽 최저 base 32.84(34쪽) → 후보 94.32(1쪽, 변경 없는 쪽).
//! - fresh WASM: 44쪽 / 44쪽. 33~44쪽 base 93.01·33.02·49.17·40.94·53.16·40.62·56.61·48.34·38.46·59.22·
//!   62.38·58.95 → 100.00·99.95·99.16·98.42·99.90·98.99·99.93·
//!   99.99·99.90·99.94·100.00·99.98. 전쪽 최저 base 33.02(34쪽) → 후보 93.13(1쪽, 변경 없는 쪽).
//! - 행 조각 아래 경계는 관계로만 검사한다(절대 px 고정 없음).
//! - 증거: `mydocs/pr/assets/issue_9000/`(native·wasm·base 전쪽 TSV, 34·37·43쪽 review·overlay PNG,
//!   base 34·37쪽 review PNG)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9000/kaf_table_section.hwp";
/// 한컴 PDF 의 전체 쪽수.
const HANCOM_PAGES: u32 = 44;
/// 큰 표가 걸친 쪽(1부터).
const TABLE_PAGES: std::ops::RangeInclusive<u32> = 33..=44;
/// (글, 한컴 PDF 에서 그 글이 놓인 쪽들). 각 쪽에 한 번씩 있다.
const PAGE_OWNED_TEXT: &[(&str, &[u32])] = &[
    ("Standard:KAFaddressstandard", &[33]),
    ("Yeongdeungpo-gu", &[35, 39]),
    ("Road,London", &[36]),
    ("landmark", &[37]),
    ("Chiswick", &[35, 36, 38]),
    ("Sampleaddress", &[34, 36, 38, 40, 41]),
    ("MsCFitzgerald", &[36, 38, 41]),
    ("AS4590", &[34, 35, 36, 38, 41]),
    ("surname,given", &[42]),
];
/// 큰 표: 구역 1 문단 32, 11×6.
const TABLE_PARA: usize = 32;

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");
    core
}

#[test]
fn page_boundary_text_stays_on_hancom_pages() {
    let core = open();
    let page_texts: Vec<(u32, String)> = TABLE_PAGES
        .map(|page| {
            (
                page,
                flat(&core.extract_page_text_native(page - 1).unwrap_or_default()),
            )
        })
        .collect();
    for (text, pages) in PAGE_OWNED_TEXT {
        let found: Vec<(u32, usize)> = page_texts
            .iter()
            .filter_map(|(page, page_text)| {
                let page = *page;
                let count = page_text.matches(text).count();
                (count > 0).then_some((page, count))
            })
            .collect();
        let expected: Vec<(u32, usize)> = pages.iter().map(|&page| (page, 1)).collect();
        assert_eq!(
            found, expected,
            "`{text}` 의 쪽 소속이 한컴 PDF 와 다르다 — 이어받는 행 조각을 저장 쪽 프레임보다 일찍 끊었다"
        );
    }
}

fn outer_table_bottom(node: &RenderNode) -> Option<f64> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(TABLE_PARA) && table.row_count == 11 && table.col_count == 6 {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children.iter().find_map(outer_table_bottom)
}

#[test]
fn continued_row_fragments_own_the_page_frame_bottom() {
    let core = open();
    let bottom = |page: u32| {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        outer_table_bottom(&tree.root).unwrap_or_else(|| panic!("{page}쪽 큰 표"))
    };
    let frame = bottom(34);
    for page in 35..=42 {
        let current = bottom(page);
        assert!(
            (current - frame).abs() <= 0.5,
            "{page}쪽: 이어지는 행 조각의 아래 경계 {current:.2}px 가 34쪽 프레임 끝 {frame:.2}px 와 다르다 \
             — 칸 안 중첩 표의 저장 쪽 프레임 경계에서 끊긴 행 조각은 쪽 물리 프레임 끝까지 상자를 소유한다"
        );
    }
    let closed = bottom(43);
    assert!(
        closed < frame - 5.0,
        "43쪽은 앞 조각이 선언 행 높이를 거의 다 써 내용 끝({closed:.2}px)에서 닫혀야 한다(프레임 끝 {frame:.2}px)"
    );
}
