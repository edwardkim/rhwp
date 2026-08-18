---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5323 검토 - HWPX paraHead 속성 역산

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5323](https://github.com/edwardkim/rhwp/pull/5323) |
| 작성자 | @planet6897 |
| 원 source head | `5fbf4aeac8a8c0f7f956f358b47adc32a933144e` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

문단 번호의 정렬, 번호 너비 사용 여부, 자동 내어쓰기를 HWP5 `attr` bits에서 유도해
HWP5→HWPX 저장 왕복에서 `paraHead` 속성이 사라지지 않도록 했다. 전용 numbering
회귀와 전체 저장 왕복 테스트를 확인했다.

차단 결함은 발견하지 못했다. 원격 source branch는 수정하지 않았다.

## 검증

- 관련 회귀 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
