---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4930 검토 - DAR COMMIT 전 정책 게이트

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4930](https://github.com/edwardkim/rhwp/pull/4930) |
| 작성자 / source | @kevin9327 / `feat/dar-policy-engine-l3` |
| 원 source head | `632aa77da0b837ca608c7ca942465b9c86251a11` |
| 기준 devel | `d9f04c6eec1f` |
| 가시성 검토 branch | `review/kevin9327-20260816` |
| local 적용 commit | `c2fb8c0b8` |
| 메인터너 보정 | `2d17db4c2` |
| 원 PR 상태 참고값 | 작성 시점 `OPEN` / `MERGEABLE`; merge 직전에 재확인 필요 |

DATP 참조 드라이버의 COMMIT 전 단계에 정책 해석과 차단 게이트를 추가한다.

## 메인터너 보정

정책 JSON이 배열이 아니거나 `rules`가 없거나 배열이 아니면, 기존 구현은 `AttributeError`를 내거나
기본값으로 진행할 여지가 있었다. `2d17db4c2`은 정책 루트·`rules`·각 rule의 object 형식을 명시적으로
검사하고 모두 `ValueError`로 거부하도록 고정했다. malformed 정책은 허용보다 거부가 안전한 경계다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| DAR 계약 | `python3 -m unittest scripts.tests.test_automation_tool_contracts` | 33 passed (정책 비정형 입력 거부 회귀 포함) |
| 자체 검사 | `python3 tools/dar/conformance.py --self-check` | 문제 0건 |
| 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,479 passed, 38 skipped |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` | 통과 |

정책 검증 도구와 문서만 바꾸므로 renderer 시각 대조는 적용하지 않았다.

## 판단

정책 경계가 예외 형태와 fail-closed 동작까지 명시됐다. **메인터너 보정을 포함해 통합 수용 권고.**

구현·적용 순서는 [메인터너 보정 기록](pr_4930_review_impl.md)에 남긴다.
