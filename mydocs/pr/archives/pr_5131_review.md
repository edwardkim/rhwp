---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5131 검토 — `edit set-picture`

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5131](https://github.com/edwardkim/rhwp/pull/5131) / `kevin9327` |
| 원 head | `0be83d03641670e4db6d8f812024997db21490f5` (`feat/cli-set-picture`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 83 files, +23,633 / -12,812 |
| 로컬 적용 | `ff01e2fca` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

본문 그림 속성 설정 CLI/MCP와 계약이 범위다. source 누적 head를 통째로 적용하지 않고 기능 커밋만
수용했으며, 최신 command 선언·dispatch는 메인터너 보정에서 확인했다. 편집 계약 재실행과 원격 CI 성공,
작업지시자 승인 전에는 원격 push·merge·comment를 하지 않는다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.
