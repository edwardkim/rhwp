---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5145 검토 — `form-value` 양식 값 조회

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5145](https://github.com/edwardkim/rhwp/pull/5145) / `kevin9327` |
| 원 head | `bc35f4a56d26df51ab5c4fb804d58413da1abb79` (`feat/cli-form-value`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 83 files, +27,136 / -16,382 |
| 로컬 적용 | `4e7044452` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

양식 개체 값 조회와 관련 계약이 범위다. 원 기능 커밋에는 뒤의 #5146·#5147 계약 소스도 함께 있었으며,
메인터너 보정 `3b97d32a2`가 command dispatch와 `text` 대상 양식 선택을 최신 기준으로 교정했다. 74건 focused
계약의 기존 통과·재실행·원격 CI 성공과 승인이 모두 수용 조건이다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.


## 최신 통합 재검증 (2026-08-18)

- GitHub 재확인: [#5145](https://github.com/edwardkim/rhwp/pull/5145)는 OPEN, non-draft, devel 대상이다.
- 최신 기준: upstream/devel efbd8da6a84786dbdad8274c0ced49669e5f3e45 위 통합 검토 브랜치에서 재검증했다.
- 통합 근거: 빌드, fmt, diff, unit-tier, 에이전트 문서 멱등성 및 set_page_hide_contract 4/4 통과. 생성 manifest/harness 드리프트는 CI 생성물로 커밋에서 제외한다.
- 원 통합 PR #5198은 이미 병합되어 닫혔으므로, 이 후속 보정은 새 통합 PR의 CI로 다시 판정한다.
