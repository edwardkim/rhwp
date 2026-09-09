# Task M100-258 Stage 16 — PR 로컬 검증 중 셀 필드 char_count 테스트 정정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `28694a98` (`task 258: 누름틀 삭제 후 커서 복귀`, rebase 후)

## 1. 문제

`upstream/devel` 동기화 후 PR 준비 로컬 검증에서 `cargo test --release --lib`가 실패했다.

- 실패 테스트: `document_core::queries::field_query::tests::set_cell_field_text_updates_text_metadata`
- 실패 내용: `"새값"` 2글자 교체 후 `updated.char_count`가 `3`인데 테스트는 `2`를 기대했다.

## 2. 판단

프로젝트 전반에서 `Paragraph.char_count`는 텍스트 UTF-16 길이에 문단 끝 마커를 더한 값으로
사용된다. 예: `text_len + 1`, 빈 셀 `char_count=1`.

따라서 `"새값"` 2글자의 정상 `char_count`는 `3`이며, 기존 테스트 기대값이 문단 끝 마커를
반영하지 못한 상태다.

## 3. 수정 방향

- 테스트 fixture의 초기 `char_count`도 `"기존값"` 3글자 + 끝 마커 기준인 `4`로 정정한다.
- 교체 후 기대값은 `"새값"` 2글자 + 끝 마커 기준인 `3`으로 정정한다.

## 4. 검증 계획

- `cargo test --release --lib set_cell_field_text_updates_text_metadata -- --exact`
- `cargo test --release --lib`
- PR 준비 로컬 검증 계속 진행

## 5. 검증 결과

- `cargo test --release --lib set_cell_field_text_updates_text_metadata -- --exact`:
  필터가 전체 모듈 경로와 맞지 않아 0개 실행됨
- `cargo test --release --lib set_cell_field_text_updates_text_metadata`: 통과
- `cargo test --release --lib`: 통과
