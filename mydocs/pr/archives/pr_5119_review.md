---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5119 검토 — `edit insert-picture`

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5119](https://github.com/edwardkim/rhwp/pull/5119) / `kevin9327` |
| 원 head | `a0c60512cf1b241cb2fbb0427fb0b615cc5b5f10` (`feat/cli-insert-picture`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 81 files, +21,787 / -11,661 |
| 로컬 적용 | `f63874f7e` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

문단 좌표 그림 삽입 CLI/MCP와 계약이 범위다. 누적 head가 아닌 원 기능을 적용했고 공통 편집 계약군의
기존 74건 통과를 보유한다. 2026-08-18 재검증과 원격 CI가 성공하고 승인될 때까지 push·merge·comment는 하지 않는다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.
