//! 관계형: 본문 문단을 고친 HWPX 저장본을 다시 열면 문단마다 편집 세션과 같은 쪽에 놓인다.
//! 주석 갱신: 2026-11-24 (KST).
//!
//! 근거: HWPX 에서 연 문서는 원본 줄의 세로 위치를 스냅샷으로 지니고, 렌더는 구역 누적 좌표를
//! 쓴다(#5847). 본문 문단을 고치면 그 문단은 스냅샷을 잃고 다시 조판한 줄만 남는다. 저장기가
//! 그 줄을 렌더 좌표 그대로 실으면, 이웃 문단의 원본 좌표와 어긋난 값이 파일에 들어가 다시 열 때
//! 저장된 쪽·단 경계로 읽힌다. `samples/HWP5-nopassword-123456.hwpx` 구역 0 문단 24 는 렌더
//! 좌표 88904, 원본 좌표 2984 로 두 좌표계가 갈린 문단이다.
//!
//! 절대 좌표·쪽 수를 고정하지 않고, 같은 문서의 편집 세션과 다시 연 저장본 사이의 쪽 소속만
//! 견준다. 시각 측정 수치는 쓰지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/HWP5-nopassword-123456.hwpx";
const PARAGRAPH: usize = 24;

fn pages(core: &DocumentCore) -> Vec<String> {
    (0..core.document().sections[0].paragraphs.len())
        .map(|para| {
            core.get_page_of_position_native(0, para)
                .unwrap_or_else(|error| format!("{error:?}"))
        })
        .collect()
}

#[test]
fn edited_body_paragraph_keeps_page_ownership_after_hwpx_reopen() {
    let bytes = std::fs::read(SAMPLE).expect("샘플을 읽는다");
    let mut core = DocumentCore::from_bytes(&bytes).expect("샘플을 연다");
    let para = &core.document().sections[0].paragraphs[PARAGRAPH];
    let source = para
        .source_line_seg_vertical_pos
        .as_ref()
        .expect("원본 줄 위치 스냅샷이 있는 문단");
    assert_ne!(
        source[0], para.line_segs[0].vertical_pos,
        "렌더 좌표와 원본 좌표가 갈린 문단이어야 이 계약을 검사한다"
    );
    let untouched = pages(&core);

    // 글자 하나를 넣었다 지운다. 글은 그대로이고 문단만 다시 조판한 줄을 갖는다.
    core.insert_text_native(0, PARAGRAPH, 1, "가")
        .expect("글자를 넣는다");
    core.delete_text_native(0, PARAGRAPH, 1, 1)
        .expect("글자를 지운다");
    let edited = pages(&core);
    assert_eq!(edited, untouched, "글이 같으면 편집 세션의 쪽 소속도 같다");

    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 로 저장한다"))
        .expect("저장본을 다시 연다");
    assert_eq!(pages(&reopened), edited, "다시 연 저장본의 쪽 소속");

    // 다시 연 문서를 한 번 더 저장해 열어도 같다.
    let resaved = DocumentCore::from_bytes(&reopened.export_hwpx_native().expect("다시 저장한다"))
        .expect("다시 저장한 문서를 연다");
    assert_eq!(pages(&resaved), edited, "다시 저장한 문서의 쪽 소속");

    // 파일에 실린 줄 위치는 원본 좌표계다. 고친 문단은 줄 위치를 싣지 않거나(다시 계산)
    // 원본과 같은 값을 싣고, 고치지 않은 문단은 원본 값을 그대로 지닌다.
    let original = DocumentCore::from_bytes(&bytes).expect("샘플을 다시 연다");
    for (index, (before, after)) in original.document().sections[0]
        .paragraphs
        .iter()
        .zip(&reopened.document().sections[0].paragraphs)
        .enumerate()
    {
        let stored = after.source_line_seg_vertical_pos.as_ref();
        if index == PARAGRAPH {
            assert!(
                stored.is_none() || stored == before.source_line_seg_vertical_pos.as_ref(),
                "고친 문단이 렌더 좌표를 파일에 실었다: {stored:?}"
            );
        } else {
            assert_eq!(
                stored,
                before.source_line_seg_vertical_pos.as_ref(),
                "문단 {index} 의 원본 줄 위치"
            );
        }
    }
}
