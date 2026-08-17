---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5157 검토 — `edit ungroup-shape`

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5157](https://github.com/edwardkim/rhwp/pull/5157) / `kevin9327` |
| 원 head | `e7855f3bc718ac66add83bb288bd53bcaf7a5ef0` (`feat/cli-ungroup-shape-next`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 85 files, +25,445 / -14,002 |
| 로컬 적용 | `4c5cbbf21` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

도형 묶음 풀기 CLI/MCP 및 계약 검증이 범위다. source 누적 head의 다른 편집 축은 재적용하지 않았고
기능 커밋만 최신 기준 후보에 반영했다. 74건 focused 계약의 재실행·후보 CI 성공·작업지시자 승인 후에만
원격 merge와 원 PR 후속 처리를 진행한다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.


## 최신 통합 재검증 (2026-08-18)

- GitHub 재확인: [#5157](https://github.com/edwardkim/rhwp/pull/5157)는 OPEN, non-draft, devel 대상이다.
- 최신 기준: upstream/devel efbd8da6a84786dbdad8274c0ced49669e5f3e45 위 통합 검토 브랜치에서 재검증했다.
- 통합 근거: 빌드, fmt, diff, unit-tier, 에이전트 문서 멱등성 및 set_page_hide_contract 4/4 통과. 생성 manifest/harness 드리프트는 CI 생성물로 커밋에서 제외한다.
- 원 통합 PR #5198은 이미 병합되어 닫혔으므로, 이 후속 보정은 새 통합 PR의 CI로 다시 판정한다.
