//! [Issue #7408] 저장 LineSeg 의 캐시 키를 **원점이 접힌 좌표계**와 견주는 바람에
//! 목록(번호·글머리표) 문단의 멀쩡한 저장 줄이 전부 재래핑됐다.
//!
//! ## 근인
//!
//! `ParagraphBox::effective()` 는 상자가 원점을 보류하면(`with_derivable_origin(false)`
//! — 그 자리에 적힌 목록 문단 차단막) 원점을 폭 안으로 접어 `0..width` 를 돌려준다.
//! 그런데 프레임은 그 사실을 몰랐고, 저장 `LineSeg` 는 여전히 **참 `column_start`** 를
//! 싣는다. 캐시 키(`stored_row_matches_frame_expectation`)는 둘을 정확히 같은지로 보므로
//! **그런 문단은 반드시 어긋난다** — 폭이 한 단위도 다르지 않은데도.
//!
//! ```text
//! footnote-01.hwp  (RHWP_DIAG_STORED_ROW_KEY)
//!   frame=0..46188   stored=[2000+46188 2000+46188]   폭 46188 동일, 원점만 접힘
//!   frame=0..45188   stored=[3000+45188 3000+45188]
//!   frame=0..47188   stored=[1000+47188]
//! ```
//!
//! 어긋나면 `Reflowed` 다. 그래서 **저장 줄은 rhwp 자신의 줄바꿈기가 같은 자리를 고를
//! 때만 지켜졌다.** `pi=4` 는 저장 컷 34 를 35 로 다시 짠다(한/글 정본은 34).
//!
//! 원점을 보류한 프레임이 그 원점을 문서 기록에 **불리한 증거로** 쓸 수는 없다. 그래서
//! 보류했을 때만 첫 슬롯이 이동량을 정하고, 나머지 양(폭과 슬롯 사이 간격)은 종전대로
//! 정확히 같은지로 본다. 커밋되는 기하는 어느 쪽이든 프레임 자신의 carve 이므로,
//! 렌더가 아직 소비하지 못하는 참 원점이 기록으로 새어 나가지 않는다.
//!
//! ## 기대값의 출처는 구현 밖이다
//!
//! 1. **문서 자신의 저장 사다리** — `LINE_SEG::text_start` 는 한/글이 저장 시점에 고른
//!    컷이다. 이 시험은 그 값을 기대값으로 쓰고, 구현이 계산한 폭은 쓰지 않는다.
//! 2. **한/글 정본** — `pdf/footnote-01-2022.pdf`·`endnote-01-2022.pdf` 도 저장 컷을
//!    전부 같은 자리에서 끊는다(`oracleReproducesStoredCut` 19/19 · 20/20).
//!    저장 사다리와 정본이 같은 답을 말한다.
//!
//! ```text
//!   본문 흐름에 그려진 저장 컷 문단      수정 전 -> 수정 후
//!     footnote-01.hwp                   15/24 -> 24/24
//!     endnote-01.hwp                    16/25 -> 25/25
//!     tac-img-02.hwp                    13/75 -> 74/75
//!
//!   정본 290쌍 전수(scripts/oracle_comparability.py)
//!     저장 끊음 재현 17,308 -> 17,732 (+424)  개선 9문서 · 악화 0
//! ```
//!
//! ## 적용 경계
//!
//! 위험은 하나다 — **재래핑이 실제로 필요한 저장 줄까지 지키면** 편집한 글자가 옛 컷을
//! 넘어 상자 밖으로 나간다. `edited_paragraph_is_rewrapped` 가 그 경계를 잠근다.
//! 폭이 다른 캐시는 이 변경과 무관하게 종전대로 거절된다 — 이동량은 첫 슬롯에서만
//! 나오고 폭은 여전히 정확히 같아야 한다(`stored_column_width_quantization` 계약).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn sample_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name)
}

/// 저장 사다리가 말하는 컷을 **보이는 글자 오프셋**으로 옮긴다.
///
/// `LineSeg::text_start` 는 문단의 UTF-16 축이고 확장 제어 하나가 8유닛을 차지한다.
/// 렌더 트리의 `char_start` 는 보이는 글자 번호이므로 `char_offsets` 로 투영한다.
/// 축 보정(`#5961` HWPX 구역 첫 문단)은 `line_seg_text_start_of` 가 갖는다.
///
/// 구현 속성(`TAG_IMPLEMENTATION_PROPERTY`) 줄은 저장본의 컷이 아니므로 뺀다.
fn stored_cuts(para: &Paragraph) -> Option<Vec<usize>> {
    let units: Vec<u32> = para
        .line_segs
        .iter()
        .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        .map(|seg| para.line_seg_text_start_of(seg.text_start))
        .collect();
    if units.len() < 2 || units.windows(2).any(|w| w[1] <= w[0]) {
        return None;
    }
    units
        .iter()
        .map(|unit| {
            para.char_offsets
                .iter()
                .position(|offset| offset >= unit)
                .or(Some(para.char_offsets.len()))
        })
        .collect()
}

/// 본문 흐름의 줄 시작 글자 오프셋을 문단별로 모은다.
///
/// 각주 영역·표·글상자는 자기 흐름이라 문단 번호 공간이 다르다 — 본문만 센다.
fn body_line_starts(node: &RenderNode, out: &mut std::collections::BTreeMap<usize, Vec<usize>>) {
    if matches!(
        node.node_type,
        RenderNodeType::FootnoteArea
            | RenderNodeType::Table(_)
            | RenderNodeType::TextBox
            | RenderNodeType::Header
            | RenderNodeType::Footer
            | RenderNodeType::MasterPage
    ) {
        return;
    }
    if let RenderNodeType::TextLine(_) = &node.node_type {
        let first = node
            .children
            .iter()
            .find_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some((run.para_index?, run.char_start?)),
                _ => None,
            });
        if let Some((para_index, char_start)) = first {
            out.entry(para_index).or_default().push(char_start);
        }
        return;
    }
    for child in &node.children {
        body_line_starts(child, out);
    }
}

/// 저장 컷을 그대로 재현한 본문 문단 수 / 저장 컷을 가진 본문 문단 수.
fn reproduced_stored_cuts(name: &str) -> (usize, usize) {
    let bytes = std::fs::read(sample_path(name)).expect("정식 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page_count = core.page_count();

    let mut rendered: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for page in 0..page_count {
        let tree = core.build_page_render_tree(page).expect("render tree");
        body_line_starts(&tree.root, &mut rendered);
    }

    let paragraphs = core.document_mut().sections[0].paragraphs.clone();
    let mut total = 0usize;
    let mut hit = 0usize;
    for (index, para) in paragraphs.iter().enumerate() {
        let Some(expected) = stored_cuts(para) else {
            continue;
        };
        // 본문 흐름에 나타나지 않은 문단(다른 흐름 소속)은 증거가 아니다.
        let Some(actual) = rendered.get(&index) else {
            continue;
        };
        total += 1;
        if actual.len() == expected.len() && actual.iter().zip(&expected).all(|(a, b)| a == b) {
            hit += 1;
        } else {
            eprintln!("[7408miss] {name} pi={index} 저장={expected:?} 실제={actual:?}");
        }
    }
    (hit, total)
}

/// 저장 사다리와 정본이 같은 자리에서 끊는 문단을 rhwp 도 전부 끊는다.
///
/// 수정 전 15/24 · 수정 후 24/24. 한/글 정본(`footnote-01-2022.pdf`)도 저장 컷을
/// 전부 지킨다(`oracleReproducesStoredCut` 19/19, 같은 축을 다른 표본 폭으로 센 값).
#[test]
fn footnote_sample_keeps_every_stored_cut() {
    let (hit, total) = reproduced_stored_cuts("footnote-01.hwp");
    assert_eq!(
        total, 24,
        "표본이 바뀌었다 — 본문에 그려진 저장 컷 문단이 24개가 아니다"
    );
    assert_eq!(
        hit, total,
        "저장 LineSeg 의 컷을 재현하지 못한 문단이 있다(수정 전 15/24). \
         원점을 보류한 프레임과 저장 기록의 좌표계가 다시 어긋났는지 확인하라 \
         (RHWP_DIAG_STORED_ROW_KEY=1)"
    );
}

/// 미주 표본도 같다 — 수정 전 16/25 · 수정 후 25/25.
#[test]
fn endnote_sample_keeps_every_stored_cut() {
    let (hit, total) = reproduced_stored_cuts("endnote-01.hwp");
    assert_eq!(total, 25, "표본이 바뀌었다 — 저장 컷 문단이 25개가 아니다");
    assert_eq!(
        hit, total,
        "저장 LineSeg 의 컷을 재현하지 못했다(수정 전 16/25)"
    );
}

/// 글자처럼 취급한 그림이 섞인 문단도 같은 이득을 본다 — 13/75 → 74/75.
///
/// 남는 1건(`pi=837`)은 저장 컷 `[0, 55]` 인데 본문에 첫 줄만 그려진다. 이 변경과
/// 다른 축(조각 소유)이라 미해결로 남긴다.
#[test]
fn tac_picture_sample_keeps_its_stored_cuts() {
    let (hit, total) = reproduced_stored_cuts("tac-img-02.hwp");
    assert_eq!(total, 75, "표본이 바뀌었다 — 본문에 그려진 저장 컷 문단 수");
    assert!(
        hit >= 74,
        "글자처럼 취급 그림이 섞인 문단의 저장 컷 재현이 줄었다 — {hit}/{total} (수정 전 13)"
    );
}

/// **적용 경계** — 글자를 넣어 저장 사다리가 낡으면 그 문단은 다시 짠다.
///
/// 이 수정이 지키는 것은 *멀쩡한* 저장 줄이다. 낡은 줄까지 지키면 편집한 글자가
/// 옛 컷을 넘어 상자 밖으로 나간다. `stale` 판정(`stored_rows_are_stale`)은 이
/// 술어보다 앞에 있으므로 여기서 함께 잠근다.
///
/// `footnote-01.hwp` pi=4 는 저장 컷이 `[0, 34]` 인 두 줄 문단이다. 첫 줄 안에
/// 글자 40 개를 넣으면 같은 폭에 그 글자열이 들어갈 수 없다 — 줄이 늘거나 컷이
/// 뒤로 밀려야 하고, 저장 컷 `[0, 34]` 를 그대로 쓰면 안 된다.
#[test]
fn edited_paragraph_is_rewrapped() {
    let bytes = std::fs::read(sample_path("footnote-01.hwp")).expect("정식 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let before = stored_cuts(&core.document_mut().sections[0].paragraphs[4])
        .expect("pi=4 는 저장 컷을 가진 문단이다");
    assert_eq!(before, vec![0, 34], "표본이 바뀌었다 — pi=4 의 저장 컷");

    core.insert_text_native(0, 4, 5, &"가".repeat(40))
        .expect("글자 삽입");

    let page_count = core.page_count();
    let mut rendered: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for page in 0..page_count {
        let tree = core.build_page_render_tree(page).expect("render tree");
        body_line_starts(&tree.root, &mut rendered);
    }
    let actual = rendered.get(&4).expect("pi=4 본문 줄");
    assert_ne!(
        actual, &before,
        "글자를 40개 넣었는데 낡은 저장 컷을 그대로 썼다 — stale 재래핑이 막혔다"
    );
    assert!(
        actual.len() > before.len() || actual[1] != before[1],
        "편집 뒤에도 줄 수·컷이 그대로다: {actual:?}"
    );
}
