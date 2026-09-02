---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5093 검토 - rand_core 0.10.1

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5093](https://github.com/edwardkim/rhwp/pull/5093) |
| 작성자 / source | `app/dependabot` / `dependabot/cargo/devel/rand_core-0.10.1` |
| 원 source head | `375be82a24d8d4782829eaa36f9c7cfc1700606a` |
| 기준 / 규모 | `devel`, 2 files, +12 / -6 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `BLOCKED` (원 PR CI 참고값) |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

`rand_core` 0.6.4→0.10.1 갱신이다.

## 통합 적용과 검증

원 SHA를 `6780dc815f5f37a7caaf5616fa863d5f2178431f`로 적용했다. `GetRandomRng`를 `TryRng<Error = Infallible>`과
`TryCryptoRng` 계약으로 옮겨 `ml-kem` 0.3 호출과 정합시켰다. security trailer 19건, GPU feature check·clippy,
full release-test nextest(6,522 passed, 38 skipped)와 #5186의 최신 CI·CodeQL이 성공했다.

## 판단

난수 adapter의 error/crypto marker 계약을 검증했다. 원 PR의 대기 상태는 통합 candidate의 녹색 CI로 재검증했다.
**통합 수용 권고.**
