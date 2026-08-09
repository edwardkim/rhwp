---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4304 검토 - Python binding drift 수정

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4304](https://github.com/edwardkim/rhwp/pull/4304) / `kevin9327` |
| 관련 roadmap | #3907 R61 D-2/D-3/D-5/D-6/D-7/D-8 |
| base / source head | `devel` / `9dce5fe4bad01aa7bba06895d88b133ef8fe6aa5` |
| 누적 적용 / 보정 | `036f91ef5`, `d6206e43a` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 8 files, +154 / -16 |

## 판정

**수용 권고, 공개 API 보정 포함.** render-diff, schema `--json`, quoting, fixed-length regex,
`UsageError.next_call` 정합을 Node·CLI 계약과 맞췄다. `TimeoutError`를 곧바로 개명하면 기존
`rhwp.TimeoutError` import/catch가 깨지므로, canonical `RhwpTimeoutError`와 동일한 compatibility
alias를 유지하고 public docs를 전체 API 표면과 맞췄다. Python 251 passed, mypy/ruff가 통과했다.

merge 전 source/checks 재확인이 필요하다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
