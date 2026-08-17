---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #4888 검토 — `rhwp scaffold` 스펙→HWPX 생성기

| 항목 | 기록 |
| --- | --- |
| PR | [#4888](https://github.com/edwardkim/rhwp/pull/4888) |
| 작성자 / base | `kevin9327` / `devel` |
| 원 head | `46a242b3ee32285a61a96c7e18975b98ace274ec` (`gym_scaffold`) |
| 작성 시점 상태 | OPEN, non-draft, `CONFLICTING` / `DIRTY`; reviewer `jangster77` |
| 규모 | 13 files, +1,143 / -6 |

## 검토와 메인터너 보정

원 기능 커밋 `90f7f8300`은 WIP·미완성 표시이며 `scaffold`의 최신 schema registry와 HWPX table
metadata에 충돌했다. 최신 구현을 유지하고, JSON schema·HWPX 직렬화/재파싱·outline·표·빈 문서·버전
가드 회귀 8건을 `d18436aaa`로 이식했다. 서식 보정은 `4d4f0c62f`에 분리했다.

## 검증과 판단

`cargo test --lib scaffold::tests --target-dir target\\pr-review`에서 **8/8 통과**. 원 PR의 conflict는
최신 기준 통합 후보에서 해소됐으나 원격 CI와 작업지시자 승인이 아직 없으므로 원격 변경은 보류한다.

## 최신 통합 검증 (2026-08-18)

[PR #5198 통합 검증](pr_5198_integration_validation.md)에 이 PR을 포함한 누적 후보의 검증 근거를 기록했다. 최신 검토 후보는 로컬 `release-test` 6,798/6,798을 통과했으며, 원격 CI·승인 전에는 원 PR 상태를 바꾸지 않는다.
