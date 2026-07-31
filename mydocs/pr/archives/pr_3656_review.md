---
kind: pr-review
status: active
---

# PR #3656 검토 — 실패한 snapshot command의 원자적 rollback

| 항목 | 값 |
| --- | --- |
| 작성자 / reviewer | `@lpaiu-cs` / `@jangster77` |
| 원 PR / 관련 이슈 | [#3656](https://github.com/edwardkim/rhwp/pull/3656) / [#3350](https://github.com/edwardkim/rhwp/issues/3350) |
| 원 head 참고값 | `9daeb0bb81ef9a7c2e3174a6ee8ab916c759a940` |
| 통합 후보 | [#3661](https://github.com/edwardkim/rhwp/pull/3661) `52903c91bf132f7f3a977afc9cc265859b024c85` |
| 원 변경 규모 | 2 files, +35 / -3 |
| 권고 | #3661로 수용. merge 시 #3350 close |

## 변경과 통합 판정

`SnapshotCommand.execute()`는 최초 실행에서 before snapshot을 저장한 뒤 operation과 after snapshot을
수행한다. 그 사이에 throw하면 command는 history에 들어가지 않아 snapshot discard 주체가 사라지고,
operation이 문서를 일부 바꾼 경우 원자성도 깨졌다.

PR은 operation과 after-save 전체를 `try`로 감싸고 실패 시 before snapshot으로 복원한 뒤 before/after
id를 discard한다. rollback도 실패하면 원 operation error와 rollback error를 `AggregateError`로 함께
전달해 원인을 지우지 않는다. 성공·no-op·redo 경로는 이전 계약을 그대로 둔다.

회귀는 현재 WASM bridge가 failure injection을 제공하지 않는 한계를 숨기지 않고, execute body에
before restore, null-safe discard, aggregate error, after-save 포함 try 범위가 존재하는 source contract로
고정한다. 기능 commit `9daeb0bb8`은 통합에서 `f0edd0889`와 patch 동등하다.

## 검증과 권고

| 검증 | 결과 |
| --- | --- |
| source #3656 CI | full CI, CodeQL, Canvas visual diff success |
| 통합 code head CI | lint·WASM check, frontend package gates, Native Skia, archive, default-feature 8 shards, CodeQL, Canvas visual diff, `Build & Test` 모두 success |
| 체리픽 동등성 | `9daeb0bb8` = `f0edd0889` patch-id, `git diff --check` 통과 |
| 추가 로컬 Cargo | exact integration CI와 중복되므로 작업지시에 따라 실행하지 않음. 성공 근거로 사용하지 않음 |

**권고: 수용.** history 등록 전 실패의 rollback 책임을 유일하게 이 command가 지는 경계를 정확히
복구한다. #3661의 current-head full CI를 확인했고, review-only fast-pass 후 merge하여 #3350 close 상태를 확인한다.
