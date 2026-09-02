---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5092 검토 - ml-kem 0.3.2

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5092](https://github.com/edwardkim/rhwp/pull/5092) |
| 작성자 / source | `app/dependabot` / `dependabot/cargo/devel/ml-kem-0.3.2` |
| 원 source head | `f06aa1a2b2aa7213c0feb4bc167a7842d8870280` |
| 기준 / 규모 | `devel`, 2 files, +32 / -41 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `BLOCKED` (원 PR CI 참고값) |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

`ml-kem` 0.2.3→0.3.2의 key/KEM API 갱신이다.

## 통합 적용과 검증

원 SHA를 `25e8b73188bfdb7e13b38c2365be5709c4726e6e`로 적용하고, 0.3의 `Kem`·키 import 및 fallible RNG 계약으로
`src/security_trailer.rs`를 이행했다. 기존 외부 형식과의 호환을 위해 2,400바이트 확장 비밀키 직렬화는 유지했다.

- `cargo test --lib security_trailer --features gpu` 19건과 GPU feature check·clippy를 통과했다.
- full release-test nextest는 6,522 passed, 38 skipped였다.
- #5186 code candidate의 Lint, Build & Test, CodeQL이 성공했다.

## 판단

키 직렬화 호환성과 암복호 경로를 검증했다. 원 PR의 `BLOCKED`는 통합 후보의 최신 CI를 대체하지 않는 참고값이다.
**통합 수용 권고.**
