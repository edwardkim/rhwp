//! Issue #2097 — RowBreak rowspan 블록 쪽 하단 밴드 필의 쪽수 핀.
//!
//! 병인: 블록이 쪽 하단 잔여를 초과할 때 plain 블록 컷 walk 는 행 시작 y 를
//! 무시하고 셀-로컬 높이만 봐서 fully_consumed 로 오판 → 분할 기각 → 통이월
//! (3248363 b=6..8: block_h 661.8 > 잔여 540.7 인데 fully=true → 쪽 2 가 46%
//! 만 사용). 한글은 이 경계에서 행 오프셋 기준 밴드 컷으로 쪽을 채운다.
//! 수정: fully_consumed 인데 블록 밴드가 예산을 초과하는 쪽 하단 경계에서
//! advance_row_block_cut_with_row_offsets 로 재시도 (RowBreak + 신규 조각 +
//! MIN_TOP_KEEP 한정).
//! 정답 쪽수는 한글 2022 COM PageCount 실측 (3248363 은 쪽 2/3 경계 내용의
//! 한글 PDF 글자 단위 정합 동반).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn page_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.extend(
            run.text
                .chars()
                .filter(|character| !character.is_whitespace()),
        );
    }
    for child in &node.children {
        page_text(child, text);
    }
}

/// 독립 한컴2020 PDF의 실제 문단 소속을 검사한다. 좌표·픽셀은 고정하지 않는다.
fn assert_simsa_paragraph_ownership(core: &DocumentCore) {
    let pages: Vec<String> = (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("심사지표 쪽 렌더");
            let mut text = String::new();
            page_text(&tree.root, &mut text);
            text
        })
        .collect();
    for (paragraph, owners) in [
        (
            "1-3.설치·운영자의이전장기요양기관평가결과(10점)",
            &[2, 6][..],
        ),
        ("3-1.운영위원회구성,개최,활용계획의적정성(5점)", &[3][..]),
        (
            "직원고충처리제도,처우개선을위한복지제도운영과건강관리",
            &[3, 7][..],
        ),
        ("등을위한운영계획마련여부를확인", &[4][..]),
        ("3-1.예산수립의적정성등재무상태의건전성(3점)", &[7][..]),
        ("4-3.복지용구관리에관한사항(4점)", &[8][..]),
    ] {
        for (index, text) in pages.iter().enumerate() {
            assert_eq!(
                text.matches(paragraph).count(),
                usize::from(owners.contains(&(index + 1))),
                "{paragraph}: 한컴 PDF의 문단 소속과 단일 출력({}쪽)",
                index + 1
            );
        }
    }
    assert!(
        pages[3]
            .find("등을위한운영계획마련여부를확인")
            .expect("앞 문단의 이어받기")
            < pages[3].find("5-1.지역사회").expect("다음 평가 항목"),
        "4쪽은 앞 문단의 이어받기를 끝낸 뒤 다음 항목을 표시"
    );
    assert!(
        pages[6].find("3-1.예산수립").expect("예산 항목")
            < pages[6]
                .find("3-2.직원건강관리")
                .expect("직원 건강관리 항목"),
        "7쪽 항목 순서 보존"
    );
}

/// (샘플, 한글 실측 쪽수)
const PINS: &[(&str, u32)] = &[
    // 블록 6..8 통이월 → 밴드 컷 (5→4쪽, p2 만충 1003.5/1009px)
    ("samples/task2097/3248363_upmu_bunjang.hwpx", 4),
    // 동계열 다중 블록 통이월 누적 (11→8쪽, delta +3 전량 해소)
    ("samples/task2097/21217935_simsa_jipyo.hwp", 8),
    // 동계열 (22→21쪽). debug/release 동일 (21 = 한글 COM 실측).
    ("samples/task2097/18095317_eogu_geumji.hwp", 21),
    // 쪽나눔=None 표의 fresh-쪽 초과 통째 배치 (6→2쪽). 선언=실측 1005px >
    // 본문 933.5px 인 표 2건을 한글은 각 1쪽 통째 + 하단 오버플로로 렌더
    // (한글 PDF 실측) — rhwp 는 각 3조각(헤더/본체/꼬리 sliver)으로 분할했다.
    ("samples/task2097/3023771_wichokjang.hwpx", 2),
    // 나란히 TopAndBottom float union 예약 (2→1쪽). 좌우 배치 그림 2개
    // (off 31/35px, h 359/335px, 세로 band 겹침)를 각자 높이 합산 예약해
    // 페이지 1226px 이중 예약 → trailing 빈 문단이 여분 페이지로. 렌더는
    // 두 그림 나란히 정상. union 예약으로 pi=9 가 1쪽에 남는다.
    ("samples/task2097/17809123_jawonbongsa.hwpx", 1),
];

#[test]
fn issue_2097_block_band_fill_page_pins() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    for (sample, expected) in PINS {
        let bytes = fs::read(Path::new(repo_root).join(sample))
            .unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let core =
            DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
        assert_eq!(
            core.page_count(),
            *expected,
            "{sample}: 한글 COM 실측 쪽수와 불일치"
        );
        if *sample == "samples/task2097/21217935_simsa_jipyo.hwp" {
            assert_simsa_paragraph_ownership(&core);
        }
    }
}
