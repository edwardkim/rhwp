---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5097 검토 - chacha20poly1305 0.11.0

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5097](https://github.com/edwardkim/rhwp/pull/5097) |
| 작성자 / source | `app/dependabot` / `dependabot/cargo/devel/chacha20poly1305-0.11.0` |
| 원 source head | `49f4507220025c22e1e4640b7e98cffe3a2c2a34` |
| 기준 / 규모 | `devel`, 2 files, +25 / -52 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `BLOCKED` (원 PR CI 참고값) |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

AEAD 의존성을 0.10.1에서 0.11.0으로 갱신한다.

## 통합 적용과 검증

원 SHA를 `fb7c9b0ef288715aafed83a0015b382cc337230a`로 적용했다. 0.11의 `Key::try_from`과 `XNonce::from` API에 맞춰
`security_trailer.rs`, `agent_seal.rs`의 길이 검증된 key/nonce 변환을 이행했다. security trailer 19건, full
release-test nextest(6,522 passed, 38 skipped), GPU clippy와 #5186 CI·CodeQL이 성공했다.

## 판단

암호 key/nonce 변환은 실패 가능한 길이 검사를 보존한다. 원 PR 대기는 통합 candidate CI로 재검증했으며
**통합 수용 권고**한다.
