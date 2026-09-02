//! [Issue #6204] 개체의 위치·크기가 바뀌어도 그 개체가 만드는 배제 밴드에 되감긴
//! 문단의 저장 `LINE_SEG` 가 갱신되지 않아, 본문이 **옛 개체 위치**에 되감긴 채 굳는다.
//! 게다가 그 사다리가 파일에 실려 **저장 → 새로 열기로도 재현**된다.
//!
//! 텍스트 편집(삽입/삭제)은 이미 `apply_body_edit_through_picture_band` 로 밴드를
//! 다시 투영하는데, **개체 속성 편집만 그 경로를 타지 않았다.**
//!
//! 이 표본은 어울림(Square) 그림을 단 문단이다. 그림을 왼쪽으로 옮기면 배제 밴드가
//! 넓어져 호스트 문단이 두 조각으로 되감겨야 한다.
//!
//! | | `horzOffset` | 저장 `ls.segment_width` |
//! |---|---|---|
//! | 이동 전 | 29015 | `[48188]` (전폭) |
//! | 이동 후 (수정 전) | 5000 | `[48188]` ✘ **그대로** |
//! | 이동 후 (수정 후) | 5000 | `[4434, 24403]` ✔ |
//! | 저장 → 재로드 | 5000 | `[4434, 24403]` ✔ |
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const SAMPLE: &str = "samples/issue6204/square_picture_band_host.hwp";
/// 어울림 그림을 단 호스트 문단과 그 안의 그림 컨트롤 인덱스.
const HOST_PARA: usize = 0;
const PICTURE_CTRL: usize = 1;
/// 옮길 목표 오프셋(HWPUNIT) — 원래 29015 에서 왼쪽으로.
const NEW_HORZ_OFFSET: i32 = 5000;

fn stored_segment_widths(core: &DocumentCore) -> Vec<i32> {
    core.document().sections[0].paragraphs[HOST_PARA]
        .line_segs
        .iter()
        .map(|seg| seg.segment_width)
        .collect()
}

fn horz_offset(core: &DocumentCore) -> Option<i32> {
    core.document().sections[0].paragraphs[HOST_PARA]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Picture(p) => Some(p.common.horizontal_offset as i32),
            _ => None,
        })
}

#[test]
fn issue_6204_moving_a_square_picture_reflows_its_band() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let before = stored_segment_widths(&core);
    assert_eq!(
        before.len(),
        1,
        "이동 전에는 전폭 한 줄이어야 한다 — 실측 {before:?}"
    );

    core.set_picture_properties_native(
        0,
        HOST_PARA,
        PICTURE_CTRL,
        &format!("{{\"horzOffset\":{NEW_HORZ_OFFSET}}}"),
    )
    .expect("set picture properties");

    assert_eq!(
        horz_offset(&core),
        Some(NEW_HORZ_OFFSET),
        "그림 오프셋은 반영돼야 한다"
    );
    let after = stored_segment_widths(&core);
    assert_ne!(
        after, before,
        "개체를 옮기면 그 배제 밴드에 되감긴 문단의 저장 LINE_SEG 도 다시 새겨야 한다 — \
         그대로 두면 본문이 옛 그림 위치에 되감긴 채 남는다 (실측 {after:?})"
    );

    // 저장본에도 갱신된 사다리가 실려야 한다 — 저장 → 재로드로 되돌아가면
    // 편집 결과가 파일에 굳어 다음에 열 때도 잘못된 조판이 재현된다.
    let bytes = core.export_hwp_native().expect("export");
    let reloaded = DocumentCore::from_bytes(&bytes).expect("reopen");
    assert_eq!(
        stored_segment_widths(&reloaded),
        after,
        "저장 → 재로드에도 갱신된 사다리가 유지돼야 한다"
    );
    assert_eq!(horz_offset(&reloaded), Some(NEW_HORZ_OFFSET));
}
