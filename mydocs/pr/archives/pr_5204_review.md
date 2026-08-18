---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5204 검토 - 외부 LINK 그림의 bindata 식별자 분리

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5204](https://github.com/edwardkim/rhwp/pull/5204) |
| 작성자 | @planet6897 |
| 원 source head | `66cc892059470bc06a41137abca355a27454331d` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

HWPX 외부 연결(`LINK`) 그림이 내장 그림과 동일한 ID를 재사용해 충돌·소실되는 경로를
분리하고, `link_pictures_do_not_alias_embedded_images` 회귀 계약을 적용했다. 내장 리소스와
외부 연결 리소스의 식별자가 독립적으로 유지되는 것을 전체 회귀에서 확인했다.

차단 결함은 발견하지 못했다. 원격 source branch는 수정하지 않았다.

## 검증

- 관련 회귀 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
