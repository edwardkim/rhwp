# 완료 보고서 — Task M100-3161

- 이슈: #3161
- 제목: HWPX→HWP5 변환: 표 CTRL_HEADER "표 번호" 비트(0x0800_0000)가 numberingType 과 무관하게 무조건 설정
- 작성일: 2026-07-23
- 브랜치: `fix/3161-table-numbering-bit-gate`

## 1. 문제

HWPX→HWP5 변환 어댑터 `materialize_table_ctrl_header_attr()`
(`src/document_core/converters/hwpx_to_hwp.rs`)가 표 CTRL_HEADER attr 를 재합성할 때
"표 번호" 범주 비트(`0x0800_0000`, bit 27)를 numberingType 과 무관하게 무조건 OR 했다.

HWPX 파서는 #2697 에서 `materialize_hwpx_table_attrs`(`src/parser/hwpx/section.rs`)에
`numbering_type == Table` 게이트를 걸어 IR 모순(`numbering_type=Picture ↔ attr=TABLE`)을
제거했으나, HWP5 저장 경로의 변환 계층이 파서가 계산한 `common.attr` 를 무조건-OR 값으로
덮어써 같은 모순을 저장 시점에 재도입했다. 결과적으로 `numberingType="PICTURE"`
(그림 번호 캡션 표)·`"NONE"`(번호 제외 표)이 HWP5 저장 시 "표 번호" 범주로 바뀐다.

## 2. 수정

- `materialize_table_ctrl_header_attr()`: 파서(#2697)와 동일하게
  `table.common.numbering_type == ObjectNumberingType::Table` 일 때만
  `HWPX_TABLE_NUMBERING_BIT` 를 OR.
- 기존 어댑터 테스트 픽스처 2건(`table_axis_materializes_hancom_record_contract`,
  `captioned_table_materializes_hancom_caption_common_attr_bit`)에 실제 파서 기본값인
  `numbering_type: Table` 을 명시 — 기대 상수(`0x082a_2311`/`0x282a_2311`) 유지.

## 3. red → green

`src/document_core/converters/hwpx_to_hwp.rs` tests 추가:

| 테스트 | 수정 전 | 수정 후 |
|---|---|---|
| `picture_numbering_table_keeps_category_on_hwp_save` | FAIL (attr `0x080a2000`) | PASS |
| `none_numbering_table_keeps_category_on_hwp_save` | FAIL | PASS |
| `table_numbering_table_still_sets_numbering_bit` (회귀 가드) | PASS | PASS |

## 4. 검증 결과

- `cargo test --lib` — 2554 passed / 0 failed
- `cargo test --test hwpx_to_hwp_adapter --test hwpx_roundtrip_baseline --test hwp5_roundtrip_baseline` — 전부 통과 (3 / 4 / 50 passed)
- `cargo fmt --check` — CRLF newline-style 경고 외 잔여 없음
- `cargo clippy --all-targets` — 신규 경고 없음 (기존 2건은 johab.rs/byte_writer.rs 무관 항목)
