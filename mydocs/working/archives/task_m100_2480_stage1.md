# 작업 2480 단계 1 - Undo split FieldRange 통합

## 범위

- 현재 메인터너 브랜치에 기여자 PR #2480을 통합한다.
- undo가 문단을 다시 분할할 때 병합됐던 두 번째 문단의 ClickHere field를 보존한다.

## 검토 근거

- PR 본문: 기존 `split_at`은 `Control::Field`를 새 문단으로 옮기지 않은 채 `field_ranges`를
  유지하거나 버렸다.
- PR 코멘트: 없음.
- 기존 `test_split_and_merge_roundtrip`에는 field control이 없어 보고된 회귀를 증명할 수 없다.

## 메인터너 보강

- ClickHere field가 있는 문단을 앞 문단에 병합한 뒤 병합 경계에서 다시 분할한다.
- 복원된 문단이 Field control, remap된 `FieldRange`, 일치하는 `CTRL_DATA`, field-range control
  mask를 소유하는지 확인한다.

## 검증 계획

1. focused paragraph model 테스트를 실행한다.
2. `cargo fmt --all --check`와 `cargo clippy --all-targets -- -D warnings`를 실행한다.
3. 최종 통합 PR 전 통합 전체 회귀에 이 그룹을 포함한다.
