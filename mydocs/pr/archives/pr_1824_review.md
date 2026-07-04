# PR #1824 리뷰 — 셀 field_name raw_list_extra 직렬화

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1824 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1808` |
| 작성 시점 참고 head | `cd9121f9317ac0993bf4f7738f23300fc04c657e` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- HWP cell `field_name` 을 한컴 raw_list_extra 계약에 맞춰 직렬화한다.
- `src/parser/control.rs`, `src/serializer/control.rs`, `src/serializer/control/tests.rs` 에 parser/serializer/test 변경이 들어간다.
- PR review 중 parser 주석의 offset 설명이 구현과 달라 메인터너 보정 커밋을 추가했다.

## 로컬 검증

누적 검토 브랜치: `local/batch-pr1823-1840-review`

- 체리픽 커밋: `cd9121f9317` -> `93d480db7`
- 메인터너 보정: `239bdf73c` (`src/parser/control.rs` offset 주석 15/17 로 정리)
- 충돌: `mydocs/orders/20260702.md` 행 충돌 1건. #1807/#1808 행을 모두 보존해 해결.
- focused 검증: `env CARGO_INCREMENTAL=0 cargo test test_cell_field_name_extra_roundtrip --lib` 통과.
- 누적 검증: `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings` 통과.

## 판단

테스트가 raw_list_extra 왕복 계약을 고정하고 있고, 보정은 코드 동작이 아니라 stale comment 수정이다. serializer
변경이지만 PR 목적과 테스트 범위가 일치한다.

## 결론

merge 후보. GitHub merge 전에는 최신 head 와 required checks 를 다시 확인한다.
