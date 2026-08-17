---
kind: pr-review
status: absorbed-upstream
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #4987 검토 — desk HWPX/HWP 변환 도구

| 항목 | 기록 |
| --- | --- |
| PR | [#4987](https://github.com/edwardkim/rhwp/pull/4987) |
| 작성자 / base | `kevin9327` / `devel` |
| 원 head | `97c80fff5b3b761f965f83fb92554f304cc8e2f8` |
| 작성 시점 상태 | OPEN, non-draft, `CONFLICTING` / `DIRTY`; reviewer `jangster77` |
| 규모 | 41 files, +10,282 / -0 |

## 검토와 판단

HWPX·HWP 변환의 ontology·배치·allowlist 연결이 범위다. 최신 기준에 동등 적용 `86f058b51`이 있어
원 체리픽은 중복이다. 후속 검증 경로와 함께 이미 기준선에 존재하므로 원격 처리는 CI와 승인 뒤에
통합 링크를 남기는 방식으로만 진행한다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.


## 최신 통합 재검증 (2026-08-18)

- GitHub 재확인: [#4987](https://github.com/edwardkim/rhwp/pull/4987)는 OPEN, non-draft, devel 대상이다.
- 최신 기준: upstream/devel efbd8da6a84786dbdad8274c0ced49669e5f3e45 위 통합 검토 브랜치에서 재검증했다.
- 통합 근거: 빌드, fmt, diff, unit-tier, 에이전트 문서 멱등성 및 set_page_hide_contract 4/4 통과. 생성 manifest/harness 드리프트는 CI 생성물로 커밋에서 제외한다.
- 원 통합 PR #5198은 이미 병합되어 닫혔으므로, 이 후속 보정은 draft PR [#5201](https://github.com/edwardkim/rhwp/pull/5201)의 CI로 다시 판정한다.
