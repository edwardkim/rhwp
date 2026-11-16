//! 관계형 — [#8998] HWPX 저장에서 미기입 누름틀 안내문 잔재를 되살린 뒤에도 뒤따르는 글자 모양 경계는 원본 IR 위치에 남는다.
//! 주석 갱신: 2026-11-12 (새 파일)
//!
//! 입력: `samples/issue-986-receipt.hwp` 첫 표 8번 칸(2행 10열) — 미기입(`dirty=0`) 누름틀 두 개
//! 「야간 진료 여부」「공휴일 진료 여부」가 글 「야간 」 앞뒤에 있다. 적재 정규화는 안내문을 값에서
//! 빼고 그 글자 모양을 같은 위치의 0폭 run 으로 남긴다(원본 IR 글자 모양 경계
//! `(0,5) (8,17) (8,5) (27,17) (27,5)`). HWPX 저장은 안내문을 run 으로 되살리는데, 첫 잔재(8 UTF-16)
//! 뒤로 직렬화 위치만 밀리고 경계는 IR 위치 그대로라 두 번째 누름틀 시작 컨트롤보다 경계 27 이 먼저
//! 잘렸다 — 다시 열면 `(19,17) (19,5)` 로 옮겨지고 두 번째 잔재의 글자 모양이 17 → 5 로 바뀌었다.
//! 기대값은 원본 HWP 를 읽은 IR 이다(저장→적재 고정점). 시각 출력과 무관한 저장 계약 검사다.
//!
//! 측정(2026-11-12, 코드 head d257f9401f, base 80b1c354c2, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, `RHWP_FONT_PATH` 글꼴 공급, `--compat 2022`, 한컴 engine 2020 Print):
//! - 내보낸 HWPX Native 1쪽 / 1쪽: 57.09, fresh WASM 1쪽 / 1쪽: 57.63. 원본 HWP 를 devel 로 그려도
//!   57.09 인 기존 미달(큰 표 칸 경계)이라 #9005 로 분리했다. 이 검사는 쪽 배치가 아니라 저장 계약만 본다.
//! - 증거: `mydocs/pr/assets/issue_8998/`, 기준 PDF `pdf/issue8998/`
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

const SAMPLE: &str = "samples/issue-986-receipt.hwp";

fn night_holiday_cell(core: &DocumentCore) -> &Paragraph {
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
        panic!("첫 문단 세 번째 컨트롤은 표");
    };
    let cell = &table.cells[8];
    assert_eq!((cell.row, cell.col), (2, 10), "8번 칸은 2행 10열");
    &cell.paragraphs[0]
}

/// (글자 모양 경계, 누름틀 범위, 누름틀별 안내문 잔재와 그 글자 모양)
type Contract = (
    Vec<(u32, u32)>,
    Vec<(usize, usize)>,
    Vec<Option<(String, u32)>>,
);

fn contract(paragraph: &Paragraph) -> Contract {
    (
        paragraph
            .char_shapes
            .iter()
            .map(|shape| (shape.start_pos, shape.char_shape_id))
            .collect(),
        paragraph
            .field_ranges
            .iter()
            .map(|range| (range.start_char_idx, range.end_char_idx))
            .collect(),
        paragraph
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Field(field) => Some(
                    field
                        .guide_residue
                        .as_ref()
                        .map(|residue| (residue.text.clone(), residue.char_shape_id)),
                ),
                _ => None,
            })
            .collect(),
    )
}

#[test]
fn hwpx_export_keeps_char_shape_boundaries_after_restored_guide_residue() {
    let bytes = std::fs::read(SAMPLE).expect("sample");
    let original = DocumentCore::from_bytes(&bytes).expect("open hwp");
    let before = contract(night_holiday_cell(&original));
    assert_eq!(
        before.2.len(),
        2,
        "미기입 누름틀 두 개(야간·공휴일 진료 여부)"
    );
    assert!(
        before.2.iter().all(Option::is_some),
        "두 누름틀 모두 안내문 잔재를 가진다: {before:?}"
    );
    let reopened = DocumentCore::from_bytes(&original.export_hwpx_native().expect("export hwpx"))
        .expect("open hwpx");
    let after = contract(night_holiday_cell(&reopened));
    assert_eq!(
        after, before,
        "HWPX 저장→적재 뒤 칸 문단 글자 모양·누름틀 계약"
    );
}
