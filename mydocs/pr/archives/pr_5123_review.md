---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5123 검토 — `edit insert-shape`

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5123](https://github.com/edwardkim/rhwp/pull/5123) / `kevin9327` |
| 원 head | `8d92686659dcacf4e7a83d6d476c44cf839a6c97` (`feat/cli-insert-shape`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 81 files, +21,624 / -11,598 |
| 로컬 적용 | `57b8c895b` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

본문 도형 삽입 CLI/MCP가 범위다. source 누적 branch의 전체 diff는 적용하지 않고 기능 커밋만 반영했다.
메인터너 보정은 CLI 표면과 MCP 노출을 최신 기준에 맞췄다. 재검증과 후보 CI 성공·승인 전에는 원격 변경을 보류한다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.
