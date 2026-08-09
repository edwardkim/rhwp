---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4308 검토 - Node binding error/session parity

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4308](https://github.com/edwardkim/rhwp/pull/4308) / `kevin9327` |
| 관련 roadmap | #3907 R61 D-14/D-16/D-17/D-20 |
| base / source head | `devel` / `07dce5949b9ddcbb1e45209e53f5c509b5166466` |
| 누적 적용 / 보정 | `4e0e960a4`, `5f3e9eaf0` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 10 files, +177 / -7 |

## 판정

**수용 권고, timeout semantics 보정 포함.** `RHWP_BIN=~/...` 확장, `EnvelopeKeyError`,
`runRaw.envelopeHint`는 Python과 정합하다. 단 initial timeout 구현은 caller wait만 reject하고
server child를 계속 실행해 edit가 뒤늦게 적용될 수 있었다. 메인터너 보정은 timeout 때 child를
종료하고 session을 closed로 만들어 retry 전에 새 Session을 강제한다. Node 427 passed,
typecheck/build가 통과했다.

merge 전 source/checks 재확인이 필요하다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
