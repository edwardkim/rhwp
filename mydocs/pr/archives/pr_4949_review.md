---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4949 검토 - agent capability handoff

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4949](https://github.com/edwardkim/rhwp/pull/4949) |
| 작성자 / source | @kevin9327 / `feat/agent-capability-handoff` |
| 원 source head | `021e42d75f283e5dd63b3a197f86e8eda50ec567` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `320d6d41d` |
| 메인터너 보정 | `566194be7` |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

외부 agent의 task 결과를 sandbox `out/` 경계, 정책, 저널, 재시도와 함께 수거하는 handoff orchestration을
추가한다. 신뢰하지 않는 결과는 `mustParse` 재개봉 및 경로·도구 선언 검증을 거쳐야만 수용된다.

## 메인터너 보정

`566194be7`은 결과가 주장한 SHA-256을 수용 계약으로 승격했다. 각 output은 64자리 hex `sha256`을 반드시
선언해야 하며, 실제 수거 파일 해시와 불일치하면 수용되지 않는다. 선언 누락과 위조 해시를 모두 재현하는
회귀를 추가해 외부 반환물의 무결성 검증 경계를 닫았다.

## 검증과 판단

| 범위 | 근거 | 결과 |
| --- | --- | --- |
| handoff 계약 | `python3 -m unittest scripts/tests/test_agent_handoff.py` | 28 passed |
| Python 문법 | `python3 -m py_compile tools/handoff/orchestrator.py scripts/tests/test_agent_handoff.py` | 통과 |
| 누적 Rust 회귀 | release-test nextest 전체 | 6,519 passed, 38 skipped |
| 품질 | fmt, clippy, diff 검사 | 통과 |

메인터너 보정은 contributor 변경의 위임·수거 모델을 넓히지 않고, 이미 선언된 산출 해시를 실제 검증으로
연결한다. **메인터너 보정을 포함해 통합 수용 권고.**
