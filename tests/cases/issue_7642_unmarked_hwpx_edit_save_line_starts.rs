//! [#7642] 생산자 마커가 없는 HWPX의 구역 첫 문단을 편집해 저장해도 편집 결과가 보존된다.
//!
//! 구역 정의·단 정의·본문 폭에 가까운 글자처럼 취급 표로 시작하는 구역 첫 문단 맨 앞에
//! 글자를 넣으면 문단은 [글자 줄, 표 줄] 두 줄이 된다. `ef642bb0e` 이전에는 rhwp 생산자
//! 마커가 없는 HWPX도 다시 읽을 때 `hp:secPr` 슬롯 한 칸(8유닛)을 더 올려 보았다. 그래서
//! 이 문단을 HWPX로 저장해 다시 열면 둘째 줄 시작이 `[0, 23]` 대신 `[0, 31]`이 되어
//! 넣은 글자가 첫 쪽에서 사라졌다.
//!
//! `#7526`은 마커가 있는 HWPX의 편집→저장→다시 열기만, `#5961`은 파싱 시점의 축 투영만
//! 보호한다. 이 파일은 마커 없는 HWPX의 같은 경로와, HWPX로 다시 연 문서의 HWP 저장을
//! 모델 수준의 왕복 항등으로 지킨다. 기대값은 한컴 출력이 아니라 메모리 안의 편집 결과다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// `samples/tac-host-spacing.hwpx` 파생: 책갈피 제거, 1×1 글자처럼 취급 표 폭 47000,
/// 생산자 마커 없음. 출처는 같은 폴더의 `README.md`.
const FIXTURE: &str = "tests/fixtures/issue_7642_unmarked_hwpx_edit_save/wide_tac_table.hwpx";
const MARKER: &str = "MARKERX";

/// 구역 0 문단 0의 줄 시작(HWP5 문단 축)과 글자.
#[derive(Debug, PartialEq, Eq)]
struct FirstParagraph {
    line_starts: Vec<u32>,
    text: String,
}

fn first_paragraph(core: &DocumentCore) -> FirstParagraph {
    let para = &core.document().sections[0].paragraphs[0];
    FirstParagraph {
        line_starts: (0..para.line_segs.len())
            .map(|i| para.line_seg_text_start(i))
            .collect(),
        text: para.text.clone(),
    }
}

/// 픽스처를 열어 구역 첫 문단 맨 앞에 글자를 넣고, 그 편집 결과를 기대값으로 돌려준다.
fn edited_fixture() -> (DocumentCore, FirstParagraph) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("픽스처")).expect("HWPX 열기");
    core.insert_text_native(0, 0, 0, MARKER)
        .expect("구역 첫 문단 맨 앞에 글자 입력");
    let edited = first_paragraph(&core);
    assert!(
        edited.line_starts.len() >= 2,
        "전제: 넓은 표 앞에 글자를 넣으면 [글자 줄, 표 줄] 두 줄 이상이다, got {edited:?}"
    );
    (core, edited)
}

fn reopen_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장")).expect("HWPX 열기")
}

fn reopen_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_native().expect("HWP 저장")).expect("HWP 열기")
}

#[test]
fn unmarked_hwpx_edit_keeps_first_paragraph_line_starts_after_hwpx_save() {
    let (core, edited) = edited_fixture();
    assert_eq!(
        first_paragraph(&reopen_hwpx(&core)),
        edited,
        "마커 없는 HWPX로 저장해 다시 연 구역 첫 문단이 편집 결과와 다르다 — 표 줄이 8유닛 늦게 시작하면 넣은 글자가 사라진다"
    );
}

#[test]
fn hwp_save_of_hwpx_reopened_document_keeps_first_paragraph_line_starts() {
    let (core, edited) = edited_fixture();
    assert_eq!(
        first_paragraph(&reopen_hwp(&reopen_hwpx(&core))),
        edited,
        "HWPX로 다시 연 문서를 HWP로 저장해 다시 연 구역 첫 문단이 편집 결과와 다르다"
    );
}
