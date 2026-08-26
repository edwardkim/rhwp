---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #4887 검토 — `rhwp explore` 문서별 affordance 라우터

| 항목 | 기록 |
| --- | --- |
| PR | [#4887](https://github.com/edwardkim/rhwp/pull/4887) |
| 작성자 / base | `kevin9327` / `devel` |
| 원 head | `997f71f9af05f6d99693a84047111563e77b32fb` (`gym_explore_tool`) |
| 작성 시점 상태 | OPEN, non-draft, `CONFLICTING` / `DIRTY`; reviewer `jangster77` |
| 규모 | 11 files, +799 / -3 |

## 검토와 메인터너 보정

원 기능 커밋 `c54190eb7`은 WIP·검증 미완으로 표시돼 있으며 최신 기준의 `explore` 구현·생성기와
충돌했다. 최신 구현을 보존하고, 원 PR의 menu 우선순위·form/table/security·장문 분기 회귀 8건만
`d31a63689`로 이식했다. 오래된 생성기 목록을 되돌리는 변경은 채택하지 않았다.

## 검증과 판단

`cargo test --lib document_core::queries::explore::tests --target-dir target\\pr-review`에서 **8/8 통과**.
원 PR의 conflict 상태 때문에 직접 merge가 아닌 최신 기준 통합 후보 수용이며, 원격 CI 성공과 승인 전에는
push·comment·merge를 하지 않는다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.


## 최신 통합 재검증 (2026-08-18)

- GitHub 재확인: [#4887](https://github.com/edwardkim/rhwp/pull/4887)는 OPEN, non-draft, devel 대상이다.
- 최신 기준: upstream/devel efbd8da6a84786dbdad8274c0ced49669e5f3e45 위 통합 검토 브랜치에서 재검증했다.
- 통합 근거: 빌드, fmt, diff, unit-tier, 에이전트 문서 멱등성 및 set_page_hide_contract 4/4 통과. 생성 manifest/harness 드리프트는 CI 생성물로 커밋에서 제외한다.
- 원 통합 PR #5198은 이미 병합되어 닫혔으므로, 이 후속 보정은 draft PR [#5201](https://github.com/edwardkim/rhwp/pull/5201)의 CI로 다시 판정한다.
