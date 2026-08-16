---
kind: pr-review
status: code-ci-running
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4909 검토 — 소프트 하이픈 원본 표기 보존

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4909](https://github.com/edwardkim/rhwp/pull/4909), @planet6897 |
| 원 head | `cb77c930d21d69cd8ca0554eb142327e97bb875b` |
| 통합 적용 | `6660f53a7` |
| 기준 | `upstream/devel@ae5f2a345` |

U+00AD가 원문에 있던 경우를 control mask로 보존하고, HWP와 HWPX serializer가 같은 표기를 다시
내보내도록 했다. 일반 공백·줄바꿈 제어와 혼동하지 않으며, HWP/HWPX 대상 소프트 하이픈 회귀 2건과
통합 전체 게이트가 통과했다.

문자 직렬화 계약만 바꾸고 레이아웃·SVG·WASM 산출은 바꾸지 않는다. 통합 PR
[#4936](https://github.com/edwardkim/rhwp/pull/4936)의 최초 코드 후보 CI는 녹색이었다. 최신 devel 동기화 뒤
docs head의 필수 CI와 head 동일성을 다시 확인하면 **수용 가능**이다.
