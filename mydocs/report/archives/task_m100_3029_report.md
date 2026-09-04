# 완료 보고서 — Task M100-3029

- 이슈: #3029
- 제목: 문단 병합(merge_from) 시 컨트롤 없는 tab/개행 텍스트의 control_mask 비트 누락
- 작성일: 2026-07-22
- 브랜치: `task/m100-2905b-paragraph-split-merge`

## 1. 완료 내용

`Paragraph::merge_from`이 `control_mask`를 갱신할 때 `other.controls`가 비어 있지 않은
경우에만 `self.control_mask |= other.control_mask`를 수행하던 것을 수정했다.
`control_mask`는 `controls` 목록뿐 아니라 텍스트의 tab(`\t`)/개행(`\n`) 존재 여부,
`field_ranges` 존재 여부에서도 파생되는 비트를 포함하므로, 병합 대상 문단이
컨트롤 없이 tab/개행만 가진 경우 해당 비트가 누락되고 있었다. `split_at`이 이미
사용하는 `compute_control_mask_for(text, controls, field_ranges)` 전체 재계산 방식을
`merge_from`에도 동일하게 적용해 컨트롤 유무와 무관하게 항상 정확한 마스크를
갱신하도록 했다.

## 2. 주요 변경

- `src/model/paragraph.rs`
  - `merge_from` 끝부분에서 조건부 `control_mask |= other.control_mask` 대신
    `self.control_mask = Self::compute_control_mask_for(&self.text, &self.controls, &self.field_ranges)`로
    전체 재계산 (split_at과 동일 패턴)
- `src/model/paragraph/tests.rs`
  - `test_merge_from_text_only_control_mask_bits` 추가: `other.controls`가 비어 있고
    `other.text`에 tab만 있는 경우에도 병합 후 `control_mask`에 tab 비트(0x9)가
    반영되는지 검증 (수정 전에는 실패 확인)

## 3. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib paragraph::tests` (54 passed)
- `rustfmt --edition 2021` (src/model/paragraph.rs, src/model/paragraph/tests.rs)

수정 전 동일 테스트로 red 상태 재현 확인 (`left: 0, right: 0` 어설션 실패) 후
수정을 되돌려 green 전환을 확인했다.

## 4. 범위 밖 항목

- 표(table) 셀 병합/분할의 `char_offsets`는 셀이 서로 독립된 텍스트 스트림이라
  concat 자체가 발생하지 않음을 확인했다 (#2905, #2912 패턴과 일치, 정상).
- `shape.rs`/`table.rs`/`picture.rs`/`note.rs`의 insert/delete `char_offsets` shift는
  threshold 기반 shift 패턴을 따르며 이번 조사 범위에서 이상 없음을 확인했다.
