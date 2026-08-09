---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4307 검토 - Python public API parity

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4307](https://github.com/edwardkim/rhwp/pull/4307) / `kevin9327` |
| 관련 roadmap | #3907 R61 D-10/D-12/D-13/D-14/D-15/D-18 |
| base / source head | `devel` / `f854aec0a9b090fc55d4c0139573e9ba12010a15` |
| 누적 적용 | `1f3b5f632` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 14 files, +276 / -18 |

## 판정

**수용 권고.** `verifyPages`, query option, `cwd`, binary constants, documented process API와 exit
helpers를 Python package root에 정합하게 노출한다. #4304와 겹친 tests는 두 PR의 regression을
모두 보존해 해결했다. `Session.timeout`은 구현되지 않은 timeout으로 오해하지 않도록 문서에서
저장 compatibility option과 실행 경계의 한계를 명확히 했다. Python 251 passed, mypy/ruff가 통과했다.

실제 merge 전 latest remote checks를 재확인한다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
