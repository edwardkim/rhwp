---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4871 검토 - export-llm RAG 청크 출력

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4871](https://github.com/edwardkim/rhwp/pull/4871) · @kevin9327 |
| 원 head | `5b2e27121c08807adb41330d6398a9db26534710` |
| 누적 순서·적용 SHA | 23/27 · `5ef30eff7` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · HWP/HWPX RAG export surface |

HWP/HWPX를 LLM-ready RAG chunk로 내보내는 명령 축을 추가한다. CLI Rust 변경을 포함하므로 전체 nextest와
GitHub code CI로 검증했다. **수용 가능**으로 판정한다.
