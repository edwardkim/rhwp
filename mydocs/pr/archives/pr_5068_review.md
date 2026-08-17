---
kind: pr-review
status: integration-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5068 검토 — `edit insert-equation`

| 항목 | 기록 |
| --- | --- |
| PR / 작성자 | [#5068](https://github.com/edwardkim/rhwp/pull/5068) / `kevin9327` |
| 원 head | `2839563964e0b6ad3e32152235cdf23c08538111` (`feat/cli-insert-equation-fastb`) |
| 상태 / 규모 | OPEN, `CONFLICTING` / `DIRTY`, 69 files, +19,307 / -11,659 |
| 로컬 적용 | `638b637c8` — 최신 `upstream/devel` 기준 기능 커밋만 적용 |

본문 수식 삽입 CLI/MCP와 계약 테스트가 범위다. 누적 branch의 전체 파일 교체를 피하고 기능 커밋만
적용했다. 공통 편집 계약 74건의 기존 통과와 2026-08-18 재검증·원격 CI 성공을 수용 조건으로 하며,
작업지시자 승인 전 push·merge·comment는 보류한다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.


## 최신 통합 재검증 (2026-08-18)

- GitHub 재확인: [#5068](https://github.com/edwardkim/rhwp/pull/5068)는 OPEN, non-draft, devel 대상이다.
- 최신 기준: upstream/devel efbd8da6a84786dbdad8274c0ced49669e5f3e45 위 통합 검토 브랜치에서 재검증했다.
- 통합 근거: 빌드, fmt, diff, unit-tier, 에이전트 문서 멱등성 및 set_page_hide_contract 4/4 통과. 생성 manifest/harness 드리프트는 CI 생성물로 커밋에서 제외한다.
- 원 통합 PR #5198은 이미 병합되어 닫혔으므로, 이 후속 보정은 새 통합 PR의 CI로 다시 판정한다.
