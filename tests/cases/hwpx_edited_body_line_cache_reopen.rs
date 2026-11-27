//! 관계형: 본문 문단을 고친 HWPX 저장본을 다시 열면 문단마다 편집 세션과 같은 쪽에 놓인다.
//! 2026-11-24 15:55 KST 갱신. 직전 커밋은 2026-11-24 09:21:25 KST의 b3301e1905edd7fe504fe400c3560e27c421ad33이다.
//! 2026-11-24 한컴 2024의 원본 및 편집 문단 24·112 저장본 Print를 96 DPI에서 비교했다. 세 입력은 각각 24쪽이며
//! 각 문서의 보정 전 2px 실루엣 전쪽 최저는 Native 99.09%, fresh WASM 99.14%다.
//! 쪽별 근거는 `mydocs/pr/assets/semanticist21-9337-9354-9377/edited-body-lines.tsv`에 있다.
//!
//! 근거: HWPX 에서 연 문서는 원본 줄의 세로 위치를 스냅샷으로 지니고, 렌더는 구역 누적 좌표를
//! 쓴다(#5847). 본문 문단을 고치면 그 문단은 스냅샷을 잃고 다시 조판한 줄만 남는다. 저장기가
//! 그 줄을 렌더 좌표 그대로 실으면, 이웃 문단의 원본 좌표와 어긋난 값이 파일에 들어가 다시 열 때
//! 저장된 쪽·단 경계로 읽힌다. `samples/HWP5-nopassword-123456.hwpx` 구역 0 문단 24 는 렌더
//! 좌표 88904, 원본 좌표 2984 로 두 좌표계가 갈린 문단이다.
//!
//! 절대 좌표·쪽 수를 고정하지 않고, 같은 문서의 편집 세션과 다시 연 저장본 사이의 쪽 소속만
//! 견준다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/HWP5-nopassword-123456.hwpx";

fn pages(core: &DocumentCore) -> Vec<String> {
    (0..core.document().sections[0].paragraphs.len())
        .map(|para| {
            core.get_page_of_position_native(0, para)
                .expect("본문 문단의 쪽 소속")
        })
        .collect()
}

#[test]
fn edited_body_paragraph_keeps_page_ownership_after_hwpx_reopen() {
    let bytes = std::fs::read(SAMPLE).expect("샘플을 읽는다");
    // 문단 24는 누적 좌표이고 112는 새 쪽에서 세로 위치가 다시 시작한다.
    for paragraph in [24, 112] {
        assert_page_ownership_after_edit(&bytes, paragraph);
    }
}

fn assert_page_ownership_after_edit(bytes: &[u8], paragraph: usize) {
    let mut core = DocumentCore::from_bytes(bytes).expect("샘플을 연다");
    let para = &core.document().sections[0].paragraphs[paragraph];
    let source = para
        .source_line_seg_vertical_pos
        .as_ref()
        .expect("원본 줄 위치 스냅샷이 있는 문단");
    if paragraph == 24 {
        assert_ne!(source[0], para.line_segs[0].vertical_pos);
    } else {
        // 원본의 문단 110·111에는 저장 줄이 없다. 직전 유효한 줄은 문단 109에 있다.
        let preceding = &core.document().sections[0].paragraphs[..paragraph];
        assert!(preceding
            .last()
            .unwrap()
            .source_line_seg_vertical_pos
            .is_none());
        let previous_stored = preceding
            .iter()
            .rev()
            .find_map(|p| p.source_line_seg_vertical_pos.as_ref()?.last())
            .expect("앞 문단 중 마지막 저장 줄");
        assert!(
            source[0] < *previous_stored,
            "쪽 경계에서 저장 세로 위치가 다시 시작한다"
        );
    }
    let offset = usize::from(!para.text.is_empty());
    let untouched = pages(&core);

    // 글자 하나를 넣었다 지운다. 글은 그대로이고 문단만 다시 조판한 줄을 갖는다.
    core.insert_text_native(0, paragraph, offset, "가")
        .expect("글자를 넣는다");
    core.delete_text_native(0, paragraph, offset, 1)
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
    let original = DocumentCore::from_bytes(bytes).expect("샘플을 다시 연다");
    assert_eq!(
        original.document().sections[0].paragraphs.len(),
        reopened.document().sections[0].paragraphs.len()
    );
    for (index, (before, after)) in original.document().sections[0]
        .paragraphs
        .iter()
        .zip(&reopened.document().sections[0].paragraphs)
        .enumerate()
    {
        assert_eq!(before.text, after.text, "문단 {index}의 본문");
        let stored = after.source_line_seg_vertical_pos.as_ref();
        if index == paragraph {
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
